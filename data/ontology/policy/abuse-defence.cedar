// Cedar v4.2 LTS — Ontology µservice abuse-defence baseline
// Authority: docs/decisions/ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md
// Authority: docs/standards/documentation-rigor.md §3.2.3
// Soak window: ≥60s per ADR-0294 before promotion to GA cells
// UX floor: docs/standards/documentation-rigor.md §3.2.3 UX-floor table
//
// PRECEDENT: Cloudflare Bot Management + Stripe Radar passive-scoring + Apple App Attest
// + WebAuthn Origin-binding.
//
// Default-path invariant: bot_score ≤ 95 + request_rate ≤ tenant_quota → ZERO friction.

// ---- Anti-bot ----

forbid (
  principal,
  action,
  resource
) when {
  principal.bot_score > 95
  && !(principal has tenant && principal.tenant.audience_type == "FRIENDLY_CRAWLER_PARTNER")
};

forbid (
  principal,
  action,
  resource
) when {
  principal has request_rate_per_minute
  && resource has rate_limit_per_minute
  && principal.request_rate_per_minute > resource.rate_limit_per_minute
};

// ---- Anti-scrape ----

forbid (
  principal,
  action in [Action::"OntologyEntityRead", Action::"OntologyQueryExecute"],
  resource
) when {
  principal has fingerprint_age_seconds
  && action has depth
  && principal.fingerprint_age_seconds < 30
  && action.depth > 50
};

forbid (
  principal,
  action == Action::"OntologyEntityRead",
  resource
) when {
  // Breadth-first ULID sweep detection — sequential entity-id reads are an anti-pattern
  principal has read_pattern_anomaly_score
  && principal.read_pattern_anomaly_score > 80
  && !(principal has tenant && principal.tenant.audience_type in ["INTERNAL_SUBSTRATE", "FRIENDLY_CRAWLER_PARTNER"])
};

// ---- Anti-spoof ----

forbid (
  principal,
  action,
  resource
) when {
  // SPIFFE workload identity required for µservice-to-µservice calls
  principal has principal_type
  && principal.principal_type == "WORKLOAD"
  && !(principal has spiffe_id)
};

forbid (
  principal,
  action in [Action::"OntologyTypeRegister", Action::"OntologyTypeDeprecate"],
  resource
) when {
  // Type-registry mutation requires phishing-resistant MFA (WebAuthn / passkey)
  !(principal has auth_class && principal.auth_class in ["WEBAUTHN_PASSKEY", "WEBAUTHN_PLATFORM"])
};

// ---- UX-floor invariants encoded as positive permits (default-path stays smooth) ----

permit (
  principal,
  action in [Action::"OntologyEntityRead", Action::"OntologyQueryExecute"],
  resource
) when {
  principal has bot_score
  && principal.bot_score <= 95
  && principal has request_rate_per_minute
  && resource has rate_limit_per_minute
  && principal.request_rate_per_minute <= resource.rate_limit_per_minute
  // Pass-through with zero friction — bot-mgmt scoring is passive
};

// ---- Friendly-crawler allow-list bypass ----

permit (
  principal,
  action == Action::"OntologyEntityRead",
  resource
) when {
  principal has tenant
  && principal.tenant.audience_type == "FRIENDLY_CRAWLER_PARTNER"
  && principal has crawler_partner_id
  // Search engines + a11y tooling + accredited researchers bypass all challenges
};
