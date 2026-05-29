// Cedar v4.2 LTS — finops-portal abuse-defence per ADR-0297 + §3.2.3
// x-fragment-soak-seconds: 60
// UX-FLOOR: legitimate finops users see zero added friction.

// 1. High bot-score forbid.
forbid (principal, action, resource) when {
    principal.bot_score > 95
    && !(principal.tenant.audience_type == "FRIENDLY_CRAWLER_PARTNER")
};

// 2. Per-fingerprint rate-limit on drill-down queries (expensive).
forbid (principal, action == Action::"view_cost_drilldown", resource) when {
    principal.request_rate_per_minute > resource.tenant.drilldown_quota_per_min
};

// 3. FOCUS export anti-scrape (high-fanout downloads).
forbid (principal, action == Action::"download_focus_export", resource) when {
    principal.focus_export_count_per_hour > 12  // 1 / 5min sustained = abuse
};

// 4. Cost-allocation-policy editor: WebAuthn step-up required (high blast radius).
forbid (principal, action == Action::"change_cost_allocation_policy", resource) when {
    principal.webauthn_assertion_age_seconds > 300
};

// 5. Stolen-credential check on admin sign-in.
forbid (principal, action == Action::"finops_admin_signin", resource) when {
    principal.credential_in_hibp_corpus == true
};

// 6. Webhook anti-spoof.
forbid (principal, action == Action::"webhook_ingest", resource) when {
    principal.hmac_signature_valid == false
    || principal.replay_window_seconds > 300
    || principal.idempotency_key_present == false
};

// 7. SPIFFE workload-identity required.
forbid (principal, action, resource) when {
    principal.principal_kind == "workload"
    && principal.spiffe_attested == false
};

// 8. UX-floor — never default-path challenge.
forbid (principal, action == Action::"challenge_presented", resource) when {
    principal.bot_score < principal.tenant.bot_score_challenge_threshold
};

// 9. Accessibility floor.
forbid (principal, action == Action::"challenge_presented", resource) when {
    resource.a11y_audio_alternative == false
    || resource.a11y_keyboard_navigable == false
    || resource.a11y_time_pressured == true
};
