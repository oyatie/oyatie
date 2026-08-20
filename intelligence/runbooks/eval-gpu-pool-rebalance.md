---
doc_class: Runbook
title: GPU Pool Rebalance (eval-case dispatcher pool)
microservice: foundry-eval
severity: "Sev-3 (queue depth elevated) / Sev-2 (publish-gate stalled)"
status: Accepted
owner_team: ops-sre-reliability + axis-foundry
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-02 GPU pool exhaustion)
  - microservices/intelligence/threat-model.md (T-D-01)
  - microservices/intelligence/capacity-model.md
doc_status: published
---

# Runbook: GPU Pool Rebalance

## Trigger

ONE of:

1. `oya_foundry_eval_gpu_pool_queue_depth_seconds > 300` (5 min queue depth) for ≥ 5 min.
2. `oya_foundry_eval_publish_gate_latency_seconds{quantile="0.99"} > 5` for ≥ 5 min.
3. Cluster autoscaler reports GPU node provisioning failures.
4. Cost-budget alarm: GPU compute spend > 130% of forecast.

## Severity

- Queue depth elevated, publish-gate latency within SLO: **Sev-3**.
- Publish-gate latency p99 > 5s: **Sev-2**.
- Publish-gate effectively unavailable (latency p50 > 10s): **Sev-1**.

## Pre-checks

1. Confirm GPU pool current size: `kubectl get nodes -l workload=foundry-eval-gpu -o wide` lists GPU-eligible nodes.
2. Confirm HPA scaling state: `kubectl describe hpa -n foundry-eval eval-runner-worker`.
3. Confirm GPU node availability quota with cloud provider.

## Steps

| Step | Action | Time budget |
|---|---|---|
| 1 | Open `#inc-<id>`; assign IC; declare severity | ≤ 5 min |
| 2 | Pre-checks above | ≤ 5 min |
| 3 | Manual scale-out: `kubectl scale -n foundry-eval deploy/eval-runner-worker --replicas <N>` (N up to HPA max) | ≤ 5 min |
| 4 | Cluster-autoscaler manual nudge: add GPU node-group node count if autoscaler not provisioning | ≤ 15 min |
| 5 | Priority-class triage: ensure publish-gate runs (priority class `foundry-eval-publish`) preempt nightly runs (priority class `foundry-eval-nightly`) | ≤ 5 min |
| 6 | If queue depth persists: pause nightly cadence temporarily (`oya admin foundry-eval pause-nightly`); resume after queue drains | per scenario |
| 7 | Investigate root cause: (a) traffic spike (capability author flooded ad-hoc runs)? (b) provider rate-limit causing slow case completion? (c) GPU pool quota hit? (d) gVisor sandbox overhead spike? | ≤ 1 h |
| 8 | For (a): apply per-capability-owner rate limit (10/h default; tighten if needed per `threat-model.md` T-D-01) | per cause |
| 9 | For (b): negotiate provider rate-limit increase OR rotate to second provider via foundry-providers router-preference | per cause |
| 10 | For (c): escalate to cloud-account team for quota increase | per cause |
| 11 | For (d): roll back to prior sandbox image | per cause |
| 12 | Resume nightly cadence; verify queue depth returns to baseline (≤ 60s) | ≤ 30 min |
| 13 | Postmortem within 5 business days for Sev-1 / Sev-2 | — |

## Cost-driven Rebalance

When triggered by cost-budget alarm:
1. Review which capabilities consumed disproportionate GPU minutes.
2. Engage owners: can eval-set size be reduced (case-deduplication)? can cohort sub-sample?
3. Consider spot-instance fleet for non-critical (nightly) workloads.
4. Update `capacity-model.md` if forecast revised.

## Verification

After completion:
- `oya_foundry_eval_gpu_pool_queue_depth_seconds <= 60` p99 sustained ≥ 1 h.
- Publish-gate latency p99 ≤ 1 s.
- GPU compute spend within forecast 110% band.

## References

- `microservices/intelligence/failure-modes.md` FM-02.
- `microservices/intelligence/threat-model.md` T-D-01.
- `microservices/intelligence/capacity-model.md`.
- `microservices/intelligence/cost-budget.md`.
