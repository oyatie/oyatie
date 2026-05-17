---
doc_class: FailureModeCatalog
title: Failure-Mode Catalog
microservice: observability
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-observability
deciders: ops-sre-reliability, axis-observability, ops-security, council-architecture
related_adrs: [ADR-0117, ADR-0130, ADR-0131]
related_artifacts:
  - microservices/observability/threat-model.md
  - microservices/observability/dpia.md
  - microservices/observability/policy/tenant-isolation.md
  - microservices/observability/incident-response.md
  - microservices/observability/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident affecting observability
doc_status: published
---

# Failure-Mode Catalog (observability µservice)

## Purpose

Enumerate the failure scenarios on-call must handle, the detection signal for each, immediate mitigation, root-cause-analysis path, recovery time objective (RTO), and the runbook that owns the recovery procedure. Cross-referenced from `incident-response.md` for severity classification.

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

## FM-01: Mimir distributor outage (single AZ)

| Field | Value |
|---|---|
| Trigger | Cluster autoscaler eviction, hardware failure, kernel panic, OOM kill of all distributor pods in one AZ |
| Detection | `mimir_distributor_request_duration_seconds{quantile="0.99"} > 1s` for ≥ 5min OR replica-count drops below replication-factor for ≥ 3min |
| Tenant impact | Ingest latency spike; samples queued by Alloy collector; no data loss within Alloy buffer window (default 5min) |
| Severity | Sev-2 (degraded; no data loss yet) |
| Immediate mitigation | Verify HPA scaling triggered; cordon affected AZ; allow cross-AZ rebalance; ensure surviving distributors absorb load |
| RTO | ≤ 15 min for distributor pod recovery; ≤ 30 min for full AZ restoration |
| Recovery runbook | `runbooks/mimir-outage.md` §"Distributor pod outage" |
| Postmortem owner | axis-observability |

## FM-02: Mimir multi-tenancy config drift

| Field | Value |
|---|---|
| Trigger | Helm config change merged without lane gate (impossible if lane is BLOCKER); OR live-cluster mutation by admin |
| Detection | `oya-governance-mimir-tenancy-enforced` lane fails OR continuous Helm-state-validator alarms |
| Tenant impact | Potential cross-tenant data exposure if not caught pre-deploy |
| Severity | Sev-1 (security breach risk) |
| Immediate mitigation | Auto-rollback to last green Helm state via ArgoCD; isolate cluster; declare incident; engage ops-security |
| RTO | ≤ 5 min auto-rollback; investigation may take days |
| Recovery runbook | `runbooks/mimir-outage.md` §"Tenancy drift" + ops-security incident playbook |
| Postmortem owner | ops-security + axis-observability |

## FM-03: slo-engine-worker outage

| Field | Value |
|---|---|
| Trigger | Worker pod crashloop (e.g., on PromQL parse bug, on OpenBao token-renewal failure) |
| Detection | `oya_observability_internal_evaluator_alive == 0` for ≥ 2min |
| Tenant impact | Every µservice's promotion **held** (fail-closed per ADR-0130); no false eligible verdicts |
| Severity | Sev-2 (gate functioning correctly — fail-closed — but operationally blocking; Sev-1 if persists > 1h) |
| Immediate mitigation | Worker HA leadership election re-runs; standby replica takes over; if all replicas fail, manual restart |
| RTO | ≤ 5 min for leadership-election recovery; ≤ 30 min for root-cause + fix |
| Recovery runbook | `runbooks/evaluator-down.md` |
| Postmortem owner | axis-observability |

## FM-04: Mimir block-storage outage (object storage unavailable)

