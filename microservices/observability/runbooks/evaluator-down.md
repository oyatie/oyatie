---
doc_class: Runbook
title: SLO engine worker (evaluator) down
microservice: observability
severity: "Sev-2 (fail-closed safe default; Sev-1 if persistent > 1h)"
status: Accepted
owner_team: axis-observability
date: 2026-05-17
related_artifacts:
  - microservices/observability/failure-modes.md (FM-03)
  - microservices/observability/incident-response.md
  - microservices/observability/runbooks/held-promotion-recovery.md
doc_status: published
---

# Runbook: SLO engine worker (evaluator) down

## Trigger

`slo-engine-worker` pods are unhealthy; `oya_observability_internal_evaluator_alive == 0` for ≥ 2 min.

## Severity

- Worker HA failover completes within ≤ 5 min: Sev-3 (transient).
- Both replicas down OR persistent > 5 min: Sev-2.
- Persistent > 1h: Sev-1.

## Impact

Every µservice's promotion is **held** (fail-closed). This is intentional and correct per ADR-0130; no false eligible verdicts ship. Tenant impact: scheduled promotions delayed.

## Pre-checks

1. Verify pod state: `kubectl -n observability get pods -l app=slo-engine-worker`.
2. Check pod logs: `kubectl -n observability logs -l app=slo-engine-worker --tail=200`.
3. Verify Mimir reachability from worker pods: `kubectl exec <pod> -- curl -s https://mimir-internal.observability.svc:9090/ready`.
4. Verify OpenBao token renewal: worker logs show `openbao_token_renewed_total{}` rate > 0.

## Recovery Path A — Single-pod crashloop

| Step | Action |
|---|---|
| 1 | Identify cause from logs (typical: PromQL parse error on a newly-deployed rule; OpenBao token-renewal failure; Mimir read latency spike causing health-check timeout). |
| 2 | If recent deploy: roll back the offending change via ArgoCD or `kubectl rollout undo`. |
| 3 | If OpenBao: rotate worker service-account token; reschedule pod (`kubectl rollout restart`). |
| 4 | If Mimir-read latency: see `runbooks/mimir-outage.md`. |
| 5 | Verify recovery: `oya_observability_internal_evaluator_alive == 1` for ≥ 5 min. |

## Recovery Path B — Both replicas down (HA failure)

| Step | Action |
|---|---|
| 1 | Declare Sev-2; engage axis-observability on-call + secondary on-call. |
| 2 | All gates held — this is fail-closed correct behavior; promotions are NOT failing wrongly. |
| 3 | Diagnose root cause: shared dependency? (OpenBao? Mimir?) If yes, fix root cause first; worker will self-recover. |
| 4 | If root cause is worker code: emergency hotfix PR; deploy via Helm; emergency-merge sign-off by ExecSponsor + ops-security (logged in audit-chain). |
| 5 | If business-critical promotions blocked > 30 min: invoke manual override per `runbooks/held-promotion-recovery.md` Path E (2-person rule + audit). |

## Recovery Path C — Leader-election storm

| Step | Action |
|---|---|
| 1 | Symptom: replicas constantly handing off leadership; verdict emission paused. |
| 2 | Verify lease object: `kubectl get lease slo-engine-worker-leader -o yaml`. |
| 3 | If lease renewal racing: increase `leaseDurationSeconds` from default to 30s; re-deploy. |
| 4 | If Kubernetes etcd is the issue (rare): engage `cloud-k8s` µservice's on-call. |

## Verification

After recovery:
- `oya_observability_internal_evaluator_alive == 1` for ≥ 5 min across both replicas.
- Verdict-emission rate returns to baseline: `rate(oya_promotion_eligibility_verdict[5m])` > 0.
- Held promotions resume: tenant-facing dashboards show new eligibility verdicts.
- Self-SLO returns to green: `https://grafana-<pack>.oyatie.dev/d/slo-engine-self/overview`.

## Post-incident updates

- Postmortem within 5 business days.
- Action: harden the failure mode (e.g., add a circuit-breaker so the worker survives one downstream Mimir outage by emitting `verdict=held` with `reason=mimir_read_timeout`; that's still fail-closed but doesn't crashloop the worker).
- Action: verify HA replica count is ≥ 2 for production; consider 3 for hyperscaler-tier packs.

## References

- `microservices/observability/failure-modes.md` FM-03.
- `microservices/observability/incident-response.md`.
- `microservices/observability/runbooks/held-promotion-recovery.md`.
- Kubernetes leader-election docs — `kubernetes.io/docs/concepts/architecture/leases/`.
