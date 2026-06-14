// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use messaging_domain::Outbox;
use messaging_file_adapter::{FileOutboxStore, FileOutboxStoreError};

#[test]
fn file_outbox_store_replays_records_and_appends_only_new_suffix() {
    let path = temp_outbox_path("append");
    let store = FileOutboxStore::new(path.clone());
    let mut outbox = Outbox::default();
    let first = outbox
        .publish(
            "ten_alpha".into(),
            "oya.object-graph.entity.upserted.v1".into(),
            "idem-1".into(),
            "ent-1".into(),
        )
        .expect("first record is valid");
    let duplicate = outbox
        .publish(
            "ten_alpha".into(),
            "oya.object-graph.entity.upserted.v1".into(),
            "idem-1".into(),
            "ent-1".into(),
        )
        .expect("duplicate is idempotent");
    assert_eq!(first, duplicate);

    assert_eq!(store.append_outbox(&outbox).expect("initial append"), 1);
    assert_eq!(store.append_outbox(&outbox).expect("idempotent append"), 0);

    let second = outbox
        .publish(
            "ten_alpha".into(),
            "oya.identity.user.upserted.v1".into(),
            "idem-2".into(),
            "usr-1".into(),
        )
        .expect("second record is valid");
    outbox
        .mark_published("ten_alpha", second.sequence)
        .expect("published transition is valid");
    assert_eq!(store.append_outbox(&outbox).expect("suffix append"), 2);

    let restored = store.load().expect("outbox can be replayed");
    assert_eq!(restored.records(), outbox.records());
    assert!(restored.records()[1].published);

    fs::remove_file(path).ok();
}

#[test]
fn file_outbox_store_persists_published_state_transition_as_append_only_event() {
    let path = temp_outbox_path("published-transition");
    let store = FileOutboxStore::new(path.clone());
    let mut outbox = Outbox::default();
    let record = outbox
        .publish(
            "ten_alpha".into(),
            "oya.object-graph.entity.upserted.v1".into(),
            "idem-1".into(),
            "ent-1".into(),
        )
        .expect("record is valid");

    assert_eq!(store.append_outbox(&outbox).expect("initial append"), 1);
    outbox
        .mark_published("ten_alpha", record.sequence)
        .expect("published transition is valid");
    assert_eq!(
        store
            .append_outbox(&outbox)
            .expect("published status event append"),
        1
    );

    let restored = store.load().expect("outbox can be replayed");
    assert_eq!(restored.records(), outbox.records());
    assert!(restored.records()[0].published);

    fs::remove_file(path).ok();
}

#[test]
fn file_outbox_store_rejects_divergent_or_malformed_history() {
    let path = temp_outbox_path("diverge");
    let store = FileOutboxStore::new(path.clone());
    let mut original = Outbox::default();
    original
        .publish(
            "ten_alpha".into(),
            "oya.object-graph.entity.upserted.v1".into(),
            "idem-1".into(),
            "ent-1".into(),
        )
        .expect("record is valid");
    store.append_outbox(&original).expect("initial append");

    let mut divergent = Outbox::default();
    divergent
        .publish(
            "ten_alpha".into(),
            "oya.object-graph.entity.deleted.v1".into(),
            "idem-1".into(),
            "ent-1".into(),
        )
        .expect("record is valid");
    assert_eq!(
        store.append_outbox(&divergent),
        Err(FileOutboxStoreError::OutboxDiverged)
    );

    fs::write(&path, "not-an-outbox-record\n").expect("malform write");
    assert_eq!(store.load(), Err(FileOutboxStoreError::MalformedRecord));

    fs::remove_file(path).ok();
}

#[test]
fn file_outbox_store_rejects_length_prefix_inside_utf8_boundary() {
    let path = temp_outbox_path("utf8-boundary");
    let store = FileOutboxStore::new(path.clone());
    fs::write(&path, "v1|0|1:é|").expect("malformed utf8-boundary record written");

    assert_eq!(store.load(), Err(FileOutboxStoreError::MalformedRecord));

    fs::remove_file(path).ok();
}

fn temp_outbox_path(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "messaging-outbox-{label}-{}-{nanos}.log",
        std::process::id()
    ))
}
