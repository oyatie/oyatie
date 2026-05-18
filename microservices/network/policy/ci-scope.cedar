// network µservice — CI-scope Cedar policy fragment
// Cedar v4.2 LTS — default-deny per ADR-0140
//
// Purpose: govern CI / automation principals (build agents, deployment
//          workflows, gate validators, registry tools, evidence emitters)
//          access to network µservice resources. Distinct from tenant-scope
//          (no tenant_id binding required; CI principals are platform-level).
// Owner:   ops-security + axis-network + ops-sre-reliability
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
// PERMIT 1 — gate validators read structural artifacts (no tenant data)
// =============================================================================

permit (
  principal in Group::"gate-validator",
  action in [
    Action::"read_artifact",
    Action::"read_catalog_record",
    Action::"read_openslo_manifest",
    Action::"read_cedar_policy",
    Action::"read_helm_chart",
    Action::"read_prd",
    Action::"read_adr",
    Action::"read_threat_model",
    Action::"read_dpia",
    Action::"read_compliance_doc",
    Action::"emit_gate_verdict"
  ],
  resource in Artifact::?a
)
when {
  resource has classification &&
  resource.classification in ["INTERNAL_ONLY", "PUBLIC"]
};

// =============================================================================
// PERMIT 2 — build agents publish containers + Helm charts
// =============================================================================

permit (
  principal in Group::"build-agent",
  action in [
    Action::"publish_container_image",
    Action::"publish_helm_chart",
    Action::"sign_artifact",
    Action::"emit_sbom",
    Action::"emit_provenance_attestation"
  ],
  resource in Artifact::?a
)
when {
  resource has microservice &&
  resource.microservice == "network"
};

// =============================================================================
// PERMIT 3 — deployment workflow promotes release pointers (gated by ADR-0130)
// =============================================================================

permit (
  principal in Group::"deployment-workflow",
  action in [
    Action::"read_release_pointer",
    Action::"propose_release_pointer_advance",
    Action::"emit_promotion_eligibility_query"
  ],
  resource in Artifact::?a
)
when {
  resource has microservice &&
  resource.microservice == "network" &&
  context has slo_eligibility_verdict &&
  context.slo_eligibility_verdict == "eligible"
};

// =============================================================================
// PERMIT 4 — bias-audit pipeline reads recruiter-stub audit records (no PII)
// =============================================================================

permit (
  principal in Group::"bias-audit-pipeline",
  action in [
    Action::"read_recruiter_audit_aggregate",
    Action::"compute_disparity_ratio",
    Action::"emit_bias_audit_verdict",
    Action::"read_model_card"
  ],
  resource in Artifact::?a
)
when {
  resource has microservice &&
  resource.microservice == "network" &&
  resource has data_class &&
  resource.data_class in ["AUDIT_AGGREGATE", "MODEL_CARD"]
};

// =============================================================================
// PERMIT 5 — registry tools read + write catalog records
// =============================================================================

permit (
  principal in Group::"registry-tool",
  action in [
    Action::"read_catalog_record",
    Action::"write_catalog_record"
  ],
  resource in Artifact::?a
)
when {
  resource has microservice &&
  resource.microservice == "network"
};

// =============================================================================
// PERMIT 6 — evidence emitters write evidence records (sealed)
// =============================================================================

permit (
  principal in Group::"evidence-emitter",
  action == Action::"write_evidence_record",
  resource in Artifact::?a
)
when {
  resource has microservice &&
  resource.microservice == "network" &&
  resource has data_class &&
  resource.data_class == "AUDIT"
};

// =============================================================================
// FORBID — CI principals can never read tenant data
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class in [
    "PII_IDENTIFYING",
    "BEHAVIORAL_TENANT_PRODUCT",
    "BEHAVIORAL_USER_CONTENT",
    "EMPLOYMENT_RECORD",
    "ENDORSEMENT_RECORD",
    "RELATIONSHIP_GRAPH",
    "INMAIL_BODY",
    "JOB_POSTING",
    "RECRUITER_DECISION_AUDIT",
    "MINOR_PROTECT"
  ] &&
  principal in [Group::"gate-validator", Group::"build-agent", Group::"deployment-workflow", Group::"registry-tool", Group::"evidence-emitter"]
};

// =============================================================================
// FORBID — bias-audit pipeline cannot access non-aggregate records
// =============================================================================

forbid (
  principal in Group::"bias-audit-pipeline",
  action,
  resource
)
when {
  resource has data_class &&
  !(resource.data_class in ["AUDIT_AGGREGATE", "MODEL_CARD", "AUDIT"])
};

// =============================================================================
// FORBID — deployment without SLO eligibility
// =============================================================================

forbid (
  principal in Group::"deployment-workflow",
  action == Action::"propose_release_pointer_advance",
  resource
)
when {
  !(context has slo_eligibility_verdict) ||
  context.slo_eligibility_verdict != "eligible"
};

// =============================================================================
// FORBID — deployment without compliance-evidence freshness
// =============================================================================

forbid (
  principal in Group::"deployment-workflow",
  action == Action::"propose_release_pointer_advance",
  resource
)
when {
  !(context has compliance_evidence_within_24h) ||
  context.compliance_evidence_within_24h == false
};
