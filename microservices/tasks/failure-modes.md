---
doc_class: FailureModes
template_id: TPL-FAILURE-MODES
microservice: tasks
status: Accepted
date: 2026-05-17
owner_team: axis-tasks + ops-sre-reliability
methodology: STAMP + FMEA + Google SRE
related_adrs: [ADR-0130, ADR-0131]
doc_status: published
---

# Failure Modes — tasks µservice

## Purpose

Enumerate failure modes, blast radius, detection signals, automated recovery, and operational runbooks. Each failure mode has at least one runbook in `runbooks/` and at least one SLO + alert in `dashboards/`.

## Failure-mode catalog

### FM-01 — Recurring-task materialisation engine OOM on complex RRULE

- **Cause:** RRULE with deeply-nested BYSETPOS + BYDAY + BYMONTH + EXDATE on 5y horizon exhausts worker heap.
- **Blast radius:** affected worker pod crash; pending materialisations queued; tenant sees recurring-task instances delayed.
- **Detection:** Pod OOMKilled event in K8s; worker queue depth > 60s of cadence; `recurring_materialise_p99_ms` exceeds 1s.
- **Automated recovery:** Worker restart by K8s; pre-warmed pool absorbs; materialisation retry with smaller window batches.
- **Runbook:** `runbooks/recurring-task-materialisation-failure.md`.
- **Mitigation hardening:** RRULE complexity bound at API; worker memory request 2GB + limit 4GB; horizon hard-cap 5y per ADR-TASKS-0003.

### FM-02 — Custom-field schema migration failure mid-flight

- **Cause:** Tenant operator alters custom-field schema while in-flight tasks reference the schema; coercion fails mid-migration.
- **Blast radius:** Some tasks have new schema; some retain old; reads error with `CustomFieldSchemaMismatch`.
- **Detection:** Migration job state machine emits `state=failed`; `tasks_schema_migration_failure_rate > 0.1%` alert.
- **Automated recovery:** Schema migrations are transactional + idempotent; partial-migration state rolls back on next worker run.
- **Runbook:** `runbooks/custom-field-schema-migration.md`.

### FM-03 — Dependency-graph cycle corruption (latent cycle written before cycle-prevention deployed)

- **Cause:** Legacy data imported pre-cycle-prevention contains cycles; new writes to those projects encounter cycles + emit `DependencyCycle::Refused`.
- **Blast radius:** Affected tenant cannot mutate dependency-graph in corrupt project until corrupted edges identified + removed.
- **Detection:** `tasks_dependency_cycle_refused_count` > 0 sustained on a previously-quiescent project; pattern is "cluster of refusals on adjacent task IDs".
- **Automated recovery:** None — manual triage required. Runbook describes the cycle-detection + manual-removal procedure.
- **Runbook:** `runbooks/dependency-cycle-corruption.md`.
- **Mitigation hardening:** Pre-import cycle scan in `oya-tasks-importers-domain` refuses any import that would create a cycle; documented in Hyrum surface #3.

### FM-04 — Search-index rebuild failure / cold Meilisearch

- **Cause:** Meilisearch cluster outage; or schema-incompatible rebuild attempt; or storage exhaustion mid-rebuild.
- **Blast radius:** Cross-project search degraded to direct-Postgres-trigram fallback (per AC-09 graceful degradation); slower but functional.
- **Detection:** `tasks_search_index_cluster_health != green`; `tasks_search_query_p99_ms > 1s`; rebuild-job state `failed`.
- **Automated recovery:** Degraded mode activates within 1 min; full rebuild from Postgres can be re-triggered after cluster recovery.
- **Runbook:** `runbooks/search-index-rebuild.md`.

### FM-05 — Bulk-edit throttle exhaustion (tenant submits 100k-task bulk-edit)

