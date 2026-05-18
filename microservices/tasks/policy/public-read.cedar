// Cedar policy: public-read
// Microservice: tasks
// Purpose: bound anonymous / unauthenticated public read access to task collections
// that tenant has explicitly published (e.g., open-source project task board,
// community-roadmap visible to public).
// Owner: ops-security
// Status: Accepted (2026-05-17)
// Related: ADR-0140 (retired per ADR-0145)
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
// PERMIT: read on explicitly-public projects (read-only).
// =====================================================================================

permit (
  principal in PublicRole::"anonymous",
  action in [
    Action::"task_read_projection",
    Action::"project_read_projection",
    Action::"view_render_public"
  ],
  resource
)
when {
  resource has visibility &&
  resource.visibility == "public" &&
  // Public project must be explicitly opted-in by tenant operator.
  resource.public_opt_in == true &&
  resource.public_opt_in_signed_by_tenant == true
};

// =====================================================================================
// REFUSE: public reads on private/internal/tenant-only collections (default-deny reinforcement).
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
  action == Action::"task_read",
  resource
)
when {
  // Public reads return only the public-projection; raw fields refused.
  resource.field_set has "description_full" ||
  resource.field_set has "comments" ||
  resource.field_set has "assignee_email" ||
  resource.field_set has "watcher_emails" ||
  resource.field_set has "custom_field_values_sensitive" ||
  resource.field_set has "private_notes" ||
  resource.field_set has "time_tracking_ticks"
};

permit (
  principal in PublicRole::"anonymous",
  action == Action::"task_read_projection",
  resource
)
when {
  resource.visibility == "public" &&
  resource.public_opt_in == true &&
  // Public projection: only title (truncated to 200 chars), status, priority, due_at, label_names.
  resource.field_set == ["title_truncated", "status", "priority", "due_at", "label_names"]
};

// =====================================================================================
// REFUSE: anonymous writes, even on public collections.
// =====================================================================================

forbid (
  principal in PublicRole::"anonymous",
  action in [
    Action::"task_write",
    Action::"task_state_change",
    Action::"task_assign",
    Action::"task_comment",
    Action::"task_dependency_add",
    Action::"task_purge",
    Action::"task_bulk_edit",
    Action::"task_import",
    Action::"task_export"
  ],
  resource
);

// =====================================================================================
// REFUSE: anonymous T2 auto-assign (refused for everyone outside tenant + conformity).
// =====================================================================================

forbid (
  principal in PublicRole::"anonymous",
  action == Action::"task_t2_auto_assign",
  resource
);

// =====================================================================================
// RATE-LIMIT: public reads enforce per-IP rate limit (Cedar context).
// =====================================================================================

forbid (
  principal in PublicRole::"anonymous",
  action == Action::"task_read_projection",
  resource
)
when {
  context.public_read_rate_limit_exceeded == true
};

// =====================================================================================
// LEAN check: drift detection.
// =====================================================================================
// LEAN lane `oya-check-public-collection-drift` scans every Postgres row with
// `visibility=public` and verifies tenant-operator opt-in signature on file.
// Drift (e.g., a row that drifted from private to public without signature) triggers
// alert + auto-revert to private + audit-chain emission.
