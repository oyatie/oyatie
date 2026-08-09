---
adr_id: ADR-PAS-0007
title: "Per-plugin action audit trail seals via audit-chain µservice; plugin-app-store NOT authoritative"
status: Proposed
date: 2026-05-18
microservice: plugin-app-store
owner_team: axis-ecosystem
related_adrs: [ADR-0213, ADR-0131, ADR-0132, ADR-0211]
doc_status: published
---

# ADR-PAS-0007: Per-plugin action audit trail seals via audit-chain µservice; plugin-app-store NOT authoritative

## Status

Proposed — 2026-05-18.

## Context

Scoped to plugin-app-store µservice substrate. Per-plugin action audit trail seals via audit-chain µservice; plugin-app-store NOT authoritative.

## Decision

Every plugin action emits a seal event to audit-chain µservice's outbox; plugin-app-store reads from audit-chain for tenant-facing trail browsing. audit-chain is authoritative; plugin-app-store caches read projections.

## Alternatives considered

### plugin-app-store authoritative (REJECTED)

Forks the audit-chain invariant; cross-µservice audit unification breaks.

## Consequences

Audit-chain becomes load-bearing dependency; mitigated by buffered emission + chain integrity gate.

## References

- ADR-0213 (parent EaaS architecture)
- microservices/plugin-app-store/PRD.md
