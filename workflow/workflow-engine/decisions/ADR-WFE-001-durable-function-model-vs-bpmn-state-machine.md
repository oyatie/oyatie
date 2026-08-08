---
id: ADR-WFE-001
title: Durable Function Model vs BPMN State Machine Runtime
status: Proposed
date: 2026-05-20
microservice: workflow-engine
related_oyatie_adrs:
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0704-k8s-port-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0708-platform-foundations-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
decision_owner: axis-workflow-engine
---

# ADR-WFE-001: Durable Function Model vs BPMN State Machine Runtime

## Context

- Workflow-engine is the durable execution substrate for Oyatie product and tenant RBAC workflows.
- The engine has no end-user UI and must serve Workflow Studio, saga orchestration, compliance automation, and internal control-plane jobs.
- The runtime must survive worker crashes, cell failover, process restarts, and partial downstream outages.
- The runtime must support tenant-scoped replay without leaking payloads across tenants or cells.
- ADR-0145 requires inter-microservice communication to use explicit contracts rather than hidden synchronous chains.
- ADR-0222 requires saga compensation to be visible, ordered, and reviewable when financial or compliance side effects occur.
- ADR-0243 requires Cedar authorization gates around every workflow action that changes state or exposes replay data.
- ADR-0244 makes tenant identity a universal scoping primitive, so workflow history keys must include tenant and cell dimensions.
- ADR-0245 places workflow-engine in the substrate layer; product-specific workflow meaning belongs in callers or Studio projections.
- ADR-0251 requires certification-level aware execution for workflows that support compliance-pack evidence.
- ADR-0263 requires consistent trace, metric, and log emission for durable operations.
- Existing requirements name Temporal-class deterministic replay as a benchmark.
- Existing requirements also name n8n-class authoring ergonomics, but that belongs to Workflow Studio rather than runtime state.
- Existing requirements compare against Temporal Cloud, Cadence, Airflow, Camunda Platform 8, Argo, AWS Step Functions, Dapr Workflows, Prefect, Restate, and Flink Stateful Functions.
- The practical runtime choice is between a durable function model and a BPMN state machine model.
- A durable function model records workflow decisions as deterministic code execution over event history.
- A BPMN state machine model records workflow execution as token movement through a modeled process graph.
- Camunda BPMN is attractive for enterprise process diagrams, audit walkthroughs, and business analyst review.
- Temporal-inspired durable functions are attractive for developer-owned sagas, compensation, timers, retries, and replay.
- The engine must compose with Workflow Studio, which emits canonical workflow specs and graph projections.
- The engine must execute workflows even when no Studio diagram exists.
- The engine must support machine-authored workflows from other substrate services.
- The engine must support long-running jobs lasting minutes, days, or months.
- The engine must handle high-cardinality tenant workloads without central scheduler lock contention.
- The engine must preserve execution determinism after binary upgrades.
- The engine must reject non-deterministic workflow code paths during replay.
- The engine must externalize large payloads rather than place unbounded data into event history.
- The engine must record human gates as durable signals rather than ephemeral UI state.
- The engine must expose compensation status for audit-chain and governance review.
- The engine must expose deterministic replay evidence for verifier tooling.
- The engine must avoid binding Oyatie to a managed external runtime control plane.
- ADR-0211 is not listed as a primary related ADR here, but the same in-house stack posture informs the choice.
- A pure BPMN engine would make process diagrams the runtime source of truth.
- A pure durable function engine would make deterministic command history the runtime source of truth.
- The architecture pressure is to preserve developer-grade reliability while still projecting business-readable diagrams.
- The runtime cannot allow a diagram editor to become an implicit production scheduler.
- The runtime cannot allow ad hoc async jobs to bypass policy, replay, and compensation guarantees.
- The runtime must distinguish workflow definition approval from workflow run authorization.
- The runtime must support per-step Cedar checks without making policy evaluation non-deterministic.
- The runtime must support regional and cell-specific queue placement.
- The runtime must support pause, replay, signal, cancel, continue-as-new, and compensation operations.
- The runtime must support backpressure when worker pools, downstream services, or event stores exceed thresholds.
- The runtime must provide safe defaults for idempotency, retry, timeout, and payload size.
- The runtime must leave enough metadata for cost attribution and SLO dashboards.
- The decision is therefore about the authoritative execution model, not about whether diagrams are allowed.

## Decision

