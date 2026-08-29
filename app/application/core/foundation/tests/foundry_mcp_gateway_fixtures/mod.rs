// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(dead_code)]

use application_foundation::{
    AutonomyTier, CapabilityAction, CapabilityMcpContract, CapabilityRegistration,
    CostBudgetRegistration, DISCOVER_SCOPE, DataClass, Foundation, FoundationError,
    IdentityRegistration, McpAccessTokenClaims, McpDiscoveryRequest, McpRateLimitPolicy,
    McpToolCallRequest, Purpose, SubjectClass, TenantCapabilityGrant, TenantRegistration,
    scope_for_tool_name,
};

pub fn onboard_tenant(
    foundation: &mut Foundation,
    tenant_id: &str,
    autonomy_ceiling: AutonomyTier,
) {
    foundation
        .onboard_tenant(TenantRegistration {
            tenant_id: tenant_id.into(),
            legal_name: format!("{tenant_id} tenant"),
            home_region: "region-home".into(),
            residency_class: "strict_home_region".into(),
            regulatory_packs: vec!["pack-alpha".into()],
            autonomy_ceiling,
        })
        .unwrap();
}

pub fn access_token(
    tenant_id: &str,
    scopes: Vec<String>,
    expires_at_epoch_seconds: u64,
) -> McpAccessTokenClaims {
    McpAccessTokenClaims {
        tenant_id: tenant_id.into(),
        subject_id: "usr_operator".into(),
        issuer: "https://auth.oyatie.test/tenants/ten_mcp".into(),
        audience: "https://mcp.foundry.region-home.oyatie.test/tenants/ten_mcp".into(),
        expires_at_epoch_seconds,
        scopes,
    }
}

pub fn register_internal_capability(
    foundation: &mut Foundation,
    capability_id: &str,
    required_tier: AutonomyTier,
    mcp_visible: bool,
) {
    super::support::seed_passing_eval(foundation, capability_id);
    foundation
        .register_capability(CapabilityRegistration {
            capability_id: capability_id.into(),
            namespace: "demo".into(),
            action: CapabilityAction::Other,
            required_tier,
            touched_privacy_data_classes: application_foundation::privacy_data_classes_from(&[
                DataClass::InternalOnly,
            ])
            .unwrap(),
            evidence_topic: "oya.foundry.capability.invoked".into(),
        })
        .unwrap();
    foundation
        .grant_capability_to_tenant(TenantCapabilityGrant {
            tenant_id: "ten_mcp".into(),
            capability_id: capability_id.into(),
            mcp_visible,
        })
        .unwrap();
}
