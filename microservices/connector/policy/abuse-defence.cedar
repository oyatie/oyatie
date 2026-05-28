// connector service — abuse-defence policy fragment
// Cedar v4.2 LTS
//
// Purpose: defence-in-depth per documentation-rigor §3.2.3 — anti-bot,
//          anti-spoof, anti-scrape; UX-floor compliant.
// Owner:   axis-integration + ops-security + product-ux
// References: documentation-rigor.md §3.2.3; ADR-0297; threat-model.md T-D-01/T-D-03
// soak-window-seconds: 60
// publish-stage: active

// =============================================================================
// FORBID — confirmed-bot above threshold (unless friendly-crawler partner)
// =============================================================================
//
// Bot-score 0-100 (Cloudflare Bot Management / Akamai Bot Manager parity).
// >95 = "confirmed bot". Friendly crawlers (search engines + a11y tooling +
// accredited researchers) get explicit allow-list bypass.

forbid (principal, action, resource)
when {
  principal has bot_score &&
  principal.bot_score > 95 &&
  principal has tenant &&
  principal.tenant has audience_type &&
  principal.tenant.audience_type != "FRIENDLY_CRAWLER_PARTNER"
};

// =============================================================================
// FORBID — request rate over per-tenant + per-route + per-action budget
// =============================================================================

forbid (principal, action, resource)
when {
  principal has request_rate_per_minute &&
  resource has rate_limit_per_minute &&
  principal.request_rate_per_minute > resource.rate_limit_per_minute
};

// =============================================================================
// FORBID — scrape-pattern: breadth-first crawl at high depth
// =============================================================================
//
// Anti-scrape per §3.2.3 row 2: detect breadth-first crawl signatures.

forbid (
  principal == ?,
  action in [Action::"catalog_read", Action::"connector_metadata_read"],
  resource
)
when {
  principal has fingerprint_age_seconds &&
  principal.fingerprint_age_seconds < 30 &&
  context has crawl_depth &&
  context.crawl_depth > 50
};

// =============================================================================
// FORBID — credential-stuffing (HIBP corpus match on OAuth callback)
// =============================================================================

forbid (
  principal,
  action == Action::"oauth_grant_callback",
  resource
)
when {
  context has password_in_hibp_corpus &&
  context.password_in_hibp_corpus == true
};

// =============================================================================
// PERMIT — challenge presentation (adaptive; UX-floor compliant)
// =============================================================================
//
// UX-floor invariants (documentation-rigor §3.2.3 UX-floor table):
//  - Default path is friction-free: bot_score ≤ 50 → zero challenge.
//  - Latency budget unchanged: edge ML scoring is passive; ≤2ms p99 added.
//  - Cognitive load floor: solvable in ≤10s.
//  - Accessibility floor: every challenge has audio + keyboard + screen-reader alternative.
//  - Tenant-tier-adaptive: paid tiers lower sensitivity; anonymous strictest.

permit (
  principal,
  action == Action::"challenge_present",
  resource
)
when {
  principal has bot_score &&
  principal.bot_score >= 51 &&
  principal.bot_score <= 95 &&
  context has challenge_class &&
  context.challenge_class in [
    "turnstile_invisible",
    "hcaptcha_visible",
    "webauthn_attestation",
    "app_attest_ios",
    "play_integrity_android",
    "js_proof_of_work_lite"
  ] &&
  context has a11y_alternative_available &&
  context.a11y_alternative_available == true &&
  context has solve_budget_seconds &&
  context.solve_budget_seconds <= 10 &&
  context has session_continuity_preserved &&
  context.session_continuity_preserved == true
};

// =============================================================================
// PERMIT — friendly-crawler-partner bypass (search + a11y + accredited)
// =============================================================================

permit (
  principal,
  action in [Action::"catalog_read", Action::"connector_metadata_read"],
  resource
)
when {
  principal has tenant &&
  principal.tenant has audience_type &&
  principal.tenant.audience_type == "FRIENDLY_CRAWLER_PARTNER" &&
  context has crawler_attestation_valid &&
  context.crawler_attestation_valid == true
};

// =============================================================================
// FORBID — honeypot route hit
// =============================================================================

forbid (principal, action, resource in HoneypotRoute::?r);

// =============================================================================
// FORBID — canary payload echo (scraper ingested canary API key + tried to use)
// =============================================================================

forbid (principal, action, resource)
when {
  context has uses_canary_secret &&
  context.uses_canary_secret == true
};
