use axum::{Router, body::Body, http::Request};
use mail_api::{
    AccountInfo, Identity, MailboxChange, MessageChange, MessageSelection, Principal, Store,
};
use mail_kernel::{Account, Command, Error};
use mail_service::{MailService, OwnerPolicy};
use mail_sqlite_store::SqliteStore;
use std::sync::{
    Arc, Condvar, Mutex,
    atomic::{AtomicUsize, Ordering},
};
use tokio::sync::Notify;

pub const TOKEN: &str = "0123456789abcdef0123456789abcdef";

#[derive(Clone, Copy, PartialEq)]
pub enum Site {
    Authentication,
    AccountInfo,
    Upload,
    Download,
}

pub struct Gate {
    pub site: Site,
    pub count: AtomicUsize,
    entered: Notify,
    released: Mutex<bool>,
    release: Condvar,
}

impl Gate {
    fn block(&self, site: Site) {
        if self.site != site {
            return;
        }
        self.count.fetch_add(1, Ordering::SeqCst);
        self.entered.notify_one();
        let mut released = self.released.lock().unwrap();
        while !*released {
            let (next, timeout) = self
                .release
                .wait_timeout(released, std::time::Duration::from_secs(5))
                .unwrap();
            released = next;
            if timeout.timed_out() && !*released {
                drop(released);
                panic!("storage barrier stalled the runtime or its test observer");
            }
        }
    }
    pub async fn entered(&self, count: usize) {
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loop {
                let wake = self.entered.notified();
                if self.count.load(Ordering::SeqCst) >= count {
                    break;
                }
                wake.await;
            }
        })
        .await
        .expect("blocking operation entered");
    }
    pub fn open(&self) {
        *self.released.lock().unwrap() = true;
        self.release.notify_all();
    }
}

struct Adapter {
    db: Arc<SqliteStore>,
    gate: Arc<Gate>,
}
impl Identity for Adapter {
    fn authenticate(&self, token: &str) -> Result<Principal, Error> {
        self.gate.block(Site::Authentication);
        if token == "panic" {
            panic!("injected identity worker panic");
        }
        self.db.authenticate(token)
    }
}
impl Store for Adapter {
    fn account_info(&self, id: &str) -> Result<AccountInfo, Error> {
        self.gate.block(Site::AccountInfo);
        self.db.account_info(id)
    }
    fn account(&self, id: &str) -> Result<Account, Error> {
        self.db.account(id)
    }
    fn messages(&self, account: &str, ids: &[String]) -> Result<MessageSelection, Error> {
        self.db.messages(account, ids)
    }
    fn resolve(&self, address: &str) -> Result<String, Error> {
        self.db.resolve(address)
    }
    fn execute(&self, id: &str, revision: u64, commands: Vec<Command>) -> Result<Account, Error> {
        self.db.execute(id, revision, commands)
    }
    fn deliver_once(
        &self,
        account: &str,
        key: &str,
        raw: &[u8],
        received_at: i64,
    ) -> Result<(), Error> {
        self.db.deliver_once(account, key, raw, received_at)
    }
    fn put_blob(&self, account: &str, raw: &[u8]) -> Result<String, Error> {
        self.gate.block(Site::Upload);
        self.db.put_blob(account, raw)
    }
    fn blob(&self, account: &str, id: &str) -> Result<Vec<u8>, Error> {
        self.gate.block(Site::Download);
        self.db.blob(account, id)
    }
    fn message_changes(
        &self,
        account: &str,
        since: u64,
        until: u64,
    ) -> Result<Vec<MessageChange>, Error> {
        self.db.message_changes(account, since, until)
    }
    fn message_changes_after(
        &self,
        account: &str,
        since: u64,
        until: u64,
    ) -> Result<Vec<MessageChange>, Error> {
        self.db.message_changes_after(account, since, until)
    }
    fn mailbox_changes(
        &self,
        account: &str,
        since: u64,
        until: u64,
    ) -> Result<Vec<MailboxChange>, Error> {
        self.db.mailbox_changes(account, since, until)
    }
}

pub struct Fixture {
    pub app: Router,
    pub gate: Arc<Gate>,
    pub blob: String,
}
impl Fixture {
    pub fn new(site: Site) -> Self {
        let db = Arc::new(SqliteStore::open(":memory:").unwrap());
        db.provision(
            Account::new("a", "t", "alice", "alice@example.org").unwrap(),
            TOKEN,
        )
        .unwrap();
        let blob = db.put_blob("a", b"independent-review").unwrap();
        let gate = Arc::new(Gate {
            site,
            count: AtomicUsize::new(0),
            entered: Notify::new(),
            released: Mutex::new(false),
            release: Condvar::new(),
        });
        let adapter = Arc::new(Adapter {
            db: db.clone(),
            gate: gate.clone(),
        });
        let service = Arc::new(MailService {
            store: adapter.clone(),
            identity: adapter,
            policy: Arc::new(OwnerPolicy),
            queue: db,
            outbound: None,
        });
        Self {
            app: mail_protocol_imap::jmap_router(service, "https://mail.example.org".into()),
            gate,
            blob,
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.gate.open();
    }
}

pub fn request(path: &str, token: Option<&str>, typ: &str, body: &str) -> Request<Body> {
    let method = if path.starts_with("/upload") || path == "/jmap" {
        "POST"
    } else {
        "GET"
    };
    let mut req = Request::builder()
        .uri(path)
        .method(method)
        .header("content-type", typ);
    if let Some(token) = token {
        req = req.header("authorization", format!("Bearer {token}"));
    }
    req.body(Body::from(body.to_owned())).unwrap()
}

impl mail_api::SubmissionStore for Adapter {
    fn submissions(
        &self,
        a: &str,
        ids: Option<&[String]>,
    ) -> Result<mail_api::SubmissionSelection, Error> {
        self.db.submissions(a, ids)
    }
    fn accept_submission(
        &self,
        a: &str,
        r: u64,
        v: mail_api::SubmissionAcceptance,
    ) -> Result<mail_api::SubmissionSelection, Error> {
        self.db.accept_submission(a, r, v)
    }
    fn cancel_submission(
        &self,
        a: &str,
        r: u64,
        id: &str,
    ) -> Result<mail_api::SubmissionSelection, mail_api::SubmissionFailure> {
        self.db.cancel_submission(a, r, id)
    }
    fn destroy_submission(&self, a: &str, r: u64, id: &str) -> Result<u64, Error> {
        self.db.destroy_submission(a, r, id)
    }
    fn query_submissions(
        &self,
        a: &str,
        r: Option<u64>,
        q: &mail_kernel::SubmissionQuery,
    ) -> Result<mail_api::SubmissionPage, mail_api::SubmissionFailure> {
        self.db.query_submissions(a, r, q)
    }
    fn submission_changes(
        &self,
        a: &str,
        r: u64,
        l: usize,
    ) -> Result<mail_api::SubmissionChanges, Error> {
        self.db.submission_changes(a, r, l)
    }
}
