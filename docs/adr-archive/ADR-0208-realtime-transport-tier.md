---
id: ADR-0208
status: Superseded
deciders: council-architecture, axis-frontend, axis-product, axis-data
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-0704]
related: [ADR-0145, ADR-0153, ADR-0182, ADR-0185, ADR-0204]
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0208 — Realtime transport tier: SSE / WebSocket / gRPC streaming with closed responsibility split

## Status

Accepted (2026-05-18). Pins the realtime transport tier model for oyatie. Each tier owns one responsibility; no overlap; no long-polling.

## Context

oyatie surfaces multiple realtime patterns:

- **One-way streams** to clients: log tail, metric tail, AI streaming responses, status feed, deploy progress.
- **Bidirectional product surfaces**: Workflow Studio canvas collab (Loro CRDT sync per ADR-0145), shared cursors, in-product chat.
- **Service-to-service streams**: inter-µservice event flow (per ADR-0145 + ADR-0153).

The bar:

- **One canonical transport per pattern.** No "ad-hoc long-polling here, SSE there."
- **Open standards.** No proprietary protocols.
- **Reconnect + resume.** Every stream resumes via `Last-Event-ID` (SSE) or session-resume token (WebSocket).
- **Multi-region.** Sticky-session anti-pattern BLOCKED per ADR-0153 outbox / per-cell architecture.

## Decision

### Three-tier transport model

| Tier | Use | When |
|---|---|---|
| **SSE (Server-Sent Events)** | One-way server → client streams | Log tail, metric tail, AI streaming responses, status feed, deploy progress |
| **WebSocket** | Bidirectional client-facing product surfaces | Workflow Studio canvas collab (Loro CRDT sync), shared cursors, chat |
| **gRPC streaming** | Service-to-service streams | Internal event fan-out (per ADR-0145 / ADR-0153) — NEVER client-facing |

### What's BLOCKED

- **Long-polling** — SSE is fully supported in every browser since 2015. Long-polling adds overhead without benefit.
- **WebTransport (HTTP/3 transport)** — defer adoption until browser support is broad (currently Chrome-only stable; Firefox / Safari behind flags). Revisit in ADR follow-up post 2027.
- **gRPC streaming on client-facing surfaces** — gRPC-Web needs a proxy hop + only supports unary + server-streaming. SSE + WebSocket cover what gRPC-Web would.
- **Custom binary protocols over WebSocket** without an AsyncAPI schema. WebSocket message framing MUST use protobuf-over-WebSocket OR JSON-over-WebSocket with an AsyncAPI / OpenAPI-Async description.

### Connection management

- **Heartbeat:** 30 seconds (ping frame for WebSocket; comment-line for SSE).
- **Reconnect:** exponential backoff (1s → 30s, capped), with jitter (±25%).
- **Resume:**
  - SSE → `Last-Event-ID` header replays from cursor.
  - WebSocket → session-resume token issued by server on first connect; on reconnect, client presents token + last seen sequence.
- **Liveness:** server drops idle connections after 5 minutes of zero traffic; client must heartbeat.

### Concrete per-tenant ceilings (scalability NOW)

- **Per-tenant concurrent WebSocket connections: 10,000.** Cell-local subscription manager enforces; over-cap returns 429.
- **Per-tenant concurrent SSE connections: 50,000.** SSE is one-way + cheaper per-connection; higher ceiling.
- **Per-cell aggregate concurrent connections: 200,000.** HPA on cell-local subscription manager pods.
- **Reconnect p99 SLO: 30 seconds** (exponential backoff cap + jitter).
- **Resume hit rate p99: 95%** within 60 seconds of disconnect (cursor still valid).
- **Payload-budget overflow rate: < 0.1%** of messages (kernel rejects oversized).

### Multi-region routing

Sticky-session is BLOCKED per ADR-0153 / outbox-pattern. Instead:

- **Subscription routing:** per-cell subscription manager backed by Redis Cluster / Valkey pub-sub (per ADR-0153).
- **Resume tokens carry the cell-id** so a reconnect on a different region/cell re-routes to the correct cell-local subscription manager.
- **Cross-cell fan-out:** the publisher writes once to its cell's outbox; outbox replicator fan-outs to other cells' subscription managers.

### Per-stack adapter table

