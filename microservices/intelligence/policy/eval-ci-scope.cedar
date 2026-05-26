// foundry-eval µservice — CI-scope policy fragment
// Cedar v4 schema; conforms to microservices/intelligence/policy/schema.cedarschema
//
// Purpose: enforce GitHub Actions CI runner scope: read-only on publish-gate
//          verdict; trigger nightly cadence; trigger A/B verdict on routing-PR;
//          NEVER write verdicts directly (worker-only via SPIFFE).
// Owner:   ops-security + axis-foundry
// Audit:   every deny emits oya_foundry_eval_ci_unauthorized_attempt_total
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny.
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT — GitHub Actions OIDC token scoped to oyatie-ci
// =============================================================================

permit (
  principal == ServiceIdentity::"spiffe://github-oidc/oyatie-ci",
  action in [
    Action::"read_publish_gate_verdict",
    Action::"trigger_publish_gate_run",
    Action::"trigger_ab_verdict_run",
    Action::"read_eval_run_latest"
  ],
  resource
)
when {
  context has oidc_audience &&
  context.oidc_audience == "oyatie-ci" &&
  context has token_ttl_seconds &&
  context.token_ttl_seconds <= 900  // 15 min max
};

// =============================================================================
// FORBID — CI runner cannot write verdicts directly
// =============================================================================

forbid (
  principal == ServiceIdentity::"spiffe://github-oidc/oyatie-ci",
  action in [
    Action::"write_eval_run_completed",
    Action::"write_parity_verdict",
    Action::"write_replay_divergence",
    Action::"write_in_house_cutover_eligible"
  ],
  resource
);

// =============================================================================
// FORBID — CI runner cannot bypass adversarial cohort gate
// =============================================================================

forbid (
  principal == ServiceIdentity::"spiffe://github-oidc/oyatie-ci",
  action == Action::"override_adversarial_cohort_failure",
  resource
);
