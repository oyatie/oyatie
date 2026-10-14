use std::sync::Arc;

use axum::http::StatusCode;
use data_boundary_kernel::{DataClass, PrivacyDataClass};
use data_ontology_kernel::{
    EntityTypeDefinition, EntityTypeId, EntityTypePropertyDefinition, LinkCardinality,
    LinkTypeDefinition, LinkTypeId, ObjectEntity, ObjectProperty, PropertyTier,
};
use foundry_ontology_app::{AppState, PolicyEnforcementPoint};
use foundry_projection_draft::{
    AppliedEntry, EntryOutcome, KeyDesignations, ProjectedLink, ProjectedObject,
};
use serde_json::{Value, json};

use crate::facade_support::{Fixture, Session, TENANT, scrape, value_of};

#[path = "policy.rs"]
mod policy;
pub use policy::EndpointPdp;

pub struct GraphFixture {
    pub fixture: Fixture,
    pub state: Arc<AppState>,
    pub policy: Arc<EndpointPdp>,
    pub session: Session,
}

pub fn object(tenant: &str, reference: &str, entity_type: &str) -> ProjectedObject {
    let privacy = if entity_type == "ety_team" {
        DataClass::PiiIdentifying
    } else {
        DataClass::InternalOnly
    };
    ProjectedObject {
        entity: ObjectEntity::new(
            tenant.into(),
            reference.into(),
            entity_type.into(),
            vec![ObjectProperty::new(
                "name".into(),
                "private fixture title".into(),
                PropertyTier::Scalar,
                PrivacyDataClass::try_from(privacy).unwrap(),
            )],
        )
        .unwrap(),
        schema_revision: 1,
        last_ordinal: 1,
        last_actor: "prn_fixture".into(),
    }
}

pub fn objects() -> Vec<ProjectedObject> {
    vec![
        object(TENANT, "ent_record", "ety_record"),
        object(TENANT, "ent_team", "ety_team"),
    ]
}

pub fn links() -> Vec<ProjectedLink> {
    [
        ("ent_record", "lty_member", "ent_team"),
        ("ent_team", "lty_owns", "ent_record"),
        ("ent_record", "lty_peer", "ent_record"),
    ]
    .into_iter()
    .map(|(from, kind, to)| ProjectedLink {
        link_type: kind.into(),
        from_object_ref: from.into(),
        to_object_ref: to.into(),
        observed_at_epoch_ms: 1700000000000,
    })
    .collect()
}

impl GraphFixture {
    pub fn new(case: &str, objects: Vec<ProjectedObject>, links: Vec<ProjectedLink>) -> Self {
        let fixture = Fixture::new(case);
        let mut state = fixture.state();
        let tenant = state.tenants.get_mut(TENANT).unwrap().get_mut();
        tenant
            .projection
            .engine
            .register_entity_type(
                EntityTypeDefinition::new(
                    TENANT,
                    EntityTypeId::new("ety_team").unwrap(),
                    "Team",
                    vec![
                        EntityTypePropertyDefinition::new(
                            "name",
                            PropertyTier::Scalar,
                            PrivacyDataClass::try_from(DataClass::PiiIdentifying).unwrap(),
                            true,
                        )
                        .unwrap(),
                    ],
                    1,
                )
                .unwrap(),
            )
            .unwrap();
        for (kind, from, to) in [
            ("lty_member", "ety_record", "ety_team"),
            ("lty_owns", "ety_team", "ety_record"),
            ("lty_peer", "ety_record", "ety_record"),
        ] {
            tenant
                .projection
                .engine
                .register_link_type(
                    LinkTypeDefinition::new(
                        TENANT,
                        LinkTypeId::new(kind).unwrap(),
                        EntityTypeId::new(from).unwrap(),
                        EntityTypeId::new(to).unwrap(),
                        LinkCardinality::ManyToMany,
                        true,
                    )
                    .unwrap(),
                )
                .unwrap();
        }
        tenant
            .projection_store
            .apply(
                AppliedEntry {
                    tenant_id: TENANT.into(),
                    ordinal: 1,
                    outcome: EntryOutcome::Applied { objects, links },
                },
                &KeyDesignations::default(),
            )
            .unwrap();
        let policy = EndpointPdp::new(state.pep.loaded_policy_version().as_str());
        state.pep = PolicyEnforcementPoint::with_pdp(policy.clone());
        let state = Arc::new(state);
        let session = Session::from_shared(state.clone());
        Self {
            fixture,
            state,
            policy,
            session,
        }
    }

    pub async fn replace(&self, objects: Vec<ProjectedObject>, links: Vec<ProjectedLink>) {
        let mut tenant = self.state.tenants[TENANT].lock().await;
        tenant.projection_store.reset_tenant(TENANT).unwrap();
        tenant
            .projection_store
            .apply(
                AppliedEntry {
                    tenant_id: TENANT.into(),
                    ordinal: 1,
                    outcome: EntryOutcome::Applied { objects, links },
                },
                &KeyDesignations::default(),
            )
            .unwrap();
    }

    pub async fn walk(&self, request: &Value) -> (StatusCode, Value) {
        let (status, reply) = self
            .session
            .post_to(
                Some(self.fixture.operator_token()),
                "/v1/search-around",
                &request.to_string(),
            )
            .await;
        (status, serde_json::from_str(&reply).unwrap())
    }

    pub async fn counters(&self) -> (u64, u64) {
        let text = scrape(&self.session).await;
        (
            value_of(&text, "foundry_read_served_total"),
            value_of(&text, "foundry_read_refused_total"),
        )
    }
}

pub fn body(key: &str) -> Value {
    json!({"idempotency_key":key,"seed":{"type":"ety_record","revision":1,"set":{"named":["ent_record"]}},"edge_types":[],"max_depth":1,"direction":"both","consent":"unrestricted","freshness_floor_epoch_seconds":0,"observed_at_epoch_seconds":1700000000})
}

pub fn refusal(reply: &Value, gate: &str) {
    assert_eq!(
        reply.as_object().unwrap().len(),
        2,
        "no graph metadata: {reply}"
    );
    assert_eq!(reply["gate"], gate);
    assert!(reply["cause"].is_string());
    for private in [
        "ent_record",
        "ent_team",
        "ety_team",
        "lty_member",
        "private fixture title",
        "prn_alice",
    ] {
        assert!(
            !reply.to_string().contains(private),
            "no identifiers or private values: {reply}"
        );
    }
}
