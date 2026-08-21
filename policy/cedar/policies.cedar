// Consolidated Cedar bundle — policy capability control surface
// Binding ADRs: ADR-0615 §5, ADR-0280 §D-13.D/§D-13.E, ADR-0243, ADR-0294.
// Cedar 4.12 (workspace lock).
//
// This file is the EXACT concatenation of policy/policy/*.cedar in lexical order. That is a
// deliberate, tested property: the fragment set is closed under consolidation, so the C0 face can
// assemble a distributable bundle by concatenation without changing any decision. The property is
// only preserved while no fragment contains a bare `forbid (principal, action, resource);` — see
// policy/cedar/README.md, which demonstrates with the real engine why that shape is unsafe here.
//
// Regenerate with:  cat policy/policy/*.cedar > /tmp/b && (this header) + /tmp/b

// ─── policy/policy/auditor-scope.cedar ───
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

// ─── policy/policy/authoring-grants.cedar ───
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

// ─── policy/policy/change-control.cedar ───
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

// ─── policy/policy/cross-tenant-isolation.cedar ───
// Cedar 4.12 fragment — cross-tenant-isolation
// Capability: policy (ADR-0615 §5, 24th registered capability)
// Binding ADRs: ADR-0280 §D-13.D, ADR-0615 §5
// Fragment soak window: >= 60s per ADR-0294
// Fragment version: 1.0.0
// Reviewed: 2026-08-20
//
// F-CROSS-TENANT. Defence in depth: every permit in this bundle already binds the tenant, and this
// forbid makes a permit that FORGETS to bind it non-exploitable. Applies to every action and every
// resource type in the namespace — all six resource entities carry `tenant_id` by schema.
//
// Note on shape: this file deliberately does NOT open with a bare
// `forbid (principal, action, resource);`. Cedar is default-deny by construction, so a bare forbid
// adds no denial — but because forbid unconditionally overrides permit, in the CONSOLIDATED bundle
// (cedar/policies.cedar) it would nullify every permit in every other fragment. See
// policy/cedar/README.md for the executable demonstration.

// F1 — a principal may never act on another tenant's authorization state.
forbid (
  principal,
  action,
  resource
)
when { principal.tenant_id != resource.tenant_id };

// ─── policy/policy/runtime-grants.cedar ───
// Cedar 4.12 fragment — runtime-grants
// Capability: policy (ADR-0615 §5, 24th registered capability)
// Binding ADRs: ADR-0280 §D-13.D/E (C0 face), ADR-0191 (edge vs origin)
// Fragment soak window: >= 60s per ADR-0294
// Fragment version: 1.0.0
// Reviewed: 2026-08-20
//
// The C0-face permits: what the per-cell runtime and its distributor may do. These are the hot
// paths, and they are the ones static-stability.cedar constrains.

// P6 — an attested PEP workload evaluates against its cell-local snapshot.
permit (
  principal,
  action == OyaPolicy::Action::"EvaluateAgainstSnapshot",
  resource
)
when {
  principal.role == "pep-workload" &&
  principal.tenant_id == resource.tenant_id &&
  principal.spiffe_id != ""
};

// P7 — an attested PEP workload reads ReBAC tuples for its own tenant.
permit (
  principal,
  action == OyaPolicy::Action::"ReadRebacTuple",
  resource
)
when {
  principal.role == "pep-workload" &&
  principal.tenant_id == resource.tenant_id &&
  principal.spiffe_id != ""
};

// P8 — the distributor propagates signed snapshots. F4 stops it propagating unverified ones.
permit (
  principal,
  action == OyaPolicy::Action::"DistributeSnapshot",
  resource
)
when {
  principal.role == "policy-distributor" &&
  principal.tenant_id == resource.tenant_id
};

// ─── policy/policy/static-stability.cedar ───
// Cedar 4.12 fragment — static-stability
// Capability: policy (ADR-0615 §5, 24th registered capability)
// Binding ADRs: ADR-0280 §D-13.E (static-stability invariant)
// Fragment soak window: >= 60s per ADR-0294
// Fragment version: 1.0.0
// Reviewed: 2026-08-20
//
// The invariant, verbatim: "a stale snapshot must DENY or route to the authoritative shard — never
// silently authorize." These forbids are the DENY half expressed as policy. The route-to-
// authoritative half is a runtime concern and is specified in policy/CONTRACT.md, because Cedar
// answers allow/deny and cannot express "ask someone else".

// F2 — a read served from a snapshot that is both stale beyond its bound AND not the authoritative
// copy is refused. Non-authoritative + fresh is fine; authoritative + stale is fine (it IS the
// truth). Only the conjunction is unsafe.
forbid (
  principal,
  action == OyaPolicy::Action::"ReadRebacTuple",
  resource
)
when {
  context.snapshot_age_seconds > 300 && !context.snapshot_is_authoritative
};

// F3 — evaluating against a snapshot older than the caller's declared tolerance is refused. The
// caller states the bound; the plane enforces it. A PEP that needs strong freshness sets it low.
forbid (
  principal,
  action == OyaPolicy::Action::"EvaluateAgainstSnapshot",
  resource
)
when { resource.age_seconds > context.max_staleness_seconds };

// F4 — an unverified snapshot never serves and never propagates. Signature verification precedes
// use, mirroring the load order in iam/adapters/cloud-pdp-bundle-file: verify, THEN parse.
forbid (
  principal,
  action in [
    OyaPolicy::Action::"EvaluateAgainstSnapshot",
    OyaPolicy::Action::"DistributeSnapshot"
  ],
  resource
)
unless { resource.signature_verified };
