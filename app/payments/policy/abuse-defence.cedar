// payments µservice — abuse-defence Cedar v4.2 LTS fragment
//
// Purpose: anti-bot + anti-spoof + anti-scrape per docs/standards/documentation-rigor.md §3.2.3.
//          Composes bot-score + rate-limit + fingerprint-age + pattern-anomaly.
// Owner:   ops-security + ops-fraud
// References: documentation-rigor.md §3.2.3; threat-model.md T-D-01
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar
// @soak-window-seconds: 60   per ADR-0294
// @publisher-svid: spiffe://oyatie/payments/foundry-agent

// =============================================================================
// Default deny — this fragment is purely composable forbid clauses. The base
// payments fragments (charge/refund/payout) provide the permits; this fragment
// adds defence-in-depth forbids that overlay every permit.
// =============================================================================

// =============================================================================
// ANTI-BOT — block when bot-score > 95 (unless friendly-crawler-partner)
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  principal has bot_score &&
  principal.bot_score > 95 &&
  !(principal has audience_type &&
    principal.audience_type == "FRIENDLY_CRAWLER_PARTNER")
};

// =============================================================================
// ANTI-BOT — rate-limit per principal
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
}
unless {
  // Validated emergency-services operators are exempt from the ordinary
  // per-minute rate limit during mass-casualty surges (§3.2.5 rows 1 + 22).
  // Modeled as an exception to THIS forbid rather than as a competing permit:
  // Cedar gives forbids precedence over permits, so a separate permit could
  // never override this denial.
  principal has audience_type &&
  principal.audience_type == "EMERGENCY_SERVICES_OPERATOR" &&
  principal has svid_present && principal.svid_present == true &&
  principal has svid_valid && principal.svid_valid == true &&
  principal has emergency_surge_token &&
  principal.emergency_surge_token != "" &&
  (!(principal has tenant_id && resource has tenant_id) ||
    principal.tenant_id == resource.tenant_id)
};

// =============================================================================
// PERMIT — authenticated production PSP webhook ingestion
// =============================================================================
//
// A production PSP webhook (stripe/adyen/toss/kakaopay/line-pay/wechat-pay/
// alipay) is ingested when the caller is provider-bound and the payload carries
// positive HMAC + replay-window evidence. Without this permit Cedar's implicit
// default deny would reject every production callback (the only matching permit
// elsewhere is the sandbox CI scope); the replay/HMAC forbids below remain as
// defense-in-depth.

permit (
  principal in WebhookCaller::?w,
  action == Action::"webhook.receive",
  resource in WebhookPayload::?p
)
when {
  principal has provider &&
  resource has provider &&
  principal.provider == resource.provider &&
  resource has hmac_valid &&
  resource.hmac_valid == true &&
  resource has webhook_age_seconds &&
  resource.webhook_age_seconds <= 300
};

// =============================================================================
// ANTI-SCRAPE — fresh-fingerprint + deep-pagination signature
// =============================================================================
//
// Fingerprint younger than 30 seconds attempting deep pagination is a strong
// scraper signal (legitimate clients warm caches and pace requests).

forbid (
  principal,
  action in [
    Action::"charge.read",
    Action::"payout.read",
    Action::"refund.read",
    Action::"sub-merchant.read"
  ],
  resource
)
when {
  principal has fingerprint_age_seconds &&
  principal.fingerprint_age_seconds < 30 &&
  resource has pagination_depth &&
  resource.pagination_depth > 50
};

// =============================================================================
// ANTI-SCRAPE — sequential-ID enumeration pattern
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  principal has sequential_id_pattern_score &&
  principal.sequential_id_pattern_score > 80
};

// =============================================================================
// ANTI-SPOOF — webhook replay outside HMAC-replay-window
// =============================================================================
//
// Webhooks older than 5 minutes are rejected (HMAC + replay-window).

forbid (
  principal in WebhookCaller::?w,
  action == Action::"webhook.receive",
  resource in WebhookPayload::?p
)
when {
  resource has webhook_age_seconds &&
  resource.webhook_age_seconds > 300
};

// =============================================================================
// ANTI-SPOOF — webhook HMAC invalid
// =============================================================================

forbid (
  principal in WebhookCaller::?w,
  action == Action::"webhook.receive",
  resource in WebhookPayload::?p
)
when {
  resource has hmac_valid &&
  resource.hmac_valid == false
};

// =============================================================================
// ANTI-SPOOF — caller SVID missing or invalid (cross-µservice mTLS)
// =============================================================================

forbid (
  principal in MicroserviceCaller::?m,
  action,
  resource
)
when {
  !(principal has svid_present) ||
  principal.svid_present == false ||
  (principal has svid_valid && principal.svid_valid == false)
};

// =============================================================================
// ANTI-SPOOF — step-up auth required for payout > $10k (per documentation-rigor §3.2.3)
// =============================================================================
//
// Step-up is a USER action taken at payout SCHEDULING time, so the verified
// evidence must travel on the payout RESOURCE (persisted when the tenant
// completes passkey/webauthn step-up), not on the later payout-worker
// principal — the worker identity never carries the user's step-up attribute,
// and without resource-carried evidence every high-value execution would be
// denied even after a valid step-up.
//
// The `unless` admits the validated emergency-services exception (documented
// in emergency-services-bypass.cedar): hospital/911 corridors process >$10k
// invoices without interactive step-up. Cedar forbids override permits, so the
// exception must live on this forbid, not on a competing permit.

forbid (
  principal,
  action == Action::"payout.execute",
  resource in Payout::?o
)
when {
  resource has amount_minor &&
  resource.amount_minor > 1000000 &&
  !(resource has step_up_auth_class &&
    resource.step_up_auth_class in ["passkey", "webauthn"])
}
unless {
  principal has audience_type &&
  principal.audience_type == "EMERGENCY_SERVICES_OPERATOR" &&
  principal has svid_present &&
  principal.svid_present == true &&
  principal has svid_valid &&
  principal.svid_valid == true
};

// =============================================================================
// ANTI-SCRAPE — honeypot route hit
// =============================================================================
//
// Honeypot routes that no legitimate client should ever hit.
// Hitting them is conclusive evidence of automated scraping.

forbid (
  principal,
  action,
  resource in HoneypotResource::?h
);

// =============================================================================
// ANTI-BOT — credential-stuffing pattern (per ADR-0297 forthcoming abuse-defence)
// =============================================================================

forbid (
  principal,
  action == Action::"charge.create",
  resource
)
when {
  principal has credential_stuffing_score &&
  principal.credential_stuffing_score > 70
};

// =============================================================================
// ANTI-BOT — JA4+ fingerprint blocklist
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  principal has ja4_fingerprint &&
  principal.ja4_fingerprint in [
    "t13d1517h2_8daaf6152771_b1ff8ab2d16f",  // known abuse fingerprint
    "t13d1516h2_8daaf6152771_b1ff8ab2d16f"   // sister abuse fingerprint
  ]
};
