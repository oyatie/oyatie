// Cedar v4.2 LTS fragment — flag-mutation-authorization
// Microservice: feature-flags
// Binding ADR: ADR-0243 (Cedar universal gate) + ADR-0159 (feature-flag substrate)
// Fragment soak window: ≥60s per ADR-0294 before activation in production
// Fragment version: 1.0.0
// Author: oyatie.feature-flags.flag-manager
// Reviewed: 2026-05-20

// ─────────────────────────────────────────────────────────────────────────────
// DEFAULT DENY — all flag mutations are forbidden unless explicitly permitted
// ─────────────────────────────────────────────────────────────────────────────
// No permit below = access denied. This is the Cedar default; stated explicitly
// for documentation-rigor.md §1 compliance (intern-buildability bar).

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: flag-manager role may create flags within their tenant
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal in Role::"flag-manager",
  action == Action::"FlagCreate",
  resource is FlagDefinition
)
when {
  principal.tenant_id == resource.tenant_id
  && principal.mfa_verified == true
  && context.step_up_class >= 1    // Class A minimum (passkey)
};

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: flag-manager may update live (non-draft) flags — step-up Class B
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal in Role::"flag-manager",
  action == Action::"FlagUpdate",
  resource is FlagDefinition
)
when {
  principal.tenant_id == resource.tenant_id
  && principal.mfa_verified == true
  && context.step_up_class >= 2    // Class B: passkey re-challenge
  && resource.lifecycle_state != "archived"
  && resource.lifecycle_state != "deleted"
};

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: flag-manager may archive flags (soft delete)
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal in Role::"flag-manager",
  action == Action::"FlagArchive",
  resource is FlagDefinition
)
when {
  principal.tenant_id == resource.tenant_id
  && principal.mfa_verified == true
  && context.step_up_class >= 2    // Class B
};

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: flag-manager may hard-delete archived flags — step-up Class C
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal in Role::"flag-manager",
  action == Action::"FlagDelete",
  resource is FlagDefinition
)
when {
  principal.tenant_id == resource.tenant_id
  && principal.mfa_verified == true
  && context.step_up_class >= 3    // Class C: TOTP + passkey
  && resource.lifecycle_state == "archived"
};

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: flag-manager may undo mutations within 15s undo window
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal in Role::"flag-manager",
  action == Action::"UndoFlagMutation",
  resource is FlagDefinition
)
when {
  principal.tenant_id == resource.tenant_id
  && principal.mfa_verified == true
  && context.seconds_since_last_mutation <= 15
  && resource.undo_window_open == true
};

// ─────────────────────────────────────────────────────────────────────────────
// FORBID: cross-tenant mutations are never permitted
// ─────────────────────────────────────────────────────────────────────────────
forbid (
  principal,
  action in [Action::"FlagCreate", Action::"FlagUpdate", Action::"FlagArchive", Action::"FlagDelete"],
  resource is FlagDefinition
)
unless {
  principal.tenant_id == resource.tenant_id
};

// ─────────────────────────────────────────────────────────────────────────────
// FORBID: mutations on pack-mandated override fields are forbidden for non-pack-agent
// ─────────────────────────────────────────────────────────────────────────────
forbid (
  principal,
  action in [Action::"FlagUpdate", Action::"FlagCreate"],
  resource is FlagDefinition
)
when {
  resource.has_pack_override == true
  && !(principal == Principal::"oyatie.feature-flags.pack-overlay-agent")
};

// ─────────────────────────────────────────────────────────────────────────────
// FORBID: mutations blocked when step-up auth is insufficient
// ─────────────────────────────────────────────────────────────────────────────
forbid (
  principal,
  action is Action,
  resource is FlagDefinition
)
when {
  action in [Action::"FlagDelete", Action::"FlagUpdate"]
  && context.step_up_class < resource.mutation_step_up_class
};

// ─────────────────────────────────────────────────────────────────────────────
// FORBID: rate limit on mutations (>60/min per principal)
// Anti-flag-flooding per §3.2.3 abuse-defence
// ─────────────────────────────────────────────────────────────────────────────
forbid (
  principal,
  action in [Action::"FlagCreate", Action::"FlagUpdate", Action::"FlagArchive", Action::"FlagDelete"],
  resource
)
when {
  principal.flag_mutation_rate_per_minute > 60
  && !(principal == Principal::"oyatie.feature-flags.pack-overlay-agent")
};
