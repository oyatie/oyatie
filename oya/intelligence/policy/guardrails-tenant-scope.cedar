// foundry-guardrails µservice — tenant-scope policy fragment
// Cedar v4 schema; conforms to microservices/intelligence/policy/schema.cedarschema
//
// Purpose: enforce per-tenant scope on classify / validate / autonomy decisions,
//          Cedar overlay reads, decision-detail reads, FP-budget operations.
// Owner:   ops-security + axis-foundry-guardrails
// Audit:   every deny emits oya_tenant_unauthorized_read_attempt_total
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny — every action refused unless an explicit permit matches.
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — foundry-runtime in-cluster caller invokes classify / validate / autonomy
// =============================================================================
//
// Only sanctioned in-cluster caller; identified by SPIFFE.

permit (
  principal == Service::"spiffe://oyatie/foundry-runtime/invocation-orchestrator",
  action in [
    Action::"classify_prompt",
    Action::"validate_output",
    Action::"enforce_autonomy"
  ],
  resource is GuardrailDecision
);

// =============================================================================
// PERMIT 2 — tenant operator reads own tenant's decisions + FP budget + overlays
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"read_decision_detail",
    Action::"read_false_positive_budget",
    Action::"list_tenant_overlays",
    Action::"read_tenant_overlay"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 3 — tenant operator marks own decision as false-positive
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action == Action::"mark_false_positive",
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  resource has decision_kind &&
  resource.decision_kind == "block"   // only blocks can be marked FP
};

// =============================================================================
// PERMIT 4 — tenant operator with DPA-authorised sensitive-data entitlement
// =============================================================================
//
// pack-us-healthcare PHI prompts require explicit BAA + Cedar entitlement.
// pack-kr sensitive-data reads (PIPA Art. 23) require explicit consent.

permit (
  principal in TenantOperator::?p,
  action == Action::"read_decision_detail",
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has sensitive_data_entitlements &&
  resource has data_class &&
  resource.data_class in principal.sensitive_data_entitlements
};

// =============================================================================
// FORBID — explicit cross-tenant deny (defence-in-depth)
// =============================================================================
//
// Cedar deny-overrides semantics; this catches any accidental permit.

forbid (
  principal,
  action in [
    Action::"read_decision_detail",
    Action::"read_false_positive_budget",
    Action::"list_tenant_overlays",
    Action::"read_tenant_overlay",
    Action::"mark_false_positive"
  ],
  resource
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// FORBID — never expose SECRET-class data via tenant reads
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};