- **Cause:** Tenant operator initiates a 100k-task bulk-edit without second-confirmation pre-check.
- **Blast radius:** Postgres write storm; tenant's other operations queued; other tenants unaffected (per-tenant rate-limit).
- **Detection:** `tasks_bulk_edit_in_flight_count > 1`; per-tenant rate-limit-exhausted counter increments.
- **Automated recovery:** Per-batch atomicity (1000 tasks/batch) prevents full-tenant lockout; throttle to baseline-tier rate; second-confirmation prompt fires.
- **Runbook:** `runbooks/bulk-edit-throttle.md`.

### FM-06 — Webhook fanout degraded (subscriber unhealthy)

- **Cause:** Tenant's webhook destination is unhealthy (500s / DNS failure / SSL expired).
- **Blast radius:** Per-subscriber circuit-breaker opens; events queued for retry; other subscribers' fanout unaffected.
- **Detection:** Per-subscription `tasks_webhook_circuit_breaker_open_count > 0`; per-subscription error-rate > 50%.
- **Automated recovery:** Exponential backoff + circuit-breaker; after 1h without success, mark subscription `held`; tenant operator notified via OnCall.
- **Runbook:** `runbooks/webhook-fanout-degraded.md`.

### FM-07 — AI auto-assign classifier produces biased output (employment-context)

- **Cause:** Classifier model drift; new training data introduces bias; per-protected-class accept-rate drops below fairness threshold.
- **Blast radius:** All T2 auto-assign decisions in that tenant + that pack-eu/pack-us employment-context may be biased; per-decision Ed25519 audit chain preserved for replay/rollback.
- **Detection:** `tasks_auto_assign_fairness_score` < 0.8 per protected class (defined per `slos/auto-assign-fairness-correctness.openslo.yaml`); weekly per-pack fairness audit.
- **Automated recovery:** Auto-rollback to prior model version per `runbooks/ai-assign-classifier-rollback.md`; surface to tenant + DPO + EU AI Act notified body.
- **Runbook:** `runbooks/ai-assign-classifier-rollback.md`.
- **Mitigation hardening:** Fairness gate at model promotion (CI); annual external bias audit per pack-eu / pack-us-employment.

### FM-08 — Postgres replica lag breaks read-after-write consistency

- **Cause:** Postgres replica lags primary by > 5s; user creates task, then refreshes and sees stale state.
- **Blast radius:** Tenant operational confusion; possible duplicate writes.
- **Detection:** `pg_replication_lag_seconds > 5` alert.
- **Automated recovery:** Read-after-write reads pinned to primary for 30s post-write; reads beyond use replica.
- **Runbook:** (see `microservices/calendar/runbooks/calendar-restore.md` equivalent for tasks; documented in incident-response.md).

### FM-09 — Workflow-engine bridge cycle (workflow creates task, task triggers workflow, loop)

- **Cause:** Misconfigured automation rule: tenant declares "when task created, run workflow X"; workflow X creates a new task; new task triggers workflow X; infinite loop.
- **Blast radius:** Tenant's task store fills with auto-created tasks; workflow-engine queues balloon.
- **Detection:** `tasks_workflow_bridge_cycle_count > 0`; per-tenant auto-created-task rate spike.
- **Automated recovery:** Workflow-engine durable-execution idempotency-key per ADR-TASKS-0005 limits per-tenant-per-workflow recursion; cycle-detection harness in workflow-engine.
- **Runbook:** workflow-engine runbook + `runbooks/webhook-fanout-degraded.md` adjacent (cross-µservice).

### FM-10 — Postgres connection pool exhaustion

- **Cause:** Burst of task writes + view-cache rebuilds + dependency-cycle-checks exhausts the per-pool max-connections.
- **Blast radius:** New requests queued; some time out; cascading 5xx.
- **Detection:** `pg_connection_pool_utilisation > 85%` alert.
- **Automated recovery:** HPA scales rest pods; short-term: rate-limit at REST layer.
- **Runbook:** runbook reused from calendar's `postgres-connection-storm.md` pattern.

### FM-11 — Cross-pack mesh partition during cross-µservice handoff

