// tenancy µservice — auditor-scope policy fragment
// Cedar v4
//
// Purpose: external auditors (SOC 2 / ISO 27001 / HIPAA / KR-PIPC / GDPR DPA / regulator)
//          read time-boxed, tenant-scoped subsets when an engagement is active.
//          For tenancy specifically — auditors may also read RLS policy state +
//          proof-of-erasure certificates (the load-bearing audit artifacts).
// Owner:   ops-security + council-privacy + ops-compliance
// References: threat-model.md T-S-05, T-L-05, T-L-10; rls-isolation.md §"Audit Trail"
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// PERMIT 1a — auditor reads tenant-scoped lifecycle state within window
// =============================================================================
//
// Auditor JIT tokens carry:
//   - engagement_id        unique per audit cycle
//   - scoped_tenants       list of tenant_ids in scope for this engagement
//   - valid_from / valid_to ISO8601 timestamps
//   - audit_framework      one of [soc2, iso27001, hipaa, gdpr-dpa, kr-pipc, ...]
//   - regulatory_pack      the pack jurisdiction the auditor is empowered for

permit (
  principal in Auditor::?a,
  action in [
    Action::"read_tenant",
    Action::"read_tenant_status",
    Action::"read_tenant_lifecycle_history",
    Action::"read_cell_assignment",
    Action::"read_audit_chain"
  ],
  resource is TenantAuditRecord
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
// PERMIT 1b — auditor reads tenant-scoped DSR and erasure evidence within window
// =============================================================================

permit (
  principal in Auditor::?a,
  action in [
    Action::"read_dsr_request",
    Action::"read_dsr_request_history",
    Action::"read_proof_of_erasure",
    Action::"read_erasure_receipt"
  ],
  resource is DsrEvidence
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
// PERMIT 2 — auditor reads RLS policy state + isolation evidence
// =============================================================================
//
// Auditors verifying tenant-isolation controls need to read:
// - The declarative RLS YAML manifests
// - The live Postgres pg_policies state
// - The drift-detection metrics
// - The CI lane execution log (rls-no-superuser-bypass + rls-force-on-tenant-tables)

permit (
  principal in Auditor::?a,
  action in [
    Action::"read_rls_policy_manifest",
    Action::"read_rls_policy_live_state",
    Action::"read_rls_drift_metrics",
    Action::"read_lean_lane_execution_log"
  ],
  resource is RlsIsolationEvidence
)
when {
  principal has valid_from &&
  principal has valid_to &&
  context has now &&
  principal.valid_from <= context.now &&
  context.now <= principal.valid_to
};

// =============================================================================
// PERMIT 3 — auditor reads policy artifacts + threat-model + DPIA + compliance
// =============================================================================
//
// These artifacts are publishable to the auditor (policy text + structural
// information; no customer data). Read without per-tenant scoping.

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
    "rls-isolation",
    "data-residency",
    "policy-cedar",
    "runbook",
    "adr",
    "prd",
    "failure-modes",
    "capacity-model",
    "cost-budget",
    "incident-response",
    "multi-region",
    "competitor-parity-matrix",
    "backfill-replay",
    "sdk-plan"
  ]
};

// =============================================================================
// PERMIT 4 — regulator-scope reads (subset of auditor with legal compulsion)
// =============================================================================
//
// Regulators (KR PIPC investigator, EU DPA, etc.) operate under legal compulsion;
// scope is the regulator's jurisdiction. Distinct from external SOC 2 auditor:
// regulator may read DSR cascade history + proof-of-erasure for any tenant in
// their jurisdiction, not just engagement-scoped tenants.

permit (
  principal in Regulator::?r,
  action in [
    Action::"read_dsr_request",
    Action::"read_proof_of_erasure",
    Action::"read_erasure_receipt"
  ],
  resource is DsrEvidence
)
when {
  principal has valid_from &&
  principal has valid_to &&
  context has now &&
  principal.valid_from <= context.now &&
  context.now <= principal.valid_to &&
  principal has jurisdiction &&
  resource has jurisdiction_code &&
  resource.jurisdiction_code == principal.jurisdiction
};

permit (
  principal in Regulator::?r,
  action == Action::"read_tenant_lifecycle_history",
  resource is TenantAuditRecord
)
when {
  principal has valid_from &&
  principal has valid_to &&
  context has now &&
  principal.valid_from <= context.now &&
  context.now <= principal.valid_to &&
  principal has jurisdiction &&
  resource has jurisdiction_code &&
  resource.jurisdiction_code == principal.jurisdiction
};

permit (
  principal in Regulator::?r,
  action == Action::"read_audit_chain",
  resource is AuditChainRecord
)
when {
  principal has valid_from &&
  principal has valid_to &&
  context has now &&
  principal.valid_from <= context.now &&
  context.now <= principal.valid_to &&
  principal has jurisdiction &&
  resource has jurisdiction_code &&
  resource.jurisdiction_code == principal.jurisdiction
};

// =============================================================================
// FORBID — auditor cannot write anything
// =============================================================================

forbid (
  principal in Auditor::?a,
  action in [
    Action::"create_tenant",
    Action::"suspend_tenant",
    Action::"resume_tenant",
    Action::"delete_tenant",
    Action::"submit_dsr_request",
    Action::"alter_rls_policy",
    Action::"disable_rls",
    Action::"rotate_jwt_signing_key",
    Action::"write_metrics",
    Action::"write_logs",
    Action::"write_traces",
    Action::"write_artifact",
    Action::"delete_artifact"
  ],
  resource
);

// =============================================================================
// FORBID — regulator cannot write either (read-only legal compulsion)
// =============================================================================

forbid (
  principal in Regulator::?r,
  action in [
    Action::"create_tenant",
    Action::"suspend_tenant",
    Action::"resume_tenant",
    Action::"delete_tenant",
    Action::"submit_dsr_request",
    Action::"alter_rls_policy",
    Action::"disable_rls",
    Action::"rotate_jwt_signing_key",
    Action::"write_metrics",
    Action::"write_logs",
    Action::"write_traces"
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
//
// Even with a valid token, an auditor cannot pivot to a non-scoped tenant.

forbid (
  principal in Auditor::?a,
  action in [
    Action::"read_tenant",
    Action::"read_tenant_status",
    Action::"read_tenant_lifecycle_history",
    Action::"read_cell_assignment",
    Action::"read_dsr_request",
    Action::"read_dsr_request_history",
    Action::"read_proof_of_erasure",
    Action::"read_erasure_receipt"
  ],
  resource
)
when {
  resource has tenant_id &&
  principal has scoped_tenants &&
  !(resource.tenant_id in principal.scoped_tenants)
};

// =============================================================================
// FORBID — auditor / regulator cannot access SECRET-class data
// =============================================================================
//
// JWT signing keys, Postgres replication password, etc. are out of audit scope.
// Audit evidence is derived facts (audit-chain records + RLS policy text +
// proof-of-erasure), not raw secrets.

forbid (
  principal in Auditor::?a,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};

forbid (
  principal in Regulator::?r,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};

// =============================================================================
// FORBID — regulator cannot pivot outside their jurisdiction
// =============================================================================

forbid (
  principal in Regulator::?r,
  action,
  resource
)
when {
  resource has jurisdiction_code &&
  principal has jurisdiction &&
  resource.jurisdiction_code != principal.jurisdiction
};
