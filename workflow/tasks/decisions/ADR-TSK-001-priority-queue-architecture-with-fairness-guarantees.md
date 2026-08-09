---
id: ADR-TSK-001
title: Priority Queue Architecture with Per-Tenant Fairness Guarantees
status: Proposed
date: 2026-05-20
microservice: tasks
related_oyatie_adrs:
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0705-product-protocol-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
decision_owner: axis-tasks
---

# ADR-TSK-001: Priority Queue Architecture with Per-Tenant Fairness Guarantees

## Context

- Tasks owns task creation, board views, recurring materialization, dependency checks, bulk edits, webhook fanout, automation triggers, and AI-assisted assignment.
- The existing task data model ADRs define task shape, dependency graph behavior, recurrence, board realtime, and automation boundaries.
- This ADR binds the queueing architecture used when task work leaves synchronous request handling.
- Named pressure TSK-P1: project managers expect urgent work to outrank routine background fanout.
- Named pressure TSK-P2: small tenants must not be starved by a large tenant importing a Jira backlog.
- Named pressure TSK-P3: recurring task materialization must not block direct user edits.
- Named pressure TSK-P4: automation-trigger fanout must keep deterministic replay evidence.
- Named pressure TSK-P5: AI auto-assignment cannot consume the same worker capacity as state-transition correctness.
- Named precedent: AWS SQS FIFO message groups show how ordering can be scoped instead of global.
- Named precedent: Google Cloud Tasks uses rate-limited queues with target-specific dispatch control.
- Named precedent: Kubernetes scheduling uses priority plus fairness controls rather than one global FIFO.
- Constraint TSK-C1: tenant scope comes from ADR-0244 and cannot be inferred from task payload fields.
- Constraint TSK-C2: every enqueue, lease, retry, deadletter, and priority override emits evidence per ADR-0263.
- Constraint TSK-C3: Cedar must authorize priority override, manual requeue, and deadletter replay per ADR-0243.
- Constraint TSK-C4: public queue APIs must remain additive under ADR-0258.
- Constraint TSK-C5: batch imports, recurring materialization, webhook fanout, and automation runs require different SLO classes.
- Constraint TSK-C6: cross-region queues must not dispatch a task into a region where its tenant pack forbids processing.
- Constraint TSK-C7: queue state must be rebuildable from task-store and audit-chain events after loss of Valkey or worker state.
- Constraint TSK-C8: fairness must be measurable from production telemetry, not argued from scheduler intent.
- Constraint TSK-C9: paid capacity policy may raise rate ceilings but must not remove starvation protection.
- Constraint TSK-C10: manual support overrides must be audited and expire automatically.
- The task service already exposes latency SLOs for create, update, bulk update, recurring materialization, search, and webhook fire.
- A single global priority queue would satisfy urgent work latency in the happy path.
- A single global priority queue would let one tenant's high-priority storm consume all workers.
- Per-tenant isolated queues would protect fairness but waste capacity during idle periods.
- The design must combine global capacity use with per-tenant fairness and bounded urgent-lane escape hatches.
- This decision is service-local and does not modify workflow-engine orchestration semantics.

## Decision

- Adopt a weighted fair priority queue architecture, not one global priority queue.
- Use one logical queue family named `tasks.dispatch.v1`.
- Partition work by `tenant_id`, `work_class`, `home_cell`, and `priority_band`.
- Keep one physical durable queue per cell and work class.
- Store fairness state in a scheduler ledger keyed by `(cell_id, tenant_id, work_class)`.
- Use deficit round robin as the fairness algorithm for tenant dispatch.
- Use priority bands inside each tenant bucket, not across all tenants globally.
- Define priority bands as `critical`, `interactive`, `standard`, `bulk`, and `maintenance`.
- Reserve `critical` for correctness and user-visible incident recovery only.
- Require Cedar permit `tasks::queue::priority_override` for any manual elevation to `critical`.
- Assign default band `interactive` to user edits, task moves, comments, and board updates.
- Assign default band `standard` to recurrence, dependency recalculation, search sync, and lightweight webhooks.
- Assign default band `bulk` to imports, migrations, and multi-select edits affecting more than 500 tasks.
- Assign default band `maintenance` to compaction, stale view cleanup, and backfill.
- Use tenant weights derived from paid capacity policy and active incident state.
- Cap paid-capacity weight amplification at 8x over baseline.
- Enforce a minimum service quantum for every active tenant bucket.
- Run urgent work through the same fairness ledger so urgency cannot bypass tenant starvation controls.
- Allow one incident-only break-glass lane with a 15-minute TTL and forced council-audit event.
- Represent every queue item as an immutable `TaskDispatchEnvelope`.
- Store payload references, not large task payloads, inside the dispatch envelope.
- Use idempotency key `{tenant_id}:{work_kind}:{subject_id}:{causal_event_id}`.
- Lease work with a short visibility timeout and heartbeat extension.
- Deadletter after bounded retries by error class, not by one static retry count.
- Replay deadletters only through an audited endpoint.
- Use Valkey Streams or Valkey Streams for hot dispatch state.
- Use Postgres append-only `task_dispatch_ledger` as the recovery authority.
- Rebuild streams from ledger when the hot queue is lost.
- Publish CloudEvents for queue lifecycle transitions.
- Keep queue admission synchronous enough to reject unauthorized or impossible work before enqueue.
- Keep worker execution asynchronous and backpressure-aware.
- Treat fairness violation as an SLO breach, not a tuning preference.