- **Cause:** Mesh partition between packs; cross-pack task creation (e.g., calendar bridge from pack-eu calendar to pack-kr tasks) times out.
- **Blast radius:** Cross-pack tasks delayed; other tenant operations unaffected (per-pack residency forbids cross-pack tasks anyway).
- **Detection:** `tasks_cross_pack_handoff_timeout_rate > 5%` alert.
- **Automated recovery:** Cross-pack timeout = 2s; on timeout, degrade to in-pack-only operation; surface to tenant.
- **Runbook:** workflow-engine + calendar cross-pack-mesh runbooks (cross-µservice).

### FM-12 — Importer payload-as-malware (Jira/Asana/Trello XML/JSON injection / XXE / billion-laughs)

- **Cause:** Tenant uploads a malicious source file targeting the importer parser.
- **Blast radius:** Subprocess sandbox prevents cluster-wide impact; only the affected import job dies; tenant sees `ImportFailed::ParserError`.
- **Detection:** Subprocess timeout / OOMKilled in cgroup; sandbox returns exit code != 0; importer worker emits `state=failed`.
- **Automated recovery:** Subprocess sandbox + cgroup memory cap + 5min timeout; failed import surfaced to tenant with redacted error message.
- **Runbook:** runbook reused from calendar's `ics-import-failure.md` pattern.

### FM-13 — Search-index data leak (tenant A's task indexed under tenant B's prefix due to bug)

- **Cause:** Bug in adapter-meilisearch prefixes the wrong tenant_id when indexing.
- **Blast radius:** Tenant B can search for and view tenant A's task content (catastrophic).
- **Detection:** `oya-check-search-index-tenant-prefix` LEAN check + property test on adapter; runtime invariant test in tasks/tests/e2e/search-tenant-isolation.rs.
- **Automated recovery:** Per-tenant index name + master key prefix prevents at the Meilisearch ACL level; even if adapter mis-prefixes, Meilisearch refuses cross-tenant query.
- **Runbook:** Sev-1 incident; runbook reused from generic Sev-1 cross-tenant leak protocol in incident-response.md.

### FM-14 — Audit-chain emission failure (silent)

- **Cause:** audit-chain µservice ingest endpoint returns 5xx; tasks's emission ack times out.
- **Blast radius:** Audit-chain seal missing for tasks created during outage; SOC 2 / ISO 27001 audit-coverage gap.
- **Detection:** `tasks_audit_emission_ack_lag_seconds > 30` alert.
- **Automated recovery:** Task write blocks (fail-closed) when emission ack > 30s; user sees "operation pending due to audit-chain unavailable"; ack-or-fail.
- **Runbook:** runbook reused from calendar's `audit-chain-emission-recovery.md` pattern.

### FM-15 — Time-tracking tick worker queue overflow (M02+)

- **Cause:** 1Hz tick rate × many concurrent timers × multi-tenant → tick worker queue grows.
- **Blast radius:** Time-tracking ticks delayed in append; ticks > 5s old may be coalesced.
- **Detection:** `tasks_time_tracking_tick_queue_depth_seconds > 5` alert.
- **Automated recovery:** Worker pod HPA scales out; queue drains within 1 min.
- **Runbook:** new runbook deferred to M02+; pattern follows webhook-fanout-degraded.md.

## Failure-mode aggregation gates

- `oya gate validate failure-mode-coverage --microservice tasks`: refuses build if any new code path lacks at least one failure-mode entry.
- Quarterly failure-mode review.
- Annual game-day: simulate FM-01, FM-04, FM-07, FM-13, FM-14 end-to-end.

## References

- ADR-0130: SLO-gated promotion.
- ADR-0131: per-microservice layout.
- ADR-TASKS-0002 (dependency-cycle); ADR-TASKS-0003 (rrule); ADR-TASKS-0005 (workflow-bridge); ADR-TASKS-0006 (AI fairness).
- `runbooks/*.md` (one per failure mode where tasks-specific; cross-µservice ones reuse calendar's).
- Google SRE Workbook ch. 6 (managing risk) + ch. 11 (managing incidents).
- NASA-STD-8729.1 (System Theoretic Accident Model and Processes, STAMP).
- `microservices/calendar/failure-modes.md` — sibling reference template.
