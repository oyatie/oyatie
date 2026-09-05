//! Shared scaffolding for the migration surfaces.
//!
//! Lifted out of the first suite because the file reached the 300-line budget
//! with the tests the review asked for still unwritten, and a second suite
//! needs the same registry evolution and the same POST helper.

use crate::facade_support::{Fixture, Session};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use data_boundary_kernel::{DataClass, PrivacyDataClass};
use data_ontology_kernel::{
    EntityTypeDefinition, EntityTypeId, EntityTypePropertyDefinition, PropertyTier, ScalarType,
    ValueTypeDeclaration,
};
use foundry_ontology_app::{AppState, Config, compose};

/// A property with a DECLARED scalar type.
///
/// An untyped declaration carries the legacy String contract, under which
/// every non-string default is incompatible and they all refuse ALIKE — so an
/// untyped target cannot tell one wire default variant from another. A typed
/// one can: `check_transform` compares the declared scalar against
/// `DefaultValue::scalar_type()`, which is a function of the VARIANT, not of
/// the value it carries.
pub(crate) fn scalar_of_type(
    name: &str,
    class: PrivacyDataClass,
    scalar_type: ScalarType,
) -> EntityTypePropertyDefinition {
    let mut property = scalar(name, class, false);
    property.value_type = Some(ValueTypeDeclaration::Scalar(scalar_type));
    property
}

pub(crate) fn scalar(
    name: &str,
    class: PrivacyDataClass,
    required: bool,
) -> EntityTypePropertyDefinition {
    EntityTypePropertyDefinition::new(name, PropertyTier::Scalar, class, required)
        .expect("a property")
}

/// Boot, then evolve the registry to a second revision of `ety_record`.
///
/// No plan can validate against the shipped seed: it registers one revision,
/// and `MigrationPlan::validate` requires `from < to` AND `head == to`, so
/// the only candidate spans revision 1 to itself. The seed is charter-bound
/// not to grow — "the seam it will replace must not quietly grow into it" —
/// so the second revision is a TEST fact, registered on a composed engine
/// the same way an evolution lane would land it.
pub(crate) fn state_with_two_revisions(config: &Config) -> AppState {
    let mut state = compose(config).expect("boots");
    let internal = PrivacyDataClass::try_from(DataClass::InternalOnly).expect("a privacy class");
    let projection = &mut state
        .tenants
        .get_mut("ten_acme")
        .expect("the served tenant")
        .get_mut()
        .projection;
    // BOTH engines. `registry_input` is the untouched fold input the runner
    // validates against and the writer stamps from; `engine` is that seed plus
    // link instances. A real evolution re-folds, seeding both from one
    // registry, so evolving only `engine` builds a state production cannot
    // reach — and it is precisely the state in which a surface reading the
    // wrong one looks correct.
    for engine in [&mut projection.registry_input, &mut projection.engine] {
        engine
            .evolve_entity_type(
                EntityTypeDefinition::new(
                    "ten_acme",
                    EntityTypeId::new("ety_record").expect("a type id"),
                    "Record",
                    vec![
                        scalar("name", internal, true),
                        scalar("note", internal, false),
                        scalar("nickname", internal, false),
                        scalar_of_type("counter", internal, ScalarType::Integer),
                    ],
                    2,
                )
                .expect("a definition")
                .with_title_property("name"),
            )
            .expect("the evolution registers");
    }
    state
}

/// POST a plan. The shared harness posts only to `/v1/actions`, and widening
/// it would have touched sixteen call sites in four unrelated files for one
/// new route.
pub(crate) async fn attest(
    session: &Session,
    token: Option<&str>,
    plan: &str,
) -> (StatusCode, String) {
    let mut request = Request::builder()
        .method("POST")
        .uri("/v1/migrations/attest")
        .header("content-type", "application/json");
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    session
        .send(
            request
                .body(Body::from(plan.to_owned()))
                .expect("a request"),
        )
        .await
}

/// A plan whose `tenant_id` names the caller's own tenant.
pub(crate) fn plan_for(tenant_id: &str) -> String {
    format!(
        r#"{{"tenant_id":"{tenant_id}","entity_type":"ety_record","from_revision":1,
            "to_revision":2,"action_type":"aty_record_write",
            "audit_event_type":"aet_upcast","declared_at_epoch_seconds":1700000000,
            "transforms":[{{"kind":"copy_as","from":"note","to":"nickname"}}]}}"#
    )
}

/// A state in which the two registries DISAGREE: only `engine` is evolved,
/// `registry_input` left at revision 1.
///
/// Production cannot reach this — a fold seeds both from one registry and
/// nothing outside tests evolves a live engine — and that is the point. It is
/// the one state that tells apart a surface validating against the runner's
/// admission input from one validating against the seed plus link instances,
/// and a fixture that evolves both cannot see the difference at all.
pub(crate) fn state_with_engine_only_evolved(config: &Config) -> AppState {
    let mut state = state_with_two_revisions(config);
    let projection = &mut state
        .tenants
        .get_mut("ten_acme")
        .expect("the served tenant")
        .get_mut()
        .projection;
    projection.registry_input = compose(config)
        .expect("boots")
        .tenants
        .get_mut("ten_acme")
        .expect("the served tenant")
        .get_mut()
        .projection
        .registry_input
        .clone();
    state
}
