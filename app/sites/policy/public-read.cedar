// Cedar policy: public-read
// Microservice: sites
// Purpose: bound anonymous / unauthenticated public read access to pages and assets
// that tenant has explicitly published (i.e., site.visibility == "public" + page.public_opt_in == true).
// Owner: ops-security + axis-sites
// Status: Accepted (2026-05-17)
// Related: ADR-0140 (retired per ADR-0145), ADR-SITES-0002, ADR-SITES-0003
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
// PERMIT: read on explicitly-public sites/pages/assets.
// =====================================================================================

permit (
  principal in PublicRole::"anonymous",
  action in [
    Action::"page_read",
    Action::"site_read",
    Action::"static_asset_read",
    Action::"sitemap_emit",
    Action::"robots_emit"
  ],
  resource
)
when {
  resource has visibility &&
  resource.visibility == "public" &&
  resource.public_opt_in == true &&
  resource.public_opt_in_signed_by_tenant == true
};

// =====================================================================================
// PERMIT: read on published cms-collection entries when their collection is public.
// =====================================================================================

permit (
  principal in PublicRole::"anonymous",
  action == Action::"cms_collection_read",
  resource
)
when {
  resource has visibility &&
  resource.visibility == "public" &&
  resource has collection_publish_state &&
  resource.collection_publish_state == "published"
};

// =====================================================================================
// REFUSE: public reads on private/intranet/draft pages (default-deny reinforcement).
// =====================================================================================

forbid (
  principal in PublicRole::"anonymous",
  action,
  resource
)
when {
  resource has visibility &&
  resource.visibility in ["private", "intranet", "draft", "restricted"]
};

// =====================================================================================
// PROJECTION: public reads return only published fields; refuse draft / private fields.
// =====================================================================================

forbid (
  principal in PublicRole::"anonymous",
  action == Action::"page_read",
  resource
)
when {
  // Public reads return only the published version; draft state never crosses the boundary.
  resource has page_state &&
  resource.page_state in ["draft", "scheduled", "internal_review"]
};

// =====================================================================================
// REFUSE: anonymous writes, even on public pages.
// =====================================================================================

forbid (
  principal in PublicRole::"anonymous",
  action in [
    Action::"site_write",
    Action::"page_write",
    Action::"page_publish",
    Action::"page_revert",
    Action::"page_delete",
    Action::"block_write",
    Action::"theme_write",
    Action::"navigation_write",
    Action::"redirect_create",
    Action::"domain_bind",
    Action::"cert_request",
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
// RATE-LIMIT: anonymous reads enforce per-IP rate limit (Cedar context).
// =====================================================================================

forbid (
  principal in PublicRole::"anonymous",
  action,
  resource
)
when {
  context.public_read_rate_limit_exceeded == true
};

// =====================================================================================
// PERMIT: anonymous form submission ROUTES to `forms` µservice; sites never persists.
// =====================================================================================
//
// (No `form_submit` action in sites scope — form-block on a sites page binds via Ontology
// to the `forms` µservice; the submission lands in forms's REST surface bypassing sites.)

// =====================================================================================
// REFUSE: anonymous AI-page-build (never).
// =====================================================================================

forbid (
  principal in PublicRole::"anonymous",
  action == Action::"ai_page_build",
  resource
);

// =====================================================================================
// PERMIT: search_query for anonymous on public-scoped index.
// =====================================================================================

permit (
  principal in PublicRole::"anonymous",
  action == Action::"search_query",
  resource
)
when {
  resource has search_scope &&
  resource.search_scope == "public_only"
};

forbid (
  principal in PublicRole::"anonymous",
  action == Action::"search_query",
  resource
)
when {
  resource has search_scope &&
  resource.search_scope in ["intranet", "draft", "tenant_only"]
};

// =====================================================================================
// COMMENT-BLOCK: anonymous read permitted; write goes to `community` µservice.
// =====================================================================================
//
// (No `comment_post` action in sites scope — comments are owned by `community` µservice;
// sites embeds via Ontology binding.)

// =====================================================================================
// LEAN check: drift detection.
// =====================================================================================
// LEAN lane `oya-check-public-page-drift` scans every Postgres row with `visibility=public`
// and verifies tenant-publisher opt-in signature on file. Drift (e.g., page that drifted from
// private to public without signature) triggers alert + auto-revert + audit-chain emission.
