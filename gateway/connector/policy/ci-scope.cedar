// connector service — CI-scope policy fragment
// Cedar v4.2 LTS
//
// Purpose: scope the oya-ci reserved tenant's access to staging vendors only;
//          forbid reads of customer-tenant data.
// Owner:   axis-foundry + axis-integration + ops-security
// References: ARCHITECTURE.md §principals; tenant reserved-IDs convention
// soak-window-seconds: 60
// publish-stage: active

forbid (principal, action, resource);

// =============================================================================
// PERMIT 1 — CI lane invokes sandbox connector actions
// =============================================================================

permit (
  principal == Service::"oya-ci",
  action == Action::"connector_invoke",
  resource in OAuthGrant::?g
)
when {
  resource has tenant_id &&
  resource.tenant_id in [
    "tenant:oya-ci",
    "tenant:oya-staging",
    "tenant:oya-sandbox"
  ] &&
  resource has environment &&
  resource.environment == "sandbox"
};

// =============================================================================
// PERMIT 2 — CI lane reads catalog (read-only)
// =============================================================================

permit (
  principal == Service::"oya-ci",
  action == Action::"catalog_read",
  resource
);

// =============================================================================
// FORBID — CI lane reads customer-tenant data
// =============================================================================

forbid (
  principal == Service::"oya-ci",
  action,
  resource
)
when {
  resource has tenant_id &&
  !(resource.tenant_id in [
    "tenant:oya-ci",
    "tenant:oya-staging",
    "tenant:oya-sandbox",
    "tenant:oya-aggregate"
  ])
};

// =============================================================================
// FORBID — CI lane invokes production connectors
// =============================================================================

forbid (
  principal == Service::"oya-ci",
  action == Action::"connector_invoke",
  resource in OAuthGrant::?g
)
when {
  resource has environment &&
  resource.environment == "production"
};
