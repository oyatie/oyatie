// payments µservice — payout-authorization Cedar v4.2 LTS fragment
//
// Purpose: gate every Payout::Schedule / Payout::Execute / Payout::Cancel action.
//          Multi-layer defence: KYB + bank-verified + cooling-period + dual-signoff
//          (for high-value) + sanctions-screen.
// Owner:   ops-security + axis-payments + ops-treasury
// References: ARCHITECTURE.md §cedar-gates; threat-model.md T-E-02 / FM-13
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar
// @soak-window-seconds: 60   per ADR-0294
// @publisher-svid: spiffe://oyatie/payments/foundry-agent

// =============================================================================
// Default deny — Cedar's implicit default deny; no unconditional forbid is
// needed (a universal forbid would deny even explicitly permitted actions).
// =============================================================================

// =============================================================================
// PERMIT 1 — tenant-operator schedules payout for own tenant
// =============================================================================
//
// A principal bound to TenantOperator may schedule a payout to a bank-account
// owned by the same tenant. Requires:
//   - tenant KYB verified
//   - bank-account verified (micro-deposit / Plaid-equivalent)
//   - past cooling-period for new bank-account (14 days)

permit (
  principal in TenantOperator::?p,
  action == Action::"payout.schedule",
  resource in Payout::?o
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has kyb_state &&
  principal.kyb_state == "verified" &&
  resource has bank_account_state &&
  resource.bank_account_state == "verified" &&
  resource has cooling_period_elapsed &&
  resource.cooling_period_elapsed == true
};

// =============================================================================
// PERMIT 2 — sub-merchant payout (Stripe Express equivalent)
// =============================================================================

permit (
  principal in SubMerchant::?s,
  action == Action::"payout.schedule",
  resource in Payout::?o
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
  resource has bank_account_state &&
  resource.bank_account_state == "verified" &&
  resource has cooling_period_elapsed &&
  resource.cooling_period_elapsed == true
};

// =============================================================================
// PERMIT 3 — payout worker executes an ordinary scheduled payout
// =============================================================================
//
// Without this permit Cedar's implicit default deny leaves every scheduled
// payout stuck: the fragment authorizes payout.schedule (permits 1-2) but the
// worker must also be able to execute. Narrowly scoped to the payout worker
// system principal, tenant-bound, and gated on the same controls the forbid
// set enforces (verified bank account, cooling period elapsed, no sanctions
// hit, KYC not revoked). The high-value dual-signoff forbid below still
// applies for amounts > $10k.

permit (
  principal in SystemPrincipal::?s,
  action == Action::"payout.execute",
  resource in Payout::?o
)
when {
  principal has system_role &&
  principal.system_role == "payout-worker" &&
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  resource has bank_account_state &&
  resource.bank_account_state == "verified" &&
  resource has cooling_period_elapsed &&
  resource.cooling_period_elapsed == true &&
  !(resource has recipient_sanctions_match &&
    resource.recipient_sanctions_match == true) &&
  !(resource has kyc_revoked && resource.kyc_revoked == true)
};

// =============================================================================
// FORBID — high-value payouts require dual-signoff
// =============================================================================
//
// Payouts > $10,000 require two authorised principals (separation of duty)
// per SOX-ITGC pack + PCI Req 7.3.

forbid (
  principal,
  action == Action::"payout.execute",
  resource in Payout::?o
)
when {
  resource has amount_minor &&
  resource.amount_minor > 1000000 &&  // $10,000 in minor units
  !(resource has dual_signoff_complete &&
    resource.dual_signoff_complete == true)
};

// =============================================================================
// FORBID — payout to non-verified bank-account
// =============================================================================

forbid (
  principal,
  action == Action::"payout.schedule",
  resource in Payout::?o
)
when {
  resource has bank_account_state &&
  resource.bank_account_state != "verified"
};

// =============================================================================
// FORBID — payout within cooling-period for new bank-account
// =============================================================================
//
// New bank-account must satisfy 14-day cooling-period before any payout.
// Prevents account-takeover -> rapid-payout extraction.

forbid (
  principal,
  action == Action::"payout.execute",
  resource in Payout::?o
)
when {
  resource has cooling_period_elapsed &&
  resource.cooling_period_elapsed == false
};

// =============================================================================
// FORBID — cross-tenant payout
// =============================================================================

forbid (
  principal,
  action in [
    Action::"payout.schedule",
    Action::"payout.execute",
    Action::"payout.cancel"
  ],
  resource
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// FORBID — sub-merchant payout to non-own sub-merchant bank-account
// =============================================================================

forbid (
  principal in SubMerchant::?s,
  action,
  resource in Payout::?o
)
unless {
  principal has sub_merchant_id &&
  resource has sub_merchant_id &&
  principal.sub_merchant_id == resource.sub_merchant_id
};

// =============================================================================
// FORBID — sanctions hit on payout recipient
// =============================================================================

forbid (
  principal,
  action == Action::"payout.execute",
  resource in Payout::?o
)
when {
  resource has recipient_sanctions_match &&
  resource.recipient_sanctions_match == true
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

// =============================================================================
// FORBID — payout during KYC-revocation window
// =============================================================================

forbid (
  principal,
  action == Action::"payout.execute",
  resource in Payout::?o
)
when {
  resource has kyc_revoked &&
  resource.kyc_revoked == true
};
