// intelligence µservice — critical-path emergency-services bypass
// Cedar v4
//
// Purpose: Implement the LIFE-SAFETY HARD RULE from docs/standards/documentation-rigor §3.2.3.
//          Emergency-services principals NEVER see a challenge, rate-limit refusal, or
//          score-gate refusal. Bypass happens at this Cedar layer (L3 policy) BEFORE any
//          bot-score or rate-limit gate fires.
//
// Authority: documentation-rigor §3.2.3 (UX-floor + LIFE-SAFETY HARD RULE)
//            documentation-rigor §3.2.5 row 1 (emergency services)
//            ADR-0244 audience_type = EMERGENCY_SERVICES
//            ADR-0298 (pending — emergency-services bypass life-safety)
//
// Owner:    ops-security + council-privacy + axis-intelligence
//
// IMPORTANT: This policy does NOT skip audit emission. Every emergency-services dispatch
//            is still tenant-scoped + tenant_id-stamped + audit-emitted per ADR-0263.
//            The audit-event-class is AbuseDefenceEmergencyServiceBypass.
//
// Attestation requirement: audience_type = EMERGENCY_SERVICES is set ONLY when the
//            principal holds a cryptographically-attested emergency-services identity:
//            - NENA-i3-registered ESInet edge identity
//            - Carrier-verified SIP-TRUNK origin
//            - Jurisdiction-registered emergency-services tenant (ADR-0244)
//            - NCMEC-registered submitter / FEMA-registered responder
//            - Federated identity from known crisis-line operator (988 / Samaritans / KR-1393)
//            Forgery defense: SPIFFE workload identity per ADR-0295 + cosign-signed federated
//            trust. On suspected forgery: revoke single attestation at trust root; do NOT
//            challenge other legitimate emergency-services traffic.

// =============================================================================
// PERMIT — Emergency-services principal: bypass all bot/rate/score gates
// =============================================================================

permit (
  principal,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  principal has audience_type &&
  principal.audience_type == "EMERGENCY_SERVICES" &&
  principal has emergency_attestation_valid &&
  principal.emergency_attestation_valid == true
};

// =============================================================================
// PERMIT — Crisis-triage bot acting for emergency-services tenant
//          (988 chat, Samaritans chat, Crisis Text Line, KR-1393 chatbot)
// =============================================================================

permit (
  principal,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  principal has audience_type &&
  principal.audience_type == "EMERGENCY_SERVICES" &&
  principal has bot_class &&
  principal.bot_class == "CRISIS_TRIAGE" &&
  principal has emergency_attestation_valid &&
  principal.emergency_attestation_valid == true
};

// =============================================================================
// PERMIT — Emergency-services principal: bypass rate-limit check action
// =============================================================================

permit (
  principal,
  action == Action::"rate_limit_check",
  resource
)
when {
  principal has audience_type &&
  principal.audience_type == "EMERGENCY_SERVICES" &&
  principal has emergency_attestation_valid &&
  principal.emergency_attestation_valid == true
};

// =============================================================================
// PERMIT — Emergency-services principal: bypass bot-score gate action
// =============================================================================

permit (
  principal,
  action == Action::"bot_score_gate",
  resource
)
when {
  principal has audience_type &&
  principal.audience_type == "EMERGENCY_SERVICES" &&
  principal has emergency_attestation_valid &&
  principal.emergency_attestation_valid == true
};

// =============================================================================
// FORBID — Forged EMERGENCY_SERVICES claim (no valid attestation)
//          Deny and emit AbuseDefenceForgeryDetected.
// =============================================================================

forbid (
  principal,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  principal has audience_type &&
  principal.audience_type == "EMERGENCY_SERVICES" &&
  (!(principal has emergency_attestation_valid) ||
   principal.emergency_attestation_valid == false)
};
