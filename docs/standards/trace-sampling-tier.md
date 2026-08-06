---
doc_class: Standard
shape: standard
length_cap: 250
authority_tier: 2
status: Accepted
date: 2026-05-18
purpose: |
  Canonical OTel head + tail sampling policy. 1% baseline head sampling at agent collectors;
  100% tail sampling on errors / p99 / new-endpoint warm-up; per-µservice manifest-tunable.
canonical_authority: docs/decisions/ADR-0706-observability-live-apex.md
related_adrs:
  - ADR-0139
  - ADR-0145
  - ADR-0186
  - ADR-0210
enforced_by: advisory (oya-governance-otel-emit lane not yet shipped; proposed)
---

# Trace Sampling Tier Standard

## Authority

This standard implements ADR-0210, extending ADR-0186 Stages 1 + 2.

## Two-stage sampling

### Stage A — Head sampling (agent collector, DaemonSet)

- **Default: 1% (100 bps) always-on baseline.**
- Per-µservice manifest override via `observability.trace_sampling_recipe.head_bps`.
- Decision at root span; propagated as trace flag.

### Stage B — Tail sampling (gateway collector, Deployment)

OTel Collector Tail Sampling Processor; closed policy set:

| Policy | Match | Sample rate |
|---|---|---|
| `status_code=ERROR` | any span has error status | 100% |
| `latency_p99` | trace duration > µservice p99 SLO threshold | 100% |
| `new_endpoint_warmup` | route ∈ µservice's new_endpoints (TTL 30 days) | 100% |
| `slo_burn` | trace touches active SLO burn window | 100% |
| `audit_event` | trace emits `EVT-*` audit event | 100% |
| `random_baseline` | fallback | 1% |

First match wins; `random_baseline` is fallback.

## Buffer + decision wait

- **`decision_wait`: 30 seconds** (covers > 99.5% of traces; long-running traces sacrificed).
- **Memory budget: 256 MiB per gateway Collector replica.**
- HPA on memory pressure.

## Cost model

Always-on 100%: ~2.5 TB/day (32-µservice fleet, moderate scale).
With tail sampling: ~250 GB/day (~10× reduction); preserves 100% of useful traces.

## Per-µservice manifest shape

```json
{
  "observability": {
    "trace_sampling_recipe": {
      "head_bps": 100,
      "tail_policies": ["status_code=ERROR", "latency_p99", "new_endpoint_warmup", "random_baseline=0.01"],
      "p99_latency_threshold_ms": 500,
      "new_endpoints": [
        {"route": "/api/v1/run-step", "added_at": "2026-05-15"}
      ]
    }
  }
}
```

## Helm chart

`microservices/observability/iac/helm/otel-tailsampling-collector/` — Tail Sampling Processor
Collector tier (gateway role; downstream of per-node agent collectors).

## Anti-patterns

1. Always-on 100% — unaffordable + Tempo retention cost balloons.
2. Pure head sampling — loses 99% of error + slow traces.
3. Dynamic head sampling — head decision can't know if the trace will error.

## Cross-references

- ADR-0210 — tail-sampling policy (this standard's authority).
- ADR-0139 — 4-window burn-rate SLO alerting (slo_burn policy).
- ADR-0186 — observability backplane layering.
- OTel Collector Tail Sampling Processor — https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/processor/tailsamplingprocessor
