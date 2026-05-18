---
adr_id: ADR-SDK-0006
title: "KYC pipeline in-house; no external KYC SaaS (Onfido, Persona, Stripe Identity)"
status: Proposed
date: 2026-05-18
microservice: developer-sdk
owner_team: axis-ecosystem
related_adrs: [ADR-0213, ADR-0131, ADR-0132, ADR-0211]
doc_status: published
---

# ADR-SDK-0006: KYC pipeline in-house; no external KYC SaaS (Onfido, Persona, Stripe Identity)

## Status

Proposed — 2026-05-18.

## Context

Scoped to developer-sdk µservice substrate. KYC pipeline in-house; no external KYC SaaS (Onfido, Persona, Stripe Identity).

## Decision

ID document OCR + selfie liveness check + sanctions screening run in-house per ADR-0211. Source data: developer-submitted document image + liveness video; output: structured KYC decision + audit trail.

## Alternatives considered

### Onfido (REJECTED)

ADR-0211 forbids external KYC SaaS.
### Stripe Identity (REJECTED)

Same.

## Consequences

Higher build cost; offset by no per-verification fee + control over false-positive rate.

## References

- ADR-0213 (parent EaaS architecture)
- microservices/developer-sdk/PRD.md
