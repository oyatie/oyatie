---
microservice: observability
ip: IP-026
title: SSE transport impl (one-way streams — log tail, metric tail, AI streaming responses)
status: Drafting
owner: axis-observability
co_owners: [axis-frontend]
date: 2026-05-18
related_adrs: [ADR-0145, ADR-0153, ADR-0186, ADR-0208]
---

# IP-026 — SSE transport impl

## Purpose

Per ADR-0208, SSE is canonical for one-way streams. Wire the SSE transport via `oya-shared-realtime-transport-kernel::SseTransport` + axum SSE adapter at the gateway tier. Cell-local subscription manager backs SSE channels (per ADR-0153 outbox).

## Acceptance criteria

1. `oya-observability-sse-adapter` crate wraps axum SSE; consumes `oya-shared-realtime-transport-kernel`.
2. `Last-Event-ID` resume across reconnects.
3. Heartbeat: 30s comment-line.
4. Per-tenant ceiling: 50,000 concurrent SSE connections (per ADR-0208).
5. Resume cursor stored in Valkey pub-sub cell-local manager.
6. ≥ 6 integration tests: SSE connect + heartbeat + reconnect resume + payload-budget reject + per-tenant cap + cross-cell routing.

## Cross-references

- ADR-0208 — realtime transport.
- ADR-0153 — outbox / per-cell.
- `oya-shared-realtime-transport-kernel`.
