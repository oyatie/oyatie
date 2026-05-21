---
doc_class: FailureModeCatalog
title: Failure-Mode Catalog
microservice: ontology
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-ontology
deciders: ops-sre-reliability, axis-ontology, ops-security, council-architecture
related_adrs: [ADR-0028, ADR-0059, ADR-0106, ADR-0117, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145)]
related_artifacts:
  - microservices/ontology/threat-model.md
  - microservices/ontology/dpia.md
  - microservices/ontology/policy/type-isolation.md
  - microservices/ontology/incident-response.md
  - microservices/ontology/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident affecting ontology
doc_status: published
---

# Failure-Mode Catalog (ontology µservice)

## Purpose

Enumerate failure scenarios on-call must handle, detection signal for each, immediate mitigation, root-cause-analysis path, recovery time objective (RTO), and recovery runbook. Cross-referenced from `incident-response.md` for severity classification.

## Failure-Mode Index

Each failure carries:
- **FM-ID**: stable identifier
- **Trigger**: precipitating event(s)
- **Detection**: SLI / alert / metric that fires
- **Tenant impact**: what tenants experience
- **Severity**: Sev-1/2/3/4 (definitions in `incident-response.md`)
- **Immediate mitigation**: actions on-call performs in the first 5 minutes
- **RTO**: target recovery time
- **Recovery runbook**: where the procedure lives
- **Postmortem owner**: who owns the after-action review

## FM-01: Postgres + Citus coordinator outage

| Field | Value |
|---|---|
| Trigger | Coordinator pod crash, hardware failure, kernel panic, OOM kill |
| Detection | `pg_up{instance=~"citus-coordinator.*"} == 0` for ≥ 2 min OR `postgres_request_duration_seconds{quantile="0.99"} > 1s` for ≥ 5 min |
| Tenant impact | All Object Type writes + Function reads queued; Cedar evaluator still works (in-process); brief outage of canonical state |
| Severity | Sev-1 (critical substrate; all µservices depend) |
| Immediate mitigation | Failover to streaming replica (read traffic continues); promote replica to coordinator if primary unrecoverable |
| RTO | ≤ 5 min coordinator-pod restart; ≤ 15 min replica promotion |
| Recovery runbook | `runbooks/postgres-citus-coordinator-restart.md` |
| Postmortem owner | axis-ontology + ops-sre-reliability |

## FM-02: Citus worker shard failure

| Field | Value |
|---|---|
| Trigger | Worker node crash; PV failure on shard |
| Detection | `citus_worker_shard_health{shard_id="<id>"} != "active"` for ≥ 2 min |
| Tenant impact | Tenants on affected shard cannot read/write their Object Types until shard recovered |
| Severity | Sev-2 (partial; subset of tenants affected) |
| Immediate mitigation | Promote shard replica (RF=3); re-balance shards if multiple workers degraded |
| RTO | ≤ 10 min shard replica promotion |
| Recovery runbook | `runbooks/citus-shard-failover.md` |
| Postmortem owner | axis-ontology + cloud-infra |

## FM-03: Postgres RLS drift (multitenancy invariant broken)

| Field | Value |
|---|---|
| Trigger | Helm config change merged without lane gate (impossible if BLOCKER) OR live-cluster mutation by superuser |
| Detection | `oya-governance-ontology-tenancy-isolation` lane fails OR continuous schema-drift detector alarms |
| Tenant impact | Potential cross-tenant data exposure if not caught pre-deploy |
| Severity | Sev-1 (security breach risk) |
| Immediate mitigation | Auto-rollback via ArgoCD to last green Helm state; isolate cluster; declare incident; engage ops-security |
| RTO | ≤ 5 min auto-rollback; investigation may take days |
| Recovery runbook | `runbooks/postgres-rls-drift.md` + ops-security incident playbook |
| Postmortem owner | ops-security + axis-ontology |

## FM-04: Schema-registry corruption (Object Type schema invalid or missing)

