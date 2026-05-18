// foundry-supervisor µservice — auditor-scope policy fragment
// Cedar v4
//
// Purpose: external auditors read time-boxed, tenant-scoped subsets during
//          active engagement (SOC 2 / ISO 27001 / HIPAA / KR-PIPC / GDPR DPA
//          / EU AI Act notified body / etc.).
// Owner:   ops-security + council-privacy + ops-compliance
// References: threat-model.md T-S-05; supervisor-isolation.md §"Audit Trail"
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
// PERMIT 1 — auditor reads tenant-scoped supervisor data within engagement window
// =============================================================================

permit (
  principal in Auditor::?a,
  action in [
    Action::"read_fleet_state",
    Action::"read_deployment_history",
    Action::"read_capability_definition",
    Action::"read_kill_switch_state",
    Action::"read_supervision_event",
    Action::"read_audit_chain",
    Action::"read_dashboards",
    Action::"read_runbooks",
    Action::"read_autonomy_entitlement"
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
// PERMIT 2 — auditor reads pack-level policy + compliance artifacts
// =============================================================================
//
// Threat-model, DPIA, compliance.md, etc. — non-tenant-scoped policy text.

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
    "supervisor-isolation",
    "data-residency",
    "policy-cedar",
    "runbook",
    "capability-definition-schema",
    "adr",
    "prd",
    "fria"
  ]
};

// =============================================================================
// FORBID — auditor cannot write anything
// =============================================================================

forbid (
  principal in Auditor::?a,
  action in [
    Action::"write_fleet_state",
    Action::"write_deployment_history",
    Action::"write_capability_definition",
    Action::"write_autonomy_entitlement",
    Action::"engage_kill_switch",
    Action::"disengage_kill_switch",
    Action::"drain_fleet",
    Action::"delete_supervision_event",
    Action::"rotate_secret",
    Action::"write_artifact"
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
    Action::"read_fleet_state",
    Action::"read_deployment_history",
    Action::"read_capability_definition",
    Action::"read_kill_switch_state",
    Action::"read_supervision_event",
    Action::"read_audit_chain",
    Action::"read_autonomy_entitlement"
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

forbid (
  principal in Auditor::?a,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};
