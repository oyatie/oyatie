// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(dead_code)]

use application_foundation::{
    AutonomyTier, CapabilityAction, CapabilityInvocationRequest, CapabilityRegistration,
    CostBudgetRegistration, DataClass, EvidenceKind, Foundation, IdentityRegistration, Purpose,
    RunDisposition, RunState, StepDisposition, StepKind, StepState, SubjectClass,
    TenantCapabilityGrant, TenantRegistration,
};

use application_foundation::Capability;

/// A foundation with the runs demo capability registered, licensed,
/// eval-passed and budgeted - the precondition every run assertion shares.
pub fn foundation_with_runs_capability() -> (Foundation, Capability) {
    let mut foundation = Foundation::default();
    foundation
        .onboard_tenant(TenantRegistration {
            tenant_id: "ten_runs".into(),
            legal_name: "Runs Tenant".into(),
            home_region: "secondary-region".into(),
            residency_class: "global".into(),
            regulatory_packs: vec!["pack-secondary".into()],
            autonomy_ceiling: AutonomyTier::T2Advisory,
        })
        .unwrap();
    foundation
        .upsert_identity(IdentityRegistration {
            tenant_id: "ten_runs".into(),
            user_id: "usr_runs_admin".into(),
            primary_identifier: "runs@example.test".into(),
            display_name: "Runs Admin".into(),
            roles: vec!["tenant-admin".into()],
        })
        .unwrap();
    super::support::allow_capability_invocation(&mut foundation, "ten_runs", "tenant-admin");
    super::support::seed_passing_eval(&mut foundation, "cap.demo.runs");
    let capability = foundation
        .register_capability(CapabilityRegistration {
            capability_id: "cap.demo.runs".into(),
            namespace: "demo".into(),
            action: CapabilityAction::Other,
            required_tier: AutonomyTier::T2Advisory,
            touched_privacy_data_classes: application_foundation::privacy_data_classes_from(&[
                DataClass::InternalOnly,
            ])
            .unwrap(),
            evidence_topic: "oya.foundry.capability.invoked.custom".into(),
        })
        .unwrap();
    foundation
        .grant_capability_to_tenant(TenantCapabilityGrant {
            tenant_id: "ten_runs".into(),
            capability_id: capability.id.clone(),
            mcp_visible: true,
        })
        .unwrap();
    foundation
        .configure_tenant_cost_budget(CostBudgetRegistration {
            tenant_id: "ten_runs".into(),
            capability_id: None,
            window_id: "2026-05".into(),
            monthly_limit_micros: 1_000,
            per_invocation_limit_micros: 100,
            warning_threshold_percent: 80,
        })
        .unwrap();

    (foundation, capability)
}
