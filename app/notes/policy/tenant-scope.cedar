// notes µservice — tenant-scope policy fragment
// Cedar v4.2 schema; conforms to microservices/notes/policy/schema.cedarschema (referenced)
//
// Purpose: enforce per-tenant read/write scope for notes entities.
// Owner:   ops-security + axis-notes
// Audit:   every deny emits oya_notes_tenant_unauthorized_attempt_total
// References: dual-context-isolation.md; ADR-NOTES-0001; ADR-NOTES-0005
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
// PERMIT 1 — tenant operator reads own tenant's Professional notes
// =============================================================================

permit (
  principal in TenantOperator::?p,
  action in [
    Action::"read_note_metadata",
    Action::"list_notes",
    Action::"read_notebook",
    Action::"read_audit_chain"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  resource has context_kind &&
  resource.context_kind == "Professional"
};

// =============================================================================
// PERMIT 2 — end-user reads/writes own notes
// =============================================================================

permit (
  principal in EndUser::?u,
  action in [
    Action::"read_note",
    Action::"create_note",
    Action::"edit_own_note",
    Action::"delete_own_note",
    Action::"tag_note",
    Action::"untag_note",
    Action::"read_backlink",
    Action::"read_tag_graph",
    Action::"render_graph_view",
    Action::"create_share_link",
    Action::"revoke_share_link",
    Action::"start_collab_session",
    Action::"invoke_ai_assist",
    Action::"create_daily_note",
    Action::"apply_template",
    Action::"import_notes",
    Action::"export_notes",
    Action::"submit_clip",
    Action::"emit_checklist_item",
    Action::"restore_version"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  resource has owner_user_id &&
  principal.user_id == resource.owner_user_id
};

// =============================================================================
// PERMIT 3 — notebook-admin manages notebook ACL
// =============================================================================

permit (
  principal in NotebookAdmin::?a,
  action in [
    Action::"add_notebook_member",
    Action::"remove_notebook_member",
    Action::"set_notebook_retention",
    Action::"set_notebook_metadata"
  ],
  resource in Notebook::?n
)
when {
  principal has notebook_admin_of &&
  resource has notebook_id &&
  resource.notebook_id in principal.notebook_admin_of &&
  resource has context_kind &&
  resource.context_kind == "Professional"
};

// =============================================================================
// PERMIT 4 — compliance-officer issues holds (Professional only)
// =============================================================================

permit (
  principal in ComplianceOfficer::?o,
  action in [
    Action::"open_ediscovery_hold",
    Action::"close_ediscovery_hold",
    Action::"list_held_notes"
  ],
  resource in TenantData::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has entitlements &&
  "ediscovery_hold" in principal.entitlements &&
  resource has context_kind &&
  resource.context_kind == "Professional"
};

// =============================================================================
// PERMIT 5 — four-eyes disclosure (Professional only)
// =============================================================================
//
// Two distinct principals with disclosure entitlement must co-sign.
// Cedar evaluates per-attempt; the four-eyes pairing requires two
// distinct accepts within an audit-chain transaction.

permit (
  principal in DisclosureApprover::?d,
  action == Action::"disclose_note_body",
  resource in Note::?m
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has entitlements &&
  "disclosure_approver" in principal.entitlements &&
  resource has context_kind &&
  resource.context_kind == "Professional" &&
  context has paired_approver_id &&
  context.paired_approver_id != principal.principal_id
};

// =============================================================================
// FORBID — never permit disclosure of Personal-tier notes
// =============================================================================
//
// Personal-context notes are E2E-encrypted server-side; even with four-eyes,
// the body is unreadable. This Cedar block belt-and-suspenders rejects any
// attempt to invoke disclosure on a Personal resource.

forbid (
  principal,
  action == Action::"disclose_note_body",
  resource in Note::?m
)
when {
  resource has context_kind &&
  resource.context_kind == "Personal"
};

// =============================================================================
// FORBID — cross-tenant access (defence-in-depth)
// =============================================================================

forbid (
  principal,
  action in [
    Action::"read_note",
    Action::"read_note_metadata",
    Action::"read_backlink",
    Action::"read_tag_graph",
    Action::"render_graph_view",
    Action::"read_notebook",
    Action::"list_notes",
    Action::"read_audit_chain"
  ],
  resource
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// FORBID — SECRET-class data never via tenant reads
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
// FORBID — non-owner read of personal note
// =============================================================================

forbid (
  principal,
  action in [
    Action::"read_note",
    Action::"edit_own_note",
    Action::"delete_own_note"
  ],
  resource in Note::?m
)
when {
  resource has context_kind &&
  resource.context_kind == "Personal"
}
unless {
  resource has owner_user_id &&
  principal has user_id &&
  principal.user_id == resource.owner_user_id
};

// =============================================================================
// FORBID — AI invocation over Personal-tier (E2E) notes
// =============================================================================
//
// Belt-and-suspenders alongside ADR-NOTES-0005 type-system + CI-lane.
// Any attempt logs to oya_notes_ai_call_blocked_e2e_total.

forbid (
  principal,
  action == Action::"invoke_ai_assist",
  resource in Note::?m
)
when {
  resource has context_kind &&
  resource.context_kind == "Personal"
};

// =============================================================================
// FORBID — collab session over Personal-tier (E2E) notes
// =============================================================================
//
// ADR-NOTES-0003 — Loro requires server-side reconciliation; refused on E2E.

forbid (
  principal,
  action == Action::"start_collab_session",
  resource in Note::?m
)
when {
  resource has context_kind &&
  resource.context_kind == "Personal"
};
