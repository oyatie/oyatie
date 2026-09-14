//! Each field in this table is a comparison `same_request` makes: a stored
//! envelope that differs from the retry in exactly that field is divergent
//! content under a spent key, and the log refuses it. The table plants the
//! stored variant directly, so each row isolates one comparison. Without a
//! row, redundant by construction: the envelope's tenant and idempotency
//! key, which the `spent` lookup fixes, and the record's wire format
//! version, which the decoder refuses to vary.

#[path = "migration_support/log.rs"]
mod log;

use data_boundary_kernel::{DataClass, PrivacyDataClass};
use data_ontology_kernel::{
    ActionInvocationRequest, ActionPolicyDecision, ActionTypeDefinition, ActionTypeId,
    AutonomyTier, EntityTypeDefinition, EntityTypeId, EntityTypePropertyDefinition, OntologyEngine,
    PropertyTier,
};
use foundry_edits::{
    ActionRecord, EditSet, OntologyEdit, WireDataClass, WireProperty, WireTier, WireValue,
    decode_action_record, encode_action_record,
};
use foundry_records_draft::{RecordsLog, RecordsLogError, SealedEnvelope};
use foundry_spine::{ActionSubmission, ApplyOutcome, ProjectionState, WriteError, submit};
use log::MemoryLog;

fn registry() -> OntologyEngine {
    let internal = PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap();
    let mut engine = OntologyEngine::default();
    engine
        .register_entity_type(
            EntityTypeDefinition::new(
                "ten_test",
                EntityTypeId::new("ety_reading").unwrap(),
                "Reading",
                vec![
                    EntityTypePropertyDefinition::new("name", PropertyTier::Scalar, internal, true)
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

fn wire(name: &str, value: &str) -> WireProperty {
    WireProperty::new(
        name,
        WireTier::Scalar,
        WireDataClass::InternalOnly,
        WireValue::String(value.into()),
    )
    .unwrap()
}

fn submission(decision_id: &str) -> ActionSubmission {
    ActionSubmission {
        request: ActionInvocationRequest {
            tenant_id: "ten_test".into(),
            principal_id: "prn_alice".into(),
            action_id: ActionTypeId::new("aty_calibrate").unwrap(),
            entity_id: "ent_r1".into(),
            idempotency_key: "idem_1".into(),
            requested_at_epoch_seconds: 1_700_000_000,
        },
        decision: ActionPolicyDecision {
            decision_id: decision_id.into(),
            tenant_id: "ten_test".into(),
            principal_id: "prn_alice".into(),
            allowed_surfaces: vec!["ops-console".into()],
            autonomy_tier: AutonomyTier::T1Assist,
        },
        parameters: vec![],
        edits: EditSet::new(vec![
            OntologyEdit::create_object("ety_reading", vec![wire("name", "Ada")]).unwrap(),
        ])
        .unwrap(),
    }
}

/// The envelope the first submission stored.
fn stored_original() -> SealedEnvelope {
    let registry = registry();
    let mut log = MemoryLog::default();
    let mut denials = MemoryLog::default();
    let mut projection = ProjectionState::new("ten_test", &registry);
    submit(submission("dec_1"), &mut log, &mut denials, &mut projection).unwrap();
    log.replay("ten_test", 1).unwrap().remove(0)
}

fn with_record(sealed: &SealedEnvelope, edit: impl FnOnce(&mut ActionRecord)) -> SealedEnvelope {
    let mut record = decode_action_record(&sealed.envelope.payload).unwrap();
    edit(&mut record);
    let mut out = sealed.clone();
    out.envelope.payload = encode_action_record(&record);
    out
}

/// Retry `dec_2` against a log holding `stored` under the same key.
fn retry_against(stored: SealedEnvelope) -> Result<ApplyOutcome, WriteError> {
    let registry = registry();
    let mut log = MemoryLog::default();
    log.seed(stored);
    let mut denials = MemoryLog::default();
    let mut projection = ProjectionState::new("ten_test", &registry);
    submit(submission("dec_2"), &mut log, &mut denials, &mut projection)
}

#[test]
fn the_unmodified_original_deduplicates_the_fresh_decision_retry() {
    let outcome = retry_against(stored_original()).unwrap();
    let ApplyOutcome::Applied { receipt } = outcome else {
        panic!("expected the original applied outcome: {outcome:?}");
    };
    assert!(receipt.deduplicated, "the control row is a dedup");
}

#[test]
fn a_stored_envelope_differing_in_any_compared_field_conflicts() {
    let original = stored_original();
    let variants: Vec<(&str, SealedEnvelope)> = vec![
        ("object_ref", {
            let mut v = original.clone();
            v.envelope.object_ref = "ent_other".into();
            v
        }),
        ("action_type", {
            let mut v = original.clone();
            v.envelope.action_type = "aty_other".into();
            v
        }),
        ("schema_revision", {
            let mut v = original.clone();
            v.envelope.schema_revision += 1;
            v
        }),
        ("observed_at", {
            let mut v = original.clone();
            v.envelope.observed_at_epoch_ms += 1;
            v
        }),
        (
            "principal_id",
            with_record(&original, |r| r.principal_id = "prn_mallory".into()),
        ),
        (
            "audit_event_type",
            with_record(&original, |r| r.audit_event_type = "other".into()),
        ),
        (
            "occurred_at",
            with_record(&original, |r| r.occurred_at_epoch_ms += 1),
        ),
        (
            "record idempotency_key",
            with_record(&original, |r| r.idempotency_key = "idem_other".into()),
        ),
        (
            "parameters",
            with_record(&original, |r| r.parameters.push(wire("extra", "x"))),
        ),
        (
            "edits",
            with_record(&original, |r| {
                r.edits = EditSet::new(vec![
                    OntologyEdit::create_object("ety_reading", vec![wire("name", "Bob")]).unwrap(),
                ])
                .unwrap();
            }),
        ),
        ("undecodable payload", {
            let mut v = original.clone();
            v.envelope.payload = b"not a record".to_vec();
            v
        }),
    ];
    assert_eq!(variants.len(), 11);
    for (field, stored) in variants {
        let error = retry_against(stored).expect_err(field);
        assert!(
            matches!(
                error,
                WriteError::Log(RecordsLogError::IdempotencyConflict { .. })
            ),
            "{field}: a stored envelope differing here is divergent content, got {error:?}"
        );
    }
}
