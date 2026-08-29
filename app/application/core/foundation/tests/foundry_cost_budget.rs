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
fn capability_invocation_requires_configured_cost_budget_and_commits_spend() {
    let mut foundation = foundation_with_capability("cap.demo.costed");

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
                capability_id: "cap.demo.costed".into(),
                purpose: Purpose::CapabilityInvocation,
                subject_class: SubjectClass::Adult,
                budget_window_id: "2026-05".into(),
                projected_cost_micros: 40,
                started_at_epoch_seconds: 1_000,
            },
        )
        .expect_err("invocation without budget configuration must fail closed");
    assert_eq!(denied, FoundationError::CostBudgetNotConfigured);
    assert_eq!(
        foundation
            .foundry_runs()
            .last()
            .expect("budget denial records a run")
            .state
            .value,
        RunState::RejectedBudget
    );

    foundation
        .configure_tenant_cost_budget(CostBudgetRegistration {
            tenant_id: "ten_budget".into(),
            capability_id: None,
            window_id: "2026-05".into(),
            monthly_limit_micros: 100,
            per_invocation_limit_micros: 100,
            warning_threshold_percent: 80,
        })
        .expect("tenant budget config is valid");

    let receipt = foundation
        .invoke_capability_as_principal(
            support::principal(
                "ten_budget",
                "usr_budget_admin",
                AutonomyTier::T3ExecuteWithApproval,
            ),
            CapabilityInvocationRequest {
                tenant_id: "ten_budget".into(),
                user_id: "usr_budget_admin".into(),
                capability_id: "cap.demo.costed".into(),
                purpose: Purpose::CapabilityInvocation,
                subject_class: SubjectClass::Adult,
                budget_window_id: "2026-05".into(),
                projected_cost_micros: 40,
                started_at_epoch_seconds: 1_000,
            },
        )
        .expect("budget headroom allows invocation");
    assert_eq!(
        receipt.cost_reservation_id.as_deref(),
        Some("res_000000000001")
    );
    let runs_after_success = foundation.foundry_runs().len();
    let steps_after_success = foundation.foundry_steps().len();
    let evidence_after_success = foundation.foundry_evidence_chain().records().len();
    let run_complete_allow_after_success = allow_surface_count(&foundation, "foundry.run.complete");
    let evidence_emit_allow_after_success =
        allow_surface_count(&foundation, "foundry.evidence.emit");
    let capability_invoke_allow_after_success =
        allow_surface_count(&foundation, "foundry.capability.invoke");

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
                capability_id: "cap.demo.costed".into(),
                purpose: Purpose::CapabilityInvocation,
                subject_class: SubjectClass::Adult,
                budget_window_id: "2026-05".into(),
                projected_cost_micros: 61,
                started_at_epoch_seconds: 1_000,
            },
        )
        .expect_err("committed spend leaves only 60 micros of monthly headroom");
    assert_eq!(denied, FoundationError::CostBudgetExceeded);
    assert_eq!(foundation.foundry_runs().len(), runs_after_success + 1);
    assert_eq!(foundation.foundry_steps().len(), steps_after_success);
    assert_eq!(
        foundation.foundry_evidence_chain().records().len(),
        evidence_after_success + 1
    );
    assert_eq!(
        allow_surface_count(&foundation, "foundry.run.complete"),
        run_complete_allow_after_success
    );
    assert_eq!(
        allow_surface_count(&foundation, "foundry.evidence.emit"),
        evidence_emit_allow_after_success + 1
    );
    assert_eq!(
        allow_surface_count(&foundation, "foundry.capability.invoke"),
        capability_invoke_allow_after_success
    );

    assert!(foundation.audit_chain().verify());
    assert!(foundation.audit_chain().events().iter().any(|event| {
        event.surface == "foundry.cost-budget.reserve" && event.decision == "DENY"
    }));
}

#[test]
fn per_capability_budget_overrides_tenant_monthly_headroom() {
    let mut foundation = foundation_with_capability("cap.demo.narrow-budget");
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
    foundation
        .configure_tenant_cost_budget(CostBudgetRegistration {
            tenant_id: "ten_budget".into(),
            capability_id: Some("cap.demo.narrow-budget".into()),
            window_id: "2026-05".into(),
            monthly_limit_micros: 50,
            per_invocation_limit_micros: 50,
            warning_threshold_percent: 80,
        })
        .unwrap();

    foundation
        .invoke_capability_as_principal(
            support::principal(
                "ten_budget",
                "usr_budget_admin",
                AutonomyTier::T3ExecuteWithApproval,
            ),
            CapabilityInvocationRequest {
                tenant_id: "ten_budget".into(),
                user_id: "usr_budget_admin".into(),
                capability_id: "cap.demo.narrow-budget".into(),
                purpose: Purpose::CapabilityInvocation,
                subject_class: SubjectClass::Adult,
                budget_window_id: "2026-05".into(),
                projected_cost_micros: 40,
                started_at_epoch_seconds: 1_000,
            },
        )
        .expect("first invocation fits per-capability budget");

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
                capability_id: "cap.demo.narrow-budget".into(),
                purpose: Purpose::CapabilityInvocation,
                subject_class: SubjectClass::Adult,
                budget_window_id: "2026-05".into(),
                projected_cost_micros: 11,
                started_at_epoch_seconds: 1_000,
            },
        )
        .expect_err("capability-specific ceiling overrides wider tenant budget");
    assert_eq!(denied, FoundationError::CostBudgetExceeded);
}

#[test]
fn capability_cost_profile_ceiling_denies_before_provider_route_side_effects() {
    let mut foundation = foundation_with_profiled_capability(
        "cap.demo.profile-ceiling",
        CapabilityCostProfile::new(25, 1_000, vec!["foundation-local".into()]).unwrap(),
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
                capability_id: "cap.demo.profile-ceiling".into(),
                purpose: Purpose::CapabilityInvocation,
                subject_class: SubjectClass::Adult,
                budget_window_id: "2026-05".into(),
                projected_cost_micros: 26,
                started_at_epoch_seconds: 1_000,
            },
        )
        .expect_err("capability cost profile ceiling is enforced before provider route");

    assert_eq!(denied, FoundationError::CostBudgetExceeded);
    assert_eq!(
        allow_surface_count(&foundation, "foundry.provider.route"),
        0
    );
    assert!(foundation.audit_chain().events().iter().any(|event| {
        event.surface == "foundry.cost-budget.reserve" && event.decision == "DENY"
    }));
}
