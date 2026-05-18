---
doc_class: FailureModeCatalog
title: Failure-Mode Catalog
microservice: workflow-engine
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-workflow
deciders: ops-sre-reliability, axis-workflow, ops-security, council-architecture
related_adrs: [ADR-0035, ADR-0103, ADR-0117, ADR-0131]
related_artifacts:
  - microservices/workflow-engine/threat-model.md
  - microservices/workflow-engine/dpia.md
  - microservices/workflow-engine/incident-response.md
  - microservices/workflow-engine/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident
doc_status: published
---

# Failure-Mode Catalog (workflow-engine µservice)

## Purpose

Enumerate the failure scenarios on-call must handle, the detection signal for each, immediate mitigation, RCA path, RTO, and the runbook that owns the recovery procedure. Cross-referenced from `incident-response.md` for severity classification.

## Failure-Mode Index

Each failure carries:
- **FM-ID**: stable identifier
- **Trigger**: precipitating event(s)
- **Detection**: SLI / alert / metric that fires
- **Tenant impact**: what tenants experience
- **Severity**: Sev-1/2/3/4
- **Immediate mitigation**: actions on-call performs in first 5 minutes
- **RTO**: target recovery time
- **Recovery runbook**: where the procedure lives
- **Postmortem owner**

## FM-01: State-machine deadlock — two runs each wait on the other

| Field | Value |
|---|---|
| Trigger | Spec authors a cross-run wait dependency; runtime resolves to a cycle |
| Detection | Cycle detector emits `oya_workflow_engine_deadlock_cycle_detected_total > 0`; SLA timer on stuck runs > 5min |
| Tenant impact | Affected runs stuck indefinitely; cascades to downstream subscribers waiting on completion event |
| Severity | Sev-2 (single-tenant) / Sev-1 if production-tier cross-tenant cascade |
| Immediate mitigation | Cycle-tear-down primitive: identify youngest run in cycle, fail it with `DeadlockBroken` reason; emit audit-chain seal; notify tenant |
| RTO | ≤ 5min cycle break; manual remediation ≤ 1h |
| Recovery runbook | `runbooks/deadlock-resolution.md` |
| Postmortem owner | axis-workflow |

## FM-02: Event-bus backpressure — subscriber slow, queue grows, engine OOM-imminent

| Field | Value |
|---|---|
| Trigger | One subscriber's consume rate < publish rate sustained; queue grows; ingester memory climbs |
| Detection | `oya_workflow_engine_event_bus_consumer_lag_seconds > 60` for one subscription OR engine pod memory > 80% |
| Tenant impact | Initial: subscriber receives stale events; cascade: engine pod OOM-kills affect other tenants |
| Severity | Sev-2 (single subscriber) / Sev-1 if cluster-wide |
| Immediate mitigation | Backpressure signal sent to slow subscriber; if non-compliant, subscription quarantined per slow-subscriber policy; disk-backed Postgres outbox absorbs surge |
| RTO | ≤ 5min quarantine; tenant remediation may take hours |
| Recovery runbook | `runbooks/event-bus-replay.md` §"Backpressure" |
| Postmortem owner | axis-workflow + ops-sre-reliability |

## FM-03: Durable-execution replay storm after engine cold-start

| Field | Value |
|---|---|
| Trigger | Cluster-wide engine restart (e.g., rolling deploy in failure-mode; node-level eviction storm); cold-start tries to resume all in-flight runs at once |
| Detection | Engine worker queue depth spikes > 50k at cold-start; lease wait p99 > 2s |
| Tenant impact | Step dispatch latency degraded; some new runs see start latency > 1s |
| Severity | Sev-2 (operational degradation; no data loss) |
| Immediate mitigation | Resume rate-limit enforced (100 runs/s/worker at cold-start); HPA ramps; new run-starts scheduled-for-distinct-tracked-work for 30s if needed |
| RTO | ≤ 10min steady-state recovery |
| Recovery runbook | `runbooks/durable-execution-restart.md` |
| Postmortem owner | axis-workflow + ops-sre-reliability |

## FM-04: Postgres lock contention — hot tenant's run state row contention starves others

| Field | Value |
|---|---|
| Trigger | One tenant has a workflow that updates the same Postgres row at very high rate (e.g., poorly-modeled iteration); contention propagates via shared connection pool |
| Detection | `pg_locks` waits > 10/sec on specific row; engine step write latency p99 > 100ms for affected tenant |
| Tenant impact | Affected tenant's runs degraded; other tenants experience minor latency increase due to shared pool |
| Severity | Sev-3 (single-tenant degradation) |
| Immediate mitigation | Per-tenant Citus partition isolates; if cross-tenant impact, increase per-tenant pool slot OR escalate to spec-rollback if rogue spec is the cause |
| RTO | ≤ 30min identification + isolation |
| Recovery runbook | `runbooks/spec-rollback.md` if spec-induced; otherwise capacity-model recalibration |
| Postmortem owner | axis-workflow + ops-finops |

