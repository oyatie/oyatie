//! `POST /v1/migrations/attest` — what a plan still owes, on the caller's own
//! tenant and no other.
//!
//! `MigrationPlan` carries its own `tenant_id`, so the plan a caller submits
//! names a tenant. The write path already settled this question — "the tenant
//! is the CREDENTIAL's; nothing in the body can move it" — and the same rule
//! has to hold here, because attestation reads a projection: a plan naming
//! another tenant would otherwise report that tenant's pending objects and
//! its poisoned ordinals to a caller who may not read them.
//!
//! It is a READ dressed as a POST — the plan does not fit in a query string
//! — so it is gated on `Use`, not `Invoke`, and it mutates nothing.

mod facade_support;
mod out_of_band;
use facade_support as support;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use data_boundary_kernel::{DataClass, PrivacyDataClass};
use data_ontology_kernel::{
    EntityTypeDefinition, EntityTypeId, EntityTypePropertyDefinition, PropertyTier,
};
use foundry_ontology_app::{AppState, Config, compose};
use support::{Fixture, Session, scrape, value_of};

fn scalar(name: &str, class: PrivacyDataClass, required: bool) -> EntityTypePropertyDefinition {
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
fn state_with_two_revisions(config: &Config) -> AppState {
    let mut state = compose(config).expect("boots");
    let internal = PrivacyDataClass::try_from(DataClass::InternalOnly).expect("a privacy class");
    let engine = &mut state
        .tenants
        .get_mut("ten_acme")
        .expect("the served tenant")
        .get_mut()
        .projection
        .engine;
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
                ],
                2,
            )
            .expect("a definition")
            .with_title_property("name"),
        )
        .expect("the evolution registers");
    state
}

/// POST a plan. The shared harness posts only to `/v1/actions`, and widening
/// it would have touched sixteen call sites in four unrelated files for one
/// new route.
async fn attest(session: &Session, token: Option<&str>, plan: &str) -> (StatusCode, String) {
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
fn plan_for(tenant_id: &str) -> String {
    format!(
        r#"{{"tenant_id":"{tenant_id}","entity_type":"ety_record","from_revision":1,
            "to_revision":2,"action_type":"aty_record_write",
            "audit_event_type":"aet_upcast","declared_at_epoch_seconds":1700000000,
            "transforms":[{{"kind":"copy_as","from":"note","to":"nickname"}}]}}"#
    )
}

