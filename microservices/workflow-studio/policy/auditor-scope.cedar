// workflow-studio µservice — auditor-scope policy fragment
// Cedar v4 schema
//
// Purpose: enforce read-only, time-boxed, tenant-scoped access for external auditors
//          (SOC 2 Type 2, ISO 27001, HIPAA, KR PIPC, EU DPA, EU AI Act notified body, etc.)
//          during audit windows.
// Owner:   ops-security + council-privacy
// References: threat-model.md §"T-E-02" (auditor JIT token + cross-tenant pivot)
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
// PERMIT — auditor reads scoped tenant Studio data within JIT token window
// =============================================================================
//
// Auditor JIT tokens issued by OpenBao with:
//   - TTL ≤ 4h
//   - principal claim "Auditor::<firm>-<engagement-id>"
//   - allowed_tenants claim listing audit-target tenant(s) only

permit (
  principal in Auditor::?a,
  action in [
    Action::"read_editor_session_metadata",
    Action::"read_save_audit_seal",
    Action::"read_license_gate_audit_seal",
    Action::"read_collab_conflict_audit_seal",
    Action::"read_llm_assist_audit_seal",
    Action::"read_jurisdiction_switch_audit_seal"
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
// FORBID — auditor cannot read tenant draft contents (only audit seals + metadata)
// =============================================================================

forbid (
  principal in Auditor::?a,
  action in [
    Action::"read_workflow_definition_draft",
    Action::"read_llm_assist_prompt_content",
    Action::"read_llm_assist_completion_content",
    Action::"read_collab_op_payload"
  ],
  resource
);

// =============================================================================
// FORBID — auditor cannot write anything
// =============================================================================

forbid (
  principal in Auditor::?a,
  action in [
    Action::"open_editor_session",
    Action::"save_workflow_definition",
    Action::"publish_crdt_op",
    Action::"invoke_llm_assist",
    Action::"open_debugger_session",
    Action::"step_through_run_frontend"
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
// Enforced by Studio REST middleware; Cedar permits the read, middleware records.