## FM-05: Redis lease coordinator outage halts step dispatch

| Field | Value |
|---|---|
| Trigger | Redis Sentinel quorum loss (e.g., AZ outage); lease coordination unavailable |
| Detection | `oya_workflow_engine_redis_sentinel_quorum_healthy == 0` for ≥ 30s OR step claim failures spike |
| Tenant impact | New step dispatch fails; in-flight runs paused at next step boundary |
| Severity | Sev-1 (cluster-wide impact) |
| Immediate mitigation | Failover to Postgres advisory locks (degraded but available); Sentinel re-quorum |
| RTO | ≤ 5min failover; ≤ 30min Sentinel recovery |
| Recovery runbook | `runbooks/redis-failover.md` |
| Postmortem owner | axis-workflow + ops-sre-reliability |

## FM-06: Spec store version downgrade detected (attempted)

| Field | Value |
|---|---|
| Trigger | Attempt to advance `release/workflow-engine/production` ref to a SHA whose spec versions have been deprecated or retired |
| Detection | LEAN lane `oya-governance-workflow-spec-signature-verification` fails; promotion gate refuses |
| Tenant impact | Promotion blocked; no production change |
| Severity | Sev-3 (operational; correctly fail-closed) |
| Immediate mitigation | Verify intent; if accidental: rebuild against current spec versions; if intentional: file ADR + 2-person rule + audit |
| RTO | ≤ 1h investigation |
| Recovery runbook | `runbooks/spec-rollback.md` |
| Postmortem owner | axis-workflow |

## FM-07: Outbox relay worker crash → unflushed events

| Field | Value |
|---|---|
| Trigger | Outbox relay worker OOM, panic, or pod eviction during in-flight flush |
| Detection | `oya_workflow_engine_outbox_lag_seconds > 30` OR worker process absent |
| Tenant impact | Event delivery lag; no event loss (outbox persistence is durable) |
| Severity | Sev-2 (no data loss; operational lag) |
| Immediate mitigation | HA leader-election fails over to standby outbox relay; new leader resumes from last persisted offset |
| RTO | ≤ 5min failover; ≤ 30min full recovery |
| Recovery runbook | `runbooks/event-bus-replay.md` §"Outbox crash recovery" |
| Postmortem owner | axis-workflow + ops-sre-reliability |

## FM-08: ClickHouse replica drift / corruption

| Field | Value |
|---|---|
| Trigger | ClickHouse insert lag; or block-level integrity check fails |
| Detection | `oya_workflow_engine_clickhouse_replication_lag_seconds > 300` OR block-validator emits SHA mismatch |
| Tenant impact | Replay-debugger analytics degraded; current run state unaffected (Postgres is authoritative) |
| Severity | Sev-3 (analytics-only impact) |
| Immediate mitigation | Halt ClickHouse writes from affected node; restore from Postgres → ClickHouse replay pipeline |
| RTO | ≤ 4h full restore from Postgres |
| Recovery runbook | `runbooks/clickhouse-replay-restore.md` (Slice B7 extension) |
| Postmortem owner | axis-workflow + ops-sre-reliability |

## FM-09: Cross-tenant subscription leak detected

| Field | Value |
|---|---|
| Trigger | LEAN check or runtime audit detects tenant-A subscriber receiving tenant-B event |
| Detection | `oya_workflow_engine_cross_tenant_delivery_total > 0` OR continuous-compliance lane alarm |
| Tenant impact | Confidentiality breach (DPIA R-02; threat T-I-03) |
| Severity | Sev-1 (security breach) |
| Immediate mitigation | Engage ops-security; freeze affected subscription endpoint; revoke implicated SDK keys; begin forensic trace |
| RTO | ≤ 5min freeze; investigation + breach notification 72h+ |
| Recovery runbook | `runbooks/security-incident.md` (cross-ref `incident-response.md` §"Sev-1") |
| Postmortem owner | ops-security |

## FM-10: PII leakage via step payload logging detected

