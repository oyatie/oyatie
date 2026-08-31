//! History and audit view pins: who/what/when per object in applied
//! order, and a governance view where poisoned entries appear with
//! their reasons instead of vanishing.

use data_boundary_kernel::{DataClass, PrivacyDataClass};
use data_ontology_kernel::{
    ActionInvocationRequest, ActionPolicyDecision, ActionTypeDefinition, ActionTypeId,
    AutonomyTier, EntityTypeDefinition, EntityTypeId, EntityTypePropertyDefinition, OntologyEngine,
    PropertyTier,
};
use foundry_edits::{
    EditSet, EditTag, OntologyEdit, WireDataClass, WireProperty, WireTier, WireValue,
};
use foundry_records_draft::{ActionEnvelope, Receipt, RecordsLog, RecordsLogError, SealedEnvelope};
use foundry_spine::{
    ActionSubmission, AuditDisposition, PoisonReason, ProjectionState, apply_sealed, audit_view,
    object_history, submit,
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
        let ordinal = self.entries.len() as u64 + 1;
        let object_sequence = self
            .entries
            .iter()
            .filter(|sealed| sealed.envelope.object_ref == envelope.object_ref)
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
        _tenant_id: &str,
        from_ordinal: u64,
    ) -> Result<Vec<SealedEnvelope>, RecordsLogError> {
        Ok(self
            .entries
            .iter()
            .filter(|sealed| sealed.receipt.ordinal >= from_ordinal)
            .cloned()
            .collect())
    }

    fn head(&self, _tenant_id: &str) -> Result<u64, RecordsLogError> {
        Ok(self.entries.len() as u64)
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

fn submission(
    object: &str,
    key: &str,
    principal: &str,
    edits: Vec<OntologyEdit>,
) -> ActionSubmission {
    ActionSubmission {
        request: ActionInvocationRequest {
            tenant_id: "ten_test".into(),
            principal_id: principal.into(),
            action_id: ActionTypeId::new("aty_calibrate").unwrap(),
            entity_id: object.into(),
            idempotency_key: key.into(),
            requested_at_epoch_seconds: 1_700_000_000,
        },
        decision: ActionPolicyDecision {
            decision_id: format!("dec_{key}"),
            tenant_id: "ten_test".into(),
            principal_id: principal.into(),
            allowed_surfaces: vec!["ops-console".into()],
            autonomy_tier: AutonomyTier::T1Assist,
        },
        parameters: vec![],
        edits: EditSet::new(edits).unwrap(),
    }
}

/// Two writers touch one object; a corrupt entry lands between them.
fn populated() -> (ProjectionState, MemoryLog) {
    let registry = registry();
    let mut log = MemoryLog::default();
    let mut projection = ProjectionState::new("ten_test", &registry);
    submit(
        submission(
            "ent_r1",
            "idem_1",
            "prn_alice",
            vec![OntologyEdit::create_object("ety_reading", vec![name_property("Ada")]).unwrap()],
        ),
        &mut log,
        &mut projection,
    )
    .unwrap();

    let corrupt = ActionEnvelope::new(
        "ten_test",
        "ent_r1",
        "aty_calibrate",
        "idem_2",
        1,
        vec![0xFF],
        1_700_000_000_000,
    )
    .unwrap();
    let receipt = log.append(corrupt.clone()).unwrap();
    apply_sealed(
        &mut projection,
        &SealedEnvelope {
            envelope: corrupt,
            receipt,
        },
    );

    // A DECODABLE poisoned entry: valid payload bytes whose embedded key
    // disagrees with the envelope -> ReceiptMismatch, attribution intact.
    let mismatched = foundry_edits::ActionRecord::new(
        "prn_mallory",
        "dec_x",
        "reading.calibrated",
        "idem_other",
        1_700_000_000_000,
        vec![],
        EditSet::new(vec![
            OntologyEdit::upsert_properties(vec![name_property("Forged")]).unwrap(),
        ])
        .unwrap(),
    )
    .unwrap();
    let forged = ActionEnvelope::new(
        "ten_test",
        "ent_r1",
        "aty_calibrate",
        "idem_forged",
        1,
        foundry_edits::encode_action_record(&mismatched),
        1_700_000_000_000,
    )
    .unwrap();
    let receipt = log.append(forged.clone()).unwrap();
    apply_sealed(
        &mut projection,
        &SealedEnvelope {
            envelope: forged,
            receipt,
        },
    );

    submit(
        submission(
            "ent_r1",
            "idem_3",
            "prn_grace",
            vec![OntologyEdit::upsert_properties(vec![name_property("Renamed")]).unwrap()],
        ),
        &mut log,
        &mut projection,
    )
    .unwrap();
    (projection, log)
}

#[test]
fn object_history_attributes_each_applied_entry() {
    let (projection, log) = populated();
    let entries = log.replay("ten_test", 1).unwrap();
    let history = object_history(&projection, &entries, "ent_r1");
    assert_eq!(history.len(), 2, "poisoned entries never appear as history");

    assert_eq!(history[0].ordinal, 1);
    assert_eq!(history[0].principal_id, "prn_alice");
    assert_eq!(history[0].decision_id, "dec_idem_1");
    assert_eq!(history[0].audit_event_type, "reading.calibrated");
    assert_eq!(history[0].occurred_at_epoch_ms, 1_700_000_000_000);
    assert_eq!(history[0].schema_revision, 1);
    assert_eq!(history[0].edits, vec![EditTag::CreateObject]);

    assert_eq!(history[1].ordinal, 4);
    assert_eq!(history[1].principal_id, "prn_grace");
    assert_eq!(history[1].edits, vec![EditTag::UpsertProperties]);

    assert!(object_history(&projection, &entries, "ent_unknown").is_empty());
}

#[test]
fn audit_view_reports_poisons_instead_of_hiding_them() {
    let (projection, log) = populated();
    let entries = log.replay("ten_test", 1).unwrap();
    let view = audit_view(&projection, &entries);
    assert_eq!(view.len(), 4, "every consumed entry appears");

    assert_eq!(view[0].ordinal, 1);
    assert_eq!(view[0].disposition, AuditDisposition::Applied);
    assert_eq!(view[0].principal_id.as_deref(), Some("prn_alice"));

    assert_eq!(view[1].ordinal, 2);
    assert!(matches!(
        view[1].disposition,
        AuditDisposition::Poisoned(PoisonReason::Decode(_))
    ));
    assert_eq!(
        view[1].principal_id, None,
        "undecodable payloads carry no attribution, and say so",
    );
    assert_eq!(view[1].idempotency_key, "idem_2");

    assert_eq!(view[2].ordinal, 3);
    assert_eq!(
        view[2].disposition,
        AuditDisposition::Poisoned(PoisonReason::ReceiptMismatch)
    );
    assert_eq!(
        view[2].principal_id.as_deref(),
        Some("prn_mallory"),
        "a decodable poisoned entry keeps its attribution",
    );

    assert_eq!(view[3].ordinal, 4);
    assert_eq!(view[3].disposition, AuditDisposition::Applied);
    assert_eq!(view[3].principal_id.as_deref(), Some("prn_grace"));
}
