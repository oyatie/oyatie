// ontology µservice — auditor-scope policy fragment
// Cedar v4
//
// Purpose: external auditors (SOC 2 / ISO 27001 / HIPAA / KR-PIPC / GDPR DPA / etc.)
//          read time-boxed, tenant-scoped subsets of Ontology entities when
//          an engagement is active.
// Owner:   ops-security + council-privacy + ops-compliance
// References: threat-model.md T-S-05, T-I-05; type-isolation.md §"Audit Trail"
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
// PERMIT 1 — auditor reads tenant-scoped data within engagement window
// =============================================================================
//
// Auditor principals are issued JIT tokens by OpenBao at engagement start.
// Token carries:
//   - engagement_id            unique per audit cycle
//   - scoped_tenants           list of tenant_ids in scope
//   - valid_from / valid_to    ISO8601 timestamps
//   - audit_framework          one of [soc2, iso27001, hipaa, gdpr-dpa, kr-pipc, ...]
//   - max_sensitivity_ceiling         max property sensitivity level the auditor may read

permit (
  principal in Auditor::?a,
  action in [
    Action::"read_object_type",
    Action::"read_link_type",
    Action::"read_function_result",
    Action::"read_action_receipt",
    Action::"read_audit_chain",
    Action::"read_object_type_schema",
    Action::"read_link_type_schema",
    Action::"read_action_type_schema",
    Action::"read_function_type_schema",
    Action::"read_dashboards",
    Action::"read_runbooks",
    Action::"read_compliance_evidence"
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
  resource.tenant_id in principal.scoped_tenants &&
  resource has property_sensitivity_level &&
  principal has max_sensitivity_ceiling &&
  principal.max_sensitivity_ceiling >= resource.property_sensitivity_level
};

// =============================================================================
// PERMIT 2 — auditor reads pack-level policy artifacts (no tenant scoping)
// =============================================================================
//
// Auditors may read per-pack overlay docs (threat-model, dpia, compliance,
// runbooks, ADRs, PRDs) without per-tenant scoping. These are policy text +
// structural information, not customer data.

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
    "type-isolation",
    "data-residency",
    "policy-cedar",
    "runbook",
    "object-type-schema",
    "link-type-schema",
    "action-type-schema",
    "function-type-schema",
    "adr",
    "prd"
  ]
};

// =============================================================================
// FORBID — auditor cannot write anything
// =============================================================================

forbid (
  principal in Auditor::?a,
  action in [
    Action::"write_object_type",
    Action::"write_link_type",
    Action::"invoke_action_type",
    Action::"write_object_type_schema",
    Action::"write_link_type_schema",
    Action::"write_action_type_schema",
    Action::"write_function_type_schema",
    Action::"write_cedar_fragment",
    Action::"write_artifact",
    Action::"delete_object_type",
    Action::"delete_artifact",
    Action::"rotate_secret",
    Action::"issue_cross_pillar_grant",
    Action::"issue_cross_tenant_link_grant"
  ],
  resource
);

// =============================================================================
// FORBID — auditor cannot read outside engagement window
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
// FORBID — auditor cannot pivot beyond scoped_tenants
// =============================================================================

forbid (
  principal in Auditor::?a,
  action in [
    Action::"read_object_type",
    Action::"read_link_type",
    Action::"read_function_result",
    Action::"read_action_receipt",
    Action::"read_audit_chain"
  ],
  resource
)
when {
  resource has tenant_id &&
  principal has scoped_tenants &&
  !(resource.tenant_id in principal.scoped_tenants)
};

// =============================================================================
// FORBID — auditor cannot access SECRET-class data
// =============================================================================
//
// SECRET-class data (signing keys, API keys, etc.) is out of audit scope.
// Audit evidence is derived facts (audit-chain records), not raw secrets.

forbid (
  principal in Auditor::?a,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};

// =============================================================================
// FORBID — auditor cannot exceed max_sensitivity_ceiling
// =============================================================================
//
// Some audits (e.g., baseline SOC 2) operate at lower tier ceilings than
// the deepest HIPAA + PHI audit. Token-time ceiling enforced regardless of
// scoped tenants.

forbid (
  principal in Auditor::?a,
  action,
  resource
)
when {
  resource has property_sensitivity_level &&
  principal has max_sensitivity_ceiling &&
  principal.max_sensitivity_ceiling < resource.property_sensitivity_level
};
