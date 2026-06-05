---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02b-substrate-ready
phase: P01-durable-execution-substrate
impl_plan_id: IP-012-replay-debugger-backend-rest-sdk-app
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, openapi-conformance]
---

# IP-012: replay-debugger REST, SDK, streaming, and app root

## §A Problem

The replay debugger backend is useful only if Studio, SDK users, and SRE tooling can start a replay, stream snapshots, and fetch analytics through a stable tenant-scoped interface. The stamped IP said "REST, SDK, app" but did not bind routes, stream semantics, read-only auditor policy, or operational startup configuration.

This IP closes the outer-surface gap for the replay debugger: REST and streaming endpoints must expose the usecase from IP-011 without allowing auditors or tenant operators to mutate runs, leak payloads, or start unbounded replay work.

## §B Approach

Create `oya-workflow-engine-replay-debugger-backend-rest`, `sdk`, and `app`. REST implements OpenAPI `/runs/{run_id}/replay`, replay-session reads, snapshot streaming bootstrap, and analytics endpoints. The SDK wraps replay-session lifecycle and snapshot streaming for Studio and tenant tooling. The app root composes REST, usecase ports, Postgres, ClickHouse, Cedar bundles, OpenBao references, and metrics exporters.

Where gRPC streaming is required, use the proto `ReplayDebuggerBackend.StreamReplaySteps` contract as the internal streaming authority and expose REST-compatible cursor paging for browser clients.

Non-goals are explicit: this IP does not create new replay algorithms or analytics SQL. It only exposes the IP-010/IP-011 backend through a bounded, policy-checked surface.
It also does not mutate workflow runs; every route is inspect, stream, or aggregate-read only.

## §C Deliverables

| Artifact | Action | Substance requirement |
|---|---|---|
| `microservices/workflow-engine/src/crates/oya-workflow-engine-replay-debugger-backend-rest/src/routes.rs` | create | replay start, replay session get, snapshot page/stream bootstrap, analytics read |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-replay-debugger-backend-rest/src/stream.rs` | create | server-side stream over `StepSnapshot` with cursor resume and cancellation handling |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-replay-debugger-backend-rest/src/middleware.rs` | create | auditor read-only scope, tenant debugger entitlement, payload redaction enforcement |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-replay-debugger-backend-sdk/src/client.rs` | create | typed replay client for start/get/stream/analytics |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-replay-debugger-backend-sdk/src/models.rs` | create | replay DTOs aligned with OpenAPI/proto without stringly status fields |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-replay-debugger-backend-app/src/main.rs` | create | app composition root and startup config validation |
| `microservices/workflow-engine/catalog/oya-workflow-engine-replay-debugger-backend-{rest,sdk,app}.yaml` | update/create | catalog rows for all three crates |

## §D Implementation

1. Implement REST route mapping for the OpenAPI replay operation and analytics responses, including `run_id`, `from_step`, `to_step`, cursor, and bounded page size.
2. Implement stream resume semantics: clients can reconnect with the last `event_cursor` and receive only later snapshots.
3. Enforce `policy/auditor-scope.cedar` and `policy/tenant-scope.cedar`: auditors can inspect snapshots and analytics but cannot trigger side-effecting run control.
4. Add payload redaction at the REST boundary so `SECRET` payloads remain hashed-only, matching IP-010 snapshot requirements.
5. Build the SDK client used by Studio to start replay, poll session status, consume snapshot streams, and request aggregate run analytics.
6. Wire the app crate with Postgres, ClickHouse, Cedar, OpenBao, and metrics config checks; fail startup when any required read path is missing.
7. Emit HTTP and stream metrics: replay start latency, snapshot stream lag, denied replay attempts, analytics query duration, and stream reconnect count.

## §E Acceptance

- `cargo nextest run -p oya-workflow-engine-replay-debugger-backend-rest --all-features`
- `cargo nextest run -p oya-workflow-engine-replay-debugger-backend-sdk --all-features`
- `cargo nextest run -p oya-workflow-engine-replay-debugger-backend-app --all-features`
- `buck2 build //:quality-lane-registry-authority-check # lane=openapi-conformance --crate oya-workflow-engine-replay-debugger-backend-rest`
- Required tests: `rest_start_replay_returns_session`, `stream_reconnect_resumes_after_cursor`, `auditor_scope_is_read_only`, `tenant_operator_without_debugger_entitlement_denied`, `secret_snapshot_payload_redacted`, and `sdk_stream_preserves_snapshot_order`.

## §F Evidence

- `microservices/workflow-engine/contracts/openapi/workflow-engine.yaml` defines replay initiation and run-read surfaces.
- `microservices/workflow-engine/contracts/proto/workflow-engine.proto` defines `StreamReplaySteps`.
- `microservices/workflow-engine/policy/auditor-scope.cedar` defines read-only audit access; `policy/tenant-scope.cedar` defines replay debugger entitlement.
- `microservices/workflow-engine/runbooks/durable-execution-history-replay.md` is the operational consumer for replay sessions.
- `microservices/workflow-engine/slos/replay-determinism-correctness.openslo.yaml` is the correctness proof target exposed through this surface.

## §G Counterparts

| Counterpart | Relevant behavior | This IP closes |
|---|---|---|
| Temporal Web | operators inspect workflow history and replay-related state through stable APIs | REST/SDK exposes replay sessions and ordered snapshots |
| Camunda Operate | process instance inspection is browser-facing but policy-scoped | auditor and debugger middleware keep inspection read-only |
| AWS Step Functions Console | execution history is paged and resumable | cursor paging and stream reconnect make large histories inspectable |
| n8n executions UI | users can inspect executions but not deterministic replay | backend stream carries replayed snapshots, not only original logs |

## Next IP

[`IP-013-observability-slo-manifests.md`](IP-013-observability-slo-manifests.md)

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/workflow-engine/contracts/openapi/workflow-engine.yaml`, `microservices/workflow-engine/contracts/asyncapi/workflow-events.yaml`, `microservices/workflow-engine/contracts/proto/workflow-engine.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.
