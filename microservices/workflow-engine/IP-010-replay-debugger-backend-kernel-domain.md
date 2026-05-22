---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02b-substrate-ready
phase: P01-durable-execution-substrate
impl_plan_id: IP-010-replay-debugger-backend-kernel-domain
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, oya-governance-deterministic-replay]
---

# IP-010: replay-debugger kernel and deterministic replay domain

## §A Problem

Replay debugging is a first-class workflow-engine product capability, not a log viewer. The OpenAPI route `/runs/{run_id}/replay` and proto `ReplayDebuggerBackend` service promise replay sessions, step snapshots, and run analytics. The original short IP named `ReplayEngine` but did not bind it to event-log shape, step snapshots, tenant-scoped reads, or Temporal-class determinism.

This IP closes the backend core for a Studio replay view and for SRE incident review: given a durable event log and a step range, the debugger must reconstruct the same state transitions without reading live mutable worker state or issuing side effects.

## §B Approach

Create `oya-workflow-engine-replay-debugger-backend-kernel` and `domain`. Kernel owns replay entities and sealed read ports. Domain owns pure replay over workflow events and checkpoint records. It consumes event types declared in `microservices/workflow-engine/contracts/asyncapi/workflow-events.yaml`, run/step identities from `contracts/proto/workflow-engine.proto`, and spec-version references from the spec-store contract. It must be usable by IP-011 without importing Postgres, ClickHouse, REST, or SDK crates.

Non-goals are explicit: this IP does not query storage, expose HTTP, or render Studio UI. It creates the replay math that later surfaces can call without smuggling adapter behavior into the domain layer.
It also does not define ClickHouse analytics queries.
Those stay in IP-011 so replay correctness remains independent of analytics storage.

## §C Deliverables

| Artifact | Action | Substance requirement |
|---|---|---|
| `microservices/workflow-engine/src/crates/oya-workflow-engine-replay-debugger-backend-kernel/src/entities.rs` | create | `ReplaySession`, `ReplayRange`, `StepSnapshot`, `ReplayDiff`, `RunAnalytics`, `EventCursor` |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-replay-debugger-backend-kernel/src/ports.rs` | create | sealed `EventLogReader`, `CheckpointReader`, `ReplaySessionStore`, `RunAnalyticsRepository` |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-replay-debugger-backend-domain/src/replay.rs` | create | deterministic replay from event stream plus pinned spec metadata |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-replay-debugger-backend-domain/src/diff.rs` | create | field-by-field original-vs-replayed snapshot comparison |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-replay-debugger-backend-domain/src/range.rs` | create | validates `from_step` and `to_step`, cursor paging, and bounded payload rules |
| `microservices/workflow-engine/catalog/oya-workflow-engine-replay-debugger-backend-{kernel,domain}.yaml` | update/create | catalog rows for both crates |
| `Cargo.toml` | update | workspace members for both crates |

## §D Implementation

1. Define replay identity as `(tenant_id, run_id, replay_session_id, spec_id, version_sha, replay_range)` so replay cannot drift to a newer spec version.
2. Define `StepSnapshot` with `state_before`, `event`, `state_after`, `step_index`, `event_cursor`, `checkpoint_seq`, and `payload_hash`; do not embed raw SECRET payloads.
3. Implement `ReplayEngine::replay(events, checkpoints, range)` as pure code that reconstructs snapshots in event order and refuses gaps, duplicate cursors, or illegal terminal-state transitions.
4. Implement `verify_identical(original, replayed)` as a structured diff that reports the first mismatching field, not a boolean-only failure.
5. Enforce range limits from `policy/spec-integrity.md` payload-size and bounded execution rules; partial replay must never read unbounded history into memory.
6. Add property tests for event ordering, duplicate event refusal, replay determinism, partial range bounds, and snapshot diff reporting.
7. Keep analytics entities in the kernel but leave ClickHouse query implementation to IP-011 so domain remains I/O-free.

## §E Acceptance

- `cargo check -p oya-workflow-engine-replay-debugger-backend-kernel --all-features`
- `cargo check -p oya-workflow-engine-replay-debugger-backend-domain --all-features`
- `cargo nextest run -p oya-workflow-engine-replay-debugger-backend-domain --all-features`
- `cargo run -p oya-dev-cli -- gate validate port-location --crate oya-workflow-engine-replay-debugger-backend-kernel`
- `cargo run -p oya-dev-cli -- gate validate layer-correctness --crate oya-workflow-engine-replay-debugger-backend-domain`
- Required tests: `replay_same_event_log_same_snapshots`, `replay_refuses_cursor_gap`, `partial_replay_respects_bounds`, `diff_reports_first_mismatch`, `secret_payload_hash_only`, and `replay_does_not_import_adapter_crates`.

## §F Evidence

- `microservices/workflow-engine/contracts/openapi/workflow-engine.yaml` defines `/runs/{run_id}/replay`.
- `microservices/workflow-engine/contracts/proto/workflow-engine.proto` defines `ReplayDebuggerBackend`, `ReplaySession`, `StepSnapshot`, and analytics RPCs.
- `microservices/workflow-engine/contracts/asyncapi/workflow-events.yaml` defines the lifecycle event stream that replay consumes.
- `microservices/workflow-engine/backfill-replay.md` and `runbooks/durable-execution-history-replay.md` define operational replay needs.
- `microservices/workflow-engine/slos/replay-determinism-correctness.openslo.yaml` is the correctness SLO this IP enables.

## §G Counterparts

| Counterpart | Relevant behavior | This IP closes |
|---|---|---|
| Temporal / Cadence | deterministic replay over event history is the core debugging primitive | pure replay engine over ordered workflow lifecycle events |
| Camunda Operate / Cockpit | incident review needs step-by-step process state visibility | `StepSnapshot` and structured diffs supply backend evidence for Studio |
| AWS Step Functions | execution history can be inspected by event id and state transition | cursor-bounded replay and field-level mismatch reporting |
| Airflow | task logs help diagnosis but do not reconstruct deterministic state | replay snapshots reconstruct state, not just logs |

## Next IP

[`IP-011-replay-debugger-backend-usecase-adapter.md`](IP-011-replay-debugger-backend-usecase-adapter.md)

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/workflow-engine/contracts/openapi/workflow-engine.yaml`, `microservices/workflow-engine/contracts/asyncapi/workflow-events.yaml`, `microservices/workflow-engine/contracts/proto/workflow-engine.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/workflow-engine/IP-010-replay-debugger-backend-kernel-domain.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/workflow-engine/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/workflow-engine/slos/payload-bytes-budget-correctness.openslo.yaml`, `microservices/workflow-engine/slos/replay-determinism-correctness.openslo.yaml`, `microservices/workflow-engine/slos/worker-poll-availability.openslo.yaml`, `microservices/workflow-engine/slos/workflow-completion-availability.openslo.yaml`, `microservices/workflow-engine/policy/auditor-scope.cedar`.
