// payments µservice — refund-authorization Cedar v4.2 LTS fragment
//
// Purpose: gate every Refund::Create / Refund::Cancel action.
// Owner:   ops-security + axis-payments
// References: ARCHITECTURE.md §cedar-gates; threat-model.md FM-04
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar
// @soak-window-seconds: 60   per ADR-0294
// @publisher-svid: spiffe://oyatie/payments/foundry-agent

// =============================================================================
// Default deny — Cedar's implicit default deny; no unconditional forbid is
// needed (a universal forbid would deny even explicitly permitted actions).
// =============================================================================

// =============================================================================
// PERMIT 1 — tenant-operator issues refund on own tenant's charge
// =============================================================================
//
// Requires: original-charge exists, original-charge captured (not voided),
// refund-amount + prior-refunds ≤ original-charge amount, within refund window.

permit (
  principal in TenantOperator::?p,
  action in [Action::"refund.create", Action::"refund.cancel"],
  resource in Refund::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  resource has original_charge_exists &&
  resource.original_charge_exists == true &&
  resource has original_charge_captured &&
  resource.original_charge_captured == true &&
  resource has refund_amount_valid &&
  resource.refund_amount_valid == true &&
  resource has within_refund_window &&
  resource.within_refund_window == true
};

// =============================================================================
// PERMIT 2 — sub-merchant issues refund on own sub-merchant's charge
// =============================================================================

permit (
  principal in SubMerchant::?s,
  action == Action::"refund.create",
  resource in Refund::?r
)
when {
  principal has tenant_id &&
  principal has sub_merchant_id &&
  resource has tenant_id &&
  resource has sub_merchant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal.sub_merchant_id == resource.sub_merchant_id &&
  principal has kyc_state &&
  principal.kyc_state == "verified" &&
  principal has restricted &&
  principal.restricted == false &&
  resource has original_charge_exists &&
  resource.original_charge_exists == true &&
  resource has within_refund_window &&
  resource.within_refund_window == true
};

// =============================================================================
// PERMIT 3 — auto-refund on detected double-charge (FM-02)
// =============================================================================
//
// System-principal may auto-refund within 24h when the reconciliation worker
// detects a double-charge.

permit (
  principal in SystemPrincipal::?s,
  action == Action::"refund.create",
  resource in Refund::?r
)
when {
  principal has system_role &&
  principal.system_role == "reconciliation-worker" &&
  resource has reason_code &&
  resource.reason_code == "auto-refund-double-charge-detected"
};

// =============================================================================
// FORBID — refund-amount exceeds original-charge
// =============================================================================

forbid (
  principal,
  action == Action::"refund.create",
  resource in Refund::?r
)
when {
  resource has refund_amount_valid &&
  resource.refund_amount_valid == false
};

// =============================================================================
// FORBID — refund outside refund window (per-tenant policy)
// =============================================================================

forbid (
  principal,
  action == Action::"refund.create",
  resource in Refund::?r
)
when {
  resource has within_refund_window &&
  resource.within_refund_window == false
};

// =============================================================================
// FORBID — cross-tenant refund
// =============================================================================

forbid (
  principal,
  action in [Action::"refund.create", Action::"refund.cancel", Action::"refund.read"],
  resource
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// FORBID — refund on a charge that was already fully-refunded
// =============================================================================

forbid (
  principal,
  action == Action::"refund.create",
  resource in Refund::?r
)
when {
  resource has original_charge_fully_refunded &&
  resource.original_charge_fully_refunded == true
};

// =============================================================================
// FORBID — sub-merchant refund on non-own sub-merchant charge
// =============================================================================

forbid (
  principal in SubMerchant::?s,
  action,
  resource in Refund::?r
)
unless {
  principal has sub_merchant_id &&
  resource has sub_merchant_id &&
  principal.sub_merchant_id == resource.sub_merchant_id
};

// =============================================================================
// FORBID — restricted sub-merchant
// =============================================================================

forbid (
  principal in SubMerchant::?s,
  action,
  resource
)
when {
  principal has restricted &&
  principal.restricted == true
};