| Field | Value |
|---|---|
| Trigger | S3-compatible object-storage outage in pack region |
| Detection | Mimir compactor + querier errors; `mimir_objstore_request_failures_total > 0` |
| Tenant impact | Recent ingest preserved in ingester memory; long-term query degraded; new block uploads delayed |
| Severity | Sev-2 (degraded read, no data loss within ingester window) |
| Immediate mitigation | Verify object-storage provider status; fail-over to DR pair where available (pack-eu has eu-frankfurt-1 + eu-amsterdam-1 DR pair) |
| RTO | Depends on provider; ≤ 1h failover for DR-pair packs; ≤ 4h for single-region packs (provider-recovery-dependent) |
| Recovery runbook | `runbooks/mimir-outage.md` §"Object storage outage" |
| Postmortem owner | ops-sre-reliability + cloud-secrets |

## FM-05: Object-storage data corruption (block SHA mismatch)

| Field | Value |
|---|---|
| Trigger | Hardware bit-rot, cosmic-ray flip, malicious tampering (T-T-02 in threat-model) |
| Detection | Mimir block-validator emits `mimir_block_sha_mismatch_total > 0`; affected block quarantined |
| Tenant impact | Specific time-range queries return partial data for affected tenant(s) |
| Severity | Sev-2 (limited data integrity issue) — Sev-1 if scope expands |
| Immediate mitigation | Quarantine affected blocks; restore from replication-factor-3 secondary; engage ops-security if pattern suggests tampering |
| RTO | ≤ 1h block-restore from replica |
| Recovery runbook | `runbooks/mimir-outage.md` §"Block corruption" |
| Postmortem owner | ops-security + axis-observability |

## FM-06: Cross-tenant query leak detected

| Field | Value |
|---|---|
| Trigger | LEAN check or runtime audit detects cross-tenant query result; Mimir's `X-Scope-OrgID` enforcement bypassed |
| Detection | `oya_tenant_unauthorized_read_attempt_total > 0` over 5min OR continuous-compliance lane alarm |
| Tenant impact | Potential confidentiality breach (DPIA R-02; threat T-I-01) |
| Severity | Sev-1 (security breach) |
| Immediate mitigation | Engage ops-security; freeze affected query endpoint; revoke implicated API keys; begin forensic trace |
| RTO | ≤ 5min for endpoint freeze; investigation + breach-notification chain may take 72h+ (per GDPR Art. 33) |
| Recovery runbook | `runbooks/security-incident.md` (cross-references `incident-response.md` §"Severity 1") |
| Postmortem owner | ops-security |

## FM-07: PII leakage via traces detected

| Field | Value |
|---|---|
| Trigger | Synthetic-PII detector (CI lane) flags a workload µservice emitting unredacted PII in span attributes |
| Detection | `oya_pii_redactor_miss_total > 0` |
| Tenant impact | DPIA R-01 risk realised; GDPR / KR PIPA / HIPAA violation possible |
| Severity | Sev-2 (data-protection breach; not full leak unless tenant cross-access also occurred) |
| Immediate mitigation | Engage workload µservice owner; patch OTel SDK redactor; purge affected trace span attributes (Tempo deletion API); enable redactor-aggressive mode globally |
| RTO | ≤ 1h for redactor patch deploy; ≤ 24h for purge of historical affected spans |
| Recovery runbook | `runbooks/security-incident.md` §"PII redaction failure" + DSR cascade if data-subject-impactful |
| Postmortem owner | ops-security + workload µservice owner |

## FM-08: Alertmanager alert storm

| Field | Value |
|---|---|
| Trigger | Cluster-wide outage triggers thousands of dependent alerts; OR mis-configured high-cardinality rule |
| Detection | `alertmanager_alerts_received_total` rate > 100/s sustained; OnCall pages backed up |
| Tenant impact | On-call fatigue; primary signals lost in noise |
| Severity | Sev-2 (operational; real signals at risk of being missed) |
| Immediate mitigation | Apply inhibition rules; silence root-cause alert chain; manually triage signal-to-noise; bring up bypass dashboard |
| RTO | ≤ 15min for silencing; permanent fix in rule tuning |
| Recovery runbook | `runbooks/alert-storm.md` (Slice B7 extension) |
| Postmortem owner | ops-sre-reliability |

