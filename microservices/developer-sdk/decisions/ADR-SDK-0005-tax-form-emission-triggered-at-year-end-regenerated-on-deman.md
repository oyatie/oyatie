---
adr_id: ADR-SDK-0005
title: "Tax form emission triggered at year-end, regenerated on demand, never re-issued silently"
status: Proposed
date: 2026-05-18
microservice: developer-sdk
owner_team: axis-ecosystem
related_adrs: [ADR-0213, ADR-0131, ADR-0132, ADR-0211]
doc_status: published
---

# ADR-SDK-0005: Tax form emission triggered at year-end, regenerated on demand, never re-issued silently

## Status

Proposed — 2026-05-18.

## Context

Scoped to developer-sdk µservice substrate. Tax form emission triggered at year-end, regenerated on demand, never re-issued silently.

## Decision

Annual 1099-MISC + VAT MOSS + KR VAT emit at fiscal year-end. Developer can request re-emission via portal; re-emission creates a new ledger entry; never silently replaces a prior form.

## Alternatives considered

### Silent re-emission (REJECTED)

Forensic integrity broken; tax authority cross-check fails.

## Consequences

Tax form ledger is append-only; audit-chain integrity preserved.

## References

- ADR-0213 (parent EaaS architecture)
- microservices/developer-sdk/PRD.md
