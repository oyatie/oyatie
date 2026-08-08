// observability µservice — auditor-scope policy fragment
// Cedar v4
//
// Purpose: external auditors (SOC 2 / ISO 27001 / HIPAA / KR-PIPC / GDPR DPA / etc.)
//          read time-boxed, tenant-scoped subsets of the data when an engagement is active.
// Owner:   ops-security + council-privacy + ops-compliance
// References: threat-model.md T-S-05, T-I-05, T-L-05; tenant-isolation.md §"Audit Trail"
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

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
// Auditor principals are issued JIT tokens by OpenBao at engagement start.
// The token carries:
//   - engagement_id        unique per audit cycle
//   - scoped_tenants       list of tenant_ids in scope for this engagement
//   - valid_from / valid_to  ISO8601 timestamps
//   - audit_framework      one of [soc2, iso27001, hipaa, gdpr-dpa, kr-pipc, ...]
//
// Cedar enforces token-time, tenant-scope, and read-only.

permit (
  principal in Auditor::?a,
  action in [
    Action::"read_metrics",
    Action::"read_logs",
    Action::"read_traces",
    Action::"read_audit_chain",
    Action::"read_promotion_history",
    Action::"read_eligibility_verdicts",
    Action::"read_dashboards",
    Action::"read_openslo_manifests",
    Action::"read_runbooks"
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
// PERMIT 2 — auditor reads pack-level configuration + policy artifacts
// =============================================================================
//
// Auditors may also read per-pack overlay docs (threat-model, dpia, compliance, etc.)
// without per-tenant scoping. These are publishable artifacts to the auditor; they
// contain policy text + structural information, not customer data.

permit (
  principal in Auditor::?a,
  action == Action::"read_artifact",
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
    "openslo-manifest",
    "adr",
    "prd"
  ]
};

// =============================================================================
// FORBID — auditor cannot write anything
// =============================================================================

forbid (
  principal in Auditor::?a,
  action in [
    Action::"write_metrics",
    Action::"write_logs",
    Action::"write_traces",
    Action::"write_openslo_manifests",
    Action::"write_artifact",
    Action::"delete_metrics",
    Action::"delete_logs",
    Action::"delete_traces",
    Action::"delete_artifact",
    Action::"rotate_secret"
  ],
  resource
);

// =============================================================================
// FORBID — auditor cannot read outside engagement window (defence-in-depth)
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
//
// Even with a valid token, an auditor cannot pivot to a non-scoped tenant.
// This complements PERMIT 1's positive scope with explicit negative bound.

forbid (
  principal in Auditor::?a,
  action in [
    Action::"read_metrics",
    Action::"read_logs",
    Action::"read_traces",
    Action::"read_audit_chain",
    Action::"read_promotion_history",
    Action::"read_eligibility_verdicts",
    Action::"read_dashboards",
    Action::"read_openslo_manifests"
  ],
  resource
)
when {
  resource has tenant_id &&
  principal has scoped_tenants &&
  !(resource.tenant_id in principal.scoped_tenants)
};

// =============================================================================
// FORBID — auditor cannot access SECRET-class data
// =============================================================================
//
// SECRET-class data (API keys, signing keys, etc.) is out of audit scope.
// Audit evidence is derived facts (audit-chain records), not the raw secrets.

forbid (
  principal in Auditor::?a,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};
