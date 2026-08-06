---
id: ADR-0100
title: Foundry Supervisor Public Contract (Lean-a10)
status: Superseded
superseded_by: [ADR-709]
doc_status: published
owner: council-architecture
date: 2026-05-15
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0100: Foundry Supervisor Public Contract (Lean-a10)

## Status
Accepted

## Context
The Foundry Supervisor needs to compose multiple existing kernels (`RoutePolicy`, `UsageEnforcement`, `Billing`, `AutonomyCeiling`). Per ADR-0056 (Port-in-Kernel), any cross-product boundary types should live in the kernel. We want to avoid polluting existing kernels with supervisor-specific APIs unless absolutely necessary.

## Decision
The Foundry Supervisor will expose zero new public APIs on existing kernels. Instead:
1. All supervisor-specific types (`SessionTicket`, `MessageId`, etc.) live in the new `oya-intelligence-supervisor-kernel`.
2. Existing kernel primitives are composed as pure ports.
3. The `AccountSnapshotProvider` port lives inside `oya-intelligence-supervisor-kernel` to keep the supervisor I/O-free without changing `oya-intelligence-account-domain`.

## Drivers
- **Lean-a10 (Zero-Surface-Change):** Maintain existing kernel stability.
- **Port-based Composition:** Favor composition over inheritance or tight coupling.

## Alternatives Considered
- **Adding supervisor types to account-domain:** Rejected to keep domain logic focused on account lifecycle, not session execution.

## Consequences
- 4 new crates (kernel, app, adapter, conformance) are introduced.
- Existing kernels remain byte-identical (verified via `cargo public-api` snapshots).
