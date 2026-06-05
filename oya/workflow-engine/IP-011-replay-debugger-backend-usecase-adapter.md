---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02b-substrate-ready
phase: P01-durable-execution-substrate
impl_plan_id: IP-011-replay-debugger-backend-usecase-adapter
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, tenant-isolation, clickhouse-query-safety]
---

# IP-011: replay-debugger usecase, API, Postgres reader, and ClickHouse analytics

## §A Problem

IP-010 makes replay pure, but the product still needs a safe way to start replay sessions, page event history, read checkpoints, and serve analytics without leaking tenant data or making ClickHouse an authority for workflow state. The stamped IP listed five crates but did not describe the split between authoritative Postgres event/run data and analytics-only ClickHouse replicas.

This IP closes the adapter boundary: SREs and Studio need replay sessions and run analytics; the implementation must read tenant-scoped durable history, run the pure replay engine, store replay-session metadata, and query ClickHouse only for aggregate analytics.

## §B Approach

Create the usecase, api, adapter, adapter-postgres, and adapter-clickhouse crates declared in `microservices/workflow-engine/manifest.json`. The usecase layer owns `ReplaySessionOrchestrator` and `AnalyticsOrchestrator`. Postgres adapters implement event-log, checkpoint, and replay-session stores over the authoritative runtime tables produced by execution-engine and state-machine IPs. ClickHouse adapters expose read-only aggregate query ports for latency, failure, and throughput analytics.

Tenant predicates are mandatory on both storage paths. ClickHouse rows are replicas for analytics; any replay snapshot or correctness decision must come from Postgres event/checkpoint reads plus the pure domain engine.

Non-goals are explicit: this IP does not add REST routes or SDK models. It prepares the usecase and storage boundary that IP-012 can expose without owning query safety itself.
It also does not make ClickHouse a recovery source for workflow history.

## §C Deliverables

| Artifact | Action | Substance requirement |
|---|---|---|
| `microservices/workflow-engine/src/crates/oya-workflow-engine-replay-debugger-backend-usecase/src/replay_session.rs` | create | starts replay session, validates entitlement, pages history, invokes domain replay, stores status |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-replay-debugger-backend-usecase/src/analytics.rs` | create | tenant-scoped run analytics orchestration with query limits |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-replay-debugger-backend-api/src/types.rs` | create | typed replay request/session/snapshot/analytics DTOs aligned to OpenAPI/proto |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-replay-debugger-backend-adapter-postgres/src/lib.rs` | create | authoritative event-log and checkpoint readers plus replay-session store |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-replay-debugger-backend-adapter-clickhouse/src/lib.rs` | create | read-only aggregate analytics queries with tenant predicate injection |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-replay-debugger-backend-adapter/src/lib.rs` | create | protocol-neutral mappers and fake adapters for tests |
| `microservices/workflow-engine/catalog/oya-workflow-engine-replay-debugger-backend-*.yaml` | update/create | rows for all five crates |

## §D Implementation

1. Implement `ReplaySessionOrchestrator::start` to authorize `replay_workflow_run` or `step_through_run` using `policy/tenant-scope.cedar` before any event read.
2. Read authoritative event log pages from Postgres by `(tenant_id, run_id, event_cursor)` and checkpoint pages by `(tenant_id, run_id, checkpoint_seq)`.
3. Invoke the IP-010 domain replay engine and persist replay-session status, snapshot cursor, mismatch summary, and audit id.
4. Implement `AnalyticsOrchestrator::get_run_analytics` with explicit tenant, run, time-window, and aggregation bounds; reject unbounded scans.
5. Build ClickHouse queries with tenant predicate injection and no string concatenation; query outputs are aggregate metrics only, never raw SECRET payloads.
6. Add fake adapters for usecase tests so replay behavior is validated without Postgres or ClickHouse.
7. Emit audit-chain events for replay start, replay complete, replay mismatch, analytics query, and entitlement denial.

## §E Acceptance

- `cargo nextest run -p oya-workflow-engine-replay-debugger-backend-usecase --all-features`
- `cargo nextest run -p oya-workflow-engine-replay-debugger-backend-adapter-postgres --all-features`
- `cargo nextest run -p oya-workflow-engine-replay-debugger-backend-adapter-clickhouse --all-features`
- `buck2 build //:quality-lane-registry-authority-check # lane=layer-correctness --crate oya-workflow-engine-replay-debugger-backend-usecase`
- Required tests: `replay_session_denies_missing_debugger_entitlement`, `postgres_event_log_read_requires_tenant_predicate`, `checkpoint_pages_replay_in_order`, `clickhouse_query_injects_tenant_predicate`, `clickhouse_never_returns_raw_payload`, and `replay_mismatch_emits_audit_event`.

## §F Evidence

- `microservices/workflow-engine/contracts/openapi/workflow-engine.yaml` defines replay and metrics endpoints.
- `microservices/workflow-engine/contracts/proto/workflow-engine.proto` defines `ReplayDebuggerBackend` RPCs.
- `microservices/workflow-engine/policy/auditor-scope.cedar` and `policy/tenant-scope.cedar` define read-only debugger and same-tenant access requirements.
- `microservices/workflow-engine/runbooks/durable-execution-history-replay.md` and `runbooks/saga-compensation-failure-investigation.md` are operational consumers.
- `microservices/workflow-engine/dashboards/step-latency.json` and `dashboards/workflow-execution-rate.json` are analytics surface anchors.

## §G Counterparts

| Counterpart | Relevant behavior | This IP closes |
|---|---|---|
| Temporal Web / Cloud UI | replay/debug reads durable history while metrics are separate from history authority | Postgres is replay authority; ClickHouse is analytics-only |
| Camunda Operate | process-instance inspection is tenant-filtered and incident-focused | replay sessions and aggregate analytics are entitlement-gated |
| AWS Step Functions | execution history and CloudWatch metrics are distinct surfaces | event-log reads are separate from ClickHouse aggregate queries |
| Datadog / OpenTelemetry-style analytics | metrics need cardinality and query bounds | analytics orchestrator enforces tenant and time-window bounds |

## Next IP

[`IP-012-replay-debugger-backend-rest-sdk-app.md`](IP-012-replay-debugger-backend-rest-sdk-app.md)

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/workflow-engine/contracts/openapi/workflow-engine.yaml`, `microservices/workflow-engine/contracts/asyncapi/workflow-events.yaml`, `microservices/workflow-engine/contracts/proto/workflow-engine.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.
