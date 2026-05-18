---
doc_class: ContractSpec
title: Backfill + Replay Contract
microservice: tasks
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-tasks
deciders: axis-tasks, council-architecture, ops-sre-reliability
related_adrs: [ADR-0028, ADR-0126, ADR-0131, ADR-TASKS-0001, ADR-TASKS-0002, ADR-TASKS-0003]
related_artifacts:
  - microservices/tasks/PRD.md
  - microservices/tasks/capacity-model.md
  - microservices/tasks/contracts/asyncapi/tasks-events.yaml
  - microservices/tasks/runbooks/search-index-rebuild.md
  - microservices/tasks/runbooks/recurring-task-materialisation-failure.md
review_cadence: annually
doc_status: published
---

# Backfill + Replay Contract (tasks µservice)

## Purpose

Specify how tasks handles four scenarios:

1. **Search-index rebuild** — Meilisearch per-tenant index rebuild from
   canonical Postgres task store (after corruption, after schema bump,
   after Cedar policy change that changes search projection rules).
2. **Recurring materialisation rebuild** — re-expansion of all
   RRULE-bound recurring tasks in a window (after RRULE engine
   upgrade per ADR-TASKS-0003, after timezone change inherited from
   calendar's tzdb refresh).
3. **Task-lifecycle replay** — re-fanout of historical task-lifecycle
   events to a newly subscribed downstream consumer (audit-chain,
   workflow-engine, observability, ontology, mail, messenger), or to
   replay missed events for a tenant onboarded mid-stream.
4. **Dependency-graph cycle re-scan** — full re-scan of dependency-
   edges in a project after a cycle-prevention algorithm upgrade or
   after a corruption-suspected incident (FM-03).

## Search-index rebuild

### Contract

Trigger sources:
- Operator-invoked: `cargo run -p oya-dev-cli -- tasks rebuild-search-index --tenant <t>`.
- Auto: Meilisearch cluster recovery triggers full rebuild; degraded mode (direct-Postgres-trigram) bridges until rebuild completes.
- Auto: schema-change in `oya-tasks-search-index-domain` projection schema bumps version; affected tenants rebuild.
- Auto: Cedar policy change in `policy/search-projection.cedar` invalidates affected per-tenant indexes.

Procedure:

1. Acquire rebuild lease in Redis (per tenant; lease TTL = 1h).
2. Enumerate Postgres `tasks_task` rows partitioned by `(tenant_id, project_id_hash)`.
3. For each task, compute the search projection per current `oya-tasks-search-index-domain` schema (respecting context isolation + redaction rules).
4. Bulk-write to Meilisearch with per-tenant index name + idempotency key `(tenant_id, task_id, schema_version)`.
5. Emit `TaskSearchIndexRebuilt` event with tuple `(tenant_id, schema_version, row_count, completed_at, signature)`.
6. Per-pack retention: rebuild window bounded by retention floor.

### Performance

- Rebuild rate target: ≥3000 tasks/sec per partition.
- A 10M-task tenant rebuilds in ≤30 minutes per AC-09.
- Avoid `runbooks/search-index-rebuild.md` Sev-2 triggering — backfill should be the slow path, not the panic path.

## Recurring materialisation rebuild

### Trigger sources

- RRULE engine version bump (per ADR-TASKS-0003 — when `rrule-rs` LTS
  pin moves and the named edge-case test matrix asserts a behaviour
  change; shared pin with calendar ADR-CAL-0002).
- Timezone change inherited from calendar's tzdb refresh worker.
- Operator-invoked: `cargo run -p oya-dev-cli -- tasks rebuild-recurrence --tenant <t> --window <yyyy-mm-dd..yyyy-mm-dd>`.

### Contract

1. Per-tenant lock on the recurrence-engine for the window.
2. Enumerate all tasks with non-null RRULE in the window.
3. Re-expand each per the (new) engine.
4. Compare to prior materialisation; emit per-task diff record.
5. Write the new materialisation to Postgres.
6. Emit `TaskRecurrenceMaterialisationRebuilt` with diff summary `(tenant_id, window, task_count, diff_count)`.
7. For tasks with diff_count > 0 in the next 30 days, emit `TaskRecurrenceMaterialisationRebuildAffectedAssignees` to notify the project owner; tenant policy decides whether to auto-update assignments.

### Bounded materialisation

Per ADR-TASKS-0003, no expansion may exceed 5y horizon. Backfill explicitly refuses windows > 5y.

## Task-lifecycle replay

### Trigger sources

- New downstream consumer onboarded (e.g., a new audit-chain instance needs to replay 30d of history).
- Tenant onboarded mid-stream (the Workflow-engine binding needs to catch up on the tenant's recent task history).
- Consumer requests replay for a specific time window for debugging / forensics.

### Contract

1. Acquire replay lease per (tenant_id, consumer_id) — replay leases are exclusive to prevent double-delivery.
2. Snapshot the `tasks.task.lifecycle.v1` event log in `(tenant_id, partition_id)` partition, ordered by `task_id + version`.
3. Stream events in batches of 1000 → consumer's Workflow webhook (idempotent per `task_id + version`).
4. After bulk, emit `TaskLifecycleReplayCompleted` with tuple `(tenant_id, consumer_id, task_count, completed_at, signature)`.
5. Per-pack retention: replay window bounded by retention floor (no replay of expired-retention tasks).

### Performance

- Replay rate: ≥1000 events/sec per consumer.
- Idempotency: every replayed event carries `(task_id, version, replay_attempt_n, original_emitted_at)` for the consumer to deduplicate.

## Dependency-graph cycle re-scan

### Special case

If FM-03 (dependency-cycle corruption) fires or if ADR-TASKS-0002 cycle-prevention algorithm upgrades, a full re-scan is required:

1. Enumerate all dependency-edges for affected project in Postgres.
2. Run cycle-detection BFS over the full graph.
3. Surface every cycle to tenant operator via `runbooks/dependency-cycle-corruption.md` workflow.
4. Optionally, in dry-run mode, identify the minimal edge set to break each cycle; operator approves removals.
5. Emit `TaskDependencyGraphCycleScanCompleted` with `(project_id, cycle_count, edge_count)`.

## Per-µservice consumer contracts

| Downstream | Replay onboarding | Replay catch-up window |
|---|---|---|
| `audit-chain` | replay from tenant onboarding | full retention horizon (per-pack; up to 5y for KR + 6y for HIPAA) |
| `workflow-engine` | replay last 30d on consumer onboarding | 30d default; configurable to 90d |
| `observability` | replay last 24h on consumer onboarding | 24h default; configurable to 7d |
| `mail` (notification bridge) | no replay (mail is downstream-only; lost notifications NOT replayed because they may have already been delivered) | n/a |
| `messenger` (notification bridge) | no replay (same reasoning as mail) | n/a |
| `calendar` (due-date bridge) | replay last 7d on consumer onboarding | 7d default |
| `drive` (attachment bridge) | no replay (attachments are write-only from tasks; deletions are explicit) | n/a |
| `ontology` | replay last 7d on consumer onboarding | 7d default |
| `tenancy` | no replay (tenancy is upstream-only) | n/a |
| `foundry-runtime` | replay last 7d on consumer onboarding (per ADR-TASKS-0006 retention for AI inference inputs) | 7d default |

## Verification

- [ ] Backfill / replay rate ≥3000 tasks/sec measured in benchmark `cargo bench -p oya-tasks-search-index-worker -- full_rebuild`.
- [ ] Backfill idempotency property test passes — `cargo nextest run -p oya-tasks-task-store-domain -- backfill_idempotent`.
- [ ] Replay window bounded by retention — `cargo nextest run -p oya-tasks-task-store-domain -- replay_retention_bound`.
- [ ] Cycle re-scan identifies all cycles in a known-corrupt fixture — `cargo nextest run -p oya-tasks-dependency-graph-domain -- cycle_scan_corpus`.

## References

- ADR-0028 — Audit-chain (Ed25519 + Merkle).
- ADR-TASKS-0001 (custom-field strict coercion).
- ADR-TASKS-0002 (dependency-cycle prevention).
- ADR-TASKS-0003 (recurrence; rrule-rs alignment with calendar ADR-CAL-0002).
- `microservices/tasks/contracts/asyncapi/tasks-events.yaml`.
- `microservices/tasks/runbooks/search-index-rebuild.md`.
- `microservices/tasks/runbooks/recurring-task-materialisation-failure.md`.
- `microservices/tasks/runbooks/dependency-cycle-corruption.md`.
- `microservices/calendar/backfill-replay.md` — sibling reference template.
