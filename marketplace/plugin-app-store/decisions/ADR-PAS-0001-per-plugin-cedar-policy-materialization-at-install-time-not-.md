---
adr_id: ADR-PAS-0001
title: "Per-plugin Cedar policy materialization at install time, not at runtime"
status: Proposed
date: 2026-05-18
microservice: plugin-app-store
owner_team: axis-ecosystem
related_adrs: [ADR-0213, ADR-0131, ADR-0132, ADR-0211]
doc_status: published
---

# ADR-PAS-0001: Per-plugin Cedar policy materialization at install time, not at runtime

## Status

Proposed — 2026-05-18.

## Context

Scoped to plugin-app-store µservice substrate. Per-plugin Cedar policy materialization at install time, not at runtime.

## Decision

Materialize the per-installation Cedar policy fragment at install time (synchronously, blocking install completion); runtime authorization queries hit the materialized fragment via the central governance evaluator. Avoids per-request synthesis cost.

## Alternatives considered

### Runtime synthesis (REJECTED)

Adds 5-10ms per authz query; unacceptable at sustained 10k qps tenant.

## Consequences

Install latency p99 ≤ 15s acceptable; runtime authz p99 ≤ 5ms achievable.

## References

- ADR-0213 (parent EaaS architecture)
- microservices/plugin-app-store/PRD.md
