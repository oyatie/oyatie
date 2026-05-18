---
doc_class: Runbook
title: Scheduler restart + cache rebuild
microservice: cell
severity: "Sev-2 (worker outage) / Sev-3 (cache poison)"
status: Accepted
owner_team: axis-cell-substrate + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/cell/failure-modes.md (FM-03 scheduler outage; FM-12 SPIRE outage; FM-14 cache poison)
doc_status: published
---

# Runbook: Scheduler Restart + Cache Rebuild

## Trigger

ONE of:

1. **Worker outage**: `oya_cell_scheduler_alive == 0` for ≥ 2 min.
2. **Placement queue depth**: `oya_cell_placement_queue_depth > 100`.
3. **Cache poison**: `cell_registry_cache_inconsistency_total > 0`.
4. **SPIRE outage**: `spire_server_attestation_success_rate < 0.99`.

## Severity

- Worker outage with HA failover successful: Sev-3.
- Worker outage with HA failover failing: Sev-2.
- Cache poison: Sev-2 (defence-in-depth holds; potential latency spike).
- SPIRE outage: Sev-2 (existing SVIDs valid ≤ 24h; new attestation fails).

## Pre-checks

1. Verify scheduler pod state: `kubectl -n cell-control-plane get pods -l app=scheduler-worker`.
2. Check leader-election lease: `kubectl -n cell-control-plane get leases scheduler-worker-leader`.
3. Check OpenBao token renewal: `oya cell scheduler-openbao-status`.
4. Check Postgres + SPIRE health.

## Recovery Path A — Worker Outage (FM-03)

| Step | Action | Time |
|---|---|---|
| 1 | Verify HA leader-election re-ran: lease holder should be a different pod within 30s of primary failure. | ≤ 30s |
| 2 | If standby took over: queue drains; new placement requests processed. | ≤ 2 min |
| 3 | If all replicas fail: inspect logs `kubectl logs ...` for crashloop cause (PromQL parse bug; OpenBao token renewal; binpack serialization). | ≤ 5 min |
| 4 | Fix cause + redeploy: scheduler is stateless; new pods rebuild cache from Postgres on startup. | ≤ 15 min |
| 5 | Verify queue drain: `oya_cell_placement_queue_depth → 0` within 5 min of recovery. | ≤ 5 min |

## Recovery Path B — Cache Poison (FM-14)

| Step | Action |
|---|---|
| 1 | Force cache flush: `oya cell scheduler-cache-flush --pack <pack>`. |
| 2 | Scheduler rebuilds cache from Postgres on next placement request (lazy rebuild). |
| 3 | Verify consistency: `cell_registry_cache_inconsistency_total` drops to 0 within 5 min. |
| 4 | If inconsistency persists: investigate Postgres state vs cache state; likely TTL bug; engage axis-cell-substrate for fix. |

## Recovery Path C — SPIRE Server Outage (FM-12)

| Step | Action |
|---|---|
| 1 | Declare Sev-2; engage ops-security + cloud-k8s. |
| 2 | Existing SVIDs valid ≤ 24h; existing workloads unaffected during TTL. |
| 3 | New attestation fails — new pods can't get SVID → can't connect to Postgres → fresh workload starts blocked. |
| 4 | SPIRE server replica failover; verify quorum. |
| 5 | Verify attestation success rate recovers to > 0.99 within 15 min. |
| 6 | If persists: SPIRE bundle integrity check; potential trust-bundle restore from OpenBao backup. |

## Recovery Path D — Placement Decision Quality Regression

Cause: Scheduler is making poor decisions (low binpack quality; over-packing some cells; under-packing others).

| Step | Action |
|---|---|
| 1 | Run replay: `oya cell scheduler-replay --pack <pack> --window 24h --compare-quality`. Outputs current vs ideal binpack score. |
| 2 | If regression detected: identify which scheduler version introduced; check release notes. |
| 3 | Rollback scheduler to prior known-good version via `oya vcs rollback --microservice cell --env production`. |
| 4 | Quarterly placement-quality audit: `oya cell scheduler-quality-report --pack <pack> --window 7d`. |

## Verification

After completion:
- `oya_cell_scheduler_alive == 1`.
- `oya_cell_placement_queue_depth == 0`.
- `cell_registry_cache_inconsistency_total == 0`.
- `spire_server_attestation_success_rate > 0.99`.
- Recent placement decisions audit-chain-sealed.

## Post-incident updates

- If worker crash recurring: root-cause + fix; consider chaos-testing the crash-loop path.
- If cache TTL bug recurring: revisit TTL strategy; consider event-driven invalidation.

## References

- `microservices/cell/failure-modes.md` FM-03, FM-12, FM-14.
- Kubernetes leader-election — `kubernetes.io/docs/concepts/architecture/leases/`.
- SPIRE recovery — `spiffe.io/docs/latest/spire-helper-charts/`.
