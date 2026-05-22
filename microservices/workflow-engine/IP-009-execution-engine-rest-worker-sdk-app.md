---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02b-substrate-ready
phase: P01-durable-execution-substrate
impl_plan_id: IP-009-execution-engine-rest-worker-sdk-app
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, oya-governance-deterministic-replay]
---

# IP-009: execution-engine REST, worker, SDK, and runtime app root

## §A Problem

The execution engine is where the substrate either behaves like Temporal or degrades into a web handler that happens to start jobs. The REST contract in `microservices/workflow-engine/contracts/openapi/workflow-engine.yaml` exposes start, pause, resume, cancel, signal, step reads, replay initiation, and tenant metrics. The proto `ExecutionEngine` service mirrors those calls. The stamped IP did not specify how the worker claims steps, resumes after pod eviction, enforces two-person cancel, or keeps SDK callers from bypassing durable state.

This IP closes the operational runtime gap: tenant and internal callers need one control surface for durable run control, while the worker must execute steps from persisted state and leases without making Valkey the source of truth.

## §B Approach

Create the four remaining execution-engine crates from `microservices/workflow-engine/manifest.json`: `rest`, `worker`, `sdk`, and `app`. REST handles authenticated control-plane calls. The worker owns step dispatch, lease claim/release, retry scheduling, SLA timer firing, and cold-start resume throttling. The SDK wraps tenant programmatic control with idempotency keys and typed request/response objects. The app root composes Postgres, Valkey, state-machine usecase, spec-store read port, event-bus publisher, Cedar policy, and audit-chain emission.

Non-goals are explicit: this IP does not redefine state-machine transitions, spec-store signing, or event-bus delivery guarantees. It composes those ports and proves the runtime surface cannot bypass them.
It also does not add workflow authoring UI; Studio remains the product surface.

## §C Deliverables

| Artifact | Action | Substance requirement |
|---|---|---|
| `microservices/workflow-engine/src/crates/oya-workflow-engine-execution-engine-rest/src/routes.rs` | create | handlers for `/runs`, pause/resume/cancel/signal, steps, replay, metrics |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-execution-engine-rest/src/middleware.rs` | create | OIDC, Cedar action mapping, two-person cancel validation |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-execution-engine-worker/src/run_dispatcher.rs` | create | finds runnable steps from durable store and dispatches with tenant active-run caps |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-execution-engine-worker/src/step_executor.rs` | create | executes idempotent activities, writes step result, emits lifecycle events |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-execution-engine-worker/src/resume_throttle.rs` | create | resumes after restart at bounded rate to avoid thundering herd |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-execution-engine-sdk/src/client.rs` | create | typed start/pause/resume/cancel/signal client with idempotency helper |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-execution-engine-app/src/main.rs` | create | runtime binary composition and config validation |
| `microservices/workflow-engine/catalog/oya-workflow-engine-execution-engine-{rest,worker,sdk,app}.yaml` | update/create | catalog rows for all four crates |

## §D Implementation

1. Map REST handlers to OpenAPI operation IDs: `startWorkflowRun`, `getWorkflowRun`, `pauseWorkflowRun`, `resumeWorkflowRun`, `cancelWorkflowRun`, `signalWorkflowRun`, and `listStepExecutions`.
2. Enforce Cedar actions from `policy/tenant-scope.cedar` before usecase calls; production cancel requires the contract's `two_person_signature` field.
3. Build the worker around durable run state from Postgres and ephemeral leases from Valkey; losing a lease can cause retry, not lost execution history.
4. Use the state-machine usecase from IP-003 for all run-status moves and the event-bus SDK from IP-007 for lifecycle event publication.
5. Implement cold-start resume throttling so a pod restart drains paused/waiting/running runs at a controlled per-worker rate, preserving the SLO budgets in `slos/workflow-step-execute-latency.openslo.yaml`.
6. Add SDK idempotency behavior for run start and signal delivery; repeated tenant calls return the original run or signal acceptance.
7. Wire app config to fail at startup when Postgres, Valkey Sentinel, Cedar bundle, OpenBao refs, or event-bus endpoints are missing.

## §E Acceptance

- `cargo nextest run -p oya-workflow-engine-execution-engine-rest --all-features`
- `cargo nextest run -p oya-workflow-engine-execution-engine-worker --all-features`
- `cargo nextest run -p oya-workflow-engine-execution-engine-sdk --all-features`
- `cargo nextest run -p oya-workflow-engine-execution-engine-app --all-features`
- `cargo run -p oya-dev-cli -- gate validate openapi-conformance --crate oya-workflow-engine-execution-engine-rest`
- Required tests: `start_run_pins_spec_version`, `cancel_production_requires_two_signatures`, `signal_wrong_tenant_denied`, `pod_eviction_resumes_on_new_worker`, `resume_throttle_caps_cold_start`, `sla_timer_emits_pause_or_escalation`, and `sdk_start_idempotency_returns_existing_run`.

## §F Evidence

- `microservices/workflow-engine/contracts/openapi/workflow-engine.yaml` is the REST authority for run control.
- `microservices/workflow-engine/contracts/proto/workflow-engine.proto` is the service-to-service RPC authority for execution control.
- `microservices/workflow-engine/runbooks/durable-execution-restart.md` and `runbooks/stuck-workflow-recovery.md` define the restart and stuck-run operational paths.
- `microservices/workflow-engine/slos/workflow-start-latency.openslo.yaml`, `workflow-step-execute-latency.openslo.yaml`, and `workflow-completion-availability.openslo.yaml` define runtime success evidence.
- `microservices/workflow-engine/policy/tenant-scope.cedar` defines run-control authorization.

## §G Counterparts

| Counterpart | Relevant behavior | This IP closes |
|---|---|---|
| Temporal / Cadence | workers poll task queues and resume from durable history | worker claims leases but persists authority in Postgres-backed run state |
| AWS Step Functions | API controls start/stop/signal-like task token flows | REST/proto control plane maps each run action to Cedar and audit evidence |
| Argo Workflows | controller reconciles workflow CR state after pod loss | resume throttle and state-machine composition prevent restart storms |
| n8n | execution workers run user automations but often lack deterministic crash replay | pinned spec, checkpoint transitions, and event emission make worker restart auditable |

## Next IP

[`IP-010-replay-debugger-backend-kernel-domain.md`](IP-010-replay-debugger-backend-kernel-domain.md)

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/workflow-engine/contracts/openapi/workflow-engine.yaml`, `microservices/workflow-engine/contracts/asyncapi/workflow-events.yaml`, `microservices/workflow-engine/contracts/proto/workflow-engine.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/workflow-engine/IP-009-execution-engine-rest-worker-sdk-app.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/workflow-engine/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/workflow-engine/slos/payload-bytes-budget-correctness.openslo.yaml`, `microservices/workflow-engine/slos/replay-determinism-correctness.openslo.yaml`, `microservices/workflow-engine/slos/worker-poll-availability.openslo.yaml`, `microservices/workflow-engine/slos/workflow-completion-availability.openslo.yaml`, `microservices/workflow-engine/policy/auditor-scope.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/workflow-engine/IP-009-execution-engine-rest-worker-sdk-app.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/workflow-engine/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: eligible only when ADR-0344 D-9 compliance-pack exclusions do not bar deferral; otherwise the Cedar scheduler rejects delay while still emitting carbon fields.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
