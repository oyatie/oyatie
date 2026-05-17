---
doc_class: FailureModeCatalog
title: Failure-Mode Catalog
microservice: cell
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-cell-substrate
deciders: ops-sre-reliability, axis-cell-substrate, ops-security, council-architecture
related_adrs: [ADR-0117, ADR-0130, ADR-0131]
related_artifacts:
  - microservices/cell/threat-model.md
  - microservices/cell/dpia.md
  - microservices/cell/policy/cell-boundary.md
  - microservices/cell/incident-response.md
  - microservices/cell/runbooks/
review_cadence: quarterly + after every Sev-1 / Sev-2 incident affecting cell
doc_status: published
---

# Failure-Mode Catalog (cell µservice)

## Purpose

Enumerate the failure scenarios on-call must handle. Each FM carries trigger, detection, tenant impact, severity, immediate mitigation, RTO, runbook reference, and postmortem owner. Cross-referenced from `incident-response.md`.

## Failure-Mode Index

## FM-01: Postgres cell-registry primary outage (single pack)

| Field | Value |
|---|---|
| Trigger | Cluster autoscaler eviction; hardware failure; kernel panic; OOM kill |
| Detection | `cell_registry_postgres_primary_up == 0` for ≥ 30s; or replica-lag > 60s |
| Tenant impact | Read path absorbs via in-process cache (60s TTL); writes briefly stalled until failover |
| Severity | Sev-2 (degraded; no data loss within streaming-replication window) |
| Immediate mitigation | Postgres HA failover to replica (auto via Patroni / CloudNativePG); verify lag = 0 post-promotion |
| RTO | ≤ 30 s for replica promotion; ≤ 5 min for full cluster restoration |
| Recovery runbook | `runbooks/cell-rebalance.md` does not own this — see ops-sre-reliability Postgres outage runbook |
| Postmortem owner | axis-cell-substrate + cloud-k8s |

## FM-02: Cell-boundary lane config drift

| Field | Value |
|---|---|
| Trigger | Lane disabled in branch-protection.yaml; OR runtime check bypassed via emergency override |
| Detection | `oya-governance-branch-protection-conformance` lane fails OR continuous-compliance validator alarms |
| Tenant impact | Risk of cross-cell coupling slipping through PR; potential tenant-isolation violation |
| Severity | Sev-1 (security boundary degraded) |
| Immediate mitigation | Auto-rollback branch-protection.yaml to last green; isolate affected branch; engage ops-security |
| RTO | ≤ 5 min auto-rollback; investigation may take days |
| Recovery runbook | `runbooks/cell-rebalance.md` cross-references ops-security incident playbook |
| Postmortem owner | ops-security + axis-foundry |

## FM-03: Scheduler worker outage

| Field | Value |
|---|---|
| Trigger | Worker pod crashloop (e.g., on binpack-decision serialization bug); OpenBao token renewal failure |
| Detection | `oya_cell_scheduler_alive == 0` for ≥ 2 min; or placement queue depth > 100 |
| Tenant impact | New tenant onboarding stalls; existing tenants unaffected (read path independent) |
| Severity | Sev-2 (operationally blocking; Sev-1 if persists > 1h) |
| Immediate mitigation | HA leadership re-election; standby replica takes over; if all replicas fail, manual restart |
| RTO | ≤ 5 min leadership-election recovery; ≤ 30 min for root-cause + fix |
| Recovery runbook | `runbooks/scheduler-restart.md` |
| Postmortem owner | axis-cell-substrate |

## FM-04: Host-pool exhaustion

| Field | Value |
|---|---|
| Trigger | Burst tenant onboarding; warm-pool depleted; new-node provisioning > pool refill rate |
| Detection | `oya_cell_warm_pool_size < 2` for ≥ 5 min |
| Tenant impact | New cell creation queued; new tenant onboarding waits (visible progress) |
| Severity | Sev-2 (degraded onboarding; existing tenants unaffected) |
| Immediate mitigation | Trigger immediate node-pool scale-up via cluster autoscaler; tighten incoming placement-request rate-limit |
| RTO | ≤ 5 min for node provisioning (hyperscaler API dependent); ≤ 15 min for warm-pool refill |
| Recovery runbook | `runbooks/host-pool-exhaustion.md` |
| Postmortem owner | ops-sre-reliability + cloud-k8s |

## FM-05: Cell-create timeout / partial state

