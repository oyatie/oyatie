// api-gateway µservice — rate-limit policy fragment
// Cedar v4.2 LTS
// Authority: ADR-0157 + ADR-0243 + ADR-0294 (fragment soak).

// =============================================================================
// FORBID — request rate exceeds per-route bucket
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  principal has request_rate_per_minute &&
  resource has rate_limit_per_minute &&
  principal.request_rate_per_minute > resource.rate_limit_per_minute
};

// =============================================================================
// FORBID — burst exceeds per-route burst-cap
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  principal has burst_count &&
  resource has burst_cap &&
  principal.burst_count > resource.burst_cap
};

// =============================================================================
// FORBID — anonymous auth-attempt rate over 5/min/IP
// =============================================================================

forbid (
  principal,
  action == Action::"Route::auth_attempt",
  resource
)
when {
  principal has audience_type &&
  principal.audience_type == "anonymous" &&
  principal has rate_per_minute_per_ip &&
  principal.rate_per_minute_per_ip > 5
};

// =============================================================================
// PERMIT 1 — friendly-crawler partner tenant_class (Googlebot / Bingbot / etc.)
// =============================================================================
// Higher rate-limit for verified friendly crawlers.

permit (
  principal in PartnerPrincipal::?p,
  action == Action::"Route::read_public",
  resource
)
when {
  principal has audience_type &&
  principal.audience_type == "FRIENDLY_CRAWLER_PARTNER" &&
  principal has crawler_verified &&
  principal.crawler_verified == true &&
  principal has request_rate_per_minute &&
  principal.request_rate_per_minute <= 6000
};

// =============================================================================
// FORBID — cost ceiling exceeded
// =============================================================================
// When tenant has consumed >hard-cap budget for the month, refuse.

forbid (
  principal in TenantPrincipal::?p,
  action,
  resource
)
when {
  principal has month_to_date_cost_usd &&
  principal has cost_hard_cap_usd &&
  principal.month_to_date_cost_usd > principal.cost_hard_cap_usd
};

// =============================================================================
// PERMIT 2 — paid tenant_class overrides
// =============================================================================
// paid tenant_class has higher defaults.

permit (
  principal in TenantPrincipal::?p,
  action,
  resource
)
when {
  principal has tenant_class &&
  principal.tenant_class == "paid" &&
  principal has request_rate_per_minute &&
  principal.request_rate_per_minute <= 60000
};

// =============================================================================
// FORBID — single-IP rate over 1000/min (anti-DDoS)
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  principal has rate_per_minute_per_ip &&
  principal.rate_per_minute_per_ip > 1000
};
