// Cedar policy: ci-scope
// Microservice: drive
// Purpose: bound CI / synthetic test runner permissions; allow only synthetic-tenant
// scope; refuse production-tenant access.
// Owner: ops-security + axis-drive
// Status: Accepted (2026-05-17)
// Related: ADR-0140 (retired per ADR-0145), policy/tenant-scope.cedar
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =====================================================================================
// DEFAULT: CI principals refuse production-tenant access outright.
// =====================================================================================

forbid (
  principal in DriveRole::"ci_runner",
  action,
  resource
)
when {
  resource has tenant_class && resource.tenant_class == "production"
};

// =====================================================================================
// PERMIT: CI runner can read/write its synthetic test tenants.
// =====================================================================================

permit (
  principal in DriveRole::"ci_runner",
  action in [
    Action::"file_read",
    Action::"file_write",
    Action::"file_delete",
    Action::"folder_list",
    Action::"folder_create",
    Action::"share_link_mint",
    Action::"share_link_revoke",
    Action::"sync_session_open",
    Action::"upload_initiate",
    Action::"upload_complete",
    Action::"download_initiate",
    Action::"search_query",
    Action::"preview_render"
  ],
  resource
)
when {
  resource has tenant_class &&
  (resource.tenant_class == "synthetic" || resource.tenant_class == "dev")
};

// =====================================================================================
// CI: refuse WORM tier on synthetic tenants (would block test cleanup).
// =====================================================================================

forbid (
  principal in DriveRole::"ci_runner",
  action == Action::"immutability_tier_apply",
  resource
);

// =====================================================================================
// CI: permit purge on synthetic tenants without 2-person rule (for cleanup between test runs).
// =====================================================================================

permit (
  principal in DriveRole::"ci_runner",
  action == Action::"file_purge",
  resource
)
when {
  resource has tenant_class && resource.tenant_class == "synthetic" &&
  (resource has worm_tier_enabled == false ||
   resource.worm_tier_enabled == false) &&
  (resource has legal_hold_open == false ||
   resource.legal_hold_open == false)
};

// =====================================================================================
// CI: refuse cross-pack access (CI runs per-pack).
// =====================================================================================

forbid (
  principal in DriveRole::"ci_runner",
  action,
  resource
)
when {
  principal has pack_tag && resource has pack_tag &&
  principal.pack_tag != resource.pack_tag
};

// =====================================================================================
// CI: rate-limit on share-link mint to prevent test-set share-link enumeration.
// =====================================================================================

forbid (
  principal in DriveRole::"ci_runner",
  action == Action::"share_link_mint",
  resource
)
when {
  context.share_link_mints_in_last_hour > 1000
};