| Field | Value |
|---|---|
| Trigger | lifecycle-manager started cell create but Cluster API / Postgres / OpenBao operation partially completed |
| Detection | Cell stuck in `provisioning` state > 30 min |
| Tenant impact | Tenant cannot be placed (placement waits for cell `ready`); fallback to retry on different cell |
| Severity | Sev-3 (operational glitch; scheduler retries) |
| Immediate mitigation | Manual reconcile via `oya cell reconcile --cell <id>`; lifecycle-manager state machine attempts forward progress; if blocked, manually transition to `decommissioning` to clean up |
| RTO | ≤ 30 min reconcile; ≤ 1h cleanup |
| Recovery runbook | `runbooks/cell-rebalance.md` §"Provisioning timeout" |
| Postmortem owner | axis-cell-substrate + cloud-k8s |

## FM-06: Migration race (two operators migrate same tenant)

| Field | Value |
|---|---|
| Trigger | Concurrent migration commands; operator coordination failure |
| Detection | Postgres advisory-lock-contention metric > 0; or second migration command receives `MigrationInProgress` error |
| Tenant impact | Bounded — advisory lock serializes; second op observes existing plan + joins or aborts |
| Severity | Sev-3 (well-handled by design; only Sev-2 if lock-contention metric trend rises) |
| Immediate mitigation | Lock-holder completes migration; second op waits or aborts |
| RTO | ≤ duration of in-flight migration (≤ 10 min p99) |
| Recovery runbook | `runbooks/tenant-migration.md` §"Concurrent migration handling" |
| Postmortem owner | axis-cell-substrate |

## FM-07: Cross-cell DB query attempt (workload µservice bug)

| Field | Value |
|---|---|
| Trigger | Workload µservice's PR slipped through cell-boundary lane; OR runtime drift bug |
| Detection | `oya_cell_boundary_violation_total > 0` (Postgres RLS deny) |
| Tenant impact | Postgres RLS refuses; query returns 0 rows; may surface as user-visible empty-result depending on workload defensive handling |
| Severity | Sev-1 (tenant-isolation breach — even if RLS catches, this is the load-bearing class) |
| Immediate mitigation | Audit lineage of offending code path; engage workload owner; revert offending PR if possible; root-cause lane gap |
| RTO | ≤ 1h investigation; ≤ 4h fix + redeploy |
| Recovery runbook | `runbooks/cell-rebalance.md` §"Cross-cell query incident" + ops-security incident playbook |
| Postmortem owner | ops-security + axis-cell-substrate + workload-owner |

## FM-08: Host drain stuck

| Field | Value |
|---|---|
| Trigger | Pod eviction blocked (PDB violation; finalizer hanging; PVC not detaching) |
| Detection | `host_pool_drain_duration_seconds > 1800` (30 min) |
| Tenant impact | Hardware retirement delayed; no direct tenant impact unless drain blocks recovery |
| Severity | Sev-3 (operational); Sev-2 if hardware retirement is urgent |
| Immediate mitigation | Inspect stuck pod; force-delete (last resort); fix PDB; engage workload owner |
| RTO | ≤ 1h investigation; ≤ 4h resolution |
| Recovery runbook | `runbooks/host-pool-exhaustion.md` §"Drain stuck" + cross-references workload-owner runbooks |
| Postmortem owner | cloud-k8s + workload-owner |

## FM-09: Cross-pack assignment attempt (residency breach risk)

| Field | Value |
|---|---|
| Trigger | scheduler bug; Cedar policy regression; insider-malicious |
| Detection | `oya_cell_cross_pack_attempt_total > 0` |
| Tenant impact | Cedar/RLS rejects; bounded blast-radius; but Sev-1 because residency breaches are regulator-notifiable |
| Severity | Sev-1 |
| Immediate mitigation | Verify rejection; trace source; engage council-privacy; check no commit occurred; if commit occurred, immediate migration to correct pack + breach-notification chain |
| RTO | ≤ 1h investigation; per `incident-response.md` for breach-notification chain |
| Recovery runbook | `runbooks/cell-decommission.md` §"Residency breach recovery" + ops-security playbook |
| Postmortem owner | council-privacy + axis-cell-substrate + ops-security |

## FM-10: Cell-decommission soft-delete window expiry (irreversible)

| Field | Value |
|---|---|
| Trigger | 30d soft-delete window elapses; Postgres schema drops + S3 prefix deletes execute |
| Detection | `oya_cell_decommission_finalized_total` (informational; not an error) |
| Tenant impact | Tenant data destroyed; recovery requires backup restore |
| Severity | Sev-3 (controlled; if surprise: Sev-1) |
| Immediate mitigation | Pre-finalization audit: confirm zero tenants still bound + zero queued migration plans referencing the cell |
| RTO | n/a (finalisation is the recovery; reversal not possible post-finalize) |
| Recovery runbook | `runbooks/cell-decommission.md` |
| Postmortem owner | axis-cell-substrate (annual review) |

