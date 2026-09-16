use super::*;

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
impl Store for ObservedStore {
    fn account_info(&self, id: &str) -> Result<AccountInfo, Error> { self.inner.account_info(id) }
    fn account(&self, id: &str) -> Result<Account, Error> {
        self.observe();
        self.inner.account(id)
    }
    fn messages(&self, a: &str, ids: &[String]) -> Result<MessageSelection, Error> { self.observe(); self.inner.messages(a, ids) }
    fn resolve(&self, a: &str) -> Result<String, Error> { self.inner.resolve(a) }
    fn execute(&self, a: &str, r: u64, c: Vec<Command>) -> Result<Account, Error> { self.inner.execute(a, r, c) }
    fn deliver_once(&self, a: &str, k: &str, r: &[u8], t: i64) -> Result<(), Error> { self.inner.deliver_once(a, k, r, t) }
    fn put_blob(&self, a: &str, r: &[u8]) -> Result<String, Error> { self.inner.put_blob(a, r) }
    fn blob(&self, a: &str, id: &str) -> Result<Vec<u8>, Error> { self.inner.blob(a, id) }
    fn message_changes(&self, a: &str, s: u64, u: u64) -> Result<Vec<MessageChange>, Error> { self.inner.message_changes(a, s, u) }
    fn message_changes_after(&self, account: &str, since: u64, until: u64) -> Result<Vec<MessageChange>, Error> { self.inner.message_changes_after(account, since, until) }
    fn mailbox_changes(&self, a: &str, s: u64, u: u64) -> Result<Vec<MailboxChange>, Error> { self.inner.mailbox_changes(a, s, u) }
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
