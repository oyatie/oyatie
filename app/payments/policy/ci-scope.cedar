// payments µservice — ci-scope Cedar v4.2 LTS fragment
//
// Purpose: CI principal scope. CI runs synthetic charges / refunds / payouts
//          against PSP sandbox-keys only; never against live tenant credentials.
// Owner:   ops-security + governance
// References: ARCHITECTURE.md §cedar-gates; ADR-0295 (SPIFFE attestation)
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar
// @soak-window-seconds: 60   per ADR-0294
// @publisher-svid: spiffe://oyatie/payments/foundry-agent

// =============================================================================
// Default deny — Cedar's implicit default deny; no unconditional forbid is
// needed (a universal forbid would deny even explicitly permitted actions).
// =============================================================================

// =============================================================================
// PERMIT 1 — CI may exercise sandbox-tenants only
// =============================================================================
//
// CI principals carry SPIFFE SVID issued by the CI cluster; they have a
// fixed sandbox-tenant allow-list and may not touch any production tenant.

permit (
  principal in CiPrincipal::?c,
  action in [
    Action::"charge.create",
    Action::"charge.authorize",
    Action::"charge.capture",
    Action::"charge.void",
    Action::"refund.create",
    Action::"payout.schedule",
    Action::"dispute.submit-evidence",
    Action::"subscription.create",
    Action::"sub-merchant.onboard",
    Action::"webhook.receive"
  ],
  resource
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has tenant_class &&
  principal.tenant_class == "sandbox" &&
  resource has tenant_class &&
  resource.tenant_class == "sandbox" &&
  principal has psp_key_class &&
  principal.psp_key_class == "sandbox"
};

// =============================================================================
// FORBID — CI cannot touch production tenants
// =============================================================================

forbid (
  principal in CiPrincipal::?c,
  action,
  resource
)
when {
  resource has tenant_class &&
  resource.tenant_class == "production"
};

// =============================================================================
// FORBID — CI cannot use live PSP keys
// =============================================================================

forbid (
  principal in CiPrincipal::?c,
  action,
  resource
)
when {
  principal has psp_key_class &&
  principal.psp_key_class == "live"
};

// =============================================================================
// FORBID — CI cannot execute payouts (only schedule them, never execute)
// =============================================================================

forbid (
  principal in CiPrincipal::?c,
  action == Action::"payout.execute",
  resource
);

// =============================================================================
// FORBID — CI cannot modify Cedar fragments
// =============================================================================
//
// Cedar-fragment publish is gated by the foundry-agent SVID (per ADR-0293
// meta-trust-root + ADR-0294 soak). CI principals do not have foundry-agent
// scope.

forbid (
  principal in CiPrincipal::?c,
  action in [
    Action::"policy.publish",
    Action::"policy.rollback"
  ],
  resource
);

// =============================================================================
// FORBID — CI without valid SPIFFE SVID
// =============================================================================

forbid (
  principal in CiPrincipal::?c,
  action,
  resource
)
when {
  !(principal has svid_present) ||
  principal.svid_present == false ||
  (principal has svid_valid && principal.svid_valid == false)
};