| Field | Value |
|---|---|
| Trigger | Failed Object Type schema migration; partial write to schema registry; Valkey cache poisoning |
| Detection | `oya_ontology_schema_registry_validation_failures_total > 0` OR Function evaluator fails on schema lookup |
| Tenant impact | Affected Object Type's writes refused; Function reads return errors |
| Severity | Sev-2 (specific Object Type affected) |
| Immediate mitigation | Roll back schema registry transaction; reload from git-versioned schemas; flush Valkey cache |
| RTO | ≤ 15 min reload + cache flush |
| Recovery runbook | `runbooks/type-registry-migration.md` |
| Postmortem owner | axis-ontology |

## FM-05: Query engine OOM (runaway Function projection)

| Field | Value |
|---|---|
| Trigger | Tenant submits a Function with unbounded scan; query plan EXPLAIN evaluation passes but runtime explodes |
| Detection | `container_memory_usage_bytes{pod=~"function-engine.*"} > 80% limit` OR pod restart count climbing |
| Tenant impact | Function reads slow / timeout for affected tenant; possibly cascade if HPA can't keep up |
| Severity | Sev-2 (tenant-specific; engine-wide if cascade) |
| Immediate mitigation | Kill the offending query (PostgreSQL `pg_terminate_backend`); rate-limit the tenant; scale up engine replicas |
| RTO | ≤ 5 min kill + scale; permanent fix in EXPLAIN pre-check tightening |
| Recovery runbook | `runbooks/query-engine-restart.md` |
| Postmortem owner | axis-ontology |

## FM-06: Cedar policy infinite-loop / runaway evaluation

| Field | Value |
|---|---|
| Trigger | Malformed Cedar fragment causing engine pathological eval; deployed via PR that bypassed lane (impossible if BLOCKER) |
| Detection | `cedar_evaluation_duration_seconds{quantile="0.99"} > 100ms` for ≥ 1 min OR Cedar timeout error rate climbs |
| Tenant impact | All Action invocations slow / timeout |
| Severity | Sev-2 (critical-path; substrate Action invocations affected) |
| Immediate mitigation | Roll back Cedar fragment; engine hard timeout 10 ms is the fail-safe |
| RTO | ≤ 15 min Cedar fragment rollback via git revert + ArgoCD apply |
| Recovery runbook | `runbooks/cedar-fragment-rollback.md` |
| Postmortem owner | axis-ontology + ops-security |

## FM-07: Cross-tenant join leak (RLS bypass via Function projection)

| Field | Value |
|---|---|
| Trigger | LEAN check or runtime audit detects cross-tenant query result; RLS bypassed (catastrophic) |
| Detection | `oya_ontology_tenant_unauthorized_read_attempt_total > 0` over 1 min OR continuous-compliance lane alarm |
| Tenant impact | Confidentiality breach (DPIA R-01) |
| Severity | Sev-1 (security breach) |
| Immediate mitigation | Engage ops-security; freeze affected Function path; revoke implicated API keys; begin forensic trace |
| RTO | ≤ 5 min endpoint freeze; investigation + breach-notification chain may take 72h+ |
| Recovery runbook | `runbooks/cross-tenant-leak-recovery.md` |
| Postmortem owner | ops-security |

## FM-08: PII leakage via Function result projection (tier escape)

| Field | Value |
|---|---|
| Trigger | Function projects Tier1Sensitive properties without tier-filter; deployed via PR that bypassed lane |
| Detection | LEAN `oya-governance-ontology-tier-enforcement` lane runtime probe; synthetic-PII detector flags response |
| Tenant impact | DPIA R-03 (tier escape); GDPR / PIPA / HIPAA violation possible |
| Severity | Sev-2 (data-protection breach; not full cross-tenant unless also cross-tenant) |
| Immediate mitigation | Patch Function projection; purge cached results; engage workload µservice owner |
| RTO | ≤ 1h patch deploy; ≤ 24h purge historical affected projections |
| Recovery runbook | `runbooks/tier-escape-recovery.md` |
| Postmortem owner | ops-security + workload µservice owner |

## FM-09: Audit-chain emission gap (outbox lag → seal incomplete)

