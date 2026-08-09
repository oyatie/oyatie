// Cedar v4.2 LTS — comms-email µservice abuse-defence per ADR-0297 + §3.2.3
// x-fragment-soak-seconds: 60
// UX-FLOOR: legitimate users see zero added friction; bot-score gate only.

// 1. High bot-score forbid.
forbid (principal, action, resource) when {
    principal.bot_score > 95
    && !(principal.tenant.audience_type == "FRIENDLY_CRAWLER_PARTNER")
};

// 2. Per-tenant + per-fingerprint outbound rate-limit (spam control).
forbid (principal, action == Action::"send_transactional", resource) when {
    principal.tenant_send_rate_per_min > resource.tenant.send_quota_per_min
};

forbid (principal, action == Action::"send_marketing", resource) when {
    principal.tenant_marketing_rate_per_hour > resource.tenant.marketing_quota_per_hour
};

// 3. Bounce-storm circuit-breaker.
forbid (principal, action == Action::"send_transactional", resource) when {
    resource.tenant.bounce_rate_15m > 5.0  // 5% bounce in 15min window
};

// 4. Reputation-drop circuit-breaker.
forbid (principal, action == Action::"send_marketing", resource) when {
    resource.tenant.reputation_score < 70
};

// 5. Stolen-credential check on admin sign-in.
forbid (principal, action == Action::"admin_signin", resource) when {
    principal.credential_in_hibp_corpus == true
};

// 6. Webhook anti-spoof: HMAC + replay window.
forbid (principal, action == Action::"webhook_ingest", resource) when {
    principal.hmac_signature_valid == false
    || principal.replay_window_seconds > 300
    || principal.idempotency_key_present == false
};

// 7. SPIFFE workload-identity required for substrate callers.
forbid (principal, action, resource) when {
    principal.principal_kind == "workload"
    && principal.spiffe_attested == false
};

// 8. UX-floor: never default-path challenge.
forbid (principal, action == Action::"challenge_presented", resource) when {
    principal.bot_score < principal.tenant.bot_score_challenge_threshold
};

// 9. Accessibility floor.
forbid (principal, action == Action::"challenge_presented", resource) when {
    resource.a11y_audio_alternative == false
    || resource.a11y_keyboard_navigable == false
    || resource.a11y_time_pressured == true
};

// 10. Anti-spam — inbound MX abuse.
forbid (principal, action == Action::"inbound_receive", resource) when {
    principal.spf_pass == false
    && principal.dkim_pass == false
    && principal.dmarc_pass == false
};
