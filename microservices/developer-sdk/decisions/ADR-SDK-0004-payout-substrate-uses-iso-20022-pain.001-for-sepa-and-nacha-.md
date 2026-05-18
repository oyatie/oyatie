---
adr_id: ADR-SDK-0004
title: "Payout substrate uses ISO 20022 pain.001 for SEPA and NACHA for ACH; in-house bank adapters"
status: Proposed
date: 2026-05-18
microservice: developer-sdk
owner_team: axis-ecosystem
related_adrs: [ADR-0213, ADR-0131, ADR-0132, ADR-0211]
doc_status: published
---

# ADR-SDK-0004: Payout substrate uses ISO 20022 pain.001 for SEPA and NACHA for ACH; in-house bank adapters

## Status

Proposed — 2026-05-18.

## Context

Scoped to developer-sdk µservice substrate. Payout substrate uses ISO 20022 pain.001 for SEPA and NACHA for ACH; in-house bank adapters.

## Decision

SEPA payouts emit ISO 20022 pain.001 XML messages; US ACH emits NACHA format files; KR uses KFTC firm-bank protocol. All adapters in-house per ADR-0211; no Stripe Connect hosted dependency.

## Alternatives considered

### Stripe Connect hosted (REJECTED)

ADR-0211 forbids external payout SaaS.

## Consequences

Higher implementation burden; offset by no per-transaction fee + full control over reconciliation.

## References

- ADR-0213 (parent EaaS architecture)
- microservices/developer-sdk/PRD.md
