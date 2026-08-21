// Consolidated Cedar bundle — policy capability control surface
// Binding ADRs: see each fragment header; live apexes ADR-0701 / ADR-0702 (archived provenance
// ADR-0615 §5, ADR-0280 §D-13.D/E, ADR-0243, ADR-0294).
// Cedar 4.12 (workspace lock). Bundle version 2.0.0.
//
// This file is the concatenation of policy/policy/*.cedar in lexical order, PLUS this header and a
// one-line separator naming each fragment. It is NOT byte-identical to `cat policy/policy/*.cedar`
// — those comment lines are the difference. What IS asserted, and tested, is that it decides
// identically: the whole conformance suite runs against THIS file. Comment-stripped, the two are
// byte-identical. Fragments stay closed under concatenation only while none carries a bare
// `forbid (principal, action, resource);` — see policy/cedar/README.md.

// ─── policy/policy/auditor-scope.cedar ───
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

// ─── policy/policy/authoring-grants.cedar ───
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
@id("P2")
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
@id("P2a")
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
@id("P3")
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
@id("P3a")
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
@id("P4")
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
@id("P5")
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

// ─── policy/policy/change-control.cedar ───
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

// ─── policy/policy/credential-freshness.cedar ───
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
@id("F10")
forbid (
  principal,
  action,
  resource
)
unless {
  context has token_age_seconds && context.token_age_seconds <= 900
};

// ─── policy/policy/cross-tenant-isolation.cedar ───
// Cedar 4.12 fragment — cross-tenant-isolation
// Capability: policy (ADR-0615 §5, 24th registered capability)
// Binding ADRs: ADR-0280 §D-13.D, ADR-0615 §5
// Fragment soak window: >= 60s per ADR-0294
// Fragment version: 2.0.0
// Reviewed: 2026-08-20 (adversarial review; v1.0.0 findings S1-S6 closed)
//
// F1 — a principal may never act on another tenant's authorization state.
//
// This is MEMBERSHIP (`principal in resource.owner_tenant`), not a string comparison. v1.0.0 wrote
// `principal.tenant_id != resource.tenant_id`, which an adversarial review defeated in one step: a
// principal whose entity parent was tenant A but whose `tenant_id` attribute said B was authorized
// against B, and this forbid — reading the same caller-supplied field as the permit it backstops —
// did not fire. A backstop derived from the same untrusted input as the thing it guards is not a
// backstop. `owner_tenant` is an entity reference and membership is structural.
//
// Note it is `unless`, not `when`: a request whose principal is in NO tenant is forbidden, rather
// than slipping through a `!=` that never becomes true.
@id("F1")
forbid (
  principal,
  action,
  resource
)
unless { principal in resource.owner_tenant };

// ─── policy/policy/runtime-attestation.cedar ───
// Cedar 4.12 fragment — runtime-attestation
// Capability: policy (ADR-0615 §5, 24th registered capability)
// Binding ADRs: ADR-0280 §D-13.D (cell-distributed PDP), ADR-0702 (identity-authz apex)
// Fragment soak window: >= 60s per ADR-0294
// Fragment version: 2.0.0
// Reviewed: 2026-08-20 (adversarial review; v1.0.0 findings S1-S6 closed)
//
// Two bounds that v1.0.0 carried only inside permits, and therefore lost the moment any future
// permit forgot them. A bound worth having is worth a forbid.

// F13 — a PEP evaluates only its OWN cell's snapshot. Cell-distribution is not a real property
// until something refuses the cross-cell case.
@id("F13")
forbid (
  principal,
  action == OyaPolicy::Action::"EvaluateAgainstSnapshot",
  resource
)
unless { principal in resource.cell };

// F14 — the runtime and distribution paths require a well-formed attested workload identity.
// v1.0.0 tested `spiffe_id != ""`, which a single space satisfied.
@id("F14")
forbid (
  principal,
  action in [
    OyaPolicy::Action::"EvaluateAgainstSnapshot",
    OyaPolicy::Action::"ReadRebacTuple",
    OyaPolicy::Action::"DistributeSnapshot"
  ],
  resource
)
unless { principal.spiffe_id like "spiffe://oyatie/*" };

// ─── policy/policy/runtime-grants.cedar ───
// Cedar 4.12 fragment — runtime-grants
// Capability: policy (ADR-0615 §5, 24th registered capability)
// Binding ADRs: ADR-0280 §D-13.D/E (C0 face), ADR-0191 (edge vs origin)
// Fragment soak window: >= 60s per ADR-0294
// Fragment version: 2.0.0
// Reviewed: 2026-08-20 (adversarial review; v1.0.0 findings S1-S6 closed)
//
// The C0-face permits: the hot paths, and the ones static-stability.cedar constrains. Each repeats
// the freshness and signature bounds its forbid enforces, so that an absent attribute denies
// instead of deleting the guard and leaving the permit standing.

