// sheets µservice — tenant-scope policy fragment
// Cedar v4.2 LTS schema; conforms to app/sheets/policy/schema.cedarschema
//
// Purpose: enforce per-tenant scope on workbook session open/read/write, CRDT op publish/subscribe,
//          AI-formula invocation, save action, XLSX import/export, connected-sheets refresh,
//          comments, share-acl changes, version-history restore, embed-bridge creation, and
//          trigger-bridge dispatch served via sheets REST + WebSocket gateway.
// Owner:   ops-security + axis-sheets
// Audit:   every deny emits sheets_unauthorized_attempt_total
// References: threat-model.md §"T-I-01" + §"T-I-04" + §"T-T-07" (per-range ACL)
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
// PERMIT 1 — tenant operator reads own tenant's workbook sessions + cells
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"read_workbook_session",
    Action::"read_workbook_metadata",
    Action::"read_sheet",
    Action::"read_cell",
    Action::"read_range",
    Action::"read_named_range",
    Action::"read_chart",
    Action::"read_pivot",
    Action::"read_conditional_format",
    Action::"read_data_validation",
    Action::"read_comment",
    Action::"read_version_history",
    Action::"read_share_acl",
    Action::"read_collab_participant_list",
    Action::"list_workbook_sessions",
    Action::"list_workbooks"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 2 — tenant operator opens new workbook session
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"open_workbook_session",
    Action::"close_workbook_session",
    Action::"resume_workbook_session"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has has_sheets_seat_entitlement &&
  principal.has_sheets_seat_entitlement == true
};

// =============================================================================
// PERMIT 3 — tenant operator authors + saves workbook + per-cell edits
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"publish_crdt_op",
    Action::"write_cell",
    Action::"write_range",
    Action::"write_formula",
    Action::"author_chart",
    Action::"author_pivot",
    Action::"author_conditional_format",
    Action::"author_data_validation",
    Action::"author_named_range",
    Action::"add_comment",
    Action::"save_workbook",
    Action::"discard_workbook_draft",
    Action::"switch_jurisdiction_overlay"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// PERMIT 4 — tenant operator subscribes to own tenant's collab streams
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action == Action::"subscribe_crdt_stream",
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  resource has workbook_id &&
  principal has subscribed_workbooks &&
  resource.workbook_id in principal.subscribed_workbooks
};

// =============================================================================
// PERMIT 5 — tenant operator invokes AI-formula (consent-required)
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"invoke_ai_formula_draft",
    Action::"invoke_smart_fill"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has ai_formula_consent &&
  principal.ai_formula_consent == true &&
  principal has has_sheets_seat_entitlement &&
  principal.has_sheets_seat_entitlement == true
};

// =============================================================================
// PERMIT 6 — tenant operator imports / exports XLSX
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"import_xlsx",
    Action::"import_ods",
    Action::"import_csv",
    Action::"import_tsv",
    Action::"import_json_sheet",
    Action::"export_xlsx",
    Action::"export_ods",
    Action::"export_csv",
    Action::"export_tsv",
    Action::"export_json_sheet"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has import_export_entitlement &&
  principal.import_export_entitlement == true
};

// =============================================================================
// PERMIT 7 — tenant operator changes sharing + per-range ACL
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"share_workbook",
    Action::"author_range_acl",
    Action::"revoke_share",
    Action::"revoke_range_acl"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has share_admin_entitlement &&
  principal.share_admin_entitlement == true
};

// =============================================================================
// PERMIT 8 — tenant operator registers + refreshes connected-sheets query
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"register_connected_query",
    Action::"refresh_connected_query",
    Action::"revoke_connected_query"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has connected_sheets_entitlement &&
  principal.connected_sheets_entitlement == true
};

// =============================================================================
// PERMIT 9 — per-range ACL gate (ADR-SHEETS-0006 named-ACL granularity)
// =============================================================================
//
// Granted only when (a) tenant-scope, (b) the requested range falls within the
// principal's allowed ranges for this workbook.

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"read_cell",
    Action::"read_range"
  ],
  resource in CellOrRange::?cr
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  resource has range_id &&
  principal has allowed_range_acls &&
  resource.range_id in principal.allowed_range_acls
};

// =============================================================================
// PERMIT 10 — tenant operator restores from version-history
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action == Action::"restore_workbook_version",
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has version_restore_entitlement &&
  principal.version_restore_entitlement == true
};

// =============================================================================
// PERMIT 11 — tenant operator opens embed-bridge (docs / slides)
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action == Action::"open_embed_bridge",
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has embed_bridge_entitlement &&
  principal.embed_bridge_entitlement == true
};

// =============================================================================
// PERMIT 12 — tenant operator authors sheet-edit trigger (cross-µservice to workflow-engine)
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action == Action::"author_edit_trigger",
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has trigger_bridge_entitlement &&
  principal.trigger_bridge_entitlement == true
};

// =============================================================================
// FORBID — explicit cross-tenant deny (defense-in-depth)
// =============================================================================

forbid (
  principal,
  action,
  resource
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// FORBID — per-range ACL bypass attempt (range outside allowed_range_acls)
// =============================================================================

forbid (
  principal in TenantOperator::?p,
  action in [
    Action::"read_cell",
    Action::"read_range",
    Action::"write_cell",
    Action::"write_range",
    Action::"write_formula"
  ],
  resource in CellOrRange::?cr
)
when {
  resource has range_id &&
  principal has allowed_range_acls &&
  !(resource.range_id in principal.allowed_range_acls)
};

// =============================================================================
// FORBID — never expose SECRET-class data
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
};

// =============================================================================
// FORBID — Sheets never executes; only emits
// =============================================================================

forbid (
  principal,
  action in [
    Action::"execute_workflow_step",
    Action::"dispatch_step_body",
    Action::"start_workflow_run",
    Action::"execute_apps_script",
    Action::"execute_vba",
    Action::"shell_exec"
  ],
  resource
);

// =============================================================================
// FORBID — production-tier save requires Cedar policy preview acknowledgment
// =============================================================================

forbid (
  principal in TenantOperator::?p,
  action == Action::"save_workbook",
  resource in TenantData::?r
)
when {
  resource has environment &&
  resource.environment == "production" &&
  principal has has_policy_preview_acknowledged &&
  principal.has_policy_preview_acknowledged != true
};

// =============================================================================
// FORBID — AI-formula in regulated domain without conformity assessment (per ADR-SHEETS-0005)
// =============================================================================

forbid (
  principal in TenantOperator::?p,
  action in [
    Action::"invoke_ai_formula_draft",
    Action::"invoke_smart_fill"
  ],
  resource in TenantData::?r
)
when {
  resource has regulated_domain &&
  resource.regulated_domain == true &&
  principal has ai_act_conformity_attested &&
  principal.ai_act_conformity_attested != true
};
