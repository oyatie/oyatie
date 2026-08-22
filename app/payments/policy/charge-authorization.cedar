// payments µservice — charge-authorization Cedar v4.2 LTS fragment
//
// Purpose: gate every Charge::Create / Charge::Authorize / Charge::Capture action.
// Owner:   ops-security + axis-payments + ops-fraud
// Audit:   every deny emits payments_authorization_denied_total
// References: ARCHITECTURE.md §cedar-gates; threat-model.md T-S-02 / T-E-01
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (canonical envelope per ADR-0064)
// @soak-window-seconds: 60   per ADR-0294
// @publisher-svid: spiffe://oyatie/payments/foundry-agent

// =============================================================================
// Default deny — every charge action is refused unless an explicit permit matches.
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — tenant-operator creates charge for own tenant
// =============================================================================
//
// A principal bound to TenantOperator::"<tenant-id>" may create a charge against
// a payment-method that is owned by the same tenant. Tenant must have completed
// KYB; tenant audience_type must match charge.audience_type.

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"charge.create",
    Action::"charge.authorize",
    Action::"charge.capture",
    Action::"charge.void"
  ],
  resource in Charge::?c
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has kyb_state &&
  principal.kyb_state == "verified" &&
  resource has audience_type &&
  resource.audience_type in ["B2B_TENANT", "B2C_CONSUMER", "PARTNER_AGENCY"]
};

// =============================================================================
// PERMIT 2 — B2C consumer creates charge via tenant's surface
// =============================================================================
//
// A consumer-principal on a tenant's surface may create a charge on their own
// payment-method. The tenant operator's surface mediates; consumer is authenticated.

permit (
  principal in Consumer::?u,
  action == Action::"charge.create",
  resource in Charge::?c
)
when {
  principal has consumer_id &&
  resource has consumer_id &&
  principal.consumer_id == resource.consumer_id &&
  principal has age_class &&
  // COPPA refusal: refuse <13. KOSA tier 14-17 has per-amount restriction.
  principal.age_class != "under_13" &&
  resource has tenant_id &&
  // Bot-Mgmt score gate
  (!principal has bot_score || principal.bot_score < 80)
};

// =============================================================================
// PERMIT 3 — sub-merchant creates charge against own sub-merchant scope
// =============================================================================

permit (
  principal in SubMerchant::?s,
  action == Action::"charge.create",
  resource in Charge::?c
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
  principal.restricted == false
};

// =============================================================================
// FORBID — KOSA tier 14-17 amount restriction
// =============================================================================

forbid (
  principal in Consumer::?u,
  action == Action::"charge.create",
  resource in Charge::?c
)
when {
  principal has age_class &&
  principal.age_class in ["age_14_15", "age_16_17"] &&
  resource has amount_minor &&
  // No charges >$50 / 50,000 minor units for KOSA-tier
  resource.amount_minor > 5000
};

// =============================================================================
// FORBID — COPPA refusal (under_13)
// =============================================================================

forbid (
  principal in Consumer::?u,
  action == Action::"charge.create",
  resource
)
when {
  principal has age_class &&
  principal.age_class == "under_13"
};

// =============================================================================
// FORBID — explicit cross-tenant deny (defence-in-depth)
// =============================================================================

forbid (
  principal,
  action in [
    Action::"charge.create",
    Action::"charge.authorize",
    Action::"charge.capture",
    Action::"charge.void",
    Action::"charge.read"
  ],
  resource
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// FORBID — per-tenant quota breach
// =============================================================================

forbid (
  principal,
  action == Action::"charge.create",
  resource
)
when {
  principal has request_rate_per_minute &&
  resource has rate_limit_per_minute &&
  principal.request_rate_per_minute > resource.rate_limit_per_minute
};

// =============================================================================
// FORBID — bot-score gate (per documentation-rigor.md §3.2.3 anti-bot)
// =============================================================================

forbid (
  principal,
  action == Action::"charge.create",
  resource
)
when {
  principal has bot_score &&
  principal.bot_score > 95 &&
  !(principal has audience_type &&
    principal.audience_type == "FRIENDLY_CRAWLER_PARTNER")
};

// =============================================================================
// FORBID — sub-merchant restricted
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
// FORBID — sanctions hit (OFAC + EU + UN + KR-FSS sanctions list)
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  principal has sanctions_match &&
  principal.sanctions_match == true
};
