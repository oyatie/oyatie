// Cedar 4.12 fragment — authoring-grants
// Capability: policy (ADR-0615 §5, 24th registered capability)
// Binding ADRs: ADR-0615 §5 (G face), ADR-0280 §D-13.D
// Fragment soak window: >= 60s per ADR-0294
// Fragment version: 1.0.0
// Reviewed: 2026-08-20
//
// The G-face permits. Every one binds the tenant explicitly; cross-tenant-isolation.cedar is the
// backstop for the day one of them forgets.

// P2 — a tenant-policy-admin authors and publishes tenant-scoped policy in their own tenant.
permit (
  principal,
  action in [
    OyaPolicy::Action::"AuthorPolicy",
    OyaPolicy::Action::"PublishPolicy",
    OyaPolicy::Action::"ActivatePolicy"
  ],
  resource
)
when {
  principal.role == "tenant-policy-admin" &&
  principal.tenant_id == resource.tenant_id &&
  resource.scope == "tenant"
};

// P3 — a platform-policy-engineer at step-up C authors and publishes at any scope. F8 still binds
// global scope to exactly this shape; this permit is the other half of that pair.
permit (
  principal,
  action in [
    OyaPolicy::Action::"AuthorPolicy",
    OyaPolicy::Action::"PublishPolicy",
    OyaPolicy::Action::"ActivatePolicy"
  ],
  resource
)
when {
  principal.role == "platform-policy-engineer" &&
  principal.step_up_class == "C" &&
  principal.tenant_id == resource.tenant_id
};

// P4 — a dedicated signer role signs. F5 stops it signing its own authorship.
permit (
  principal,
  action == OyaPolicy::Action::"SignPolicy",
  resource
)
when {
  principal.role == "policy-signer" &&
  principal.tenant_id == resource.tenant_id
};

// P5 — a tenant-policy-admin writes ReBAC tuples in their own tenant.
permit (
  principal,
  action == OyaPolicy::Action::"WriteRebacTuple",
  resource
)
when {
  principal.role == "tenant-policy-admin" &&
  principal.tenant_id == resource.tenant_id
};
