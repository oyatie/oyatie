---
microservice: observability
ip: IP-027
title: WebSocket transport impl (bidirectional product surfaces — canvas collab, chat)
status: Drafting
owner: axis-observability
co_owners: [axis-frontend]
date: 2026-05-18
related_adrs: [ADR-0145, ADR-0153, ADR-0186, ADR-0208]
---

# IP-027 — WebSocket transport impl

## Purpose

Per ADR-0208, WebSocket is canonical for bidirectional surfaces (Workflow Studio canvas collab via Loro CRDT, shared cursors, in-product chat). Wire via `oya-shared-realtime-transport-kernel::WebSocketTransport` + axum WebSocket adapter.

## Acceptance criteria

1. `oya-observability-websocket-adapter` crate wraps axum WebSocket + tokio-tungstenite.
2. Session-resume token issued on first connect.
3. Heartbeat: 30s ping frame.
4. Per-tenant ceiling: 10,000 concurrent WebSocket connections (per ADR-0208).
5. Protobuf-over-WebSocket framing (AsyncAPI-described).
6. Cell-local subscription manager + cross-cell outbox replicator.
7. ≥ 6 integration tests.

## Cross-references

- ADR-0208 — realtime transport.
- `oya-shared-realtime-transport-kernel`.
