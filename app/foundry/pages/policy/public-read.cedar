// Cedar policy: public-read
// Microservice: docs
// Purpose: bound anonymous / unauthenticated public read access to docs that the tenant
// has explicitly published (M04-onward; not GA at M03 — see PRD §"Open Questions" #6).
// At M03 this policy is effectively a default-deny stub for the public-read surface
// to ensure no implicit public exposure during the Strangler migration.
// Owner: ops-security
// Status: Accepted (2026-05-17)
// Related: ADR-0140 (retired per ADR-0145), PRD §"Open Questions" #6
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =====================================================================================
// DEFAULT POSTURE: anonymous principal has NO access.
// =====================================================================================

forbid (
  principal in PublicRole::"anonymous",
  action,
  resource
);

// =====================================================================================
// PERMIT: read on explicitly-public docs (M04-onward; not active at M03).
// =====================================================================================

permit (
  principal in PublicRole::"anonymous",
  action in [
    Action::"document_read_projection"
  ],
  resource
)
when {
  resource has visibility &&
  resource.visibility == "public" &&
  // Public doc must be explicitly opted-in by tenant operator + co-signed by tenant admin.
  resource.public_opt_in == true &&
  resource.public_opt_in_signed_by_tenant == true &&
  resource.public_opt_in_signed_by_tenant_admin == true
};

// =====================================================================================
// REFUSE: public reads on private/internal docs (default-deny reinforcement).
// =====================================================================================

forbid (
  principal in PublicRole::"anonymous",
  action,
  resource
)
when {
  resource has visibility &&
  resource.visibility in ["private", "internal", "tenant_only", "restricted"]
};

// =====================================================================================
// PROJECTION: public reads return only minimum-necessary fields.
// =====================================================================================

forbid (
  principal in PublicRole::"anonymous",
  action == Action::"document_read",
  resource
)
when {
  // Public reads return only the public-projection; raw fields refused.
  resource.field_set has "raw_content" ||
  resource.field_set has "comments" ||
  resource.field_set has "suggestions" ||
  resource.field_set has "private_blocks" ||
  resource.field_set has "edit_history" ||
  resource.field_set has "share_grants"
};

permit (
  principal in PublicRole::"anonymous",
  action == Action::"document_read_projection",
  resource
)
when {
  resource.visibility == "public" &&
  resource.public_opt_in == true &&
  // Public projection: only title + published_at + published-block content + author display name.
  resource.field_set == ["title", "published_at", "public_block_content", "author_display_name"]
};

// =====================================================================================
// REFUSE: anonymous writes, even on public docs.
// =====================================================================================

forbid (
  principal in PublicRole::"anonymous",
  action in [
    Action::"document_write",
    Action::"document_purge",
    Action::"comment_write",
    Action::"suggestion_write",
    Action::"share_grant",
    Action::"share_link_issue",
    Action::"attachment_upload",
    Action::"crdt_op_publish"
  ],
  resource
);

// =====================================================================================
// PERMIT: anonymous share-link redemption (token-bound).
// =====================================================================================

permit (
  principal in PublicRole::"anonymous_share_link_holder",
  action in [
    Action::"document_read",
    Action::"comment_read"
  ],
  resource
)
when {
  context.share_link_token_valid == true &&
  context.share_link_token_signature_verified == true &&
  context.share_link_expires_at >= context.now &&
  context.share_link_revoked == false &&
  context.share_link_role in ["viewer", "commenter"] &&
  resource.id == context.share_link_document_id
};

forbid (
  principal in PublicRole::"anonymous_share_link_holder",
  action in [
    Action::"document_write",
    Action::"share_grant",
    Action::"share_link_issue",
    Action::"document_import",
    Action::"attachment_upload"
  ],
  resource
);

// =====================================================================================
// RATE-LIMIT: anonymous reads enforce per-IP rate limit (Cedar context).
// =====================================================================================

forbid (
  principal in PublicRole::"anonymous",
  action == Action::"document_read_projection",
  resource
)
when {
  context.public_read_rate_limit_exceeded == true
};

forbid (
  principal in PublicRole::"anonymous_share_link_holder",
  action == Action::"document_read",
  resource
)
when {
  context.share_link_redemption_rate_limit_exceeded == true
};

// =====================================================================================
// LEAN check: drift detection.
// =====================================================================================
// LEAN lane `check-public-collection-drift` scans every Postgres row with
// `visibility=public` and verifies tenant-operator + tenant-admin co-signed
// opt-in on file. Drift triggers alert + auto-revert to private + audit-chain
// emission. Same pattern as the calendar µservice.
