// Cedar v4.2 LTS — Social µservice abuse-defence baseline (EXTENSIVE)
// Authority: ADR-0297; documentation-rigor.md §3.2.3
// Soak ≥60s per ADR-0294.
// Hyperscaler precedent: Twitter/X anti-spam + Meta CIB detection + Cloudflare Bot Management
//                        + Apple App Attest + Google Play Integrity.
// UX-floor (CRITICAL): default-path is friction-free; challenge only on confirmed suspicion.

// ---- Anti-bot: account creation ----

forbid (
  principal,
  action == Action::"SocialAccountSignup",
  resource
) when {
  principal.bot_score > 95
};

forbid (
  principal,
  action == Action::"SocialAccountSignup",
  resource
) when {
  // Sock-puppet cluster signal: same device fingerprint + email-domain + IP-range
  // already linked to ≥3 accounts within 24h
  principal has sock_puppet_cluster_score
  && principal.sock_puppet_cluster_score > 80
};

forbid (
  principal,
  action == Action::"SocialAccountSignup",
  resource
) when {
  // No device attestation on supported clients = block
  action has client_class
  && action.client_class in ["NATIVE_IOS", "NATIVE_ANDROID"]
  && !(action has device_attestation && action.device_attestation == "VALID")
};

// ---- Anti-bot: engagement ----

forbid (
  principal,
  action in [Action::"SocialFollow", Action::"SocialEngagementEmit", Action::"SocialPost"],
  resource
) when {
  principal has request_rate_per_minute
  && resource has rate_limit_per_minute
  && principal.request_rate_per_minute > resource.rate_limit_per_minute
};

forbid (
  principal,
  action == Action::"SocialFollow",
  resource
) when {
  // Engagement-bot pattern: rapid follow/unfollow cycling
  principal has follow_churn_rate_per_hour
  && principal.follow_churn_rate_per_hour > 100
};

// ---- Anti-spoof: impersonation ----

forbid (
  principal,
  action == Action::"SocialAccountSignup",
  resource
) when {
  // Confusingly-similar display-name or handle to verified account (Levenshtein ≤2)
  action has display_name_similarity_to_verified
  && action.display_name_similarity_to_verified > 90
};

forbid (
  principal,
  action == Action::"SocialFederationInboundAccept",
  resource
) when {
  // ActivityPub inbound: HTTP signature must verify against actor pubkey
  !(action has ap_signature_verified && action.ap_signature_verified == true)
};

forbid (
  principal,
  action == Action::"SocialFederationInboundAccept",
  resource
) when {
  // Replay window ≤5min
  action has signed_at_skew_seconds
  && action.signed_at_skew_seconds > 300
};

forbid (
  principal,
  action,
  resource
) when {
  // SPIFFE workload identity required for substrate calls
  principal has principal_type
  && principal.principal_type == "WORKLOAD"
  && !(principal has spiffe_id)
};

// ---- Anti-scrape ----

forbid (
  principal,
  action in [Action::"SocialProfileRead", Action::"SocialFeedRead", Action::"SocialSearch"],
  resource
) when {
  // Breadth-first or sequential-id traversal pattern
  principal has read_pattern_anomaly_score
  && principal.read_pattern_anomaly_score > 80
  && !(principal has tenant && principal.tenant.audience_type in ["INTERNAL_SUBSTRATE", "FRIENDLY_CRAWLER_PARTNER"])
};

forbid (
  principal,
  action == Action::"SocialProfileRead",
  resource
) when {
  // Unauthenticated profile-read: very low per-IP cap
  !(principal has tenant)
  && principal has unauthenticated_read_rate_per_minute
  && principal.unauthenticated_read_rate_per_minute > 30
};

forbid (
  principal,
  action == Action::"SocialBulkExport",
  resource
) when {
  // Bulk export requires paid-API tier
  !(principal has paid_api_tier && principal.paid_api_tier in ["TIER_1", "TIER_2", "TIER_3"])
};

// ---- Friendly-crawler allow-list bypass ----

permit (
  principal,
  action in [Action::"SocialProfileRead", Action::"SocialPostRead"],
  resource
) when {
  principal has tenant
  && principal.tenant.audience_type == "FRIENDLY_CRAWLER_PARTNER"
  && resource has public_listed
  && resource.public_listed == true
};

// ---- UX-floor default permit (default path is friction-free) ----

permit (
  principal,
  action in [Action::"SocialFeedRead", Action::"SocialProfileRead", Action::"SocialPost", Action::"SocialEngagementEmit", Action::"SocialFollow"],
  resource
) when {
  principal.bot_score <= 95
  && principal.request_rate_per_minute <= resource.rate_limit_per_minute
  && !(principal has minor_age_band && principal.minor_age_band == "COPPA_UNDER_13")
};
