---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02b-substrate-ready
phase: P01-durable-execution-substrate
impl_plan_id: IP-003-state-machine-kernel-domain
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, oya-governance-deterministic-replay]
---

# IP-003: workflow state-machine transition kernel through Postgres checkpoints

## §A Problem

`workflow-engine` cannot be a Temporal-class durable substrate if transition rules live as incidental REST or worker code. The service contract already exposes run reads, step reads, pause/resume/cancel/signal, and transition history in `microservices/workflow-engine/contracts/openapi/workflow-engine.yaml`; the proto mirrors this with `ExecutionEngine` RPCs in `microservices/workflow-engine/contracts/proto/workflow-engine.proto`. This IP closes the missing state-machine boundary between immutable workflow specs, run-state transitions, and checkpoint persistence.

The gap is specific to workflow runtime semantics: every `WorkflowStarted`, `StepStarted`, `StepCompleted`, `StepFailed`, `StepRetried`, `WorkflowPaused`, `WorkflowResumed`, `WorkflowCancelled`, `WorkflowCompleted`, and `WorkflowFailed` event in `microservices/workflow-engine/contracts/asyncapi/workflow-events.yaml` needs one deterministic transition path. Without a pure transition evaluator, replay debugging can only compare logs after the fact; it cannot prove the engine would make the same state move from the same event.

## §B Approach

Create a dedicated `state-machine` bounded-context stack under the crate names already declared in `microservices/workflow-engine/manifest.json`: `oya-workflow-engine-state-machine-kernel`, `domain`, `usecase`, `api`, `adapter`, and `adapter-postgres`. The kernel owns value objects and sealed ports. The domain owns pure transition evaluation. The usecase composes domain decisions with checkpoint writes. The adapter-postgres crate persists append-only checkpoints keyed by `(tenant_id, run_id, checkpoint_seq)`.

The transition evaluator consumes only a current `StateCheckpoint`, a typed workflow event, and the pinned `spec_id/version_sha`; it must not call wall-clock time, random number generation, network I/O, or storage. That preserves the spec-integrity doctrine in `microservices/workflow-engine/policy/spec-integrity.md`, where system-time access, non-deterministic RNG, uncached I/O, and circular sub-workflow references are forbidden because they break replay.

## §C Deliverables

| Artifact | Action | Substance requirement |
|---|---|---|
| `microservices/workflow-engine/src/crates/oya-workflow-engine-state-machine-kernel/Cargo.toml` | create | no adapter dependencies; depends only on shared ID/error crates already accepted by repo naming gates |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-state-machine-kernel/src/entities.rs` | create | `WorkflowState`, `Transition`, `TransitionRule`, `StateCheckpoint`, `CheckpointSeq`, `InvariantViolation` |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-state-machine-kernel/src/ports.rs` | create | sealed `TransitionEngine`, `InvariantValidator`, `StateCheckpointStore` ports |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-state-machine-domain/src/transition_eval.rs` | create | pure evaluator for lifecycle and step events from the AsyncAPI contract |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-state-machine-domain/src/invariant_check.rs` | create | tenant equality, legal terminal-state, pause/resume, cancel, and signal invariants |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-state-machine-usecase/src/compose.rs` | create | orchestrates evaluate -> validate -> append checkpoint with expected sequence |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-state-machine-api/src/types.rs` | create | maps OpenAPI/proto transition history fields to kernel types without stringly transitions |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-state-machine-adapter-postgres/src/lib.rs` | create | checkpoint repository with tenant predicate and optimistic append |
| `microservices/workflow-engine/catalog/oya-workflow-engine-state-machine-*.yaml` | update/create | six catalog rows matching manifest crate list |
| `Cargo.toml` | update | workspace members for all six crates |

## §D Implementation

1. Define `WorkflowState` as an enum aligned to proto `RunStatus` and step status fields, with conversion tests proving no unknown production state is silently accepted.
2. Implement `TransitionEngine::next_state(current, event, spec_ref)` in the domain crate, with event variants sourced from the AsyncAPI lifecycle message names rather than ad hoc strings.
3. Add invariant checks for tenant equality, monotonic checkpoint sequence, no transition out of `completed/failed/cancelled`, no resume from non-paused state, and no cancel without the policy context required by `policy/tenant-scope.cedar`.
4. Add the usecase function that loads the current checkpoint, invokes the pure evaluator, validates invariants, and appends a new checkpoint through `StateCheckpointStore` with `expected_checkpoint_seq`.
5. Implement Postgres append semantics in `adapter-postgres`: one insert per transition; reads always include `tenant_id`; unique key `(tenant_id, run_id, checkpoint_seq)`.
6. Wire API types so `/runs/{run_id}/steps`, `/runs/{run_id}`, and proto `ListStepExecutions` can rely on checkpoint state without re-deriving from mutable worker state.
7. Register catalog rows and workspace members, then run port-location and layer-correctness gates against each crate.

## §E Acceptance

- `cargo check -p oya-workflow-engine-state-machine-kernel --all-features`
- `cargo check -p oya-workflow-engine-state-machine-domain --all-features`
- `cargo nextest run -p oya-workflow-engine-state-machine-domain --all-features`
- `cargo nextest run -p oya-workflow-engine-state-machine-usecase --all-features`
- `cargo nextest run -p oya-workflow-engine-state-machine-adapter-postgres --all-features`
- `buck2 build //:quality-lane-registry-authority-check # lane=port-location --crate oya-workflow-engine-state-machine-kernel`
- `buck2 build //:quality-lane-registry-authority-check # lane=layer-correctness --crate oya-workflow-engine-state-machine-domain`
- Required tests: `transition_eval_replays_identically`, `terminal_state_refuses_late_event`, `tenant_mismatch_refused_before_store`, `checkpoint_append_conflict_detected`, and `pause_resume_signal_sequence_preserved`.

