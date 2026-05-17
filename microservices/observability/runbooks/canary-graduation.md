---
doc_class: Runbook
title: Canary cohort graduation
microservice: observability
severity: "Sev-3 (degraded) / Sev-2 (stuck > 1h)"
status: Accepted
owner_team: ops-sre-reliability + axis-observability
date: 2026-05-17
related_artifacts:
  - microservices/observability/failure-modes.md (FM-12 mesh weight stuck)
  - /specs/agentic-slo-gated-promotion.json §"canary_cohort_weighting"
doc_status: published
---

# Runbook: Canary cohort graduation

## Trigger

A µservice's canary cohort is stuck at an intermediate weight (1% / 10% / 50%) and is not progressing to 100% within the expected window, OR has been aborted due to a burn-rate breach.

## Severity

- Stuck < 1h: Sev-3.
- Stuck > 1h OR aborted by burn-rate: Sev-2 (signal degraded; gate decisions less reliable).

## Pre-checks

1. Identify the µservice + the current canary weight: read `service_mesh_traffic_split_weight{microservice="<ms>", cohort="canary"}` from Mimir.
2. Identify the time at this weight: `service_mesh_traffic_split_weight_changed_at`.
3. Check expected progression: per `/specs/agentic-slo-gated-promotion.json` §"canary_cohort_weighting" (1→10→50→100 % with min-duration + exit-criterion).
4. Check for active burn-rate alerts on the µservice — if active, an abort is the correct behavior.

## Recovery Path A — VirtualService update failure (FM-12)

Cause: Istio VirtualService update propagation failed; weighted routing stale.

| Step | Action |
|---|---|
| 1 | Verify Istio control-plane health: `kubectl -n istio-system get pods` + `istioctl proxy-status`. |
| 2 | If control-plane unhealthy: engage `runbooks/mimir-outage.md` (cross-pollination: Istio control-plane outages are observability-adjacent). |
| 3 | If control-plane healthy: manually re-apply VirtualService — `kubectl -n <ms> apply -f <virtualservice.yaml>` from the µservice's IaC source. |
| 4 | Verify proxy-config push: `istioctl proxy-config route <pod> | grep <ms>` reflects expected weights. |
| 5 | Verify weight applied in Mimir: `service_mesh_traffic_split_weight{microservice="<ms>", cohort="canary"}` matches expected step weight. |

## Recovery Path B — Abort due to burn-rate breach

Cause: A burn-rate alert fired during a canary step; the controller aborted and drained canary back to 0%.

| Step | Action |
|---|---|
| 1 | Verify the abort completed: weight=0% for canary cohort. |
| 2 | Investigate the burn-rate breach via dashboard. |
| 3 | If genuine regression: do not retry canary; let the gate hold; fix underlying issue per `held-promotion-recovery.md` Path B. |
| 4 | If transient: wait for burn-rate to clear; re-initiate canary at 1%; close monitoring. |
| 5 | If retries persistently fail: declare Sev-2; engage axis-observability; consider tighter SLI definition OR step-size adjustment. |

## Recovery Path C — Canary signal insufficient at current weight

Cause: 1% weight does not generate enough samples for the burn-rate window to compute reliable verdict (small traffic volume).

| Step | Action |
|---|---|
| 1 | Verify sample count per the µservice's SLI: `count_over_time(<sli>[1h])` — if < 100 samples in window, signal is too sparse. |
| 2 | Decision: increase the next step's weight earlier (skip 10% → go directly to 50%); document deviation in `evidence/multispectrum/<change_id>.json`. |
| 3 | Alternative: extend the at-1% min-duration to accumulate more samples. |
| 4 | Long-term: tune per-µservice canary-step-policy in `/specs/agentic-slo-gated-promotion.json` to match µservice traffic profile. |

## Verification

After completion:
- `service_mesh_traffic_split_weight{microservice="<ms>", cohort="canary"}` matches expected step.
- Burn-rate within target.
- Promotion proceeds to next gate-tick.

## Post-incident updates

- If recurring: surface to capacity-model.md (small-tenant traffic profile may need different canary policy).
- Update `/specs/agentic-slo-gated-promotion.json` if the per-µservice canary-step-policy needs adjustment.

## References

- `microservices/observability/failure-modes.md` FM-12.
- `/specs/agentic-slo-gated-promotion.json` §"canary_cohort_weighting".
- Istio VirtualService docs — `istio.io/latest/docs/reference/config/networking/virtual-service/`.
- Argo Rollouts (for reference; oyatie uses Istio-native today) — `argoproj.github.io/rollouts/`.