## Alternatives Considered

### Single Global Priority Queue

- Pros: simplest worker implementation.
- Pros: urgent work is easy to push to the front.
- Pros: dashboards and capacity math are initially straightforward.
- Cons: one tenant can starve the whole platform with priority storm traffic.
- Cons: priority inflation becomes a product-support arms race.
- Cons: sovereign-cell limits become bolted onto dispatch instead of native.
- Rejected because global priority optimizes the loudest workload and fails tenant fairness.

### Per-Tenant Isolated Queues

- Pros: strong starvation isolation.
- Pros: easy tenant-specific throttling.
- Pros: tenant queue deletion and replay are clear.
- Cons: idle capacity cannot be reused without extra work-stealing semantics.
- Cons: thousands of small tenants create queue-management overhead.
- Cons: global incident response must scan too many queues.
- Rejected because strict isolation wastes capacity and complicates operations at scale.

### FIFO Queue with Worker-Side Priority Checks

- Pros: easy to layer onto existing brokers.
- Pros: minimal schema change.
- Pros: preserves enqueue order.
- Cons: priority is not visible at the broker and cannot drive capacity.
- Cons: head-of-line blocking remains.
- Cons: fairness cannot be proven without replaying worker decisions.
- Rejected because fairness and priority must be scheduler-visible.

### Workflow-Engine Delegated Scheduling

- Pros: reuses workflow-engine orchestration controls.
- Pros: centralizes retries and backoff.
- Pros: avoids a task-local scheduler.
- Cons: task board edits and recurrence need lower latency than generic workflow orchestration.
- Cons: workflow-engine outage would block core task correctness.
- Cons: service-local data shapes would leak into a substrate service.
- Rejected because tasks needs a local dispatch plane for product-critical work.

### Weighted Fair Priority Queue

- Pros: preserves urgent work within tenant bounds.
- Pros: uses idle capacity while preventing starvation.
- Pros: gives SRE one fairness dashboard across work classes.
- Cons: scheduler logic is more complex than FIFO.
- Cons: fairness weights require governance and telemetry.
- Cons: operators must understand deficit counters during incidents.
- Accepted because it matches the workload shape and makes fairness observable.

## Consequences

- Positive: direct user edits stay responsive during imports and backfills.
- Positive: small tenants keep a guaranteed dispatch quantum.
- Positive: paid tenants can buy more capacity without removing global fairness floors.
- Positive: incident overrides become explicit, expiring, and auditable.
- Positive: scheduler state can be rebuilt from append-only ledger rows.
- Positive: recurring materialization can scale independently from board realtime work.
- Positive: webhook fanout retries stop competing with task-state correctness.
- Positive: support can explain queue delay from tenant bucket, priority band, and deficit state.
- Negative: dispatch code must maintain and test fairness invariants.
- Negative: a misconfigured tenant weight can create visible latency skew.
- Negative: queue observability requires more cardinality budgeting.
- Negative: workers need idempotent execution because leases can expire and retry.
- Negative: urgent-lane misuse becomes a governance concern.
- Neutral: the queue architecture does not change task schema or board schema.
- Neutral: workflow-engine still owns cross-service orchestration; tasks owns service-local dispatch.
- Neutral: work classes can be added additively under `/v1/tasks/dispatch`.
- Neutral: broker choice can change if the envelope and ledger contracts remain stable.
- Neutral: batch imports remain asynchronous and user-visible with progress streaming.

