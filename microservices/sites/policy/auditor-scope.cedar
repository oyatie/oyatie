// Cedar policy: auditor-scope
// Microservice: sites
// Purpose: bound external auditor + internal audit roles to read-only access on
// audit-relevant fields; refuse write operations and refuse access to tenant content
// not in audit scope.
// Owner: ops-security + council-privacy
// Status: Accepted (2026-05-17)
// Related: ADR-0140 (retired per ADR-0145), ADR-0028, compliance.md
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =====================================================================================
// DEFAULT: auditor has no access.
// =====================================================================================

forbid (
  principal in AuditorRole::"external_auditor",
  action,
  resource
);

forbid (
  principal in AuditorRole::"internal_auditor",
  action,
  resource
);

// =====================================================================================
// PERMIT: read-only access on audit-relevant fields.
// =====================================================================================

permit (
  principal in AuditorRole::"external_auditor",
  action in [
    Action::"site_read",
    Action::"page_read",
    Action::"cms_collection_read",
    Action::"audit_chain_read",
    Action::"sitemap_emit",   // read-only emit
    Action::"robots_emit"     // read-only emit
  ],
  resource
)
when {
  // External auditor must be in tenant's auditor whitelist.
  resource has auditor_whitelist &&
  resource.auditor_whitelist.contains(principal.auditor_id) &&
  // Audit scope is bounded by tenant DPA + audit engagement contract.
  context.audit_scope_in_engagement == true
};

permit (
  principal in AuditorRole::"internal_auditor",
  action in [
    Action::"site_read",
    Action::"page_read",
    Action::"cms_collection_read",
    Action::"audit_chain_read"
  ],
  resource
)
when {
  // Internal auditor bound to scoped audit engagement.
  principal has audit_engagement_id &&
  context.audit_engagement_active == true
};

// =====================================================================================
// REFUSE: any write action by auditor.
// =====================================================================================

forbid (
  principal in [AuditorRole::"external_auditor", AuditorRole::"internal_auditor"],
  action in [
    Action::"site_write",
    Action::"site_delete",
    Action::"page_write",
    Action::"page_publish",
    Action::"page_revert",
    Action::"page_delete",
    Action::"block_write",
    Action::"theme_write",
    Action::"navigation_write",
    Action::"redirect_create",
    Action::"domain_bind",
    Action::"domain_unbind",
    Action::"cert_request",
    Action::"cert_revoke",
    Action::"seo_meta_write",
    Action::"cms_collection_write",
    Action::"cms_entry_write",
    Action::"search_index_write",
    Action::"cdn_purge",
    Action::"cdn_publish",
    Action::"image_upload",
    Action::"ai_page_build"
  ],
  resource
);

// =====================================================================================
// PERMIT: auditor reads PHI / sensitive data only with engagement-bound consent.
// =====================================================================================

forbid (
  principal in [AuditorRole::"external_auditor", AuditorRole::"internal_auditor"],
  action == Action::"page_read",
  resource
)
when {
  resource has data_class &&
  resource.data_class in ["HIPAA_PHI", "SENSITIVE_PIPA_ART23", "EU_SPECIAL_CATEGORY_ART9"] &&
  !context.engagement_includes_phi_scope
};

permit (
  principal in AuditorRole::"external_auditor",
  action == Action::"page_read",
  resource
)
when {
  resource has data_class &&
  resource.data_class in ["HIPAA_PHI", "SENSITIVE_PIPA_ART23", "EU_SPECIAL_CATEGORY_ART9"] &&
  context.engagement_includes_phi_scope == true &&
  context.baa_or_dpa_on_file == true
};

// =====================================================================================
// SUNSET: auditor access bounded by engagement expiry.
// =====================================================================================

forbid (
  principal in [AuditorRole::"external_auditor", AuditorRole::"internal_auditor"],
  action,
  resource
)
when {
  context has engagement_expires_at &&
  context.engagement_expires_at <= context.now
};

// =====================================================================================
// AUDIT: every auditor read emits audit-chain.
// =====================================================================================
//
// Every auditor-triggered audit-chain event carries `principal_class=auditor` for
// downstream filtering. council-privacy + ops-security review auditor activity quarterly.
