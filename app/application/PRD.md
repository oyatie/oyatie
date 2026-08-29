---
doc_class: Owner-PRD
owner: app/application
status: Active
date: 2026-08-29
authority:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
  - app/application/ADR.md
---

# Application product requirements

<product_boundary>

Application is the tenant-facing slice of the platform: the surface a customer
binds to, and the path a customer's agentic capability actually runs through.
It composes the platform kernels; it does not own them. Identity, policy,
billing, tenancy and intelligence each own their own contracts, and this
product depends on them rather than restating them.

</product_boundary>

<requirements>

## A customer binds to a surface, not to a substrate

The product surface a customer buys is phase-invariant: it does not change
when fulfilment moves from rented public-cloud capacity to operated colo to
owned datacentres. Substrate movement is a routing concern behind the surface.

## Every invocation is gated before it runs

A capability invocation passes principal binding, licensing, policy
authorization, the tenant's autonomy ceiling, the data-use boundary and the
cost profile before any provider is called. A gate that refuses records why,
and no gate can be skipped by reaching a later one first.

## Autonomy is a ceiling the tenant sets

A capability may run no more autonomously than the tenant permits, and a
break-glass raise is itself recorded as evidence with the pre-raise decision
preserved alongside it.

## Spend is reserved before it is incurred

Budget is checked, then reserved, then committed. A failure after reservation
compensates rather than leaking the reservation, and the original error is
never masked by the compensation's own outcome.

## Every run leaves evidence

A run, its steps, its provider call and its disposition are recorded with an
attributable evidence chain and emitted to the outbox, whether the invocation
succeeded, was denied, or failed part-way.

</requirements>
