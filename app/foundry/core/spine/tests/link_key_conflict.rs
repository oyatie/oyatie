mod catchup_support;
#[path = "migration_support/log.rs"]
mod log;
#[path = "link_submission/support.rs"]
mod support;

use foundry_projection_draft::ProjectionStore;
use foundry_records_draft::{ActionEnvelope, Receipt, RecordsLog, RecordsLogError, SealedEnvelope};
use foundry_spine::{WriteError, submit_through};
use log::MemoryLog;
use support::{Fixture, linking, submission};

fn spent_link() -> Fixture {
    let mut fixture = Fixture::new();
    fixture.seed("ent_source", "ety_reading");
    fixture.seed("ent_target", "ety_reading");
    fixture
        .send(submission(
            "ent_source",
            "spent",
            vec![linking("lty_measures", "ent_target")],
        ))
        .unwrap()
        .mirror
        .unwrap();
    fixture
}

#[test]
fn divergent_spent_link_reaches_the_logs_conflict_without_a_denial() {
    let mut fixture = spent_link();
    let before = fixture.state.clone();
    let entries = fixture.log.replay("ten_test", 1).unwrap();
    let mut denials = MemoryLog::default();
    let result = submit_through(
        submission(
            "ent_source",
            "spent",
            vec![linking("lty_measures", "ent_missing")],
        ),
        &mut fixture.log,
        &mut denials,
        &mut fixture.state,
        &mut fixture.store,
    );
    assert!(
        matches!(result, Err(WriteError::Log(RecordsLogError::IdempotencyConflict {
        tenant_id, idempotency_key,
    })) if tenant_id == "ten_test" && idempotency_key == "spent")
    );
    assert_eq!(denials.head("ten_test").unwrap(), 0);
    assert_eq!(fixture.state, before);
    assert_eq!(fixture.log.replay("ten_test", 1).unwrap(), entries);
    assert_eq!(
        fixture.store.applied_head("ten_test").unwrap(),
        before.applied_ordinal
    );
}

struct AppendUnavailable<'a>(&'a mut MemoryLog);

impl RecordsLog for AppendUnavailable<'_> {
    fn append(&mut self, _: ActionEnvelope) -> Result<Receipt, RecordsLogError> {
        Err(RecordsLogError::Storage {
            detail: "injected_append_storage".into(),
        })
    }

    fn replay(&self, tenant: &str, from: u64) -> Result<Vec<SealedEnvelope>, RecordsLogError> {
        self.0.replay(tenant, from)
    }

    fn head(&self, tenant: &str) -> Result<u64, RecordsLogError> {
        self.0.head(tenant)
    }
}

#[test]
fn divergent_spent_link_preserves_the_logs_append_storage_failure() {
    let mut fixture = spent_link();
    let before = fixture.state.clone();
    let entries = fixture.log.replay("ten_test", 1).unwrap();
    let mut denials = MemoryLog::default();
    let result = submit_through(
        submission(
            "ent_source",
            "spent",
            vec![linking("lty_measures", "ent_missing")],
        ),
        &mut AppendUnavailable(&mut fixture.log),
        &mut denials,
        &mut fixture.state,
        &mut fixture.store,
    );
    assert!(
        matches!(result, Err(WriteError::Log(RecordsLogError::Storage { detail }))
        if detail == "injected_append_storage")
    );
    assert_eq!(denials.head("ten_test").unwrap(), 0);
    assert_eq!(fixture.state, before);
    assert_eq!(fixture.log.replay("ten_test", 1).unwrap(), entries);
    assert_eq!(
        fixture.store.applied_head("ten_test").unwrap(),
        before.applied_ordinal
    );
}
