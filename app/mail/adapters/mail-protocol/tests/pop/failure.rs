use super::support::*;
use mail_api::{AccountInfo, MailboxChange, MessageChange, MessageSelection, Store};
use mail_kernel::{Account, Command, Error};
use mail_service::{MailService, OwnerPolicy};
use mail_sqlite::SqliteStore;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

struct FaultStore {
    db: Arc<SqliteStore>,
    conflicts: AtomicUsize,
    reject_batch: bool,
    batches: Mutex<Vec<usize>>,
}
impl Store for FaultStore {
    fn account_info(&self, id: &str) -> Result<AccountInfo, Error> {
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
    fn execute(
        &self,
        id: &str,
        revision: u64,
        mut commands: Vec<Command>,
    ) -> Result<Account, Error> {
        self.batches.lock().unwrap().push(commands.len());
        if self
            .conflicts
            .try_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_sub(1))
            .is_ok()
        {
            return Err(Error::Conflict);
        }
        if self.reject_batch {
            commands.push(Command::Destroy {
                id: "missing".into(),
            });
        }
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
        self.db.put_blob(account, raw)
    }
    fn blob(&self, account: &str, id: &str) -> Result<Vec<u8>, Error> {
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

#[tokio::test]
async fn quit_retries_whole_batches_on_conflict_and_never_commits_a_failed_batch() {
    for (conflicts, reject_batch) in [(2, false), (3, false), (0, true)] {
        let (_, db) = service();
        deliver(&db, b"Subject: one\r\n\r\none\r\n");
        deliver(&db, b"Subject: two\r\n\r\ntwo\r\n");
        let before = db.account("a").unwrap();
        let store = Arc::new(FaultStore {
            db: db.clone(),
            conflicts: AtomicUsize::new(conflicts),
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
        if conflicts == 2 {
            client.close(task).await;
            assert!(db.account("a").unwrap().messages.is_empty());
        } else {
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
        }
        assert_eq!(
            *store.batches.lock().unwrap(),
            vec![2; if reject_batch { 1 } else { 3 }]
        );
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
