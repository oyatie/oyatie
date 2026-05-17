// cell µservice — auditor-scope policy fragment
// Cedar v4
//
// Purpose: external auditors (SOC 2 / ISO 27001 / HIPAA / KR-PIPC / GDPR DPA)
//          read time-boxed, tenant-scoped subsets of cell substrate evidence.
// Owner:   ops-security + council-privacy + ops-compliance
// References: threat-model.md T-S-05; cell-boundary.md §"Audit Trail"

// =============================================================================
// Default deny
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — auditor reads tenant-scoped cell evidence within engagement window
// =============================================================================

permit (
  principal in Auditor::?a,
  action in [
    Action::"read_cell_assignment",
    Action::"read_migration_history",
    Action::"read_audit_chain",
    Action::"read_dashboards",
    Action::"read_runbooks",
    Action::"read_decommission_history"
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
// PERMIT 2 — auditor reads pack-level cell topology + policy artifacts
// =============================================================================

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
    "cell-boundary",
    "data-residency",
    "policy-cedar",
    "runbook",
    "adr",
    "prd",
    "multi-region"
  ]
};

// =============================================================================
// PERMIT 3 — auditor reads aggregate cell-utilisation (DP-noise protected)
// =============================================================================

permit (
  principal in Auditor::?a,
  action == Action::"read_metrics",
  resource in MetricSeries::?m
)
when {
  principal has valid_from &&
  principal has valid_to &&
  context has now &&
  principal.valid_from <= context.now &&
  context.now <= principal.valid_to &&
  resource has tenant_id &&
  resource.tenant_id == "tenant:oya-aggregate" &&
  resource has dp_epsilon &&
  resource.dp_epsilon <= 1.0
};

// =============================================================================
// FORBID — auditor cannot write anything
// =============================================================================

forbid (
  principal in Auditor::?a,
  action in [
    Action::"write_cell_metadata",
    Action::"write_cell_assignment",
    Action::"transition_cell_state",
    Action::"decommission_cell",
    Action::"drain_host",
    Action::"rotate_secret",
    Action::"write_artifact",
    Action::"delete_artifact"
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
    Action::"read_cell_assignment",
    Action::"read_migration_history",
    Action::"read_audit_chain",
    Action::"read_dashboards",
    Action::"read_decommission_history"
  ],
  resource
)
when {
  resource has tenant_id &&
  principal has scoped_tenants &&
  !(resource.tenant_id in principal.scoped_tenants)
};

// =============================================================================
// FORBID — auditor cannot access cell-adjacency (re-identification protection)
// =============================================================================

forbid (
  principal in Auditor::?a,
  action == Action::"read_cell_adjacency",
  resource
);

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
