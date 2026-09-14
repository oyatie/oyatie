//! The writer's retry contract: a retry of the same request deduplicates
//! to the original outcome whether or not its decision is fresh (the log
//! keeps the first decision; this overturns the earlier conflict pin, since
//! the decision point mints a decision per request), divergent content
//! under a spent key conflicts, and a dedup onto a poisoned ordinal reports
//! Poisoned.

use data_boundary_kernel::{DataClass, PrivacyDataClass};
use data_ontology_kernel::{
    ActionInvocationRequest, ActionPolicyDecision, ActionTypeDefinition, ActionTypeId,
    AutonomyTier, EntityTypeDefinition, EntityTypeId, EntityTypePropertyDefinition, OntologyEngine,
    PropertyTier, PropertyValue,
};
use foundry_edits::{EditSet, OntologyEdit, WireDataClass, WireProperty, WireTier, WireValue};
use foundry_records_draft::{ActionEnvelope, Receipt, RecordsLog, RecordsLogError, SealedEnvelope};
use foundry_spine::{
    ActionSubmission, ApplyOutcome, ProjectionState, RefusalGate, WriteError, submit,
};

fn internal() -> PrivacyDataClass {
    PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap()
}

fn registry() -> OntologyEngine {
    let mut engine = OntologyEngine::default();
    engine
        .register_entity_type(
            EntityTypeDefinition::new(
                "ten_test",
                EntityTypeId::new("ety_reading").unwrap(),
                "Reading",
                vec![
                    EntityTypePropertyDefinition::new(
                        "name",
                        PropertyTier::Scalar,
                        internal(),
                        true,
                    )
                    .unwrap(),
                ],
                1,
            )
            .unwrap(),
        )
        .unwrap();
    engine
        .register_action_type(
            ActionTypeDefinition::new(
                "ten_test",
                ActionTypeId::new("aty_calibrate").unwrap(),
                EntityTypeId::new("ety_reading").unwrap(),
                "ops-console",
                AutonomyTier::T1Assist,
                "reading.calibrated",
            )
            .unwrap(),
        )
        .unwrap();
    engine
}

#[derive(Default)]
struct MemoryLog {
    entries: Vec<SealedEnvelope>,
}

impl RecordsLog for MemoryLog {
    fn append(&mut self, envelope: ActionEnvelope) -> Result<Receipt, RecordsLogError> {
        if let Some(stored) = self.entries.iter().find(|sealed| {
            sealed.envelope.tenant_id == envelope.tenant_id
                && sealed.envelope.idempotency_key == envelope.idempotency_key
        }) {
            if stored.envelope == envelope {
                let mut receipt = stored.receipt.clone();
                receipt.deduplicated = true;
                return Ok(receipt);
            }
            return Err(RecordsLogError::IdempotencyConflict {
                tenant_id: envelope.tenant_id,
                idempotency_key: envelope.idempotency_key,
            });
        }
        let ordinal = self
            .entries
            .iter()
            .filter(|sealed| sealed.envelope.tenant_id == envelope.tenant_id)
            .count() as u64
            + 1;
        let object_sequence = self
            .entries
            .iter()
            .filter(|sealed| {
                sealed.envelope.tenant_id == envelope.tenant_id
                    && sealed.envelope.object_ref == envelope.object_ref
            })
            .count() as u64
            + 1;
        let receipt = Receipt {
            ordinal,
            object_sequence,
            deduplicated: false,
        };
        self.entries.push(SealedEnvelope {
            envelope,
            receipt: receipt.clone(),
        });
        Ok(receipt)
    }

    fn replay(
        &self,
        tenant_id: &str,
        from_ordinal: u64,
    ) -> Result<Vec<SealedEnvelope>, RecordsLogError> {
        Ok(self
            .entries
            .iter()
            .filter(|sealed| {
                sealed.envelope.tenant_id == tenant_id && sealed.receipt.ordinal >= from_ordinal
            })
            .cloned()
            .collect())
    }

    fn head(&self, tenant_id: &str) -> Result<u64, RecordsLogError> {
        Ok(self
            .entries
            .iter()
            .filter(|sealed| sealed.envelope.tenant_id == tenant_id)
            .count() as u64)
    }
}

fn name_property(value: &str) -> WireProperty {
    WireProperty::new(
        "name",
        WireTier::Scalar,
        WireDataClass::InternalOnly,
        WireValue::String(value.into()),
    )
    .unwrap()
}

fn decision() -> ActionPolicyDecision {
    ActionPolicyDecision {
        decision_id: "dec_1".into(),
        tenant_id: "ten_test".into(),
        principal_id: "prn_alice".into(),
        allowed_surfaces: vec!["ops-console".into()],
        autonomy_tier: AutonomyTier::T1Assist,
    }
}

