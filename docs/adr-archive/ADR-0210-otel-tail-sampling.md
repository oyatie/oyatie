---
id: ADR-0210
status: Superseded
deciders: council-architecture, ops-sre-reliability, axis-observability, axis-finops
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-0706]
related: [ADR-0139, ADR-0145, ADR-0153, ADR-0174, ADR-0186]
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/microservices/manifest-schema.json
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0210 — OpenTelemetry tail-sampling: 100% error / p99 / new-endpoint traces + 1% baseline

## Status

Accepted (2026-05-18). Pins the OTel tail-sampling policy: head-sample at 1% baseline; tail-sample 100% of errors + 100% of slow traces + 100% of new-endpoint traces during the warm-up window.

## Context

ADR-0186 fixed the LGTM observability backplane with OpenTelemetry Collector at Stage 1 (collection) and Grafana Tempo at Stage 2 (trace storage). Trace storage cost scales with retention + sampling rate; always-on 100% sampling is unaffordable at fleet scale (32 µservices × 7-day hot retention × full request volume).

The classic head-sampling at 1% leaves us blind to:

- **Errors** — a 1% sample of error traces means most errors lack traces.
- **Slow traces** — p99 latency outliers are the most useful for debugging; head sampling discards them by definition.
- **New endpoints** — newly-deployed endpoints have no operator intuition; full trace data during a warm-up window is invaluable.

Hyperscaler practice (Google Dapper / Honeycomb / Lightstep) is **head-sample low + tail-sample on the interesting tail**. Tail sampling buffers the trace until the root span closes, then evaluates a sampling policy with full trace context.

## Decision

### Two-stage sampling

**Stage A — Head sampling (per agent collector, DaemonSet):**

- Default: **1% always-on baseline** at the per-µservice agent collector (per ADR-0186 Stage 1).
- Configurable per µservice via `manifest.json` `observability.trace_sampling_recipe.head_bps` (basis points; default 100 = 1%).
- Random sampling decision at the entry point (root span); the decision propagates as a trace flag.

**Stage B — Tail sampling (gateway collector, Deployment):**

OpenTelemetry Collector Tail Sampling Processor evaluates each completed trace against a closed policy set:

| Policy | Match | Sample rate |
|---|---|---|
| `status_code=ERROR` | any span has error status | 100% |
| `latency_p99` | trace duration > µservice p99 SLO threshold | 100% |
| `new_endpoint_warmup` | route_template ∈ µservice's `manifest.json` `observability.trace_sampling_recipe.new_endpoints` (TTL 30 days) | 100% |
| `random_baseline` | default | 1% |
| `slo_burn` | trace touches a span attributed to an active SLO burn window | 100% |
| `audit_event` | trace emits an `EVT-*` audit event (per ADR-0145) | 100% |

Policy evaluation order: the first matching policy wins; `random_baseline` is the fallback.

### Buffer + decision wait

Tail sampling needs to buffer until the root span closes. The Collector's Tail Sampling Processor uses a `decision_wait` of **30 seconds** (covers > 99.5% of traces; the long tail is sacrificed). Memory budget: **256 MiB** per gateway Collector replica; horizontal scale on memory pressure.

### Cost model

Always-on 100% trace volume → ~2.5 TB/day for a 32-µservice fleet at moderate scale. Tail sampling reduces this to ~250 GB/day (~10×) while preserving 100% of errors + slow + new-endpoint traces — the traces operators actually use.

### High-traffic-µservice escape hatch

At sustained > 5,000 req/sec for a single µservice, even 1% head sampling + 100% error tail sampling can saturate Tempo ingestion. Per-µservice manifest tunes:

- **`head_bps: 10` (0.1%)** when sustained > 5,000 req/sec — relies on tail policies for non-error traces.
- **`head_bps: 1` (0.01%)** when sustained > 50,000 req/sec.
- Tail-policy 100% of errors / p99 / new-endpoint / SLO-burn / audit-event preserved at every tier.

This keeps Tempo storage cost roughly linear in error rate × workload, not in raw request count.

### Per-µservice manifest field

```json
{
  "observability": {
    "trace_sampling_recipe": {
      "head_bps": 100,                 // 1% head sample baseline
      "tail_policies": ["status_code=ERROR", "latency_p99", "new_endpoint_warmup", "random_baseline=0.01"],
      "p99_latency_threshold_ms": 500, // µservice-specific
      "new_endpoints": [               // 30-day warm-up window per endpoint
        {"route": "/api/v1/run-step", "added_at": "2026-05-15"}
      ]
    }
  }
}
```

