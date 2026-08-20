// notes µservice — auditor-scope policy fragment
// Cedar v4.2; for external/internal auditors (SOC 2, ISO 27001, KR PIPC, GDPR DPO).
//
// Purpose: grant audit-chain + policy + IaC read; refuse note body access;
// refuse AI invocation; refuse cross-tenant access.
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

forbid (
  principal,
  action,
  resource
);

// Auditor reads audit-chain, policy artifacts, evidence topics
permit (
  principal in Auditor::?a,
  action in [
    Action::"read_audit_chain",
    Action::"read_policy_artifact",
    Action::"read_evidence_topic",
    Action::"read_compliance_report",
    Action::"validate_per_microservice_layout",
    Action::"validate_dual_context_isolation",
    Action::"validate_e2e_ai_refusal"
  ],
  resource
)
when {
  principal has scope &&
  resource has microservice &&
  resource.microservice == "notes" &&
  resource has data_class &&
  resource.data_class in [
    "AUDIT",
    "INTERNAL_ONLY",
    "OPERATIONAL"
  ]
};

// Auditor MUST NOT read tenant-content note bodies
forbid (
  principal in Auditor::?a,
  action in [
    Action::"read_note",
    Action::"disclose_note_body",
    Action::"invoke_ai_assist",
    Action::"read_attachment"
  ],
  resource
);

// Auditor MUST NOT read SECRET-class data
forbid (
  principal,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};
