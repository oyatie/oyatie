// intelligence µservice — CI-scope policy fragment
// Cedar v4
//
// Purpose: scope oya-ci reserved principal's access to compliance-evidence + manifest reads.
// Owner:   axis-foundry + axis-intelligence + ops-security
// References: observability/policy/ci-scope.cedar (parallel pattern).

// =============================================================================
// Default deny
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT — CI lane reads compliance evidence + dispatch SLI rollups
// =============================================================================

permit (
  principal == Service::"oya-ci",
  action == Action::"read_metrics",
  resource in MetricSeries::?m
)
when {
  resource has metric_name &&
  (
    resource.metric_name like "oya_intelligence_*" ||
    resource.metric_name == "oya:intelligence_dispatch_availability" ||
    resource.metric_name == "oya:intelligence_first_token_latency_p99" ||
    resource.metric_name == "oya:intelligence_audit_emission_success"
  ) &&
  resource has tenant_id &&
  resource.tenant_id in ["tenant:oya-ci", "tenant:oya-aggregate", "tenant:oya-self"]
};

// =============================================================================
// PERMIT — CI lane reads manifest + IPs + catalog + capabilities + slos files
// =============================================================================

permit (
  principal == Service::"oya-ci",
  action == Action::"read_artifact",
  resource in PolicyArtifact::?r
)
when {
  resource has artifact_path &&
  resource.artifact_path like "microservices/intelligence/*"
};

// =============================================================================
// FORBID — CI principal cannot dispatch
// =============================================================================

forbid (
  principal == Service::"oya-ci",
  action == Action::"dispatch",
  resource
);

// =============================================================================
// FORBID — CI principal cannot read customer-tenant audit-tap data
// =============================================================================

forbid (
  principal == Service::"oya-ci",
  action in [Action::"read_audit_tap_record", Action::"read_dispatch_envelope_meta"],
  resource
)
when {
  resource has tenant_id &&
  !(resource.tenant_id in ["tenant:oya-ci", "tenant:oya-aggregate", "tenant:oya-self"])
};
