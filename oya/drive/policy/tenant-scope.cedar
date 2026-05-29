// Cedar policy: tenant-scope
// Microservice: drive
// Purpose: enforce per-tenant scoping on every drive operation; refuse cross-tenant
// read/write/share unless an explicit CrossTenantShareGrant is on file.
// Owner: ops-security + axis-drive
// Status: Accepted (2026-05-17)
// Related: ADR-0140 (retired per ADR-0145), policy/dual-context-isolation.md, policy/data-residency.md, ADR-DRIVE-0003 (share-link), ADR-DRIVE-0006 (WORM)
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =====================================================================================
// DEFAULT POSTURE: forbid cross-tenant reads / writes / shares by default.
// =====================================================================================

forbid (
  principal,
  action in [
    Action::"file_read",
    Action::"file_write",
    Action::"file_delete",
    Action::"folder_list",
    Action::"folder_create",
    Action::"folder_delete",
    Action::"share_link_mint",
    Action::"share_link_revoke",
    Action::"permission_grant",
    Action::"permission_revoke",
    Action::"sync_session_open",
    Action::"upload_initiate",
    Action::"upload_complete",
    Action::"download_initiate",
    Action::"search_query",
    Action::"preview_render",
    Action::"webdav_propfind",
    Action::"webdav_put",
    Action::"webdav_delete",
    Action::"s3_get_object",
    Action::"s3_put_object",
    Action::"s3_delete_object",
    Action::"s3_list_objects",
    Action::"tus_create",
    Action::"tus_patch"
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
  principal in DriveRole::"tenant_operator",
  action in [
    Action::"file_read",
    Action::"file_write",
    Action::"folder_list",
    Action::"folder_create",
    Action::"share_link_mint",
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
  principal.tenant_id == resource.tenant_id &&
  principal.context_kind == resource.context_kind
};

// =====================================================================================
// CROSS-TENANT GRANT: permit only specific shared files via explicit grant.
// =====================================================================================

permit (
  principal in DriveRole::"cross_tenant_share_recipient",
  action == Action::"file_read",
  resource
)
when {
  principal.tenant_id != resource.tenant_id &&
  context.share_grant_exists == true &&
  context.share_grant_valid == true &&
  context.share_grant_scope == "read_only" &&
  // The grant must bind to the specific file or folder; no enumeration.
  context.share_grant_resource_id == resource.id
};

// Cross-tenant download requires the same grant + a signed download ticket.
permit (
  principal in DriveRole::"cross_tenant_share_recipient",
  action == Action::"download_initiate",
  resource
)
when {
  principal.tenant_id != resource.tenant_id &&
  context.share_grant_exists == true &&
  context.share_grant_valid == true &&
  context.share_grant_resource_id == resource.id &&
  context.signed_download_ticket_valid == true
};

// =====================================================================================
// PUBLIC SHARE-LINK: permits unauthenticated read via signed link (TTL + view-cap).
// =====================================================================================

permit (
  principal in DriveRole::"share_link_viewer",
  action == Action::"file_read",
  resource
)
when {
  // Link must exist, be unrevoked, within TTL, and within view-cap.
  context.share_link_id != null &&
  context.share_link_signature_valid == true &&
  context.share_link_not_expired == true &&
  context.share_link_view_count_remaining > 0 &&
  // Password protection: if link has password, request carries proof.
  (context.share_link_password_protected == false ||
   context.share_link_password_proof_valid == true) &&
  // Link must bind to the specific resource.
  context.share_link_resource_id == resource.id
};

forbid (
  principal in DriveRole::"share_link_viewer",
  action in [
    Action::"file_write",
    Action::"file_delete",
    Action::"folder_list",
    Action::"share_link_mint",
    Action::"permission_grant"
  ],
  resource
);

// =====================================================================================
// DUAL-CONTEXT ISOLATION: Personal vs Professional.
// =====================================================================================

// Refuse Professional reader on Personal-context file.
forbid (
  principal in DriveRole::"professional_reader",
  action in [Action::"file_read", Action::"folder_list", Action::"search_query"],
  resource in FileContext::"Personal"
);

// Refuse Personal reader on Professional-context file.
forbid (
  principal in DriveRole::"personal_reader",
  action in [Action::"file_read", Action::"folder_list", Action::"search_query"],
  resource in FileContext::"Professional"
);

// Permit context-bridge ONLY at file-level explicit grant.
permit (
  principal in DriveRole::"context_bridge_reader",
  action == Action::"file_read",
  resource
)
when {
  context.personal_to_professional_grant.file_id == resource.id &&
  context.personal_to_professional_grant.consent_at <= context.now &&
  context.personal_to_professional_grant.revoked_at == null
};

// =====================================================================================
// LEGAL HOLD + WORM IMMUTABILITY: refuse hard-delete on held / WORM files.
// =====================================================================================

forbid (
  principal,
  action == Action::"file_purge",
  resource
)
when {
  resource has legal_hold_open && resource.legal_hold_open == true
};

// WORM-tier files refuse purge before retention floor expires, EVEN by tenant-root.
forbid (
  principal,
  action == Action::"file_purge",
  resource
)
when {
  resource has worm_tier_enabled && resource.worm_tier_enabled == true &&
  context.now < resource.worm_retention_floor
};

// Even hard-delete of un-held / non-WORM files requires 2-person rule per ADR-DRIVE-0006.
permit (
  principal in DriveRole::"compliance_officer_with_2person_rule",
  action == Action::"file_purge",
  resource
)
when {
  resource has legal_hold_open && resource.legal_hold_open == false &&
  (resource has worm_tier_enabled == false ||
   context.now >= resource.worm_retention_floor) &&
  context.two_person_rule_approved == true &&
  context.audit_chain_pre_emitted == true
};

// =====================================================================================
// SHARE-LINK MINT: refuse on flagged content (DLP gate).
// =====================================================================================

forbid (
  principal,
  action == Action::"share_link_mint",
  resource
)
when {
  resource has dlp_flagged && resource.dlp_flagged == true &&
  // Tenant-policy admin can override per-content with audit-chain emission.
  context.dlp_override_approved != true
};

// =====================================================================================
// OWNERSHIP TRANSFER.
// =====================================================================================

permit (
  principal in DriveRole::"file_owner",
  action == Action::"ownership_transfer",
  resource
)
when {
  principal.user_id == resource.owner_user_id &&
  context.transfer_acknowledged_by_owner == true
};

forbid (
  principal,
  action == Action::"ownership_transfer",
  resource
)
unless {
  principal.user_id == resource.owner_user_id
};

// =====================================================================================
// CROSS-PACK DATA TRANSFER: forbid by default; permit with SCC.
// =====================================================================================

forbid (
  principal,
  action in [
    Action::"file_read",
    Action::"file_write",
    Action::"download_initiate"
  ],
  resource
)
when {
  principal has pack_tag && resource has pack_tag &&
  principal.pack_tag != resource.pack_tag
};

permit (
  principal in DriveRole::"cross_pack_reader",
  action == Action::"file_read",
  resource
)
when {
  principal has pack_tag && resource has pack_tag &&
  principal.pack_tag != resource.pack_tag &&
  context.scc_on_file == true &&
  context.tenant_dpa_includes_scc_clause == true
};
