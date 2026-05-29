// Cedar v4.2 LTS fragment — abuse-defence
// Microservice: feature-flags
// Binding ADR: ADR-0297 (abuse-defence) + ADR-0243
// Fragment soak window: ≥60s per ADR-0294
// Fragment version: 1.0.0
// UX-floor: see §3.2.3 of documentation-rigor.md
// LIFE-SAFETY: emergency-services bypass is unconditional (row 1 §3.2.5)
// Reviewed: 2026-05-20

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: emergency-services principals bypass ALL abuse-defence gates
// LIFE-SAFETY HARD RULE per §3.2.3 documentation-rigor.md
// Cryptographic attestation required; audit retained unconditionally
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal,
  action,
  resource
)
when {
  principal.audience_type == "EMERGENCY_SERVICES"
  && context.emergency_attestation_valid == true
};

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: friendly-crawler-partner bypass (search engines, accredited researchers)
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal,
  action == Action::"FlagEvaluate",
  resource
)
when {
  principal.audience_type == "FRIENDLY_CRAWLER_PARTNER"
};

// ─────────────────────────────────────────────────────────────────────────────
// FORBID: high bot-score on evaluation path
// UX-floor: only blocks when score > 95 (passive, pre-computed at edge)
// Default path (legitimate users) unaffected — zero added latency
// ─────────────────────────────────────────────────────────────────────────────
forbid (
  principal,
  action == Action::"FlagEvaluate",
  resource
)
when {
  principal.bot_score > 95
  && !(principal.audience_type == "EMERGENCY_SERVICES")
  && !(principal.audience_type == "FRIENDLY_CRAWLER_PARTNER")
};

// ─────────────────────────────────────────────────────────────────────────────
// FORBID: mutation rate limiting (anti-flag-flooding)
// Rate: >60 mutations/min per principal (engineering ceiling)
// Does not affect pack-overlay-agent (automated; elevated quota)
// ─────────────────────────────────────────────────────────────────────────────
forbid (
  principal,
  action in [Action::"FlagCreate", Action::"FlagUpdate", Action::"FlagArchive", Action::"FlagDelete"],
  resource
)
when {
  principal.flag_mutation_rate_per_minute > 60
  && !(principal == Principal::"oyatie.feature-flags.pack-overlay-agent")
  && !(principal.audience_type == "EMERGENCY_SERVICES")
};

// ─────────────────────────────────────────────────────────────────────────────
// FORBID: quota gate — max 1000 active flags per tenant (configurable)
// Prevents flag-flooding exhaustion of quota
// ─────────────────────────────────────────────────────────────────────────────
forbid (
  principal,
  action == Action::"FlagCreate",
  resource is FlagDefinition
)
when {
  resource.tenant.active_flag_count >= resource.tenant.flag_quota
  && !(principal.role == "platform-admin")
};

// ─────────────────────────────────────────────────────────────────────────────
// FORBID: experiment assignment manipulation — client cannot supply assignment hash
// Anti-experiment-manipulation per §abuse-defence in ARCHITECTURE.md
// ─────────────────────────────────────────────────────────────────────────────
forbid (
  principal,
  action == Action::"FlagEvaluate",
  resource is FlagDefinition
)
when {
  context.client_supplied_bucket != ""  // client attempting to inject bucket value
  && resource.flag_type == "experiment"
};

// ─────────────────────────────────────────────────────────────────────────────
// FORBID: honeypot flag evaluation (no legitimate client should hit these keys)
// Detection: access triggers AbuseDetected audit event (via post-evaluation hook)
// ─────────────────────────────────────────────────────────────────────────────
forbid (
  principal,
  action == Action::"FlagEvaluate",
  resource is FlagDefinition
)
when {
  resource.is_honeypot == true
};
