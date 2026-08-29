// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(dead_code)]

use application_foundation::{
    AutonomyTier, CapabilityAction, CapabilityInvocationPrincipal, CapabilityInvocationRequest,
    CapabilityRegistration, CostBudgetRegistration, DataClass, Foundation, FoundationError,
    IdentityRegistration, PolicyEffect, PolicyRuleInput, PolicyScope, PolicyVersion, Purpose,
    RunDisposition, RunState, SubjectClass, TenantCapabilityGrant, TenantRegistration,
};

pub fn foundation_with_capability() -> Foundation {
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
    super::support::seed_passing_eval(&mut foundation, "cap.demo.authz");
    foundation
        .register_capability(CapabilityRegistration {
            capability_id: "cap.demo.authz".into(),
            namespace: "demo".into(),
            action: CapabilityAction::Other,
            required_tier: AutonomyTier::T2Advisory,
            touched_privacy_data_classes: application_foundation::privacy_data_classes_from(&[
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

pub fn invocation() -> CapabilityInvocationRequest {
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

pub fn assert_authorization_denial_reason(foundation: &Foundation, expected_reason: &str) {
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
