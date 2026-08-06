---
adr_id: ADR-PAS-0002
title: "Vetting pipeline ordered stage execution, never parallelized"
status: Proposed
date: 2026-05-18
microservice: plugin-app-store
owner_team: axis-ecosystem
related_adrs: [ADR-0213, ADR-0131, ADR-0132, ADR-0211]
doc_status: published
---

# ADR-PAS-0002: Vetting pipeline ordered stage execution, never parallelized

## Status

Proposed — 2026-05-18.

## Context

Scoped to plugin-app-store µservice substrate. Vetting pipeline ordered stage execution, never parallelized.

## Decision

Stages run in fixed declared order (signature → vulnerability → isolation → capability → data-use → accessibility → AI-Act → perf-budget); rejection on first failure with all downstream stages skipped. Determinism > throughput.

## Alternatives considered

### Parallel stage execution (REJECTED)

Rejection reason becomes non-deterministic (which stage 'won'?); inflates audit-chain surface.

## Consequences

Throughput dominated by slowest stage; mitigated by horizontal worker scaling.

## References

- ADR-0213 (parent EaaS architecture)
- microservices/plugin-app-store/PRD.md
