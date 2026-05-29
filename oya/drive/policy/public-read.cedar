// Cedar policy: public-read
// Microservice: drive
// Purpose: govern public share-link read; enforce TTL, view-cap, password, and
// flagged-content refusal. Default-deny on every other action.
// Owner: ops-security + axis-drive
// Status: Accepted (2026-05-17)
// Related: ADR-0140 (retired per ADR-0145), ADR-DRIVE-0003 (share-link security model), policy/tenant-scope.cedar
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =====================================================================================
// DEFAULT: anonymous principals (no tenant_id) refused on everything.
// =====================================================================================

forbid (
  principal,
  action,
  resource
)
when {
  !(principal has tenant_id) &&
  !(principal in DriveRole::"share_link_viewer")
};

// =====================================================================================
// PUBLIC SHARE-LINK VIEWER: permit read only on explicit signed link.
// =====================================================================================

permit (
  principal in DriveRole::"share_link_viewer",
  action == Action::"file_read",
  resource
)
when {
  context.share_link_id != null &&
  context.share_link_signature_valid == true &&
  context.share_link_not_expired == true &&
  context.share_link_view_count_remaining > 0 &&
  (context.share_link_password_protected == false ||
   context.share_link_password_proof_valid == true) &&
  context.share_link_resource_id == resource.id &&
  // Refuse if file is flagged by DLP or has malware verdict.
  (resource has dlp_flagged == false || resource.dlp_flagged == false) &&
  (resource has virus_verdict == "clean" || !(resource has virus_verdict))
};

permit (
  principal in DriveRole::"share_link_viewer",
  action == Action::"download_initiate",
  resource
)
when {
  context.share_link_id != null &&
  context.share_link_signature_valid == true &&
  context.share_link_not_expired == true &&
  context.share_link_view_count_remaining > 0 &&
  (context.share_link_password_protected == false ||
   context.share_link_password_proof_valid == true) &&
  context.share_link_resource_id == resource.id &&
  (resource has dlp_flagged == false || resource.dlp_flagged == false) &&
  (resource has virus_verdict == "clean" || !(resource has virus_verdict))
};

permit (
  principal in DriveRole::"share_link_viewer",
  action == Action::"preview_render",
  resource
)
when {
  context.share_link_id != null &&
  context.share_link_signature_valid == true &&
  context.share_link_not_expired == true &&
  context.share_link_view_count_remaining > 0 &&
  context.share_link_resource_id == resource.id &&
  // Preview-only links permit preview without download (per tenant-policy choice).
  context.share_link_preview_only_allowed == true
};

// =====================================================================================
// SHARE-LINK VIEWER: refuse all writes and meta operations.
// =====================================================================================

forbid (
  principal in DriveRole::"share_link_viewer",
  action in [
    Action::"file_write",
    Action::"file_delete",
    Action::"file_purge",
    Action::"folder_list",
    Action::"folder_create",
    Action::"share_link_mint",
    Action::"share_link_revoke",
    Action::"permission_grant",
    Action::"permission_revoke",
    Action::"upload_initiate",
    Action::"upload_complete",
    Action::"sync_session_open",
    Action::"webdav_put",
    Action::"webdav_delete",
    Action::"s3_put_object",
    Action::"s3_delete_object"
  ],
  resource
);

// =====================================================================================
// REVOKED LINK: refuse explicitly.
// =====================================================================================

forbid (
  principal in DriveRole::"share_link_viewer",
  action,
  resource
)
when {
  context.share_link_revoked == true
};

// =====================================================================================
// VIEW-CAP EXCEEDED: refuse explicitly.
// =====================================================================================

forbid (
  principal in DriveRole::"share_link_viewer",
  action,
  resource
)
when {
  context.share_link_view_count_remaining <= 0
};

// =====================================================================================
// PASSWORD REQUIRED BUT NOT PROVIDED: refuse explicitly.
// =====================================================================================

forbid (
  principal in DriveRole::"share_link_viewer",
  action,
  resource
)
when {
  context.share_link_password_protected == true &&
  context.share_link_password_proof_valid != true
};

// =====================================================================================
// RATE-LIMIT: refuse if per-IP enumeration anomaly detected.
// =====================================================================================

forbid (
  principal in DriveRole::"share_link_viewer",
  action,
  resource
)
when {
  context.client_ip_share_link_verify_rate_per_minute > 100
};
