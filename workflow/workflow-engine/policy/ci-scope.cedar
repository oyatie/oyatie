// workflow-engine µservice — CI-scope policy fragment
// Cedar v4 schema
//
// Purpose: enforce CI runner scope on engine read operations during gate validation;
//          CI must read workflow spec and run state across tenants for governance lanes,
//          but is bounded to read-only and to specific metric / row name patterns.
// Owner:   axis-foundry + ops-security
// References: threat-model.md §"T-S-03" (engine-worker SA impersonation)
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny — every action refused unless explicit permit matches.
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — CI runner (oya-ci tenant) reads engine state for governance lanes
// =============================================================================
//
// The CI runner reads workflow specs + engine SLO + lane status across tenants
// (cross-tenant aggregation is permitted at the lane layer for governance only).
//
// Reserved CI tenant: oya-workflow-engine-ci.

permit (
  principal == ServiceAccount::"oya-workflow-engine-ci",
  action in [
    Action::"read_workflow_spec_metadata",
    Action::"read_workflow_run_count",
    Action::"read_engine_self_slo",
    Action::"read_governance_lane_status"
  ],
  resource
);

// =============================================================================
// FORBID — CI never reads tenant step payload contents
// =============================================================================
//
// CI may read counts + metadata for governance gates; tenant-specific step payloads
// remain bounded to TenantOperator.

forbid (
  principal == ServiceAccount::"oya-workflow-engine-ci",
  action in [
    Action::"read_step_payload",
    Action::"inspect_step_payload",
    Action::"replay_workflow_run"
  ],
  resource
);

// =============================================================================
// FORBID — CI never writes; refuses any spec submission or run-start
// =============================================================================

forbid (
  principal == ServiceAccount::"oya-workflow-engine-ci",
  action in [
    Action::"submit_workflow_spec",
    Action::"deprecate_workflow_spec",
    Action::"start_workflow_run",
    Action::"pause_workflow_run",
    Action::"cancel_workflow_run",
    Action::"publish_workflow_event"
  ],
  resource
);

// =============================================================================
// FORBID — CI cannot read SECRET-class
// =============================================================================

forbid (
  principal == ServiceAccount::"oya-workflow-engine-ci",
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};
