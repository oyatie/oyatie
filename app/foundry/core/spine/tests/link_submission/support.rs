use super::{catchup_support::registry, log::MemoryLog};
use data_boundary_kernel::{DataClass, PrivacyDataClass};
use data_ontology_kernel::{
    ActionInvocationRequest, ActionPolicyDecision, ActionTypeDefinition, ActionTypeId,
    AutonomyTier, EntityTypeDefinition, EntityTypeId, EntityTypePropertyDefinition,
    LinkCardinality, LinkTypeDefinition, LinkTypeId, PropertyTier,
};
use foundry_edits::{EditSet, OntologyEdit, WireDataClass, WireProperty, WireTier, WireValue};
use foundry_projection_draft::{MemoryProjectionStore, ProjectionStore};
use foundry_records_draft::RecordsLog;
use foundry_spine::{
    ActionSubmission, ApplyOutcome, Mirrored, ProjectionState, RefusalGate, WriteError,
    submit_through,
};

pub(super) fn create(kind: &str, name: &str) -> OntologyEdit {
    OntologyEdit::create_object(
        kind,
        vec![
            WireProperty::new(
                "name",
                WireTier::Scalar,
                WireDataClass::InternalOnly,
                WireValue::String(name.into()),
            )
            .unwrap(),
        ],
    )
    .unwrap()
}

pub(super) fn linking(kind: &str, target: &str) -> OntologyEdit {
    OntologyEdit::create_link(kind, target).unwrap()
}

pub(super) fn submission(source: &str, key: &str, edits: Vec<OntologyEdit>) -> ActionSubmission {
    ActionSubmission {
        request: ActionInvocationRequest {
            tenant_id: "ten_test".into(),
            principal_id: "prn_alice".into(),
            action_id: ActionTypeId::new("aty_calibrate").unwrap(),
            entity_id: source.into(),
            idempotency_key: key.into(),
            requested_at_epoch_seconds: 1_700_000_000,
        },
        decision: ActionPolicyDecision {
            decision_id: format!("dec_{key}"),
            tenant_id: "ten_test".into(),
            principal_id: "prn_alice".into(),
            allowed_surfaces: vec!["ops-console".into()],
            autonomy_tier: AutonomyTier::T1Assist,
        },
        parameters: vec![],
        edits: EditSet::new(edits).unwrap(),
    }
}

pub(super) struct Fixture {
    pub(super) state: ProjectionState,
    pub(super) log: MemoryLog,
    denials: MemoryLog,
    pub(super) store: MemoryProjectionStore,
}

impl Fixture {
    pub(super) fn new() -> Self {
        let mut registry = registry();
        registry
            .register_entity_type(
                EntityTypeDefinition::new(
                    "ten_test",
                    EntityTypeId::new("ety_other").unwrap(),
                    "Other",
                    vec![
                        EntityTypePropertyDefinition::new(
                            "name",
                            PropertyTier::Scalar,
                            PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap(),
                            true,
                        )
                        .unwrap(),
                    ],
                    1,
                )
                .unwrap(),
            )
            .unwrap();
        registry
            .register_action_type(
                ActionTypeDefinition::new(
                    "ten_test",
                    ActionTypeId::new("aty_other").unwrap(),
                    EntityTypeId::new("ety_other").unwrap(),
                    "ops-console",
                    AutonomyTier::T1Assist,
                    "other.created",
                )
                .unwrap(),
            )
            .unwrap();
        for (id, from, to, cardinality) in [
            (
                "lty_directed",
                "ety_reading",
                "ety_other",
                LinkCardinality::ManyToMany,
            ),
            (
                "lty_from_other",
                "ety_other",
                "ety_reading",
                LinkCardinality::ManyToMany,
            ),
            (
                "lty_single",
                "ety_reading",
                "ety_reading",
                LinkCardinality::OneToOne,
            ),
        ] {
            registry
                .register_link_type(
                    LinkTypeDefinition::new(
                        "ten_test",
                        LinkTypeId::new(id).unwrap(),
                        EntityTypeId::new(from).unwrap(),
                        EntityTypeId::new(to).unwrap(),
                        cardinality,
                        false,
                    )
                    .unwrap(),
                )
                .unwrap();
        }
        Self {
            state: ProjectionState::new("ten_test", &registry),
            log: MemoryLog::default(),
            denials: MemoryLog::default(),
            store: MemoryProjectionStore::default(),
        }
    }

    pub(super) fn send(&mut self, request: ActionSubmission) -> Result<Mirrored, WriteError> {
        submit_through(
            request,
            &mut self.log,
            &mut self.denials,
            &mut self.state,
            &mut self.store,
        )
    }

    pub(super) fn seed(&mut self, reference: &str, kind: &str) {
        let mut request = submission(
            reference,
            &format!("seed_{reference}"),
            vec![create(kind, reference)],
        );
        if kind == "ety_other" {
            request.request.action_id = ActionTypeId::new("aty_other").unwrap();
        }
        let result = self.send(request).unwrap();
        assert!(matches!(result.outcome, ApplyOutcome::Applied { .. }));
        result.mirror.unwrap();
    }

    pub(super) fn refused(&mut self, request: ActionSubmission, cause: &str) {
        let before = self.state.clone();
        let entries = self.log.replay("ten_test", 1).unwrap();
        let head = self.store.applied_head("ten_test").unwrap();
        let links = self.store.links_from("ten_test", "ent_source").unwrap();
        let denials = self.denials.head("ten_test").unwrap();
        let Err(WriteError::Refused(refused)) = self.send(request) else {
            panic!("link must be refused");
        };
        assert_eq!(refused.gate, RefusalGate::Admission);
        assert_eq!(refused.cause, cause);
        assert!(refused.recorded_on_trail);
        assert_eq!(self.state, before);
        assert_eq!(self.log.replay("ten_test", 1).unwrap(), entries);
        assert_eq!(self.store.applied_head("ten_test").unwrap(), head);
        assert_eq!(
            self.store.links_from("ten_test", "ent_source").unwrap(),
            links
        );
        assert_eq!(self.denials.head("ten_test").unwrap(), denials + 1);
    }
}
