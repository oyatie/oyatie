// cell µservice — public-read policy fragment
// Cedar v4
//
// Purpose: govern anonymous / unauthenticated access to non-sensitive cell aggregates
//          (e.g., the public capacity status page, the public OpenSLO-style cell-health
//          summary).
// Owner:   ops-security + gtm-marketing
// References: threat-model.md T-I-04, T-L-05; dpia.md §"Aggregate Aggregations"
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
// PERMIT 1 — anonymous read of platform cell-health (oya-self tenant, PUBLIC class)
// =============================================================================
//
// The public status page reads platform-wide cell-health indicators (e.g., "cell
// substrate availability over the last 30 days") with no per-customer attribution.

permit (
  principal == User::"anonymous",
  action == Action::"read_metrics",
  resource in MetricSeries::?m
)
when {
  resource has tenant_id &&
  resource.tenant_id == "tenant:oya-self" &&
  resource has data_class &&
  resource.data_class == "PUBLIC" &&
  resource has metric_name &&
  resource.metric_name like "oya_cell_public_*"
};

// =============================================================================
// PERMIT 2 — anonymous read of DP-noise-protected aggregate capacity
// =============================================================================

permit (
  principal == User::"anonymous",
  action == Action::"read_metrics",
  resource in MetricSeries::?m
)
when {
  resource has tenant_id &&
  resource.tenant_id == "tenant:oya-aggregate" &&
  resource has data_class &&
  resource.data_class == "PUBLIC" &&
  resource has dp_epsilon &&
  resource.dp_epsilon <= 1.0 &&
  resource has metric_name &&
  resource.metric_name like "oya_cell_capacity_*"
};

// =============================================================================
// FORBID — anonymous cannot read any non-public class
// =============================================================================

forbid (
  principal == User::"anonymous",
  action,
  resource
)
when {
  resource has data_class &&
  !(resource.data_class == "PUBLIC")
};

// =============================================================================
// FORBID — anonymous cannot read any customer-tenant data
// =============================================================================

forbid (
  principal == User::"anonymous",
  action,
  resource
)
when {
  resource has tenant_id &&
  !(resource.tenant_id in ["tenant:oya-self", "tenant:oya-aggregate"])
};

// =============================================================================
// FORBID — anonymous cannot write anything
// =============================================================================

forbid (
  principal == User::"anonymous",
  action in [
    Action::"write_cell_metadata",
    Action::"write_cell_assignment",
    Action::"transition_cell_state",
    Action::"decommission_cell",
    Action::"drain_host",
    Action::"write_artifact"
  ],
  resource
);

// =============================================================================
// FORBID — anonymous cannot read cell adjacency (re-identification protection)
// =============================================================================

forbid (
  principal == User::"anonymous",
  action == Action::"read_cell_adjacency",
  resource
);
