// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(dead_code)]

use application_foundation::{
    AutonomyTier, CapabilityAction, CapabilityCostProfile, CapabilityInvocationRequest,
    CapabilityRegistration, CostBudgetRegistration, DataClass, Foundation, FoundationError,
    IdentityRegistration, Purpose, RunState, SubjectClass, TenantCapabilityGrant,
    TenantRegistration,
};

pub fn allow_surface_count(foundation: &Foundation, surface: &str) -> usize {
    foundation
        .audit_chain()
        .events()
        .iter()
        .filter(|event| event.surface == surface && event.decision == "ALLOW")
        .count()
}

pub fn foundation_with_profiled_capability(
    capability_id: &str,
    cost_profile: CapabilityCostProfile,
) -> Foundation {
    let mut foundation = foundation_with_capability_fixture(capability_id);
    super::support::seed_passing_eval(&mut foundation, capability_id);
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

pub fn foundation_with_capability_fixture(_capability_id: &str) -> Foundation {
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
    super::support::allow_capability_invocation(&mut foundation, "ten_budget", "tenant-admin");
    foundation
}

pub fn foundation_with_capability(capability_id: &str) -> Foundation {
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
    super::support::allow_capability_invocation(&mut foundation, "ten_budget", "tenant-admin");
    super::support::seed_passing_eval(&mut foundation, capability_id);
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