| Field | Value |
|---|---|
| Trigger | Audit-chain worker outage; Kafka outbox-consumer lag spike |
| Detection | `oya:ontology_audit_chain_completeness:rate < 1.0` over 1 min OR Kafka consumer lag > 60 s |
| Tenant impact | Action invocations succeed but audit seal pending (eventually completes); DSR cascade may report incomplete trail |
| Severity | Sev-2 (audit gap; not data loss; affects provenance claims) |
| Immediate mitigation | Restart audit-chain worker; replay missed offsets; scale up worker replicas |
| RTO | ≤ 10 min worker restart; ≤ 30 min full lag drain |
| Recovery runbook | `runbooks/audit-chain-replay.md` |
| Postmortem owner | axis-ontology + audit-chain µservice |

## FM-10: ClickHouse history-mirror lag (OLAP queries return stale)

| Field | Value |
|---|---|
| Trigger | ClickHouse compactor saturation; Kafka mirror-consumer behind |
| Detection | `clickhouse_mirror_lag_seconds > 60` |
| Tenant impact | OLAP Function reads return up-to-stale data; OLTP Function reads unaffected |
| Severity | Sev-3 (degraded analytics; not service down) |
| Immediate mitigation | Scale up ClickHouse mirror-ingester; throttle OLAP reads if lag > 5 min |
| RTO | ≤ 30 min lag drain |
| Recovery runbook | `runbooks/clickhouse-rebalance.md` |
| Postmortem owner | axis-ontology |

## FM-11: Object Type deprecation broke a tenant's deployed code

| Field | Value |
|---|---|
| Trigger | Object Type schema deprecation merged; downstream tenant deployment still uses deprecated property |
| Detection | `oya_ontology_deprecated_property_read_total > 0` after schema wave completion |
| Tenant impact | Affected tenant's reads return error |
| Severity | Sev-2 (tenant-specific) |
| Immediate mitigation | Restore deprecated property; engage tenant; extend deprecation timeline |
| RTO | ≤ 1h restore via git revert + ArgoCD apply |
| Recovery runbook | `runbooks/object-type-deprecation.md` |
| Postmortem owner | axis-ontology + tenant owner |

## FM-12: Agent gateway runaway LLM loop (DoS via tool-call cascade)

| Field | Value |
|---|---|
| Trigger | LLM agent caught in tool-call loop; per-session rate limit too high |
| Detection | `agent_gateway_concurrent_calls{session_id="<id>"} > 100/min` |
| Tenant impact | Agent gateway throughput degraded; cost overrun for tenant |
| Severity | Sev-3 |
| Immediate mitigation | Trip circuit breaker for offending session; engage tenant on LLM agent design |
| RTO | ≤ 5 min circuit breaker activation |
| Recovery runbook | `runbooks/agent-gateway-circuit-breaker.md` |
| Postmortem owner | axis-ontology + tenant owner |

## FM-13: Audit-chain Merkle tampering detected

| Field | Value |
|---|---|
| Trigger | Verification check detects Merkle root mismatch; or block-SHA validator alarms |
| Detection | `oya_ontology_audit_chain_merkle_tamper_total > 0` OR scheduled verification job exit non-zero |
| Tenant impact | Provenance claim invalidated for affected tenant + period |
| Severity | Sev-1 (audit trust gone) |
| Immediate mitigation | Quarantine affected period; engage ops-security; begin forensic trace; trigger DPA/PIPC/OCR notification chain |
| RTO | ≤ 30 min quarantine; investigation may take days |
| Recovery runbook | `runbooks/cross-tenant-leak-recovery.md` §"Audit tampering" |
| Postmortem owner | ops-security + audit-chain µservice |

## FM-14: Cross-pillar grant misuse (org-pillar data accessed via person-pillar context)

| Field | Value |
|---|---|
| Trigger | Cedar `pillar.cedar` evaluator returns deny; OR malformed grant passed verification |
| Detection | `oya_ontology_cross_pillar_unauthorized_total > 0` over 5 min |
| Tenant impact | Affected reads refused; if grant was forged, pillar boundary holds (defence-in-depth) |
| Severity | Sev-2 (security incident; not breach unless grant forged AND Cedar bypassed) |
| Immediate mitigation | Revoke grant; investigate; tenant + ops-security engaged |
| RTO | ≤ 5 min revoke |
| Recovery runbook | `runbooks/cross-tenant-leak-recovery.md` §"Cross-pillar leak" |
| Postmortem owner | ops-security + council-privacy |

