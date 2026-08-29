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

fn allow_surface_count(foundation: &Foundation, surface: &str) -> usize {
    foundation
        .audit_chain()
        .events()
        .iter()
        .filter(|event| event.surface == surface && event.decision == "ALLOW")
        .count()
}

fn foundation_with_profiled_capability(
    capability_id: &str,
    cost_profile: CapabilityCostProfile,
) -> Foundation {
    let mut foundation = foundation_with_capability_fixture(capability_id);
    support::seed_passing_eval(&mut foundation, capability_id);
    foundation
        .register_capability_with_cost_profile(
            CapabilityRegistration {
                capability_id: capability_id.into(),
                namespace: "demo".into(),
                action: CapabilityAction::Other,
                required_tier: AutonomyTier::T2Advisory,
                touched_privacy_data_classes: application_foundation::privacy_data_classes_from(&[
                    DataClass::InternalOnly,
                ])
                .unwrap(),
                evidence_topic: "oya.foundry.capability.invoked".into(),
            },
            cost_profile,
        )
        .unwrap();
    foundation
        .grant_capability_to_tenant(TenantCapabilityGrant {
            tenant_id: "ten_budget".into(),
            capability_id: capability_id.into(),
            mcp_visible: true,
        })
        .unwrap();
    foundation
}

fn foundation_with_capability_fixture(_capability_id: &str) -> Foundation {
    let mut foundation = Foundation::default();
    foundation
        .onboard_tenant(TenantRegistration {
            tenant_id: "ten_budget".into(),
            legal_name: "Budget Tenant".into(),
            home_region: "failover-region".into(),
            residency_class: "global".into(),
            regulatory_packs: vec!["pack-gamma".into()],
            autonomy_ceiling: AutonomyTier::T3ExecuteWithApproval,
        })
        .unwrap();
    foundation
        .upsert_identity(IdentityRegistration {
            tenant_id: "ten_budget".into(),
            user_id: "usr_budget_admin".into(),
            primary_identifier: "budget@example.test".into(),
            display_name: "Budget Admin".into(),
            roles: vec!["tenant-admin".into()],
        })
        .unwrap();
    support::allow_capability_invocation(&mut foundation, "ten_budget", "tenant-admin");
    foundation
}

fn foundation_with_capability(capability_id: &str) -> Foundation {
    let mut foundation = Foundation::default();
    foundation
        .onboard_tenant(TenantRegistration {
            tenant_id: "ten_budget".into(),
            legal_name: "Budget Tenant".into(),
            home_region: "failover-region".into(),
            residency_class: "global".into(),
            regulatory_packs: vec!["pack-gamma".into()],
            autonomy_ceiling: AutonomyTier::T3ExecuteWithApproval,
        })
        .unwrap();
    foundation
        .upsert_identity(IdentityRegistration {
            tenant_id: "ten_budget".into(),
            user_id: "usr_budget_admin".into(),
            primary_identifier: "budget@example.test".into(),
            display_name: "Budget Admin".into(),
            roles: vec!["tenant-admin".into()],
        })
        .unwrap();
    support::allow_capability_invocation(&mut foundation, "ten_budget", "tenant-admin");
    support::seed_passing_eval(&mut foundation, capability_id);
    foundation
        .register_capability(CapabilityRegistration {
            capability_id: capability_id.into(),
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
            tenant_id: "ten_budget".into(),
            capability_id: capability_id.into(),
            mcp_visible: true,
        })
        .unwrap();
    foundation
}
