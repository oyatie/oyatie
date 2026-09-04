//! Appending to a tenant's durable log behind the process's back.
//!
//! Shared rather than copied. Two files need it — the lag totals and the
//! readiness refusals — and a duplicated writer is the arrangement where one
//! copy is fixed and the other silently stops producing the state it names,
//! which is the rule `failing_log` states about its own double.
//!
//! This is the cheapest way to DRIVE a projection behind its log; it is not
//! the argument that the state matters. `AppState` declares SQLite
//! single-writer, so the in-contract breach is elsewhere:
//! `append_with_receipt` commits before `apply_sealed` runs, and a panic
//! between them leaves this process permanently one behind for its lifetime
//! with no second writer anywhere.

use foundry_records_draft::{ActionEnvelope, RecordsLog};
use foundry_records_sqlite_draft::SqliteRecordsLog;

/// One entry for `tenant_id`, carrying bytes the fold cannot decode — so it
/// counts as lag while unfolded, and as poison once a boot fold consumes it.
pub fn append_for(action_log: &std::path::Path, tenant_id: &str, key: &str) {
    let mut log = SqliteRecordsLog::open(action_log).expect("open the log");
    log.append(
        ActionEnvelope::new(
            tenant_id,
            "ent_alpha",
            "aty_record_write",
            key,
            1,
            b"these bytes are not a canonical action record".to_vec(),
            1_700_000_000_000,
        )
        .expect("a well-formed envelope"),
    )
    .expect("the append succeeds");
}
