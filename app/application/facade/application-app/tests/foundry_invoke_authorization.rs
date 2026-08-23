// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod support;

use application_app::{
    AutonomyTier, CapabilityAction, CapabilityInvocationPrincipal, CapabilityInvocationRequest,
    CapabilityRegistration, CostBudgetRegistration, DataClass, Foundation, FoundationError,
    IdentityRegistration, PolicyEffect, PolicyRuleInput, PolicyScope, PolicyVersion, Purpose,
    RunDisposition, RunState, SubjectClass, TenantCapabilityGrant, TenantRegistration,
};

#[test]
fn capability_invocation_requires_cedar_allow_policy_before_budget_or_execution() {
    let mut foundation = foundation_with_capability();

    let denied = foundation
        .invoke_capability_as_principal(
            support::principal("ten_invoke_authz", "usr_operator", AutonomyTier::T2Advisory),
            invocation(),
        )
        .expect_err("missing Cedar allow policy must fail closed");

    assert_eq!(denied, FoundationError::CapabilityInvocationUnauthorized);
    let run = foundation
        .foundry_runs()
        .last()
        .expect("authorization denial records a run");
    assert_eq!(run.state.value, RunState::RejectedPolicy);
    assert_eq!(
        run.disposition.value,
        Some(RunDisposition::FailureAuthorization)
    );
    let evidence = foundation
        .foundry_evidence_chain()
        .records()
        .last()
        .expect("authorization denial records evidence");
    assert_eq!(
        evidence.fields.value.get("reason").map(String::as_str),
        Some("authorization")
    );
    assert_eq!(foundation.foundry_steps().len(), 0);

    support::allow_capability_invocation(&mut foundation, "ten_invoke_authz", "tenant-admin");
    let receipt = foundation
        .invoke_capability_as_principal(
            support::principal("ten_invoke_authz", "usr_operator", AutonomyTier::T2Advisory),
            invocation(),
        )
        .expect("Cedar allow policy unlocks the normal invocation path");

    assert_eq!(receipt.capability_id, "cap.demo.authz");
    assert!(receipt.run_id.is_some());
    assert!(foundation.audit_chain().verify());
}

#[test]
fn capability_invocation_principal_mismatch_records_rejected_policy_run() {
    let mut foundation = foundation_with_capability();
    support::allow_capability_invocation(&mut foundation, "ten_invoke_authz", "tenant-admin");
    foundation
        .upsert_identity(IdentityRegistration {
            tenant_id: "ten_invoke_authz".into(),
            user_id: "usr_impersonator".into(),
            primary_identifier: "impersonator@example.test".into(),
            display_name: "Impersonator".into(),
            roles: vec!["tenant-admin".into()],
        })
        .unwrap();

    let denied = foundation
        .invoke_capability_as_principal(
            CapabilityInvocationPrincipal {
                tenant_id: "ten_invoke_authz".into(),
                user_id: "usr_impersonator".into(),
                autonomy_ceiling: AutonomyTier::T2Advisory,
            },
            invocation(),
        )
        .expect_err("principal cannot invoke as another user");

    assert_eq!(denied, FoundationError::CapabilityInvocationUnauthorized);
    let run = foundation
        .foundry_runs()
        .last()
        .expect("principal mismatch records a run");
    assert_eq!(run.state.value, RunState::RejectedPolicy);
    let evidence = foundation
        .foundry_evidence_chain()
        .records()
        .last()
        .expect("principal mismatch records evidence");
    assert_eq!(
        evidence.fields.value.get("reason").map(String::as_str),
        Some("principal_mismatch")
    );
}

#[test]
fn principal_mismatch_precedes_body_user_lookup_and_records_when_context_exists() {
    let mut foundation = foundation_with_capability();
    support::allow_capability_invocation(&mut foundation, "ten_invoke_authz", "tenant-admin");

    let mut request = invocation();
    request.user_id = "usr_missing_body_user".into();
    let denied = foundation
        .invoke_capability_as_principal(
            support::principal("ten_invoke_authz", "usr_operator", AutonomyTier::T2Advisory),
            request,
        )
        .expect_err("principal/body drift fails before body-user lookup");

    assert_eq!(denied, FoundationError::CapabilityInvocationUnauthorized);
    let run = foundation
        .foundry_runs()
        .last()
        .expect("principal mismatch records a run before body lookup");
    assert_eq!(run.state.value, RunState::RejectedPolicy);
    let evidence = foundation
        .foundry_evidence_chain()
        .records()
        .last()
        .expect("principal mismatch records evidence before body lookup");
    assert_eq!(
        evidence.fields.value.get("reason").map(String::as_str),
        Some("principal_mismatch")
    );
}

#[test]
fn cedar_policy_can_deny_on_effective_autonomy_ceiling_context() {
    let mut foundation = foundation_with_capability();
    support::allow_capability_invocation(&mut foundation, "ten_invoke_authz", "tenant-admin");
    foundation
        .publish_policy(PolicyVersion {
            policy_id: "pol_ten_invoke_authz_effective_ceiling_deny".into(),
            version: "1.0.0".into(),
            scope: PolicyScope::Tenant("ten_invoke_authz".into()),
            supersedes: None,
            rules: vec![PolicyRuleInput {
                effect: PolicyEffect::Deny,
                principal_role: "tenant-admin".into(),
                action: "foundry.capability.invoke".into(),
                resource_prefix: "capability:cap.demo.authz".into(),
                required_attribute: Some(("effective_ceiling".into(), "T2Advisory".into())),
                annotations: vec![],
            }],
        })
        .unwrap();

    let denied = foundation
        .invoke_capability_as_principal(
            support::principal("ten_invoke_authz", "usr_operator", AutonomyTier::T2Advisory),
            invocation(),
        )
        .expect_err("Cedar deny rule can inspect effective autonomy ceiling");

    assert_eq!(denied, FoundationError::CapabilityInvocationUnauthorized);
    assert_authorization_denial_reason(&foundation, "explicit deny policy");
}

