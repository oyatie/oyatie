// Cedar 4.12 fragment — credential-freshness
// Capability: policy (ADR-0615 §5, 24th registered capability)
// Binding ADRs: ADR-0702 (identity-authz apex), ADR-0615 §5
// Fragment soak window: >= 60s per ADR-0294
// Fragment version: 2.0.0
// Reviewed: 2026-08-20 (adversarial review; v1.0.0 findings S1-S6 closed)
//
// F10 — a stale credential authorizes nothing, on ANY action.
//
// v1.0.0 bounded token age only on `ReadDecisionLog`, so the read-only audit path was gated harder
// than every mutating path: author, sign, publish, activate, write-tuple and distribute all accepted
// a credential of any age.
//
// Every permit in this bundle ALSO carries `context.token_age_seconds <= 900`. That duplication is
// deliberate and load-bearing — see policy/cedar/README.md "Why every bound appears twice".
// `context has token_age_seconds &&` is what makes this FAIL CLOSED. Without the `has` guard a
// request that simply omits the attribute makes this policy ERROR, Cedar drops the erroring policy,
// and dropping a FORBID leaves the permit standing — the guard deletes itself exactly when it is
// needed. That was finding S1 of the v1.0.0 review, and `has` is the structural fix; mirroring the
// bound into every permit is only the second line.
forbid (
  principal,
  action,
  resource
)
unless {
  context has token_age_seconds && context.token_age_seconds <= 900
};
