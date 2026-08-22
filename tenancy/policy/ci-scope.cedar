// tenancy µservice — CI-scope policy fragment
// Cedar v4
//
// Purpose: scope the ci reserved tenant's access to governance lanes only;
//          forbid reads or writes against customer-tenant data.
// Owner:   axis-platform + axis-tenancy + ops-security
// References: rls-isolation.md §"Reserved tenant IDs"; threat-model.md T-S-03, T-I-03
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// PERMIT 1 — CI lane reads governance-state for promotion-readiness verdicts
// =============================================================================
//
// The governance-promotion-readiness CI lane (running as principal Service::"ci")
// may read the governance state that gates tenancy promotion.

permit (
  principal == Service::"ci",
  action in [
    Action::"read_governance_state",
    Action::"read_rls_policy_state",
    Action::"read_branch_protection_state",
    Action::"read_lifecycle_event_count"
  ],
  resource
)
when {
  resource has tenant_id &&
  resource.tenant_id in ["tenant:ci", "tenant:self", "tenant:aggregate", "tenant:system"]
};

// =============================================================================
// PERMIT 2 — slo-engine-worker writes self self-observability metrics
// =============================================================================
//
// tenancy self-SLOs run as tenant:self; the slo-engine-worker emits
// verdict metrics with that tenant scope.

permit (
  principal == Service::"slo-engine-worker",
  action in [Action::"write_metrics", Action::"write_logs", Action::"write_traces"],
  resource
)
when {
  resource has tenant_id &&
  resource.tenant_id == "tenant:self"
};

// =============================================================================
// PERMIT 3 — tenancy-isolation-policy-worker emits CI-tenant metrics
// =============================================================================
//
// The tenancy isolation-policy-worker emits tenancy_* metrics into
// tenant:ci for CI lane consumption (e.g., rls-drift metrics).

permit (
  principal == Service::"tenancy-isolation-policy-worker",
  action == Action::"write_metrics",
  resource
)
when {
  resource has tenant_id &&
  resource.tenant_id == "tenant:ci" &&
  resource has metric_name &&
  resource.metric_name like "tenancy_*"
};

// =============================================================================
// FORBID — CI principal cannot read any customer-tenant data
// =============================================================================
//
// The CI lane reads aggregated governance state only; per-tenant lifecycle
// records + DSR records + cell-assignment per-tenant rows are out of scope.

forbid (
  principal == Service::"ci",
  action,
  resource
)
when {
  resource has tenant_id &&
  !(resource.tenant_id in ["tenant:ci", "tenant:self", "tenant:aggregate", "tenant:system"])
};

// =============================================================================
// FORBID — CI principal cannot mutate tenancy state
// =============================================================================

forbid (
  principal == Service::"ci",
  action in [
    Action::"create_tenant",
    Action::"suspend_tenant",
    Action::"resume_tenant",
    Action::"delete_tenant",
    Action::"submit_dsr_request",
    Action::"alter_rls_policy",
    Action::"disable_rls",
    Action::"rotate_jwt_signing_key"
  ],
  resource
);

// =============================================================================
// FORBID — CI principal cannot read SECRET data
// =============================================================================

forbid (
  principal == Service::"ci",
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};

// =============================================================================
// FORBID — tenancy-isolation-policy-worker cannot read customer-tenant data
// =============================================================================
//
// The worker operates on its own self-observability + CI-scoped metrics.
// Reading per-customer-tenant data is an explicit cross-tenant boundary
// violation prevented here. RLS at Postgres layer would also block; this is
// the defence-in-depth layer.

forbid (
  principal == Service::"tenancy-isolation-policy-worker",
  action in [Action::"read_tenant", Action::"read_dsr_request"],
  resource
)
when {
  resource has tenant_id &&
  !(resource.tenant_id in ["tenant:ci", "tenant:self", "tenant:aggregate", "tenant:system"])
};

// =============================================================================
// FORBID — reserved-tenant write by non-authorised principals
// =============================================================================
//
// tenant:ci / self / aggregate / system are reserved.
// Only specific SPIFFE identities may write to them.

forbid (
  principal,
  action in [Action::"write_metrics", Action::"write_logs", Action::"write_traces"],
  resource
)
when {
  resource has tenant_id &&
  resource.tenant_id in ["tenant:ci", "tenant:self", "tenant:aggregate", "tenant:system"]
}
unless {
  principal in [
    Service::"ci",
    Service::"slo-engine-worker",
    Service::"tenancy-isolation-policy-worker",
    Service::"tenancy-cell-assignment-worker",
    Service::"tenancy-dsr-cascade-worker"
  ]
};
