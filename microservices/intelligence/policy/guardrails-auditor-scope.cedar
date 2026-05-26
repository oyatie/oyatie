// foundry-guardrails µservice — auditor-scope policy fragment
// Cedar v4
//
// Purpose: time-boxed JIT auditor read access to decisions + audit-chain seals,
//          scoped to the audit-target tenants (NOT all tenants), and refused
//          on SECRET-class material.
// Owner: ops-security + council-privacy
// Audit: every auditor read is itself audit-emitted (audit-of-audits).
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
// PERMIT 1 — auditor reads scoped tenant decisions during engagement window
// =============================================================================

permit (
  principal in Auditor::?a,
  action in [
    Action::"read_decision_detail",
    Action::"read_decision_history",
    Action::"read_audit_chain_seal"
  ],
  resource in TenantData::?r
)
when {
  // engagement_window + scoped_tenants from OpenBao JIT issuance
  principal has engagement_active &&
  principal.engagement_active == true &&
  principal has scoped_tenants &&
  resource has tenant_id &&
  resource.tenant_id in principal.scoped_tenants &&
  // TTL: ≤ 4h non-renewable
  principal has token_expires_at &&
  principal.token_expires_at > context.now
};

// =============================================================================
// PERMIT 2 — auditor reads classifier-model deployment + rule-mutation history
// (these are non-tenant-scoped operational records; auditor can verify
//  policies-as-deployed at engagement-window time)
// =============================================================================

permit (
  principal in Auditor::?a,
  action in [
    Action::"read_classifier_model_deploy_history",
    Action::"read_rule_mutation_history",
    Action::"read_cedar_bundle_history"
  ],
  resource is OperationalRecord
)
when {
  principal has engagement_active &&
  principal.engagement_active == true &&
  principal has token_expires_at &&
  principal.token_expires_at > context.now
};

// =============================================================================
// FORBID — auditor cannot pivot beyond scoped tenants
// =============================================================================

forbid (
  principal in Auditor::?a,
  action in [
    Action::"read_decision_detail",
    Action::"read_decision_history",
    Action::"read_audit_chain_seal"
  ],
  resource in TenantData::?r
)
unless {
  principal has scoped_tenants &&
  resource has tenant_id &&
  resource.tenant_id in principal.scoped_tenants
};

// =============================================================================
// FORBID — auditor cannot mutate (read-only by design)
// =============================================================================

forbid (
  principal in Auditor::?a,
  action in [
    Action::"write_decision",
    Action::"write_rule_definition",
    Action::"write_cedar_fragment_registry",
    Action::"mark_false_positive",
    Action::"deploy_classifier_model"
  ],
  resource
);

// =============================================================================
// FORBID — never expose SECRET-class material to auditors
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
