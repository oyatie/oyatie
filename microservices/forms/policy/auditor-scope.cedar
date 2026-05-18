// forms µservice — auditor-scope policy fragment
// Cedar v4 schema
//
// Purpose: enforce read-only, time-boxed, tenant-scoped access for external auditors
//          (SOC 2 Type 2, ISO 27001, HIPAA, KR PIPC, EU DPA, EU AI Act notified body, etc.)
//          during audit windows.
// Owner:   ops-security + council-privacy
// References: threat-model.md §"T-I-07" (auditor JIT token + cross-tenant pivot)
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
// PERMIT — auditor reads scoped tenant Forms audit data within JIT token window
// =============================================================================
//
// Auditor JIT tokens issued by OpenBao with:
//   - TTL ≤ 4h
//   - principal claim "Auditor::<firm>-<engagement-id>"
//   - allowed_tenants claim listing audit-target tenant(s) only

permit (
  principal in Auditor::?a,
  action in [
    Action::"read_form_publish_audit_seal",
    Action::"read_response_audit_seal",
    Action::"read_form_edit_audit_seal",
    Action::"read_ai_build_audit_seal",
    Action::"read_dsr_audit_seal",
    Action::"read_bulk_distribute_audit_seal",
    Action::"read_export_audit_seal",
    Action::"read_form_metadata",
    Action::"read_response_aggregate_count",
    Action::"read_audit_chain_seal_status"
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
// FORBID — auditor cannot read tenant draft contents or PII responses
//          (only audit seals + metadata)
// =============================================================================

forbid (
  principal in Auditor::?a,
  action in [
    Action::"read_form_definition",
    Action::"read_response_redacted",
    Action::"read_response_with_pii",
    Action::"read_ai_build_prompt_content",
    Action::"read_ai_build_completion_content",
    Action::"export_responses_csv",
    Action::"export_responses_xlsx",
    Action::"export_responses_json"
  ],
  resource
);

// =============================================================================
// FORBID — auditor cannot write anything
// =============================================================================

forbid (
  principal in Auditor::?a,
  action in [
    Action::"create_form",
    Action::"edit_form_draft",
    Action::"publish_form",
    Action::"submit_response",
    Action::"invoke_ai_form_build",
    Action::"bulk_distribute_form_email"
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
// Enforced by Forms REST middleware; Cedar permits the read, middleware records.
