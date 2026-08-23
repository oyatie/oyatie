---
doc_class: Spec
shape: anchor
length_cap: 200
authority_tier: 1
status: Accepted
date: 2026-05-12
purpose: |
  Flagger as the K8s-native canary controller; Argo Rollouts as the second sanctioned option.
  Metric-gated promotion at 1% → 5% → 25% → 50% → 100% with SLO-burn-rate-bounded hold durations.
planned_enforcement_ref:
  - governance-canary-required
  - pipeline-slo-coverage
related_adrs: [ADR-0040, ADR-0042, ADR-0044, ADR-0053, ADR-0055]
adr_citations: [ADR-0053, ADR-0055]
doc_status: published
---

# Canary Rail Specification


## 1. Controllers

**Primary:** [Flagger](https://flagger.app/) — CNCF, service-mesh-native, lightweight. Aligns with Istio Ambient ([ADR-0044](../../decisions/ADR-0044-service-mesh-istio-ambient-and-envoy-gateway.md)).

**Secondary:** [Argo Rollouts](https://argoproj.github.io/argo-rollouts/) — CNCF Graduated, richer analysis-templates, used where cross-axis lockstep promotion or BG primitives are needed.

Both adapted behind `platform-rollout-controller-kernel` (NEW) + adapter crates.

## 2. Stage progression (default)

| Stage | Weight | Wall-clock floor | Gate (per [`slo-burn-rate-rollback-spec.md`](slo-burn-rate-rollback-spec.md)) |
|---|---|---|---|
| 1 | 1% | 5 min | fast-burn (5min×2hr) ≤ threshold; ≥ 200 sampled requests |
| 2 | 5% | 10 min | fast-burn ≤ threshold; per-tenant error-rate stable |
| 3 | 25% | 30 min | fast-burn + slow-burn (1hr×6hr) both ≤ threshold |
| 4 | 50% | 1 h | fast-burn + slow-burn ≤ threshold; latency P95 within 10% baseline |
| 5 | 100% | 24 h soak | slow-burn ≤ threshold sustained |

**Hold = max(wall-clock floor, burn-rate sample sufficiency)**. The clock is the floor; the data is the gate. A noisy stage holds longer; a clean stage cannot promote faster than the floor.

Per [ADR-0055](../../decisions/ADR-0055-four-layer-branch-pipeline.md) §5, canary reaching 100% for ≥ M hours is gate 3 of the staging → prod 5-gate verification.

## 3. Flagger Canary manifest (canonical-form)

```yaml
apiVersion: flagger.app/v1beta1
kind: Canary
metadata:
  name: oyatie-<axis>-<service>
spec:
  provider: istio
  targetRef: { apiVersion: apps/v1, kind: Deployment, name: oyatie-<axis>-<service> }
  analysis:
    interval: 1m
    threshold: 5            # ≤ 5 failed checks before rollback
    maxWeight: 100
    stepWeights: [1, 5, 25, 50, 100]
    metrics:
      - name: slo-fast-burn
        templateRef: { name: slo-burn-rate-fast }
        thresholdRange: { max: 14.4 }
        interval: 1m
      - name: slo-slow-burn
        templateRef: { name: slo-burn-rate-slow }
        thresholdRange: { max: 6.0 }
        interval: 5m
    webhooks:
      - name: cohort-honor-check
        url: http://platform-tenant-cohort-kernel:8080/canary/intersect
      - name: rollback-evidence-emit
        url: http://intelligence-evidence-kernel:8080/d14/emit
```

## 4. Argo Rollouts Rollout (when needed)

Used when: cross-axis lockstep promotion required; BG primitives needed; per-cell experiment graph required. See [ADR-0040](../../decisions/ADR-0040-progressive-delivery-canary-blue-green-metric-gated-rollback.md) for canonical manifest; we extend with the 1% stage at the head.

## 5. Per-cell scope

Each Canary/Rollout is scoped to one cell. Cross-cell promotion is orchestrated by `platform-rollout-controller-kernel`, which honours per-region phasing (primary cell → primary region secondaries → secondary region → global) per ADR-0040.

## 6. Analysis sources

- Prometheus 3.11+ (current mainline) — primary metric store.
- VictoriaMetrics — long-retention fallback (per [ADR-0042](../../decisions/ADR-0042-observability-stack-otel-and-in-house-ui.md)).
- Datadog / Honeycomb — accessed via `platform-metric-source-adapter-<provider>` if a tenant brings their own.

Provider-neutral query is `platform-slo-burn-rate-kernel` (NEW; see [`slo-burn-rate-rollback-spec.md`](slo-burn-rate-rollback-spec.md)).

## 7. Auto-rollback

On threshold breach: Flagger fires `traffic-shift: 0%` to canary; Argo Rollouts fires `Abort`. Both emit a D14 rollback record via webhook. Per-cell on-call paged within 60 s (Sev-1 path) or 5 min (Sev-2 path) per `governance-rollback-evidence`.

## 8. Pre-rollout gates (inherits from ADR-0040)

- supply-chain PASS ([ADR-0039](../../decisions/ADR-0039-supply-chain-security-trivy-cosign-sbom-signed-commits.md)).
- api-semver PASS ([ADR-0037](../../decisions/ADR-0037-public-api-stability-tiers-and-deprecation.md)).
- SLO catalog freshness within 1 h.
- canary-required lane PASS.

## 9. Hyperscaler equivalents

AWS CodeDeploy "Canary 10/45" + "Linear 10/3min"; Microsoft Azure Deployment Rings + Container Apps revisions; Oracle OCI Traffic Steering canary policies. Flagger/Argo are the open equivalents we use.

## 10. Compliance gates

- `governance-canary-required` (NEW; BLOCKER for kernel/domain/app/api/adapter classes).
- `pipeline-slo-coverage` (existing; extended).

## 11. ADR citations

- [ADR-0055](../../decisions/ADR-0055-four-layer-branch-pipeline.md) — canary 100% ≥ M hours is gate 3 of the staging → prod 5-gate; `EVT-CANARY-COMPLETE` triggers `prod-promoter`.
