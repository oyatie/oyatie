// ops-dashboard-control-center — ci-scope policy fragment
// Cedar v4.2 LTS
//
// Purpose: Foundry CI pipeline principals have read-only access to scorecard,
//          SLO state, policy artifacts, and promotion eligibility data.
//          CI cannot mutate any operator-facing resource.
//
// Owner:   ops-sre-reliability
// Binding ADR: ADR-0243 (Cedar universal gate), ADR-0247 (Foundry self-modification)
// References: ARCHITECTURE.md §cedar-gates, manifest.json
// Soak window: ≥60s per ADR-0294

// =============================================================================
// Default deny
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT — CI / Foundry pipeline reads
// =============================================================================

permit (
  principal in FoundryPipeline::?p,
  action in [
    Action::"scorecard_read",
    Action::"slo_state_read",
    Action::"policy_artifact_read",
    Action::"promotion_eligibility_read",
    Action::"manifest_read",
    Action::"catalog_read",
    Action::"dashboard_view",
    Action::"openslo_manifest_read",
    Action::"compliance_artifact_read"
  ],
  resource
)
when {
  principal has authenticated &&
  principal.authenticated == true &&
  principal has spiffe_svid &&
  principal.spiffe_svid != ""
};

// =============================================================================
// FORBID — CI cannot mutate operator-facing resources
// =============================================================================

forbid (
  principal in FoundryPipeline::?p,
  action in [
    Action::"incident_declare",
    Action::"deployment_approve",
    Action::"rollback_execute",
    Action::"admin_console_mutation",
    Action::"cedar_fragment_publish",
    Action::"pack_author_publish",
    Action::"oncall_handoff_create",
    Action::"tenant_isolation_policy_change"
  ],
  resource
);

// =============================================================================
// FORBID — CI cannot access PII or AUTHENTICATION data classes
// =============================================================================

forbid (
  principal in FoundryPipeline::?p,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class in ["PII_IDENTIFYING", "AUTHENTICATION", "SECRET"]
};
