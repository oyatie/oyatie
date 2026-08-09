// api-gateway µservice — public-read fragment
// Cedar v4.2 LTS
// Authority: ADR-0243 (Cedar universal gate).

// =============================================================================
// PERMIT — anonymous principal reads public-classified resource
// =============================================================================
// Used for unauthenticated read endpoints (marketing pages, robots.txt, sitemap).

permit (
  principal,
  action in [
    Action::"Route::read_public",
    Action::"Route::sitemap",
    Action::"Route::robots_txt",
    Action::"Route::healthz"
  ],
  resource in PublicRoute::?r
)
when {
  resource has visibility &&
  resource.visibility == "public" &&
  principal has bot_score &&
  principal.bot_score < 80
};

// =============================================================================
// PERMIT — robots.txt / sitemap.xml served unconditionally
// =============================================================================
// Even high-bot-score principals get robots.txt — it's the ANTI-bot signal.

permit (
  principal,
  action in [
    Action::"Route::robots_txt",
    Action::"Route::sitemap"
  ],
  resource
);

// =============================================================================
// PERMIT — /healthz served unconditionally (k8s readiness)
// =============================================================================

permit (
  principal,
  action == Action::"Route::healthz",
  resource
)
when {
  principal has spiffe_id &&
  principal.spiffe_id like "spiffe://oyatie.cell-*/health-probe/*"
};

// =============================================================================
// FORBID — public-read attempted on tenant-scoped resource
// =============================================================================

forbid (
  principal,
  action == Action::"Route::read_public",
  resource in TenantRoute::?r
);
