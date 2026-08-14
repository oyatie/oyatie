// payments µservice — subscription-authorization Cedar v4.2 LTS fragment
//
// Purpose: gate every Subscription::Create / Subscription::Read action for
//          tenant operators (recurring billing lifecycle). Default deny;
//          permit only tenant-bound principals acting on their own tenant's
//          subscription.
// Owner:   ops-security + axis-payments + axis-billing
// References: ARCHITECTURE.md §cedar-gates; threat-model.md FM-11
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar
// @soak-window-seconds: 60   per ADR-0294
// @publisher-svid: spiffe://oyatie/payments/foundry-agent

// =============================================================================
// Default deny — Cedar's implicit default deny; no unconditional forbid is
// needed (a universal forbid would deny even explicitly permitted actions).
// =============================================================================

// =============================================================================
// PERMIT 1 — tenant operator creates/reads subscriptions for own tenant
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"subscription.create",
    Action::"subscription.read"
  ],
  resource in Subscription::?s
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has kyb_state &&
  principal.kyb_state == "verified"
};

// =============================================================================
// FORBID — cross-tenant subscription access
// =============================================================================

forbid (
  principal,
  action in [
    Action::"subscription.create",
    Action::"subscription.read",
    Action::"subscription.cancel"
  ],
  resource
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};
