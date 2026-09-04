//! A durable log that answers every call with a STORAGE fault.
//!
//! Shared because two lanes need the same fault on opposite paths — the write
//! path's `append` and the read path's `replay` — and two copies of a
//! fault-injection double is the arrangement where one gets fixed and the
//! other silently stops injecting the fault it names.
//!
//! `Storage` is the adapter-level I/O and corruption variant, not the
//! caller's idempotency conflict. Installable only because the tenant holds
//! its log behind the port; producing it from a real SQLite handle would mean
//! vandalising the database from a second connection and a `rusqlite`
//! dependency in this crate.

use foundry_ontology_app::{AppState, compose};
use foundry_records_draft::{ActionEnvelope, Receipt, RecordsLog, RecordsLogError, SealedEnvelope};

pub struct AlwaysFailingLog {
    pub detail: &'static str,
}

impl AlwaysFailingLog {
    fn fault(&self) -> RecordsLogError {
        RecordsLogError::Storage {
            detail: self.detail.to_owned(),
        }
    }
}

impl RecordsLog for AlwaysFailingLog {
    fn append(&mut self, _envelope: ActionEnvelope) -> Result<Receipt, RecordsLogError> {
        Err(self.fault())
    }

    fn replay(
        &self,
        _tenant_id: &str,
        _from_ordinal: u64,
    ) -> Result<Vec<SealedEnvelope>, RecordsLogError> {
        Err(self.fault())
    }

    fn head(&self, _tenant_id: &str) -> Result<u64, RecordsLogError> {
        Err(self.fault())
    }
}

/// Boot a process normally, then replace every tenant's action log with one
/// that fails. Composing first is deliberate: boot must succeed exactly as it
/// does in production, so the fault is a runtime one rather than a
/// configuration the process would have refused.
pub fn state_with_a_failing_log(
    config: &foundry_ontology_app::Config,
    detail: &'static str,
) -> AppState {
    let mut state = compose(config).expect("boots");
    for tenant in state.tenants.values_mut() {
        tenant.get_mut().action_log = Box::new(AlwaysFailingLog { detail });
    }
    state
}
