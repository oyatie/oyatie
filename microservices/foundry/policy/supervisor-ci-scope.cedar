// foundry-supervisor µservice — CI-scope policy fragment
// Cedar v4
//
// Purpose: scope the oya-ci reserved tenant's access; forbid customer-tenant data reads.
// Owner:   axis-foundry-control-plane + axis-observability + ops-security
// References: supervisor-isolation.md §"Reserved tenant IDs"
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
// PERMIT 1 — CI lane reads HG-FND-SUP claim metrics + e2e drill artifacts
// =============================================================================

permit (
  principal == Service::"oya-ci",
  action in [Action::"read_metric", Action::"read_artifact"],
  resource in CIResource::?r
)
when {
  resource has metric_name &&
  (
    resource.metric_name like "oya_supervisor_*" ||
    resource.metric_name == "oya:current_supervisor_verdict:by_tenant_capability" ||
    resource.metric_name == "oya:kill_switch_coverage:by_tenant"
  ) &&
  resource has tenant_id &&
  resource.tenant_id in ["tenant_oya_ci", "tenant_oya_aggregate", "tenant_oya_self"]
};

// =============================================================================
// PERMIT 2 — supervisor controller writes self-SLI metrics
// =============================================================================

permit (
  principal == Service::"foundry-supervisor-controller",
  action == Action::"write_metric",
  resource in MetricSeries::?m
)
when {
  resource has metric_name &&
  resource.metric_name like "oya_supervisor_*" &&
  resource has tenant_id &&
  resource.tenant_id in ["tenant_oya_self", "tenant_oya_ci"]
};

// =============================================================================
// PERMIT 3 — supervisor worker emits aggregate (DP-noise-protected)
// =============================================================================

permit (
  principal == Service::"foundry-supervisor-aggregator",
  action == Action::"write_metric",
  resource in MetricSeries::?m
)
when {
  resource has tenant_id &&
  resource.tenant_id == "tenant_oya_aggregate" &&
  resource has dp_epsilon &&
  resource.dp_epsilon <= 1.0
};

// =============================================================================
// FORBID — CI principal cannot read any customer-tenant data
// =============================================================================

forbid (
  principal == Service::"oya-ci",
  action,
  resource
)
when {
  resource has tenant_id &&
  !(resource.tenant_id in ["tenant_oya_ci", "tenant_oya_aggregate", "tenant_oya_self"])
};

// =============================================================================
// FORBID — CI principal cannot write to non-reserved tenants
// =============================================================================

forbid (
  principal == Service::"oya-ci",
  action in [Action::"write_metric", Action::"write_artifact"],
  resource
)
unless {
  resource has tenant_id &&
  resource.tenant_id in ["tenant_oya_ci", "tenant_oya_aggregate"]
};

// =============================================================================
// FORBID — CI principal cannot engage kill-switch on customer tenants
// =============================================================================
//
// CI may test kill-switch only against synthetic tenant_oya_ci capabilities.

forbid (
  principal == Service::"oya-ci",
  action in [Action::"engage_kill_switch", Action::"disengage_kill_switch"],
  resource
)
unless {
  resource has tenant_id &&
  resource.tenant_id == "tenant_oya_ci"
};

// =============================================================================
// FORBID — CI cannot mutate autonomy entitlements
// =============================================================================

forbid (
  principal == Service::"oya-ci",
  action in [
    Action::"write_autonomy_entitlement",
    Action::"escalate_autonomy_level"
  ],
  resource
);
