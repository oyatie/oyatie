// social µservice — auditor-scope policy fragment
// Cedar v4
//
// Purpose: external auditors (SOC 2 / ISO 27001 / HIPAA / KR-PIPC / GDPR DPA /
//          EU DSA Coordinator / EU AI Act notified body) read time-boxed,
//          tenant-scoped subsets of profile + post + audit + classifier data.
// Owner:   ops-security + council-privacy + ops-compliance
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
// PERMIT 1 — auditor reads tenant-scoped audit-chain + policy artifacts
// =============================================================================

permit (
  principal in Auditor::?a,
  action in [
    Action::"read_audit_chain",
    Action::"read_profile_metadata",
    Action::"read_retention_policy",
    Action::"read_disclosure_history",
    Action::"read_ediscovery_hold_history",
    Action::"read_moderation_verdict_history",
    Action::"read_appeal_history",
    Action::"read_classifier_evidence",
    Action::"read_dsa_transparency_record",
    Action::"read_artifact"
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
// PERMIT 2 — auditor reads pack-level policy artifacts (no per-tenant scope)
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
    "dual-context-isolation",
    "data-residency",
    "policy-cedar",
    "runbook",
    "openslo-manifest",
    "adr",
    "prd",
    "transfer-register",
    "sub-processor-list",
    "model-card",
    "eu-ai-act-evidence",
    "dsa-transparency-report"
  ]
};

// =============================================================================
// FORBID — auditor cannot read post bodies
// =============================================================================
//
// Auditors verify control existence + audit-chain integrity; they do NOT read
// raw post bodies (which would itself be a privacy breach). Auditors read
// audit-chain seals (hashes + timestamps) and policy artifacts.

forbid (
  principal in Auditor::?a,
  action in [
    Action::"read_post",
    Action::"read_thread",
    Action::"read_media_blob",
    Action::"disclose_post_body"
  ],
  resource
);

// =============================================================================
// FORBID — auditor cannot read PII age-attestation table
// =============================================================================

forbid (
  principal in Auditor::?a,
  action == Action::"read_age_attestation",
  resource
);

// =============================================================================
// FORBID — auditor cannot write anything
// =============================================================================

forbid (
  principal in Auditor::?a,
  action in [
    Action::"publish_post",
    Action::"edit_post",
    Action::"delete_post",
    Action::"react_to_post",
    Action::"follow",
    Action::"upload_media",
    Action::"open_ediscovery_hold",
    Action::"close_ediscovery_hold",
    Action::"issue_moderation_verdict",
    Action::"rotate_secret"
  ],
  resource
);

// =============================================================================
// FORBID — outside engagement window
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
// FORBID — outside scoped_tenants
// =============================================================================

forbid (
  principal in Auditor::?a,
  action,
  resource
)
when {
  resource has tenant_id &&
  principal has scoped_tenants &&
  !(resource.tenant_id in principal.scoped_tenants)
};

// =============================================================================
// FORBID — SECRET-class data is out of audit scope
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
