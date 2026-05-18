// anonymous µservice — auditor-scope policy fragment
// Cedar v4.2 LTS; default-deny.
//
// Purpose: scope what external auditors (SOC 2, ISO 27001, GDPR DPA, KR PIPC,
//          UK ICO) can read. Auditors get audit-trail visibility, NEVER user
//          content. The auditor scope is what allows oyatie to ship a SOC 2
//          Type 2 + ISO 27001 + GDPR DPA-friendly substrate.
// Owner:   ops-security + council-privacy
// Audit:   every auditor read emits oya_anonymous_auditor_read_total{auditor,verdict}
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT — auditor reads audit-chain seals + chain-of-custody records
// =============================================================================

permit (
  principal in Auditor::?p,
  action in [
    Action::"read_audit_chain_seal",
    Action::"read_chain_of_custody",
    Action::"read_legal_process_record_metadata",
    Action::"read_transparency_report",
    Action::"read_moderation_verdict_log",
    Action::"read_classifier_version_history"
  ],
  resource in AuditArtifact::?r
)
when {
  principal has auditor_scope &&
  resource has auditor_visible &&
  resource.auditor_visible == true
};

// =============================================================================
// PERMIT — auditor reads aggregate metrics (no per-user data)
// =============================================================================

permit (
  principal in Auditor::?p,
  action == Action::"read_aggregate_metric",
  resource in MetricArtifact::?r
)
when {
  resource has aggregate_only &&
  resource.aggregate_only == true &&
  resource has k_anonymity_floor_satisfied &&
  resource.k_anonymity_floor_satisfied == true
};

// =============================================================================
// PERMIT — DPA reads legal-process disclosure records (regulator privilege)
// =============================================================================
//
// EU GDPR DPAs (Art. 58) and the UK ICO have inspection rights; KR PIPC
// (PIPA Art. 63) likewise. The legal-process disclosure record metadata is
// visible to the relevant DPA but NEVER the disclosed content.

permit (
  principal in DataProtectionAuthority::?p,
  action in [
    Action::"read_legal_process_record_metadata",
    Action::"read_legal_process_record_disclosure_decision",
    Action::"read_legal_process_user_notice_log"
  ],
  resource in LegalProcessRecord::?r
)
when {
  principal has jurisdiction &&
  resource has jurisdiction &&
  principal.jurisdiction == resource.jurisdiction
};

// =============================================================================
// FORBID — auditor CANNOT read user content (I1)
// =============================================================================

forbid (
  principal in Auditor::?p,
  action in [
    Action::"read_post_body",
    Action::"read_comment_body",
    Action::"read_dm_ciphertext",
    Action::"read_dm_plaintext",
    Action::"read_blinded_commitment_seed",
    Action::"correlate_user_to_post"
  ],
  resource
);

// =============================================================================
// FORBID — auditor CANNOT trigger legal-process disclosure
// =============================================================================

forbid (
  principal in Auditor::?p,
  action == Action::"execute_legal_process_disclosure",
  resource
);