## Implementation Notes

- Data shape `TaskDispatchEnvelope`: `{tenant_id, cell_id, work_id, work_kind, subject_id, priority_band, causal_event_id, idempotency_key, payload_ref, enqueue_reason, created_at}`.
- Data shape `TenantQueueBucket`: `{tenant_id, work_class, weight, deficit, min_quantum, last_served_at, active_break_glass_until}`.
- Data shape `TaskDispatchLease`: `{lease_id, work_id, worker_id, visibility_expires_at, heartbeat_at, attempt, execution_cell}`.
- Data shape `TaskDeadletter`: `{work_id, tenant_id, work_kind, last_error_class, attempts, deadlettered_at, replay_policy}`.
- Data shape `PriorityOverride`: `{tenant_id, work_id, from_band, to_band, reason, approved_by, expires_at, audit_event_id}`.
- Ledger table `task_dispatch_ledger` is append-only and partitioned by `cell_id` and `created_at`.
- Ledger row includes the immutable envelope hash.
- Ledger row includes the scheduler decision hash for lease events.
- Hot stream key pattern is `tasks:dispatch:{cell_id}:{work_class}`.
- Fairness ledger key pattern is `tasks:fairness:{cell_id}:{work_class}:{tenant_id}`.
- REST endpoint `POST /v1/tasks/dispatch/enqueue` accepts service-local enqueue requests from trusted adapters.
- REST endpoint `POST /v1/tasks/dispatch/{work_id}/priority` performs Cedar-authorized priority override.
- REST endpoint `POST /v1/tasks/dispatch/{work_id}/leases` leases work to a worker.
- REST endpoint `POST /v1/tasks/dispatch/{work_id}/heartbeat` extends an active lease.
- REST endpoint `POST /v1/tasks/dispatch/{work_id}/complete` records successful completion.
- REST endpoint `POST /v1/tasks/dispatch/{work_id}/deadletter/replay` replays through a new work id.
- AsyncAPI channel `tasks.dispatch.enqueued.v1` publishes accepted queue admission.
- AsyncAPI channel `tasks.dispatch.leased.v1` publishes scheduler selection.
- AsyncAPI channel `tasks.dispatch.completed.v1` publishes worker completion.
- AsyncAPI channel `tasks.dispatch.deadlettered.v1` publishes terminal retry state.
- AsyncAPI channel `tasks.dispatch.priority_overridden.v1` publishes manual priority changes.
- Cedar permit `tasks::queue::enqueue` requires service identity and matching tenant scope.
- Cedar permit `tasks::queue::priority_override` requires tenant operator role, incident context, and expiry.
- Cedar forbid `tasks::queue::priority_override` when `context.reason == ""`.
- Cedar permit `tasks::queue::deadletter_replay` requires support role and original payload hash match.
- Cedar forbid `tasks::queue::cross_cell_dispatch` when `resource.home_cell != context.execution_cell`.
- Audit event `EVT-TASKS-DISPATCH-ENQUEUED` includes work id, class, band, and envelope hash.
- Audit event `EVT-TASKS-DISPATCH-LEASED` includes selected tenant, deficit before, deficit after, and worker id.
- Audit event `EVT-TASKS-DISPATCH-DEADLETTERED` includes error class and retry profile.
- Audit event `EVT-TASKS-PRIORITY-OVERRIDDEN` includes approval principal and TTL.
- Metric `tasks_dispatch_lag_seconds` is histogrammed by cell, work class, band, and tenant_class.
- Metric `tasks_dispatch_fairness_deficit` tracks scheduler deficit by tenant bucket with top-k export only.
- Metric `tasks_dispatch_starvation_seconds` measures time since last served for active tenant buckets.
- Metric `tasks_dispatch_deadletter_total` is counted by work kind and error class.
- Metric `tasks_dispatch_break_glass_active` pages when any break-glass lane is active beyond 15 minutes.
- Trace span `tasks.dispatch.enqueue` records Cedar decision id and idempotency key hash.
- Trace span `tasks.dispatch.schedule` records selected bucket and priority band.
- Trace span `tasks.worker.execute` links back to the causal task event.
- Log schema `TaskDispatchDecisionLog` contains `tenant_id_hash`, `work_class`, `band`, `decision`, and `reason_code`.
- SLO target: interactive dispatch p99 <= 2 seconds per home cell.
- SLO target: recurring materialization dispatch p99 <= 60 seconds.
- SLO target: webhook fanout dispatch p99 <= 30 seconds excluding downstream outages.
- SLO target: active tenant starvation max <= 90 seconds during normal operation.
- SLO target: fairness violation count equals zero outside declared incidents.
- Capacity math: if peak interactive arrival is 5,000 work items per second and p95 schedule latency is 10 ms, Little's Law yields 50 in-flight scheduler decisions before safety factor.
- Capacity math: provision 500 scheduler decision slots per cell for 10x burst.
- Capacity math: if 2,000 tenants are active and min quantum is 1 item per 90 seconds, the floor requires 23 items per second, leaving headroom for paid capacity weights.
- Capacity math: a 1 million task import at 200 items per second finishes in 83 minutes without exhausting interactive buckets.
- Rollback path: disable new priority override writes and continue serving existing buckets.
- Rollback path: rebuild hot streams from `task_dispatch_ledger` up to last completed event.
- Rollback path: demote all active break-glass lanes after audit emission if scheduler math regresses.
- Multi-region path: enqueue in tenant home cell and replicate lifecycle events read-only to remote dashboards.
- Sovereign-cell path: KR, EU, CN-PIPL, FedRAMP-High, and IL5/6 packs force dispatch execution inside approved cell set.
- Versioning: `TaskDispatchEnvelope` v1 is additive only.
- Deprecation: priority band names require 180-day compatibility for dashboards and SDKs.

