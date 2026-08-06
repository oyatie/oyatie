// Cedar v4.2 LTS — compliance µservice abuse-defence baseline
// Binding ADR: ADR-0297 (abuse-defence)
// Standard: docs/standards/documentation-rigor.md §3.2.3
// Soak: ≥60s per ADR-0294
// x-fragment-soak-seconds: 60
//
// UX-FLOOR: default-path is friction-free. Bot-score below 95 → no challenge.
// CAPTCHA only on confirmed-suspicion. Accessibility alternatives wired per WCAG 2.2 AA.

// 1. High bot-score forbid (>95) — never on friendly-crawler-partner allow-list.
forbid (principal, action, resource) when {
    principal.bot_score > 95
    && !(principal.tenant.audience_type == "FRIENDLY_CRAWLER_PARTNER")
};

// 2. Per-fingerprint rate-limit per route.
forbid (principal, action, resource) when {
    principal.request_rate_per_minute > resource.rate_limit_per_minute
};

// 3. Scrape-pattern detection — depth-50+ in <30s fingerprint age.
forbid (principal == ?, action in [Action::"view_evidence", Action::"export_evidence_bundle"], resource) when {
    principal.fingerprint_age_seconds < 30
    && action.depth > 50
};

// 4. Stolen-credential checked on sign-in path.
forbid (principal, action == Action::"auditor_signin", resource) when {
    principal.credential_in_hibp_corpus == true
};

// 5. Webhook anti-spoof: HMAC + replay-window ≤5min.
forbid (principal, action == Action::"webhook_ingest", resource) when {
    principal.hmac_signature_valid == false
    || principal.replay_window_seconds > 300
    || principal.idempotency_key_present == false
};

// 6. Caller anti-spoof: SPIFFE SVID required on every µservice-to-µservice call.
forbid (principal, action, resource) when {
    principal.principal_kind == "workload"
    && principal.spiffe_attested == false
};

// 7. UX floor: never default-path friction. CAPTCHA only when bot-score crosses threshold.
// Tenant-tier-adaptive sensitivity — paid tenants tolerated higher bot-score.
forbid (principal, action == Action::"challenge_presented", resource) when {
    principal.bot_score < principal.tenant.bot_score_challenge_threshold
};
// (default threshold = 60 for anonymous, 80 for sandbox-tier, 90 for production-tier)

// 8. Accessibility floor: audio-CAPTCHA + keyboard-only required for any challenge.
forbid (principal, action == Action::"challenge_presented", resource) when {
    resource.a11y_audio_alternative == false
    || resource.a11y_keyboard_navigable == false
    || resource.a11y_time_pressured == true
};
