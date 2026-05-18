---
doc_class: FailureModeCatalog
title: Failure-Mode Catalog (foundry-supervisor)
microservice: foundry-supervisor
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-foundry-control-plane
deciders: ops-sre-reliability, axis-foundry-control-plane, ops-security, council-architecture
related_adrs: [ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/foundry-supervisor/threat-model.md
  - microservices/foundry-supervisor/dpia.md
  - microservices/foundry-supervisor/policy/supervisor-isolation.md
  - microservices/foundry-supervisor/incident-response.md
  - microservices/foundry-supervisor/runbooks/
review_cadence: quarterly + after every Sev-1/2 incident
doc_status: published
---

# Failure-Mode Catalog (foundry-supervisor µservice)

## Purpose

Enumerate the failure scenarios on-call must handle; the detection signal; immediate mitigation; RTO; recovery runbook. Cross-referenced from `incident-response.md`.

## FM-01: Kill-switch latency spike (engage > p99 target)

| Field | Value |
|---|---|
| Trigger | Redis cluster slow; CRD watch fan-out delayed; supervisor REST queue backlog |
| Detection | `oya_supervisor_kill_switch_engage_latency_p99 > 1s` for ≥ 2 min |
| Tenant impact | Safety-critical SLO breach; runaway capabilities may continue acting briefly |
| Severity | **Sev-1 (always)** — supervisor-down on safety surface |
| Immediate mitigation | Verify Redis cluster health; engage degradation mode (assume-engaged fail-closed); engage ops-security on-call |
| RTO | ≤ 5 min for fail-closed mode; ≤ 30 min for root-cause fix |
| Recovery runbook | `runbooks/kill-switch-engage.md` |
| Postmortem owner | axis-foundry-control-plane + ops-security |

## FM-02: Deployment stuck (canary phase hung)

| Field | Value |
|---|---|
| Trigger | foundry-runtime drain hung; observability EligibilityChanged not arriving; observe-window misconfig |
| Detection | `oya_supervisor_deployment_phase_age_seconds > 900` (> 15 min in same phase) |
| Tenant impact | Capability rollout stuck at current canary; new requests follow prior version |
| Severity | Sev-2 |
| Immediate mitigation | Inspect rollout state; manually advance or rollback; engage tenant DPO if their config |
| RTO | ≤ 15 min advance/rollback |
| Recovery runbook | `runbooks/deployment-rollback.md` §"Stuck canary" |
| Postmortem owner | axis-foundry-control-plane |

## FM-03: Fleet-state corruption (Postgres rows inconsistent with K8s reality)

| Field | Value |
|---|---|
| Trigger | Postgres rollback during write; Operator reconcile partial; admission webhook permissive bug |
| Detection | Nightly drift-detector emits `oya_supervisor_fleet_state_divergence_total > 0` |
| Tenant impact | Fleet-state queries return stale data; deployment decisions may be wrong |
| Severity | Sev-2 (escalates to Sev-1 if drift is wide-scale) |
| Immediate mitigation | Block writes via REST circuit-breaker; replay reconcile from K8s CRDs (CRDs are source-of-truth) |
| RTO | ≤ 30 min reconcile replay |
| Recovery runbook | `runbooks/fleet-state-recovery.md` |
| Postmortem owner | axis-foundry-control-plane |

## FM-04: Redis failover (one shard fails over to replica)

| Field | Value |
|---|---|
| Trigger | Redis shard pod loss; OOMkilled; network partition |
| Detection | `redis_cluster_replica_promoted_total > 0`; `oya_supervisor_kill_switch_engage_latency_p99` may briefly spike |
| Tenant impact | Brief latency spike (≤ 2 s); kill-switch SLO normally preserved |
| Severity | Sev-2 if latency stays elevated, else Sev-3 |
| Immediate mitigation | Verify Redis cluster mode is functioning; rescale shards if persistent |
| RTO | ≤ 5 min replica promotion (automatic) |
| Recovery runbook | `runbooks/kubernetes-operator-restart.md` (covers Redis chaos) |
| Postmortem owner | ops-sre-reliability |

## FM-05: Postgres master loss

| Field | Value |
|---|---|
| Trigger | Pod loss; AZ failure; OOMkilled |
| Detection | Patroni status + `postgres_master_unreachable` alarm |
| Tenant impact | Control-plane availability gap ≤ 30 s; in-flight precondition checks may fail |
| Severity | Sev-2 |
| Immediate mitigation | Verify Patroni promoted replica; PgBouncer re-routes |
| RTO | ≤ 30 s automated promotion; manual oversight by ops-sre-reliability |
| Recovery runbook | `runbooks/kubernetes-operator-restart.md` §"Postgres master loss" |
| Postmortem owner | ops-sre-reliability + axis-foundry-control-plane |

## FM-06: Kubernetes Operator crashloop

| Field | Value |
|---|---|
| Trigger | CRD parse bug; OpenBao token-renewal failure; controller-runtime regression |
| Detection | `kubernetes_operator_alive == 0` for ≥ 2 min |
| Tenant impact | CRD reconcile lag; deployment + kill-switch propagation delayed |
| Severity | Sev-2 (escalates to Sev-1 if all replicas crashloop > 10 min) |
| Immediate mitigation | Lease-leadership re-runs; standby controller takes over; if all replicas fail, manual restart + revert offending CRD/code |
| RTO | ≤ 5 min HA failover; ≤ 30 min for root-cause + fix |
| Recovery runbook | `runbooks/kubernetes-operator-restart.md` |
| Postmortem owner | axis-foundry-control-plane |

## FM-07: Autonomy-policy denial flood (false negatives)

| Field | Value |
|---|---|
| Trigger | OpenBao tenant-resolver returns stale entitlements; Cedar fragment regression |
| Detection | `oya_supervisor_autonomy_violation_total` rate spike + tenant complaints |
| Tenant impact | Legitimate invocations refused; tenant operations degraded |
| Severity | Sev-2 |
| Immediate mitigation | Trace per-tenant; rollback offending Cedar fragment via Helm; manual override with 2-person rule for affected tenant scope |
| RTO | ≤ 15 min rollback; ≤ 1 h for tenant-scope overrides |
| Recovery runbook | `runbooks/autonomy-violation.md` |
| Postmortem owner | ops-security + axis-foundry-control-plane |

## FM-08: Supervision-event-bus backlog

| Field | Value |
|---|---|
| Trigger | Deployment storm; Redis Streams consumer lag; foundry-evidence ingest slow |
| Detection | `oya_supervisor_supervision_event_bus_lag_p99 > 500 ms` |
| Tenant impact | Audit-chain seals delayed; observability dashboards lagging |
| Severity | Sev-2 |
| Immediate mitigation | Backpressure publisher (pause non-critical events); scale evidence ingest; engage supervision-bus replay |
| RTO | ≤ 30 min |
| Recovery runbook | `runbooks/supervision-bus-replay.md` |
| Postmortem owner | axis-foundry-control-plane |

## FM-09: Cedar evaluation latency spike

| Field | Value |
|---|---|
| Trigger | Pathological input shape; Cedar runtime upgrade regression |
| Detection | `oya_supervisor_cedar_eval_p99 > 50 ms` |
| Tenant impact | Per-invocation precondition checks slow; runtime queues |
| Severity | Sev-3 (degraded; not blocking) |
| Immediate mitigation | Apply field-length bounds at REST; rollback Cedar fragment if recently changed |
| RTO | ≤ 15 min |
| Recovery runbook | `runbooks/autonomy-violation.md` §"Cedar latency" |
| Postmortem owner | axis-foundry-control-plane |

## FM-10: CRD admission webhook outage

| Field | Value |
|---|---|
| Trigger | Webhook pod crash; cert-manager cert expiry |
| Detection | `kubernetes_admission_webhook_unreachable` |
| Tenant impact | New deployments rejected; rollouts paused |
| Severity | Sev-2 |
| Immediate mitigation | Restart webhook pods; rotate cert if expired; failover to backup CA |
| RTO | ≤ 10 min |
| Recovery runbook | `runbooks/kubernetes-operator-restart.md` §"Admission webhook" |
| Postmortem owner | ops-sre-reliability |

## FM-11: Drain stuck (in-flight workers not completing)

| Field | Value |
|---|---|
| Trigger | foundry-runtime worker hung; agent capability long-running |
| Detection | `oya_supervisor_drain_age_seconds > 600` (> 10 min) |
| Tenant impact | Deployment held; new requests blocked |
| Severity | Sev-2 |
| Immediate mitigation | Inspect agent state; engage tenant DPO; force-terminate after grace period (with audit-chain emission) |
| RTO | ≤ 30 min force-terminate |
| Recovery runbook | `runbooks/fleet-state-recovery.md` §"Drain stuck" |
| Postmortem owner | axis-foundry-control-plane |

## FM-12: Capability YAML schema-violation flood

| Field | Value |
|---|---|
| Trigger | Tenant adopts new YAML pattern not yet supported; schema regression |
| Detection | `oya_supervisor_admit_schema_rejection_total` spike |
| Tenant impact | Tenant deployments blocked |
| Severity | Sev-3 |
| Immediate mitigation | Notify tenant; provide migration guidance; if regression, rollback schema validator |
| RTO | ≤ 1 h |
| Recovery runbook | `runbooks/deployment-rollback.md` §"Schema regression" |
| Postmortem owner | axis-foundry-control-plane |

## FM-13: Cross-pack misroute (tenant fleet writes go to wrong pack)

| Field | Value |
|---|---|
| Trigger | Workload OTel/Operator config bug; pack-router Cedar fragment regression |
| Detection | Integration test catches at CI; OR runtime `oya_supervisor_pack_misroute_total > 0` |
| Tenant impact | Cross-border-transfer violation (DPIA R-07); GDPR / KR PIPA breach |
| Severity | **Sev-1** (regulatory breach) |
| Immediate mitigation | Quarantine misrouted data; engage ops-security + council-privacy; correct pack-router; begin breach-notification |
| RTO | ≤ 1 h correction; ≤ 72 h breach notification |
| Recovery runbook | `runbooks/security-incident.md` §"Cross-pack misroute" (cross-references incident-response.md) |
| Postmortem owner | council-privacy + ops-security |

## FM-14: Signing-key rotation failure

| Field | Value |
|---|---|
| Trigger | OpenBao rotation runs; supervisor doesn't pick up new key in time |
| Detection | `oya_supervisor_signing_key_age_days > 90` OR `oya_supervisor_event_signature_invalid_total > 0` |
| Tenant impact | Supervision events fail audit-chain validation; rollbacks may fail signature check |
| Severity | Sev-2 |
| Immediate mitigation | Force-reload signing key from OpenBao; verify new sig; emit `signing_key_rotated` audit event |
| RTO | ≤ 15 min reload |
| Recovery runbook | `runbooks/kubernetes-operator-restart.md` §"Signing key" |
| Postmortem owner | ops-security |

## FM-15: 2-person-rule bypass attempt (insider threat)

| Field | Value |
|---|---|
| Trigger | Lone actor attempts fleet-wide kill-switch via OpenBao JIT (would require 2 signatures) |
| Detection | `oya_supervisor_two_person_rule_violation_total > 0` |
| Tenant impact | Engagement refused; no impact (defense worked) |
| Severity | Sev-2 (security event, but defense worked) — Sev-1 if successful |
| Immediate mitigation | Engage ops-security; forensic trace of actor identity; review JIT escalation policy |
| RTO | ≤ 5 min engagement refusal (automatic); forensic may take days |
| Recovery runbook | `runbooks/security-incident.md` §"Insider 2-person bypass" |
| Postmortem owner | ops-security |

## RTO / RPO Summary

| Failure | RTO | RPO |
|---|---|---|
| Kill-switch latency spike | 5 min fail-closed; 30 min fix | 0 (state in CRD source-of-truth) |
| Deployment stuck | 15 min | 0 |
| Fleet-state corruption | 30 min reconcile | 0 (CRDs authoritative) |
| Redis failover | 5 min (auto) | 0 (AOF + cluster replication) |
| Postgres master loss | 30 s (auto Patroni) | 0 (synchronous replication) |
| Operator crashloop | 5 min HA failover | 0 |
| Autonomy denial flood | 15 min rollback | 0 |
| Supervision-bus backlog | 30 min | depends on lag |
| Cedar latency | 15 min | 0 |
| Admission webhook outage | 10 min | 0 |
| Drain stuck | 30 min force-terminate | varies |
| Schema-violation flood | 1 h | 0 |
| Cross-pack misroute | 1 h + 72 h breach-notif | N/A |
| Signing-key rotation failure | 15 min reload | 0 |
| 2-person-rule bypass | 5 min refusal | N/A |

## SLO on Failure-Detection Pipeline

| SLI | Target | Burn-rate alert |
|---|---|---|
| Alert-to-page latency p99 | ≤ 60 s | 14.4× over 1h |
| Detection-coverage (synthetic chaos fault catch rate) | ≥ 99.5% | 6× over 6h |
| Two-channel corroboration completion | ≥ 99% within 90s | ticket burn 3d |

## References

- `microservices/foundry-supervisor/threat-model.md`.
- `microservices/foundry-supervisor/dpia.md`.
- `microservices/foundry-supervisor/incident-response.md`.
- `microservices/foundry-supervisor/runbooks/`.
- `microservices/foundry-supervisor/capacity-model.md`.
- PostgreSQL HA, Redis Cluster, kube-rs, Cedar v4.
- Google SRE Workbook ch. 12.
