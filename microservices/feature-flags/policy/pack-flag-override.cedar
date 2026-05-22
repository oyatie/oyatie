// Cedar v4.2 LTS fragment — pack-flag-override
// Microservice: feature-flags
// Binding ADR: ADR-0243 + ADR-0251 (compliance packs) + ADR-0159
// Fragment soak window: ≥60s per ADR-0294
// Fragment version: 1.0.0
// Reviewed: 2026-05-20

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: pack-overlay-agent may apply pack-mandated overrides
// This is the ONLY principal that may write pack_overrides on FlagDefinition
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal == Principal::"oyatie.feature-flags.pack-overlay-agent",
  action == Action::"PackOverrideApply",
  resource is FlagDefinition
)
when {
  context.pack_id in resource.tenant.active_packs
  && context.override_value == context.pack_mandated_value
};

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: compliance-officer may READ (not write) pack overrides
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal in Role::"compliance-officer",
  action == Action::"PackOverrideRead",
  resource is FlagDefinition
)
when {
  principal.tenant_id == resource.tenant_id
};

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: pack-overlay-agent may subscribe to pack-activation events
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal == Principal::"oyatie.feature-flags.pack-overlay-agent",
  action == Action::"PackOverrideSubscribe",
  resource is PackActivationEvent
);

// ─────────────────────────────────────────────────────────────────────────────
// FORBID: no principal except pack-overlay-agent may write pack_overrides
// Tenant admins cannot override pack-mandated values
// ─────────────────────────────────────────────────────────────────────────────
forbid (
  principal,
  action == Action::"PackOverrideApply",
  resource is FlagDefinition
)
unless {
  principal == Principal::"oyatie.feature-flags.pack-overlay-agent"
};

// ─────────────────────────────────────────────────────────────────────────────
// FORBID: FlagUpdate that would change a pack-overridden field
// Ensures tenant flag mutations cannot bypass pack-mandated values
// ─────────────────────────────────────────────────────────────────────────────
forbid (
  principal,
  action == Action::"FlagUpdate",
  resource is FlagDefinition
)
when {
  context.mutation_field in resource.pack_override_locked_fields
  && !(principal == Principal::"oyatie.feature-flags.pack-overlay-agent")
};

// ─────────────────────────────────────────────────────────────────────────────
// FORBID: pack override on emergency-services-class flags that would disable bypass
// Anti-pattern: a HIPAA pack override must not accidentally disable the
// emergency-services bypass (which is a life-safety invariant)
// ─────────────────────────────────────────────────────────────────────────────
forbid (
  principal,
  action == Action::"PackOverrideApply",
  resource is FlagDefinition
)
when {
  resource.flag_key in [
    "emergency-services-bypass-flag",
    "healthcare-break-glass-enable",
    "emergency-dispatch-routing-override"
  ]
  && context.override_value == "off"
};
