---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-workflow-engine
microservice: workflow-engine
status: Accepted
sales_segment: shared-substrate
tier: internal
milestone_first_ship: M02b-substrate-ready
bominal_source:
  - ADR-0035   # Workflow engine (hybrid state machine + DAG)
  - ADR-0103   # Workflow hexagonal migration
  - ADR-0148   # Workflow engine (extended)
  - ADR-0028   # Audit chain
  - ADR-0107   # Agent gateway (agentic mode nodes)
  - ADR-0037   # Plugin substrate (WASM custom nodes)
  - ADR-0009   # Cell architecture
  - ADR-0019   # Runtime target metadata model
related_adrs: [ADR-0056, ADR-0103, ADR-0105, ADR-0110, ADR-0123, ADR-0139, ADR-0131]
related_specs: [/specs/microservices/workflow.json, /specs/per-microservice-flat-layout.json]
related_unbundle_adr: ADR-0131
unbundle_sibling: microservices/workflow-studio/
date: 2026-05-17
owner_team: axis-workflow
doc_status: published
---

# PRD-workflow-engine: Workflow Execution Substrate

## Purpose

The `workflow-engine` µservice is oyatie's **durable workflow execution substrate**. It owns the state machine + DAG runtime; the durable-execution layer (Temporal-class semantics: deterministic replay, multi-week run lifetimes, crash-safe step persistence); the event bus that carries workflow events between µservices; the spec-store that holds compiled workflow definitions; and the replay/debugger backend that powers post-hoc inspection of completed and in-flight runs.

This µservice is the **substrate half** of the workflow product unbundle (ADR-0131). The visual editor — drag-drop canvas, node library UX, template gallery — lives in the sibling `workflow-studio` µservice. The engine has **zero UI surface**; its consumers are: the Studio editor (via REST + gRPC); every other oyatie µservice (via Workflow events; this µservice is the cross-product orchestration adapter per `feedback_workflow_objectgraph_adapter_layer.md`); and tenant workloads that embed the engine SDK for in-process triggers.

This µservice is **shared substrate**, not a hero product. It is consumed by every other oyatie µservice. Direct product-to-product calls are prohibited (LEAN-A2); the engine is the load-bearing adapter that routes typed events between µservices via state machines + DAGs.

This µservice inherits Bominal ADRs 0035, 0103, 0148, 0028, 0107, 0037, 0009, 0019. Bominal's Corporate-ownership stance overridden per `feedback_workflow_is_shared.md`: workflow-engine is shared/* and exists in oyatie under the per-microservice flat layout (ADR-0131).

## Tenant Value

