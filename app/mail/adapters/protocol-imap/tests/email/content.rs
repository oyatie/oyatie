use super::*;
use mail_api::{
    AccountInfo, AuditRow, BlobStore, ChangeFeed, Consumer, Dirty, Execution, FeedRead,
    HistoryPage, MailboxSelection, Precondition, Resume,
};
use mail_kernel::{BlobRef, Error};
use std::sync::{
    Mutex,
    atomic::{AtomicBool, Ordering},
};

struct ObservedStore {
    inner: Arc<SqliteStore>,
    reads: Mutex<Vec<String>>,
    refuse_snapshot: AtomicBool,
    missing_body: AtomicBool,
}

impl MetadataStore for ObservedStore {
    fn messages(&self, account: &str, ids: &[String]) -> Result<mail_api::MessageSelection, Error> {
        self.inner.messages(account, ids)
    }
    fn mailbox_uids(&self, account: &str, mailbox: &str) -> Result<MailboxSelection, Error> {
        self.inner.mailbox_uids(account, mailbox)
    }
    fn account_info(&self, id: &str) -> Result<AccountInfo, Error> {
        self.inner.account_info(id)
    }
    fn account(&self, id: &str) -> Result<Account, Error> {
        if self.refuse_snapshot.load(Ordering::SeqCst) {
            return Err(Error::Unavailable);
        }
        self.inner.account(id)
    }
    fn resolve(&self, address: &str) -> Result<String, Error> {
        self.inner.resolve(address)
    }
    fn execute(
        &self,
        id: &str,
        precondition: Precondition,
        commands: Vec<Command>,
    ) -> Result<Execution, Error> {
        self.inner.execute(id, precondition, commands)
    }
    fn deliver_once(
        &self,
        account: &str,
        key: &str,
        raw: &[u8],
        received_at: i64,
    ) -> Result<(), Error> {
        self.inner.deliver_once(account, key, raw, received_at)
    }
    fn put_blob(&self, account: &str, raw: &[u8]) -> Result<String, Error> {
        self.inner.put_blob(account, raw)
    }
    fn blob(&self, account: &str, id: &str) -> Result<Vec<u8>, Error> {
        self.reads.lock().unwrap().push(id.into());
        if self.missing_body.load(Ordering::SeqCst) {
            return Err(Error::NotFound);
        }
        self.inner.blob(account, id)
    }
    fn history(&self, account: &str, since: u64, limit: usize) -> Result<HistoryPage, Error> {
        self.inner.history(account, since, limit)
    }
    fn compact_history(
        &self,
        account: &str,
        now: i64,
        policy: mail_kernel::RetentionPolicy,
        cursors: &[(mail_api::Consumer, u64)],
    ) -> Result<mail_kernel::Retention, Error> {
        self.inner.compact_history(account, now, policy, cursors)
    }
}

