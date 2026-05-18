---
microservice: observability
ip: IP-028
title: Loro presence binding (CRDT awareness protocol → per-cell subscription manager)
status: Drafting
owner: axis-observability
co_owners: [axis-frontend]
date: 2026-05-18
related_adrs: [ADR-0145, ADR-0204, ADR-0208]
---

# IP-028 — Loro presence binding

## Purpose

Wire `oya-shared-presence-kernel::LoroPresenceTracker` to the cell-local subscription manager so Workflow Studio canvas + similar surfaces can replicate awareness state across collaborators (per ADR-0145 Loro pin + ADR-0204 canvas integration).

## Acceptance criteria

1. `oya-observability-presence-adapter` crate wires Loro awareness to Valkey pub-sub.
2. Per-tenant + per-room isolation (kernel invariant).
3. Stale-entry pruning at 30s idle.
4. Cursor coordinates validated (kernel rejects NaN / Inf).
5. ≥ 5 integration tests.

## Cross-references

- ADR-0145 — Loro pin.
- ADR-0204 — canvas.
- `oya-shared-presence-kernel`.
