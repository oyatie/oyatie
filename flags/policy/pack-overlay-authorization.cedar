// Cedar v4.2 LTS fragment — pack-overlay-authorization
// Microservice: feature-flags
// Binding ADR: ADR-0243 + ADR-0251 + ADR-0247 (self-modification doctrine)
// Fragment soak window: ≥60s per ADR-0294
// Fragment version: 1.0.0
// Reviewed: 2026-05-20
//
// This fragment governs which principals may SUBSCRIBE to and ACT ON
// pack-overlay events. Separate from pack-flag-override.cedar which governs
// the write side of overrides.

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: pack-overlay-agent principal may subscribe to pack-activation events
// and apply pack-mandated flag overrides across tenants in that pack
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal == Principal::"oyatie.feature-flags.pack-overlay-agent",
  action in [
    Action::"PackOverrideSubscribe",
    Action::"PackOverrideApply",
    Action::"PackOverrideRead",
    Action::"PackOverrideAudit"
  ],
  resource
)
when {
  // Foundry meta-trust root attestation must be present (ADR-0247 + ADR-0293)
  context.foundry_attestation_present == true
  && context.cosign_signature_valid == true
};

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: audit-emitter may read pack overrides to include in audit events
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal == Principal::"oyatie.feature-flags.audit-emitter",
  action == Action::"PackOverrideRead",
  resource is FlagDefinition
);

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: platform-safety-officer may read all pack overrides (cross-tenant audit)
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal in Role::"platform-safety-officer",
  action in [Action::"PackOverrideRead", Action::"PackOverrideAudit"],
  resource
);

// ─────────────────────────────────────────────────────────────────────────────
// FORBID: any principal without Foundry attestation may not apply pack overrides
// Prevents rogue pack-overlay attempts
// ─────────────────────────────────────────────────────────────────────────────
forbid (
  principal,
  action == Action::"PackOverrideApply",
  resource
)
unless {
  context.foundry_attestation_present == true
};
