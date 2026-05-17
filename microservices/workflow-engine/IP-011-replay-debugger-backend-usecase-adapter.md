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
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness]
---

# IP-011: oya-workflow-engine-replay-debugger-backend-{usecase,api,adapter,adapter-postgres,adapter-clickhouse}

## Intent

Replay-debugger-backend usecase (orchestrators) + api (typed I/O) + adapter + Postgres adapter (read-side over run state) + ClickHouse adapter (analytics queries).

## ChangeSet boundary

5 new crates.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `src/crates/oya-workflow-engine-replay-debugger-backend-usecase/{...}` | create | ReplaySessionOrchestrator, AnalyticsOrchestrator |
| `src/crates/oya-workflow-engine-replay-debugger-backend-api/{...}` | create | typed I/O |
| `src/crates/oya-workflow-engine-replay-debugger-backend-adapter/{...}` | create | protocol-neutral impls |
| `src/crates/oya-workflow-engine-replay-debugger-backend-adapter-postgres/{...}` | create | read-side over run state + outbox |
| `src/crates/oya-workflow-engine-replay-debugger-backend-adapter-clickhouse/{...}` | create | analytics queries over ClickHouse replica |
| `microservices/workflow-engine/catalog/oya-workflow-engine-replay-debugger-backend-*.yaml` | create | 5 catalog rows |

## Test Plan

| Test | Verifies |
|---|---|
| `test_replay_session_orchestrator_happy` | session created, replayed, sealed |
| `test_clickhouse_analytics_query_tenant_partitioned` | tenant_id partition predicate enforced |
| `test_postgres_event_log_read_paginated` | large event logs paginate correctly |

## Next IP

[`IP-012-replay-debugger-backend-rest-sdk-app.md`](IP-012-replay-debugger-backend-rest-sdk-app.md)

## References

- PRD §"Bounded Contexts" replay-debugger-backend row
- `backfill-replay.md`
- ClickHouse — `clickhouse.com/docs/`
