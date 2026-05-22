// intelligence µservice — abuse-defence policy fragment
// Cedar v4
//
// Purpose: anti-bot, anti-spoof, anti-scrape, per docs/standards/documentation-rigor.md §3.2.3.
// Owner:   ops-security + axis-intelligence
// References: documentation-rigor §3.2.3.

// =============================================================================
// Default deny
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// FORBID — Unattested automated client (no User-Agent + no client-attestation token)
// =============================================================================

forbid (
  principal,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  resource has request_meta &&
  resource.request_meta has user_agent &&
  resource.request_meta.user_agent == "" &&
  resource.request_meta has client_attestation &&
  resource.request_meta.client_attestation == null
};

// =============================================================================
// FORBID — IP reputation block (per shared abuse-IP feed)
// =============================================================================

forbid (
  principal,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  resource has request_meta &&
  resource.request_meta has source_ip_reputation &&
  resource.request_meta.source_ip_reputation == "blocked"
};

// =============================================================================
// FORBID — High-cardinality scrape pattern (one principal querying many distinct topics rapidly)
// =============================================================================

forbid (
  principal,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  principal has scrape_signal &&
  principal.scrape_signal == true
};

// =============================================================================
// FORBID — Spoofed audience-tag (audience claim mismatch with bearer token claim)
// =============================================================================

forbid (
  principal,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  principal has token_audience &&
  resource has audience_tag &&
  principal.token_audience != resource.audience_tag
};

// =============================================================================
// PERMIT — Clean dispatch from attested client
// =============================================================================

permit (
  principal,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  resource has request_meta &&
  resource.request_meta has client_attestation &&
  resource.request_meta.client_attestation != null &&
  resource.request_meta has source_ip_reputation &&
  resource.request_meta.source_ip_reputation == "clean"
};
