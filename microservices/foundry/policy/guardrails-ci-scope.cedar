// foundry-guardrails µservice — CI-scope policy fragment
// Cedar v4
//
// Purpose: scope the oya-ci reserved tenant's access to:
//          - rule-mutation operations (via PR-bound CI runner)
//          - shadow-vs-enforce delta read for promote-readiness gate
//          - classifier-model deployment events
//          - audit-chain seal lookups for compliance lane
//          NOT customer-tenant decision data.
// Owner: axis-foundry + axis-foundry-guardrails + ops-security
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
// PERMIT 1 — CI rule-mutation writer (bound to PR + signed-commit SHA)
// =============================================================================

permit (
  principal == Service::"oya-ci",
  action in [
    Action::"write_rule_definition",
    Action::"write_cedar_fragment_registry"
  ],
  resource is RuleStoreEntry
)
when {
  // PR merge SHA + author SPIFFE recorded in CI invocation context.
  principal has pr_sha &&
  resource has author_spiffe
};

// =============================================================================
// PERMIT 2 — CI shadow-vs-enforce delta read for promote-readiness gate
// =============================================================================

permit (
  principal == Service::"oya-ci",
  action == Action::"read_shadow_enforce_delta",
  resource is ShadowEnforceDelta
);

// =============================================================================
// PERMIT 3 — CI classifier-model deployment audit lookup
// =============================================================================

permit (
  principal == Service::"oya-ci",
  action == Action::"read_classifier_model_deploy_history",
  resource is ClassifierDeploymentRecord
);

// =============================================================================
// PERMIT 4 — CI compliance-evidence lane reads audit-chain seal aggregates
// =============================================================================

permit (
  principal == Service::"oya-ci",
  action == Action::"read_audit_chain_seal_aggregate",
  resource in MetricSeries::?m
)
when {
  resource has metric_name &&
  (
    resource.metric_name like "foundry_guardrails_audit_chain_seal_*" ||
    resource.metric_name like "foundry_guardrails_compliance_evidence_*"
  ) &&
  resource has tenant_id &&
  resource.tenant_id in ["tenant:oya-ci", "tenant:oya-aggregate", "tenant:oya-self"]
};

// =============================================================================
// FORBID — CI principal cannot read any customer-tenant decision data
// =============================================================================

forbid (
  principal == Service::"oya-ci",
  action in [
    Action::"read_decision_detail",
    Action::"read_tenant_overlay",
    Action::"classify_prompt",
    Action::"validate_output"
  ],
  resource
);

// =============================================================================
// FORBID — CI principal cannot write to customer-tenant data
// =============================================================================

forbid (
  principal == Service::"oya-ci",
  action in [
    Action::"write_decision",
    Action::"mark_false_positive",
    Action::"write_tenant_overlay"
  ],
  resource
);
