use messenger_domain::{
    Delivery, Error, InstallationSpec, IntegrationCapability as Cap, IntegrationKind,
};
use messenger_installation_api::InstallationStore;
use messenger_installation_memory::MemoryInstallations;
use serde_json::json;
use std::{collections::BTreeSet, time::Duration};

const ROOM: &str = "!room:local";
const OTHER: &str = "!other:local";
const ID: &str = "install-1";

fn spec() -> InstallationSpec {
    InstallationSpec {
        id: ID.into(),
        room: ROOM.into(),
        service: "support".into(),
        workload: "support-worker".into(),
        matrix_user: Some("@support:local".into()),
        kind: IntegrationKind::Bridge,
        capabilities: BTreeSet::from([Cap::ReadMessages]),
    }
}

fn delivery(generation: u64, event: &str) -> Delivery {
    Delivery {
        installation: ID.into(),
        generation,
        event: event.into(),
        capability: Cap::ReadMessages,
        body: json!({"text":"private message"}),
        via: vec![],
    }
}

#[tokio::test]
async fn install_replays_the_same_command_and_rejects_a_changed_one() {
    let store = MemoryInstallations::new("acme");
    let spec = spec();
    let first = store.install(&spec, 0, "command-1").await.unwrap();
    assert_eq!(first.generation, 1);
    assert_eq!(first, store.install(&spec, 0, "command-1").await.unwrap());
    let mut changed = spec.clone();
    changed.service = "other".into();
    assert!(store.install(&changed, 0, "command-1").await.is_err());
    assert!(store.install(&spec, 0, "command-2").await.is_err());
    assert!(store.get(OTHER, ID).await.unwrap().is_none());
    assert_eq!(store.get(ROOM, ID).await.unwrap().as_ref(), Some(&first));
}

#[tokio::test]
async fn enqueue_is_idempotent_and_fences_a_changed_body() {
    let store = MemoryInstallations::new("acme");
    let spec = spec();
    store.install(&spec, 0, "command-1").await.unwrap();
    let event = delivery(1, "event-1");
    store.enqueue(ROOM, &event).await.unwrap();
    store.enqueue(ROOM, &event).await.unwrap();
    let mut changed = event.clone();
    changed.body = json!({"text":"changed"});
    assert!(matches!(
        store.enqueue(ROOM, &changed).await,
        Err(Error::Denied)
    ));
}

#[tokio::test]
async fn concurrent_claim_hands_the_delivery_to_one_caller() {
    let store = MemoryInstallations::new("acme");
    let installation = store.install(&spec(), 0, "command-1").await.unwrap();
    store.enqueue(ROOM, &delivery(1, "event-1")).await.unwrap();
    let (a, b) = tokio::join!(store.claim(&installation), store.claim(&installation));
    let (a, b) = (a.unwrap(), b.unwrap());
    assert_ne!(a.is_some(), b.is_some());
    let first = a.or(b).unwrap();
    assert_eq!(first.delivery, delivery(1, "event-1"));
}

#[tokio::test]
async fn expired_lease_keeps_idempotency_and_issues_a_new_lease() {
    let store = MemoryInstallations::new("acme");
    let installation = store.install(&spec(), 0, "command-1").await.unwrap();
    store.enqueue(ROOM, &delivery(1, "event-1")).await.unwrap();
    let first = store.claim(&installation).await.unwrap().unwrap();
    store.elapse(Duration::from_secs(31)).await;
    let second = store.claim(&installation).await.unwrap().unwrap();
    assert_eq!(first.idempotency_key, second.idempotency_key);
    assert_ne!(first.lease, second.lease);
    assert!(store.complete(&first, &json!({"ok": true})).await.is_err());
    store.complete(&second, &json!({"ok": true})).await.unwrap();
    store.complete(&second, &json!({"ok": true})).await.unwrap();
    assert!(
        store
            .complete(&second, &json!({"ok": false}))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn revocation_fences_new_work_and_lets_a_started_dispatch_finish() {
    let store = MemoryInstallations::new("acme");
    let installation = store.install(&spec(), 0, "command-1").await.unwrap();
    store.enqueue(ROOM, &delivery(1, "event-1")).await.unwrap();
    let dispatch = store.claim(&installation).await.unwrap().unwrap();
    let revoked = store.revoke(ROOM, ID, 1, "revoke-1").await.unwrap();
    assert_eq!(revoked.generation, 2);
    assert!(!revoked.enabled);
    assert!(store.claim(&installation).await.is_err());
    assert!(store.enqueue(ROOM, &delivery(1, "event-1")).await.is_err());
    store
        .complete(&dispatch, &json!({"ok": true}))
        .await
        .unwrap();
    assert!(store.retry(&dispatch).await.is_err());
    let active = store.install(&spec(), 2, "command-2").await.unwrap();
    assert_eq!(active.generation, 3);
    assert!(active.enabled);
}

#[tokio::test]
async fn pending_work_from_a_revoked_generation_does_not_dispatch() {
    let store = MemoryInstallations::new("acme");
    store.install(&spec(), 0, "command-1").await.unwrap();
    store.enqueue(ROOM, &delivery(1, "event-1")).await.unwrap();
    store.revoke(ROOM, ID, 1, "revoke-1").await.unwrap();
    let active = store.install(&spec(), 2, "command-2").await.unwrap();
    store.enqueue(ROOM, &delivery(3, "event-2")).await.unwrap();
    let claimed = store.claim(&active).await.unwrap().unwrap();
    assert_eq!(claimed.delivery.event, "event-2");
}

#[tokio::test]
async fn list_is_room_scoped_and_cursor_ordered() {
    let store = MemoryInstallations::new("acme");
    let mut first = spec();
    first.id = "install-a".into();
    let mut second = spec();
    second.id = "install-b".into();
    let mut other = spec();
    other.id = "install-a".into();
    other.room = OTHER.into();
    store.install(&first, 0, "c-a").await.unwrap();
    store.install(&second, 0, "c-b").await.unwrap();
    store.install(&other, 0, "c-o").await.unwrap();
    let listed = store.list(ROOM, None).await.unwrap();
    assert_eq!(
        listed
            .iter()
            .map(|value| value.spec.id.as_str())
            .collect::<Vec<_>>(),
        ["install-a", "install-b"]
    );
    let after = store.list(ROOM, Some("install-a")).await.unwrap();
    assert_eq!(after.len(), 1);
    assert_eq!(after[0].spec.id, "install-b");
}

#[tokio::test]
async fn retry_after_backoff_reuses_idempotency_until_revocation() {
    let store = MemoryInstallations::new("acme");
    let installation = store.install(&spec(), 0, "command-1").await.unwrap();
    store.enqueue(ROOM, &delivery(1, "event-1")).await.unwrap();
    let first = store.claim(&installation).await.unwrap().unwrap();
    store.retry(&first).await.unwrap();
    assert!(store.claim(&installation).await.unwrap().is_none());
    store.elapse(Duration::from_secs(2)).await;
    let second = store.claim(&installation).await.unwrap().unwrap();
    assert_eq!(first.idempotency_key, second.idempotency_key);
    assert_ne!(first.lease, second.lease);
}

#[tokio::test]
async fn queue_capacity_is_per_current_generation() {
    let store = MemoryInstallations::new("acme");
    store.install(&spec(), 0, "command-1").await.unwrap();
    for index in 0..1000 {
        store
            .enqueue(ROOM, &delivery(1, &format!("event-{index}")))
            .await
            .unwrap();
    }
    assert!(matches!(
        store.enqueue(ROOM, &delivery(1, "event-full")).await,
        Err(Error::Unavailable(_))
    ));
}