// P6 — an attested PEP workload evaluates against its OWN CELL's snapshot.
//
// `principal in resource.cell` is the cell binding. v1.0.0 declared `cell_id` on both Tenant and
// Snapshot and then never referenced it in any policy, so the cell-distributed model (ADR-0280
// §D-13.D) had no enforcement at all — a PEP in cell c1 could evaluate cell c99's snapshot.
@id("P6")
permit (
  principal,
  action == OyaPolicy::Action::"EvaluateAgainstSnapshot",
  resource
)
when {
  principal in OyaPolicy::Role::"pep-workload" &&
  principal in resource.owner_tenant &&
  principal in resource.cell &&
  principal.spiffe_id like "spiffe://oyatie/*" &&
  resource.signature_verified &&
  resource.age_seconds <= context.max_staleness_seconds &&
  (resource.age_seconds <= 300 || resource.is_authoritative) &&
  context.token_age_seconds <= 900
};

// P7 — an attested PEP workload reads ReBAC tuples for its own tenant, from a verified and
// sufficiently fresh snapshot.
@id("P7")
permit (
  principal,
  action == OyaPolicy::Action::"ReadRebacTuple",
  resource
)
when {
  principal in OyaPolicy::Role::"pep-workload" &&
  principal in resource.owner_tenant &&
  principal.spiffe_id like "spiffe://oyatie/*" &&
  resource.served_from.signature_verified &&
  (resource.served_from.age_seconds <= 300 || resource.served_from.is_authoritative) &&
  context.token_age_seconds <= 900
};

// P8 — the distributor propagates signed, sufficiently fresh snapshots.
//
// Attestation is required here too. v1.0.0 demanded a SPIFFE id for the local read paths but not
// for distribution, which has the higher blast radius — the asymmetry was backwards.
@id("P8")
permit (
  principal,
  action == OyaPolicy::Action::"DistributeSnapshot",
  resource
)
when {
  principal in OyaPolicy::Role::"policy-distributor" &&
  principal in resource.owner_tenant &&
  principal.spiffe_id like "spiffe://oyatie/*" &&
  resource.signature_verified &&
  (resource.age_seconds <= 300 || resource.is_authoritative) &&
  context.token_age_seconds <= 900
};

// ─── policy/policy/static-stability.cedar ───
// Cedar 4.12 fragment — static-stability
// Capability: policy (ADR-0615 §5, 24th registered capability)
// Binding ADRs: ADR-0280 §D-13.E (static-stability invariant)
// Fragment soak window: >= 60s per ADR-0294
// Fragment version: 2.0.0
// Reviewed: 2026-08-20 (adversarial review; v1.0.0 findings S1-S6 closed)
//
// The invariant: "a stale snapshot must DENY or route to the authoritative shard — never silently
// authorize." These forbids are the DENY half. The route half is a runtime concern specified in
// policy/CONTRACT.md, because Cedar answers allow/deny and cannot express "ask someone else".
//
// 300s is the ABSOLUTE ceiling. v1.0.0 let the caller declare `max_staleness_seconds` with no cap,
// so a PEP passing i64::MAX evaluated a 30-day-old non-authoritative snapshot to Allow. A bound the
// constrained party chooses is not a bound.

// F2 — a tuple read served from a snapshot that is both stale past the ceiling AND not the
// authoritative copy is refused. Non-authoritative + fresh is fine; authoritative + stale is fine
// (it IS the truth). Only the conjunction is unsafe.
//
// This reads the snapshot through `resource.served_from`. v1.0.0 asked the CALLER for
// `context.snapshot_age_seconds`, i.e. it asked the party being constrained to report the fact that
// constrains it — a request could always claim age 0.
@id("F2")
forbid (
  principal,
  action == OyaPolicy::Action::"ReadRebacTuple",
  resource
)
when {
  resource.served_from.age_seconds > 300 && !resource.served_from.is_authoritative
};

// F3 — evaluating a snapshot older than the caller's declared tolerance is refused. The caller may
// only tighten below the ceiling, never loosen past it; F3b is what makes that true.
@id("F3")
forbid (
  principal,
  action == OyaPolicy::Action::"EvaluateAgainstSnapshot",
  resource
)
unless {
  context has max_staleness_seconds &&
  resource.age_seconds <= context.max_staleness_seconds
};

// F3b — the absolute ceiling, independent of anything the caller asserts. Also covers distribution:
// v1.0.0 gated distribution on signature only, so an ancient non-authoritative snapshot propagated
// fleet-wide.
@id("F3b")
forbid (
  principal,
  action in [
    OyaPolicy::Action::"EvaluateAgainstSnapshot",
    OyaPolicy::Action::"DistributeSnapshot"
  ],
  resource
)
when { resource.age_seconds > 300 && !resource.is_authoritative };

// F4 — an unverified snapshot never serves and never propagates. Verification precedes use,
// mirroring the load order in iam/adapters/cloud-pdp-bundle-file: verify, THEN parse.
@id("F4")
forbid (
  principal,
  action in [
    OyaPolicy::Action::"EvaluateAgainstSnapshot",
    OyaPolicy::Action::"DistributeSnapshot"
  ],
  resource
)
unless { resource.signature_verified };

// F4b — the same rule on the tuple read path, which reaches its snapshot indirectly.
@id("F4b")
forbid (
  principal,
  action == OyaPolicy::Action::"ReadRebacTuple",
  resource
)
unless { resource.served_from.signature_verified };