#[tokio::test]
async fn an_operator_attests_a_plan_against_their_own_projection() {
    let fixture = Fixture::new("attest-own");
    let session = Session::from_state(state_with_two_revisions(&fixture.config()));

    let (status, body) = attest(
        &session,
        Some(fixture.operator_token()),
        &plan_for("ten_acme"),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(
        body.contains(r#""fixpoint":true"#) && body.contains(r#""pending":[]"#),
        "an empty projection owes nothing: {body}"
    );
    assert!(
        body.contains(r#""poisoned":[]"#),
        "and no poison is laundered out of the answer: {body}"
    );
    let served = scrape(&session).await;
    assert_eq!(
        value_of(&served, "foundry_read_served_total"),
        1,
        "an attestation is a read the process served"
    );
    assert_eq!(
        value_of(&served, "foundry_read_refused_total"),
        0,
        "and it refused nothing on the way"
    );
}

/// The tenant is the credential's. A plan naming another tenant is refused
/// FOR THAT REASON — never attested against the tenant the body names, and
/// never against the caller's own registry under the foreign name.
///
/// The status alone proves nothing: `validate` looks the entity type up under
/// `plan.tenant_id`, so deleting the credential check would still 400, as
/// `UnknownEntityType`, having consulted the caller's registry for a tenant
/// the caller never named. The state is the two-revision one, so the plan is
/// otherwise executable and the credential check is the only thing between
/// this request and a 200. The cause is what says so.
#[tokio::test]
async fn a_plan_naming_another_tenant_does_not_read_that_tenant() {
    let fixture = Fixture::new("attest-foreign-plan");
    let session = Session::from_state(state_with_two_revisions(&fixture.config()));

    let (status, body) = attest(
        &session,
        Some(fixture.operator_token()),
        &plan_for("ten_other"),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(
        value_of(&scrape(&session).await, "foundry_read_refused_total"),
        1,
        "a refusal this surface makes is a refusal the process counts"
    );
    assert!(
        body.contains("the plan names a tenant other than the credential's"),
        "the refusal must be the credential check, not a registry miss under \
         the foreign name: {body}"
    );
}

#[tokio::test]
async fn a_roleless_caller_may_not_attest() {
    let fixture = Fixture::new("attest-roleless");
    let session = fixture.session();

    let (status, body) = attest(
        &session,
        Some(fixture.roleless_token()),
        &plan_for("ten_acme"),
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
}

/// A plan the registry refuses is a typed refusal, not a 500 and not a
/// fixpoint claim over a plan nothing validated.
#[tokio::test]
async fn an_invalid_plan_is_refused_with_its_reason() {
    let fixture = Fixture::new("attest-invalid");
    let session = fixture.session();
    let unknown = plan_for("ten_acme").replace("ety_record", "ety_absent");

    let (status, body) = attest(&session, Some(fixture.operator_token()), &unknown).await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert!(
        body.contains("UnknownEntityType"),
        "the reason must be the registry's own, not a generic refusal: {body}"
    );
    assert!(
        !body.contains(r#""fixpoint""#),
        "a refused plan attests nothing: {body}"
    );
}

/// An object the plan still owes is NAMED, and the fixpoint claim is false.
///
/// The empty-projection case above cannot tell a real attestation from a
/// hardcoded `fixpoint: true` — every honest answer over an empty projection
/// IS true. This writes a record carrying `note` and no `nickname`, exactly
/// what the plan's `copy_as` owes, so the only correct answer is false with
/// that object named.
#[tokio::test]
async fn an_object_owing_the_transform_is_named_pending() {
    let fixture = Fixture::new("attest-pending");
    let session = Session::from_state(state_with_two_revisions(&fixture.config()));
    let (written, reply) = session
        .post(
            Some(fixture.operator_token()),
            r#"{"object_ref":"ent_alpha","action_type":"aty_record_write",
                "idempotency_key":"idem_1","occurred_at_epoch_seconds":1700000000,
                "properties":{"name":"Ada","note":"Countess"}}"#,
        )
        .await;
    assert_eq!(
        written,
        StatusCode::OK,
        "the fixture write must land: {reply}"
    );

    let (status, body) = attest(
        &session,
        Some(fixture.operator_token()),
        &plan_for("ten_acme"),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(
        body.contains(r#""fixpoint":false"#),
        "an object still owed is not a fixpoint: {body}"
    );
    assert!(
        body.contains(r#""pending":["ent_alpha"]"#),
        "and the attestation names what it owes: {body}"
    );
}

/// A conversion this process does not perform is refused BY NAME.
///
/// The status cannot carry this on its own: an unrecognised conversion
/// silently read as `integer_to_string` would also 400, from `validate`,
/// as a kind mismatch against a string source. The cause is what separates
/// "this vocabulary has no such word" from "that word does not fit here".
#[tokio::test]
async fn a_conversion_this_process_does_not_perform_is_refused() {
    let fixture = Fixture::new("attest-unknown-conversion");
    let session = Session::from_state(state_with_two_revisions(&fixture.config()));
    let plan = plan_for("ten_acme").replace(
        r#"{"kind":"copy_as","from":"note","to":"nickname"}"#,
        r#"{"kind":"convert_as","from":"note","to":"nickname","conversion":"note_to_nickname"}"#,
    );

    let (status, body) = attest(&session, Some(fixture.operator_token()), &plan).await;

    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert!(
        body.contains("that conversion is not one this process performs"),
        "an unknown conversion is refused as unknown, not mapped to a known \
         one and refused later for the wrong reason: {body}"
    );
}

/// A poisoned ordinal is REPORTED, not quietly dropped from the answer.
///
/// The empty-projection case reads `"poisoned":[]`, which is also what an
/// attestation that discards poison reports. Bytes the fold refuses are
/// seeded before boot so the projection carries real poison: an operator
/// deciding whether to run a migration is deciding against a projection with
/// a hole in it, and a hole this surface hid is one they would run over.
#[tokio::test]
async fn a_poisoned_ordinal_is_reported_not_hidden() {
    let fixture = Fixture::new("attest-poisoned");
    out_of_band::append_for(&fixture.action_log_path(), "ten_acme", "idem_poison_1");
    let session = Session::from_state(state_with_two_revisions(&fixture.config()));

    let (status, body) = attest(
        &session,
        Some(fixture.operator_token()),
        &plan_for("ten_acme"),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(
        body.contains(r#""poisoned":[1]"#),
        "the refused ordinal is named in the attestation: {body}"
    );
}