- **Tenant Outcome 1 — Durable execution.** Workflow runs survive process restarts, pod evictions, and region-level events; state persisted to Postgres before every step; deterministic replay reconstructs run history identically. Tenants never lose a run.
- **Tenant Outcome 2 — Sub-second event-to-action latency.** Outbox → event bus → worker → step dispatch p99 ≤ 500ms; step execution start p99 ≤ 200ms; meets the n8n/Make UX expectation while exceeding their durability story (which is typically best-effort).
- **Tenant Outcome 3 — 10k+ concurrent runs per cell.** Horizontally shardable workers + per-tenant Postgres Citus partitioning + cross-cell Kafka bridge. Linearly scales to 100k+ aggregate runs without architectural change.
- **Tenant Outcome 4 — Audit-sealed run history.** Every run sealed with Ed25519 + Merkle audit-chain per Bominal ADR-0028; sealed within 1s of completion; tamper-evident.
- **Internal Outcome 5 — Cross-µservice orchestration adapter.** All inter-µservice action flows route through this engine. HR `EmployeeHired` → workflow-engine → fan-out to payroll-enrollment, connect-provisioning, etc. Eliminates direct product-to-product coupling.
- **Internal Outcome 6 — Replay-capable debugger.** Any in-flight or completed run can be re-played in a debugger view: step-by-step state inspection, edge condition replay, payload mutation testing.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | workflow author (via Studio) | to submit a compiled workflow spec to the engine | the engine can durably store + version + execute it | spec-store | Must |
| FR-02 | workflow author | to receive a deterministic `workflow_version_sha` after spec compile | I can pin specific tenant deployments to specific versions | spec-store | Must |
| FR-03 | µservice (any) | to publish a typed workflow event onto the event bus | downstream consumers (other µservices, Studio replay, audit) receive it | event-bus | Must |
| FR-04 | µservice (any) | to subscribe to one or more event types with backpressure-aware delivery | I can drive my own internal state from workflow signal | event-bus | Must |
| FR-05 | engine worker | to claim the next step of a run, execute it, and atomically persist new state | runs are crash-safe with exactly-once-effect semantics on a per-step basis | execution-engine | Must |
| FR-06 | engine worker | to dispatch a step to an external integration with retry + backoff per the spec's retry policy | transient failures handled automatically per Workato/Temporal parity | execution-engine | Must |
| FR-07 | engine worker | to deterministically replay a run from its event log starting at any checkpoint | audit + debug + retroactive bug-fix scenarios are supported | replay-debugger-backend | Must |
| FR-08 | state machine | to validate transitions against the spec's declared invariants (e.g. four-eyes constraint on approvals) | invalid state mutations are refused at write time | state-machine | Must |
| FR-09 | timer | to arm an SLA timer at step entry, cancel on completion, fire escalation on breach | SLA enforcement is built into engine, not bolted on | execution-engine | Must |
| FR-10 | sub-workflow caller | to invoke a child workflow as a step with parent-child correlation + return value | composable workflow runs (DRY) | execution-engine | Must |
| FR-11 | rest consumer (Studio) | to fetch the current state of any run | live monitoring + debugger | replay-debugger-backend | Must |
| FR-12 | rest consumer (Studio) | to pause, resume, cancel, or signal a run | operator intervention on long-running workflows | execution-engine | Must |
| FR-13 | event consumer | to replay events from a saved checkpoint to a target time | backfill of subscriber state | event-bus | Must |
| FR-14 | tenant operator | to fetch metrics on runs (count, p50/p99 step latency, failure rate) per (tenant, workflow_id) | operational visibility | execution-engine | Must |
| FR-15 | engine | to refuse new run starts when a tenant is over their per-tenant budget | fair-share isolation | execution-engine | Must |

## Non-Functional Requirements

### Performance

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Workflow spec save → trigger-ready | 20ms | 100ms | 250ms | end-to-end compile + register |
| Workflow execution start → first step | 50ms | 200ms | 500ms | engine dispatch latency |
| Step execution (local action) | 5ms | 50ms | 100ms | excludes network-bound external calls |
| Step execution (external HTTP) | — | 200ms | 1s | network-bound; measured separately, not part of core SLO |
| Event-to-action latency (outbox → bus → worker) | 50ms | 500ms | 1s | full pipeline |
| State persistence (per-step write) | 5ms | 25ms | 100ms | Postgres single-row write |
| Replay throughput | — | 1000 steps/s/worker | — | deterministic replay; CPU-bound |
| Concurrent active runs per cell | — | 10,000 | — | baseline; sharding to 100k+ aggregate |
| Run durability ceiling | — | 90 days paused-in-place; weeks active | — | Temporal parity |
| Audit chain seal per run | — | 1s | — | Ed25519 per (tenant, run_id); ADR-0028 |
| Cold-start of worker pod | — | 500ms | 1s | pre-warmed pool of 10 standby |

### Security

- JWT `tenant_id` enforced at every REST/gRPC entry; engine refuses runs without resolvable tenant.
- Per-tenant workflow library; per-tenant run isolation; per-tenant event-bus topic namespace.
- Workflow specs are versioned + signed (Ed25519); spec tampering detected on read.
- Plugin substrate per ADR-0037: custom nodes run in Wasmtime sandbox; no host filesystem access; memory + CPU bounded per execution.
- Replay-attack window on inbound webhooks (when Studio routes them through engine): ≤5min via HMAC-SHA256 signature + nonce.
- Cedar policy fragments gate which roles can submit specs, start runs, pause/resume, cancel, replay.
- Audit-chain emission on every run start, every state transition, every operator intervention.

### Audit + Compliance

- Every (tenant_id, run_id) sealed via Merkle + Ed25519 per Bominal ADR-0028; seal latency p99 ≤ 1s.
- Deterministic replay: given identical initial state + event log, replay produces identical step sequence (Temporal parity; required for audit and debugging).
- Per-tenant `jurisdiction_code` inherited per ADR-0117; runs pinned to pack region; cross-pack run state replication forbidden by default.
- PII fields in step payloads encrypted at rest (ciphertext property type per ADR-0111); `data_class` annotations enforced at every entity field.

### Availability + SLO

