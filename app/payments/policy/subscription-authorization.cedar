// payments µservice — subscription-authorization Cedar v4.2 LTS fragment
//
// Purpose: gate the recurring-billing lifecycle for tenant operators
//          (create / read / cancel / upgrade / downgrade) and for the
//          dunning/reconciliation workers that retry failed renewals.
//          Default deny; permit only tenant-bound principals acting on their
//          own tenant's subscription.
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
// PERMIT 1 — tenant operator lifecycle actions for own tenant
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"subscription.create",
    Action::"subscription.read",
    Action::"subscription.cancel",
    Action::"subscription.upgrade",
    Action::"subscription.downgrade"
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
// PERMIT 2 — dunning/reconciliation worker retries a failed renewal
// =============================================================================
//
// The dunning CronJob (iac/helm/payments-app dunning-worker) retries failed
// subscription renewals on behalf of the tenant; without a tenant-bound
// worker permit Cedar would default-deny every retry and failed renewals
// would be unrecoverable.

permit (
  principal in SystemPrincipal::?s,
  action in [
    Action::"subscription.retry",
    Action::"subscription.dunning-attempt"
  ],
  resource in Subscription::?s
)
when {
  principal has system_role &&
  principal.system_role in [
    "dunning-worker",
    "reconciliation-worker"
  ] &&
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// FORBID — cross-tenant subscription access
// =============================================================================

forbid (
  principal,
  action in [
    Action::"subscription.create",
    Action::"subscription.read",
    Action::"subscription.cancel",
    Action::"subscription.upgrade",
    Action::"subscription.downgrade",
    Action::"subscription.retry",
    Action::"subscription.dunning-attempt"
  ],
  resource
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};