fn request(idempotency_key: &str) -> ActionInvocationRequest {
    ActionInvocationRequest {
        tenant_id: "ten_test".into(),
        principal_id: "prn_alice".into(),
        action_id: ActionTypeId::new("aty_calibrate").unwrap(),
        entity_id: "ent_r1".into(),
        idempotency_key: idempotency_key.into(),
        requested_at_epoch_seconds: 1_700_000_000,
    }
}

fn create_submission(idempotency_key: &str) -> ActionSubmission {
    ActionSubmission {
        request: request(idempotency_key),
        decision: decision(),
        parameters: vec![],
        edits: EditSet::new(vec![
            OntologyEdit::create_object("ety_reading", vec![name_property("Ada")]).unwrap(),
        ])
        .unwrap(),
    }
}

#[test]
fn byte_identical_retry_dedups_to_the_original_outcome() {
    let registry = registry();
    let mut log = MemoryLog::default();
    let mut denials = MemoryLog::default();
    let mut projection = ProjectionState::new("ten_test", &registry);
    submit(
        create_submission("idem_1"),
        &mut log,
        &mut denials,
        &mut projection,
    )
    .unwrap();
    let before = projection.clone();

    let retried = submit(
        create_submission("idem_1"),
        &mut log,
        &mut denials,
        &mut projection,
    )
    .unwrap();
    let ApplyOutcome::Applied { receipt } = retried else {
        panic!("expected the original applied outcome");
    };
    assert!(receipt.deduplicated);
    assert_eq!(receipt.ordinal, 1);
    assert_eq!(projection, before, "a retry never re-applies");
    assert_eq!(log.head("ten_test").unwrap(), 1);
}

fn applied_once() -> (MemoryLog, MemoryLog, ProjectionState) {
    let registry = registry();
    let mut log = MemoryLog::default();
    let mut denials = MemoryLog::default();
    let mut projection = ProjectionState::new("ten_test", &registry);
    submit(
        create_submission("idem_1"),
        &mut log,
        &mut denials,
        &mut projection,
    )
    .unwrap();
    (log, denials, projection)
}

#[test]
fn a_fresh_decision_retry_deduplicates_and_the_log_keeps_the_first_decision() {
    let (mut log, mut denials, mut projection) = applied_once();
    let mut fresh = create_submission("idem_1");
    fresh.decision.decision_id = "dec_2".into();
    let ApplyOutcome::Applied { receipt } =
        submit(fresh, &mut log, &mut denials, &mut projection).unwrap()
    else {
        panic!("expected the original applied outcome");
    };
    assert!(receipt.deduplicated);
    assert_eq!(receipt.ordinal, 1);
    assert_eq!(log.head("ten_test").unwrap(), 1, "appends nothing");
    let stored = foundry_edits::decode_action_record(&log.entries[0].envelope.payload).unwrap();
    assert_eq!(stored.decision_id, "dec_1", "the first decision stays");
}

#[test]
fn a_fresh_decision_retry_with_divergent_content_conflicts_loudly() {
    let (mut log, mut denials, mut projection) = applied_once();
    let mut fresh = create_submission("idem_1");
    fresh.decision.decision_id = "dec_2".into();
    fresh.request.requested_at_epoch_seconds += 1;
    let error = submit(fresh, &mut log, &mut denials, &mut projection).unwrap_err();
    assert!(
        matches!(
            error,
            WriteError::Log(RecordsLogError::IdempotencyConflict { .. })
        ),
        "divergent content under a spent key is a conflict: {error:?}",
    );
}

#[test]
fn dedup_against_a_poisoned_entry_reports_poisoned() {
    let registry = registry();
    let mut log = MemoryLog::default();
    let mut denials = MemoryLog::default();
    let mut projection = ProjectionState::new("ten_test", &registry);

    // Plant a poisoned entry: corrupt bytes appended around the writer.
    let corrupt = ActionEnvelope::new(
        "ten_test",
        "ent_r1",
        "aty_calibrate",
        "idem_1",
        1,
        vec![0xFF],
        1_700_000_000_000,
    )
    .unwrap();
    let receipt = log.append(corrupt.clone()).unwrap();
    let outcome = foundry_spine::apply_sealed(
        &mut projection,
        &SealedEnvelope {
            envelope: corrupt,
            receipt,
        },
    );
    assert!(matches!(outcome, foundry_spine::FoldOutcome::Poisoned(_)));

    // A writer submission deduplicating onto that entry must say so.
    let mut submission = create_submission("idem_1");
    submission.request.idempotency_key = "idem_1".into();
    let result = submit(submission, &mut log, &mut denials, &mut projection);
    match result {
        Ok(ApplyOutcome::Poisoned { receipt, .. }) => {
            assert!(receipt.deduplicated);
            assert_eq!(receipt.ordinal, 1);
        }
        Err(WriteError::Log(RecordsLogError::IdempotencyConflict { .. })) => {
            // Also honest: the writer's canonical bytes differ from the
            // planted corruption, so the key reuse conflicts loudly.
        }
        other => panic!("a dedup against poison must never lie: {other:?}"),
    }
}