- Availability target: 99.95% monthly for execution path (the engine must be available even when individual µservices it gates are degraded).
- 99.9% for replay/debugger backend (read-side).
- RTO ≤ 15s per-cell; RPO ≤ 5s (outbox durability).
- Self-observability: engine emits its own SLO via the observability µservice; burn-rate alarms feed Grafana OnCall.

### Data residency

- Workflow specs, run state, and per-tenant event logs inherit the tenant's `jurisdiction_code` per ADR-0117. Postgres Citus is partitioned by tenant_id and physically pinned to the pack region.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase` rename for new crates), layers used by this µservice are: `kernel`, `domain`, `usecase`, `api`, `adapter`, `rest`, `worker`, `sdk`, `app`. The `state-machine` and `execution-engine` BCs include backend-qualified `*-adapter-<backend>` crates per ADR-0105 Amendment 3 (Postgres for state; Redis for ephemeral; ClickHouse for analytics replay).

| BC | Crate family (BNF v4.1 + ADR-0105) | Purpose | Key entities |
|---|---|---|---|
| `spec-store` | `oya-workflow-engine-spec-store-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,sdk,app}` | Compiled workflow spec storage; version history; signature verification; hot-reload | `WorkflowSpec`, `SpecVersion`, `SpecSignature` |
| `execution-engine` | `oya-workflow-engine-execution-engine-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-redis,rest,worker,sdk,app}` | Run dispatch, step execution, retry, timer, sub-workflow invocation | `WorkflowRun`, `StepExecution`, `RetryAttempt`, `SlaTimer` |
| `state-machine` | `oya-workflow-engine-state-machine-{kernel,domain,usecase,api,adapter,adapter-postgres}` | Transition validation; invariant enforcement; durable state checkpoints | `Transition`, `TransitionRule`, `StateCheckpoint` |
| `event-bus` | `oya-workflow-engine-event-bus-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-redis,rest,worker,sdk,app}` | Typed event publish/subscribe; outbox; replay-from-offset; backpressure | `WorkflowEvent`, `EventOffset`, `Subscription` |
| `replay-debugger-backend` | `oya-workflow-engine-replay-debugger-backend-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-clickhouse,rest,sdk,app}` | Deterministic replay over event log; debugger-state reconstruction; analytics queries | `ReplaySession`, `StepSnapshot`, `RunAnalytics` |

Naming justification — `spec-store`:

```
NAME: oya-workflow-engine-spec-store-<layer>
JUSTIFICATION:
- microservice = workflow-engine: this µservice (per-microservice flat layout, ADR-0131).
  Engine half of the workflow unbundle; studio is sibling µservice at microservices/workflow-studio/.
- bc-tokens = spec-store: primary BC for compiled workflow spec storage (distinct from execution-engine,
  state-machine, event-bus, replay-debugger-backend). ADR-0056 v4.1 BC-optionality rule honoured (4 sibling
  BCs exist, justifying explicit BC token).
- layer = <layer>: one crate per layer per ADR-0105 13-value canonical enum.
  - kernel: port-trait + sealed-trait + entity types (WorkflowSpec, SpecVersion, SpecSignature). Zero I/O.
  - domain: pure spec compilation, validation, semver-comparison logic; deterministic.
  - usecase (per ADR-0106): orchestrators reading + writing specs via ports.
  - api: protocol-neutral typed I/O contracts.
  - adapter: protocol-neutral implementations.
  - adapter-postgres: backend-qualified (per ADR-0105 Amendment 3); Postgres-backed spec storage with
    semver indexing + signature column.
  - rest: HTTP handler/route layer.
  - sdk: client library for tenant-side spec submission.
  - app: composition-root binary.
- exemptions claimed: none.
```

Naming justification — `execution-engine`:

```
NAME: oya-workflow-engine-execution-engine-<layer>
JUSTIFICATION:
- microservice = workflow-engine.
- bc-tokens = execution-engine: primary BC for run dispatch, step execution, retry, timer logic.
- layer = <layer>: one crate per layer.
  - kernel: port-trait + entities (WorkflowRun, StepExecution, RetryAttempt, SlaTimer). Zero I/O.
  - domain: pure step-state arithmetic, retry-backoff math, SLA-timer arithmetic; deterministic.
  - usecase: orchestrators driving run lifecycle.
  - api: typed contracts.
  - adapter: protocol-neutral implementations.
  - adapter-postgres: Postgres-backed run state (the durable-execution authoritative store).
  - adapter-redis: Redis-backed ephemeral state (in-flight step claim leases; coordinator locks).
  - rest: HTTP surface for run operations.
  - worker: long-lived step-dispatch worker binary.
  - sdk: client library.
  - app: composition-root.
- exemptions claimed: none.
```