- Adopt a Temporal-inspired durable function model as the authoritative workflow-engine runtime.
- Treat the event history as the source of truth for each workflow run.
- Treat workflow code and compiled workflow specs as deterministic interpreters over event history.
- Use activity tasks for non-deterministic side effects, remote service calls, external IO, and long CPU work.
- Use workflow tasks for deterministic orchestration decisions only.
- Use durable timers instead of process sleep or scheduler-local timers.
- Use signals for human gates, external callbacks, and cross-service event delivery.
- Use query handlers only for read-only state inspection.
- Use saga compensation records as first-class runtime state.
- Use continue-as-new when a run reaches 50,000 history events or 50 MiB of serialized history, whichever comes first.
- Use a hard per-activity input payload limit of 256 KiB.
- Use a hard per-workflow compiled spec limit of 2 MiB.
- Store large payloads in tenant-scoped object storage and reference them by content-addressed handle.
- Require all activity executions to provide an idempotency key.
- Require all outbound side effects to include `workflow_run_id`, `activity_id`, `attempt`, and `tenant_id`.
- Default activity start-to-close timeout is 5 minutes.
- Maximum activity start-to-close timeout is 24 hours and requires an explicit policy exception.
- Default workflow run timeout is 30 days.
- Maximum workflow run timeout is 400 days for compliance retention and human approval workflows.
- Default retry policy is exponential backoff starting at 1 second, multiplier 2.0, capped at 10 minutes.
- Default retry attempts are unlimited for explicitly idempotent transient errors and 3 for unknown errors.
- Terminal business errors must not be retried unless the workflow spec marks them as recoverable.
- BPMN diagrams may be imported, exported, and displayed as projections.
- BPMN diagrams are not the authoritative runtime state machine.
- Camunda-compatible BPMN import is allowed only through a compiler that emits the canonical workflow spec.
- Workflow Studio may render BPMN-like lanes and nodes, but engine execution remains durable-function based.
- Engine APIs must expose run history, pending activities, pending timers, signals, and compensation plans.
- Engine APIs must not expose raw tenant payloads to operators without Cedar permission.
- Worker queue placement must include `cell_id`, `tenant_id`, `workflow_namespace`, and `task_queue`.
- Workflow namespace names must be stable and versioned.
- Workflow definition versions must be immutable after approval.
- A new workflow definition version is required for any deterministic behavior change.
- Runtime upgrades must include replay tests for every approved workflow definition in the target cell.
- Replay determinism is a correctness SLO, not an availability SLO.
- The target for replay determinism is 100 percent for approved workflow definitions.
- Workflow start latency target is p99 under 500 ms inside a healthy cell.
- Workflow task scheduling latency target is p99 under 250 ms.
- Activity dispatch latency target is p99 under 1 second for uncongested queues.
- Worker poll availability target is 99.95 percent monthly per certified cell.
- Workflow completion availability target is 99.9 percent monthly for non-paused workflows.
- The runtime must emit OpenTelemetry traces according to ADR-0263.
- Every run must produce audit-chain compatible lifecycle events.
- Every compensation step must be individually addressable in evidence.

## Alternatives Considered

### Alternative 1: Camunda BPMN as the authoritative runtime

- Camunda BPMN provides a mature graphical process model.
- Camunda BPMN makes human process reviews approachable for enterprise stakeholders.
- Camunda BPMN has established language concepts for gateways, events, timers, and subprocesses.
- Camunda BPMN has a large ecosystem of modeling tools and examples.
- Camunda BPMN can make compliance narratives easier to present.
- Camunda BPMN token semantics are harder to map to developer-owned code evolution.
- Camunda BPMN requires process diagrams to carry runtime-critical meaning.
- Camunda BPMN creates pressure for business diagrams to become production control flow.
- Camunda BPMN complicates deterministic replay when workers and integrations evolve independently.
- Camunda BPMN may encourage product-specific semantics to leak into the substrate runtime.
- Camunda BPMN was rejected as the source of truth because Oyatie needs developer-grade durable execution first.
- Camunda BPMN remains acceptable as an import, export, and projection format.

### Alternative 2: Temporal Cloud as the managed control plane

