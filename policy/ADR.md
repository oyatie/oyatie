---
doc_class: Owner-ADR
owner: policy
status: Accepted
date: 2026-08-29
inherits:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
---

# Policy decisions in force

This file specializes ADR-0719 for `policy/`. ADR-0719 lists `policy` as a
BUILD capability and directs that the authorization crates be extracted from
`iam/`. This file records the boundary that extraction establishes. It does
not claim that a combined Cedar + relationship-based decision path has landed.

<current_state>

## Evidence at owner-law adoption

- `core/cedar-domain` holds the Cedar-shaped policy kernel: versioned policy
  records, role and attribute evaluation, authoring-time lint, version
  diffing, obligation annotations, and the Zanzibar relationship-tuple
  vocabulary with its tuple-store port.
- The tuple-store port has no adapter. `UsersetRewrite` is a data shape with
  no evaluator. No namespace configuration binds an object type and relation
  to a rewrite tree.
- The `cedar-policy`-backed PDP, its bundle store, its publish API, and its
  runnable service remain under `iam/`. They are not yet this owner's.

</current_state>

<decisions>

## Authorization is a capability, not an identity feature

**achieves:** one owner for policy evaluation, graph expansion, consistency
tokens, and decisions, addressable by every capability without depending on
the identity product.

**origin:** the decision plane grew inside `iam/` because the first policy
enforcement point was an identity surface. Consumers in `tenancy/`, `k8s/`,
`app/`, and `intelligence/` now reach across a capability boundary for a
decision that has nothing to do with who is authenticating.

**rule:** policy evaluation, relationship expansion, consistency tokens, and
decision emission are owned by `policy/`. Identity supplies authenticated
principals, groups, and relationship facts; it does not decide. A capability
that needs a decision depends on `policy/`, never on `iam/`, for it.

**ensure:** the decision path compiles and tests inside `policy/` with no
path dependency on an `iam/` crate other than the shared platform contract
kernel; `iam/` retains no policy evaluation surface of its own beyond the
workload-identity gate ADR-0631 assigns to identity.

**overturn_when:** measured evidence shows a decision surface that cannot be
served without identity-internal state that the contract kernel cannot carry.

## The relationship graph materializes into the entity hierarchy

**achieves:** one decision algorithm. Relationship-derived membership and
statically supplied membership reach the engine through the same structure,
so a policy cannot observe which one produced an edge.

**origin:** treating a relationship check as a second decision path alongside
policy evaluation duplicates the deny-by-default and forbid-overrides-permit
semantics, and the duplicate is where they drift apart.

**rule:** userset expansion produces entity-parent edges consumed by the
existing policy-decision port. Expansion is a pre-evaluation step, not a
parallel authorizer, and it never returns a decision of its own.

**ensure:** the expander's output type is the engine's entity-slice input
type, and a conformance suite drives allow and deny through relationship
membership and static membership against identical assertions.

**overturn_when:** a required semantic is unrepresentable as a parent edge and
a recorded challenge shows no encoding of it satisfies the same tests.

</decisions>
