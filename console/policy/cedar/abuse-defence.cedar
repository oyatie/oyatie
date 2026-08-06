// ops-dashboard-control-center — abuse-defence policy fragment
// Cedar v4.2 LTS
//
// Purpose: internal-scraping prevention + per-employee scope enforcement.
//          UX floor: legitimate operators MUST experience zero friction on default path.
//          EMERGENCY_SERVICES bypass: never challenged per §3.2.3 LIFE-SAFETY HARD RULE.
//
// Owner:   ops-sre-reliability + council-security
// Binding ADR: ADR-0297 (abuse-defence), ADR-0298 (emergency-services bypass)
// References: ARCHITECTURE.md §abuse-defence, docs/standards/documentation-rigor.md §3.2.3
// Soak window: ≥60s per ADR-0294

// =============================================================================
// Default deny
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT — EMERGENCY_SERVICES unconditional bypass (LIFE-SAFETY HARD RULE)
// =============================================================================
//
// No challenge under any circumstance for emergency-services principals.
// Bypass at edge, not at µservice. Audit retained (never skipped).
// Cryptographic attestation required on principal.audience_type claim.

permit (
  principal,
  action,
  resource
)
when {
  principal has audience_type &&
  principal.audience_type == "EMERGENCY_SERVICES" &&
  principal has emergency_attestation_chain &&
  principal.emergency_attestation_chain != ""
};

// =============================================================================
// PERMIT — legitimate internal operator (below bot_score threshold, no anomaly)
// =============================================================================

permit (
  principal in OpsOperator::?p,
  action,
  resource
)
when {
  principal has audience_type &&
  principal.audience_type in ["INTERNAL_OPS", "PARTNER_AGENCY_OPS", "AUDITOR"] &&
  principal has authenticated &&
  principal.authenticated == true &&
  // Bot-score below threshold — default path is friction-free
  (!(principal has bot_score) || principal.bot_score <= 70) &&
  // Rate limit per operator: 1000 req/min
  (!(context has request_rate_per_minute) || context.request_rate_per_minute <= 1000)
};

// =============================================================================
// PERMIT — SECURITY_RESEARCHER allow-list (bug bounty / responsible disclosure)
// =============================================================================

permit (
  principal in SecurityResearcher::?p,
  action in [
    Action::"dashboard_view",
    Action::"incident_read",
    Action::"evidence_pack_read"
  ],
  resource
)
when {
  principal has audience_type &&
  principal.audience_type == "SECURITY_RESEARCHER" &&
  principal has bug_bounty_token &&
  principal.bug_bounty_token != ""
};

// =============================================================================
// FORBID — high bot-score (passive scoring; never on default path)
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
    principal.audience_type in ["EMERGENCY_SERVICES", "SECURITY_RESEARCHER"])
};

// =============================================================================
// FORBID — per-operator rate limit exceeded
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  context has request_rate_per_minute &&
  context.request_rate_per_minute > 1000 &&
  !(principal has audience_type &&
    principal.audience_type == "EMERGENCY_SERVICES")
};

// =============================================================================
// FORBID — honeypot tenant_id access (canary trap for scrapers)
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  resource has tenant_id &&
  resource has is_honeypot &&
  resource.is_honeypot == true
};

// =============================================================================
// FORBID — UEBA anomaly score above threshold triggers session suspend
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  principal has ueba_anomaly_score &&
  principal.ueba_anomaly_score > 90 &&
  !(principal has audience_type &&
    principal.audience_type == "EMERGENCY_SERVICES")
};