- Temporal Cloud provides a proven durable execution service.
- Temporal Cloud provides mature worker SDK patterns and operational guidance.
- Temporal Cloud would reduce short-term engine implementation effort.
- Temporal Cloud would provide familiar concepts for engineers who know Temporal.
- Temporal Cloud creates external control-plane dependency for a core substrate service.
- Temporal Cloud may not fit Oyatie cell sovereignty, tenant isolation, and in-house stack requirements.
- Temporal Cloud limits how deeply Cedar policy, audit-chain evidence, and cell admission can be embedded.
- Temporal Cloud introduces vendor operational boundaries for replay evidence and history storage.
- Temporal Cloud was rejected as the production control plane.
- Temporal-inspired concepts were retained because they match the runtime problem shape.

### Alternative 3: AWS Step Functions style state machine

- AWS Step Functions provides declarative state machines and explicit service integration.
- AWS Step Functions has strong operational semantics for retries and timeouts.
- AWS Step Functions supports visual workflows and service orchestration.
- AWS Step Functions style JSON definitions are easy to diff and validate.
- A Step Functions style model is less expressive for local deterministic orchestration code.
- A Step Functions style model can become verbose for nested sagas and compensation logic.
- A Step Functions style model can push complex business branching into giant declarative graphs.
- A Step Functions style model is also cloud-provider shaped.
- The pattern was rejected as authoritative runtime.
- Selected pieces, including explicit retry blocks and typed state transitions, may influence spec design.

### Alternative 4: Airflow or Prefect DAG orchestration

- Airflow and Prefect provide familiar DAG-oriented batch orchestration.
- DAG orchestration is strong for scheduled data workflows.
- DAG orchestration has broad operator ecosystems.
- DAG orchestration can represent many acyclic business jobs.
- DAG orchestration does not naturally model long-lived interactive sagas.
- DAG orchestration does not naturally model arbitrary signals and compensation chains.
- DAG orchestration can make per-run deterministic replay secondary to scheduler state.
- DAG orchestration is less appropriate for product transaction workflows.
- Airflow and Prefect were rejected for runtime core.
- DAG projections may still be emitted for batch-like workflow analysis.

### Alternative 5: Ad hoc event choreography without a runtime

- Ad hoc choreography keeps each microservice locally simple at first.
- Ad hoc choreography avoids a central workflow-engine implementation.
- Ad hoc choreography can use event bus subscriptions and local retry tables.
- Ad hoc choreography is hard to reason about across multi-step failures.
- Ad hoc choreography scatters compensation logic across services.
- Ad hoc choreography makes tenant-scoped replay and pause/resume inconsistent.
- Ad hoc choreography makes evidence collection expensive and incomplete.
- Ad hoc choreography conflicts with ADR-0222 saga visibility requirements.
- Ad hoc choreography was rejected because Oyatie needs a durable substrate, not informal async chains.

## Consequences

- Positive: workflow-engine receives a single authoritative runtime model.
- Positive: Workflow Studio can evolve UI projections without changing runtime semantics.
- Positive: developers can express complex sagas with durable timers, activities, and compensation.
- Positive: replay evidence can be generated from event history without reconstructing external scheduler state.
- Positive: long-running workflows can survive worker restarts and process upgrades.
- Positive: Cedar authorization can be centralized around workflow operations.
- Positive: audit-chain can consume normalized lifecycle and compensation events.
- Positive: observability dashboards can use common run, task, and queue dimensions.
- Positive: business-readable BPMN remains possible as a projection rather than a runtime dependency.
- Negative: Oyatie must implement and operate a durable execution substrate.
- Negative: deterministic replay constraints will surprise engineers who are new to this model.
- Negative: workflow code versioning becomes strict and requires compatibility discipline.
- Negative: payload externalization adds object-store dependency for large workflow data.
- Negative: debugging requires history inspection tools rather than only reading current database rows.
- Negative: Studio import/export compilers must be tested carefully to avoid semantic drift.
- Neutral: BPMN remains useful for documentation and enterprise import.
- Neutral: Camunda remains a benchmark and compatibility reference, not a production runtime dependency.
- Neutral: some product teams may still define simple declarative workflows.
- Neutral: the engine can expose both code-first and spec-first authoring lanes.
- Follow-up: define `workflow_spec.v1.json` canonical schema in the workflow-engine contract package.
- Follow-up: define deterministic replay compatibility tests for every approved workflow definition.
- Follow-up: define worker SDK constraints for non-deterministic APIs, time, random, and external IO.
- Follow-up: define the BPMN import subset and explicit unsupported construct list.
- Follow-up: define history archival and continue-as-new retention policies.
- Follow-up: define stuck-workflow recovery runbooks for certified cells.
- Follow-up: define activity idempotency proof requirements for payments, governance, and compliance workflows.
- Follow-up: define per-cell queue admission limits and overload responses.
- Follow-up: define Studio projection error codes when diagrams cannot compile to runtime specs.
- Follow-up: define audit-chain event names for run started, run signaled, activity completed, compensation scheduled, and run closed.

