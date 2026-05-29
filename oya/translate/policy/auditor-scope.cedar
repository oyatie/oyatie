// translate µservice — auditor-scope policy fragment
// Cedar v4.2
//
// Purpose: external auditor read scope (SOC 2 / ISO 27001 / DPA / HIPAA OCR /
//          PIPC / EU DPA / EU AI Office / etc.).
// Owner:   council-privacy + ops-security
// Audit:   every read emits oya_translate_auditor_read_total
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — auditor reads audit-chain events (TranslationCompleted etc.)
// =============================================================================

permit (
  principal in Auditor::?p,
  action in [
    Action::"audit.read_events",
    Action::"audit.verify_envelope_signature",
    Action::"audit.read_evidence"
  ],
  resource in AuditScope::?a
)
when {
  principal has audit_scope_id &&
  resource has audit_scope_id &&
  principal.audit_scope_id == resource.audit_scope_id &&
  principal has framework &&
  principal.framework in ["SOC2", "ISO27001", "HIPAA", "GDPR", "EU-AI-Act", "PIPA", "DPDPA", "LGPD", "APPI", "PDPL"]
};

// =============================================================================
// PERMIT 2 — auditor reads policy + ADR artifacts
// =============================================================================

permit (
  principal in Auditor::?p,
  action == Action::"docs.read",
  resource in DocsArtifact::?d
)
when {
  resource has classification &&
  resource.classification in ["INTERNAL_ONLY", "PUBLIC"]
};

// =============================================================================
// PERMIT 3 — auditor exports per-pack DPIA / threat-model / compliance
// =============================================================================

permit (
  principal in Auditor::?p,
  action == Action::"docs.export",
  resource in DocsArtifact::?d
)
when {
  principal has audit_request_id &&
  resource has doc_class &&
  resource.doc_class in ["DPIA", "ThreatModel", "ComplianceSpec", "PolicySpec", "AdrSpec"]
};

// =============================================================================
// PERMIT 4 — auditor reviews EU AI Act disclosure record subset
// =============================================================================

permit (
  principal in Auditor::?p,
  action == Action::"eu_ai_act_disclosure.read",
  resource in AuditScope::?a
)
when {
  principal has framework &&
  principal.framework == "EU-AI-Act" &&
  resource has pack &&
  resource.pack == "eu"
};

// =============================================================================
// FORBID — auditor NEVER reads tenant content directly
// =============================================================================

forbid (
  principal in Auditor::?p,
  action in [
    Action::"translate.invoke",
    Action::"tm.lookup_value",
    Action::"doc.read_content",
    Action::"stream.open",
    Action::"bulk.read_chunk_content"
  ],
  resource
);

// =============================================================================
// FORBID — auditor NEVER resolves credentials
// =============================================================================

forbid (
  principal in Auditor::?p,
  action == Action::"credential.resolve",
  resource
);
