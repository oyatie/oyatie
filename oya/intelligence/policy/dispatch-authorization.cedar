// intelligence µservice — dispatch-authorization policy fragment
// Cedar v4
//
// Purpose: gate every dispatch on (principal, tenant, audience, purpose, modality, pack).
// Owner:   axis-intelligence + ops-security
// References: ARCHITECTURE.md §7 (Cedar gate roster); threat-model.md T-S-01, T-E-01.
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar

// =============================================================================
// Default deny
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — Authenticated tenant principal may dispatch on its own tenant
// =============================================================================

permit (
  principal in TenantPrincipal::?p,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  resource has audience_tag &&
  principal has allowed_audiences &&
  resource.audience_tag in principal.allowed_audiences &&
  resource has purpose &&
  resource has modality &&
  resource.modality in ["text", "image", "audio", "video", "multi"]
};

// =============================================================================
// PERMIT 2 — Foundry agent (internal) may dispatch with audience=internal-foundry
// =============================================================================
//
// Foundry SPIFFE identities under spiffe://oyatie/foundry/<role> may dispatch as
// audience=internal-foundry with elevated rate limit + platform-default credentials.

permit (
  principal in FoundryAgent::?a,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  principal has spiffe_id &&
  principal.spiffe_id like "spiffe://oyatie/foundry/*" &&
  resource has audience_tag &&
  resource.audience_tag == "internal-foundry"
};

// =============================================================================
// PERMIT 3 — Consumer end-user may dispatch as audience=consumer with platform-default
// =============================================================================

permit (
  principal in ConsumerEndUser::?u,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  resource has audience_tag &&
  resource.audience_tag == "consumer" &&
  resource has secret_reference &&
  resource.secret_reference.kind == "platform_default" &&
  principal has consumer_daily_budget_remaining_usd &&
  principal.consumer_daily_budget_remaining_usd > 0.0
};

// =============================================================================
// FORBID — Audience tag elevation (consumer → developer → internal-foundry)
// =============================================================================

forbid (
  principal,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  resource has audience_tag &&
  principal has allowed_audiences &&
  !(resource.audience_tag in principal.allowed_audiences)
};

// =============================================================================
// FORBID — Cross-tenant dispatch
// =============================================================================

forbid (
  principal,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id != resource.tenant_id
};

// =============================================================================
// FORBID — Consumer dispatch when daily budget exhausted
// =============================================================================

forbid (
  principal in ConsumerEndUser::?u,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  resource has audience_tag &&
  resource.audience_tag == "consumer" &&
  resource has secret_reference &&
  resource.secret_reference.kind == "platform_default" &&
  principal has consumer_daily_budget_remaining_usd &&
  principal.consumer_daily_budget_remaining_usd <= 0.0
};

// =============================================================================
// FORBID — Tenant cost-cap breach
// =============================================================================

forbid (
  principal in TenantPrincipal::?p,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  principal has tenant_cost_cap_status &&
  principal.tenant_cost_cap_status in ["exceeded_daily", "exceeded_monthly"]
};