## Implementation Notes

- Store workflow definitions in `workflow_definition`.
- Store immutable versions in `workflow_definition_version`.
- Store active runs in `workflow_run`.
- Store append-only history in `workflow_run_history`.
- Store pending workflow tasks in `workflow_task_queue`.
- Store pending activity tasks in `activity_task_queue`.
- Store durable timers in `workflow_timer`.
- Store signal envelopes in `workflow_signal`.
- Store compensation plans in `workflow_compensation`.
- Store large payload pointers in `workflow_payload_ref`.
- Use `tenant_id` as the first partition key for every logical table.
- Use `cell_id` as a required routing dimension for execution tables.
- Use `workflow_run_id` as a globally unique sortable identifier.
- Use `workflow_namespace` to isolate product domains and substrate jobs.
- Use `definition_id` and `definition_version` to bind a run to immutable behavior.
- Use `history_sequence` as a monotonic per-run integer.
- Use `event_id` as the stable identifier for replay and trace linking.
- Use optimistic concurrency on run state transitions.
- Use append-only writes for history events after run creation.
- Use idempotent inserts for activity completion by `(activity_id, attempt)`.
- Use queue leases with `lease_owner`, `lease_expires_at`, and `delivery_attempt`.
- Use dead-letter queues only after terminal policy evaluation.
- Use separate queues for workflow tasks and activity tasks.
- Use queue names in the form `cell.<cell_id>.tenant.<tenant_id>.namespace.<namespace>.<task_queue>`.
- Use the `WorkflowSpec` shape with fields `spec_id`, `version`, `namespace`, `entrypoint`, `steps`, `signals`, `timeouts`, `retry_defaults`, `compensation_defaults`, and `policy_tags`.
- Use the `WorkflowRun` shape with fields `run_id`, `tenant_id`, `cell_id`, `definition_id`, `definition_version`, `status`, `started_by`, `started_at`, `closed_at`, `current_history_size_bytes`, and `current_history_event_count`.
- Use the `RunHistoryEvent` shape with fields `event_id`, `run_id`, `sequence`, `event_type`, `occurred_at`, `attributes_hash`, `payload_ref`, `trace_id`, and `causation_event_id`.
- Use the `ActivityTask` shape with fields `activity_id`, `run_id`, `activity_type`, `task_queue`, `input_ref`, `idempotency_key`, `attempt`, `schedule_to_close_timeout`, `start_to_close_timeout`, and `heartbeat_timeout`.
- Use the `SagaCompensation` shape with fields `compensation_id`, `run_id`, `forward_activity_id`, `compensation_activity_type`, `status`, `reason_code`, `registered_at`, `started_at`, and `completed_at`.
- Use the `SignalEnvelope` shape with fields `signal_id`, `run_id`, `signal_name`, `sender_principal`, `payload_ref`, `dedupe_key`, `received_at`, and `policy_decision_id`.
- Use `POST /v1/workflow-engine/definitions` to register a draft workflow definition.
- Use `POST /v1/workflow-engine/definitions/{definition_id}/versions` to create an immutable approved version.
- Use `GET /v1/workflow-engine/definitions/{definition_id}/versions/{version}` to fetch an approved version.
- Use `POST /v1/workflow-engine/runs` to start a workflow run.
- Use `GET /v1/workflow-engine/runs/{run_id}` to inspect run status.
- Use `GET /v1/workflow-engine/runs/{run_id}/history` to inspect filtered history.
- Use `POST /v1/workflow-engine/runs/{run_id}/signals` to deliver a signal.
- Use `POST /v1/workflow-engine/runs/{run_id}/pause` to pause scheduling.
- Use `POST /v1/workflow-engine/runs/{run_id}/resume` to resume scheduling.
- Use `POST /v1/workflow-engine/runs/{run_id}/cancel` to request cancellation.
- Use `POST /v1/workflow-engine/runs/{run_id}/replay` to request deterministic replay verification.
- Use `POST /v1/workflow-engine/runs/{run_id}/continue-as-new` only from trusted runtime principals.
- Use `GET /v1/workflow-engine/runs/{run_id}/compensations` to list compensation state.
- Use `POST /v1/workflow-engine/runs/{run_id}/compensations/{compensation_id}/approve` for human-approved compensation gates.
- Use `POST /v1/workflow-engine/workers/poll` for worker task polling.
- Use `POST /v1/workflow-engine/workers/complete` for task completion.
- Use `POST /v1/workflow-engine/workers/heartbeat` for long-running activity heartbeat.
- Cedar action `workflow_engine::definition::create` allows draft creation by authorized designers and service principals.
- Cedar action `workflow_engine::definition::approve_version` requires definition owner, governance delegate, or certified release automation.
- Cedar action `workflow_engine::run::start` requires tenant scope, namespace permission, and definition version approval.
- Cedar action `workflow_engine::run::signal` requires signal-specific permission and run tenant match.
- Cedar action `workflow_engine::run::pause` requires operator permission and non-terminal run status.
- Cedar action `workflow_engine::run::replay` requires audit or engineering verifier permission.
- Cedar action `workflow_engine::run::history_read` redacts payload refs unless the principal has payload inspection rights.
- Cedar action `workflow_engine::compensation::approve` requires compensation policy tag and separation-of-duties checks.
- Cedar action `workflow_engine::worker::poll` requires worker registration, cell match, and task queue grant.
- Cedar action `workflow_engine::worker::complete` requires lease ownership and activity idempotency proof.
- Cedar action `workflow_engine::payload::read` requires tenant, cell, and data classification grants.
- SLO `workflow-start-latency` target is p99 under 500 ms.
- SLO `workflow-step-execute-latency` target is p99 under 250 ms for workflow task decisions.
- SLO `activity-dispatch-latency` target is p99 under 1 second for uncongested queues.
- SLO `worker-poll-availability` target is 99.95 percent monthly per certified cell.
- SLO `workflow-completion-availability` target is 99.9 percent monthly for eligible runs.
- SLO `replay-determinism-correctness` target is 100 percent for approved definitions.
- SLO `payload-bytes-budget-correctness` target is 100 percent rejection of over-limit inline payloads.
- Emit trace span `workflow_engine.run.start` for run creation.
- Emit trace span `workflow_engine.workflow_task.execute` for deterministic decision execution.
- Emit trace span `workflow_engine.activity.dispatch` for activity queue dispatch.
- Emit trace span `workflow_engine.activity.complete` for completion handling.
- Emit trace span `workflow_engine.signal.accept` for signal delivery.
- Emit trace span `workflow_engine.compensation.schedule` for compensation registration.
- Emit metric `workflow_engine_run_started_total` tagged by tenant, cell, namespace, and definition version.
- Emit metric `workflow_engine_run_closed_total` tagged by close status and reason.
- Emit metric `workflow_engine_history_events_total` tagged by event type.
- Emit metric `workflow_engine_activity_attempts_total` tagged by activity type and retry class.
- Emit metric `workflow_engine_replay_failures_total` tagged by definition version and failure class.
- Emit metric `workflow_engine_queue_depth` tagged by queue name and priority.
- Emit metric `workflow_engine_continue_as_new_total` tagged by reason.
- Dashboard `workflow-engine-execution-health` shows starts, completions, failures, and queue depth.
- Dashboard `workflow-engine-replay-determinism` shows replay pass rate and failure classes.
- Dashboard `workflow-engine-saga-compensation` shows compensation pending, approved, failed, and completed counts.
- Dashboard `workflow-engine-worker-pools` shows poll rate, lease age, heartbeat gaps, and backlog.
- Dashboard `workflow-engine-payload-budget` shows inline payload rejection and external payload storage use.
- Runbook `durable-execution-restart` must prove workers can restart without run loss.
- Runbook `stuck-workflow-recovery` must document pause, signal, compensation, and cancel order.
- Runbook `event-bus-replay` must distinguish external event replay from workflow history replay.
- Runbook `spec-rollback` must explain why approved definition versions are immutable and how to start a new version.

