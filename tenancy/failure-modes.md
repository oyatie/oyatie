---
doc_class: FailureModeCatalog
title: Failure-Mode Catalog
microservice: tenancy
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-tenancy
deciders: ops-sre-reliability, axis-tenancy, ops-security, council-architecture
related_adrs: [ADR-0018, ADR-0028, ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/tenancy/threat-model.md
  - microservices/tenancy/dpia.md
  - microservices/tenancy/policy/rls-isolation.md
  - microservices/tenancy/incident-response.md
  - microservices/tenancy/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident affecting tenancy
doc_status: published
---

# Failure-Mode Catalog (tenancy µservice)

## Purpose

Enumerate the failure scenarios on-call must handle, the detection signal for each, immediate mitigation, root-cause-analysis path, recovery time objective (RTO), and the runbook that owns the recovery procedure.

## Failure-Mode Index

## FM-01: Postgres primary outage (Patroni failover)

| Field | Value |
|---|---|
| Trigger | OCI compute failure, OOM, kernel panic, or scheduled maintenance on the Postgres primary pod |
| Detection | Patroni REST `/cluster` reports `state: failover_in_progress`; `oya_tenancy_postgres_primary_alive == 0` for ≥ 5s |
| Tenant impact | Brief validate-path latency spike during failover (≤ 10s); no data loss (sync replicas absorb writes) |
| Severity | Sev-2 (degraded; auto-failover handles); Sev-1 if persists > 5min |
| Immediate mitigation | Patroni elects new primary from sync replicas; tenancy DB connection-pool re-routes; Valkey absorbs read load during ≤ 10s window |
| RTO | ≤ 10s for failover; ≤ 5min for full cluster re-stabilisation |
| Recovery runbook | `runbooks/citus-rebalance.md` §"Postgres primary failover" |
| Postmortem owner | ops-sre-reliability + axis-tenancy |

## FM-02: RLS policy drift (live state diverges from declared YAML)

| Field | Value |
|---|---|
| Trigger | Adversarial / accidental live DB mutation (DBA JIT misuse); CI lane evasion |
| Detection | `oya-tenancy-rls-state-validator` 5min cadence; `oya_tenancy_rls_drift_total > 0` |
| Tenant impact | Potential cross-tenant data exposure (catastrophic); time-bounded to ≤ 5min by validator cadence |
| Severity | Sev-1 (security breach risk) |
| Immediate mitigation | Auto-rollback via ArgoCD to last-green YAML state; engage ops-security; freeze affected schema; begin forensic trace |
| RTO | ≤ 5min auto-rollback; investigation days |
| Recovery runbook | `runbooks/rls-drift-recovery.md` |
| Postmortem owner | ops-security + axis-tenancy |

## FM-03: Citus coordinator outage

| Field | Value |
|---|---|
| Trigger | OCI failure on coordinator pod; Citus version-upgrade hung |
| Detection | `oya_tenancy_citus_coordinator_alive == 0` for ≥ 30s; tenant-lifecycle write path latency spike |
| Tenant impact | New tenant creations blocked; existing reads via worker nodes proceed; rebalance halted |
| Severity | Sev-2 (write degraded; reads fine) |
| Immediate mitigation | Patroni-managed Citus coordinator failover; new coordinator elected from sync replica |
| RTO | ≤ 30s failover |
| Recovery runbook | `runbooks/citus-rebalance.md` §"Coordinator outage" |
| Postmortem owner | ops-sre-reliability + axis-tenancy |

## FM-04: JWT signing key compromise suspected

| Field | Value |
|---|---|
| Trigger | OpenBao alarm on signing-key access pattern anomaly; secret-scanner finds key fragment in commit; pen-test reveals exposure |
| Detection | OpenBao audit log anomaly + manual escalation |
| Tenant impact | Potential cross-tenant identity forgery (T-S-01); critical |
| Severity | Sev-1 (security breach) |
| Immediate mitigation | Emergency rotation: OpenBao generates new Ed25519 keypair; fingerprint advertised via Workflow; old key revoked immediately (no 30d grace; in-flight tokens require re-auth) |
| RTO | ≤ 15min emergency rotation; ≤ 1h for verifier cache propagation across every µservice |
| Recovery runbook | `runbooks/jwt-key-rotation.md` §"Emergency rotation" |
| Postmortem owner | ops-security + axis-tenancy |

## FM-05: Tenant activation stuck (schema migration hung)

| Field | Value |
|---|---|
| Trigger | sqlx migration deadlock; Citus shard placement contention; Patroni replication lag spike |
| Detection | `oya_tenancy_activation_duration_seconds{quantile="0.99"} > 600` (10min) sustained ≥ 5min |
| Tenant impact | Specific tenant onboarding delayed; Sev-3 if isolated, Sev-2 if multi-tenant |
| Severity | Sev-3 (single tenant; recoverable) |
| Immediate mitigation | Identify deadlock via `pg_locks`; manual `pg_cancel_backend` or `pg_terminate_backend`; restart activation worker for the stuck tenant; alert tenant operator |
| RTO | ≤ 15min for manual unblock |
| Recovery runbook | `runbooks/tenant-onboarding.md` §"Stuck activation" |
| Postmortem owner | axis-tenancy |

## FM-06: Tenant deletion DSR cascade incomplete (missing receipt)

| Field | Value |
|---|---|
| Trigger | A µservice's DSR handler crashes / times out / is missing |
| Detection | `oya-tenancy-dsr-cascade-worker` SLA timer at 80% of per-pack legal SLA; missing-receipt alert |
| Tenant impact | DSR potentially not honoured within statutory window (regulatory breach risk) |
| Severity | Sev-2 (regulatory; escalates to Sev-1 if SLA breach imminent) |
| Immediate mitigation | Engage workload µservice owner; if handler bug → emergency hotfix; if handler missing → escalate to DPO for manual-override path (alternative-measure documentation); LEAN check `oya-governance-dsr-handler-conformance` ensures rare case |
| RTO | varies (depends on root cause); legal SLA is the hard ceiling |
| Recovery runbook | `runbooks/tenant-deletion-dsr-cascade.md` |
| Postmortem owner | council-privacy + axis-tenancy + µservice owner |

## FM-07: Valkey cache outage

| Field | Value |
|---|---|
| Trigger | Valkey pod OOM; cluster split-brain; OCI failure |
| Detection | Valkey health probe failure; `oya_tenancy_valkey_alive == 0` |
| Tenant impact | Validate-path latency spike (Valkey miss falls through to Postgres); p99 could reach 20ms vs 5ms target |
| Severity | Sev-3 (degraded; not blocking; fallback to Postgres works) |
| Immediate mitigation | Restart Valkey pods (HPA scales); verify Postgres absorbs load; tighten per-tenant rate limits if needed |
| RTO | ≤ 5min |
| Recovery runbook | `runbooks/tenant-onboarding.md` §"Valkey recovery" |
| Postmortem owner | ops-sre-reliability |

## FM-08: Citus shard rebalance hung / failed

| Field | Value |
|---|---|
| Trigger | Logical replication lag; coordinator restart mid-rebalance |
| Detection | `oya_tenancy_rebalance_duration_seconds{quantile="0.99"} > 3600` sustained; tenant write latency spike |
| Tenant impact | Specific tenant(s) experience write delays during their shard's stuck rebalance window |
| Severity | Sev-2 |
| Immediate mitigation | Abort rebalance via `citus_rebalance_stop()`; verify pre-rebalance + post-rebalance row counts match (no data loss); resume from a clean state in next cycle |
| RTO | ≤ 30min for abort + cleanup |
| Recovery runbook | `runbooks/citus-rebalance.md` §"Stuck rebalance" |
| Postmortem owner | ops-sre-reliability + axis-tenancy |

## FM-09: Pack-routing misroute (tenant data flows to wrong pack)

| Field | Value |
|---|---|
| Trigger | Tenancy adapter bug routes pack-eu tenant to pack-us cluster |
| Detection | Integration test caught at CI; runtime detector `oya_tenancy_pack_misroute_total > 0` |
| Tenant impact | Cross-border transfer violation (DPIA R-04); GDPR / KR PIPA breach |
| Severity | Sev-1 (regulatory breach) |
| Immediate mitigation | Quarantine misrouted data; engage ops-security + council-privacy; correct adapter config; begin breach-notification chain |
| RTO | ≤ 1h routing correction; ≤ 72h breach notification (GDPR Art. 33) |
| Recovery runbook | `runbooks/rls-drift-recovery.md` §"Pack misroute" (cross-pollination) + ops-security incident playbook |
| Postmortem owner | council-privacy + ops-security |

## FM-10: Tenant suspension propagation lag (some µservice doesn't honour suspension)

| Field | Value |
|---|---|
| Trigger | Workflow event consumer in some µservice is down / lagging |
| Detection | Per-µservice TenantSuspended event lag exceeds 60s; `oya_tenancy_event_propagation_lag_seconds{microservice=<>} > 60` |
| Tenant impact | Suspended tenant continues to receive requests at the lagging µservice; brief window of allowed access |
| Severity | Sev-3 (operational delay; suspension still applied via RLS / JWT) |
| Immediate mitigation | Engage workload µservice owner; verify event consumer health; manually trigger event re-emission |
| RTO | ≤ 30min |
| Recovery runbook | `runbooks/tenant-suspension.md` §"Propagation lag" |
| Postmortem owner | axis-tenancy + µservice owner |

## FM-11: OpenBao tenant-resolver outage

| Field | Value |
|---|---|
| Trigger | OpenBao pod failure; OpenBao DB outage |
| Detection | OpenBao health probe; `oya_tenancy_openbao_alive == 0` |
| Tenant impact | Tenant creation blocked (cannot assign canonical → hashed tenant_id); existing tenants unaffected (JWT verification uses cached pubkeys) |
| Severity | Sev-2 |
| Immediate mitigation | Engage `cloud-secrets` µservice on-call; verify OpenBao HA state; new tenant onboarding paused until restored |
| RTO | ≤ 30min |
| Recovery runbook | upstream `cloud-secrets/runbooks/openbao-outage.md` |
| Postmortem owner | cloud-secrets + axis-tenancy |

## FM-12: Patroni DCS (etcd) outage causing split-brain risk

| Field | Value |
|---|---|
| Trigger | DCS quorum loss (e.g., 2 of 3 etcd pods down) |
| Detection | `etcd_server_has_leader == 0`; Patroni emits split-brain warning |
| Tenant impact | Patroni refuses to elect new primary; if current primary fails, write path stops until quorum restored |
| Severity | Sev-1 (HA broken) |
| Immediate mitigation | Restore etcd quorum (scale up etcd replicas); engage cloud-k8s on-call; if extended, manual Patroni intervention |
| RTO | ≤ 30min |
| Recovery runbook | `runbooks/citus-rebalance.md` §"DCS recovery" |
| Postmortem owner | ops-sre-reliability + cloud-k8s |

## FM-13: Secret leak (JWT signing key / Postgres password) detected in logs

| Field | Value |
|---|---|
| Trigger | Secret-scanner CI lane detection; GitHub secret-scanning push protection |
| Detection | `oya-governance-evidence-secret-scan` lane alarm |
| Tenant impact | Time-window between leak and rotation = exposure risk (mitigated by rotation speed) |
| Severity | Sev-1 (security breach) |
| Immediate mitigation | OpenBao rotates secret < 60s of detection; revoke old credential; forensic trace of leak path; tenant notification per breach-notification SLA if applicable |
| RTO | ≤ 1min rotation; investigation hours-days |
| Recovery runbook | `runbooks/jwt-key-rotation.md` §"Emergency rotation" |
| Postmortem owner | ops-security + cloud-secrets |

## FM-14: Audit-chain seal latency spike

| Field | Value |
|---|---|
| Trigger | Audit-chain µservice degraded; back-pressure on seal queue |
| Detection | `oya_tenancy_audit_seal_latency_seconds{quantile="0.99"} > 5` sustained |
| Tenant impact | Lifecycle events delayed sealing; not blocked (eventual consistency); regulatory artifact freshness degraded briefly |
| Severity | Sev-3 |
| Immediate mitigation | Engage audit-chain µservice on-call; monitor seal queue; if persistent, batched seal mode (queue → batch every 1min instead of per-event) |
| RTO | ≤ 30min |
| Recovery runbook | upstream `audit-chain/runbooks/seal-latency.md` |
| Postmortem owner | audit-chain + axis-tenancy |

## FM-15: Validate-path overload (single tenant burst)

| Field | Value |
|---|---|
| Trigger | Tenant runaway traffic; misconfigured tenant validating with no caching layer |
| Detection | Per-tenant `oya_tenancy_validate_rps{tenant_id=<>} > threshold` |
| Tenant impact | Tenant's own request latency spikes; potential noisy-neighbor effect |
| Severity | Sev-3 |
| Immediate mitigation | Enforce per-tenant rate limit (WAF + ingress); engage tenant on cause; if production-tier and within global budget, increase tenant limit; otherwise tenant remediates |
| RTO | ≤ 15min limit enforcement |
| Recovery runbook | `runbooks/tenant-onboarding.md` §"Rate-limit overage" |
| Postmortem owner | ops-sre-reliability |

## RTO / RPO Summary

| Failure | RTO | RPO |
|---|---|---|
| Postgres primary outage | 10s | 0 (sync replicas) |
| RLS policy drift | 5min auto-rollback | 0 |
| Citus coordinator outage | 30s | 0 |
| JWT-key compromise | 15min emergency rotation | N/A |
| Tenant activation stuck | 15min | N/A |
| DSR cascade incomplete | varies (legal SLA bound) | N/A |
| Valkey outage | 5min | 0 (Postgres fallback) |
| Citus rebalance hung | 30min | 0 (transactional cut-over) |
| Pack misroute | 1h + 72h breach notify | N/A |
| Suspension propagation lag | 30min | N/A |
| OpenBao outage | 30min (onboarding only) | N/A |
| DCS outage | 30min | 0 |
| Secret leak | 1min rotation | N/A |
| Audit-chain seal lag | 30min | eventual |
| Validate overload | 15min | N/A |

## SLO on Failure-Detection Pipeline

| SLI | Target | Burn-rate alert |
|---|---|---|
| Alert-to-page latency (p99) | ≤ 60s | 14.4× burn over 1h |
| RLS drift detection (probe-to-page) | ≤ 5min | 6× burn over 6h |
| Validate-path availability (mtbf) | 99.99% monthly | 2× burn over 1h (highest sensitivity) |
| Audit-chain seal completion | ≥ 99% within 1s | 14.4× over 1h |

## References

- `microservices/tenancy/threat-model.md` (each FM has at least one corresponding STRIDE / LINDDUN threat).
- `microservices/tenancy/dpia.md` (FM-02, FM-04, FM-06, FM-09 map to R-01, R-02, R-05, R-04 respectively).
- `microservices/tenancy/incident-response.md` §"Severity Definitions".
- `microservices/tenancy/runbooks/*` (recovery procedures).
- `microservices/tenancy/capacity-model.md` (FM-15 + per-tenant limits).
- Patroni operations docs — `patroni.readthedocs.io`.
- Citus operations docs — `docs.citusdata.com`.
- Google SRE Workbook ch. 12 (Postmortem culture).
