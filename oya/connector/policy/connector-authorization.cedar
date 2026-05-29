// connector service — connector-authorization policy fragment
// Cedar v4.2 LTS
//
// Purpose: gate which connector actions a tenant principal may invoke; enforce
//          per-tenant scope, scope-of-grant, compliance-pack filtering, and
//          audience-type allow-listing.
// Owner:   axis-integration + ops-security
// References: ARCHITECTURE.md §cedar-gates; PRD.md FR-04; threat-model.md T-S-05/T-E-05
// soak-window-seconds: 60
// publish-stage: active

// =============================================================================
// Default deny — every action must be permitted explicitly
// =============================================================================

forbid (principal, action, resource);

// =============================================================================
// PERMIT 1 — tenant principal invokes a granted connector action
// =============================================================================

permit (
  principal in TenantPrincipal::?p,
  action == Action::"connector_invoke",
  resource in OAuthGrant::?g
)
when {
  resource has tenant_id &&
  principal has tenant_id &&
  resource.tenant_id == principal.tenant_id &&
  resource has status &&
  resource.status == "ACTIVE" &&
  resource has scopes &&
  context has action_required_scope &&
  context.action_required_scope in resource.scopes
};

// =============================================================================
// PERMIT 2 — workflow-engine dispatcher invokes on behalf of tenant
// =============================================================================

permit (
  principal == Service::"oyatie.workflow-engine.dispatcher",
  action == Action::"connector_invoke",
  resource in OAuthGrant::?g
)
when {
  context has on_behalf_of_tenant &&
  resource has tenant_id &&
  resource.tenant_id == context.on_behalf_of_tenant
};

// =============================================================================
// FORBID — cross-tenant action invocation (defence in depth)
// =============================================================================

forbid (
  principal in TenantPrincipal::?p,
  action == Action::"connector_invoke",
  resource in OAuthGrant::?g
)
when {
  resource has tenant_id &&
  principal has tenant_id &&
  resource.tenant_id != principal.tenant_id
};

// =============================================================================
// FORBID — connector not in tenant's active compliance-pack allow-list
// =============================================================================

forbid (
  principal in TenantPrincipal::?p,
  action == Action::"connector_invoke",
  resource in OAuthGrant::?g
)
when {
  principal has compliance_packs &&
  resource has required_packs &&
  !(resource.required_packs.containsAny(principal.compliance_packs))
};

// =============================================================================
// FORBID — minor user audience invokes non-minor-eligible connector
// =============================================================================

forbid (
  principal in TenantPrincipal::?p,
  action == Action::"connector_invoke",
  resource in OAuthGrant::?g
)
when {
  principal has audience_minor_tier &&
  principal.audience_minor_tier == "coppa_under_13" &&
  resource has minor_eligible &&
  resource.minor_eligible != true
};

// =============================================================================
// FORBID — Tier-3 air-gap cell invokes internet-egress connector
// =============================================================================

forbid (
  principal in TenantPrincipal::?p,
  action == Action::"connector_invoke",
  resource in OAuthGrant::?g
)
when {
  principal has cell_tier &&
  principal.cell_tier == "tier-3" &&
  resource has requires_internet_egress &&
  resource.requires_internet_egress == true
};
