// foundry-evidence µservice — regulator-export-scope policy fragment
// Cedar v4
//
// Purpose: gate the regulator-export capability — framework-filtered,
//          signed evidence-pack bundle assembly. Two-person rule enforced.
// Owner:   council-privacy
// Audit:   every export-issuance audit-emitted
// References: threat-model.md T-E-04, T-S-03, T-I-06; PRD.md FR-06.
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT — RegulatorExporter principal, 2-person rule, valid framework
// =============================================================================

permit (
  principal in RegulatorExporter::?p,
  action == Action::"foundry_evidence_regulator_export",
  resource in RegulatorExportRequest::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&

  // 2-person rule: approver_principal must be distinct from requester_principal
  resource has requester_principal &&
  resource has approver_principal &&
  resource.requester_principal != resource.approver_principal &&

  // approver must also be a RegulatorExporter on the same tenant
  resource has approver_role &&
  resource.approver_role == "RegulatorExporter" &&

  // framework must be one of the six supported profiles
  resource has framework &&
  resource.framework in [
    "eu-ai-act",
    "hipaa",
    "gdpr",
    "kr-pipa",
    "soc2",
    "iso-27001"
  ] &&

  // receiving-bucket must be bound to a tenant-controlled or regulator-controlled SCC
  resource has receiving_bucket_scc_id &&
  resource has dpa_export_plan_id &&

  // time-range must be bounded (no open-ended exports)
  resource has time_range_start &&
  resource has time_range_end
};

// =============================================================================
// PERMIT — AuditorPrincipal in active engagement reads bundle metadata
// =============================================================================

permit (
  principal in AuditorPrincipal::?a,
  action == Action::"foundry_evidence_read_regulator_export_metadata",
  resource in RegulatorExportBundle::?b
)
when {
  principal has engagement_id &&
  resource has engagement_id &&
  principal.engagement_id == resource.engagement_id &&
  principal has framework &&
  resource has framework &&
  principal.framework == resource.framework
};

// =============================================================================
// FORBID — Cross-tenant export
// =============================================================================

forbid (
  principal,
  action == Action::"foundry_evidence_regulator_export",
  resource in RegulatorExportRequest::?r
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// FORBID — Self-approval (defence-in-depth on 2-person rule)
// =============================================================================

forbid (
  principal,
  action == Action::"foundry_evidence_regulator_export",
  resource in RegulatorExportRequest::?r
)
when {
  resource has requester_principal &&
  resource has approver_principal &&
  resource.requester_principal == resource.approver_principal
};

// =============================================================================
// FORBID — Unsupported framework
// =============================================================================

forbid (
  principal,
  action == Action::"foundry_evidence_regulator_export",
  resource in RegulatorExportRequest::?r
)
unless {
  resource has framework &&
  resource.framework in [
    "eu-ai-act",
    "hipaa",
    "gdpr",
    "kr-pipa",
    "soc2",
    "iso-27001"
  ]
};

// =============================================================================
// FORBID — Unbounded time range
// =============================================================================

forbid (
  principal,
  action == Action::"foundry_evidence_regulator_export",
  resource in RegulatorExportRequest::?r
)
unless {
  resource has time_range_start &&
  resource has time_range_end
};

// =============================================================================
// FORBID — Cross-pack export (cross-pack chain merge is forbidden)
// =============================================================================

forbid (
  principal,
  action == Action::"foundry_evidence_regulator_export",
  resource in RegulatorExportRequest::?r
)
when {
  resource has source_pack &&
  resource has receiving_pack &&
  resource.source_pack != resource.receiving_pack &&
  !(resource has cross_pack_scc_attestation_id)
};

// =============================================================================
// FORBID — Pre-signed URL TTL > 5 min
// =============================================================================

forbid (
  principal,
  action == Action::"foundry_evidence_issue_presigned_url",
  resource in PreSignedUrlRequest::?u
)
unless {
  resource has ttl_seconds &&
  resource.ttl_seconds <= 300
};