## FM-09: Grafana OnCall integration outage

| Field | Value |
|---|---|
| Trigger | OnCall service crash, webhook signing key rotation failure |
| Detection | Two-channel corroboration fails: Mimir verdict transition fires but no OnCall ticket created within 60s |
| Tenant impact | Breaches paged late; auto-rollback still fires (rollback is independent of paging) |
| Severity | Sev-2 (operational visibility degraded; safety net for rollback still active) |
| Immediate mitigation | Failover to backup paging (PagerDuty-as-fallback if configured); manual escalation through Slack ops channel |
| RTO | ≤ 5min failover; OnCall recovery ≤ 1h |
| Recovery runbook | `runbooks/oncall-rotation.md` §"OnCall outage" |
| Postmortem owner | ops-sre-reliability |

## FM-10: Per-component release pointer corruption

| Field | Value |
|---|---|
| Trigger | Force-push attempt blocked by branch-protection; OR signed-commit verification fails |
| Detection | GitHub branch-protection rejects PATCH; emit `oya_release_pointer_rejected_total > 0` |
| Tenant impact | Promotion stuck; manual rollback path via PR review required |
| Severity | Sev-2 (operational delay) |
| Immediate mitigation | Verify SPIFFE identity of writer; re-sign with rotated key; escalate to ops-security if pattern suggests compromise |
| RTO | ≤ 1h for legitimate cases; security incident if not |
| Recovery runbook | `runbooks/rollback.md` §"Pointer corruption" |
| Postmortem owner | axis-observability + ops-security |

## FM-11: Recording-rule evaluation failure (Mimir ruler down)

| Field | Value |
|---|---|
| Trigger | Mimir ruler pod crash, expression-syntax error in newly-deployed rule |
| Detection | `cortex_ruler_evaluations_failed_total > 0` OR specific rule absent from recording-rule output |
| Tenant impact | `oya:current_verdict:by_microservice_env` aggregate stale; CI lane reads stale data; gate decisions may be wrong |
| Severity | Sev-2 (gate fail-closed safe default applies, but operational delay) |
| Immediate mitigation | Rollback offending rule; restart ruler; verify aggregate freshness via direct PromQL query |
| RTO | ≤ 15min for ruler restart; rule rollback via PR |
| Recovery runbook | `runbooks/mimir-outage.md` §"Ruler outage" |
| Postmortem owner | axis-observability |

## FM-12: Service-mesh canary cohort weight stuck

| Field | Value |
|---|---|
| Trigger | Istio VirtualService update fails; weighted routing stale |
| Detection | `service_mesh_traffic_split_lag_seconds > 60` |
| Tenant impact | Canary cohort over-/under-provisioned; signal quality degraded; gate decisions less reliable |
| Severity | Sev-3 (degraded; not blocking) |
| Immediate mitigation | Manual VirtualService re-apply via kubectl; verify Istio control-plane health |
| RTO | ≤ 30min for VirtualService re-apply |
| Recovery runbook | `runbooks/canary-graduation.md` §"Stuck weight" |
| Postmortem owner | ops-sre-reliability + axis-observability |

## FM-13: Pack-routing misroute (tenant data flows to wrong pack)

| Field | Value |
|---|---|
| Trigger | Workload µservice OTel collector config bug routes pack-eu tenant to pack-us cluster |
| Detection | Integration test caught at CI; OR runtime detector emits `oya_pack_misroute_total > 0` |
| Tenant impact | Cross-border-transfer violation (DPIA R-11); GDPR / KR PIPA breach |
| Severity | Sev-1 (regulatory breach) |
| Immediate mitigation | Quarantine misrouted data; engage ops-security + council-privacy; correct OTel config; begin breach-notification chain |
| RTO | ≤ 1h for routing correction; ≤ 72h for breach notification (GDPR Art. 33) |
| Recovery runbook | `runbooks/security-incident.md` §"Cross-border misroute" |
| Postmortem owner | council-privacy + ops-security |

