use super::support::*;
use mail_api::{
    AccountInfo, AuditRow, BlobStore, ChangeFeed, Consumer, Dirty, Execution, FeedRead,
    HistoryPage, MailboxSelection, MessageSelection, MetadataStore, Precondition, Resume,
};
use mail_kernel::{Account, BlobRef, Command, Error};
use mail_service::{MailService, OwnerPolicy};
use mail_sqlite_store::SqliteStore;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

struct FaultStore {
    db: Arc<SqliteStore>,
    busy: AtomicUsize,
    reject_batch: bool,
    batches: Mutex<Vec<usize>>,
}
impl MetadataStore for FaultStore {
    fn account_info(&self, id: &str) -> Result<AccountInfo, Error> {
        self.db.account_info(id)
    }
    fn account(&self, id: &str) -> Result<Account, Error> {
        self.db.account(id)
    }
    fn messages(&self, account: &str, ids: &[String]) -> Result<MessageSelection, Error> {
        self.db.messages(account, ids)
    }
    fn mailbox_uids(&self, account: &str, mailbox: &str) -> Result<MailboxSelection, Error> {
        self.db.mailbox_uids(account, mailbox)
    }
    fn resolve(&self, address: &str) -> Result<String, Error> {
        self.db.resolve(address)
    }
    fn execute(
        &self,
        id: &str,
        precondition: Precondition,
        mut commands: Vec<Command>,
    ) -> Result<Execution, Error> {
        self.batches.lock().unwrap().push(commands.len());
        if self
            .busy
            .try_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_sub(1))
            .is_ok()
        {
            return Err(Error::Busy);
        }
        if self.reject_batch {
            commands.push(Command::Destroy {
                id: "missing".into(),
            });
        }
        self.db.execute(id, precondition, commands)
    }
    fn deliver_once(
        &self,
        account: &str,
        key: &str,
        raw: &[u8],
        received_at: i64,
    ) -> Result<Option<String>, Error> {
        self.db.deliver_once(account, key, raw, received_at)
    }
    fn put_blob(&self, account: &str, raw: &[u8]) -> Result<String, Error> {
        self.db.put_blob(account, raw)
    }
    fn blob(&self, account: &str, id: &str) -> Result<Vec<u8>, Error> {
        self.db.blob(account, id)
    }
    fn history(&self, account: &str, since: u64, limit: usize) -> Result<HistoryPage, Error> {
        self.db.history(account, since, limit)
    }
    fn compact_history(
        &self,
        account: &str,
        now: i64,
        policy: mail_kernel::RetentionPolicy,
        cursors: &[(mail_api::Consumer, u64)],
    ) -> Result<mail_kernel::Retention, Error> {
        self.db.compact_history(account, now, policy, cursors)
    }
}

impl BlobStore for FaultStore {
    fn persist(&self, a: &str, s: &str, r: &[u8], ttl: i64) -> Result<BlobRef, Error> {
        self.db.persist(a, s, r, ttl)
    }
    fn renew(&self, a: &str, s: &str, ttl: i64) -> Result<(), Error> {
        self.db.renew(a, s, ttl)
    }
    fn read(&self, a: &str, b: &BlobRef) -> Result<Vec<u8>, Error> {
        self.db.read(a, b)
    }
    fn orphan_sweep(&self, now: i64, limit: usize) -> Result<usize, Error> {
        self.db.orphan_sweep(now, limit)
    }
}

impl ChangeFeed for FaultStore {
    fn dirty(
        &self,
        c: Consumer,
        r: &mut Resume,
        now: i64,
        per: usize,
        limit: usize,
    ) -> Result<Vec<Dirty>, Error> {
        self.db.dirty(c, r, now, per, limit)
    }
    fn changes(&self, c: Consumer, a: &str, limit: usize) -> Result<FeedRead, Error> {
        self.db.changes(c, a, limit)
    }
    fn acknowledge(
        &self,
        c: Consumer,
        a: &str,
        rev: u64,
        now: i64,
        delay: i64,
    ) -> Result<(), Error> {
        self.db.acknowledge(c, a, rev, now, delay)
    }
    fn poison(&self, c: Consumer, a: &str, reason: &str) -> Result<(), Error> {
        self.db.poison(c, a, reason)
    }
    fn poisoned(&self, c: Consumer) -> Result<Vec<(Dirty, String)>, Error> {
        self.db.poisoned(c)
    }
    fn cursors(&self, a: &str, enabled: &[Consumer]) -> Result<Vec<(Consumer, u64)>, Error> {
        self.db.cursors(a, enabled)
    }
    fn reconcile(&self, c: Consumer, op: &str, reason: &str) -> Result<u64, Error> {
        self.db.reconcile(c, op, reason)
    }
    fn dead_letter(
        &self,
        c: Consumer,
        a: &str,
        rev: u64,
        op: &str,
        reason: &str,
    ) -> Result<(), Error> {
        self.db.dead_letter(c, a, rev, op, reason)
    }
    fn retire_cursor(
        &self,
        c: Consumer,
        t: Option<&str>,
        op: &str,
        reason: &str,
    ) -> Result<u64, Error> {
        self.db.retire_cursor(c, t, op, reason)
    }
    fn audit(&self, limit: usize) -> Result<Vec<AuditRow>, Error> {
        self.db.audit(limit)
    }
}

/// QUIT commits one observed batch: a busy store is retried by the socket
/// task with the same whole batch, and a batch the store refuses commits
/// nothing.
#[tokio::test]
async fn quit_retries_whole_batches_while_busy_and_never_commits_a_failed_batch() {
    for (busy, reject_batch) in [(1, false), (2, false), (0, true)] {
        let (_, db) = service();
        deliver(&db, b"Subject: one\r\n\r\none\r\n");
        deliver(&db, b"Subject: two\r\n\r\ntwo\r\n");
        let before = db.account("a").unwrap();
        let store = Arc::new(FaultStore {
            db: db.clone(),
            busy: AtomicUsize::new(busy),
            reject_batch,
            batches: Mutex::new(vec![]),
        });
        let service = Arc::new(MailService {
            outbound: None,
            queue: db.clone(),
            store: store.clone(),
            identity: db.clone(),
            policy: Arc::new(OwnerPolicy),
        });
        let (mut client, task) = Client::connect(service, true).await;
        client.login().await;
        client.ok("DELE 1").await;
        client.ok("DELE 2").await;
        assert_eq!(db.account("a").unwrap(), before);
        if reject_batch {
            client.error("QUIT").await;
            task.await.unwrap().unwrap();
            assert_eq!(db.account("a").unwrap(), before);
            assert_eq!(
                db.blob("a", &before.messages[0].id).unwrap(),
                b"Subject: one\r\n\r\none\r\n"
            );
            assert_eq!(
                db.blob("a", &before.messages[1].id).unwrap(),
                b"Subject: two\r\n\r\ntwo\r\n"
            );
        } else {
            client.close(task).await;
            assert!(db.account("a").unwrap().messages.is_empty());
        }
        assert_eq!(*store.batches.lock().unwrap(), vec![2; busy + 1]);
    }
}

impl mail_api::SubmissionStore for FaultStore {
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
