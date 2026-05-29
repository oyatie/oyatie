// sheets µservice — CI-scope policy fragment
// Cedar v4.2 LTS schema
//
// Purpose: enforce CI runner scope on Sheets read operations during gate validation;
//          CI must read workbook session metadata + license-gate decisions + collab stream stats
//          + formula-engine corpus + XLSX import/export job metadata for governance lanes, but is
//          bounded to read-only and to specific metric / row name patterns.
// Owner:   axis-foundry + ops-security
// References: threat-model.md §"T-S-02" (Sheets SPIFFE impersonation)
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
// PERMIT 1 — CI runner reads Sheets metadata for governance lanes
// =============================================================================

permit (
  principal == ServiceAccount::"oya-sheets-ci",
  action in [
    Action::"read_workbook_session_metadata",
    Action::"read_collab_session_count",
    Action::"read_license_gate_aggregate",
    Action::"read_sheets_self_slo",
    Action::"read_governance_lane_status",
    Action::"read_formula_engine_corpus_result",
    Action::"read_xlsx_import_job_metadata",
    Action::"read_xlsx_export_job_metadata",
    Action::"read_recalc_engine_metrics",
    Action::"read_share_acl_drift_report",
    Action::"read_range_acl_coverage"
  ],
  resource
);

// =============================================================================
// FORBID — CI never reads tenant draft contents
// =============================================================================

forbid (
  principal == ServiceAccount::"oya-sheets-ci",
  action in [
    Action::"read_cell",
    Action::"read_range",
    Action::"read_workbook_metadata",
    Action::"read_collab_participant_list",
    Action::"read_comment",
    Action::"invoke_ai_formula_draft",
    Action::"invoke_smart_fill",
    Action::"refresh_connected_query"
  ],
  resource
);

// =============================================================================
// FORBID — CI never writes; refuses any save or session-open
// =============================================================================

forbid (
  principal == ServiceAccount::"oya-sheets-ci",
  action in [
    Action::"open_workbook_session",
    Action::"write_cell",
    Action::"write_range",
    Action::"write_formula",
    Action::"save_workbook",
    Action::"publish_crdt_op",
    Action::"switch_jurisdiction_overlay",
    Action::"discard_workbook_draft",
    Action::"share_workbook",
    Action::"author_range_acl",
    Action::"register_connected_query",
    Action::"author_edit_trigger",
    Action::"open_embed_bridge",
    Action::"restore_workbook_version"
  ],
  resource
);

// =============================================================================
// FORBID — CI cannot read SECRET-class
// =============================================================================

forbid (
  principal == ServiceAccount::"oya-sheets-ci",
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};
