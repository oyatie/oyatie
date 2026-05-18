---
adr_id: ADR-SDK-0001
title: "ED25519 signing keys via OpenBao transit engine only; private keys never leave OpenBao perimeter"
status: Proposed
date: 2026-05-18
microservice: developer-sdk
owner_team: axis-ecosystem
related_adrs: [ADR-0213, ADR-0131, ADR-0132, ADR-0211]
doc_status: published
---

# ADR-SDK-0001: ED25519 signing keys via OpenBao transit engine only; private keys never leave OpenBao perimeter

## Status

Proposed — 2026-05-18.

## Context

Scoped to developer-sdk µservice substrate. ED25519 signing keys via OpenBao transit engine only; private keys never leave OpenBao perimeter.

## Decision

All developer signing keys are ED25519, generated and stored in OpenBao's transit secrets engine; signing operations are performed by OpenBao; private keys never returned to developer-sdk process memory.

## Alternatives considered

### Developer-managed private keys (REJECTED)

Compromise risk shifts to developer; revocation cascades break; not 2026 industry-grade.

## Consequences

OpenBao becomes load-bearing; mitigated by HA + auto-unseal via cloud-secrets µservice handoff.

## References

- ADR-0213 (parent EaaS architecture)
- microservices/developer-sdk/PRD.md
