// foundry-evidence µservice — auditor-scope policy fragment
// Cedar v4
//
// Purpose: scope what an external auditor principal (in a recorded engagement)
//          may read on foundry-evidence; gate plaintext access via per-framework
//          + per-tenant engagement entitlement; never permit write.
// Owner:   council-privacy
// Audit:   every auditor read emits a foundry-evidence-of-audit-of-audits chain event
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — Auditor reads pack metadata + payload_sha within engagement window
// =============================================================================

permit (
  principal in AuditorPrincipal::?a,
  action in [
    Action::"foundry_evidence_query",
    Action::"foundry_evidence_read_payload_hash",
    Action::"foundry_evidence_read_pack_metadata"
  ],
  resource in EvidencePack::?e
)
when {
  principal has engagement_id &&
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has engagement_window_start &&
  principal has engagement_window_end &&
  resource has invocation_ts &&
  resource.invocation_ts >= principal.engagement_window_start &&
  resource.invocation_ts <= principal.engagement_window_end &&
  principal has scoped_frameworks &&
  resource has data_class_set &&
  // auditor may read packs whose data-class is consistent with framework scope
  principal.scoped_frameworks contains resource.framework_tag
};

// =============================================================================
// PERMIT 2 — Auditor reads plaintext only when entitlement explicit
// =============================================================================

permit (
  principal in AuditorPrincipal::?a,
  action == Action::"foundry_evidence_read_plaintext",
  resource in EvidencePack::?e
)
when {
  principal has engagement_id &&
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has plaintext_entitlement &&
  principal.plaintext_entitlement == true &&
  resource has payload_data_class &&
  principal has sensitive_data_entitlements &&
  resource.payload_data_class in principal.sensitive_data_entitlements
};

// =============================================================================
// FORBID — Auditor writes anything, ever
// =============================================================================

forbid (
  principal in AuditorPrincipal::?a,
  action in [
    Action::"foundry_evidence_record",
    Action::"foundry_evidence_retention_override",
    Action::"foundry_evidence_pack_rebuild",
    Action::"foundry_evidence_archive_migrate",
    Action::"foundry_evidence_regulator_export"
  ],
  resource
);

// =============================================================================
// FORBID — Out-of-window read
// =============================================================================

forbid (
  principal in AuditorPrincipal::?a,
  action,
  resource in EvidencePack::?e
)
when {
  resource has invocation_ts &&
  principal has engagement_window_end &&
  resource.invocation_ts > principal.engagement_window_end
};

// =============================================================================
// FORBID — Cross-tenant auditor reads
// =============================================================================

forbid (
  principal in AuditorPrincipal::?a,
  action,
  resource in EvidencePack::?e
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};
