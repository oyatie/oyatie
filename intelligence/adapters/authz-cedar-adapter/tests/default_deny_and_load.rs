//! D7 — load + default-deny invariants.
//!
//! - The bundled policy must parse without errors.
//! - An empty-policy gate denies everything (Cedar default-deny).
//! - A garbage policy returns a parse error rather than panicking.
use intelligence_authz_cedar_adapter::{CedarAuthzGate, CedarAuthzGateError};
use intelligence_kernel::{
    AgentId, AuthzAction, AuthzDecision, AuthzGate, AuthzRequest, Provider, TenantId,
};

#[test]
fn bundled_policy_parses() {
    let g = CedarAuthzGate::with_default_policy().expect("bundled policy must parse");
    assert!(
        g.policy_count() >= 5,
        "bundled policy must have multiple rules"
    );
}

#[test]
fn empty_policy_denies_everything() {
    let g = CedarAuthzGate::from_policy_text("").expect("empty policy parses to empty set");
    let t = TenantId::new("acme").unwrap();
    let a = AgentId::new("agent-1").unwrap();
    let r = AuthzRequest {
        principal_tenant: &t,
        principal_agent: &a,
        action: AuthzAction::SelectSeat,
        resource_tenant: &t,
        resource_provider: Provider::Anthropic,
    };
    assert_eq!(g.decide(&r), AuthzDecision::Forbid);
}

#[test]
fn garbage_policy_returns_parse_error() {
    let result = CedarAuthzGate::from_policy_text("this is not cedar (");
    match result {
        Err(CedarAuthzGateError::PolicyParse(_)) => {}
        Ok(_) => panic!("expected parse error for malformed policy text"),
    }
}

#[test]
fn forbid_only_policy_denies_everything() {
    let policy = r#"
        forbid (principal, action, resource);
    "#;
    let g = CedarAuthzGate::from_policy_text(policy).unwrap();
    let t = TenantId::new("acme").unwrap();
    let a = AgentId::new("agent-1").unwrap();
    let r = AuthzRequest {
        principal_tenant: &t,
        principal_agent: &a,
        action: AuthzAction::SelectSeat,
        resource_tenant: &t,
        resource_provider: Provider::Anthropic,
    };
    assert_eq!(g.decide(&r), AuthzDecision::Forbid);
}

#[test]
fn forbid_wins_when_permit_also_matches() {
    // Construct a policy where both a permit AND a forbid match the same request.
    // Cedar's forbid-wins rule means Forbid is the final decision.
    let policy = r#"
        permit (principal, action, resource);
        forbid (principal, action, resource);
    "#;
    let g = CedarAuthzGate::from_policy_text(policy).unwrap();
    let t = TenantId::new("acme").unwrap();
    let a = AgentId::new("agent-1").unwrap();
    let r = AuthzRequest {
        principal_tenant: &t,
        principal_agent: &a,
        action: AuthzAction::SelectSeat,
        resource_tenant: &t,
        resource_provider: Provider::Anthropic,
    };
    assert_eq!(g.decide(&r), AuthzDecision::Forbid);
}
