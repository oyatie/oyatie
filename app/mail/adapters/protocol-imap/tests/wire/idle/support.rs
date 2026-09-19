use super::*;
use mail_api::{
    AccountInfo, BlobStore, Consumer, Execution, HistoryPage, MailboxSelection, MessageSelection,
    Precondition,
};
use mail_kernel::BlobRef;

pub(super) struct ObservedStore {
    pub(super) inner: Arc<SqliteStore>,
    io_thread: std::thread::ThreadId,
    pub(super) reads: AtomicUsize,
    pub(super) slow: AtomicBool,
    pub(super) entered: AtomicBool,
}
impl ObservedStore {
    fn observe(&self) {
        assert_ne!(
            std::thread::current().id(),
            self.io_thread,
            "IDLE store calls must run off the socket runtime"
        );
        self.reads.fetch_add(1, Ordering::SeqCst);
        if self.slow.load(Ordering::SeqCst) {
            self.entered.store(true, Ordering::SeqCst);
            std::thread::sleep(Duration::from_millis(500));
        }
    }
}
#[rustfmt::skip]
impl MetadataStore for ObservedStore {
    fn account_info(&self, id: &str) -> Result<AccountInfo, Error> { self.inner.account_info(id) }
    fn account(&self, id: &str) -> Result<Account, Error> {
        self.observe();
        self.inner.account(id)
    }
    fn messages(&self, a: &str, ids: &[String]) -> Result<MessageSelection, Error> { self.observe(); self.inner.messages(a, ids) }
    fn mailbox_uids(&self, a: &str, m: &str) -> Result<MailboxSelection, Error> { self.observe(); self.inner.mailbox_uids(a, m) }
    fn resolve(&self, a: &str) -> Result<String, Error> { self.inner.resolve(a) }
    fn execute(&self, a: &str, p: Precondition, c: Vec<Command>) -> Result<Execution, Error> { self.inner.execute(a, p, c) }
    fn deliver_once(&self, a: &str, k: &str, r: &[u8], t: i64) -> Result<(), Error> { self.inner.deliver_once(a, k, r, t) }
    fn put_blob(&self, a: &str, r: &[u8]) -> Result<String, Error> { self.inner.put_blob(a, r) }
    fn blob(&self, a: &str, id: &str) -> Result<Vec<u8>, Error> { self.inner.blob(a, id) }
    fn history(&self, a: &str, s: u64, l: usize) -> Result<HistoryPage, Error> { self.observe(); self.inner.history(a, s, l) }
    fn compact_history(&self, a: &str, n: i64, p: mail_kernel::RetentionPolicy, c: &[(Consumer, u64)]) -> Result<mail_kernel::Retention, Error> { self.inner.compact_history(a, n, p, c) }
}

impl BlobStore for ObservedStore {
    fn persist(&self, a: &str, s: &str, r: &[u8], ttl: i64) -> Result<BlobRef, Error> {
        self.inner.persist(a, s, r, ttl)
    }
    fn renew(&self, a: &str, s: &str, ttl: i64) -> Result<(), Error> {
        self.inner.renew(a, s, ttl)
    }
    fn read(&self, a: &str, b: &BlobRef) -> Result<Vec<u8>, Error> {
        self.inner.read(a, b)
    }
    fn orphan_sweep(&self, now: i64, limit: usize) -> Result<usize, Error> {
        self.inner.orphan_sweep(now, limit)
    }
}

pub(super) fn observed() -> (Arc<MailService>, Arc<ObservedStore>) {
    let (_, db) = super::service();
    let store = Arc::new(ObservedStore {
        inner: db.clone(),
        io_thread: std::thread::current().id(),
        reads: AtomicUsize::new(0),
        slow: AtomicBool::new(false),
        entered: AtomicBool::new(false),
    });
    (
        Arc::new(MailService {
            outbound: None,
            queue: db.clone(),
            store: store.clone(),
            identity: db,
            policy: Arc::new(OwnerPolicy),
        }),
        store,
    )
}

pub(super) async fn wait_for(predicate: impl Fn() -> bool) {
    tokio::time::timeout(Duration::from_secs(5), async {
        while !predicate() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
}

pub(super) async fn idle(client: &mut Client) {
    client.get_mut().write_all(b"i IDLE\r\n").await.unwrap();
    loop {
        let mut line = String::new();
        tokio::time::timeout(Duration::from_secs(5), client.read_line(&mut line))
            .await
            .unwrap()
            .unwrap();
        assert!(
            !line.is_empty() && !line.starts_with("i "),
            "expected IDLE continuation: {line}"
        );
        if line.starts_with('+') {
            return;
        }
    }
}

impl mail_api::SubmissionStore for ObservedStore {
    fn submissions(
        &self,
        a: &str,
        ids: Option<&[String]>,
    ) -> Result<mail_api::SubmissionSelection, Error> {
        self.inner.submissions(a, ids)
    }
    fn accept_submission(
        &self,
        a: &str,
        r: u64,
        v: mail_api::SubmissionAcceptance,
    ) -> Result<mail_api::SubmissionSelection, Error> {
        self.inner.accept_submission(a, r, v)
    }
    fn cancel_submission(
        &self,
        a: &str,
        r: u64,
        id: &str,
    ) -> Result<mail_api::SubmissionSelection, mail_api::SubmissionFailure> {
        self.inner.cancel_submission(a, r, id)
    }
    fn destroy_submission(&self, a: &str, r: u64, id: &str) -> Result<u64, Error> {
        self.inner.destroy_submission(a, r, id)
    }
    fn query_submissions(
        &self,
        a: &str,
        r: Option<u64>,
        q: &mail_kernel::SubmissionQuery,
    ) -> Result<mail_api::SubmissionPage, mail_api::SubmissionFailure> {
        self.inner.query_submissions(a, r, q)
    }
    fn submission_changes(
        &self,
        a: &str,
        r: u64,
        l: usize,
    ) -> Result<mail_api::SubmissionChanges, Error> {
        self.inner.submission_changes(a, r, l)
    }
}
