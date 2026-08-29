// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod support;

use application_foundation::{
    AutonomyTier, CapabilityAction, CapabilityCostProfile, CapabilityInvocationRequest,
    CapabilityRegistration, CostBudgetRegistration, DataClass, Foundation, FoundationError,
    IdentityRegistration, Purpose, RunState, SubjectClass, TenantCapabilityGrant,
    TenantRegistration,
};

mod foundry_cost_budget_fixtures;

use foundry_cost_budget_fixtures::*;

#[test]
fn capability_provider_preference_is_not_ignored_by_foundation_routing() {
    let mut foundation = foundation_with_profiled_capability(
        "cap.demo.unsupported-provider",
        CapabilityCostProfile::new(100, 1_000, vec!["anthropic-api".into()]).unwrap(),
    );
    foundation
        .configure_tenant_cost_budget(CostBudgetRegistration {
            tenant_id: "ten_budget".into(),
            capability_id: None,
            window_id: "2026-05".into(),
            monthly_limit_micros: 1_000,
            per_invocation_limit_micros: 1_000,
            warning_threshold_percent: 80,
        })
        .unwrap();

    let denied = foundation
        .invoke_capability_as_principal(
            support::principal(
                "ten_budget",
                "usr_budget_admin",
                AutonomyTier::T3ExecuteWithApproval,
            ),
            CapabilityInvocationRequest {
                tenant_id: "ten_budget".into(),
                user_id: "usr_budget_admin".into(),
                capability_id: "cap.demo.unsupported-provider".into(),
                purpose: Purpose::CapabilityInvocation,
                subject_class: SubjectClass::Adult,
                budget_window_id: "2026-05".into(),
                projected_cost_micros: 10,
                started_at_epoch_seconds: 1_000,
            },
        )
        .expect_err("unsupported declared provider preference must not fall back to hardcoded local provider");

    assert_eq!(denied, FoundationError::CostBudgetExceeded);
    assert!(
        foundation
            .audit_chain()
            .events()
            .iter()
            .any(|event| { event.surface == "foundry.provider.route" && event.decision == "DENY" })
    );
    assert_eq!(
        allow_surface_count(&foundation, "foundry.provider.route"),
        0
    );
}
