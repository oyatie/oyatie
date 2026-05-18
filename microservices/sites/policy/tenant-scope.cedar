// Cedar policy: tenant-scope
// Microservice: sites
// Purpose: enforce per-tenant scoping on every sites operation; refuse cross-tenant
// read/write except via explicitly-public projections.
// Owner: ops-security + axis-sites
// Status: Accepted (2026-05-17)
// Related: ADR-0140 (retired per ADR-0145), policy/editor-isolation.md, policy/data-residency.md, policy/public-read.cedar
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =====================================================================================
// DEFAULT POSTURE: forbid cross-tenant reads / writes by default.
// =====================================================================================

forbid (
  principal,
  action in [
    Action::"site_read",
    Action::"site_write",
    Action::"site_publish",
    Action::"site_unpublish",
    Action::"site_delete",
    Action::"page_read",
    Action::"page_write",
    Action::"page_publish",
    Action::"page_revert",
    Action::"page_delete",
    Action::"block_read",
    Action::"block_write",
    Action::"theme_read",
    Action::"theme_write",
    Action::"navigation_read",
    Action::"navigation_write",
    Action::"url_routing_read",
    Action::"url_routing_write",
    Action::"redirect_create",
    Action::"domain_bind",
    Action::"domain_unbind",
    Action::"cert_request",
    Action::"cert_revoke",
    Action::"seo_meta_write",
    Action::"sitemap_emit",
    Action::"robots_emit",
    Action::"cms_collection_read",
    Action::"cms_collection_write",
    Action::"cms_entry_write",
    Action::"search_query",
    Action::"search_index_write",
    Action::"cdn_purge",
    Action::"cdn_publish",
    Action::"image_upload",
    Action::"ai_page_build"
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
  principal in SitesRole::"tenant_editor",
  action in [
    Action::"site_read",
    Action::"site_write",
    Action::"page_read",
    Action::"page_write",
    Action::"block_read",
    Action::"block_write",
    Action::"theme_read",
    Action::"navigation_read",
    Action::"navigation_write",
    Action::"url_routing_read",
    Action::"seo_meta_write",
    Action::"cms_collection_read",
    Action::"cms_entry_write",
    Action::"search_query",
    Action::"image_upload"
  ],
  resource
)
when {
  principal.tenant_id == resource.tenant_id
};

permit (
  principal in SitesRole::"tenant_publisher",
  action in [
    Action::"page_publish",
    Action::"page_revert",
    Action::"site_publish",
    Action::"sitemap_emit",
    Action::"robots_emit",
    Action::"cdn_publish",
    Action::"cdn_purge"
  ],
  resource
)
when {
  principal.tenant_id == resource.tenant_id
};

permit (
  principal in SitesRole::"tenant_admin",
  action in [
    Action::"site_delete",
    Action::"page_delete",
    Action::"theme_write",
    Action::"redirect_create",
    Action::"domain_bind",
    Action::"domain_unbind",
    Action::"cert_request",
    Action::"cms_collection_write",
    Action::"search_index_write"
  ],
  resource
)
when {
  principal.tenant_id == resource.tenant_id
};

// =====================================================================================
// CERT REVOCATION: 2-person rule.
// =====================================================================================

forbid (
  principal,
  action == Action::"cert_revoke",
  resource
)
unless {
  principal in SitesRole::"tenant_admin" &&
  context.two_person_rule_approved == true &&
  context.audit_chain_pre_emitted == true
};

// =====================================================================================
// AI-PAGE-BUILD (T2): per-tenant tier gate + EU AI Act overlay gate.
// =====================================================================================

permit (
  principal in SitesRole::"tenant_editor",
  action == Action::"ai_page_build",
  resource
)
when {
  principal.tenant_id == resource.tenant_id &&
  principal.tenant_tier in ["pro", "business", "enterprise"] &&
  // T2 is reversibility-window bound; not for HR/legal/medical overlays per ADR-SITES-0006.
  resource.context_overlay in ["marketing", "blog", "product", "landing", "intranet_general"] &&
  context.reversibility_window_seconds == 30
};

// REFUSE T2 in HR/legal/medical contexts until ADR-SITES-XXXX conformity assessment.
forbid (
  principal,
  action == Action::"ai_page_build",
  resource
)
when {
  resource.context_overlay in ["hr", "legal", "medical", "employment_decision", "credit_decision"]
};

// =====================================================================================
// LEGAL HOLD: refuse hard-delete on held pages.
// =====================================================================================

forbid (
  principal,
  action in [Action::"page_delete", Action::"site_delete"],
  resource
)
when {
  resource has legal_hold_open && resource.legal_hold_open == true
};

permit (
  principal in SitesRole::"compliance_officer_with_2person_rule",
  action == Action::"page_delete",
  resource
)
when {
  resource has legal_hold_open && resource.legal_hold_open == false &&
  context.two_person_rule_approved == true &&
  context.audit_chain_pre_emitted == true
};

// =====================================================================================
// CROSS-PACK DATA TRANSFER: forbid by default; permit with SCC.
// =====================================================================================

forbid (
  principal,
  action in [
    Action::"site_read",
    Action::"site_write",
    Action::"page_read",
    Action::"page_write",
    Action::"cms_collection_read",
    Action::"cms_collection_write"
  ],
  resource
)
when {
  principal has pack_tag && resource has pack_tag &&
  principal.pack_tag != resource.pack_tag
};

permit (
  principal in SitesRole::"cross_pack_reader",
  action in [Action::"page_read"],
  resource in PageVisibility::"public"
)
when {
  // Public-by-tenant-choice pages can be read cross-pack via CDN edge anycast (per ADR-SITES-0003).
  resource.visibility == "public" &&
  resource.public_opt_in == true
};

// =====================================================================================
// REDIRECT SCOPE: refuse cross-tenant redirect destinations.
// =====================================================================================

forbid (
  principal,
  action == Action::"redirect_create",
  resource
)
when {
  // Redirect target host must belong to the same tenant; cross-tenant 301 = potential hijack.
  resource.target_host_tenant_id != principal.tenant_id
};

// =====================================================================================
// DOMAIN BINDING: refuse if domain not DNS-verified.
// =====================================================================================

forbid (
  principal,
  action == Action::"cert_request",
  resource
)
when {
  resource has dns_verified && resource.dns_verified == false
};

// =====================================================================================
// LEAN check: drift detection on published-public pages.
// =====================================================================================
// LEAN lane `oya-check-public-page-drift` scans every Postgres row with `visibility=public`
// and verifies tenant-publisher opt-in signature on file. Drift triggers alert + auto-revert
// to draft + audit-chain emission.
