// network µservice — auditor-scope Cedar policy fragment
// Cedar v4.2 LTS — default-deny per ADR-0140 (retired per ADR-0145)
//
// Purpose: govern external + internal auditor access to network µservice
//          artifacts + redacted aggregates. SOC 2 / ISO 27001 / GDPR DPA /
//          KR PIPC / HIPAA OCR / EU AI Act notified body / EU DSA Coordinator
//          / EEOC / NYC DCWP LL144 / CA AG / CO AG / DPDP Board / ANPD / UAE
//          Data Office / SDAIA all flow through this policy.
//
//          Auditors NEVER directly access tenant PII; they access:
//            (a) structural artifacts (PRD, ADR, threat-model, DPIA, ...)
//            (b) per-tenant compliance attestations (signed status)
//            (c) audit-chain seals (hash-only)
//            (d) bias-audit aggregates (no per-individual disclosure)
//          Per-individual reads require four-eyes disclosure flow.
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)
// Owner:   ops-security + ops-compliance + council-privacy + axis-network

// =============================================================================
// Default deny
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — auditor reads structural artifacts (always permitted)
// =============================================================================

permit (
  principal in Group::"auditor",
  action == Action::"read_artifact",
  resource in Artifact::?a
)
when {
  resource has classification &&
  resource.classification in ["INTERNAL_ONLY", "PUBLIC"] &&
  resource has data_class &&
  resource.data_class == "INTERNAL_ONLY"
};

// =============================================================================
// PERMIT 2 — auditor reads compliance attestations
// =============================================================================

permit (
  principal in Group::"auditor",
  action == Action::"read_compliance_attestation",
  resource in Artifact::?a
)
when {
  resource has microservice &&
  resource.microservice == "network" &&
  resource has data_class &&
  resource.data_class == "AUDIT"
};

// =============================================================================
// PERMIT 3 — auditor reads audit-chain seals (hash-only)
// =============================================================================

permit (
  principal in Group::"auditor",
  action == Action::"read_audit_chain_seal",
  resource in Artifact::?a
)
when {
  resource has microservice &&
  resource.microservice == "network" &&
  resource has seal_payload_kind &&
  resource.seal_payload_kind == "hash-only"
};

// =============================================================================
// PERMIT 4 — auditor reads bias-audit aggregates (per release; per protected group)
// =============================================================================
//
// Used by EU AI Act notified body + NYC DCWP + EEOC + ICO + CA AG + CO AG.

permit (
  principal in Group::"auditor",
  action in [
    Action::"read_bias_audit_aggregate",
    Action::"read_recruiter_disparity_ratio",
    Action::"read_eu_ai_act_model_card",
    Action::"read_eu_ai_act_fria",
    Action::"read_ll144_bias_audit_summary",
    Action::"read_ab331_impact_assessment",
    Action::"read_eeoc_4_5_rule_evidence"
  ],
  resource in Artifact::?a
)
when {
  resource has microservice &&
  resource.microservice == "network" &&
  resource has aggregation_level &&
  resource.aggregation_level == "aggregate"
};

// =============================================================================
// PERMIT 5 — auditor reads transparency reports (EU DSA Art. 24)
// =============================================================================

permit (
  principal in Group::"eu-dsa-coordinator",
  action == Action::"read_transparency_report",
  resource in Artifact::?a
)
when {
  resource has microservice &&
  resource.microservice == "network" &&
  resource has report_kind &&
  resource.report_kind == "eu-dsa-art-24"
};

// =============================================================================
// PERMIT 6 — EU AI Act notified body reads conformity-assessment evidence
// =============================================================================

permit (
  principal in Group::"eu-ai-act-notified-body",
  action in [
    Action::"read_conformity_assessment_evidence",
    Action::"read_model_card",
    Action::"read_eu_ai_act_fria",
    Action::"read_post_deployment_monitoring_log"
  ],
  resource in Artifact::?a
)
when {
  resource has microservice &&
  resource.microservice == "network" &&
  resource has high_risk_classification &&
  resource.high_risk_classification == true
};

// =============================================================================
// PERMIT 7 — four-eyes disclosure (per-individual PII read requires 2 principals)
// =============================================================================

permit (
  principal in Group::"compliance-disclosure-officer",
  action == Action::"disclose_individual_record",
  resource in Artifact::?a
)
when {
  context has four_eyes_pair_present &&
  context.four_eyes_pair_present == true &&
  context has four_eyes_pair_distinct &&
  context.four_eyes_pair_distinct == true &&
  context has disclosure_reason_code &&
  context.disclosure_reason_code in [
    "regulator-subpoena",
    "lawful-intercept",
    "court-order",
    "tenant-admin-request-attested",
    "incident-investigation-sealed"
  ]
};

// =============================================================================
// FORBID — auditor cannot read raw PII
// =============================================================================

forbid (
  principal in Group::"auditor",
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class in [
    "PII_IDENTIFYING",
    "EMPLOYMENT_RECORD",
    "ENDORSEMENT_RECORD",
    "INMAIL_BODY",
    "MINOR_PROTECT"
  ]
};

// =============================================================================
// FORBID — four-eyes mis-pairing
// =============================================================================
//
// Detect insider attempt to satisfy both halves of four-eyes. Same-principal
// pair-violation is FM-12 Sev-1 incident.

forbid (
  principal in Group::"compliance-disclosure-officer",
  action == Action::"disclose_individual_record",
  resource
)
when {
  !(context has four_eyes_pair_present) ||
  context.four_eyes_pair_present == false ||
  !(context has four_eyes_pair_distinct) ||
  context.four_eyes_pair_distinct == false
};

// =============================================================================
// FORBID — auditor cannot read minor-protect records under any path
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  resource has minor_protect &&
  resource.minor_protect == true &&
  principal in [Group::"auditor", Group::"eu-dsa-coordinator", Group::"eu-ai-act-notified-body"]
};