#[tokio::test]
async fn content_fetch_is_selective_and_authorization_never_fetches_mailbox_state() {
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    let raw = b"Subject: first\r\n\r\nbody";
    db.deliver(&["alice@example.org".into()], raw).unwrap();
    db.deliver(
        &["alice@example.org".into()],
        b"Subject: unrelated\r\n\r\nlarge body",
    )
    .unwrap();
    let store = Arc::new(ObservedStore {
        inner: db.clone(),
        reads: Mutex::default(),
        refuse_snapshot: AtomicBool::new(false),
        missing_body: AtomicBool::new(false),
    });
    let service = Arc::new(MailService {
        outbound: None,
        queue: db.clone(),
        store: store.clone(),
        identity: db,
        policy: Arc::new(OwnerPolicy),
    });
    let app = mail_protocol_imap::jmap_router(service.clone(), "https://localhost".into());
    let metadata = call(
        &app,
        "Email/get",
        json!({"accountId":"a","ids":["e1"],"properties":["id","keywords","size"]}),
    )
    .await;
    assert_eq!(metadata["list"][0]["size"], raw.len());
    assert!(store.reads.lock().unwrap().is_empty());
    let content = call(&app, "Email/get", json!({"accountId":"a","ids":["e1"]})).await;
    assert_eq!(content["list"][0]["subject"], "first");
    assert_eq!(*store.reads.lock().unwrap(), ["e1"]);
    store.refuse_snapshot.store(true, Ordering::SeqCst);
    assert_eq!(service.read(TOKEN, "a"), Err(Error::Unavailable));
    let selected = call(
        &app,
        "Email/get",
        json!({"accountId":"a","ids":["e1"],"properties":["id","subject","threadId"]}),
    )
    .await;
    assert_eq!(selected["list"][0]["subject"], "first");
    store.missing_body.store(true, Ordering::SeqCst);
    let removed = response(
        &app,
        "Email/get",
        json!({"accountId":"a","ids":["e1"],"properties":["subject"]}),
    )
    .await;
    assert_eq!(removed["methodResponses"][0][1]["type"], "serverFail");
    let metadata = call(
        &app,
        "Email/get",
        json!({"accountId":"a","ids":["e1"],"properties":["id"]}),
    )
    .await;
    assert_eq!(metadata["list"][0]["id"], "e1");
    store.missing_body.store(false, Ordering::SeqCst);
    assert!(
        service
            .authorize(TOKEN, "a", mail_api::Action::Read)
            .is_ok()
    );
    assert_eq!(service.download(TOKEN, "a", "e1").unwrap(), raw);
    let history = service.history(TOKEN, "a", 0, 100).unwrap();
    assert!(
        history
            .rows
            .iter()
            .any(|(_, entry)| entry.message() == Some("e1")),
        "{history:?}"
    );
    let blob = service.upload(TOKEN, "a", b"standalone content").unwrap();
    assert_eq!(
        service.download(TOKEN, "a", &blob).unwrap(),
        b"standalone content"
    );
    let session = app
        .oneshot(
            Request::get("/.well-known/jmap")
                .header("authorization", format!("Bearer {TOKEN}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(session.status().is_success());
}

#[tokio::test]
async fn compose_bounds_repeated_blob_reads_and_encoded_message_size() {
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        TOKEN,
    )
    .unwrap();
    let store = Arc::new(ObservedStore {
        inner: db.clone(),
        reads: Mutex::default(),
        refuse_snapshot: AtomicBool::new(false),
        missing_body: AtomicBool::new(false),
    });
    let app = mail_protocol_imap::jmap_router(
        Arc::new(MailService {
            outbound: None,
            queue: db.clone(),
            store: store.clone(),
            identity: db.clone(),
            policy: Arc::new(OwnerPolicy),
        }),
        "https://localhost".into(),
    );
    let blob = db.put_blob("a", &vec![0; 6 * 1024 * 1024]).unwrap();
    let part = json!({"type":"application/octet-stream", "blobId":blob});
    let result = call(&app, "Email/set", json!({"accountId":"a", "create":{"large":{
        "mailboxIds":{"inbox":true}, "bodyStructure":{"type":"multipart/mixed", "subParts":vec![part.clone(); 10]}
    }}})).await;
    assert_eq!(result["notCreated"]["large"]["type"], "tooLarge");
    assert!(
        store.reads.lock().unwrap().len() <= 5,
        "stop fetching once decoded body budget is exhausted"
    );
    store.reads.lock().unwrap().clear();
    // Four 6MiB bodies fit decoded budget, but base64 plus MIME framing exceeds 25MiB.
    let result = call(&app, "Email/set", json!({"accountId":"a", "create":{"encoded":{
        "mailboxIds":{"inbox":true}, "bodyStructure":{"type":"multipart/mixed", "subParts":vec![part; 4]}
    }}})).await;
    assert_eq!(result["notCreated"]["encoded"]["type"], "tooLarge");
    assert!(db.account("a").unwrap().messages.is_empty());
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

#[path = "content_feed.rs"]
mod content_feed;
