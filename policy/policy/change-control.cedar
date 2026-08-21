// Cedar 4.12 fragment — change-control
// Capability: policy (ADR-0615 §5, 24th registered capability)
// Binding ADRs: ADR-0294 (soak window), ADR-0615 §5
// Fragment soak window: >= 60s per ADR-0294
// Fragment version: 2.0.0
// Reviewed: 2026-08-20 (adversarial review; v1.0.0 findings S1-S6 closed)
//
// Separation of duties and the soak window across the authoring lifecycle
// (Authored -> Reviewed -> Signed -> Published -> Soaking -> Activated -> Audited, ADR-0294).

// F5 — the author of a policy version may not also sign it.
//
// Entity equality, not a string compare. v1.0.0 compared `resource.author_principal_id ==
// principal.principal_id`, which a second registered identifier for the same actor defeated: one
// human with two id strings satisfied two-party control.
@id("F5")
forbid (
  principal,
  action == OyaPolicy::Action::"SignPolicy",
  resource
)
when { resource.author == principal };

// F5b — nor may a DIFFERENT Principal sharing the author's workload identity sign it.
// Scoped with `resource.author != principal` so it is disjoint from F5: F5 owns the same-entity
// case, F5b the distinct-entity-same-identity case. Before this scoping F5b subsumed F5 entirely
// (a principal always shares its own spiffe_id), which made F5 untestable — no mutation could
// reach it. Two rules that cannot be told apart by any test are one rule with a spare. Entity equality
// alone does not stop one actor registering two Principal entities; the review demonstrated exactly
// that, using two principals with the SAME spiffe_id. Two-party control has to be two parties, so
// the attested identity must differ too.
@id("F5b")
forbid (
  principal,
  action == OyaPolicy::Action::"SignPolicy",
  resource
)
when {
  resource.author != principal &&
  resource.author.spiffe_id == principal.spiffe_id
};

// F6 — nothing unsigned is ever published or activated.
@id("F6")
forbid (
  principal,
  action in [
    OyaPolicy::Action::"PublishPolicy",
    OyaPolicy::Action::"ActivatePolicy"
  ],
  resource
)
unless { resource.signed };

// F7 — the ADR-0294 soak window is mandatory. No break-glass: the soak is what makes rollback cheap.
@id("F7")
forbid (
  principal,
  action == OyaPolicy::Action::"ActivatePolicy",
  resource
)
unless {
  context has soak_elapsed_seconds && context.soak_elapsed_seconds >= 60
};

// F8 — least privilege on blast radius. Global scope is the whole fleet.
//
// `SignPolicy` is in this list. v1.0.0 omitted it, so a signer with no step-up could sign a
// global-scope policy — and the signature is exactly what F6 turns into the publish/activate
// precondition, i.e. the ungated step was the one that confers trust.
@id("F8")
forbid (
  principal,
  action in [
    OyaPolicy::Action::"AuthorPolicy",
    OyaPolicy::Action::"SignPolicy",
    OyaPolicy::Action::"PublishPolicy",
    OyaPolicy::Action::"ActivatePolicy"
  ],
  resource
)
when { resource.scope == "global" }
unless {
  principal in OyaPolicy::Role::"platform-policy-engineer" &&
  principal.step_up_class == "C"
};

// F11 — scope is a closed set. An unrecognised value (including a case variant such as "GLOBAL")
// is refused positively rather than relying on no permit happening to match it.
@id("F11")
forbid (
  principal,
  action in [
    OyaPolicy::Action::"AuthorPolicy",
    OyaPolicy::Action::"SignPolicy",
    OyaPolicy::Action::"PublishPolicy",
    OyaPolicy::Action::"ActivatePolicy"
  ],
  resource
)
unless { resource.scope == "tenant" || resource.scope == "global" };

// F12 — ReBAC tuple writes are authorization state with no signature, no version and no soak to
// hang change control on, so they carry a step-up requirement instead. v1.0.0 omitted
// `WriteRebacTuple` from every control above, leaving the path that grants relations directly as
// the least-governed surface in the capability.
@id("F12")
forbid (
  principal,
  action == OyaPolicy::Action::"WriteRebacTuple",
  resource
)
unless {
  principal.step_up_class == "B" || principal.step_up_class == "C"
};
