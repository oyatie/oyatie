// Cedar 4.12 fragment — authoring-grants
// Capability: policy (ADR-0615 §5, 24th registered capability)
// Binding ADRs: ADR-0615 §5 (G face), ADR-0280 §D-13.D
// Fragment soak window: >= 60s per ADR-0294
// Fragment version: 2.0.0
// Reviewed: 2026-08-20 (adversarial review; v1.0.0 findings S1-S6 closed)
//
// The G-face permits. Every one binds the tenant by MEMBERSHIP and repeats every bound its matching
// forbid enforces. The repetition is not redundancy — see policy/cedar/README.md.

// P2 — a tenant-policy-admin authors and publishes tenant-scoped policy in their own tenant.
permit (
  principal,
  action in [
    OyaPolicy::Action::"AuthorPolicy",
    OyaPolicy::Action::"PublishPolicy"
  ],
  resource
)
when {
  principal in OyaPolicy::Role::"tenant-policy-admin" &&
  principal in resource.owner_tenant &&
  resource.scope == "tenant" &&
  context.token_age_seconds <= 900
};

// P2a — activation, carrying the soak bound F7 enforces.
permit (
  principal,
  action == OyaPolicy::Action::"ActivatePolicy",
  resource
)
when {
  principal in OyaPolicy::Role::"tenant-policy-admin" &&
  principal in resource.owner_tenant &&
  resource.scope == "tenant" &&
  resource.signed &&
  context.token_age_seconds <= 900 &&
  context.soak_elapsed_seconds >= 60
};

// P3 — a platform-policy-engineer at step-up C authors and publishes at any scope.
permit (
  principal,
  action in [
    OyaPolicy::Action::"AuthorPolicy",
    OyaPolicy::Action::"PublishPolicy"
  ],
  resource
)
when {
  principal in OyaPolicy::Role::"platform-policy-engineer" &&
  principal.step_up_class == "C" &&
  principal in resource.owner_tenant &&
  context.token_age_seconds <= 900
};

// P3a — the same, for activation, carrying the soak bound.
permit (
  principal,
  action == OyaPolicy::Action::"ActivatePolicy",
  resource
)
when {
  principal in OyaPolicy::Role::"platform-policy-engineer" &&
  principal.step_up_class == "C" &&
  principal in resource.owner_tenant &&
  resource.signed &&
  context.token_age_seconds <= 900 &&
  context.soak_elapsed_seconds >= 60
};

// P4 — a dedicated signer signs. F5 stops it signing its own authorship; F8 stops it signing
// global scope without step-up C.
permit (
  principal,
  action == OyaPolicy::Action::"SignPolicy",
  resource
)
when {
  principal in OyaPolicy::Role::"policy-signer" &&
  principal in resource.owner_tenant &&
  context.token_age_seconds <= 900
};

// P5 — a tenant-policy-admin writes ReBAC tuples in their own tenant, at step-up B or better.
permit (
  principal,
  action == OyaPolicy::Action::"WriteRebacTuple",
  resource
)
when {
  principal in OyaPolicy::Role::"tenant-policy-admin" &&
  principal in resource.owner_tenant &&
  (principal.step_up_class == "B" || principal.step_up_class == "C") &&
  context.token_age_seconds <= 900
};