## FM-15: DSR cascade timed out (subject not fully tombstoned within SLA)

| Field | Value |
|---|---|
| Trigger | DSR runner takes > 30d to complete erasure across Object Types |
| Detection | DSR queue dashboard SLO breach |
| Tenant impact | Regulatory non-compliance with GDPR Art. 17 / PIPA Art. 36 |
| Severity | Sev-2 (compliance gap) |
| Immediate mitigation | Manual scan + tombstone; escalate to council-privacy for tenant notification |
| RTO | ≤ 24h manual completion + tenant notification |
| Recovery runbook | `runbooks/dsr-cascade-recovery.md` |
| Postmortem owner | council-privacy + axis-ontology |

## FM-16: Pack misroute (tenant data written to wrong pack)

| Field | Value |
|---|---|
| Trigger | Workload µservice SDK config bug routes pack-eu tenant write to pack-us cluster |
| Detection | Integration test at CI; OR runtime detector emits `oya_ontology_pack_misroute_total > 0` |
| Tenant impact | Cross-border-transfer violation (DPIA R-11); GDPR / KR PIPA breach |
| Severity | Sev-1 (regulatory breach) |
| Immediate mitigation | Quarantine misrouted data; engage ops-security + council-privacy; correct SDK config; begin breach-notification chain |
| RTO | ≤ 1h routing correction; ≤ 72h breach notification (GDPR Art. 33) |
| Recovery runbook | `runbooks/cross-tenant-leak-recovery.md` §"Pack misroute" |
| Postmortem owner | council-privacy + ops-security |

## RTO / RPO Summary

| Failure | RTO | RPO |
|---|---|---|
| Postgres coordinator outage | 5–15 min | 0 (RF=3 + streaming replica) |
| Citus shard failure | 10 min | 0 (RF=3) |
| RLS drift | 5 min auto-rollback | 0 |
| Schema registry corruption | 15 min | 0 (git-versioned) |
| Function engine OOM | 5 min | N/A |
| Cedar runaway | 15 min | 0 |
| Cross-tenant leak | 5 min freeze | N/A (breach occurred) |
| Tier escape | 1h patch + 24h purge | N/A |
| Audit-chain gap | 10 min restart + 30 min drain | varies |
| ClickHouse lag | 30 min drain | varies |
| Deprecation broke tenant | 1h restore | 0 |
| Agent loop DoS | 5 min circuit break | N/A |
| Audit chain tampering | 30 min quarantine | N/A (Merkle invalidated) |
| Cross-pillar misuse | 5 min revoke | N/A |
| DSR cascade timed out | 24h manual | N/A |
| Pack misroute | 1h + 72h breach-notif | N/A |

## SLO on Failure-Detection Pipeline

Meta-SLO: the ontology substrate itself has a self-observability SLO that no failure remains undetected longer than its detection-window target.

| SLI | Target | Burn-rate alert |
|---|---|---|
| Alert-to-page latency (p99) | ≤ 60 s | 14.4× burn over 1h |
| Detection-coverage (proportion of injected synthetic faults caught) | ≥ 99.5 % | 6× burn over 6h |
| Two-channel corroboration completion | ≥ 99 % within 90 s | ticket burn 3d |
| False-positive page rate | ≤ 1 / week / on-call | informational |

## References

- `microservices/ontology/threat-model.md` (each FM has at least one corresponding STRIDE / LINDDUN threat ID).
- `microservices/ontology/dpia.md` (FM-07, FM-08, FM-14, FM-15, FM-16 map to R-01, R-03, R-02, R-08, R-11 respectively).
- `microservices/ontology/incident-response.md` §"Severity Definitions".
- `microservices/ontology/runbooks/*` (recovery procedures).
- `microservices/ontology/capacity-model.md` (FM-05 + per-tenant limits).
- Postgres + Citus reliability docs — `docs.citusdata.com`.
- ClickHouse operations — `clickhouse.com/docs/en/operations/`.
- Cedar v4 — `cedarpolicy.com`.
- Google SRE Workbook ch. 12 (Postmortem culture).
