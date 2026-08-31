//! The denial audit trail: refused submissions land as DenialRecords on
//! a SEPARATE audit log — never consuming a tenant object ordinal — with
//! deterministic keys, and an audit-append failure never masks the
//! refusal returned to the caller.

use data_boundary_kernel::{DataClass, PrivacyDataClass};
use data_ontology_kernel::{
    ActionInvocationRequest, ActionPolicyDecision, ActionTypeDefinition, ActionTypeId,
    AutonomyTier, EntityTypeDefinition, EntityTypeId, EntityTypePropertyDefinition, OntologyEngine,
    PropertyTier,
};
use foundry_edits::{
    EditSet, OntologyEdit, WireDataClass, WireProperty, WireTier, WireValue, decode_denial_record,
};
use foundry_records_draft::{ActionEnvelope, Receipt, RecordsLog, RecordsLogError, SealedEnvelope};
use foundry_spine::{ActionSubmission, ProjectionState, RefusalGate, WriteError, submit};

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
        let receipt = Receipt {
            ordinal: self.entries.len() as u64 + 1,
            object_sequence: 1,
            deduplicated: false,
        };
        self.entries.push(SealedEnvelope {
            envelope,
            receipt: receipt.clone(),
        });
        Ok(receipt)
    }

    fn replay(&self, _tenant_id: &str, _from: u64) -> Result<Vec<SealedEnvelope>, RecordsLogError> {
        Ok(self.entries.clone())
    }

    fn head(&self, _tenant_id: &str) -> Result<u64, RecordsLogError> {
        Ok(self.entries.len() as u64)
    }
}

/// An audit sink that always fails — the refusal must survive it.
struct BrokenLog;

impl RecordsLog for BrokenLog {
    fn append(&mut self, _: ActionEnvelope) -> Result<Receipt, RecordsLogError> {
        Err(RecordsLogError::Storage {
            detail: "audit disk gone".into(),
        })
    }

    fn replay(&self, _: &str, _: u64) -> Result<Vec<SealedEnvelope>, RecordsLogError> {
        Err(RecordsLogError::Storage {
            detail: "audit disk gone".into(),
        })
    }

    fn head(&self, _: &str) -> Result<u64, RecordsLogError> {
        Err(RecordsLogError::Storage {
            detail: "audit disk gone".into(),
        })
    }
}

fn unauthorized_submission() -> ActionSubmission {
    ActionSubmission {
        request: ActionInvocationRequest {
            tenant_id: "ten_test".into(),
            principal_id: "prn_mallory".into(),
            action_id: ActionTypeId::new("aty_calibrate").unwrap(),
            entity_id: "ent_r1".into(),
            idempotency_key: "idem_1".into(),
            requested_at_epoch_seconds: 1_700_000_000,
        },
        decision: ActionPolicyDecision {
            decision_id: "dec_1".into(),
            tenant_id: "ten_test".into(),
            principal_id: "prn_mallory".into(),
            allowed_surfaces: vec!["someone-elses-console".into()],
            autonomy_tier: AutonomyTier::T1Assist,
        },
        parameters: vec![],
        edits: EditSet::new(vec![
            OntologyEdit::create_object(
                "ety_reading",
                vec![
                    WireProperty::new(
                        "name",
                        WireTier::Scalar,
                        WireDataClass::InternalOnly,
                        WireValue::String("Ada".into()),
                    )
                    .unwrap(),
                ],
            )
            .unwrap(),
        ])
        .unwrap(),
    }
}

#[test]
fn a_refusal_lands_on_the_audit_log_and_nowhere_else() {
    let registry = registry();
    let mut log = MemoryLog::default();
    let mut denials = MemoryLog::default();
    let mut projection = ProjectionState::new("ten_test", &registry);

    let WriteError::Refused(refused) = submit(
        unauthorized_submission(),
        &mut log,
        &mut denials,
        &mut projection,
    )
    .unwrap_err() else {
        panic!("expected a gate refusal");
    };
    assert_eq!(refused.gate, RefusalGate::Authorization);

    assert_eq!(log.head("ten_test").unwrap(), 0, "no object ordinal spent");
    assert_eq!(
        denials.head("ten_test").unwrap(),
        1,
        "the denial is durable"
    );

    let sealed = &denials.entries[0];
    let record = decode_denial_record(&sealed.envelope.payload).expect("canonical denial bytes");
    assert_eq!(record.gate, "authorization");
    assert_eq!(record.principal_id, "prn_mallory");
    assert_eq!(record.decision_id, "dec_1");
    assert_eq!(record.action_id, "aty_calibrate");
    assert_eq!(record.object_ref, "ent_r1");
    assert_eq!(record.occurred_at_epoch_ms, 1_700_000_000_000);
    assert_eq!(sealed.envelope.object_ref, "ent_r1");
}

#[test]
fn an_identical_refusal_retries_deduplicate_on_the_audit_log() {
    let registry = registry();
    let mut log = MemoryLog::default();
    let mut denials = MemoryLog::default();
    let mut projection = ProjectionState::new("ten_test", &registry);

    for _ in 0..2 {
        let _ = submit(
            unauthorized_submission(),
            &mut log,
            &mut denials,
            &mut projection,
        );
    }
    assert_eq!(
        denials.head("ten_test").unwrap(),
        1,
        "a deterministic key dedups the identical denial",
    );
}

#[test]
fn an_audit_append_failure_never_masks_the_refusal() {
    let registry = registry();
    let mut log = MemoryLog::default();
    let mut denials = BrokenLog;
    let mut projection = ProjectionState::new("ten_test", &registry);

    let error = submit(
        unauthorized_submission(),
        &mut log,
        &mut denials,
        &mut projection,
    )
    .unwrap_err();
    let WriteError::Refused(refused) = error else {
        panic!("the caller must see the refusal, not the audit failure: {error:?}");
    };
    assert_eq!(refused.gate, RefusalGate::Authorization);
    assert_eq!(log.head("ten_test").unwrap(), 0);
}

#[test]
fn an_applied_submission_writes_no_denial() {
    let registry = registry();
    let mut log = MemoryLog::default();
    let mut denials = MemoryLog::default();
    let mut projection = ProjectionState::new("ten_test", &registry);
    let mut authorized = unauthorized_submission();
    authorized.decision.allowed_surfaces = vec!["ops-console".into()];
    submit(authorized, &mut log, &mut denials, &mut projection).unwrap();
    assert_eq!(log.head("ten_test").unwrap(), 1);
    assert_eq!(denials.head("ten_test").unwrap(), 0);
}
