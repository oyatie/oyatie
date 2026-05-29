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

## Wave 15 substance conversion

### A. Problem this IP closes

Observability owns one-way operational streams: log tail, metric tail, eligibility verdict transitions, burn-rate updates, dashboard rollups, and long-running AI/operator response streams where the client should not send collaborative edits back on the same channel.
The previous IP named SSE generically, but did not bind it to `SloEngine.StreamEligibilityVerdicts`, OpenAPI read paths, tenant-scoped `X-Scope-OrgID`, cell labels, or the existing telemetry data-class rules in `crates/oya-observability-domain/src/lib.rs`.
This IP closes the product gap against Datadog live tail, New Relic live views, Grafana dashboard refresh, and Honeycomb query streaming while preserving Oyatie's no-vendor-lock observability substrate.

### B. Approach

Implement an SSE gateway adapter that publishes one-way events from the observability app layer to authorized tenant/operator clients.
The adapter must consume domain-safe values, not raw telemetry payloads, and apply `TelemetryLogExposure` rules for redaction/forbid decisions.
Resume uses `Last-Event-ID` with a cell-local cursor store and tenant-scoped stream key.
Every event carries `tenant_id` hash or scope key, `cell_id`, `microservice`, `environment`, `stream_kind`, `cursor`, and data-class exposure metadata.

### C. Deliverables

- Add crate `oya-observability-sse-adapter` or fold it into the existing gateway adapter if the architecture chooses one adapter crate.
- Add SSE endpoint definitions to `microservices/observability/contracts/openapi/slo-engine.yaml` if REST streaming is part of the public contract, or record the contract gap.
- Bind gRPC `StreamEligibilityVerdicts` from `contracts/proto/slo-engine.proto` to SSE where REST clients need one-way verdict streams.
- Add integration tests for connect, heartbeat, reconnect with `Last-Event-ID`, tenant mismatch, payload budget, and cell routing.
- Add runbook link to `microservices/observability/runbooks/realtime-transport-connection-leak.md`.
- Add dashboard/metric counters for active SSE connections, reconnects, dropped payloads, and tenant cap rejections.

### D. Implementation steps

1. Inventory current stream consumers: eligibility verdicts, burn-rate snapshots, log tail, metric tail, and operator response streams.
2. Define `SseStreamKind` with low-cardinality values and map each value to allowed data classes.
3. Implement auth extraction using the same OIDC and `X-Scope-OrgID` rules as `slo-engine.yaml`.
4. Create per-tenant/per-cell cursor keys and reject cross-tenant resume cursors.
5. Emit 30-second comment heartbeat and track missed heartbeat disconnects.
6. Enforce the 50,000 concurrent connection tenant ceiling and a lower configured ceiling for OCI Always Free contexts.
7. Apply `log_exposure_for_classification` and `RedactedTelemetryValue` before serializing payload data.
8. Add backpressure behavior: drop or coalesce metric updates, never drop terminal eligibility verdict transitions.
9. Add tests for forbidden data-class payload rejection and children/PII redaction.
10. Add an operator runbook section for leak, reconnect storm, and cursor-store loss.

### E. Acceptance

- SSE reconnect resumes from the last event without replaying another tenant's event.
- Heartbeats arrive every 30 seconds under normal load.
- Payloads marked `Forbid` by `TelemetryLogExposure` cannot be serialized.
- Tenant connection cap rejection emits a metric and clear error event.
- Live eligibility stream aligns with `StreamEligibilityVerdicts` and REST auth semantics.

### F. Evidence

- `microservices/observability/contracts/openapi/slo-engine.yaml` OIDC and `X-Scope-OrgID` convention.
- `microservices/observability/contracts/proto/slo-engine.proto` `StreamEligibilityVerdicts`.
- `crates/oya-observability-domain/src/lib.rs` telemetry fields and log exposure rules.
- `microservices/observability/runbooks/realtime-transport-connection-leak.md`.
- `microservices/observability/competitor-parity-matrix.md` Grafana/Datadog/New Relic/Honeycomb counterparts.

### G. Counterpart closure

| Counterpart | One-way stream expectation | This IP closure |
|---|---|---|
| Datadog | live log tail and monitor state updates | tenant-scoped log/metric/verdict SSE streams |
| New Relic | live service-level and alert views | eligibility and burn-rate SSE stream |
| Grafana | dashboard refresh and alert state stream | low-cardinality dashboard update stream |
| Honeycomb | query/result streaming without vendor lock | domain-safe streamed result events |
| GitHub | Actions promotion status stream | eligibility verdict SSE can feed release-gate UI without polling |

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/observability/IP-026-sse-transport-impl.md` matched `openapi, .proto`; contract files `microservices/observability/contracts/openapi/slo-engine.yaml, microservices/observability/contracts/asyncapi/eligibility-events.yaml, microservices/observability/contracts/proto/slo-engine.proto`; type anchor `crates/oya-cloud-observability-api/src/lib.rs::CloudObservabilityAuditRecord`.
