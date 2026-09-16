use super::support::*;
use mail_api::Store;

#[tokio::test]
async fn deletion_is_pending_until_quit_and_rset_or_disconnect_rolls_back() {
    let (service, db) = service();
    deliver(&db, b"Subject: first\r\n\r\nfirst\r\n");
    deliver(&db, b"Subject: second\r\n\r\nsecond\r\n");
    let (mut client, task) = Client::connect(service.clone(), true).await;
    client.login().await;
    client.ok("DELE 1").await;
    assert_eq!(db.account("a").unwrap().messages.len(), 2);
    client.error("DELE 1").await;
    client.error("RETR 1").await;
    assert!(client.ok("STAT").await.starts_with(b"+OK 1 "));
    client.ok("RSET").await;
    assert!(client.ok("STAT").await.starts_with(b"+OK 2 "));
    client.ok("DELE 2").await;
    drop(client);
    task.await.unwrap().unwrap();
    assert_eq!(db.account("a").unwrap().messages.len(), 2);
    let (mut client, task) = Client::connect(service, true).await;
    client.login().await;
    client.ok("DELE 1").await;
    client.close(task).await;
    let remaining = db.account("a").unwrap();
    assert_eq!(remaining.messages.len(), 1);
    assert!(
        db.blob("a", &remaining.messages[0].id)
            .unwrap()
            .ends_with(b"second\r\n")
    );
}

#[tokio::test]
async fn message_numbers_are_snapshotted_uidl_is_stable_and_new_deliveries_survive_quit() {
    let (service, db) = service();
    deliver(&db, b"Subject: one\r\n\r\none\r\n");
    let (mut first, first_task) = Client::connect(service.clone(), true).await;
    first.login().await;
    let uidl = first.ok("UIDL 1").await;
    deliver(&db, b"Subject: two\r\n\r\ntwo\r\n");
    assert!(first.ok("STAT").await.starts_with(b"+OK 1 "));
    first.error("RETR 2").await;
    let (mut second, second_task) = Client::connect(service, true).await;
    second.login().await;
    assert_eq!(second.ok("UIDL 1").await, uidl);
    first.ok("DELE 1").await;
    first.close(first_task).await;
    second.error("RETR 1").await;
    assert!(second.multiline("RETR 2").await.ends_with(b"two\r\n.\r\n"));
    second.close(second_task).await;
    assert_eq!(db.account("a").unwrap().messages.len(), 1);
}

#[tokio::test]
async fn every_data_command_and_quit_recheck_revocation() {
    let (service, db) = service();
    deliver(&db, b"Subject: secret\r\n\r\nsecret\r\n");
    let (mut client, task) = Client::connect(service, true).await;
    client.login().await;
    client.ok("DELE 1").await;
    db.revoke("a").unwrap();
    for command in [
        "STAT", "LIST", "UIDL", "RETR 1", "TOP 1 0", "DELE 1", "RSET", "NOOP", "QUIT",
    ] {
        client.error(command).await;
    }
    task.await.unwrap().unwrap();
    assert_eq!(db.account("a").unwrap().messages.len(), 1);
}

#[tokio::test]
async fn read_only_policy_allows_retrieval_but_never_pending_deletion_or_seen_changes() {
    use mail_api::{AccountInfo, Action, Policy, Principal};
    use mail_kernel::Error;
    use mail_service::{MailService, OwnerPolicy};
    use std::sync::Arc;
    struct ReadOnly;
    impl Policy for ReadOnly {
        fn authorize(
            &self,
            principal: &Principal,
            action: Action,
            account: &AccountInfo,
        ) -> Result<(), Error> {
            OwnerPolicy.authorize(principal, action, account)?;
            if action == Action::Read {
                Ok(())
            } else {
                Err(Error::Forbidden)
            }
        }
    }
    let (_, db) = service();
    deliver(&db, b"Subject: policy\r\n\r\nbody\r\n");
    let before = db.account("a").unwrap();
    let service = Arc::new(MailService {
        outbound: None,
        queue: db.clone(),
        store: db.clone(),
        identity: db.clone(),
        policy: Arc::new(ReadOnly),
    });
    let (mut client, task) = Client::connect(service, true).await;
    client.login().await;
    client.multiline("RETR 1").await;
    client.error("DELE 1").await;
    client.close(task).await;
    assert_eq!(db.account("a").unwrap(), before);
}

#[tokio::test]
async fn quit_preserves_other_mailbox_memberships_and_commits_pipelined_deletions_together() {
    use mail_kernel::Command;
    use tokio::io::AsyncWriteExt;
    let (service, db) = service();
    deliver(&db, b"Subject: first\r\n\r\nfirst\r\n");
    deliver(&db, b"Subject: second\r\n\r\nsecond\r\n");
    let account = db.account("a").unwrap();
    let id = account.messages[0].id.clone();
    let account = db
        .execute(
            "a",
            account.revision,
            vec![Command::CreateMailbox {
                name: "Archive".into(),
            }],
        )
        .unwrap();
    let archive = account
        .mailboxes
        .iter()
        .find(|m| m.name == "Archive")
        .unwrap()
        .id
        .clone();
    db.execute(
        "a",
        account.revision,
        vec![Command::SetMailboxes {
            id: id.clone(),
            mailboxes: vec!["inbox".into(), archive.clone()],
        }],
    )
    .unwrap();
    let (mut client, task) = Client::connect(service, true).await;
    client.login().await;
    let revision = db.account("a").unwrap().revision;
    let events = mail_api::Events::pending(db.as_ref(), "pop-test", 100)
        .unwrap()
        .len();
    client
        .0
        .get_mut()
        .write_all(b"DELE 1\r\nDELE 2\r\nQUIT\r\n")
        .await
        .unwrap();
    for _ in 0..3 {
        assert!(client.read().await.starts_with(b"+OK"));
    }
    task.await.unwrap().unwrap();
    let account = db.account("a").unwrap();
    assert_eq!(account.revision, revision + 2);
    assert_eq!(
        mail_api::Events::pending(db.as_ref(), "pop-test", 100)
            .unwrap()
            .len(),
        events + 1
    );
    assert_eq!(account.messages.len(), 1);
    assert_eq!(account.messages[0].id, id);
    assert_eq!(
        account.messages[0]
            .mailboxes
            .keys()
            .cloned()
            .collect::<Vec<_>>(),
        vec![archive]
    );
}