## FM-11: Split-brain in cell-registry HA failover

| Field | Value |
|---|---|
| Trigger | Network partition causing both Postgres primary candidates to accept writes; CloudNativePG / Patroni failure |
| Detection | `postgres_write_quorum_break == 1` |
| Tenant impact | Risk of divergent cell-assignment state across replicas; reads inconsistent during partition |
| Severity | Sev-1 (data-consistency risk) |
| Immediate mitigation | Quorum loss → both nodes step down to read-only; manual operator decides "preferred primary"; force-fence the loser; reconcile diverged writes from union-merge ledger |
| RTO | ≤ 1h operator decision + fence; ≤ 4h reconciliation |
| Recovery runbook | `runbooks/split-brain.md` |
| Postmortem owner | cloud-k8s + axis-cell-substrate |

## FM-12: SPIRE server outage (per-cell SVID issuance halts)

| Field | Value |
|---|---|
| Trigger | SPIRE server pod crashloop; HA quorum loss |
| Detection | `spire_server_attestation_success_rate < 0.99` for ≥ 5 min |
| Tenant impact | New pod attestation fails → new workload pods can't get SVID → can't connect to Postgres → cells don't start fresh workloads; existing workloads (with valid SVID) unaffected during TTL |
| Severity | Sev-2 (degraded; existing SVIDs valid ≤ 24h) |
| Immediate mitigation | SPIRE server replica failover; verify quorum |
| RTO | ≤ 15 min recovery |
| Recovery runbook | `runbooks/scheduler-restart.md` cross-references SPIRE recovery |
| Postmortem owner | ops-security + cloud-k8s |

## FM-13: Cluster API control-plane outage

| Field | Value |
|---|---|
| Trigger | Management cluster failure |
| Detection | Cluster API controllers not reconciling; CRD events stale |
| Tenant impact | New cell create / delete halted; existing cells unaffected |
| Severity | Sev-2 |
| Immediate mitigation | Management cluster recovery (etcd / API server / controllers); lifecycle-manager queues requests for replay on recovery |
| RTO | ≤ 1h |
| Recovery runbook | `runbooks/cell-rebalance.md` §"Management cluster outage" |
| Postmortem owner | cloud-k8s |

## FM-14: Tenant-assignment cache poisoning

| Field | Value |
|---|---|
| Trigger | In-process cache corruption due to TTL bug; OR cache poisoning attack (rare) |
| Detection | `cell_registry_cache_inconsistency_total > 0` (cache vs Postgres mismatch) |
| Tenant impact | Stale cell_id returned briefly; workload may attempt connect with wrong credentials → Postgres auth refuses (defence-in-depth) |
| Severity | Sev-2 (defence-in-depth holds; potential latency spike during cache-rebuild) |
| Immediate mitigation | Force cache flush; rebuild from Postgres; verify consistency |
| RTO | ≤ 5 min cache rebuild |
| Recovery runbook | `runbooks/scheduler-restart.md` §"Cache rebuild" |
| Postmortem owner | axis-cell-substrate |

## FM-15: Audit-chain seal outage (cannot seal cell events)

| Field | Value |
|---|---|
| Trigger | audit-chain µservice outage; OR signing-key rotation failure |
| Detection | `audit_chain_seal_latency_seconds_p99 > 5` |
| Tenant impact | Cell events accumulate unsealed in queue; bounded by audit-chain SLO |
| Severity | Sev-2 (audit posture degraded) |
| Immediate mitigation | Audit-chain recovery per its own runbook; cell writes continue with deferred seal (acceptable for short window) |
| RTO | ≤ 1h |
| Recovery runbook | cross-references `microservices/audit-chain/runbooks/` |
| Postmortem owner | audit-chain + axis-cell-substrate |

## Per-Pack Adjustments

Each pack may have specific FM-NN extensions (e.g., HIPAA pack adds FM-PHI-LEAK; KR pack adds FM-PIPC-INSPECTION); recorded in `regional-packs/<pack>/cell-failure-modes-overlay.md`.

## References

- `microservices/cell/threat-model.md`.
- `microservices/cell/dpia.md`.
- `microservices/cell/policy/cell-boundary.md`.
- `microservices/cell/incident-response.md`.
- `microservices/cell/runbooks/`.
- Bominal ADR-0009 + ADR-0019.
- Kubernetes Multi-Tenancy SIG failure-mode patterns.