| Field | Value |
|---|---|
| Trigger | Synthetic-PII detector (CI lane) flags a workflow spec emitting unredacted PII in step payload logs |
| Detection | `oya_workflow_engine_pii_redactor_miss_total > 0` |
| Tenant impact | DPIA R-01 risk realised; GDPR / KR PIPA / HIPAA violation possible |
| Severity | Sev-2 (data-protection breach) |
| Immediate mitigation | Engage spec author; patch SDK redactor; purge affected payload logs; enable redactor-aggressive mode |
| RTO | ≤ 1h patch deploy; ≤ 24h purge |
| Recovery runbook | `runbooks/security-incident.md` §"PII redaction failure" + DSR cascade |
| Postmortem owner | ops-security + spec author's team |

## FM-11: Stuck workflow run (unrecoverable state)

| Field | Value |
|---|---|
| Trigger | Spec-induced infinite-wait state; OR transient downstream failure that no retry policy can recover; OR a SLA-timer arming race that doesn't fire |
| Detection | `oya_workflow_engine_run_stuck_seconds > 3600` (1h heuristic) |
| Tenant impact | Run will never complete; tenant must intervene |
| Severity | Sev-3 (single run) |
| Immediate mitigation | Notify tenant; operator-initiated cancel with 2-person rule + audit; spec author files ADR for root cause |
| RTO | ≤ 30min cancel; tenant remediation per spec change |
| Recovery runbook | `runbooks/stuck-workflow-recovery.md` |
| Postmortem owner | axis-workflow + spec author |

## FM-12: Workflow event poisoning — malformed event causes consumer crash

| Field | Value |
|---|---|
| Trigger | Subscriber crashes on receiving a specific malformed event (e.g., new field added without subscriber update) |
| Detection | Subscriber consume error rate spikes; subscription consumer-lag climbs |
| Tenant impact | Subscriber's downstream automation halted |
| Severity | Sev-2 (per-subscriber) |
| Immediate mitigation | Engine event-bus quarantines the poison event after N consume failures; subscriber notified; tenant operator can replay or skip the poison event |
| RTO | ≤ 30min quarantine; tenant remediation per subscriber update |
| Recovery runbook | `runbooks/event-bus-replay.md` §"Poison event" |
| Postmortem owner | axis-workflow + subscriber owner |

## FM-13: Audit chain seal gap detected

| Field | Value |
|---|---|
| Trigger | Audit-chain verifier detects missing seal in a per-run sequence (out-of-order or absent) |
| Detection | `oya_workflow_engine_audit_chain_seal_gap_total > 0` |
| Tenant impact | Audit integrity question; downstream compliance posture may be impacted |
| Severity | Sev-1 (compliance-impact) |
| Immediate mitigation | Engage ops-security + audit-chain µservice; quarantine affected per-run sequence; investigate root cause |
| RTO | ≤ 1h quarantine; investigation may take days |
| Recovery runbook | `runbooks/security-incident.md` §"Audit-chain integrity" |
| Postmortem owner | ops-security + audit-chain |

## RTO / RPO Summary

| Failure | RTO | RPO |
|---|---|---|
| State-machine deadlock | 5min cycle-break | N/A |
| Event-bus backpressure | 5min quarantine | 0 (outbox durable) |
| Durable-execution replay storm | 10min | 0 |
| Postgres lock contention | 30min identification | 0 |
| Redis lease coordinator outage | 5min failover | 0 (state in Postgres) |
| Spec version downgrade attempt | 1h investigation | N/A |
| Outbox relay worker crash | 5min HA failover | 0 (durable outbox) |
| ClickHouse drift / corruption | 4h restore from Postgres | varies (analytics-only) |
| Cross-tenant subscription leak | 5min freeze | N/A (breach occurred) |
| PII leakage | 1h patch + 24h purge | N/A |
| Stuck workflow run | 30min cancel | N/A |
| Event poisoning | 30min quarantine | 0 |
| Audit chain seal gap | 1h quarantine | N/A |

## SLO on Failure-Detection Pipeline

Meta-SLO: workflow-engine's own failures must be detected within window.

| SLI | Target | Burn-rate alert |
|---|---|---|
| Alert-to-page latency (p99) | ≤ 60s | 14.4× burn over 1h |
| Detection-coverage (synthetic faults caught) | ≥ 99.5% | 6× burn over 6h |
| Two-channel corroboration completion | ≥ 99% within 90s | ticket burn 3d |
| False-positive page rate | ≤ 1 / week / on-call | informational |

## References

- `microservices/workflow-engine/threat-model.md` (each FM has at least one corresponding STRIDE / LINDDUN threat).
- `microservices/workflow-engine/dpia.md` (FM-09, FM-10 map to R-02, R-01 respectively).
- `microservices/workflow-engine/incident-response.md` §"Severity Definitions".
- `microservices/workflow-engine/runbooks/*` (recovery procedures).
- `microservices/workflow-engine/capacity-model.md`.
- Google SRE Workbook ch. 12 (Postmortem culture).
