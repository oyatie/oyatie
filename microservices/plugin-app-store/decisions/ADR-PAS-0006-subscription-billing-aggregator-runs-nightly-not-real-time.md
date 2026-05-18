---
adr_id: ADR-PAS-0006
title: "Subscription billing aggregator runs nightly, not real-time"
status: Proposed
date: 2026-05-18
microservice: plugin-app-store
owner_team: axis-ecosystem
related_adrs: [ADR-0213, ADR-0131, ADR-0132, ADR-0211]
doc_status: published
---

# ADR-PAS-0006: Subscription billing aggregator runs nightly, not real-time

## Status

Proposed — 2026-05-18.

## Context

Scoped to plugin-app-store µservice substrate. Subscription billing aggregator runs nightly, not real-time.

## Decision

Aggregate per-tenant per-plugin billing on a nightly batch; emit one consolidated handoff to finops-portal. Real-time aggregation creates feedback loops with subscription state changes.

## Alternatives considered

### Real-time aggregator (REJECTED)

Couples subscription mutations to billing state machine; ordering becomes fragile.

## Consequences

Tenant invoice lag ≤ 24h; acceptable per industry norm.

## References

- ADR-0213 (parent EaaS architecture)
- microservices/plugin-app-store/PRD.md
