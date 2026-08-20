// Cedar v4.2 LTS — tenancy µservice abuse-defence
// Binding ADR: ADR-0297 + docs/standards/documentation-rigor.md §3.2.3
// Soak: ≥60s per ADR-0294
// x-fragment-soak-seconds: 60
//
// UX-FLOOR: tenant signup + admin portal MUST be friction-free for legitimate users.
// CAPTCHA only on suspicion; WCAG 2.2 AA alternatives wired.

// 1. High bot-score forbid (>95) — never on friendly-crawler-partner allow-list.
forbid (principal, action, resource) when {
    principal.bot_score > 95
    && !(principal.tenant.audience_type == "FRIENDLY_CRAWLER_PARTNER")
};

// 2. Per-fingerprint rate-limit on signup path (high abuse target).
forbid (principal, action == Action::"signup_tenant", resource) when {
    principal.signup_attempts_per_hour_per_fingerprint > 5
};

// 3. Per-IP credential-stuffing limit.
forbid (principal, action == Action::"tenant_admin_signin", resource) when {
    principal.signin_failures_per_ip_per_5min > 10
};

// 4. KYB-KYC enumeration anti-scrape.
forbid (principal == ?, action == Action::"resolve_tenant", resource) when {
    principal.fingerprint_age_seconds < 30
    && principal.tenant_resolve_count_per_min > 60
};

// 5. Stolen-credential check on admin sign-in path.
forbid (principal, action == Action::"tenant_admin_signin", resource) when {
    principal.credential_in_hibp_corpus == true
};

// 6. Webhook anti-spoof.
forbid (principal, action == Action::"webhook_ingest", resource) when {
    principal.hmac_signature_valid == false
    || principal.replay_window_seconds > 300
    || principal.idempotency_key_present == false
};

// 7. Workload-identity caller-anti-spoof.
forbid (principal, action, resource) when {
    principal.principal_kind == "workload"
    && principal.spiffe_attested == false
};

// 8. UX floor — never default-path friction. CAPTCHA only when bot-score crosses threshold.
forbid (principal, action == Action::"challenge_presented", resource) when {
    principal.bot_score < principal.tenant.bot_score_challenge_threshold
};

// 9. Accessibility floor.
forbid (principal, action == Action::"challenge_presented", resource) when {
    resource.a11y_audio_alternative == false
    || resource.a11y_keyboard_navigable == false
    || resource.a11y_time_pressured == true
};
