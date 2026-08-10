// Cedar policy: ci-scope
// Microservice: sites
// Purpose: bound CI / pipeline principals to test-tenant resources only; refuse
// production-tenant access from CI.
// Owner: ops-security
// Status: Accepted (2026-05-17)
// Related: ADR-0140 (retired per ADR-0145), ADR-0117
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =====================================================================================
// DEFAULT: CI principal has no access.
// =====================================================================================

forbid (
  principal in CiRole::"ci_runner",
  action,
  resource
);

// =====================================================================================
// PERMIT: CI accesses test-tenant resources only.
// =====================================================================================

permit (
  principal in CiRole::"ci_runner",
  action in [
    Action::"site_read",
    Action::"site_write",
    Action::"page_read",
    Action::"page_write",
    Action::"page_publish",
    Action::"block_read",
    Action::"block_write",
    Action::"theme_read",
    Action::"theme_write",
    Action::"navigation_read",
    Action::"navigation_write",
    Action::"url_routing_read",
    Action::"redirect_create",
    Action::"seo_meta_write",
    Action::"sitemap_emit",
    Action::"robots_emit",
    Action::"cms_collection_read",
    Action::"cms_collection_write",
    Action::"cms_entry_write",
    Action::"search_query",
    Action::"image_upload",
    Action::"cdn_publish",
    Action::"cdn_purge"
  ],
  resource
)
when {
  resource has tenant_class &&
  resource.tenant_class == "synthetic_test" &&
  principal.pack_tag == resource.pack_tag
};

// =====================================================================================
// REFUSE: CI ACME cert requests against real Let's Encrypt — staging-only.
// =====================================================================================

forbid (
  principal in CiRole::"ci_runner",
  action == Action::"cert_request",
  resource
)
when {
  resource has acme_directory_url &&
  !(resource.acme_directory_url like "*acme-staging*")
};

permit (
  principal in CiRole::"ci_runner",
  action == Action::"cert_request",
  resource
)
when {
  resource has acme_directory_url &&
  resource.acme_directory_url like "*acme-staging*" &&
  resource.tenant_class == "synthetic_test"
};

// =====================================================================================
// REFUSE: CI domain-bind on tenant-owned real domains.
// =====================================================================================

forbid (
  principal in CiRole::"ci_runner",
  action == Action::"domain_bind",
  resource
)
when {
  !(resource.domain like "*.test.oyatie.dev")
};

// =====================================================================================
// REFUSE: CI AI-page-build on real LLM providers — synthetic-only.
// =====================================================================================

forbid (
  principal in CiRole::"ci_runner",
  action == Action::"ai_page_build",
  resource
)
when {
  resource has llm_provider &&
  resource.llm_provider != "synthetic_local"
};

// =====================================================================================
// REFUSE: CI cdn_purge against production CDN account.
// =====================================================================================

forbid (
  principal in CiRole::"ci_runner",
  action == Action::"cdn_purge",
  resource
)
when {
  resource has cdn_account &&
  resource.cdn_account != "synthetic_test"
};

// =====================================================================================
// AUDIT: CI principal carries `ci` audit tag.
// =====================================================================================
//
// Every CI-triggered audit-chain event carries `principal_class=ci_runner` for
// downstream filtering. ops-security can recompute coverage by excluding ci_runner.
