//! The policy enforcement point: this process decides by the checked-in
//! Cedar seed, evaluated by the platform's own engine, or it refuses.
//!
//! The load-bearing test here is the ALLOW path. A deny-only suite would
//! pass just as happily against a PEP that denies everything — including a
//! PEP broken by a context-type mismatch, where the seed compares
//! `context.autonomy_tier <= 1` as a Cedar `Long` while the request carries
//! JSON. If that conversion were lossy the permit would never fire and the
//! process would deny every caller while looking correctly fail-closed.
//! So: one Allow proven end to end, then the denials around it.
//!
//! Operator procedure: a 403 with gate `Authorization` means the PDP
//! returned Deny or refused. `/statusz` reports `loaded_policy_version`;
//! compare it against the bundle you expect. A boot refusal here means the
//! seed did not strict-validate — the process never serves a policy set it
//! could not compile.

use foundry_ontology_app::{Caller, PepError, PolicyEnforcementPoint, Surface};

fn pep() -> PolicyEnforcementPoint {
    PolicyEnforcementPoint::load("psv-000001").expect("the checked-in seed must load")
}

fn operator() -> Caller {
    Caller {
        tenant_id: "ten_acme".into(),
        principal_id: "prn_alice".into(),
        roles: vec!["foundry-operator".into()],
    }
}

#[test]
fn the_seed_loads_and_reports_its_version() {
    let pep = pep();
    assert_eq!(pep.loaded_policy_version().as_str(), "psv-000001");
}

#[test]
fn an_operator_is_allowed_to_invoke_within_its_own_tenant() {
    // THE context-conversion proof: `autonomy_tier` crosses as a JSON
    // number and must arrive at Cedar as a Long, or this permit's
    // `context.autonomy_tier <= 1` can never be satisfied.
    let decision = pep()
        .decide(&operator(), Surface::Invoke, "ent_widget")
        .expect("an in-tenant operator invocation is permitted");
    assert_eq!(decision.tenant_id, "ten_acme");
    assert_eq!(decision.principal_id, "prn_alice");
    assert_eq!(decision.allowed_surfaces, vec!["ops-console".to_owned()]);
    assert!(
        !decision.decision_id.is_empty(),
        "every allow carries its PDP decision id, so the write it authorizes is attributable"
    );
}

#[test]
fn an_operator_is_allowed_to_read_within_its_own_tenant() {
    let decision = pep()
        .decide(&operator(), Surface::Use, "ent_widget")
        .expect("an in-tenant operator read is permitted");
    assert_eq!(decision.allowed_surfaces, vec!["ops-console".to_owned()]);
}

#[test]
fn a_cross_tenant_caller_is_refused_on_both_surfaces() {
    let pep = pep();
    let foreign = Caller {
        tenant_id: "ten_other".into(),
        principal_id: "prn_alice".into(),
        roles: vec!["foundry-operator".into()],
    };
    for surface in [Surface::Invoke, Surface::Use] {
        assert_eq!(
            pep.decide(&foreign, surface, "ent_widget"),
            Err(PepError::Denied),
            "the structural forbid covers every action, not only invocation"
        );
    }
}

#[test]
fn an_unknown_principal_is_denied_by_default() {
    // No role: this process must not invent membership for whoever asks,
    // or the seed's `principal is Principal in Role::"foundry-operator"`
    // clause would be vacuous and the permit would cover the world.
    let stranger = Caller {
        tenant_id: "ten_acme".into(),
        principal_id: "prn_nobody".into(),
        roles: Vec::new(),
    };
    assert_eq!(
        pep().decide(&stranger, Surface::Invoke, "ent_widget"),
        Err(PepError::Denied),
        "a principal outside the operator role is denied by absence of a permit"
    );
}

#[test]
fn a_malformed_bundle_version_refuses_to_load() {
    // A PDP that cannot report which policy version it serves cannot be
    // audited, so an unusable version is a load refusal, not a default.
    assert!(
        matches!(
            PolicyEnforcementPoint::load(""),
            Err(PepError::BundleRejected { .. })
        ),
        "an empty policy version must refuse to load"
    );
}