| Stack | SSE adapter | WebSocket adapter |
|---|---|---|
| SvelteKit | native `EventSource` | native `WebSocket` |
| Leptos | `gloo-net` SSE | `gloo-net` WebSocket |
| SwiftUI (Apple) | `URLSession.bytes` (HTTP/2 streaming) | `URLSessionWebSocketTask` |
| Compose (Android) | `OkHttp` SSE | `OkHttp` WebSocket |
| GTK 4 (Linux) | `libsoup3` | `libsoup3` WebSocket |
| WinUI 3 (Windows) | `HttpClient` streaming | `MessageWebSocket` |

### Coverage gate

`oya-check-realtime-transport-tier` (advisory) scans every µservice's realtime stream declarations and flags:

1. Bidirectional traffic on SSE → promote to WebSocket.
2. WebSocket on one-way streams → demote to SSE.
3. gRPC streaming on client surface → migrate to WebSocket.
4. Unknown tier labels.

## Alternatives considered

### (a) WebSocket-only — REJECTED

- **Pros:** one protocol.
- **Cons:** SSE is simpler + auto-reconnect from the browser + works through every HTTP intermediary.
- **Rejected**: SSE is genuinely better for one-way.

### (b) gRPC-Web for everything — REJECTED

- **Pros:** unified protobuf wire.
- **Cons:** gRPC-Web requires a proxy translator; doesn't support client-streaming; weak browser-native ergonomics.
- **Rejected**: proxy hop + weaker DX.

### (c) WebTransport (HTTP/3) — DEFERRED

- **Pros:** future-state HTTP transport.
- **Cons:** browser support patchy; spec still moving.
- **Deferred**: revisit post 2027.

### (d) MQTT / NATS for client-facing — REJECTED

- **Pros:** rich pub-sub.
- **Cons:** browser support requires JS client; not a wire protocol the browser speaks natively.
- **Rejected**: not client-native.

### (e) **CHOSEN: SSE + WebSocket + gRPC streaming (service-to-service only)**

- **Pros:** open standards (IETF HTML Living Standard / RFC 6455 / RFC 9114-compatible HTTP/2 for gRPC); browser-native ergonomics; each tier covers exactly its concern; no overlap.
- **Cons:** three transports to operate. Mitigation: closed enum; advisory gate prevents drift.
- **Accepted**.

## Consequences

### Positive

1. **Closed responsibility split.** Each transport owns one pattern.
2. **Open standards.** No vendor lock-in.
3. **Multi-region by design.** No sticky-session anti-pattern.
4. **Resume + reconnect** standardized across the tier.

### Negative

1. **Three transports to operate.** Mitigation: kernel + adapter; CI gate.
2. **No client-streaming on the web surface.** Mitigation: WebSocket covers bidi.

### Operational

- `oya-shared-realtime-transport-kernel` defines the tier enum + envelope.
- Per-stack adapters live under `microservices/observability/clients/realtime-adapter/` + per-µservice adapters as needed.
- Standards doc at `docs/standards/realtime-transport-tier.md`.

## In-house roadmap

**Vendor classification:** WebSocket (RFC 6455, IETF), Server-Sent Events (HTML Living Standard, WHATWG), gRPC (CNCF Incubating, BSD-3) are all **open standards / community-maintained**.

- **No in-house transport rebuild planned.** Building a competing realtime transport would forfeit browser-native ergonomics + IETF interop.
- **What we DO build in-house:**
  - `oya-shared-realtime-transport-kernel` tier model + closed enum.
  - Per-µservice subscription manager (Valkey pub-sub binding).
  - Resume-token issuer + cell-id routing.
  - `oya-check-realtime-transport-tier` advisory gate.

## Rollback

- Per-stream tier rollback: feature-flag the stream's tier label.
- Subscription-manager rollback: Valkey pub-sub binding is per-cell.

## References

- WebSocket — RFC 6455; IETF; 2011.
- Server-Sent Events — WHATWG HTML Living Standard; https://html.spec.whatwg.org/multipage/server-sent-events.html
- gRPC — https://grpc.io ; CNCF Incubating; BSD-3.
- AsyncAPI — https://www.asyncapi.com ; spec 3.0.
- ADR-0145 — inter-microservice communication reform (Loro CRDT pin).
- ADR-0153 — observability backplane high-level reference (outbox / per-cell).
- ADR-0182 — API gateway north-south.
- ADR-0185 — Workflow Studio client stack.
- ADR-0204 — canvas (WebSocket consumer for Loro sync).
- LTS-rotation cadence: standards current as of 2026-05-18; review per ADR-0098.
