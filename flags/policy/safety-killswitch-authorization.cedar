// Cedar v4.2 LTS fragment — safety-killswitch-authorization
// Microservice: feature-flags
// Binding ADR: ADR-0243 + ADR-0159 + ADR-0298 (emergency-services bypass)
// Fragment soak window: ≥60s per ADR-0294
// Fragment version: 1.0.0
// LIFE-SAFETY: This fragment MUST NOT be modified without dual-approval from
//              axis-platform-oncall and axis-governance-security.
// Reviewed: 2026-05-20

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: sre-oncall may engage kill-switches — step-up Class C
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal in Role::"sre-oncall",
  action == Action::"KillSwitchEngage",
  resource is FlagDefinition
)
when {
  principal.mfa_verified == true
  && context.step_up_class >= 3    // Class C: TOTP + passkey (highest standard)
  && resource.flag_type == "kill_switch"
};

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: killswitch-operator (dedicated role) may engage kill-switches
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal in Role::"killswitch-operator",
  action == Action::"KillSwitchEngage",
  resource is FlagDefinition
)
when {
  principal.mfa_verified == true
  && context.step_up_class >= 3
  && resource.flag_type == "kill_switch"
};

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: sre-oncall may disengage kill-switches — step-up Class B
// Lower bar than engagement (re-enabling is recoverable; disabling is safer)
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal in Role::"sre-oncall",
  action == Action::"KillSwitchDisengage",
  resource is FlagDefinition
)
when {
  principal.mfa_verified == true
  && context.step_up_class >= 2    // Class B
  && resource.flag_type == "kill_switch"
  && resource.lifecycle_state == "kill_switch_active"
};

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: pack-overlay-agent may engage/disengage pack-mandated kill-switches
// (e.g., HIPAA pack forces phi-exposure-flag to kill_switch_active)
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal == Principal::"oyatie.feature-flags.pack-overlay-agent",
  action in [Action::"KillSwitchEngage", Action::"KillSwitchDisengage"],
  resource is FlagDefinition
)
when {
  resource.has_pack_override == true
};

// ─────────────────────────────────────────────────────────────────────────────
// FORBID: kill-switch on emergency-services-bypass flags is absolutely prohibited
// LIFE-SAFETY HARD RULE: this FORBID cannot be overridden by any permit
// Per §3.2.3 documentation-rigor.md and ADR-0298
// ─────────────────────────────────────────────────────────────────────────────
forbid (
  principal,
  action == Action::"KillSwitchEngage",
  resource is FlagDefinition
)
when {
  resource.flag_key in [
    "emergency-services-bypass-flag",
    "healthcare-break-glass-enable",
    "emergency-dispatch-routing-override",
    "crisis-line-bypass-flag",
    "ehr-break-glass-enable"
  ]
};

// ─────────────────────────────────────────────────────────────────────────────
// FORBID: only sre-oncall and killswitch-operator may engage kill-switches
// Default-deny: product managers, engineers, and other roles cannot kill-switch
// ─────────────────────────────────────────────────────────────────────────────
forbid (
  principal,
  action == Action::"KillSwitchEngage",
  resource
)
unless {
  principal in Role::"sre-oncall"
  || principal in Role::"killswitch-operator"
  || principal == Principal::"oyatie.feature-flags.pack-overlay-agent"
};

// ─────────────────────────────────────────────────────────────────────────────
// FORBID: cross-tenant kill-switch operations
// ─────────────────────────────────────────────────────────────────────────────
forbid (
  principal,
  action in [Action::"KillSwitchEngage", Action::"KillSwitchDisengage"],
  resource is FlagDefinition
)
unless {
  principal.tenant_id == resource.tenant_id
  || principal == Principal::"oyatie.feature-flags.pack-overlay-agent"
};
