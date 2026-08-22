// observability µservice — public-read policy fragment
// Cedar v4
//
// Purpose: govern anonymous / unauthenticated access to non-sensitive aggregated
//          metrics (e.g., the public status page, the public differential-privacy
//          aggregated leaderboard, the public OpenSLO schema-validation endpoint).
// Owner:   ops-security + gtm-marketing
// References: threat-model.md T-I-05; dpia.md §"Cross-tenant aggregations"
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
// PERMIT 1 — anonymous read of platform status (self tenant, public-class)
// =============================================================================
//
// The public status page reads aggregated platform availability metrics from
// the self tenant. These are platform-wide health indicators (e.g., "Mimir
// cluster availability over the last 30 days") with no per-customer attribution.

permit (
  principal == User::"anonymous",
  action == Action::"read_metrics",
  resource in MetricSeries::?m
)
when {
  resource has tenant_id &&
  resource.tenant_id == "tenant:self" &&
  resource has data_class &&
  resource.data_class == "PUBLIC" &&
  resource has metric_name &&
  resource.metric_name like "observability_public_*"
};

// =============================================================================
// PERMIT 2 — anonymous read of differential-privacy aggregates (aggregate)
// =============================================================================
//
// The aggregate tenant carries DP-noise-injected cross-tenant aggregates
// suitable for product marketing leaderboards. Per-tenant attribution is
// statistically prevented by the noise; ε ≤ 1.

permit (
  principal == User::"anonymous",
  action == Action::"read_metrics",
  resource in MetricSeries::?m
)
when {
  resource has tenant_id &&
  resource.tenant_id == "tenant:aggregate" &&
  resource has data_class &&
  resource.data_class == "PUBLIC" &&
  resource has dp_epsilon &&
  resource.dp_epsilon <= 1.0
};

// =============================================================================
// PERMIT 3 — anonymous validation of OpenSLO manifest (schema-validate only)
// =============================================================================
//
// The public schema-validation endpoint accepts an OpenSLO YAML and returns
// schema-conformance feedback. It does NOT persist the submission, and it
// does NOT reveal whether the SLI is reachable in the caller's tenant.

permit (
  principal == User::"anonymous",
  action == Action::"validate_openslo_schema",
  resource in PublicEndpoint::"openslo-schema-validator"
);

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
  !(resource.tenant_id in ["tenant:self", "tenant:aggregate"])
};

// =============================================================================
// FORBID — anonymous cannot write anything
// =============================================================================

forbid (
  principal == User::"anonymous",
  action in [
    Action::"write_metrics",
    Action::"write_logs",
    Action::"write_traces",
    Action::"write_openslo_manifests",
    Action::"write_artifact"
  ],
  resource
);

// =============================================================================
// FORBID — anonymous schema-validation endpoint cannot exfiltrate the submitted
// manifest. The validator returns schema-conformance feedback only; submissions
// are not persisted, logged, or transmitted off-host.
// =============================================================================
//
// This is enforced at the endpoint code level + Cedar prohibits any action
// against PolicyArtifact except read_artifact (which is auditor-only above).

forbid (
  principal == User::"anonymous",
  action in [
    Action::"read_artifact",
    Action::"write_artifact",
    Action::"delete_artifact"
  ],
  resource
);
