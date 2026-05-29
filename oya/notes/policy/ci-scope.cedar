// notes µservice — CI-scope policy fragment
// Cedar v4.2; refused by default; permits only the CI-runner principal class.
//
// Purpose: bound CI lanes to read-only inspection + lane-emit; deny any
// write-back to tenant data; deny any AI invocation; deny any disclosure.
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

forbid (
  principal,
  action,
  resource
);

// CI-runner reads its own evidence lake + per-lane manifests
permit (
  principal in CiRunner::?c,
  action in [
    Action::"read_evidence_topic",
    Action::"emit_lane_artifact",
    Action::"read_audit_chain",
    Action::"validate_per_microservice_layout",
    Action::"validate_dual_context_isolation",
    Action::"validate_e2e_ai_refusal"
  ],
  resource
)
when {
  principal has lane_name &&
  resource has microservice &&
  resource.microservice == "notes"
};

// CI MUST NOT read note bodies (Personal or Professional)
forbid (
  principal in CiRunner::?c,
  action in [
    Action::"read_note",
    Action::"disclose_note_body",
    Action::"invoke_ai_assist",
    Action::"start_collab_session",
    Action::"create_share_link",
    Action::"submit_clip"
  ],
  resource
);

// CI MUST NOT write to tenant data
forbid (
  principal in CiRunner::?c,
  action in [
    Action::"create_note",
    Action::"edit_own_note",
    Action::"delete_own_note",
    Action::"tag_note",
    Action::"untag_note",
    Action::"emit_checklist_item",
    Action::"import_notes",
    Action::"export_notes"
  ],
  resource
);
