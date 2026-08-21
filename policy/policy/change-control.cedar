// Cedar 4.12 fragment — change-control
// Capability: policy (ADR-0615 §5, 24th registered capability)
// Binding ADRs: ADR-0294 (soak window), ADR-0615 §5
// Fragment soak window: >= 60s per ADR-0294
// Fragment version: 1.0.0
// Reviewed: 2026-08-20
//
// Separation of duties and the soak window across the authoring lifecycle
// (Authored -> Reviewed -> Signed -> Published -> Soaking -> Activated -> Audited, ADR-0294).

// F5 — the author of a policy version may not also sign it. Two humans, or a human and an attested
// service, must touch every change that widens authorization.
forbid (
  principal,
  action == OyaPolicy::Action::"SignPolicy",
  resource
)
when { resource.author_principal_id == principal.principal_id };

// F6 — nothing unsigned is ever published or activated.
forbid (
  principal,
  action in [
    OyaPolicy::Action::"PublishPolicy",
    OyaPolicy::Action::"ActivatePolicy"
  ],
  resource
)
unless { resource.signed };

// F7 — the ADR-0294 soak window is mandatory. Activation before the fragment has soaked is refused
// regardless of role; there is no break-glass here, because the soak is what makes rollback cheap.
forbid (
  principal,
  action == OyaPolicy::Action::"ActivatePolicy",
  resource
)
when { context.soak_elapsed_seconds < 60 };

// F8 — least privilege on blast radius. Global-scope policy is the whole fleet; only a
// platform-policy-engineer with a step-up credential may move it.
forbid (
  principal,
  action in [
    OyaPolicy::Action::"AuthorPolicy",
    OyaPolicy::Action::"PublishPolicy",
    OyaPolicy::Action::"ActivatePolicy"
  ],
  resource
)
when { resource.scope == "global" }
unless {
  principal.role == "platform-policy-engineer" && principal.step_up_class == "C"
};
