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

## Wave 15 substance conversion

### A. Problem this IP closes

Observability needs bidirectional realtime channels only where operators or product surfaces send control input back: collaborative incident dashboards, operator annotations, dashboard-share sessions, and presence-backed debugging rooms.
The earlier shell incorrectly framed WebSocket mainly around Workflow Studio and chat without explaining observability's own bidirectional use cases or boundaries.
This IP closes the WebSocket path for observability-owned operator sessions while keeping canvas/chat domain logic in their own product services.

### B. Approach

Expose WebSocket as a gateway adapter for observability session channels with protobuf-framed messages, tenant/cell scope, resume token, heartbeat, and policy checks.
Allowed inbound messages are narrow: subscribe/unsubscribe, ack cursor, operator annotation, dashboard-share pointer, and incident-room presence ping.
The adapter must not accept arbitrary telemetry writes or product chat messages.
Outbound messages include burn-rate updates, alert state, cursor ack, presence delta, and operator annotation echo.

### C. Deliverables

- Add crate `oya-observability-websocket-adapter` or a documented gateway module inside an existing adapter crate.
- Add AsyncAPI or OpenAPI extension documentation for WebSocket message types if missing.
- Add protobuf message definitions or a local framing schema for subscribe, ack, annotation, presence, and alert update.
- Add session-resume token issuance and validation.
- Add integration tests for handshake, authz, heartbeat, resume, cross-tenant rejection, payload frame limit, and unsupported inbound message rejection.
- Link runbook `microservices/observability/runbooks/realtime-transport-connection-leak.md`.

### D. Implementation steps

1. Identify observability-owned bidirectional sessions and exclude Workflow Studio canvas/chat behavior from this IP.
2. Define `ObservabilityRealtimeChannel` values: gate eligibility, incident dashboard, dashboard share, tail-sampling debug, and cell health if present.
3. Implement OIDC plus `X-Scope-OrgID` validation at handshake.
4. Issue resume token bound to tenant, principal, cell, channel, expiry, and protocol version.
5. Add 30-second ping/pong heartbeat and stale-session eviction.
6. Add protobuf frame envelope with max payload size and message kind enum.
7. Reject inbound telemetry samples; telemetry ingestion remains OTel/Alloy, not WebSocket.
8. Persist operator annotations through a usecase port that emits audit events.
9. Emit connection metrics by tenant class, channel, cell, and close reason.
10. Add load test for the 10,000 concurrent tenant ceiling and lower profile limits for OCI guest contexts.

### E. Acceptance

- WebSocket accepts only documented observability channel messages.
- Cross-tenant resume token reuse fails.
- Unsupported inbound telemetry write returns protocol error and closes the connection.
- Operator annotation emits audit evidence.
- Connection leak runbook has metrics needed to identify stale sessions.

### F. Evidence

- `microservices/observability/contracts/openapi/slo-engine.yaml` auth and tenant-scope convention.
- `microservices/observability/contracts/proto/slo-engine.proto` service state types.
- `microservices/observability/dashboards/operator-burn-rate.json` and `gate-eligibility.json`.
- `microservices/observability/runbooks/realtime-transport-connection-leak.md`.
- `microservices/observability/feature-parity-matrix-2026-05-20.md` Grafana/Datadog/New Relic realtime and dashboard expectations.

### G. Counterpart closure

| Counterpart | Bidirectional expectation | This IP closure |
|---|---|---|
| Grafana | shared dashboards and live operator state | dashboard-share WebSocket channel with annotations |
| Datadog | incident/monitor collaboration surfaces | incident dashboard channel and audit-backed annotations |
| New Relic | collaborative alert/entity exploration | scoped channel subscriptions and cursor ack |
| Slack | incident room presence, but not chat ownership | presence/annotation only; chat remains out of scope |

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/observability/IP-027-websocket-transport-impl.md` matched `openapi, .proto`; contract files `microservices/observability/contracts/openapi/slo-engine.yaml, microservices/observability/contracts/asyncapi/eligibility-events.yaml, microservices/observability/contracts/proto/slo-engine.proto`; type anchor `crates/oya-cloud-observability-api/src/lib.rs::CloudObservabilityAuditRecord`.
