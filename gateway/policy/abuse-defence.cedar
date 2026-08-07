// api-gateway µservice — abuse-defence fragment
// Cedar v4.2 LTS
// Authority: ADR-0297 (in flight) + documentation-rigor.md §3.2.3.

// =============================================================================
// FORBID — bot-score >95 (unless friendly-crawler verified)
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  principal has bot_score &&
  principal.bot_score > 95 &&
  !(principal has audience_type && principal.audience_type == "FRIENDLY_CRAWLER_PARTNER")
};

// =============================================================================
// FORBID — fingerprint too young + deep pagination (scrape signature)
// =============================================================================

forbid (
  principal,
  action in [
    Action::"Route::read_tenant",
    Action::"Route::read_public",
    Action::"Route::list_tenant"
  ],
  resource
)
when {
  principal has fingerprint_age_seconds &&
  principal.fingerprint_age_seconds < 30 &&
  resource has page_depth &&
  resource.page_depth > 50
};

// =============================================================================
// FORBID — honeypot route accessed by non-friendly principal
// =============================================================================

forbid (
  principal,
  action,
  resource in HoneypotRoute::?r
)
when {
  !(principal has audience_type && principal.audience_type == "FRIENDLY_CRAWLER_PARTNER")
};

// =============================================================================
// FORBID — sequential-ID scrape signature
// =============================================================================
// Scraper signature: sequential numeric IDs at high rate.

forbid (
  principal,
  action,
  resource
)
when {
  principal has sequential_id_rate_per_minute &&
  principal.sequential_id_rate_per_minute > 100
};

// =============================================================================
// FORBID — alphabetical-pagination scrape signature
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  principal has alphabetical_pagination_rate_per_minute &&
  principal.alphabetical_pagination_rate_per_minute > 60
};

// =============================================================================
// PERMIT — CAPTCHA-passed principal at higher bot-score threshold
// =============================================================================

permit (
  principal,
  action,
  resource
)
when {
  principal has bot_score &&
  principal.bot_score < 90 &&
  principal has captcha_passed &&
  principal.captcha_passed == true &&
  principal has captcha_age_seconds &&
  principal.captcha_age_seconds < 1800
};

// =============================================================================
// FORBID — stolen-credential principal
// =============================================================================
// HIBP / internal credential-stuffing-detector flagged.

forbid (
  principal,
  action == Action::"Route::auth_attempt",
  resource
)
when {
  principal has credential_stuffing_score &&
  principal.credential_stuffing_score > 80
};

// =============================================================================
// FORBID — JA4 fingerprint on residential-proxy network at high request rate
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  principal has asn_class &&
  principal.asn_class == "residential_proxy" &&
  principal has request_rate_per_minute &&
  principal.request_rate_per_minute > 30
};

// =============================================================================
// FORBID — known-bad ASN
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  principal has asn_reputation &&
  principal.asn_reputation == "known_bad"
};

// =============================================================================
// PERMIT — adaptive-challenge passed
// =============================================================================
// JS proof-of-work / Turnstile / hCaptcha challenge completed.

permit (
  principal,
  action,
  resource
)
when {
  principal has adaptive_challenge_passed &&
  principal.adaptive_challenge_passed == true &&
  principal has challenge_age_seconds &&
  principal.challenge_age_seconds < 600 &&
  principal has bot_score &&
  principal.bot_score < 92
};
