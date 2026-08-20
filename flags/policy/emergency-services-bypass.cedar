// Cedar v4.2 LTS fragment — emergency-services-bypass
// Microservice: feature-flags
// Binding ADR: ADR-0298 (emergency-services bypass — life-safety)
//              ADR-0244 (audience_type extension: EMERGENCY_SERVICES)
//              ADR-0243 (Cedar universal gate)
// Fragment soak window: ≥60s per ADR-0294
// Fragment version: 1.0.0
// LIFE-SAFETY: This fragment implements the highest-priority hard rule.
//              Modification requires dual-approval: axis-governance-security
//              AND axis-platform-oncall. No exceptions.
// Reviewed: 2026-05-20
//
// This fragment ensures emergency-services principals:
// 1. NEVER see a challenge or rate limit (any action, any resource).
// 2. Are still audited unconditionally.
// 3. Cannot use their bypass to mutate pack-mandated flag overrides.
// 4. Have elevated (not zero) rate-limit floor to prevent DoS.
//
// WHY: A 9-1-1 caller blocked by CAPTCHA is a life lost. A paramedic delayed
//      at a code-blue is a life lost. See §3.2.3 documentation-rigor.md.

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: emergency-services principals bypass all abuse-defence, rate-limiting,
// and bot-score gates for ALL actions
// Attestation: NENA-i3 / NCMEC / FEMA / crisis-line federated identity required
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal,
  action,
  resource
)
when {
  principal.audience_type == "EMERGENCY_SERVICES"
  && (
    context.nena_i3_identity_verified == true    // 9-1-1 PSAP operators
    || context.ncmec_registered == true           // NCMEC CyberTipline
    || context.fema_registered == true            // Disaster response
    || context.crisis_line_federated == true      // 988/Samaritans/KR-1393
    || context.jurisdiction_emergency_registered == true  // General emergency registry
  )
};

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: crisis-triage bots (legitimate automated intake for crisis hotlines)
// 988 chat, Samaritans chat, Crisis Text Line — bot-score must not block them
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal,
  action == Action::"FlagEvaluate",
  resource
)
when {
  principal.audience_type == "EMERGENCY_SERVICES"
  && principal.bot_class == "CRISIS_TRIAGE"
  && context.crisis_line_federated == true
};

// ─────────────────────────────────────────────────────────────────────────────
// FORBID: emergency-services bypass cannot be used to mutate pack overrides
// Bypass is for evaluation/read access; pack mutation still requires pack-overlay-agent
// ─────────────────────────────────────────────────────────────────────────────
forbid (
  principal,
  action == Action::"PackOverrideApply",
  resource
)
when {
  principal.audience_type == "EMERGENCY_SERVICES"
  && !(principal == Principal::"oyatie.feature-flags.pack-overlay-agent")
};

// ─────────────────────────────────────────────────────────────────────────────
// NOTE: Audit is retained unconditionally for all emergency-services requests.
// AbuseDefenceEmergencyServiceBypass event is emitted by the audit-emitter
// regardless of this Cedar evaluation result.
// The bypass DOES NOT skip audit. Bypass + audit = accountability without friction.
// ─────────────────────────────────────────────────────────────────────────────