## §F Evidence

- Product requirement: `microservices/workflow-engine/PRD.md` names deterministic replay, crash recovery, pause/resume, long-lived signals, and multi-tenant isolation as core engine requirements.
- Contract evidence: `microservices/workflow-engine/contracts/openapi/workflow-engine.yaml` exposes run state, step reads, and transition history; `contracts/proto/workflow-engine.proto` exposes the same RPC surface.
- Policy evidence: `microservices/workflow-engine/policy/spec-integrity.md` forbids non-deterministic constructs; `policy/tenant-scope.cedar` binds reads and mutations to tenant-owned workflow data.
- Runbook evidence: `microservices/workflow-engine/runbooks/workflow-state-corruption-recovery.md` and `runbooks/durable-execution-restart.md` depend on trustworthy checkpoints.

## §G Counterparts

| Counterpart | Relevant behavior | This IP closes |
|---|---|---|
| Temporal / Cadence | event-history replay drives deterministic state reconstruction | pure transition evaluator plus append-only checkpoints |
| Camunda 8 Zeebe | workflow instance state is broker-owned, not REST-handler-owned | state-machine BC owns transitions before adapters persist |
| AWS Step Functions | state transitions are explicit and replayable from execution history | typed transition set tied to OpenAPI/proto/AsyncAPI events |
| n8n | best-effort execution state is not deterministic-replay-grade | checkpoint invariants prevent opaque mutable worker state from becoming authority |

## Next IP

[`IP-004-execution-engine-kernel-domain.md`](IP-004-execution-engine-kernel-domain.md)

## API Versioning (per ADR-0342)

- Authority: ADR-0342.
- Contract evidence: `microservices/workflow-engine/contracts/openapi/workflow-engine.yaml`, `microservices/workflow-engine/contracts/asyncapi/workflow-events.yaml`, `microservices/workflow-engine/contracts/proto/workflow-engine.proto`.
- Carrier: `YYYY-MM-DD` value via `Oyatie-Version` header + `/v/<date>/` URL prefix + public proto3 `string oyatie_version = 8001`.
- Initial `declared_version`: `2026-05-21`.
- Support window: `N=3` public versions for at least `180` days after deprecation.
- Internal-mesh exemption: per ADR-0145, internal gRPC over HTTP/3 remains proto3 tag-compatible and does not carry public version routing.