Naming justification — `state-machine`:

```
NAME: oya-workflow-engine-state-machine-<layer>
JUSTIFICATION:
- microservice = workflow-engine.
- bc-tokens = state-machine: BC for transition validation and invariant enforcement; distinct from
  execution-engine because state-machine concerns are PURE (rule eval) vs execution-engine is I/O-driven.
- layer = <layer>: trimmed crate set; no rest/worker/sdk because state-machine is consumed by
  execution-engine, not by external clients directly.
  - kernel: port-trait + entities (Transition, TransitionRule, StateCheckpoint). Zero I/O.
  - domain: pure transition evaluation; deterministic invariant checks.
  - usecase: orchestrators that compose transitions.
  - api: typed contracts.
  - adapter: protocol-neutral implementations.
  - adapter-postgres: checkpoint persistence.
- exemptions claimed: none.
```

Naming justification — `event-bus`:

```
NAME: oya-workflow-engine-event-bus-<layer>
JUSTIFICATION:
- microservice = workflow-engine.
- bc-tokens = event-bus: BC for typed workflow event pub/sub + outbox + replay-from-offset.
- layer = <layer>: full set because event-bus is externally consumed (Studio + every µservice +
  tenant SDK).
  - kernel: port-trait + entities (WorkflowEvent, EventOffset, Subscription). Zero I/O.
  - domain: pure event serialization, offset arithmetic.
  - usecase: orchestrators.
  - api: typed contracts.
  - adapter: protocol-neutral implementations.
  - adapter-postgres: outbox-pattern Postgres-backed durable event log.
  - adapter-redis: ephemeral subscription state.
  - rest: HTTP surface.
  - worker: long-lived outbox-relay worker.
  - sdk: client library.
  - app: composition-root.
- exemptions claimed: none.
```

Naming justification — `replay-debugger-backend`:

```
NAME: oya-workflow-engine-replay-debugger-backend-<layer>
JUSTIFICATION:
- microservice = workflow-engine.
- bc-tokens = replay-debugger-backend: BC for deterministic replay + debugger state reconstruction +
  analytics queries; distinct from execution-engine because READ-side concerns.
- layer = <layer>:
  - kernel: port-trait + entities (ReplaySession, StepSnapshot, RunAnalytics).
  - domain: pure replay logic over event log; deterministic.
  - usecase: orchestrators.
  - api: typed contracts.
  - adapter: protocol-neutral implementations.
  - adapter-postgres: read-side over the run state store.
  - adapter-clickhouse: analytics queries over the ClickHouse replica of run history.
  - rest: HTTP surface for Studio debugger.
  - sdk: client library.
  - app: composition-root.
- exemptions claimed: none.
```

Layer mapping per BC (13-layer canonical enum from ADR-0105; `usecase` per ADR-0106):

| BC | kernel | domain | usecase | api | adapter | adapter-postgres | adapter-redis | adapter-clickhouse | rest | worker | sdk | app |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `spec-store` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | ✓ | — | ✓ | ✓ |
| `execution-engine` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ | ✓ | ✓ | ✓ |
| `state-machine` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — |
| `event-bus` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ | ✓ | ✓ | ✓ |
| `replay-debugger-backend` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ | ✓ | — | ✓ | ✓ |

Total crates introduced by this µservice: **41** (9 spec-store + 12 execution-engine + 6 state-machine + 11 event-bus + 11 replay-debugger-backend; counting the backend-qualified adapters).

Port traits declared in each kernel (zero business logic; zero I/O; `data_class` annotated per Bominal ADR-0028):