## Verification

- Test `workflow_replay_determinism_passes_for_approved_specs` replays every approved fixture.
- Test `workflow_replay_rejects_wall_clock_access` fails when workflow code reads system time outside deterministic APIs.
- Test `workflow_replay_rejects_random_access` fails when workflow code uses non-deterministic randomness.
- Test `workflow_history_is_append_only` verifies old history rows cannot be updated.
- Test `workflow_continue_as_new_at_event_limit` forces 50,000 events and expects a new run continuation.
- Test `workflow_continue_as_new_at_byte_limit` forces 50 MiB serialized history and expects continuation.
- Test `activity_payload_inline_limit` rejects payloads above 256 KiB.
- Test `workflow_spec_size_limit` rejects compiled specs above 2 MiB.
- Test `activity_completion_idempotency` accepts duplicate completion for the same attempt without duplicate side effects.
- Test `activity_completion_rejects_wrong_lease_owner` verifies worker lease ownership.
- Test `signal_delivery_requires_cedar_grant` denies cross-tenant signal injection.
- Test `history_read_redacts_payload_without_permission` verifies payload redaction.
- Test `compensation_approval_requires_separation_of_duties` enforces policy.
- Test `bpmn_import_emits_canonical_spec` compiles supported BPMN fixtures to `workflow_spec.v1.json`.
- Test `bpmn_import_rejects_unsupported_construct` returns explicit diagnostics.
- Test `run_pause_blocks_new_activity_dispatch` verifies paused runs stop scheduling.
- Test `run_resume_restarts_scheduling` verifies scheduling resumes after policy check.
- Test `cancel_schedules_compensation_when_required` verifies ADR-0222 behavior.
- Test `worker_heartbeat_timeout_reschedules_activity` verifies long-running activity leases.
- Test `worker_poll_requires_cell_match` denies cross-cell poll attempts.
- Metric check `workflow_engine_replay_failures_total == 0` for release candidate definitions.
- Metric check `histogram_quantile(0.99, workflow_engine_start_latency_seconds) < 0.5`.
- Metric check `histogram_quantile(0.99, workflow_engine_task_execute_seconds) < 0.25`.
- Metric check `workflow_engine_payload_inline_rejection_total` increments on oversized inline payloads.
- Metric check `workflow_engine_queue_depth` remains below admission threshold during load test.
- Metric check `workflow_engine_activity_attempts_total{retry_class="terminal"}` does not increase after terminal business errors.
- Dashboard check `workflow-engine-execution-health` renders run states for every certified cell.
- Dashboard check `workflow-engine-replay-determinism` exposes failure class and definition version.
- Dashboard check `workflow-engine-saga-compensation` links each compensation to the forward activity.
- Dashboard check `workflow-engine-worker-pools` shows heartbeat gaps before lease expiry.
- Audit check confirms run start, signal, activity completion, compensation, and closure events are emitted.
- Load test starts 10,000 short workflows in a certified cell and keeps p99 start latency under 500 ms.
- Failure test kills workers during activity execution and verifies no duplicate non-idempotent side effects.
- Upgrade test replays previous-version histories against new runtime binaries.
- Backpressure test saturates one activity queue and verifies unrelated queues continue dispatching.
- Security test attempts cross-tenant history read and expects Cedar denial.
- Compliance test exports replay evidence for a workflow attached to a certification-level tag.

