---
doc_class: Owner-SPEC
owner: app/application
status: Active
date: 2026-08-29
authority:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
  - app/application/ADR.md
  - app/application/PRD.md
---

# Application contract

<trust_boundary>

## Trust zones

- The invocation request and its principal are untrusted until the principal
  is bound to the request's tenant and user. A mismatch is refused before any
  body lookup.
- Capability registrations, cost profiles and MCP contracts are tenant data,
  validated structurally before they can affect a decision.
- Provider responses are untrusted; a receipt is constructed from the route,
  not from the provider's own claims about itself.

</trust_boundary>

<surfaces>

`core/foundation` — the composition. Tenants and cells, identity and tokens,
data-use grants, policy publication and authorization, capability
registration with eval sets and tenant grants, MCP discovery and invocation,
cost budgets, capability invocation, settlement and denial, regional packs,
the object graph and the outbox.

`core/surface` — the phase-invariant product surface: `CloudSurface` over
compute, storage, network, identity, regions, billing, observability and
FinOps, with the compute SKU taxonomy and its fulfilment phases.

`facade/` — the shell frontend and the SaaS plugin app.

</surfaces>

<invariants>

- An invocation denial writes its audit trail before the error returns; an
  audit-write failure surfaces ahead of the denial rather than replacing it.
- A budget reservation is either committed or compensated. Compensation never
  replaces the error that caused it.
- A run's disposition, its evidence kind, and the reason recorded on a denial
  agree with the gate that refused it.
- A surface is emitted with its schema version stamped, and a fulfilment set
  that does not cover the declared phases is refused at construction.
- Annotations reach a caller only on an allow.

</invariants>
