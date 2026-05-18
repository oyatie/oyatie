// sheets µservice — auditor-scope policy fragment
// Cedar v4.2 LTS schema
//
// Purpose: enforce read-only, time-boxed, tenant-scoped access for external auditors
//          (SOC 2 Type 2, ISO 27001, HIPAA, KR PIPC, EU DPA, EU AI Act notified body,
//          WCAG accessibility auditor, etc.) during audit windows.
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
// PERMIT — auditor reads scoped tenant Sheets data within JIT token window
// =============================================================================
//
// Auditor JIT tokens issued by OpenBao with:
//   - TTL ≤ 4h
//   - principal claim "Auditor::<firm>-<engagement-id>"
//   - allowed_tenants claim listing audit-target tenant(s) only

permit (
  principal in Auditor::?a,
  action in [
    Action::"read_workbook_session_metadata",
    Action::"read_cell_edit_audit_seal",
    Action::"read_license_gate_audit_seal",
    Action::"read_collab_conflict_audit_seal",
    Action::"read_ai_formula_audit_seal",
    Action::"read_share_acl_audit_seal",
    Action::"read_range_acl_audit_seal",
    Action::"read_xlsx_import_audit_seal",
    Action::"read_xlsx_export_audit_seal",
    Action::"read_formula_engine_version_change_audit_seal",
    Action::"read_jurisdiction_switch_audit_seal",
    Action::"read_connected_query_audit_seal"
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
    Action::"read_cell",
    Action::"read_range",
    Action::"read_workbook_metadata",
    Action::"read_collab_op_payload",
    Action::"read_ai_formula_prompt_content",
    Action::"read_ai_formula_completion_content",
    Action::"read_comment",
    Action::"read_connected_query_result"
  ],
  resource
);

// =============================================================================
// FORBID — auditor cannot write anything
// =============================================================================

forbid (
  principal in Auditor::?a,
  action in [
    Action::"open_workbook_session",
    Action::"save_workbook",
    Action::"write_cell",
    Action::"write_range",
    Action::"write_formula",
    Action::"publish_crdt_op",
    Action::"invoke_ai_formula_draft",
    Action::"invoke_smart_fill",
    Action::"share_workbook",
    Action::"author_range_acl",
    Action::"register_connected_query"
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
// Enforced by Sheets REST middleware; Cedar permits the read, middleware records.
