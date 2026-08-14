// payments µservice — sub-merchant-onboarding Cedar v4.2 LTS fragment
//
// Purpose: gate Stripe / Adyen MarketPay sub-merchant onboarding.
//          Enforce KYC tier, ToS acceptance, sanctions screen, age verification
//          (no minor as beneficial-owner), per-jurisdiction restrictions.
// Owner:   ops-security + axis-payments + ops-compliance + ops-fraud
// References: ARCHITECTURE.md §cedar-gates; threat-model.md T-S-03
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar
// @soak-window-seconds: 60   per ADR-0294
// @publisher-svid: spiffe://oyatie/payments/foundry-agent

// =============================================================================
// Default deny — Cedar's implicit default deny; no unconditional forbid is
// needed (a universal forbid would deny even explicitly permitted actions).
// =============================================================================

// =============================================================================
// PERMIT 1 — tenant onboards a sub-merchant
// =============================================================================
//
// Requirements:
//   - tenant KYB verified.
//   - sub-merchant ToS accepted.
//   - sub-merchant beneficial-owner KYC tier 1 (identity verified).
//   - sub-merchant business-jurisdiction in tenant's allowed-jurisdictions.

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"sub-merchant.onboard",
    Action::"sub-merchant.update"
  ],
  resource in SubMerchant::?s
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has kyb_state &&
  principal.kyb_state == "verified" &&
  resource has tos_accepted &&
  resource.tos_accepted == true &&
  resource has kyc_tier &&
  resource.kyc_tier in ["tier_1", "tier_2", "tier_3"] &&
  resource has beneficial_owner_age_class &&
  // No minor as beneficial-owner
  !(resource.beneficial_owner_age_class in ["under_13", "age_14_15", "age_16_17"]) &&
  resource has jurisdiction_allowed &&
  resource.jurisdiction_allowed == true
};

// =============================================================================
// PERMIT 2 — auto-restrict by reconciliation worker
// =============================================================================
//
// System-principal may flip sub-merchant.restricted=true on detected fraud or
// PSP-side restriction-event.

permit (
  principal in SystemPrincipal::?s,
  action == Action::"sub-merchant.restrict",
  resource in SubMerchant::?m
)
when {
  principal has system_role &&
  principal.system_role in [
    "reconciliation-worker",
    "fraud-handler",
    "webhook-receiver"
  ] &&
  // Bind the restriction to the principal's tenant and require positive
  // evidence that the triggering event was an authenticated PSP/check event;
  // without these a compromised receiver could suspend merchants platform-wide.
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  resource has psp_event_authenticated &&
  resource.psp_event_authenticated == true
};

// =============================================================================
// FORBID — ToS not accepted
// =============================================================================

forbid (
  principal,
  action == Action::"sub-merchant.onboard",
  resource in SubMerchant::?s
)
when {
  resource has tos_accepted &&
  resource.tos_accepted == false
};

// =============================================================================
// FORBID — beneficial owner is a minor
// =============================================================================

forbid (
  principal,
  action == Action::"sub-merchant.onboard",
  resource in SubMerchant::?s
)
when {
  resource has beneficial_owner_age_class &&
  resource.beneficial_owner_age_class in ["under_13", "age_14_15", "age_16_17"]
};

// =============================================================================
// FORBID — jurisdiction not allowed
// =============================================================================

forbid (
  principal,
  action == Action::"sub-merchant.onboard",
  resource in SubMerchant::?s
)
when {
  resource has jurisdiction_allowed &&
  resource.jurisdiction_allowed == false
};

// =============================================================================
// FORBID — sanctions hit
// =============================================================================

forbid (
  principal,
  action == Action::"sub-merchant.onboard",
  resource in SubMerchant::?s
)
when {
  resource has sanctions_match &&
  resource.sanctions_match == true
};

// =============================================================================
// FORBID — KYC below tier_1
// =============================================================================

forbid (
  principal,
  action == Action::"sub-merchant.onboard",
  resource in SubMerchant::?s
)
when {
  resource has kyc_tier &&
  !(resource.kyc_tier in ["tier_1", "tier_2", "tier_3"])
};

// =============================================================================
// FORBID — cross-tenant
// =============================================================================

forbid (
  principal,
  action in [Action::"sub-merchant.onboard", Action::"sub-merchant.update", Action::"sub-merchant.restrict"],
  resource
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};
