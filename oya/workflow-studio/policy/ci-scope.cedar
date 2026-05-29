// workflow-studio µservice — CI-scope policy fragment
// Cedar v4 schema
//
// Purpose: enforce CI runner scope on Studio read operations during gate validation;
//          CI must read editor session metadata + license-gate decisions + collab stream stats
//          across tenants for governance lanes, but is bounded to read-only and to specific
//          metric / row name patterns.
// Owner:   axis-foundry + ops-security
// References: threat-model.md §"T-S-02" (Studio SPIFFE impersonation)
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny.
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — CI runner reads Studio metadata for governance lanes
// =============================================================================
//
// Reserved CI tenant: oya-workflow-studio-ci.

permit (
  principal == ServiceAccount::"oya-workflow-studio-ci",
  action in [
    Action::"read_editor_session_metadata",
    Action::"read_collab_session_count",
    Action::"read_license_gate_aggregate",
    Action::"read_studio_self_slo",
    Action::"read_governance_lane_status",
    Action::"read_node_library_signature_status"
  ],
  resource
);

// =============================================================================
// FORBID — CI never reads tenant draft contents
// =============================================================================

forbid (
  principal == ServiceAccount::"oya-workflow-studio-ci",
  action in [
    Action::"read_workflow_definition_draft",
    Action::"read_collab_participant_list",
    Action::"step_through_run_frontend",
    Action::"open_debugger_session",
    Action::"invoke_llm_assist"
  ],
  resource
);

// =============================================================================
// FORBID — CI never writes; refuses any save or session-open
// =============================================================================

forbid (
  principal == ServiceAccount::"oya-workflow-studio-ci",
  action in [
    Action::"open_editor_session",
    Action::"save_workflow_definition",
    Action::"publish_crdt_op",
    Action::"switch_jurisdiction_overlay",
    Action::"discard_workflow_definition_draft"
  ],
  resource
);

// =============================================================================
// FORBID — CI cannot read SECRET-class
// =============================================================================

forbid (
  principal == ServiceAccount::"oya-workflow-studio-ci",
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};
