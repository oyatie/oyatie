// Cedar v4.2 LTS fragment — ci-scope
// Microservice: feature-flags
// Binding ADR: ADR-0243 + ADR-0110 (changeset state machine) + ADR-0139 (SLO-gated promotion)
// Fragment soak window: ≥60s per ADR-0294
// Fragment version: 1.0.0
// Reviewed: 2026-05-20
//
// Governs what CI/CD and Foundry principals may read/write during
// automated pipeline operations (lint, test, flag-lifecycle gate, promotion).

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: Foundry CI principal may read flag definitions (for lifecycle gate checks)
// E.g., checking that release_toggle flags have sunset_at set before CI green
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal in Role::"foundry-ci",
  action == Action::"FlagRead",
  resource is FlagDefinition
)
when {
  context.foundry_attestation_present == true
};

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: Foundry CI may read SLO evaluation state (for rollout gate decisions)
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal in Role::"foundry-ci",
  action == Action::"SLOStateRead",
  resource is SLOEvaluationResult
)
when {
  context.foundry_attestation_present == true
};

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: Foundry CI may advance rollout stages (automated progression)
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal in Role::"foundry-ci",
  action == Action::"RolloutStageAdvance",
  resource is RolloutPlan
)
when {
  context.foundry_attestation_present == true
  && resource.auto_advance_enabled == true
  && context.slo_gate_passed == true
};

// ─────────────────────────────────────────────────────────────────────────────
// PERMIT: Foundry CI may trigger automated rollback on SLO breach
// ─────────────────────────────────────────────────────────────────────────────
permit (
  principal in Role::"foundry-ci",
  action == Action::"RolloutRollback",
  resource is RolloutPlan
)
when {
  context.foundry_attestation_present == true
  && context.slo_breach_detected == true
};

// ─────────────────────────────────────────────────────────────────────────────
// FORBID: CI principal cannot mutate flag definitions directly
// Flag mutations require human step-up auth; CI is read-only on definitions
// ─────────────────────────────────────────────────────────────────────────────
forbid (
  principal in Role::"foundry-ci",
  action in [Action::"FlagCreate", Action::"FlagUpdate", Action::"FlagArchive", Action::"FlagDelete"],
  resource
);

// ─────────────────────────────────────────────────────────────────────────────
// FORBID: CI principal cannot engage kill-switches
// Kill-switch requires human on-call; CI automation must not auto-kill
// ─────────────────────────────────────────────────────────────────────────────
forbid (
  principal in Role::"foundry-ci",
  action == Action::"KillSwitchEngage",
  resource
);
