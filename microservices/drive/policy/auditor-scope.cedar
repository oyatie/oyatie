// Cedar policy: auditor-scope
// Microservice: drive
// Purpose: bound external-auditor (SOC 2 / ISO 27001 / SEC 17a-4 / HIPAA OCR) access;
// time-boxed JIT; tenant-row-scoped; read-only.
// Owner: ops-security + council-architecture
// Status: Accepted (2026-05-17)
// Related: ADR-0140 (retired per ADR-0145), policy/tenant-scope.cedar
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =====================================================================================
// DEFAULT: auditor principals are read-only.
// =====================================================================================

forbid (
  principal in DriveRole::"external_auditor",
  action in [
    Action::"file_write",
    Action::"file_delete",
    Action::"file_purge",
    Action::"folder_create",
    Action::"folder_delete",
    Action::"share_link_mint",
    Action::"share_link_revoke",
    Action::"permission_grant",
    Action::"permission_revoke",
    Action::"upload_initiate",
    Action::"upload_complete",
    Action::"webdav_put",
    Action::"webdav_delete",
    Action::"s3_put_object",
    Action::"s3_delete_object"
  ],
  resource
);

// =====================================================================================
// PERMIT: auditor can read metadata + audit-chain records within time-boxed window.
// =====================================================================================

permit (
  principal in DriveRole::"external_auditor",
  action in [
    Action::"file_read",
    Action::"file_metadata_read",
    Action::"folder_list",
    Action::"audit_chain_read",
    Action::"search_query",
    Action::"preview_render"
  ],
  resource
)
when {
  // JIT window ≤ 4h.
  context.auditor_jit_token_valid == true &&
  context.auditor_jit_expires_at > context.now &&
  // Tenant-row-scoped: auditor has explicit tenant grant.
  context.auditor_tenant_grant_includes(resource.tenant_id) == true
};

// =====================================================================================
// AUDIT-CHAIN: every auditor read emits to audit-chain.
// =====================================================================================

permit (
  principal in DriveRole::"external_auditor",
  action == Action::"audit_chain_emit",
  resource
)
when {
  context.auditor_jit_token_valid == true
};

// =====================================================================================
// AUDITOR: refuse cross-pack pivot.
// =====================================================================================

forbid (
  principal in DriveRole::"external_auditor",
  action,
  resource
)
when {
  principal has pack_tag && resource has pack_tag &&
  principal.pack_tag != resource.pack_tag
};

// =====================================================================================
// AUDITOR: SEC 17a-4 auditor has expanded scope on WORM tier records.
// =====================================================================================

permit (
  principal in DriveRole::"sec_17a4_auditor",
  action in [
    Action::"file_read",
    Action::"file_metadata_read",
    Action::"audit_chain_read",
    Action::"immutability_record_read"
  ],
  resource
)
when {
  resource has worm_tier_enabled &&
  resource.worm_tier_enabled == true &&
  context.auditor_jit_token_valid == true &&
  context.auditor_tenant_grant_includes(resource.tenant_id) == true
};

// =====================================================================================
// HIPAA OCR AUDITOR: BAA-eligible tenant scope only.
// =====================================================================================

permit (
  principal in DriveRole::"hipaa_ocr_auditor",
  action in [
    Action::"file_read",
    Action::"file_metadata_read",
    Action::"audit_chain_read"
  ],
  resource
)
when {
  resource has pack_tag &&
  resource.pack_tag == "pack-us-healthcare" &&
  context.baa_on_file == true &&
  context.auditor_jit_token_valid == true
};
