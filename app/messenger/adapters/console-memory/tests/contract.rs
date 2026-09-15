use messenger_collaboration_api::ConsoleCollaboration;
use messenger_console_memory::MemoryConsole;
use messenger_domain::{ConsoleCommand, ConsoleObjectRef, Error};
use serde_json::json;
use std::sync::Arc;

const COMPANY: &str = "11111111-1111-4111-8111-111111111111";
const OBJECT_TYPE: &str = "22222222-2222-4222-8222-222222222222";
const INSTANCE: &str = "33333333-3333-4333-8333-333333333333";
const COMMAND_ID: &str = "44444444-4444-4444-8444-444444444444";

fn object() -> ConsoleObjectRef {
    ConsoleObjectRef {
        company: COMPANY.into(),
        object_type: OBJECT_TYPE.into(),
        instance: INSTANCE.into(),
        revision: 7,
    }
}

fn command() -> ConsoleCommand {
    ConsoleCommand {
        object: object(),
        action: "assign".into(),
        command_id: COMMAND_ID.into(),
        params: json!({"assignee": "alice"}),
        reason: None,
        checklist_all_acknowledged: None,
        four_eyes_request_ref: None,
    }
}

async fn seeded() -> Arc<MemoryConsole> {
    let store = MemoryConsole::new(COMPANY);
    store.grant("viewer", COMPANY).await.unwrap();
    store
        .put(object(), json!({"title": "Visible"}))
        .await
        .unwrap();
    store
}

#[tokio::test]
async fn pinned_read_returns_seeded_body_and_rejects_a_moved_revision() {
    let store = seeded().await;
    let read = store.read("viewer", &object()).await.unwrap();
    assert_eq!(read.body, json!({"title": "Visible"}));
    assert_eq!(read.object, object());
    let mut stale = object();
    stale.revision = 8;
    assert!(matches!(
        store.read("viewer", &stale).await,
        Err(Error::Invalid(_))
    ));
    let mut missing = object();
    missing.instance = "55555555-5555-4555-8555-555555555555".into();
    assert_eq!(store.read("viewer", &missing).await, Err(Error::Denied));
}

#[tokio::test]
async fn foreign_company_and_unbound_credentials_never_execute() {
    let store = seeded().await;
    let mut foreign = command();
    foreign.object.company = "66666666-6666-4666-8666-666666666666".into();
    assert_eq!(store.execute("viewer", &foreign).await, Err(Error::Denied));
    for credential in ["", "forged", "viewer\n", "other-company"] {
        assert_eq!(
            store.execute(credential, &command()).await,
            Err(Error::Denied)
        );
    }
    assert_eq!(
        store
            .read("viewer", &object())
            .await
            .unwrap()
            .object
            .revision,
        7
    );
}

#[tokio::test]
async fn preflight_does_not_commit_and_execute_does_not_need_it() {
    let store = seeded().await;
    let preview = store.preflight("viewer", &command()).await.unwrap();
    assert!(preview.would_execute);
    assert_eq!(
        store
            .read("viewer", &object())
            .await
            .unwrap()
            .object
            .revision,
        7
    );
    let receipt = store.execute("viewer", &command()).await.unwrap();
    assert!(!receipt.reused);
    assert_eq!(receipt.command_id, COMMAND_ID);
    assert_eq!(receipt.object.revision, 8);
    assert!(matches!(
        store.read("viewer", &object()).await,
        Err(Error::Invalid(_))
    ));
    assert!(
        !store
            .preflight("viewer", &command())
            .await
            .unwrap()
            .would_execute
    );
}

#[tokio::test]
async fn lost_ack_retries_keep_the_original_receipt_and_revision() {
    let store = seeded().await;
    let first = store.execute("viewer", &command()).await.unwrap();
    let mut changed = command();
    changed.params = json!({"assignee": "bob"});
    let retry = store.execute("viewer", &changed).await.unwrap();
    assert!(retry.reused);
    assert_eq!(retry.command_id, first.command_id);
    assert_eq!(retry.object.revision, first.object.revision);
    let mut head = object();
    head.revision = 8;
    store.read("viewer", &head).await.unwrap();
}

#[tokio::test]
async fn invalid_command_is_refused_before_a_receipt_exists() {
    let store = seeded().await;
    let mut bad = command();
    bad.action.clear();
    assert!(matches!(
        store.execute("viewer", &bad).await,
        Err(Error::Invalid(_))
    ));
    let receipt = store.execute("viewer", &command()).await.unwrap();
    assert!(!receipt.reused);
    assert_eq!(receipt.object.revision, 8);
}

#[tokio::test]
async fn concurrent_execute_commits_exactly_one_revision() {
    let store = seeded().await;
    let left = command();
    let mut right = command();
    right.command_id = "55555555-5555-4555-8555-555555555555".into();
    let (one, two) = tokio::join!(
        store.execute("viewer", &left),
        store.execute("viewer", &right)
    );
    let ok = [one, two].into_iter().filter(Result::is_ok).count();
    assert_eq!(ok, 1);
    let mut head = object();
    head.revision = 8;
    store.read("viewer", &head).await.unwrap();
    head.revision = 9;
    assert!(matches!(
        store.read("viewer", &head).await,
        Err(Error::Invalid(_))
    ));
}
