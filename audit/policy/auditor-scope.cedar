// audit-chain µservice — auditor-scope policy fragment
// Cedar v4
//
// Purpose: external auditors (SOC 2 / ISO 27001 / HIPAA OCR / KR PIPC / EU DPA / etc.)
//          read time-boxed, tenant-scoped, framework-scoped subsets of the chain.
// Owner:   ops-security + council-privacy + ops-compliance
// References: threat-model.md T-S-03, T-I-03; tenant-isolation analogue from observability
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
// PERMIT 1 — Auditor reads tenant-scoped chain within engagement window
// =============================================================================
//
// Auditor JIT tokens issued by OpenBao carry:
//   - engagement_id        unique per audit cycle
//   - scoped_tenants       list of tenant_ids in scope
//   - scoped_packs         list of packs (e.g., [pack-eu] for an EU DPA inquiry)
//   - scoped_event_classes filter (e.g., [PromotionExecuted, RollbackExecuted] for SOC 2 CC8.x)
//   - valid_from / valid_to
//   - audit_framework      [soc2, iso27001, hipaa, gdpr-dpa, kr-pipc, ...]

permit (
  principal in Auditor::?a,
  action in [
    Action::"audit_query",
    Action::"audit_verify",
    Action::"audit_read_proof",
    Action::"audit_export_request",
    Action::"audit_read_seal_record",
    Action::"audit_read_root_manifest",
    Action::"audit_read_key_manifest"
  ],
  resource in AuditEvent::?e
)
when {
  principal has valid_from &&
  principal has valid_to &&
  context has now &&
  principal.valid_from <= context.now &&
  context.now <= principal.valid_to &&
  resource has tenant_id &&
  principal has scoped_tenants &&
  resource.tenant_id in principal.scoped_tenants &&
  resource has pack &&
  principal has scoped_packs &&
  resource.pack in principal.scoped_packs
};

// =============================================================================
// PERMIT 2 — Auditor reads policy artefacts (this µservice's policy docs)
// =============================================================================

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
    "seal-integrity",
    "data-residency",
    "policy-cedar",
    "runbook",
    "adr",
    "prd",
    "phase-spec"
  ]
};

// =============================================================================
// PERMIT 3 — Auditor receives signed export bundle
// =============================================================================
//
// The bundle is constructed by query-rest in the auditor's scoped pack; it is
// signed by the pack HSM; auditor receives the bundle + verification SDK to
// independently validate.

permit (
  principal in Auditor::?a,
  action == Action::"audit_receive_export_bundle",
  resource in ExportBundle::?b
)
when {
  principal has engagement_id &&
  resource has engagement_id &&
  principal.engagement_id == resource.engagement_id &&
  context has now &&
  principal has valid_from &&
  principal has valid_to &&
  principal.valid_from <= context.now &&
  context.now <= principal.valid_to
};

// =============================================================================
// FORBID — Auditor cannot write or mutate anything
// =============================================================================

forbid (
  principal in Auditor::?a,
  action in [
    Action::"audit_emit",
    Action::"wal_write",
    Action::"seal_record_write",
    Action::"audit_redact",
    Action::"audit_emergency_redact",
    Action::"audit_retention_override",
    Action::"hsm_sign",
    Action::"hsm_key_generate",
    Action::"hsm_key_rotate",
    Action::"hsm_key_retire",
    Action::"hsm_key_destroy",
    Action::"hsm_key_revoke",
    Action::"write_artifact",
    Action::"delete_artifact"
  ],
  resource
);

// =============================================================================
// FORBID — Auditor cannot read outside engagement window
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
// FORBID — Auditor cannot pivot beyond scoped_tenants
// =============================================================================

forbid (
  principal in Auditor::?a,
  action in [
    Action::"audit_query",
    Action::"audit_verify",
    Action::"audit_read_proof",
    Action::"audit_export_request",
    Action::"audit_read_seal_record"
  ],
  resource in AuditEvent::?e
)
when {
  resource has tenant_id &&
  principal has scoped_tenants &&
  !(resource.tenant_id in principal.scoped_tenants)
};

// =============================================================================
// FORBID — Auditor cannot pivot beyond scoped_packs (residency)
// =============================================================================

forbid (
  principal in Auditor::?a,
  action,
  resource in AuditEvent::?e
)
when {
  resource has pack &&
  principal has scoped_packs &&
  !(resource.pack in principal.scoped_packs)
};

// =============================================================================
// FORBID — Auditor cannot access SECRET-class events (keys, secrets)
// =============================================================================

forbid (
  principal in Auditor::?a,
  action,
  resource in AuditEvent::?e
)
when {
  resource has payload_data_class &&
  resource.payload_data_class == "SECRET"
};
