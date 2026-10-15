#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use super::*;
#[test]
fn exact_entry_verification_executes_no_sqlite_mutation() {
    let directory = std::env::temp_dir().join(format!(
        "foundry-verify-entry-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    let mut store = SqliteProjectionStore::open(&directory.join("projection")).unwrap();
    let entry = AppliedEntry {
        tenant_id: "ten_verify".into(),
        ordinal: 1,
        outcome: foundry_projection_draft::EntryOutcome::Poisoned {
            reason: "malformed_payload".into(),
        },
    };
    assert_eq!(
        store.verify_entry(&entry),
        Err(ProjectionStoreError::DivergentReplay { ordinal: 1 })
    );
    store
        .apply(entry.clone(), &KeyDesignations::default())
        .unwrap();
    let changed: u64 = store
        .connection
        .query_row("SELECT total_changes()", [], |row| row.get(0))
        .unwrap();
    let before: Vec<u8> = store
        .connection
        .query_row(
            "SELECT entry_bytes FROM projection_entries WHERE tenant_id='ten_verify'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    store.verify_entry(&entry).unwrap();
    let mut different = entry;
    different.outcome = foundry_projection_draft::EntryOutcome::Poisoned {
        reason: "unknown_revision".into(),
    };
    assert_eq!(
        store.verify_entry(&different),
        Err(ProjectionStoreError::DivergentReplay { ordinal: 1 })
    );
    let after: u64 = store
        .connection
        .query_row("SELECT total_changes()", [], |row| row.get(0))
        .unwrap();
    assert_eq!(after, changed);
    let bytes: Vec<u8> = store
        .connection
        .query_row(
            "SELECT entry_bytes FROM projection_entries WHERE tenant_id='ten_verify'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(bytes, before);
    assert_eq!(store.applied_head("ten_verify").unwrap(), 1);
    drop(store);
    std::fs::remove_dir_all(directory).unwrap();
}