## References

- ADR-0145, Inter Microservice Communication Reform, `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`.
- ADR-0222, Saga Compensation Portfolio Policy, `docs/decisions/ADR-0704-k8s-port-live-apex.md`.
- ADR-0243, Cedar as Universal Gate, `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- ADR-0244, Tenant as Universal Scoping Primitive, `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
- ADR-0245, Substrate vs Product Layering, `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`.
- ADR-0251, Compliance Pack Cell Certification Levels, `docs/decisions/ADR-0708-platform-foundations-live-apex.md`.
- ADR-0263, Observability Emission Contract, `docs/decisions/ADR-0706-observability-live-apex.md`.
- Temporal documentation, Workflows, https://docs.temporal.io/workflows.
- Temporal documentation, Durable Execution, https://docs.temporal.io/temporal.
- Temporal documentation, Saga pattern, https://docs.temporal.io/develop/go/saga.
- Camunda documentation, BPMN, https://docs.camunda.io/docs/components/modeler/bpmn/bpmn/.
- Object Management Group, Business Process Model and Notation 2.0.2, https://www.omg.org/spec/BPMN/2.0.2/.
- Hector Garcia-Molina and Kenneth Salem, Sagas, ACM SIGMOD 1987.
- AWS Step Functions Developer Guide, https://docs.aws.amazon.com/step-functions/latest/dg/welcome.html.
- Dapr documentation, Workflow building block, https://docs.dapr.io/developing-applications/building-blocks/workflow/.
- Restate documentation, Durable Execution, https://docs.restate.dev/.
