//! The governance seam, end to end: the checked-in Cedar seed
//! strict-validates against the REAL engine, an authorized operator's
//! Allow becomes the kernel's ActionPolicyDecision and the write
//! applies, and every unauthorized shape — wrong tenant, wrong surface,
//! no role — evaluates Deny so the writer refuses at gate 1 and the
//! denial lands on the audit trail. Deny-by-default: no permit, no
//! write.

use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use cedar_policy::{
    Authorizer, Context, Decision, Entities, Entity, EntityUid, PolicySet, Request,
    RestrictedExpression, Schema, ValidationMode, Validator,
};
use data_boundary_kernel::{DataClass, PrivacyDataClass};
use data_ontology_kernel::{
    ActionInvocationRequest, ActionPolicyDecision, ActionTypeDefinition, ActionTypeId,
    AutonomyTier, EntityTypeDefinition, EntityTypeId, EntityTypePropertyDefinition, OntologyEngine,
    PropertyTier,
};
use foundry_edits::{EditSet, OntologyEdit, WireDataClass, WireProperty, WireTier, WireValue};
use foundry_records_draft::{ActionEnvelope, Receipt, RecordsLog, RecordsLogError, SealedEnvelope};
use foundry_spine::{
    ActionSubmission, ApplyOutcome, ProjectionState, RefusalGate, WriteError, submit,
};

const SCHEMA_SRC: &str = include_str!("../../../cedar/foundry.cedarschema");
const POLICIES_SRC: &str = include_str!("../../../cedar/foundry-policies.cedar");

fn schema() -> Schema {
    let (schema, _warnings) =
        Schema::from_cedarschema_str(SCHEMA_SRC).expect("foundry.cedarschema must parse");
    schema
}

fn policy_set() -> PolicySet {
    PolicySet::from_str(POLICIES_SRC).expect("foundry-policies.cedar must parse")
}

fn entity(uid: &str, attrs: &[(&str, &str)], parents: &[&str]) -> Entity {
    let uid = EntityUid::from_str(uid).expect("uid parses");
    let attrs: HashMap<String, RestrictedExpression> = attrs
        .iter()
        .map(|(k, v)| {
            (
                (*k).to_owned(),
                RestrictedExpression::new_string((*v).to_owned()),
            )
        })
        .collect();
    let parents: HashSet<EntityUid> = parents
        .iter()
        .map(|p| EntityUid::from_str(p).expect("parent parses"))
        .collect();
    Entity::new(uid, attrs, parents).expect("entity builds")
}

fn entities() -> Entities {
    Entities::from_entities(
        vec![
            entity("Tenant::\"ten_test\"", &[], &[]),
            entity("Tenant::\"ten_other\"", &[], &[]),
            entity("Role::\"foundry-operator\"", &[], &["Tenant::\"ten_test\""]),
            entity(
                "Principal::\"prn_alice\"",
                &[("tenant", "ten_test")],
                &["Role::\"foundry-operator\""],
            ),
            entity(
                "Principal::\"prn_mallory\"",
                &[("tenant", "ten_other")],
                &[],
            ),
            entity(
                "OntologyObject::\"ent_r1\"",
                &[("tenant", "ten_test")],
                &["Tenant::\"ten_test\""],
            ),
        ],
        Some(&schema()),
    )
    .expect("entities build")
}

/// Evaluate the PDP question for one principal; an Allow becomes the
/// kernel decision, a Deny becomes a decision that covers nothing.
fn evaluate(principal: &str, surface: &str) -> Option<ActionPolicyDecision> {
    let request = Request::new(
        EntityUid::from_str(&format!("Principal::\"{principal}\"")).unwrap(),
        EntityUid::from_str("Action::\"InvokeAction\"").unwrap(),
        EntityUid::from_str("OntologyObject::\"ent_r1\"").unwrap(),
        Context::from_pairs([
            (
                "surface".to_owned(),
                RestrictedExpression::new_string(surface.to_owned()),
            ),
            (
                "autonomy_tier".to_owned(),
                RestrictedExpression::new_long(1),
            ),
        ])
        .unwrap(),
        Some(&schema()),
    )
    .unwrap();
    let response = Authorizer::new().is_authorized(&request, &policy_set(), &entities());
    match response.decision() {
        Decision::Allow => Some(ActionPolicyDecision {
            decision_id: format!("dec_cedar_{principal}"),
            tenant_id: "ten_test".into(),
            principal_id: principal.into(),
            allowed_surfaces: vec![surface.into()],
            autonomy_tier: AutonomyTier::T1Assist,
        }),
        Decision::Deny => None,
    }
}

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

    fn replay(&self, _: &str, _: u64) -> Result<Vec<SealedEnvelope>, RecordsLogError> {
        Ok(self.entries.clone())
    }

    fn head(&self, _: &str) -> Result<u64, RecordsLogError> {
        Ok(self.entries.len() as u64)
    }
}

fn submission(principal: &str, decision: ActionPolicyDecision) -> ActionSubmission {
    ActionSubmission {
        request: ActionInvocationRequest {
            tenant_id: "ten_test".into(),
            principal_id: principal.into(),
            action_id: ActionTypeId::new("aty_calibrate").unwrap(),
            entity_id: "ent_r1".into(),
            idempotency_key: format!("idem_{principal}"),
            requested_at_epoch_seconds: 1_700_000_000,
        },
        decision,
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

/// The operator's Allow becomes a covering decision and the write lands.
#[test]
fn an_authorized_operator_writes_through_the_gate() {
    let decision = evaluate("prn_alice", "ops-console")
        .expect("the operator role on the right surface must Allow");
    let registry = registry();
    let mut log = MemoryLog::default();
    let mut denials = MemoryLog::default();
    let mut projection = ProjectionState::new("ten_test", &registry);
    let outcome = submit(
        submission("prn_alice", decision),
        &mut log,
        &mut denials,
        &mut projection,
    )
    .unwrap();
    assert!(matches!(outcome, ApplyOutcome::Applied { .. }));
    assert_eq!(denials.head("ten_test").unwrap(), 0);
}

/// Every unauthorized shape evaluates Deny; a non-covering decision is
/// refused at gate 1 and the denial lands on the audit trail.
#[test]
fn unauthorized_shapes_deny_and_the_writer_refuses() {
    assert!(
        evaluate("prn_mallory", "ops-console").is_none(),
        "a foreign-tenant principal with no role must Deny",
    );
    assert!(
        evaluate("prn_alice", "grid-console").is_none(),
        "the right principal on an ungranted surface must Deny",
    );

    // Deny means the PDP covers nothing; the kernel gate then refuses.
    let uncovering = ActionPolicyDecision {
        decision_id: "dec_deny".into(),
        tenant_id: "ten_test".into(),
        principal_id: "prn_mallory".into(),
        allowed_surfaces: vec![],
        autonomy_tier: AutonomyTier::T0Suggest,
    };
    let registry = registry();
    let mut log = MemoryLog::default();
    let mut denials = MemoryLog::default();
    let mut projection = ProjectionState::new("ten_test", &registry);
    let WriteError::Refused(refused) = submit(
        submission("prn_mallory", uncovering),
        &mut log,
        &mut denials,
        &mut projection,
    )
    .unwrap_err() else {
        panic!("an uncovered invocation must refuse");
    };
    assert_eq!(refused.gate, RefusalGate::Authorization);
    assert_eq!(log.head("ten_test").unwrap(), 0);
    assert_eq!(
        denials.head("ten_test").unwrap(),
        1,
        "the denial is durable"
    );
}