## FM-14: Promotion-readiness CI lane flaky (false negatives)

| Field | Value |
|---|---|
| Trigger | Transient Mimir read failure causes lane to report "held" when verdict is actually "eligible" |
| Detection | `oya:current_verdict:by_microservice_env` returns eligible but CI lane fails with `mimir_read_error` |
| Tenant impact | Healthy promotion held; operational delay |
| Severity | Sev-3 |
| Immediate mitigation | Retry the lane; if persistent, manual override with 2-person rule + audit-chain emission |
| RTO | ≤ 15min for retry; manual override ≤ 1h |
| Recovery runbook | `runbooks/held-promotion-recovery.md` |
| Postmortem owner | axis-observability |

## FM-15: Capacity exhaustion (Mimir series limit hit)

| Field | Value |
|---|---|
| Trigger | Tenant exceeds `max_global_series_per_user` due to label explosion |
| Detection | Tenant's `mimir_distributor_received_samples_dropped_total` rate climbs; tenant rate-limit-exceeded SLO breach |
| Tenant impact | Excess metrics dropped; SLO data incomplete; eligibility verdicts may become unreliable |
| Severity | Sev-3 (tenant-specific, not cluster-wide) |
| Immediate mitigation | Notify tenant; increase per-tenant cardinality budget (if production-tier and within global budget); engage tenant on label-cardinality discipline |
| RTO | ≤ 1h budget increase; tenant remediation may take days |
| Recovery runbook | `runbooks/capacity-exhaustion.md` |
| Postmortem owner | ops-sre-reliability |

## RTO / RPO Summary

| Failure | RTO | RPO |
|---|---|---|
| Mimir distributor outage (single AZ) | 15min | 0 (replication-factor ≥ 3) |
| Mimir tenancy drift | 5min auto-rollback | 0 |
| Worker outage | 5min HA failover | 0 (stateless worker) |
| Object-storage outage | 1h (DR pair) / 4h (single region) | 5min (last ingester flush) |
| Block corruption | 1h | 0 (RF-3 secondary) |
| Cross-tenant leak | 5min freeze | N/A (breach occurred) |
| PII leakage | 1h patch + 24h purge | N/A |
| Alert storm | 15min silence | N/A |
| OnCall outage | 5min failover | N/A |
| Pointer corruption | 1h | 0 |
| Ruler outage | 15min | 0 |
| Mesh weight stuck | 30min | N/A |
| Pack misroute | 1h + 72h breach-notif | N/A |
| CI lane flaky | 15min retry | N/A |
| Capacity exhaustion | 1h budget | varies |

## SLO on Failure-Detection Pipeline

Meta-SLO: the observability substrate itself has an SLO that no failure remains undetected longer than its detection-window target.

| SLI | Target | Burn-rate alert |
|---|---|---|
| Alert-to-page latency (p99) | ≤ 60s | 14.4× burn over 1h |
| Detection-coverage (proportion of injected synthetic faults caught within window) | ≥ 99.5% | 6× burn over 6h |
| Two-channel corroboration completion | ≥ 99% within 90s | ticket burn 3d |
| False-positive page rate | ≤ 1 / week / on-call | informational |

## References

- `microservices/observability/threat-model.md` (each FM has at least one corresponding STRIDE / LINDDUN threat ID).
- `microservices/observability/dpia.md` (FM-06, FM-07, FM-13 map to R-02, R-01, R-11 respectively).
- `microservices/observability/incident-response.md` §"Severity Definitions".
- `microservices/observability/runbooks/*` (recovery procedures).
- `microservices/observability/capacity-model.md` (FM-15 + Mimir per-tenant limits).
- Grafana Mimir reliability docs — `grafana.com/docs/mimir/latest/operations/`.
- Google SRE Workbook ch. 12 (Postmortem culture).
