#[path = "pop/support.rs"]
mod support;
use mail_api::{Action, MetadataStore, Policy};
use mail_kernel::{Command, Error};
use mail_service::{MailService, OwnerPolicy};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use support::*;

#[tokio::test]
async fn rejected_reauthentication_cannot_clear_or_rebind_pending_deletions() {
    let (service, db) = service();
    deliver(&db, b"Subject: one\r\n\r\none\r\n");
    deliver(&db, b"Subject: two\r\n\r\ntwo\r\n");
    let (mut client, task) = Client::connect(service, true).await;
    client.login().await;
    client.ok("DELE 1").await;
    for command in [
        "AUTH",
        "AUTH PLAIN",
        "USER bob@example.org",
        "PASS invalid",
        "STLS",
    ] {
        client.error(command).await;
    }
    assert!(client.ok("STAT").await.starts_with(b"+OK 1 "));
    client.close(task).await;
    let account = db.account("a").unwrap();
    assert_eq!(account.messages.len(), 1);
    assert!(
        db.blob("a", &account.messages[0].id)
            .unwrap()
            .ends_with(b"two\r\n")
    );
}

#[tokio::test]
async fn stale_pop_uid_cannot_read_or_delete_a_readded_inbox_membership() {
    let (service, db) = service();
    deliver(&db, b"Subject: retained\r\n\r\nretained\r\n");
    let account = db.account("a").unwrap();
    let account = db
        .execute(
            "a",
            mail_api::Precondition::Observed(account.revision),
            vec![Command::CreateMailbox {
                name: "Archive".into(),
            }],
        )
        .map(|_| db.account("a").unwrap())
        .unwrap();
    let archive = account
        .mailboxes
        .iter()
        .find(|m| m.name == "Archive")
        .unwrap()
        .id
        .clone();
    let id = account.messages[0].id.clone();
    let (mut old, old_task) = Client::connect(service.clone(), true).await;
    old.login().await;
    let uidl = old.ok("UIDL 1").await;
    old.ok("DELE 1").await;
    let moved = db
        .execute(
            "a",
            mail_api::Precondition::Observed(account.revision),
            vec![Command::SetMailboxes {
                id: id.clone(),
                mailboxes: vec![archive.clone()],
            }],
        )
        .map(|_| db.account("a").unwrap())
        .unwrap();
    let restored = db
        .execute(
            "a",
            mail_api::Precondition::Observed(moved.revision),
            vec![Command::SetMailboxes {
                id: id.clone(),
                mailboxes: vec![archive, "inbox".into()],
            }],
        )
        .map(|_| db.account("a").unwrap())
        .unwrap();
    old.ok("RSET").await;
    old.error("RETR 1").await;
    old.ok("DELE 1").await;
    old.close(old_task).await;
    assert_eq!(db.account("a").unwrap(), restored);
    let (mut fresh, fresh_task) = Client::connect(service, true).await;
    fresh.login().await;
    assert_ne!(fresh.ok("UIDL 1").await, uidl);
    assert!(
        fresh
            .multiline("RETR 1")
            .await
            .ends_with(b"retained\r\n.\r\n")
    );
    fresh.close(fresh_task).await;
}

struct WriteSwitch(Arc<AtomicBool>);
impl Policy for WriteSwitch {
    fn authorize(
        &self,
        principal: &mail_api::Principal,
        action: Action,
        account: &mail_api::AccountInfo,
    ) -> Result<(), Error> {
        OwnerPolicy.authorize(principal, action, account)?;
        if action == Action::Write && !self.0.load(Ordering::SeqCst) {
            Err(Error::Forbidden)
        } else {
            Ok(())
        }
    }
}
#[tokio::test]
async fn write_only_revocation_at_quit_preserves_marked_messages_and_blobs() {
    let (_, db) = service();
    deliver(&db, b"Subject: retained\r\n\r\nretained\r\n");
    let before = db.account("a").unwrap();
    let allowed = Arc::new(AtomicBool::new(true));
    let service = Arc::new(MailService {
        outbound: None,
        queue: db.clone(),
        identity: db.clone(),
        store: db.clone(),
        policy: Arc::new(WriteSwitch(allowed.clone())),
    });
    let (mut client, task) = Client::connect(service, true).await;
    client.login().await;
    client.ok("DELE 1").await;
    allowed.store(false, Ordering::SeqCst);
    client.ok("STAT").await;
    client.error("QUIT").await;
    task.await.unwrap().unwrap();
    assert_eq!(db.account("a").unwrap(), before);
    assert!(
        db.blob("a", &before.messages[0].id)
            .unwrap()
            .ends_with(b"retained\r\n")
    );
}

#[tokio::test]
async fn disconnect_during_retrieval_cannot_commit_queued_quit() {
    use tokio::io::AsyncWriteExt;
    let (service, db) = service();
    deliver(&db, b"Subject: retained\r\n\r\nretained\r\n");
    let mut raw = b"Subject: large\r\n\r\n".to_vec();
    raw.extend(vec![b'x'; 2 * 1024 * 1024]);
    deliver(&db, &raw);
    let before = db.account("a").unwrap();
    let (mut client, task) = Client::connect(service, true).await;
    client.login().await;
    client.ok("DELE 1").await;
    client
        .0
        .get_mut()
        .write_all(b"RETR 2\r\nQUIT\r\n")
        .await
        .unwrap();
    assert!(client.read().await.starts_with(b"+OK"));
    drop(client);
    assert!(
        tokio::time::timeout(std::time::Duration::from_secs(10), task)
            .await
            .unwrap()
            .unwrap()
            .is_err()
    );
    assert_eq!(db.account("a").unwrap(), before);
}
