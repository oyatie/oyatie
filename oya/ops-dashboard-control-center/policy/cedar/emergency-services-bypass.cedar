// ops-dashboard-control-center — emergency-services-bypass policy fragment
// Cedar v4.2 LTS
//
// Purpose: implement the LIFE-SAFETY HARD RULE — emergency-services principals
//          MUST NEVER see a challenge, rate-limit block, or step-up friction.
//          This fragment is evaluated at the EDGE (L0) before any other fragment.
//          Bypass does NOT skip audit — every emergency request is still audit-emitted.
//
// Owner:   ops-sre-reliability + council-legal
// Binding ADR: ADR-0298 (emergency-services bypass, wave-3 backlog)
// References: docs/standards/documentation-rigor.md §3.2.3 LIFE-SAFETY HARD RULE,
//             ARCHITECTURE.md §critical-path-edge-cases row 1,
//             compliance.md §critical-path-edge-cases
// Regulatory: US FCC 47 CFR §9 (E9-1-1), NENA i3, HHS 988, NCMEC 18 USC §2258A,
//             EU EECC Art.109, KR 119 mandate, JP 119 mandate
// Soak window: ≥60s per ADR-0294; this fragment must soak before ANY modification.

// =============================================================================
// PERMIT — unconditional pass for cryptographically-attested EMERGENCY_SERVICES
// =============================================================================
//
// Evaluation order: this PERMIT fires BEFORE any FORBID in other fragments.
// The `emergency_attestation_chain` must be one of:
//   - NENA-i3-registered ESInet edge identity
//   - Carrier-verified SIP-TRUNK origin
//   - Jurisdiction-registered emergency-services tenant
//   - NCMEC-registered submitter
//   - FEMA-registered responder
//   - Federated identity from known crisis-line operator (988/Samaritans/KR-1393)
//
// Rate-limit floor is elevated (10× consumer tier), NOT zero —
// a compromised emergency-services tenant cannot DoS the platform.

permit (
  principal,
  action,
  resource
)
when {
  principal has audience_type &&
  principal.audience_type == "EMERGENCY_SERVICES" &&
  principal has emergency_attestation_chain &&
  principal.emergency_attestation_chain != "" &&
  // Audit emission is NEVER skipped — bypass + audit = accountability without friction
  context has audit_emission_confirmed &&
  context.audit_emission_confirmed == true
};

// =============================================================================
// PERMIT — crisis-hotline bot (EMERGENCY_SERVICES + bot_class = CRISIS_TRIAGE)
// =============================================================================
//
// Crisis-line text-bot triage (988 chat, Samaritans, Crisis Text Line) routes
// legitimate automated intake — bot-score MUST NOT block them.

permit (
  principal,
  action,
  resource
)
when {
  principal has audience_type &&
  principal.audience_type == "EMERGENCY_SERVICES" &&
  principal has bot_class &&
  principal.bot_class == "CRISIS_TRIAGE" &&
  principal has emergency_attestation_chain &&
  principal.emergency_attestation_chain != ""
};

// =============================================================================
// FORBID — forged EMERGENCY_SERVICES claim (no attestation chain)
// =============================================================================
//
// Forgery defense: revoke the single attestation at trust root; do NOT challenge
// legitimate emergency traffic. If attestation chain is missing/invalid, DENY.

forbid (
  principal,
  action,
  resource
)
when {
  principal has audience_type &&
  principal.audience_type == "EMERGENCY_SERVICES" &&
  (!(principal has emergency_attestation_chain) ||
   principal.emergency_attestation_chain == "")
};

// =============================================================================
// FORBID — EMERGENCY_SERVICES cannot bypass audit emission
// =============================================================================
//
// Bypass + audit = accountability without friction. This is the hard rule.
// If audit_emission_confirmed is false, even EMERGENCY_SERVICES gets denied.
// (The audit middleware must guarantee confirmation before this check.)

forbid (
  principal,
  action,
  resource
)
when {
  principal has audience_type &&
  principal.audience_type == "EMERGENCY_SERVICES" &&
  action != Action::"oncall_emergency_page" &&
  (!(context has audit_emission_confirmed) ||
   context.audit_emission_confirmed == false)
};
