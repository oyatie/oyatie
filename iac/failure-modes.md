---
doc_class: FailureModeCatalog
title: Failure-Mode Catalog
microservice: cloud-iac
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-cloud-iac
deciders: ops-sre-reliability, axis-cloud-iac, ops-security, council-architecture
related_adrs: [ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/cloud-iac/threat-model.md
  - microservices/cloud-iac/dpia.md
  - microservices/cloud-iac/policy/iac-isolation.md
  - microservices/cloud-iac/incident-response.md
  - microservices/cloud-iac/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident affecting cloud-iac
doc_status: published
---

# Failure-Mode Catalog (cloud-iac µservice)

## Purpose

Enumerate the failure scenarios on-call must handle, the detection signal, immediate mitigation, root-cause-analysis path, recovery time objective (RTO), and the runbook that owns the recovery procedure. Cross-referenced from `incident-response.md` for severity classification.

## Failure-Mode Index

Each failure carries:
- **FM-ID**: stable identifier
- **Trigger**: precipitating event(s)
- **Detection**: SLI / alert / metric
- **Tenant impact**: tenant-facing experience
- **Severity**: Sev-1/2/3/4
- **Immediate mitigation**: on-call first-5-minute actions
- **RTO**: target recovery time
- **Recovery runbook**: procedure location
- **Postmortem owner**

## FM-01: Stuck apply (apply-job hangs > 15min)

| Field | Value |
|---|---|
| Trigger | Kubernetes apiserver resource conflict; finalizer loop; webhook hang; OpenTofu state-lock not released |
| Detection | `cloud_iac_apply_duration_seconds{quantile="0.99"} > 900` for ≥ 5min OR specific apply job in `Running` state > 15min |
| Tenant impact | One µservice's promotion blocked; queued applies for the same µservice queue up (per-µservice serialisation) |
| Severity | Sev-2 (single-µservice blocked) — Sev-1 if cluster-wide |
| Immediate mitigation | Abort apply via `kubectl delete pod` + `terraform force-unlock`; retry-with-backoff; if stuck-apply count > 3 over 1h, page secondary on-call |
| RTO | ≤ 15min for abort + retry; ≤ 1h if cluster-wide pattern |
| Recovery runbook | `runbooks/stuck-apply-recovery.md` |
| Postmortem owner | axis-cloud-iac |

## FM-02: Drift cascade (one mutation triggers wave of drift events)

| Field | Value |
|---|---|
| Trigger | Operator manually mutates a widely-shared resource (e.g., a ServiceMonitor) cascading into many µservices' drift reports |
| Detection | `cloud_iac_drift_events_total` rate > 100/min for ≥ 5min OR specific µservice drift-burst > 20 events in 5min |
| Tenant impact | Alert storm; on-call fatigue |
| Severity | Sev-2 (operational; real drift may hide in noise) |
| Immediate mitigation | Apply throttle: drift event grouping by µservice + resource-kind; silence flood patterns; manually triage signal |
| RTO | ≤ 15min for silencing; permanent fix in throttle config |
| Recovery runbook | `runbooks/drift-remediation.md` §"Cascade handling" |
| Postmortem owner | ops-sre-reliability + axis-cloud-iac |

## FM-03: Registry corruption (Postgres iac-state-index unrecoverable)

| Field | Value |
|---|---|
| Trigger | Postgres hardware failure; corrupt index; WAL replay failure |
| Detection | Postgres `pg_isready` fails OR replica lag > 60s OR application-level errors writing to iac-state-index |
| Tenant impact | Apply path blocked (registry unavailable); reads degraded |
| Severity | Sev-1 (registry is the source of truth) |
| Immediate mitigation | Failover to read-replica (promote to primary); if WAL replay fails, restore from S3 archive PITR; engage cloud-secrets for replica health |
| RTO | ≤ 30min replica promotion; ≤ 4h PITR restore |
| Recovery runbook | `runbooks/registry-restore.md` |
| Postmortem owner | axis-cloud-iac + cloud-secrets |

## FM-04: State-lock contention (concurrent applies race on OpenTofu state)

| Field | Value |
|---|---|
| Trigger | Two iac-applier-worker replicas attempt apply on same (microservice, pack, env) simultaneously; advisory-lock contention |
| Detection | `iac_state_lock_wait_seconds_p99 > 30` OR `iac_state_lock_timeout_total > 0` |
| Tenant impact | Apply latency degraded; one applier waits; second eventually aborts on lock-timeout |
| Severity | Sev-3 (transient; resolved by lock release) — Sev-2 if persistent |
| Immediate mitigation | Verify only one applier replica is processing; release stale locks via `terraform force-unlock` if held > 10min |
| RTO | ≤ 15min for force-unlock; permanent fix in applier-lock-acquisition discipline |
| Recovery runbook | `runbooks/state-lock-break.md` |
| Postmortem owner | axis-cloud-iac |

## FM-05: GitOps reconciler down (ArgoCD or Flux outage)

| Field | Value |
|---|---|
| Trigger | ArgoCD application-controller crash; etcd unavailable; reconciler pod-eviction storm |
| Detection | `argocd_app_info` cardinality drops; ArgoCD UI returns 5xx; `cloud_iac_reconciler_alive == 0` |
| Tenant impact | No automated git→cluster reconcile; manual apply via CLI required |
| Severity | Sev-2 (degraded; not blocking pre-existing applies); Sev-1 if persistent > 1h |
| Immediate mitigation | Verify HA replicas alive; restart unhealthy pods; failover ArgoCD primary to standby instance if pack has DR pair; engage Flux fallback for tenant-choice clusters |
| RTO | ≤ 30min for restart; ≤ 1h for DR-pair failover |
| Recovery runbook | `runbooks/gitops-reconciler-restart.md` |
| Postmortem owner | ops-sre-reliability + axis-cloud-iac |

## FM-06: Apply-elevation escape (apply mutates out-of-scope resource)

| Field | Value |
|---|---|
| Trigger | LEAN check failure missed at PR; Cedar policy bug; cluster RBAC misconfiguration |
| Detection | `cloud_iac_apply_scope_violation_total > 0` over 5min OR continuous-compliance lane alarm |
| Tenant impact | Potential cross-µservice mutation (DPIA R-01; threat T-T-03) |
| Severity | Sev-1 (security breach) |
| Immediate mitigation | Engage ops-security; freeze affected applier; revoke applier SA token; begin forensic trace; trigger rollback if mutation already occurred |
| RTO | ≤ 5min freeze; investigation + breach-notification may take 72h+ per GDPR Art. 33 |
| Recovery runbook | `runbooks/security-incident.md` (cross-references `incident-response.md` §"Severity 1") |
| Postmortem owner | ops-security |

## FM-07: SLSA L3 verification failure (Cosign / Rekor unreachable)

| Field | Value |
|---|---|
| Trigger | Rekor public log outage; Fulcio CA rotation issue; transient network failure to Sigstore upstream |
| Detection | `cloud_iac_slsa_verify_failure_total > 0` |
| Tenant impact | Applies refused; promotions held pending substrate recovery |
| Severity | Sev-2 (operational; gate fail-closed correct behavior) |
| Immediate mitigation | Verify Sigstore upstream status; cache last-known-good Rekor entries from local mirror; if persistent > 30min, declare Sev-2 incident |
| RTO | ≤ 30min for transient; ≤ 4h if Sigstore upstream outage |
| Recovery runbook | `runbooks/security-incident.md` §"Sigstore upstream outage" |
| Postmortem owner | ops-security + axis-cloud-iac |

## FM-08: Helm chart upstream-dep tampered (supply-chain attack)

| Field | Value |
|---|---|
| Trigger | Upstream chart registry serves a tampered chart (typosquat or repo compromise) |
| Detection | LEAN check `check-helm-chart-allowlist` refusal OR Cosign verify failure mid-pipeline |
| Tenant impact | Apply refused at validator; no actual mutation occurred |
| Severity | Sev-1 (supply-chain attempt; even if blocked) |
| Immediate mitigation | Quarantine the chart; engage ops-security; verify upstream registry integrity; remove offending allowlist entry; emergency-rotate any signing keys that touched the chart |
| RTO | ≤ 1h for quarantine; permanent investigation may take days |
| Recovery runbook | `runbooks/security-incident.md` §"Supply-chain compromise" |
| Postmortem owner | ops-security |

## FM-09: Rollback chain depth > 1 (rollback of rollback)

| Field | Value |
|---|---|
| Trigger | Reverting to prior SHA introduces a new regression; rollback-of-rollback invoked |
| Detection | `cloud_iac_rollback_chain_depth > 1` |
| Tenant impact | µservice accumulates regression debt; on-going production instability |
| Severity | Sev-2 (operational; gate functioning but situation indicates code quality issue) |
| Immediate mitigation | Escalate to ExecSponsor; manual review; consider longer-lived prior-good pointer |
| RTO | depends on root cause; aim for ≤ 4h to restore steady-state |
| Recovery runbook | `runbooks/rollback-orchestration.md` §"Rollback-of-rollback" |
| Postmortem owner | axis-cloud-iac + observability |

## FM-10: Cross-pack misroute (apply target wrong pack)

| Field | Value |
|---|---|
| Trigger | Workload µservice's pack-router config bug routes to wrong pack |
| Detection | Integration test at CI; runtime detector emits `cloud_iac_pack_misroute_total > 0` |
| Tenant impact | Cross-border-transfer violation (DPIA R-10); GDPR / KR PIPA breach risk |
| Severity | Sev-1 (regulatory breach) |
| Immediate mitigation | Quarantine misrouted data; engage ops-security + council-privacy; correct pack-router config; begin breach-notification chain |
| RTO | ≤ 1h routing correction; ≤ 72h breach notification |
| Recovery runbook | `runbooks/security-incident.md` §"Cross-pack misroute" |
| Postmortem owner | council-privacy + ops-security |

## FM-11: Drift-detection coverage gap (cycles missed > 1h)

| Field | Value |
|---|---|
| Trigger | Validator-worker outage; Postgres lag; Kubernetes apiserver throttling |
| Detection | `cloud_iac_drift_coverage_pct < 99.5` over 1h window |
| Tenant impact | Silent drift may persist beyond 1h SLO target |
| Severity | Sev-2 |
| Immediate mitigation | Verify validator-worker pods; check Postgres replica lag; throttle other workloads if apiserver-throttled |
| RTO | ≤ 30min for validator-worker recovery; ≤ 1h for cycle catch-up |
| Recovery runbook | `runbooks/drift-remediation.md` §"Coverage gap" |
| Postmortem owner | axis-cloud-iac |

## FM-12: Registry-worker outage (apply-state index writes stalled)

| Field | Value |
|---|---|
| Trigger | Worker crashloop; Postgres connection-pool exhaustion |
| Detection | `cloud_iac_registry_worker_alive == 0` for ≥ 2min |
| Tenant impact | New apply-state writes blocked; reads succeed from replica |
| Severity | Sev-2 |
| Immediate mitigation | Restart registry-worker pods; check Postgres connection-pool size; scale up if exhausted |
| RTO | ≤ 5min for restart; permanent fix in pool-sizing |
| Recovery runbook | `runbooks/registry-restore.md` §"Worker outage" |
| Postmortem owner | axis-cloud-iac |

## FM-13: Apply audit emission failure (audit-chain unreachable)

| Field | Value |
|---|---|
| Trigger | audit-chain µservice unreachable; transient network failure |
| Detection | `cloud_iac_audit_emit_failure_total > 0` |
| Tenant impact | Audit-chain seal pending; apply may proceed (depending on policy); audit gap if persistent |
| Severity | Sev-2 (compliance risk if persistent) |
| Immediate mitigation | Verify audit-chain availability; buffer events to local persistent queue; replay on recovery; if > 1h, escalate to Sev-1 (audit gap is compliance-sensitive) |
| RTO | ≤ 15min for transient; ≤ 1h for audit-chain failover |
| Recovery runbook | `runbooks/security-incident.md` §"Audit-chain unavailable" |
| Postmortem owner | axis-cloud-iac + audit-chain |

## FM-14: ArgoCD admission-webhook outage

| Field | Value |
|---|---|
| Trigger | Webhook pod crashloop; cert rotation failure |
| Detection | ArgoCD admission errors; `argocd_admission_webhook_request_failures_total > 0` |
| Tenant impact | New Application resources rejected; existing apps continue reconciling |
| Severity | Sev-2 |
| Immediate mitigation | Restart webhook pods; check cert renewal; if persistent, fail-open temporarily (with audit-chain emission) until restored |
| RTO | ≤ 15min |
| Recovery runbook | `runbooks/gitops-reconciler-restart.md` §"Admission webhook" |
| Postmortem owner | axis-cloud-iac + ops-sre-reliability |

## FM-15: Renderer non-determinism (re-render produces different digest)

| Field | Value |
|---|---|
| Trigger | Helm chart references current timestamp; environment-variable interpolation; non-deterministic value source |
| Detection | LEAN check `cloud-iac-render-determinism` fails OR runtime content-digest mismatch on re-render |
| Tenant impact | Render cache invalidated unnecessarily; apply-state index may drift |
| Severity | Sev-3 |
| Immediate mitigation | Identify non-deterministic source in IaC; fix at PR; document non-determinism convention violation |
| RTO | ≤ 30min for identification + fix |
| Recovery runbook | `runbooks/drift-remediation.md` §"Non-determinism" |
| Postmortem owner | axis-cloud-iac + offending µservice owner |

## RTO / RPO Summary

| Failure | RTO | RPO |
|---|---|---|
| Stuck apply | 15min | N/A |
| Drift cascade | 15min silence | N/A |
| Registry corruption | 30min replica promote / 4h PITR | 5min |
| State-lock contention | 15min | N/A |
| GitOps reconciler down | 30min restart / 1h DR | N/A |
| Apply-elevation escape | 5min freeze | N/A (breach occurred) |
| SLSA verify failure | 30min transient / 4h upstream | N/A |
| Helm supply-chain | 1h quarantine | N/A |
| Rollback chain > 1 | 4h to steady-state | 0 |
| Cross-pack misroute | 1h + 72h breach-notif | N/A |
| Drift coverage gap | 30min | N/A |
| Registry worker outage | 5min restart | 0 |
| Audit emission failure | 15min transient / 1h failover | varies |
| Admission webhook outage | 15min | N/A |
| Renderer non-determinism | 30min fix | N/A |

## SLO on Failure-Detection Pipeline

Meta-SLO: the cloud-iac substrate itself has an SLO that no failure remains undetected longer than its detection-window target.

| SLI | Target | Burn-rate alert |
|---|---|---|
| Alert-to-page latency (p99) | ≤ 60s | 14.4× burn over 1h |
| Detection-coverage (proportion of injected synthetic faults caught) | ≥ 99.5% | 6× burn over 6h |
| Drift-coverage SLO (% of clusters polled per 1h cycle) | ≥ 99.5% | 14.4× burn over 1h |
| Apply success-rate SLO | ≥ 99.5% | 14.4× burn over 1h |
| False-positive page rate | ≤ 1 / week / on-call | informational |

## References

- `microservices/cloud-iac/threat-model.md` (each FM maps to STRIDE / LINDDUN threat IDs).
- `microservices/cloud-iac/dpia.md` (FM-06, FM-10 map to R-01, R-10).
- `microservices/cloud-iac/incident-response.md` §"Severity Definitions".
- `microservices/cloud-iac/runbooks/*` (recovery procedures).
- `microservices/cloud-iac/capacity-model.md`.
- ArgoCD operations docs — `argo-cd.readthedocs.io/en/stable/operator-manual/`.
- OpenTofu state-lock docs — `opentofu.org/docs/language/state/locking/`.
- Google SRE Workbook ch. 12 (Postmortem culture).