The manifest schema field is parent-wired; this ADR declares the shape.

### Helm chart

`microservices/observability/iac/helm/otel-tailsampling-collector/` deploys the gateway Collector tier with Tail Sampling Processor enabled.

## Alternatives considered

### (a) Always-on 100% sampling — REJECTED

- **Pros:** never lose a trace.
- **Cons:** ~2.5 TB/day; Tempo hot retention cost balloons; query latency degrades.
- **Rejected**: unaffordable at fleet scale.

### (b) Pure head sampling at 1% — REJECTED

- **Pros:** lowest cost.
- **Cons:** loses 99% of error + slow traces — the traces operators need.
- **Rejected**: blind to the long tail.

### (c) Dynamic head sampling (rate-adjusted by error rate) — REJECTED

- **Pros:** captures more during incidents.
- **Cons:** head-sampling decision is made before the trace completes; cannot know the trace will error.
- **Rejected**: doesn't solve the right problem.

### (d) Commercial APM (Datadog, New Relic) intelligent sampling — REJECTED

- **Pros:** turn-key.
- **Cons:** vendor lock-in per ADR-0173; SaaS data egress conflicts with sovereignty (ADR-0164).
- **Rejected**: lock-in + sovereignty.

### (e) **CHOSEN: head 1% + tail 100% on errors / p99 / new-endpoints**

- **Pros:**
  - Industry-standard Dapper-style policy.
  - OpenTelemetry Collector Tail Sampling Processor is the canonical impl.
  - ~10× cost reduction vs always-on 100%.
  - Preserves the traces operators actually use.
- **Cons:** Collector buffer memory budget required. Mitigation: 256 MiB per gateway replica; HPA on memory pressure.
- **Accepted**.

## Consequences

### Positive

1. **~10× cost reduction** vs always-on 100% trace volume.
2. **100% of errors + slow + new-endpoint traces preserved.**
3. **Per-µservice tunable** via manifest field.
4. **OpenTelemetry Collector** is the canonical processor; no new component.

### Negative

1. **30-second `decision_wait` buffer** means traces longer than 30s may be lost. Mitigation: span attribute marks long-running traces; explicit policy retains.
2. **Memory budget per Collector replica** (256 MiB). Mitigation: HPA on memory.
3. **New-endpoint warm-up TTL drift.** Mitigation: 30-day TTL auto-expires; manifest reviewed quarterly.

### Operational

- Helm chart at `microservices/observability/iac/helm/otel-tailsampling-collector/`.
- Per-µservice manifest field declared in ADR.
- Standards doc at `docs/standards/trace-sampling-tier.md`.

## In-house roadmap

**Vendor classification:** OpenTelemetry Collector + Tail Sampling Processor are **community standards** (CNCF Graduated; opentelemetry-collector-contrib).

- **No in-house tail-sampler rebuild planned.** The community processor is mature + the de-facto standard.
- **What we DO build in-house:**
  - Per-µservice sampling-recipe schema (manifest field).
  - Helm chart deployment (`otel-tailsampling-collector/`).
  - Tail-sample fidelity test (regression test ensuring error traces survive sampling).
  - Per-µservice p99 threshold authoring (tied to OpenSLO source per ADR-0186 Stage 5).

## Rollback

- Per-policy rollback: drop the policy from `tail_policies` array; redeploy Collector Helm release.
- Full tail-sampling rollback: drop the Tail Sampling Processor entirely; head sampling at 1% continues. Cost reverts to ~2.5 TB/day until rollback resolves.
- Per-µservice opt-out: set `head_bps=0` in manifest (effectively no tracing); should require ADR exception.

## References

- OpenTelemetry Collector — https://opentelemetry.io/docs/collector/ ; CNCF Graduated; Apache 2.0.
- Tail Sampling Processor — https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/processor/tailsamplingprocessor
- Dapper paper (Google) — https://research.google/pubs/pub36356/
- Honeycomb sampling guide — https://www.honeycomb.io/blog/dynamic-sampling-by-example
- ADR-0139 — 4-window burn-rate SLO alerting (slo_burn policy).
- ADR-0145 — inter-microservice communication reform (audit event = `EVT-*`).
- ADR-0153 — observability backplane high-level reference.
- ADR-0174 — finops cost attribution (sampling cost surfaces here).
- ADR-0186 — observability backplane layering (this ADR extends Stage 1 + Stage 2).
- LTS-rotation cadence: Collector version current as of 2026-05-18; review per ADR-0098.
