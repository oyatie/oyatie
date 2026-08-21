// Cedar 4.12 fragment — auditor-scope
// Capability: policy (ADR-0615 §5, 24th registered capability)
// Binding ADRs: ADR-0615 §5, ADR-0702 (identity-authz apex)
// Fragment soak window: >= 60s per ADR-0294
// Fragment version: 2.0.0
// Reviewed: 2026-08-20 (adversarial review; v1.0.0 findings S1-S6 closed)
//
// The audit read path. Auditors read decisions; they never author, sign, publish or activate —
// that separation is what makes the decision log admissible evidence.
//
// Credential freshness is enforced fleet-wide by F10 in credential-freshness.cedar and mirrored
// into P1 below. v1.0.0 carried a dedicated `ReadDecisionLog` forbid; that rule is now the general
// case, so the special case is gone rather than duplicated.

// P1 — an auditor reads the decision log of their own tenant on a fresh token.
@id("P1")
permit (
  principal,
  action == OyaPolicy::Action::"ReadDecisionLog",
  resource
)
when {
  principal in OyaPolicy::Role::"auditor" &&
  principal in resource.owner_tenant &&
  context.token_age_seconds <= 900
};
