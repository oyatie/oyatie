---
doc_class: Standard
shape: standard
length_cap: 250
authority_tier: 2
status: Accepted
date: 2026-05-18
purpose: |
  Canonical realtime transport tier: SSE for one-way; WebSocket for bidirectional product surfaces;
  gRPC streaming for service-to-service only. Reconnect + resume rules.
canonical_authority: docs/decisions/ADR-0704-k8s-port-live-apex.md
related_adrs:
  - ADR-0145
  - ADR-0153
  - ADR-0204
  - ADR-0208
enforced_by: check-realtime-transport-tier
---

# Realtime Transport Tier Standard

## Authority

This standard implements ADR-0208.

## Tier table (closed)

| Tier | Use | Library / primitive |
|---|---|---|
| **SSE** | One-way streams to clients (log tail, metric tail, AI streaming responses, status feed, deploy progress) | `EventSource` (web), `URLSession.bytes` (Apple), `OkHttp` SSE (Android), `libsoup3` (GTK), `HttpClient` streaming (WinUI 3) |
| **WebSocket** | Bidirectional product surfaces (canvas collab via Loro CRDT, shared cursors, chat) | `WebSocket` (web), `URLSessionWebSocketTask` (Apple), `OkHttp` (Android), `libsoup3` (GTK), `MessageWebSocket` (WinUI 3) |
| **gRPC streaming** | Service-to-service streams (inter-µservice event flow) — NEVER client-facing | `tonic` (Rust); gRPC interop via protobuf |

## BLOCKED

1. **Long-polling** — SSE handles every use case long-polling does.
2. **WebTransport (HTTP/3)** — deferred until browser support broad (≥ 2027).
3. **gRPC streaming on client-facing surfaces** — gRPC-Web needs a proxy hop.
4. **Custom binary protocols over WebSocket without an AsyncAPI schema** — use protobuf-over-WebSocket
   or JSON-over-WebSocket described by AsyncAPI / OpenAPI-Async.

## Connection management (RFC-2119)

1. Heartbeat **MUST** fire every 30 seconds (SSE comment line; WebSocket ping frame).
2. Reconnect **MUST** use exponential backoff (1s → 30s) with ± 25% jitter.
3. Resume:
   - SSE — `Last-Event-ID` header replays from cursor.
   - WebSocket — session-resume token + last-seen sequence.
4. Server **MUST** drop idle connections after 5 minutes of zero traffic.

## Multi-region routing

Sticky-session is BLOCKED per ADR-0153.

- Per-cell subscription manager backed by Valkey pub-sub.
- Resume tokens carry cell-id; reconnect routes back to the correct cell.
- Cross-cell fan-out via outbox pattern (per ADR-0153).

## Payload-budget envelope

| Tier | Default max payload |
|---|---|
| SSE | 1 MiB |
| WebSocket | 4 MiB |
| gRPC streaming | 16 MiB |

Per-µservice tunable in `manifest.json` `realtime.payload_budget`.

## Coverage gate

`check-realtime-transport-tier` (advisory) flags:

1. Bidirectional traffic on SSE.
2. WebSocket on one-way surface.
3. gRPC streaming on client-facing surface.
4. Unknown tier labels.

## Cross-references

- ADR-0208 — realtime transport tier (this ADR is the authority).
- ADR-0145 — inter-microservice communication (gRPC streaming canonical for service-to-service).
- ADR-0153 — observability backplane (outbox + per-cell architecture).
- ADR-0204 — canvas (WebSocket consumer for Loro CRDT sync).
- RFC 6455 — WebSocket Protocol.
- WHATWG HTML Living Standard — Server-Sent Events.
