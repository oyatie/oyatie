// workflow-engine µservice — auditor-scope policy fragment
// Cedar v4 schema
//
// Purpose: enforce read-only, time-boxed, tenant-scoped access for external auditors
//          (SOC 2 Type 2, ISO 27001, HIPAA, KR PIPC, EU DPA, etc.) during audit windows.
// Owner:   ops-security + council-privacy
// References: threat-model.md §"T-S-05" + §"T-L-05" (auditor JIT token + cross-tenant pivot)
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny.
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT — auditor reads scoped tenant data within JIT token window
// =============================================================================
//
// Auditor JIT tokens are issued by OpenBao with:
//   - TTL ≤ 4h
//   - principal claim "Auditor::<firm>-<engagement-id>"
//   - allowed_tenants claim listing the audit-target tenant(s) only

permit (
  principal in Auditor::?a,
  action in [
    Action::"read_workflow_spec_metadata",
    Action::"read_workflow_run",
    Action::"read_step_execution",
    Action::"read_audit_chain_seal",
    Action::"read_run_analytics"
  ],
  resource in TenantData::?r
)
when {
  principal has allowed_tenants &&
  resource has tenant_id &&
  resource.tenant_id in principal.allowed_tenants
};

// =============================================================================
// FORBID — auditor cannot pivot to non-scoped tenants
// =============================================================================

forbid (
  principal in Auditor::?a,
  action,
  resource
)
when {
  resource has tenant_id &&
  !(resource.tenant_id in principal.allowed_tenants)
};

// =============================================================================
// FORBID — auditor cannot read sensitive step payloads (PHI / PII raw)
// =============================================================================

forbid (
  principal in Auditor::?a,
  action == Action::"inspect_step_payload",
  resource
);

// =============================================================================
// FORBID — auditor cannot replay workflow runs (side-effecting)
// =============================================================================

forbid (
  principal in Auditor::?a,
  action in [
    Action::"replay_workflow_run",
    Action::"step_through_run",
    Action::"submit_workflow_spec",
    Action::"start_workflow_run",
    Action::"cancel_workflow_run"
  ],
  resource
);

// =============================================================================
// FORBID — auditor cannot read SECRET-class
// =============================================================================

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
// AUDIT — every auditor read is itself audit-emitted (audit-chain of audits)
// =============================================================================
//
// Implementation note: this is enforced by the engine REST middleware, not by
// Cedar policy semantics directly. Cedar permits the read; middleware records.