| Port trait | Kernel crate | Implemented in | Data classes touched |
|---|---|---|---|
| `WorkflowSpecRepository` | `oya-workflow-engine-spec-store-kernel` | `-adapter-postgres` (Postgres-backed spec storage) | `INTERNAL_ONLY` (spec text) + `AUDIT` (signature) |
| `SpecCompiler` | `oya-workflow-engine-spec-store-kernel` | `-usecase` (pure logic via domain) | `INTERNAL_ONLY` |
| `WorkflowRunStore` | `oya-workflow-engine-execution-engine-kernel` | `-adapter-postgres` (durable run state) | `BEHAVIORAL_TENANT_PRODUCT` (per-tenant run state) + occasional `PII_IDENTIFYING` (step payloads) |
| `StepDispatcher` | `oya-workflow-engine-execution-engine-kernel` | `-usecase` (orchestrator) | `BEHAVIORAL_TENANT_PRODUCT` |
| `RetryPolicyEvaluator` | `oya-workflow-engine-execution-engine-kernel` | `-domain` (pure math) | `INTERNAL_ONLY` |
| `SlaTimerStore` | `oya-workflow-engine-execution-engine-kernel` | `-adapter-redis` (ephemeral) + `-adapter-postgres` (durable mirror) | `BEHAVIORAL_TENANT_PRODUCT` |
| `EphemeralStateStore` | `oya-workflow-engine-execution-engine-kernel` | `-adapter-redis` (in-flight step claims) | `BEHAVIORAL_TENANT_PRODUCT` |
| `TransitionEngine` | `oya-workflow-engine-state-machine-kernel` | `-domain` (pure) | `INTERNAL_ONLY` |
| `InvariantValidator` | `oya-workflow-engine-state-machine-kernel` | `-domain` (pure) | `INTERNAL_ONLY` |
| `StateCheckpointStore` | `oya-workflow-engine-state-machine-kernel` | `-adapter-postgres` | `AUDIT` |
| `EventBus` | `oya-workflow-engine-event-bus-kernel` | `-adapter-postgres` (outbox) + `-adapter-redis` (subscriptions) | `BEHAVIORAL_TENANT_PRODUCT` (event payloads) |
| `OutboxRelay` | `oya-workflow-engine-event-bus-kernel` | `-worker` | `AUDIT` |
| `EventLogReader` | `oya-workflow-engine-replay-debugger-backend-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT` |
| `ReplayEngine` | `oya-workflow-engine-replay-debugger-backend-kernel` | `-domain` (pure) | `INTERNAL_ONLY` |
| `RunAnalyticsRepository` | `oya-workflow-engine-replay-debugger-backend-kernel` | `-adapter-clickhouse` | `BEHAVIORAL_TENANT_PRODUCT` |

Data-class enforcement: every kernel struct field carries a `#[data_class(...)]` annotation; the `oya-check-data-class` LEAN lane refuses unannotated fields at PR-time.

Cross-product rule: `workflow-engine` MUST NOT import any other product µservice crate at any layer. All cross-product flows go through the engine's own event-bus (this µservice IS the orchestration adapter) or through Ontology reads/writes. LEAN-A2 CI lane enforces.

CI lanes that must green:

- `oya gate validate lean-a1 --microservice workflow-engine` — dependency-direction
- `oya gate validate lean-a2 --microservice workflow-engine` — cross-product-refusal (workflow-engine is the EXCEPTION-bearer; this lane permits the µservice to be imported AS an adapter, but the µservice itself doesn't import others)
- `oya gate validate port-location --microservice workflow-engine`
- `oya gate validate layer-correctness --microservice workflow-engine`
- `oya gate validate per-microservice-layout --microservice workflow-engine`
- `oya gate validate statelessness --microservice workflow-engine` — engine workers stateless beyond Postgres + Redis
- `oya gate validate shardability --microservice workflow-engine`
- `oya gate validate deterministic-replay --microservice workflow-engine` — NEW lane (per spec) asserting replay determinism for shipped workflow specs

## Integration via Workflow + Ontology

The engine is itself the action adapter. Its event-bus topics carry the typed events that every other µservice publishes and consumes.

### Workflow events produced

| Event type | Trigger | Consumed by | State machine / DAG |
|---|---|---|---|
| `WorkflowStarted` | run start | Studio (live monitor), audit-chain, replay-debugger-backend | run-lifecycle SM |
| `StepStarted` | step dispatch | Studio, replay-debugger-backend | step-lifecycle SM |
| `StepCompleted` | step success | Studio, audit-chain, replay-debugger-backend, downstream subscribers | step-lifecycle SM |
| `StepFailed` | step error (post-retry exhaustion) | Studio, audit-chain, replay-debugger-backend, alerting | step-lifecycle SM |
| `StepRetried` | retry attempt | Studio, replay-debugger-backend | — |
| `WorkflowPaused` | operator intervention | Studio, audit-chain | run-lifecycle SM |
| `WorkflowResumed` | operator intervention | Studio, audit-chain | run-lifecycle SM |
| `WorkflowCancelled` | operator or self-cancel | Studio, audit-chain | run-lifecycle SM |
| `WorkflowCompleted` | terminal state | Studio, audit-chain, subscribers, replay-debugger-backend | run-lifecycle SM |
| `WorkflowFailed` | terminal failure | Studio, audit-chain, subscribers, alerting | run-lifecycle SM |
| `SlaTimerArmed` | step entry | execution-engine internal | — |
| `SlaTimerFired` | timer expiry | escalation handler | — |

### Workflow events consumed

Every typed event published by any µservice via `oya-workflow-engine-event-bus-sdk` is in scope. Examples:

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `EmployeeHired` | `hr` | execution-engine triggers | fan out to payroll-enrollment, connect-provisioning, ... |
| `PayrollRunCompleted` | `payroll` | execution-engine triggers | trigger accounting journal-posting workflow |
| `OpenSLOManifestUpdated` | `observability` | spec-store hot-reload | re-validate any specs that reference observability SLI |
| `TenantActivated` | `tenancy` | execution-engine triggers | trigger product-onboarding workflows |

### Ontology writes

| Object Type | Link Type | Written by BC | Audit trail |
|---|---|---|---|
| `WorkflowRun{tenant, run_id, spec_id, started_at, completed_at, terminal_state}` | `run_of→WorkflowSpec` | execution-engine | Ed25519 |
| `StepExecution{run_id, step_index, status, started_at, completed_at, error?, retry_count}` | `step_of→WorkflowRun` | execution-engine | Ed25519 |
| `WorkflowSpec{tenant, spec_id, version_sha, body_hash, signed_by}` | `version_of→WorkflowSpec` | spec-store | Ed25519 |

### Ontology reads

| Object Type / Function | Read by BC | Query shape |
|---|---|---|
| `Tenant` (catalog) | execution-engine | `where(active=true).limits()` to enforce per-tenant rate caps |
| `WorkflowSpec` (versions) | spec-store | for hot-reload + replay |

## Competitive Benchmark

| Competitor | Product / feature | Parity dimensions | Primary source |
|---|---|---|---|
| Temporal | Temporal Cloud + open-source SDK | Durable execution, deterministic replay, multi-week run lifetimes, event-sourcing, versioning, signals/queries | `docs.temporal.io` |
| Cadence | Uber Cadence (Temporal's predecessor) | same semantics; older API surface | `cadenceworkflow.io` |
| Apache Airflow | Airflow 2.x scheduler + DAG executor | DAG-based pipelines, sensor operators, backfill, task retries | `airflow.apache.org/docs` |
| Camunda | Camunda Platform 8 (Zeebe engine) | BPMN 2.0 engine; broker-based; cluster shardable | `docs.camunda.io` |
| Argo Workflows | Argo Workflows on K8s | container-native step execution; DAG + step templates; resource-bounded | `argoproj.github.io/argo-workflows` |
| n8n (engine layer) | n8n engine (separated from Studio editor) | sub-second event-to-action; node retry; webhook trigger | `docs.n8n.io` (engine internals) |
| Step Functions | AWS Step Functions | managed state machine; ASL DSL; integrated with AWS services | `docs.aws.amazon.com/step-functions` |
| Dapr Workflows | Dapr Workflow building block | actor-based durable execution; multi-language | `docs.dapr.io/developing-applications/building-blocks/workflow` |

Key parity gaps to close (ordered by priority for M02b substrate-ready milestone):

1. **Durable execution at Temporal parity** — engine process kill mid-run; on restart, run resumes from last completed step; event log replay produces identical step sequence. This is the hardest engineering requirement.
2. **Sub-second event-to-action latency** — n8n's UX expectation; outbox → bus → worker → step dispatch p99 ≤ 500ms.
3. **Cross-µservice orchestration adapter** — none of Temporal/Cadence/Airflow/Camunda are bundled as the cross-product event router by design. oyatie unique: engine is the load-bearing adapter (per `feedback_workflow_objectgraph_adapter_layer.md`).
4. **Per-tenant Citus sharding** — Temporal Cloud is per-namespace; Camunda 8 is per-cluster; Airflow has no native multi-tenancy. oyatie target: per-tenant linear shard addition.
5. **Replay-as-debugger** — Temporal exposes replay as a developer-tools concept; oyatie target: replay-as-tenant-facing-debugger via Studio (engine ships the backend; Studio renders).

## Performance Targets

(Duplicated from §"Non-Functional Requirements" for ease of citation by downstream PRD consumers.)

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Workflow execution start → first step | 50ms | 200ms | 500ms | engine dispatch latency |
| Step execution (local action) | 5ms | 50ms | 100ms | excludes network-bound external calls |
| Event-to-action latency (outbox → bus → worker) | 50ms | 500ms | 1s | full pipeline |
| Concurrent active runs per cell | — | 10,000 | — | sharding to 100k+ aggregate |
| Run durability ceiling | — | 90 days paused | — | Temporal parity |
| Audit chain seal | — | 1s | — | Ed25519 per run |

Error budget:
- Monthly error budget for execution path: 0.05% (≈22 min/month).
- Monthly error budget for replay/debugger backend: 0.1%.
- Burn-rate alarms: 14.4× burn over 1h for execution path; 6× burn over 6h for replay/debugger.

## Horizontal Scalability

**State strategy** (per Bominal ADR-0019 enum): `mixed`. Rationale:
- Workflow specs + run state + outbox: `postgres` (Citus-distributed by tenant_id).
- Ephemeral step claim leases + subscription state: `redis` (per-cell cluster; can be lost without data corruption — claims are re-derivable from Postgres outbox).
- Analytics replay history (long-term): `clickhouse` (replica of run history; read-side only).
- Object storage for large step payloads (e.g., file artifacts): OCI Object Storage.

**Active-active compatibility**: `stateless-compatible` for REST/SDK/event-bus subscribers; `single-writer-compatible` for engine workers processing a specific run (one worker owns one run at a time via Redis lease; no concurrent writers per run).

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Concurrent active runs | 10,000 | 500,000 | Worker queue depth > 5k OR step lease wait p99 > 200ms |
| Workflow specs per tenant | 1,000 | 100,000 | Postgres shard fill > 80% |
| Events/sec inbound (event-bus) | 10,000 | 1,000,000 | Kafka-equivalent consumer lag > 1s |
| Steps/sec dispatched | 5,000 | 200,000 | Step dispatch lease wait p99 > 100ms |

Scale-out policy:
- Engine workers: stateless HPA on queue depth > 5k; min 3 replicas; max 200 replicas.
- REST/gRPC: stateless HPA on CPU > 70%; min 2 replicas; max 50.
- Outbox relay worker: stateless; one leader per partition via Redis lease.
- Postgres + Citus: tenant_id shard key; linear shard addition; ADR-0117 sharding posture.
- ClickHouse replica: read-replicated; sharded by tenant_id.
- Redis: per-cell cluster; HA via Sentinel.
- Pre-warmed worker pool: 10 standby pods; cold-start budget ≤ 500ms.

Cross-region story:
- M02b substrate launch: single KR region (OCI ap-seoul-1).
- Post-M02b: active-active per-cell cross-region per ADR-0117 stages; run state replicated via outbox + cross-cell bridge.
- Long-running workflows: paused state written to Postgres; survives region failover if replica is current.

Sharding:
- Postgres + Citus on `tenant_id`; run log append-only; Citus distributed table.
- Event-bus: per-tenant topic namespace; cell-local cluster; cross-cell bridge for multi-cell workflow fan-out.
- `oya-check-shardability-cli` CI lane enforces `tenant_id` partition key presence on all engine tables.

## Acceptance Criteria

| AC-ID | Criterion | Verification method |
|---|---|---|
| AC-01 | Workflow spec submitted → trigger fires → first step executes in ≤ 200ms (p99) | integration test `tests/e2e/workflow-execution-latency.rs` |
| AC-02 | Deterministic replay: same event log produces same step sequence 100% of the time | `cargo nextest run -p oya-workflow-engine-replay-debugger-backend-domain --test deterministic_replay` |
| AC-03 | Durable execution: engine process killed mid-run; on restart, run resumes from last completed step with no duplicated effects | integration test `tests/e2e/engine-durability-restart.rs` |
| AC-04 | Run survives pod eviction; resumed on a different node; identical step sequence on completion | integration test `tests/e2e/pod-eviction-resumption.rs` |
| AC-05 | 10k concurrent active runs per cell; p99 step execution ≤ 200ms | k6 load: `tests/load/engine-10k-runs.js` |
| AC-06 | Tenant isolation: tenant A run cannot observe tenant B event payloads via any subscription | `cargo nextest run -p oya-workflow-engine-event-bus-domain --test tenant_subscription_isolation` |
| AC-07 | Audit chain: every run sealed; tampering detected on verification | `oya gate validate audit-chain --ms workflow-engine` |
| AC-08 | LEAN-A2: no product µservice imports (hr / payroll / connect / etc.) | `oya gate validate lean-a2 --microservice workflow-engine` exits 0 |
| AC-09 | Outbox crash → no event loss; recovery resumes from last persisted offset | integration test `tests/e2e/outbox-crash-recovery.rs` |
| AC-10 | SLA timer: armed at step entry; fires escalation on breach; not before | `cargo nextest run -p oya-workflow-engine-execution-engine-domain --test sla_escalation_timing` |
| AC-11 | Replay throughput: ≥ 1000 steps/s/worker on a single CPU; deterministic | `cargo nextest run -p oya-workflow-engine-replay-debugger-backend-worker --test replay_throughput` |
| AC-12 | Spec signature verification: tampered spec is refused at read-time | `cargo nextest run -p oya-workflow-engine-spec-store-domain --test spec_signature_tampering_detected` |
| AC-13 | `oya gate validate per-microservice-layout --microservice workflow-engine` exit 0 | ADR-0131 lane |
| AC-14 | `oya gate validate authority-cohesion` exit 0 | ADR-0123 lane; HG-WF-ENGINE registered |
| AC-15 | `oya gate validate deterministic-replay --microservice workflow-engine` exit 0 | new lane spec'd in PHASE-01 |

## Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | Durable-execution: bespoke Postgres-backed state machine vs embed Temporal SDK (Rust)? Bias: bespoke per `feedback_autonomous_implementation_artifacts.md` (no compat seams). | council-architecture | ADR-XXXX (highest priority; gates IP-005) |
| 2 | Event-bus substrate: Postgres outbox + LISTEN/NOTIFY, NATS JetStream, Kafka KRaft, or Redis Streams? Latency vs durability vs operational complexity trade-off. | council-architecture + ops-sre-reliability | ADR-XXXX (gates IP-007) |
| 3 | Workflow DSL format the engine accepts: YAML, JSON IR, or both? Bias: JSON IR canonical (machine-emitted by Studio), YAML accepted as input but compiled to JSON IR at submit time. | council-architecture | resolved inline; see spec-store IP |
| 4 | ClickHouse replica: full mirror of Postgres run history, or selective columns? Mirror-cost vs query-richness trade-off. | axis-workflow + ops-finops | resolved inline |
| 5 | Sub-workflow invocation: synchronous (caller blocks) or asynchronous (caller waits for completion event)? Both semantics needed; default = async with synchronous opt-in. | council-architecture | resolved inline; see execution-engine IP |
| 6 | Replay determinism guarantee: which standard-library / time / random APIs are forbidden inside step bodies? | council-architecture | ADR-XXXX (`docs/standards/workflow-step-determinism.md` successor-IP) |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| ADR-0035 (Bominal) | Workflow engine (hybrid state machine + DAG) | inherited; engine architecture |
| ADR-0103 (Bominal) | Workflow hexagonal migration | inherited; clean-arch placement |
| ADR-0148 (Bominal) | Workflow engine (extended) | inherited |
| ADR-0028 (Bominal) | Audit chain (Merkle + Ed25519) | inherited; per-run seal |
| ADR-0107 (Bominal) | Ontology agent gateway | inherited; agentic step nodes |
| ADR-0037 (Bominal) | Plugin substrate (WASM) | inherited; custom node SDK |
| ADR-0009 (Bominal) | Cell architecture | inherited; per-cell run isolation |
| ADR-0019 (Bominal) | Runtime target metadata | inherited; active-active posture |
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0105 | 13-layer enum | layer authority |
| ADR-0110 | ChangeSet state machine | each IP is one ChangeSet |
| ADR-0123 | Hyperscaler maturity claim gate | HG-WF-ENGINE registers here |
| ADR-0139 | Agentic SLO-gated promotion | engine SLO promotion gates this µservice |
| ADR-0131 | Per-microservice flat layout + workflow unbundle | this µservice authored natively under it; sibling = workflow-studio |
| oyatie override | Workflow is shared (not Corporate) | `feedback_workflow_is_shared.md` |
| oyatie override | Workflow + Ontology = ecosystem adapter | `feedback_workflow_objectgraph_adapter_layer.md` |
| oyatie split | engine vs studio unbundle | `feedback_workflow_studio_scope.md` |
