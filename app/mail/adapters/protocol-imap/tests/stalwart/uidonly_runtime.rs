use super::*;
use mail_api::MetadataStore;
use mail_kernel::Account;
use mail_service::{MailService, OwnerPolicy};
use mail_sqlite_store::SqliteStore;
use std::sync::Arc;

#[tokio::test]
async fn uidonly_idle_uses_uidfetch_and_vanished_after_uid_sequence_divergence() {
    let service = fixture();
    client::initialize(service.clone());
    let mut client = connect().await;
    login(&mut client).await;
    request(&mut client, "SELECT INBOX", ResponseType::Ok).await;
    request(&mut client, "ENABLE UIDONLY", ResponseType::Ok).await;
    request(
        &mut client,
        "UID STORE 1 +FLAGS (\\Deleted)",
        ResponseType::Ok,
    )
    .await;
    request(&mut client, "EXPUNGE", ResponseType::Ok).await;
    client.send("IDLE").await;
    let response = client.read_prefix("+").await;
    assert!(
        response.contains("* 2 UIDFETCH (") && !response.contains(" FETCH ("),
        "{response}"
    );
    let account = service.store.account("a").unwrap();
    let id = account.messages[0].id.clone();
    let account = service
        .store
        .execute(
            "a",
            mail_api::Precondition::Observed(account.revision),
            vec![Command::Keywords {
                id: id.clone(),
                keywords: vec!["$flagged".into()],
            }],
        )
        .unwrap();
    let response = client.read_prefix("* 2 UIDFETCH").await;
    assert!(
        response.contains("\\Flagged") && !response.contains(" FETCH ("),
        "{response}"
    );
    service
        .store
        .execute(
            "a",
            mail_api::Precondition::Observed(account.revision),
            vec![Command::Destroy { id }],
        )
        .unwrap();
    let response = client.read_prefix("* VANISHED ").await;
    assert!(
        response.contains("* VANISHED 2") && !response.contains(" EXPUNGE"),
        "{response}"
    );
    client.send_raw(b"DONE\r\n").await;
    let response = client.assert_read(Type::Tagged, ResponseType::Ok).await;
    assert!(response.last().unwrap().contains(" OK "), "{response:?}");
    client.close().await;
}

fn tenants() -> (Arc<MailService>, Arc<SqliteStore>) {
    let db = Arc::new(SqliteStore::open(":memory:").unwrap());
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        crate::TOKEN,
    )
    .unwrap();
    db.provision(
        Account::new("b", "other", "bob", "bob@example.org").unwrap(),
        &"b".repeat(32),
    )
    .unwrap();
    db.deliver(
        &["alice@example.org".into()],
        b"Subject: alice\r\n\r\nalice private message",
    )
    .unwrap();
    db.deliver(
        &["bob@example.org".into()],
        b"Subject: bob\r\n\r\nbob private message",
    )
    .unwrap();
    let service = Arc::new(MailService {
        outbound: None,
        queue: db.clone(),
        store: db.clone(),
        identity: db.clone(),
        policy: Arc::new(OwnerPolicy),
    });
    (service, db)
}

#[tokio::test]
async fn unauthenticate_clears_identity_and_saved_results_before_another_tenant_login() {
    let (service, db) = tenants();
    client::initialize(service);
    let mut client = connect().await;
    login(&mut client).await;
    request(&mut client, "SELECT INBOX", ResponseType::Ok).await;
    request(&mut client, "ENABLE UIDONLY", ResponseType::Ok).await;
    request(
        &mut client,
        "UID SEARCH RETURN (SAVE) ALL",
        ResponseType::Ok,
    )
    .await;
    request(&mut client, "UNAUTHENTICATE", ResponseType::Ok).await;
    request(&mut client, "UID FETCH $ BODY[]", ResponseType::No).await;
    client
        .authenticate("bob@example.org", &"b".repeat(32))
        .await;
    request(&mut client, "SELECT INBOX", ResponseType::Ok).await;
    let response = request(&mut client, "UID FETCH $ BODY[]", ResponseType::Ok).await;
    assert!(
        !response.contains("FETCH (") && !response.contains("alice private"),
        "{response}"
    );
    let response = request(&mut client, "FETCH 1 BODY.PEEK[]", ResponseType::Ok).await;
    assert!(
        response.contains("bob private message") && !response.contains("alice private"),
        "{response}"
    );
    assert!(db.account("a").unwrap().messages[0].keywords.is_empty());
    assert!(db.account("b").unwrap().messages[0].keywords.is_empty());
    client.close().await;
}

#[tokio::test]
async fn uidonly_readonly_and_revoked_credentials_remain_enforced() {
    let (service, db) = tenants();
    client::initialize(service);
    let mut client = connect().await;
    login(&mut client).await;
    request(&mut client, "EXAMINE INBOX", ResponseType::Ok).await;
    request(&mut client, "ENABLE UIDONLY", ResponseType::Ok).await;
    let before = db.account("a").unwrap();
    request(&mut client, "UID FETCH 1 BODY[]", ResponseType::Ok).await;
    request(
        &mut client,
        "UID STORE 1 +FLAGS (\\Deleted)",
        ResponseType::No,
    )
    .await;
    request(&mut client, "EXPUNGE", ResponseType::No).await;
    assert_eq!(db.account("a").unwrap(), before);
    db.revoke("a").unwrap();
    let response = request(&mut client, "UID FETCH 1 BODY[]", ResponseType::No).await;
    assert!(
        !response.contains("UIDFETCH") && !response.contains("alice private"),
        "{response}"
    );
    request(&mut client, "UNAUTHENTICATE", ResponseType::Ok).await;
    client.close().await;
}

#[tokio::test]
async fn failed_enable_does_not_partially_switch_to_uidonly_or_change_selected_state() {
    client::initialize(fixture());
    let mut client = connect().await;
    login(&mut client).await;
    request(&mut client, "SELECT INBOX", ResponseType::Ok).await;
    for text in [
        "ENABLE UIDONLY UNKNOWN",
        "ENABLE UIDONLY)",
        "ENABLE (UIDONLY)",
    ] {
        let response = request(&mut client, text, ResponseType::Bad).await;
        assert!(!response.contains("ENABLED"), "{response}");
        let response = request(&mut client, "FETCH 1 (UID)", ResponseType::Ok).await;
        assert!(
            response.contains(" FETCH (") && !response.contains("UIDFETCH"),
            "{response}"
        );
    }
    request(&mut client, "ENABLE UIDONLY QRESYNC", ResponseType::Ok).await;
    let response = request(
        &mut client,
        "SELECT INBOX (QRESYNC (1 0 1:2 (1:2 1:2)))",
        ResponseType::Bad,
    )
    .await;
    assert!(response.contains("[UIDREQUIRED]"), "{response}");
    assert!(
        request(&mut client, "UID FETCH 1 (UID)", ResponseType::Ok)
            .await
            .contains("UIDFETCH")
    );
    let response = request(
        &mut client,
        "SELECT INBOX (QRESYNC (1 0 1:2))",
        ResponseType::Ok,
    )
    .await;
    assert!(
        response.contains("UIDFETCH (") && !response.contains(" FETCH ("),
        "{response}"
    );
    client.close().await;
}
