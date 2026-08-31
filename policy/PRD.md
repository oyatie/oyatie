---
doc_class: Owner-PRD
owner: policy
status: Active
date: 2026-08-29
authority:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
  - policy/ADR.md
---

# Policy decision requirements

<product_boundary>

Policy answers one question for every capability and product in the
repository: may this principal take this action on this resource, right now,
under this tenant's rules. It owns the answer and the evidence for it.

Policy does not authenticate. It receives an already-verified principal and
the facts about that principal's relationships. Identity owns who the caller
is; Policy owns what that caller may do.

</product_boundary>

<requirements>

## Decisions are fail-closed

Every refusal is a deny. A malformed request, an unknown action, an
unreachable tuple store, a stale consistency pin, or an internal fault
produces a deny or a typed refusal that the enforcement point must treat as
deny. There is no path on which absence of a decision reads as permission.

## Decisions are attributable

Every decision — allow or deny, freshly evaluated or served from cache —
yields one record naming the principal, action, resource, tenant, the policy
version it was decided against, and the policies that determined it.

## Relationships are first-class inputs

Authorization questions in this product are relationship questions: a
document is visible because it sits in a folder a group can read, and that
group's membership is itself derived. Policy expands those relationships at
decision time from a tuple store, at a caller-pinnable snapshot, rather than
requiring each enforcement point to assemble the graph correctly on its own.

## Consistency is explicit and callable

A caller that has just written a relationship can demand a decision no older
than that write. A caller that can tolerate staleness can say so and get a
cheaper answer. The token that expresses this is opaque to callers and
ordered by the tuple store.

## Tenant isolation is structural

A tenant's policies apply only to that tenant's decisions, and cross-tenant
grants are refused by a rule that no per-tenant policy can override.

</requirements>
