// Cedar policy: tenant-scope
// Microservice: docs
// Purpose: enforce per-tenant scoping on every docs operation; refuse cross-tenant
// read/write unless an explicit ShareGrant is on file.
// Owner: ops-security + axis-docs
// Status: Accepted (2026-05-17)
// Related: ADR-0140 (retired per ADR-0145), policy/editor-isolation.md, policy/data-residency.md,
//          ADR-DOCS-0004 (per-block ACL)
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =====================================================================================
// DEFAULT POSTURE: forbid cross-tenant reads / writes by default.
// =====================================================================================

forbid (
  principal,
  action in [
    Action::"document_read",
    Action::"document_write",
    Action::"document_archive",
    Action::"comment_read",
    Action::"comment_write",
    Action::"suggestion_read",
    Action::"suggestion_write",
    Action::"share_grant",
    Action::"share_revoke",
    Action::"share_link_issue",
    Action::"share_link_redeem",
    Action::"document_export",
    Action::"document_import",
    Action::"attachment_upload",
    Action::"attachment_download",
    Action::"embed_resolve",
    Action::"version_restore",
    Action::"crdt_op_publish",
    Action::"crdt_op_subscribe"
  ],
  resource
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id != resource.tenant_id
};

// =====================================================================================
// PERMIT: same-tenant access with role-based authorization.
// =====================================================================================

permit (
  principal in DocsRole::"tenant_author",
  action in [
    Action::"document_read",
    Action::"document_write",
    Action::"comment_read",
    Action::"comment_write",
    Action::"suggestion_read",
    Action::"suggestion_write",
    Action::"share_grant",
    Action::"share_revoke",
    Action::"share_link_issue",
    Action::"document_export",
    Action::"document_import",
    Action::"attachment_upload",
    Action::"attachment_download",
    Action::"embed_resolve",
    Action::"version_restore",
    Action::"crdt_op_publish",
    Action::"crdt_op_subscribe"
  ],
  resource
)
when {
  principal.tenant_id == resource.tenant_id &&
  principal.context_kind == resource.context_kind
};

permit (
  principal in DocsRole::"tenant_reader",
  action in [
    Action::"document_read",
    Action::"comment_read",
    Action::"attachment_download",
    Action::"embed_resolve",
    Action::"document_export"
  ],
  resource
)
when {
  principal.tenant_id == resource.tenant_id &&
  principal.context_kind == resource.context_kind &&
  // Reader role requires explicit share grant.
  context.share_grant_exists == true &&
  context.share_grant_valid == true
};

permit (
  principal in DocsRole::"tenant_commenter",
  action in [
    Action::"document_read",
    Action::"comment_read",
    Action::"comment_write",
    Action::"attachment_download",
    Action::"embed_resolve"
  ],
  resource
)
when {
  principal.tenant_id == resource.tenant_id &&
  principal.context_kind == resource.context_kind &&
  context.share_grant_exists == true &&
  context.share_grant_valid == true &&
  context.share_grant_role in ["commenter", "editor"]
};

// =====================================================================================
// CROSS-TENANT SHARE: permit only the granted role over an explicit share-grant.
// =====================================================================================

permit (
  principal in DocsRole::"cross_tenant_share_recipient",
  action in [
    Action::"document_read",
    Action::"comment_read",
    Action::"embed_resolve"
  ],
  resource
)
when {
  principal.tenant_id != resource.tenant_id &&
  context.share_grant_exists == true &&
  context.share_grant_valid == true &&
  context.share_grant_role in ["viewer", "commenter"] &&
  // Cross-tenant write FORBIDDEN even with grant; grants are read+comment-only.
  // Editor role requires same-tenant.
  context.share_grant_signed_by_owning_tenant == true
};

forbid (
  principal in DocsRole::"cross_tenant_share_recipient",
  action in [
    Action::"document_write",
    Action::"comment_write",
    Action::"suggestion_write",
    Action::"share_grant",
    Action::"share_revoke",
    Action::"version_restore",
    Action::"document_export",
    Action::"document_import",
    Action::"attachment_upload"
  ],
  resource
);

// =====================================================================================
// DUAL-CONTEXT ISOLATION: Personal vs Professional.
// =====================================================================================

forbid (
  principal in DocsRole::"professional_reader",
  action == Action::"document_read",
  resource in DocumentContext::"Personal"
);

forbid (
  principal in DocsRole::"personal_reader",
  action == Action::"document_read",
  resource in DocumentContext::"Professional"
);

permit (
  principal in DocsRole::"context_bridge_reader",
  action == Action::"document_read",
  resource
)
when {
  context.personal_to_professional_grant.field == resource.field_name &&
  context.personal_to_professional_grant.consent_at <= context.now &&
  context.personal_to_professional_grant.revoked_at == null
};

// =====================================================================================
// PER-BLOCK ACL (per ADR-DOCS-0004): block-level grant must be evaluated for each block.
// =====================================================================================

forbid (
  principal,
  action == Action::"document_read",
  resource in BlockKind::"private"
)
when {
  resource has block_acl_principals &&
  !(principal.user_id in resource.block_acl_principals)
};

permit (
  principal in DocsRole::"tenant_author",
  action == Action::"document_read",
  resource in BlockKind::"team_visible"
)
when {
  principal.tenant_id == resource.tenant_id &&
  principal.team_id in resource.block_acl_teams
};

// =====================================================================================
// LEGAL HOLD: refuse hard-delete on held documents.
// =====================================================================================

forbid (
  principal,
  action == Action::"document_purge",
  resource
)
when {
  resource has legal_hold_open && resource.legal_hold_open == true
};

permit (
  principal in DocsRole::"compliance_officer_with_2person_rule",
  action == Action::"document_purge",
  resource
)
when {
  resource has legal_hold_open && resource.legal_hold_open == false &&
  context.two_person_rule_approved == true &&
  context.audit_chain_pre_emitted == true
};

// =====================================================================================
// SUGGESTION ACCEPTANCE: only doc author + explicit grantee.
// =====================================================================================

permit (
  principal in DocsRole::"document_author",
  action == Action::"suggestion_respond",
  resource
)
when {
  principal.user_id == resource.author_user_id &&
  context.suggestion_acknowledged_by_author == true
};

forbid (
  principal,
  action == Action::"suggestion_respond",
  resource
)
unless {
  principal.user_id == resource.author_user_id ||
  context.suggestion_response_delegated_to_principal == principal.user_id
};

// =====================================================================================
// CROSS-PACK DATA TRANSFER: forbid by default; permit only via embed-resolver snapshot.
// =====================================================================================

forbid (
  principal,
  action in [
    Action::"document_read",
    Action::"document_write"
  ],
  resource
)
when {
  principal has pack_tag && resource has pack_tag &&
  principal.pack_tag != resource.pack_tag
};

permit (
  principal in DocsRole::"cross_pack_embed_reader",
  action == Action::"embed_resolve",
  resource
)
when {
  principal has pack_tag && resource has pack_tag &&
  principal.pack_tag != resource.pack_tag &&
  context.scc_on_file == true &&
  context.tenant_dpa_includes_scc_clause == true &&
  // Cross-pack returns embed snapshot ONLY; raw doc content forbidden.
  resource.access_kind == "embed_snapshot"
};

// =====================================================================================
// CRDT OP PUBLISH: bound to (tenant, document) lease at WS gateway.
// =====================================================================================

forbid (
  principal,
  action == Action::"crdt_op_publish",
  resource
)
unless {
  principal.ws_lease.tenant_id == resource.tenant_id &&
  principal.ws_lease.document_id == resource.document_id
};
