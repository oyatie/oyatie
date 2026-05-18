---
adr_id: ADR-SDK-0007
title: "Dev portal as Backstage extension, not standalone app"
status: Proposed
date: 2026-05-18
microservice: developer-sdk
owner_team: axis-ecosystem
related_adrs: [ADR-0213, ADR-0131, ADR-0132, ADR-0211]
doc_status: published
---

# ADR-SDK-0007: Dev portal as Backstage extension, not standalone app

## Status

Proposed — 2026-05-18.

## Context

Scoped to developer-sdk µservice substrate. Dev portal as Backstage extension, not standalone app.

## Decision

Per ADR-0170, Backstage is the canonical developer portal substrate. Developer-sdk ships a Backstage extension that mounts at oyatie.dev/developers; not a standalone Next.js / Astro / Hugo app.

## Alternatives considered

### Standalone Next.js portal (REJECTED)

ADR-0170 forbids; loses Backstage ecosystem benefits (TechDocs, Service Catalog).

## Consequences

Tied to Backstage release cadence; mitigated by extension semver pinning.

## References

- ADR-0213 (parent EaaS architecture)
- microservices/developer-sdk/PRD.md
