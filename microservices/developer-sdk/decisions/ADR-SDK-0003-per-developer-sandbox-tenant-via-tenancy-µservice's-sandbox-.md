---
adr_id: ADR-SDK-0003
title: "Per-developer sandbox tenant via tenancy µservice's sandbox-class, not a fork"
status: Proposed
date: 2026-05-18
microservice: developer-sdk
owner_team: axis-ecosystem
related_adrs: [ADR-0213, ADR-0131, ADR-0132, ADR-0211]
doc_status: published
---

# ADR-SDK-0003: Per-developer sandbox tenant via tenancy µservice's sandbox-class, not a fork

## Status

Proposed — 2026-05-18.

## Context

Scoped to developer-sdk µservice substrate. Per-developer sandbox tenant via tenancy µservice's sandbox-class, not a fork.

## Decision

Sandbox tenants are first-class tenants in tenancy µservice with sandbox class. Re-uses tenancy substrate; no parallel sandbox-tenant code path.

## Alternatives considered

### Fork tenancy code for sandbox (REJECTED)

Code duplication + divergence; ADR-0132 no-suite policy guidance applies.

## Consequences

tenancy becomes load-bearing for sandbox; mitigated by per-class quota.

## References

- ADR-0213 (parent EaaS architecture)
- microservices/developer-sdk/PRD.md
