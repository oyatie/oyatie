---
doc_class: Runbook
title: Mimir outage (distributor / ingester / ruler / object-storage)
microservice: observability
severity: "Sev-1 (multi-AZ outage) / Sev-2 (single component)"
status: Accepted
owner_team: ops-sre-reliability + axis-observability
date: 2026-05-17
related_artifacts:
  - microservices/observability/failure-modes.md (FM-01, FM-02, FM-04, FM-05, FM-11)
  - microservices/observability/multi-region.md
  - microservices/observability/capacity-model.md
doc_status: published
---

# Runbook: Mimir outage

## Trigger

Any of:
- Mimir distributor errors > threshold (FM-01)
- Mimir multi-tenancy config drift detected (FM-02)
- Mimir object-storage outage (FM-04)
- Mimir block-SHA mismatch (FM-05)
- Mimir ruler evaluation failure (FM-11)

## Severity

- Single-component, single-AZ: Sev-2.
- Multi-AZ within a pack region: Sev-1 (pack potentially failed over to DR pair).
- Tenancy config drift: Sev-1 (security risk).
- Object-storage outage: Sev-2 (recent ingest preserved; cold queries degraded).

## Distributor pod outage (FM-01)

| Step | Action | Time |
|---|---|---|
| 1 | Verify HPA is scaling: `kubectl -n observability get hpa mimir-distributor` | ≤ 2 min |
| 2 | Verify cross-AZ rebalance: `kubectl get pods -n observability -l app=mimir-distributor -o wide` (multiple AZs represented) | ≤ 2 min |
| 3 | If pod-eviction storm: cordon affected node, wait for HPA to scale up replacements | ≤ 10 min |
| 4 | Verify ingest path latency: `mimir_distributor_request_duration_seconds{quantile="0.99"} < 1s` | ≤ 5 min |
| 5 | If persistent: declare Sev-1; consider DR failover per `multi-region.md` | – |

## Tenancy config drift (FM-02)

| Step | Action | Time |
|---|---|---|
| 1 | Engage ops-security; declare Sev-1 + open `#inc-sec-<id>` | immediate |
| 2 | Auto-rollback via ArgoCD to last green Helm state (CI lane has already prevented merge if pre-deploy; live mutation is what triggered) | ≤ 5 min |
| 3 | Validate `mimir_distributor_runtime_config_hash` against expected | ≤ 2 min |
| 4 | Audit: who mutated the cluster? OpenBao audit log + Kubernetes audit log | – |
| 5 | If exposure confirmed (`oya_tenant_unauthorized_read_attempt_total > 0` during drift window): begin breach-notification chain per `incident-response.md` §"Regulatory Notifications" | per pack |

## Object-storage outage (FM-04)

| Step | Action | Time |
|---|---|---|
| 1 | Verify object-storage provider status (Oracle OCI status page) | ≤ 5 min |
| 2 | Verify ingest path is unaffected (ingesters buffer in-memory until storage returns) | ≤ 5 min |
| 3 | Validate ingester memory pressure: `kubectl top pods -l app=mimir-ingester` (memory < 80% before going OOM) | ≤ 5 min |
| 4 | If memory > 80%: scale up ingester replicas; accelerate compactor flush attempts | ≤ 10 min |
| 5 | If outage > 30 min AND pack has DR pair: initiate DR failover per `multi-region.md` §"DR Failover" | ≤ 35 min |
| 6 | Notify tenants: status page + email per `incident-response.md` templates | ≤ 30 min |

## Block corruption (FM-05)

| Step | Action | Time |
|---|---|---|
| 1 | Identify affected blocks: `mimir_block_sha_mismatch_total` + block ID labels | ≤ 5 min |
| 2 | Quarantine: tag affected blocks with `oya:quarantined="true"`; Mimir querier skips them | ≤ 2 min |
| 3 | Restore from RF-3 secondary copy: `mimir-tools block-restore --block-id <id>` | ≤ 30 min |
| 4 | Validate restored block SHA matches expected | ≤ 2 min |
| 5 | Audit: did tampering signature appear? engage ops-security if pattern is suspicious | – |

## Ruler outage (FM-11)

| Step | Action | Time |
|---|---|---|
| 1 | Verify ruler pods: `kubectl -n observability get pods -l app=mimir-ruler` | ≤ 2 min |
| 2 | If pods running but expressions failing: review `cortex_ruler_evaluations_failed_total` per rule | ≤ 5 min |
| 3 | Rollback the offending rule file: git revert + Helm-apply | ≤ 10 min |
| 4 | Validate recording-rule output: `oya:current_verdict:by_microservice_env` is up-to-date | ≤ 5 min |
| 5 | If outage persists: gate fails-closed; CI lane reads stale; safe but operationally blocking | – |

## DR Failover invocation

When primary-region degraded and DR pair exists, see `multi-region.md` §"DR Failover" Steps 1–10. Total time budget: ≤ 35 min.

## Verification

After recovery:
- Mimir `mimir_distributor_request_duration_seconds{quantile="0.99"} < 200ms` for ≥ 30 min.
- No active alerts on Mimir self-SLI.
- Ingest queues drained.
- Recording rules current.
- Self-observability dashboard green: `https://grafana-<pack>.oyatie.dev/d/mimir-self/overview`.

## Post-incident updates

- Postmortem within 5 business days.
- If FM-02 (tenancy drift): determine how live-mutation was possible despite CI lanes; harden control plane access.
- If FM-04 (storage outage): assess provider reliability for the pack; consider multi-provider strategy.
- If FM-05 (block corruption) ≥ 2 incidents in 12 months: investigate hardware patterns; consider Reed-Solomon EC at storage tier.

## References

- `microservices/observability/failure-modes.md`.
- `microservices/observability/multi-region.md` §"DR Failover".
- `microservices/observability/capacity-model.md`.
- `microservices/observability/incident-response.md`.
- Grafana Mimir operations docs — `grafana.com/docs/mimir/latest/operations/`.
