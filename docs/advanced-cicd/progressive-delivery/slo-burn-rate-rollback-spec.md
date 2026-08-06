---
doc_class: Spec
shape: anchor
length_cap: 200
authority_tier: 1
status: Accepted
date: 2026-05-12
adrs_cited: [ADR-0053, ADR-0052, ADR-0054]
purpose: |
  Multi-window multi-burn-rate alerting (per Google SRE Workbook §5): fast-burn (5min×2hr) +
  slow-burn (1hr×6hr). Auto-rollback at fast-burn threshold; auto-pause at slow-burn threshold.
  Provider-agnostic burn-rate kernel.
planned_enforcement_ref:
  - cloud-ci-slo-coverage
  - oya-governance-rollback-evidence
related_adrs: [ADR-0040, ADR-0042, ADR-0037]
doc_status: published
---

# SLO Burn-Rate Rollback Specification

> **Status:** pending approval. **Owner:** `axis-foundry`. **Date:** 2026-05-12. **Source:** [Google SRE Workbook §5 — Alerting on SLOs](https://sre.google/workbook/alerting-on-slos/).

## 1. Burn-rate formula

```
burn_rate(window) = (errors_in_window / requests_in_window) / (1 - SLO_target)
```

For an SLO target T = 99.9%, the steady-state burn rate is 1.0. A 1-hour window that consumes **2% of the 30-day error budget** burns at 14.4× — the Sev-1 trigger.

## 2. Multi-window multi-burn-rate (the canonical pattern)

Per SRE Workbook, two windows are evaluated jointly to suppress false positives while catching fast incidents:

| Severity | Long window | Short window | Burn-rate threshold | Action |
|---|---|---|---|---|
| **Sev-1 (fast)** | 1 h | 5 min | **14.4×** (both windows must breach) | Auto-rollback within 60 s |
| **Sev-1 (urgent)** | 6 h | 30 min | **6.0×** (both windows must breach) | Auto-pause + page on-call |
| **Sev-2 (slow)** | 24 h | 2 h | **3.0×** (both windows must breach) | Auto-pause + page on-call |
| **Sev-3 (info)** | 3 d | 6 h | **1.0×** (long window only) | Ticket; review at next cadence |

**Both windows must breach** = AND-gate to suppress single-spike false positives. This is the workbook's prescription; we adopt as-is.

## 3. The kernel: `oya-platform-slo-burn-rate-kernel` (NEW)

Provider-agnostic burn-rate computation. Inputs: SLO definition (target, window, scope), metric-source adapter (Prometheus / VictoriaMetrics / Datadog / Honeycomb). Outputs: burn-rate per window, severity classification, rollback decision.

Adapter crates:

- `oya-platform-metric-source-adapter-prometheus` (NEW) — Prometheus 3.11+ PromQL.
- `oya-platform-metric-source-adapter-victoriametrics` (NEW).
- `oya-platform-metric-source-adapter-datadog` (NEW; for tenants that BYO observability).
- `oya-platform-metric-source-adapter-honeycomb` (NEW).
- `oya-platform-metric-source-adapter-otel-collector` (NEW) — for cases where OTel collector aggregates pre-source.

Provider-neutral. Swap adapters without changing the kernel or call sites. [Directive 4](../../plans/MASTERPLAN.md) compliant.

## 4. PromQL example (rendered by adapter)

```promql
# fast-burn (5 min vs 1 h)
(
  sum(rate(http_requests_total{job="oya-<axis>-<svc>",status=~"5.."}[5m]))
  /
  sum(rate(http_requests_total{job="oya-<axis>-<svc>"}[5m]))
) / (1 - 0.999) > 14.4
AND
(
  sum(rate(http_requests_total{job="oya-<axis>-<svc>",status=~"5.."}[1h]))
  /
  sum(rate(http_requests_total{job="oya-<axis>-<svc>"}[1h]))
) / (1 - 0.999) > 14.4
```

## 5. Rollout integration

Flagger/Argo Rollouts pull burn-rate via `slo-burn-rate-fast` and `slo-burn-rate-slow` AnalysisTemplate (canonical name; per [`canary-rail-spec.md`](canary-rail-spec.md)). On Sev-1 breach: rollout aborts, traffic shifts to previous revision, D14 emit fires.

## 6. Per-service SLO catalog

Per-service SLOs are owned by [`docs/SLO-CATALOG.md`](../../../docs/SLO-CATALOG.md). Default targets by API-stability tier ([ADR-0037](../../../docs/decisions/ADR-0709-general-live-apex.md)):

| Tier | SLO target | Window | Steady burn |
|---|---|---|---|
| Preview | none required | — | — |
| GA | 99.95% | 30 d | 1.0× |
| Stable | 99.99% (critical: audit, identity, KMS) | 30 d | 1.0× |

`cloud-ci-slo-coverage` (existing) is extended to require burn-rate alert wiring per GA+ service.

## 7. Per-cohort burn-rate (regulated cohorts)

For stable-cohort tenants ([`stable-cohort-spec.md`](stable-cohort-spec.md)), burn-rate is computed **per cohort**, not globally. A regulated tenant breaching on green while others are clean triggers per-tenant rollback (per [`blue-green-spec.md`](blue-green-spec.md) §5).

## 8. Hyperscaler equivalents

- Google SRE Workbook §5 (the source).
- AWS CloudWatch + Synthetic Canaries + CodeDeploy alarms (linear/canary triggers).
- Microsoft Azure Monitor SLO + Action Groups.
- Datadog SLO burn-rate alerts (commercial; supported via adapter).
- Honeycomb SLO burn-alerts (commercial; supported via adapter).

We adopt the Google formulation as the canonical math; provider adapters surface it identically.

## 9. Failure modes (designed-against)

- **No traffic during rollout** → division by zero. Kernel returns `Insufficient` and holds the stage (does not promote).
- **Metric source down** → kernel returns `Unknown`; rollout pauses + on-call paged. Never silent-pass.
- **Clock skew between windows** → kernel anchors both windows to the same `eval_ts`; adapter responsible for time alignment.

## 10. Compliance gates

- `cloud-ci-slo-coverage` (existing; extended to require burn-rate alerts on GA+ services).
- `oya-governance-rollback-evidence` (NEW; BLOCKER).

## 11. Lift target

`oyatie/docs/release/slo-burn-rate-rollback-spec.md` on approval.
