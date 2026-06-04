---
id: ADR-0478
title: "oya-billing — bespoke Rust billing engine superseding Lago"
status: Accepted
date: 2026-05-28
authority: founder
milestone: M-BILLING-ENGINE-V2
planning_impact: true
supersedes: []
superseded_by: []
related: [ADR-0083, ADR-0509]
---

# ADR-0478 — oya-billing: bespoke Rust billing engine

## Status

Accepted — 2026-05-28 (founder-locked). Retires the Lago Phase-1 plan; no status-bearing ADR record exists for that predecessor plan.

## Context

The Lago Phase-1 plan adopted Lago (AGPL-3.0) as a Phase-1 billing engine stepping stone. Two blockers have
since hardened into decision criteria:

1. **AGPL-3.0 avoidance.** Lago's server binary is AGPL-3.0. Oyatie's billing surface is exposed
   as a product primitive to tenants (D5); AGPL redistribution risk cannot be contained in that
   topology. Stripe, Shopify, and AWS all run bespoke billing planes — none run AGPL-gated OSS.

2. **Rust doctrine.** The Lago Phase-1 plan accepted a Ruby/Rails service. The hyperscaler-lens filter requires
   a fully self-hostable, Rust-native component at every substrate seam. A bespoke engine written
   in Rust + Axum + Connect-RPC matches this doctrine without exception.

## Decision

Build `oya-billing` — a bespoke Rust billing engine — as the canonical billing plane.

### D1 — New µservice `microservices/oya-billing/`

Rust workspace. Axum HTTP + Connect-RPC (Connect-RPC API plan) API surface. PostgreSQL (PostgreSQL storage plan) for durable
billing state (subscriptions, invoices, line items, credits). Apache Pulsar (Pulsar 4.x + Oxia substrate plan) for
billable-event stream fan-out. Flat single-concern layout per ADR-0131 + ADR-0132.

### D2 — Billing primitives

Pricing plans: tiered, usage-based, prepaid, freemium. Subscription lifecycle states:
`active / canceled / past_due / trial`. Invoicing: line items, taxes, prorations, credit notes.
Payment integration: Stripe API as default processor; per-tenant alternate processors via a
`PaymentAdapter` trait (GoCardless, Adyen, wire). All state changes emit Pulsar events.

### D3 — Upstream integrations

`oya-meter` (ADR-0479) usage events arrive via Pulsar → oya-billing billable-metrics pipeline.
`oya-cost` (ADR-0480) K8s allocation charges arrive as add-on charge records. Both upstream
services push; oya-billing is the single downstream sink for all chargeable signals.

### D4 — Isolation and delivery

Per-tenant subscription isolation enforced via Cedar (ADR-0083) policy:
`billing-admin` action set gated; cross-tenant invoice access always forbids-wins.
Crossplane (Crossplane XR plan) `TenantSubscription` XR provisions oya-billing plan bindings.
Invoice delivery via oya-notify (oya-notify plan) on Pulsar invoice-finalized events.

### D5 — Billing-as-a-product

Tenants running services on oyatie use oya-billing for their own end-customers. The Connect-RPC
surface and webhook fan-out are exposed as a product primitive with per-tenant namespace
isolation. This is a multi-tenant oya-billing model — distinct from oyatie billing its tenants.

## Hyperscaler-lens

| Criterion | Assessment |
|---|---|
| (a) Active upstream | ✅ Bespoke — no upstream dependency risk |
| (b) License | ✅ Bespoke — no AGPL-3.0 or copyleft exposure |
| (c) Self-hostable | ✅ Bespoke — fully self-hostable by definition |
| (d) Hyperscaler-equivalent | ✅ Stripe Billing, Shopify Billing, AWS Billing are all bespoke internal planes; oya-billing follows this pattern exactly |

## Alternatives

- **Lago Phase-1 plan:** AGPL-3.0 redistribution risk in billing-as-a-product topology;
  Ruby/Rails stack violates Rust doctrine. Retired as transitional stepping stone.
- **Metronome / Amberflo:** Commercial managed SaaS; violates self-host requirement.
- **OpenMeter alone:** Meters but does not invoice, manage subscriptions, or dispatch payments.

## Consequences

Investment: ~6–9 months engineering to reach parity with Lago Phase-1 scope. Accepted by founder.
PostgreSQL + Pulsar must be provisioned first (both already required by other services).
`TenantSubscription` XR schema requires a Crossplane XRD version bump — coordinate with Crossplane XR plan.
The Lago Phase-1 plan is retired; any Lago deployment work is stopped.

## Integration

```
oya-meter (ADR-0479) ──► Pulsar ──► oya-billing billable-metrics ──► Invoice generation
oya-cost  (ADR-0480) ──► Pulsar ──► oya-billing add-on charges    ──► oya-notify (oya-notify plan)
Crossplane XR (Crossplane XR plan) ──► TenantSubscription ──► plan binding
Cedar (ADR-0083) ──► billing-admin authz gate
Stripe API / PaymentAdapter ──► payment dispatch
```

## Promotion Rationale

Bespoke billing is the hyperscaler pattern (Stripe, Shopify, AWS). Lago's AGPL gate is
irreconcilable with billing-as-a-product topology. Rust + Connect-RPC matches the doctrine
already established across the substrate. This ADR unblocks M-BILLING-ENGINE-V2.

## Implementation pattern (ADR-0509 alignment)

Per ADR-0509 (Hyperscaler service decomposition pattern), `oya-billing` ships as **single-crate-per-service with mod-based subsystems**. Per-use-case crate sprawl is superseded. Use cases remain valid as domain concepts (subsystem boundaries inside `src/<subsystem>/`).