#[test]
fn cedar_policy_can_deny_on_agentic_ads_cap_context() {
    let mut foundation = foundation_with_capability();
    support::seed_passing_eval(&mut foundation, "cap.ads.bid");
    foundation
        .register_capability(CapabilityRegistration {
            capability_id: "cap.ads.bid".into(),
            namespace: "ads".into(),
            action: CapabilityAction::AdsBid,
            required_tier: AutonomyTier::T1ViewOnly,
            touched_privacy_data_classes: application_app::privacy_data_classes_from(&[
                DataClass::InternalOnly,
            ])
            .unwrap(),
            evidence_topic: "oya.foundry.capability.invoked".into(),
        })
        .unwrap();
    foundation
        .grant_capability_to_tenant(TenantCapabilityGrant {
            tenant_id: "ten_invoke_authz".into(),
            capability_id: "cap.ads.bid".into(),
            mcp_visible: true,
        })
        .unwrap();
    support::allow_capability_invocation(&mut foundation, "ten_invoke_authz", "tenant-admin");
    foundation
        .publish_policy(PolicyVersion {
            policy_id: "pol_ten_invoke_authz_agentic_ads_deny".into(),
            version: "1.0.0".into(),
            scope: PolicyScope::Tenant("ten_invoke_authz".into()),
            supersedes: None,
            rules: vec![PolicyRuleInput {
                effect: PolicyEffect::Deny,
                principal_role: "tenant-admin".into(),
                action: "foundry.capability.invoke".into(),
                resource_prefix: "capability:cap.ads.".into(),
                required_attribute: Some(("agentic_ads_cap".into(), "T1ViewOnly".into())),
                annotations: vec![],
            }],
        })
        .unwrap();
    let mut request = invocation();
    request.capability_id = "cap.ads.bid".into();

    let denied = foundation
        .invoke_capability_as_principal(
            support::principal(
                "ten_invoke_authz",
                "usr_operator",
                AutonomyTier::T4AutoExecute,
            ),
            request,
        )
        .expect_err("Cedar deny rule can inspect agentic ads cap");

    assert_eq!(denied, FoundationError::CapabilityInvocationUnauthorized);
    assert_authorization_denial_reason(&foundation, "explicit deny policy");
}

fn foundation_with_capability() -> Foundation {
    let mut foundation = Foundation::default();
    foundation
        .onboard_tenant(TenantRegistration {
            tenant_id: "ten_invoke_authz".into(),
            legal_name: "Invoke AuthZ Tenant".into(),
            home_region: "failover-region".into(),
            residency_class: "global".into(),
            regulatory_packs: vec!["pack-gamma".into()],
            autonomy_ceiling: AutonomyTier::T2Advisory,
        })
        .unwrap();
    foundation
        .upsert_identity(IdentityRegistration {
            tenant_id: "ten_invoke_authz".into(),
            user_id: "usr_operator".into(),
            primary_identifier: "operator@example.test".into(),
            display_name: "Operator".into(),
            roles: vec!["tenant-admin".into()],
        })
        .unwrap();
    support::seed_passing_eval(&mut foundation, "cap.demo.authz");
    foundation
        .register_capability(CapabilityRegistration {
            capability_id: "cap.demo.authz".into(),
            namespace: "demo".into(),
            action: CapabilityAction::Other,
            required_tier: AutonomyTier::T2Advisory,
            touched_privacy_data_classes: application_app::privacy_data_classes_from(&[
                DataClass::InternalOnly,
            ])
            .unwrap(),
            evidence_topic: "oya.foundry.capability.invoked".into(),
        })
        .unwrap();
    foundation
        .grant_capability_to_tenant(TenantCapabilityGrant {
            tenant_id: "ten_invoke_authz".into(),
            capability_id: "cap.demo.authz".into(),
            mcp_visible: true,
        })
        .unwrap();
    foundation
        .configure_tenant_cost_budget(CostBudgetRegistration {
            tenant_id: "ten_invoke_authz".into(),
            capability_id: None,
            window_id: "2026-05".into(),
            monthly_limit_micros: 1_000,
            per_invocation_limit_micros: 100,
            warning_threshold_percent: 80,
        })
        .unwrap();
    foundation
}

fn invocation() -> CapabilityInvocationRequest {
    CapabilityInvocationRequest {
        tenant_id: "ten_invoke_authz".into(),
        user_id: "usr_operator".into(),
        capability_id: "cap.demo.authz".into(),
        purpose: Purpose::CapabilityInvocation,
        subject_class: SubjectClass::Adult,
        budget_window_id: "2026-05".into(),
        projected_cost_micros: 10,
        started_at_epoch_seconds: 1_000,
    }
}

fn assert_authorization_denial_reason(foundation: &Foundation, expected_reason: &str) {
    let run = foundation
        .foundry_runs()
        .last()
        .expect("authorization denial records a run");
    assert_eq!(run.state.value, RunState::RejectedPolicy);
    assert_eq!(
        run.disposition.value,
        Some(RunDisposition::FailureAuthorization)
    );
    let evidence = foundation
        .foundry_evidence_chain()
        .records()
        .last()
        .expect("authorization denial records evidence");
    assert_eq!(
        evidence.fields.value.get("reason").map(String::as_str),
        Some("authorization")
    );
    assert_eq!(
        evidence
            .fields
            .value
            .get("authorization_reason")
            .map(String::as_str),
        Some(expected_reason)
    );
    assert_eq!(foundation.foundry_steps().len(), 0);
}