## Verification

- Unit test `enqueue_requires_tenant_scope_from_context` rejects payload-forged tenant ids.
- Unit test `priority_override_requires_expiry_and_reason` proves Cedar guard shape.
- Unit test `deficit_round_robin_serves_small_tenant_under_large_import` proves starvation floor.
- Unit test `critical_band_does_not_bypass_tenant_bucket` prevents global urgent jumps.
- Unit test `deadletter_replay_requires_payload_hash_match` prevents mutation during replay.
- Property test `scheduler_never_serves_zero_weight_bucket` generates tenant weight maps.
- Property test `active_bucket_served_within_starvation_budget` generates mixed tenant arrivals.
- Property test `idempotency_key_dedupes_retried_enqueue` covers duplicate causal events.
- Fuzz test `dispatch_envelope_parser_rejects_unknown_required_fields` protects worker intake.
- Integration test `recurring_materialization_does_not_block_task_update` simulates mixed work classes.
- Integration test `bulk_import_one_million_tasks_preserves_interactive_p99` validates queue split.
- Integration test `cross_cell_dispatch_forbidden_for_sovereign_pack` validates residency.
- Integration test `hot_stream_rebuild_from_ledger` deletes hot state and recovers dispatch.
- Load test `tasks_dispatch_5000_interactive_per_second` keeps schedule p95 below 10 ms.
- Load test `tasks_import_1m_with_2000_active_tenants` checks fairness SLO.
- Chaos test `worker_crash_after_lease_retries_idempotently` verifies lease expiry.
- Chaos test `audit_chain_backpressure_blocks_priority_override` proves evidence-first behavior.
- Metric check: dashboard `tasks/throughput-and-engagement` adds dispatch lag and starvation panels.
- Metric check: dashboard `tasks/automation-and-ai-quality` separates AI work from correctness work.
- Alert check: `tasks_dispatch_starvation_seconds` above 90 seconds pages SRE.
- Audit check: every manual priority override emits `EVT-TASKS-PRIORITY-OVERRIDDEN`.
- Static check: worker code cannot deserialize queue payloads without `tenant_id` and `idempotency_key`.
- Contract check: OpenAPI documents the dispatch endpoints as internal service APIs.
- Regression check: existing task create/update SLO files remain unchanged.

## References

- AWS SQS FIFO message group documentation.
- Google Cloud Tasks queue and dispatch-rate documentation.
- Kubernetes scheduler priority and fairness design notes.
- Valkey Streams consumer group documentation.
- Valkey Streams compatibility documentation.
- CloudEvents 1.0.2 specification.
- Cedar policy language documentation.
- ADR-0244 tenant-as-universal-scoping-primitive.
- ADR-0243 Cedar-as-universal-gate.
- ADR-0263 observability-emission-contract.
- ADR-0258 API-versioning-model.
- microservices/tasks/PRD.md.
- microservices/tasks/capacity-model.md.
- microservices/tasks/runbooks/bulk-edit-throttle.md.
- microservices/tasks/runbooks/recurring-task-materialisation-failure.md.
- microservices/tasks/runbooks/webhook-fanout-degraded.md.
