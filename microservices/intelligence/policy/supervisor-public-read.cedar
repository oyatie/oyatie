// foundry-supervisor µservice — public-read policy fragment
// Cedar v4
//
// Purpose: govern anonymous access to non-sensitive aggregated supervisor
//          metrics (public status page, DP-aggregated kill-switch coverage,
//          capability schema validation).
// Owner:   ops-security + gtm-marketing
// References: threat-model.md T-I-04; dpia.md §"Cross-tenant aggregations"
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
// PERMIT 1 — anonymous read of supervisor platform-availability metrics
// =============================================================================

permit (
  principal == User::"anonymous",
  action == Action::"read_metric",
  resource in MetricSeries::?m
)
when {
  resource has tenant_id &&
  resource.tenant_id == "tenant_oya_self" &&
  resource has data_class &&
  resource.data_class == "PUBLIC" &&
  resource has metric_name &&
  resource.metric_name like "oya_supervisor_public_*"
};

// =============================================================================
// PERMIT 2 — anonymous read of DP-aggregated kill-switch coverage
// =============================================================================
//
// The oya-aggregate tenant carries DP-noise-injected cross-tenant aggregates
// for public dashboards (e.g., "platform-wide kill-switch SLA over the last
// 30 days"). Per-tenant attribution is statistically prevented; ε ≤ 1.

permit (
  principal == User::"anonymous",
  action == Action::"read_metric",
  resource in MetricSeries::?m
)
when {
  resource has tenant_id &&
  resource.tenant_id == "tenant_oya_aggregate" &&
  resource has data_class &&
  resource.data_class == "PUBLIC" &&
  resource has dp_epsilon &&
  resource.dp_epsilon <= 1.0
};

// =============================================================================
// PERMIT 3 — anonymous schema-validation of capability YAML
// =============================================================================
//
// Public endpoint accepts a capability YAML and returns conformance feedback.
// Does NOT persist the submission, does NOT reveal whether the capability is
// already deployed.

permit (
  principal == User::"anonymous",
  action == Action::"validate_capability_schema",
  resource in PublicEndpoint::"capability-schema-validator"
);

// =============================================================================
// FORBID — anonymous cannot read any non-public-class data
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
  !(resource.tenant_id in ["tenant_oya_self", "tenant_oya_aggregate"])
};

// =============================================================================
// FORBID — anonymous cannot write anything
// =============================================================================

forbid (
  principal == User::"anonymous",
  action in [
    Action::"write_fleet_state",
    Action::"write_capability_definition",
    Action::"write_autonomy_entitlement",
    Action::"engage_kill_switch",
    Action::"disengage_kill_switch",
    Action::"drain_fleet",
    Action::"write_artifact"
  ],
  resource
);

// =============================================================================
// FORBID — anonymous cannot engage kill-switch (mediated by tenant + ops-security only)
// =============================================================================

forbid (
  principal == User::"anonymous",
  action in [
    Action::"engage_kill_switch",
    Action::"disengage_kill_switch"
  ],
  resource
);
