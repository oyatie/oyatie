// payments µservice — auditor-scope Cedar v4.2 LTS fragment
//
// Purpose: External auditor read scope (PCI QSA, KR-FSS auditor, EU PSD2
//          examiner, SOC 2 examiner, ISO 27001 auditor, GDPR DPA).
//          Time-boxed, tenant-scoped, read-only.
// Owner:   ops-security + ops-compliance + council-privacy
// References: ARCHITECTURE.md §cedar-gates; compliance.md §audit
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar
// @soak-window-seconds: 60   per ADR-0294
// @publisher-svid: spiffe://oyatie/payments/foundry-agent

// =============================================================================
// Default deny
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — auditor reads tenant-scoped data within engagement window
// =============================================================================
//
// Auditor principals are issued JIT tokens at engagement start.
// Token carries: engagement_id, scoped_tenants, valid_from / valid_to,
// audit_framework (one of: pci-qsa, kr-fss, eu-psd2, soc2, iso27001, gdpr-dpa).

permit (
  principal in Auditor::?a,
  action in [
    Action::"charge.read",
    Action::"refund.read",
    Action::"payout.read",
    Action::"dispute.read",
    Action::"subscription.read",
    Action::"sub-merchant.read",
    Action::"audit-chain.read",
    Action::"settlement.read"
  ],
  resource
)
when {
  principal has valid_from &&
  principal has valid_to &&
  context has now &&
  principal.valid_from <= context.now &&
  context.now <= principal.valid_to &&
  resource has tenant_id &&
  principal has scoped_tenants &&
  resource.tenant_id in principal.scoped_tenants
};

// =============================================================================
// PERMIT 2 — auditor reads policy artifacts (cross-pack)
// =============================================================================

permit (
  principal in Auditor::?a,
  action == Action::"artifact.read",
  resource in PolicyArtifact::?r
)
when {
  principal has valid_from &&
  principal has valid_to &&
  context has now &&
  principal.valid_from <= context.now &&
  context.now <= principal.valid_to &&
  resource has artifact_type &&
  resource.artifact_type in [
    "threat-model",
    "dpia",
    "compliance",
    "tenant-isolation",
    "data-residency",
    "policy-cedar",
    "runbook",
    "openapi",
    "asyncapi",
    "proto",
    "adr",
    "prd",
    "audit-findings"
  ]
};

// =============================================================================
// FORBID — auditor cannot write
// =============================================================================

forbid (
  principal in Auditor::?a,
  action in [
    Action::"charge.create",
    Action::"charge.authorize",
    Action::"charge.capture",
    Action::"refund.create",
    Action::"payout.schedule",
    Action::"payout.execute",
    Action::"dispute.submit-evidence",
    Action::"subscription.create",
    Action::"sub-merchant.onboard",
    Action::"sub-merchant.update",
    Action::"sub-merchant.restrict",
    Action::"artifact.write",
    Action::"artifact.delete"
  ],
  resource
);

// =============================================================================
// FORBID — auditor cannot read outside engagement window
// =============================================================================

forbid (
  principal in Auditor::?a,
  action,
  resource
)
unless {
  principal has valid_from &&
  principal has valid_to &&
  context has now &&
  principal.valid_from <= context.now &&
  context.now <= principal.valid_to
};

// =============================================================================
// FORBID — auditor cannot pivot beyond scoped_tenants
// =============================================================================

forbid (
  principal in Auditor::?a,
  action in [
    Action::"charge.read",
    Action::"refund.read",
    Action::"payout.read",
    Action::"dispute.read",
    Action::"subscription.read",
    Action::"sub-merchant.read",
    Action::"audit-chain.read",
    Action::"settlement.read"
  ],
  resource
)
when {
  resource has tenant_id &&
  principal has scoped_tenants &&
  !(resource.tenant_id in principal.scoped_tenants)
};

// =============================================================================
// FORBID — auditor cannot access raw PSP credentials
// =============================================================================
//
// Even with a valid engagement token, raw PSP credentials are out of audit scope.
// Audit evidence is the audit-chain (derived facts), not the raw secrets.

forbid (
  principal in Auditor::?a,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};

// =============================================================================
// FORBID — auditor cannot access raw card-data (PCI scope; never present)
// =============================================================================
//
// Defence-in-depth — PAN should not exist in our system; this guarantees that
// even if a developer error introduced raw card-data, auditors cannot read it.

forbid (
  principal in Auditor::?a,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class in ["PAN", "PIN", "TRACK_DATA", "CVV"]
};
