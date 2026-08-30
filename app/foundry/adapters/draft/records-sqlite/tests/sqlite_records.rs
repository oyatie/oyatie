//! The SQLite records log, held to the port's conformance suite — including
//! the durability clause the in-memory reference could only decline.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use foundry_records_draft::conformance::{
    RecordsFixture, check_append_assigns_dense_per_tenant_ordinals,
    check_conflicting_idempotency_key_reuse_is_refused, check_durability_across_reopen,
    check_head_tracks_the_last_ordinal, check_idempotent_replay_returns_the_original_receipt,
    check_object_sequences_are_dense_per_object, check_replay_is_tenant_isolated,
    check_replay_returns_envelopes_in_order,
};
use foundry_records_draft::{ActionEnvelope, RecordsLog};
use foundry_records_sqlite_draft::SqliteRecordsLog;

struct SqliteFixture {
    path: PathBuf,
    log: SqliteRecordsLog,
}

impl SqliteFixture {
    fn new(case: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "foundry-records-{case}-{}-{}.sqlite",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_file(&path);
        let log = SqliteRecordsLog::open(&path).expect("open a fresh database");
        Self { path, log }
    }
}

impl Drop for SqliteFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

impl RecordsFixture for SqliteFixture {
    type Log = SqliteRecordsLog;

    fn log(&mut self) -> &mut Self::Log {
        &mut self.log
    }

    fn reopen(&mut self) -> bool {
        // The durability clause: drop the connection entirely and come back
        // through the front door. Byte-identical replay is then checked by the
        // suite, not assumed here.
        self.log = SqliteRecordsLog::open(&self.path).expect("reopen the database");
        true
    }
}

type Check = fn(&mut SqliteFixture) -> Result<(), String>;

#[test]
fn sqlite_log_satisfies_every_conformance_check() {
    let checks: [(&str, Check); 8] = [
        (
            "dense ordinals",
            check_append_assigns_dense_per_tenant_ordinals,
        ),
        (
            "object sequences",
            check_object_sequences_are_dense_per_object,
        ),
        (
            "idempotent replay",
            check_idempotent_replay_returns_the_original_receipt,
        ),
        (
            "conflict refusal",
            check_conflicting_idempotency_key_reuse_is_refused,
        ),
        ("ordered replay", check_replay_returns_envelopes_in_order),
        ("tenant isolation", check_replay_is_tenant_isolated),
        ("head tracking", check_head_tracks_the_last_ordinal),
        ("durability", check_durability_across_reopen),
    ];
    for (name, check) in checks {
        let mut fixture = SqliteFixture::new(name.replace(' ', "-").as_str());
        check(&mut fixture).unwrap_or_else(|violation| panic!("{name}: {violation}"));
    }
}

#[test]
fn a_deduplicated_append_survives_reopen_with_the_original_receipt() {
    let mut fixture = SqliteFixture::new("dedup-reopen");
    let envelope =
        ActionEnvelope::new("ten_a", "obj:1", "create", "key-1", 1, b"{}".to_vec(), 5).unwrap();
    let first = fixture.log().append(envelope.clone()).unwrap();
    assert!(fixture.reopen());
    let again = fixture.log().append(envelope).unwrap();
    assert!(again.deduplicated);
    assert_eq!(
        (again.ordinal, again.object_sequence),
        (first.ordinal, first.object_sequence)
    );
    assert_eq!(fixture.log().head("ten_a").unwrap(), 1);
}
