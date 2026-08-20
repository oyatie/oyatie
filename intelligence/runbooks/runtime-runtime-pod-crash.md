---
doc_class: Runbook
title: Runtime pod crashloop / sibling unreachable / capacity exhaustion / long invocation
microservice: foundry-runtime
severity: "Sev-2 (single AZ or pattern) / Sev-3 (single pod)"
status: Accepted
owner_team: ops-sre-reliability + axis-foundry-runtime
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-01, FM-08, FM-14, FM-15)
  - microservices/intelligence/incident-response.md
  - microservices/intelligence/capacity-model.md
doc_status: published
---

# Runbook: Runtime pod crashloop / sibling unreachable / capacity exhaustion / long invocation

## Trigger

ONE of:
- Runtime pod crashloop (FM-01): `kube_pod_status_phase{phase="Running",pod=~"oya-intelligence-runtime-.*"}` drops; `oya_foundry_runtime_pod_restarts_total` rises.
- Sibling unreachable (FM-08): per-sibling circuit-breaker opens; `oya_foundry_runtime_sibling_failures_total{sibling="..."} > threshold`.
- Capacity exhaustion (FM-14): rate-limit-exceeded climbs; HPA at ceiling.
- Long invocation (FM-15): invocation duration tail spikes.

## Severity

- Single pod crashloop: Sev-3.
- Pattern (3+ pods in 5min) or sibling outage: Sev-2.
- Cluster-wide capacity exhaustion: Sev-2.

## Pod crashloop (FM-01)

| Step | Action | Time |
|---|---|---|
| 1 | Verify pod state: `kubectl -n foundry-runtime get pods -l app=oya-intelligence-runtime-capability-executor-app` | ≤2min |
| 2 | Check pod logs: `kubectl -n foundry-runtime logs <pod> --tail=200 --previous` for crash cause | ≤5min |
| 3 | Verify HPA scaling: `kubectl -n foundry-runtime get hpa` | ≤2min |
| 4 | Cordon affected node if pattern; allow cross-AZ rebalance | ≤5min |
| 5 | If recent deploy: roll back via ArgoCD or `kubectl rollout undo deployment/<name>` | ≤10min |
| 6 | Verify recovery: pods Running ≥5min; no restarts | ≤5min |
| 7 | If persistent: declare Sev-2; consider DR failover per `multi-region.md` | – |

## Sibling unreachable (FM-08)

| Step | Action | Time |
|---|---|---|
| 1 | Identify failing sibling: `kubectl -n foundry-runtime logs <executor-pod> \| grep "sibling_failure"` | ≤2min |
| 2 | Verify circuit-breaker state: `oya_foundry_runtime_circuit_breaker_state{sibling="..."}` | ≤2min |
| 3 | Engage sibling on-call (foundry-providers / foundry-guardrails / foundry-evidence / foundry-supervisor) | ≤5min |
| 4 | Per-sibling fallback policy: |
|  | foundry-providers down → invocations fail `InvocationFailed{reason=provider_unreachable}`; tenant retries | – |
|  | foundry-guardrails down → fail-closed (refuse dispatch); tenant comms | – |
|  | foundry-evidence down → continue dispatch; queue evidence emissions (retry); audit-chain seal scheduled-for-distinct-tracked-work | – |
|  | foundry-supervisor down → cache stays usable; registry hot-reloads delayed (FM-04) | – |
| 5 | Verify recovery: circuit-breaker closes; sibling reachability restored | ≤15min |

## Capacity exhaustion (FM-14)

| Step | Action | Time |
|---|---|---|
| 1 | Verify rate-limit-exceeded source: `topk(5, sum by (tenant_id) (rate(oya_foundry_runtime_rate_limit_exceeded_total[5m])))` | ≤2min |
| 2 | Check HPA at ceiling: `kubectl -n foundry-runtime get hpa oya-intelligence-runtime-capability-executor-app` | ≤2min |
| 3 | If single tenant: engage tenant on capability-concurrency discipline; surface their per-capability dispatch dashboard | ≤30min |
| 4 | If cluster-wide: raise HPA ceiling within global budget (per `capacity-model.md`); scale pool warm pods | ≤15min |
| 5 | Verify recovery: rate-limit-exceeded rate returns to baseline | ≤15min |

## Long invocation (FM-15)

| Step | Action | Time |
|---|---|---|
| 1 | Identify offending capability: `topk(5, oya_foundry_runtime_invocation_duration_seconds{quantile="0.99"})` by `capability_id` | ≤2min |
| 2 | Verify TimeoutClock is enforcing: failing invocations emit `InvocationFailed{reason=timeout}` | ≤2min |
| 3 | Engage tenant operator: capability descriptor's timeout may be too generous OR the tool-call loop is runaway | ≤30min |
| 4 | Tighten descriptor timeout via foundry-supervisor PR | ≤1h |
| 5 | Verify recovery: long invocation tail returns to baseline | ≤15min |

## Verification

After recovery:
- `oya_foundry_runtime_dispatch_latency_seconds{quantile="0.99"} < 50ms` for ≥30 min.
- No active alerts on runtime self-SLI.
- HPA stable (not at ceiling).
- Self-observability dashboard green.

## Post-incident updates

- Postmortem within 5 business days.
- For FM-01: investigate panic cause; harden defensive code paths.
- For FM-08: assess sibling resilience; consider increasing circuit-breaker buffer.
- For FM-14: revisit `capacity-model.md` projections; adjust per-tenant defaults if pattern indicates baseline mismatch.
- For FM-15: descriptor authoring guidance updated.

## References

- `microservices/intelligence/failure-modes.md` FM-01, FM-08, FM-14, FM-15.
- `microservices/intelligence/multi-region.md` §"DR Failover".
- `microservices/intelligence/incident-response.md`.
- Kubernetes HPA docs — `kubernetes.io/docs/tasks/run-application/horizontal-pod-autoscale/`.
