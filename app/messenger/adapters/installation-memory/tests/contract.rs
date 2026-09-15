use messenger_domain::{InstallationSpec, IntegrationCapability as Cap, IntegrationKind};
use messenger_installation_api::InstallationStore;
use messenger_installation_memory::MemoryInstallations;
use std::collections::BTreeSet;

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

#[tokio::test]
async fn install_replays_the_same_command_and_rejects_a_changed_one() {
    let store = MemoryInstallations::new();
    let spec = spec();
    let first = store.install(&spec, 0, "command-1").await.unwrap();
    assert_eq!(first.generation, 1);
    assert!(first.enabled);
    assert_eq!(first, store.install(&spec, 0, "command-1").await.unwrap());
    let mut changed = spec.clone();
    changed.service = "other".into();
    assert!(store.install(&changed, 0, "command-1").await.is_err());
    assert!(store.install(&spec, 0, "command-2").await.is_err());
    assert!(store.get(OTHER, ID).await.unwrap().is_none());
    assert_eq!(store.get(ROOM, ID).await.unwrap().as_ref(), Some(&first));
}

#[tokio::test]
async fn failed_install_does_not_occupy_the_command() {
    let store = MemoryInstallations::new();
    let spec = spec();
    store.install(&spec, 0, "command-1").await.unwrap();
    assert!(store.install(&spec, 0, "command-2").await.is_err());
    store.revoke(ROOM, ID, 1, "revoke-1").await.unwrap();
    let again = store.install(&spec, 2, "command-2").await.unwrap();
    assert_eq!(again.generation, 3);
    assert!(again.enabled);
}

#[tokio::test]
async fn revoke_is_generation_cas_and_command_idempotent() {
    let store = MemoryInstallations::new();
    store.install(&spec(), 0, "command-1").await.unwrap();
    let revoked = store.revoke(ROOM, ID, 1, "revoke-1").await.unwrap();
    assert_eq!(revoked.generation, 2);
    assert!(!revoked.enabled);
    assert_eq!(
        revoked,
        store.revoke(ROOM, ID, 1, "revoke-1").await.unwrap()
    );
    assert!(store.revoke(ROOM, ID, 1, "revoke-2").await.is_err());
    assert!(store.revoke(OTHER, ID, 2, "revoke-3").await.is_err());
    let active = store.install(&spec(), 2, "command-2").await.unwrap();
    assert_eq!(active.generation, 3);
    assert!(active.enabled);
}

#[tokio::test]
async fn list_is_room_scoped_and_cursor_ordered() {
    let store = MemoryInstallations::new();
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
