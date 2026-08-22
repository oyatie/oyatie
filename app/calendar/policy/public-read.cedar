// Cedar policy: public-read
// Microservice: calendar
// Purpose: bound anonymous / unauthenticated public read access to calendar collections
// that tenant has explicitly published (e.g., conference schedule, public event listing,
// room schedule for shared facility booking).
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
// PERMIT: read on explicitly-public collections.
// =====================================================================================

permit (
  principal in PublicRole::"anonymous",
  action in [
    Action::"event_read_projection",
    Action::"caldav_propfind_public",
    Action::"caldav_report_public"
  ],
  resource
)
when {
  resource has visibility &&
  resource.visibility == "public" &&
  // Public collection must be explicitly opted-in by tenant operator.
  resource.public_opt_in == true &&
  resource.public_opt_in_signed_by_tenant == true
};

// =====================================================================================
// REFUSE: public reads on private/internal collections (default-deny reinforcement).
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
  action == Action::"event_read",
  resource
)
when {
  // Public reads return only the public-projection; raw fields refused.
  resource.field_set has "title_full" ||
  resource.field_set has "description" ||
  resource.field_set has "attendees" ||
  resource.field_set has "private_notes"
};

permit (
  principal in PublicRole::"anonymous",
  action == Action::"event_read_projection",
  resource
)
when {
  resource.visibility == "public" &&
  resource.public_opt_in == true &&
  // Public projection: only title (truncated to first 80 chars), start/end, location_public.
  resource.field_set == ["title_truncated", "starts_at", "ends_at", "location_public"]
};

// =====================================================================================
// REFUSE: anonymous writes, even on public collections.
// =====================================================================================

forbid (
  principal in PublicRole::"anonymous",
  action in [
    Action::"event_write",
    Action::"event_purge",
    Action::"room_book",
    Action::"invitation_send",
    Action::"caldav_put",
    Action::"caldav_delete"
  ],
  resource
);

// =====================================================================================
// RATE-LIMIT: public reads enforce per-IP rate limit (Cedar context).
// =====================================================================================

forbid (
  principal in PublicRole::"anonymous",
  action == Action::"event_read_projection",
  resource
)
when {
  context.public_read_rate_limit_exceeded == true
};

// =====================================================================================
// LEAN check: drift detection.
// =====================================================================================
// LEAN lane `check-public-collection-drift` scans every Postgres row with
// `visibility=public` and verifies tenant-operator opt-in signature on file.
// Drift (e.g., a row that drifted from private to public without signature) triggers
// alert + auto-revert to private + audit-chain emission.
