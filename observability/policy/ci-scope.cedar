// observability µservice — CI-scope policy fragment
// Cedar v4
//
// Purpose: scope the oya-ci reserved tenant's access to promotion-readiness queries only;
//          forbid reads of customer-tenant per-series data.
// Owner:   axis-foundry + axis-observability + ops-security
// References: tenant-isolation.md §"Reserved tenant IDs", §"Invariant TI-06"
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
// PERMIT 1 — CI lane reads promotion-eligibility verdicts only
// =============================================================================
//
// The oya-governance-promotion-readiness CI lane (running as principal Service::"oya-ci")
// may read the aggregated verdict metrics from the oya-aggregate or oya-ci
// reserved tenants. It MAY NOT read customer-tenant per-series data.

permit (
  principal == Service::"oya-ci",
  action == Action::"read_metrics",
  resource in MetricSeries::?m
)
when {
  resource has metric_name &&
  (
    resource.metric_name like "oya_promotion_*" ||
    resource.metric_name == "oya:current_verdict:by_microservice_env" ||
    resource.metric_name == "oya:all_eligible:by_sha"
  ) &&
  resource has tenant_id &&
  resource.tenant_id in ["tenant:oya-ci", "tenant:oya-aggregate", "tenant:oya-self"]
};

// =============================================================================
// PERMIT 2 — slo-engine-worker writes verdict metrics
// =============================================================================
//
// The worker (SPIFFE: spiffe://oyatie/observability/slo-engine-worker)
// is the sole authorised writer of oya_promotion_* metrics into tenant:oya-ci.

permit (
  principal == Service::"slo-engine-worker",
  action == Action::"write_metrics",
  resource in MetricSeries::?m
)
when {
  resource has metric_name &&
  resource.metric_name like "oya_promotion_*" &&
  resource has tenant_id &&
  resource.tenant_id == "tenant:oya-ci"
};

// =============================================================================
// PERMIT 3 — slo-engine-worker emits to oya-self for self-observability
// =============================================================================

permit (
  principal == Service::"slo-engine-worker",
  action == Action::"write_metrics",
  resource in MetricSeries::?m
)
when {
  resource has tenant_id &&
  resource.tenant_id == "tenant:oya-self"
};

// =============================================================================
// FORBID — CI principal cannot read any customer-tenant data
// =============================================================================

forbid (
  principal == Service::"oya-ci",
  action,
  resource in MetricSeries::?m
)
when {
  resource has tenant_id &&
  !(resource.tenant_id in ["tenant:oya-ci", "tenant:oya-aggregate", "tenant:oya-self"])
};

// =============================================================================
// FORBID — CI principal cannot write to non-reserved tenants
// =============================================================================

forbid (
  principal == Service::"oya-ci",
  action in [Action::"write_metrics", Action::"write_logs", Action::"write_traces"],
  resource
)
unless {
  resource has tenant_id &&
  resource.tenant_id in ["tenant:oya-ci", "tenant:oya-aggregate"]
};

// =============================================================================
// FORBID — slo-engine-worker cannot read customer-tenant data
// =============================================================================
//
// Even the worker only operates on its own metrics + aggregations.
// Reading per-customer-tenant data is an explicit cross-tenant boundary
// violation prevented here.

forbid (
  principal == Service::"slo-engine-worker",
  action in [Action::"read_logs", Action::"read_traces"],
  resource in TenantData::?r
)
when {
  resource has tenant_id &&
  !(resource.tenant_id in ["tenant:oya-ci", "tenant:oya-aggregate", "tenant:oya-self"])
};
