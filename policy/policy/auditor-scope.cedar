// Cedar 4.12 fragment — auditor-scope
// Capability: policy (ADR-0615 §5, 24th registered capability)
// Binding ADRs: ADR-0615 §5, ADR-0702 (identity-authz apex)
// Fragment soak window: >= 60s per ADR-0294
// Fragment version: 1.0.0
// Reviewed: 2026-08-20
//
// The audit read path. Auditors read decisions; they never author, sign, publish or activate —
// that separation is what makes the decision log admissible evidence.

// P1 — an auditor reads the decision log of their own tenant on a fresh token.
permit (
  principal,
  action == OyaPolicy::Action::"ReadDecisionLog",
  resource
)
when {
  principal.role == "auditor" &&
  principal.tenant_id == resource.tenant_id &&
  context.token_age_seconds <= 900
};

// F9 — an expired token reads nothing, whatever the role.
forbid (
  principal,
  action == OyaPolicy::Action::"ReadDecisionLog",
  resource
)
when { context.token_age_seconds > 900 };
