---
adr_id: ADR-PAS-0005
title: "Per-installation rate-limit default 100 req/s; per-plugin override requires admin approval"
status: Proposed
date: 2026-05-18
microservice: plugin-app-store
owner_team: axis-ecosystem
related_adrs: [ADR-0213, ADR-0131, ADR-0132, ADR-0211]
doc_status: published
---

# ADR-PAS-0005: Per-installation rate-limit default 100 req/s; per-plugin override requires admin approval

## Status

Proposed — 2026-05-18.

## Context

Scoped to plugin-app-store µservice substrate. Per-installation rate-limit default 100 req/s; per-plugin override requires admin approval.

## Decision

Default rate limit applies to all installations at install time; per-plugin override (e.g., bursting allowance) requires tenant admin approval captured in Cedar policy fragment.

## Alternatives considered

### No default limit (REJECTED)

Misbehaving plugin can DoS tenant compute.
### Per-developer override (REJECTED)

Developers cannot self-grant higher limits; admin gating mandatory.

## Consequences

Misbehaving plugin contained to declared budget; admin override path documented.

## References

- ADR-0213 (parent EaaS architecture)
- microservices/plugin-app-store/PRD.md
