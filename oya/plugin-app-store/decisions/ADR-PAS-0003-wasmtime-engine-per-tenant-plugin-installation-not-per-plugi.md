---
adr_id: ADR-PAS-0003
title: "Wasmtime engine per tenant-plugin installation, not per-plugin globally"
status: Proposed
date: 2026-05-18
microservice: plugin-app-store
owner_team: axis-ecosystem
related_adrs: [ADR-0213, ADR-0131, ADR-0132, ADR-0211]
doc_status: published
---

# ADR-PAS-0003: Wasmtime engine per tenant-plugin installation, not per-plugin globally

## Status

Proposed — 2026-05-18.

## Context

Scoped to plugin-app-store µservice substrate. Wasmtime engine per tenant-plugin installation, not per-plugin globally.

## Decision

One Wasmtime engine instance per tenant-plugin installation tuple. Shared engines across tenants would require capability gating at every call; per-installation engine moves the boundary to the kernel.

## Alternatives considered

### Per-plugin shared engine across tenants (REJECTED)

Cross-tenant data leakage risk; cannot tolerate.

## Consequences

Memory overhead ~128Mi per active installation; mitigated by opportunistic teardown after 60s idle.

## References

- ADR-0213 (parent EaaS architecture)
- microservices/plugin-app-store/PRD.md
