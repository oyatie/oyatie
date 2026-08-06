---
id: ADR-0042
status: Superseded
doc_status: published
superseded_by: [ADR-0383]
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0042: Observability stack — OpenTelemetry SDK + VictoriaMetrics, in-house Leptos portal long-horizon, gen_ai semconv per capability

> **Status:** Superseded by [ADR-0383](ADR-0383-observability-stack-reconciliation-loki-tempo-mimir-grafana.md)
> **Supersedes:** -
> **Superseded-by:** ADR-0383
> **Owner:** `foundry`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0003, ADR-0007, ADR-0011, ADR-0028, ADR-0036, ADR-0037, ADR-0038, ADR-0040, ADR-0043

---

## Superseded by

> **This ADR is superseded by [ADR-0383](ADR-0383-observability-stack-reconciliation-loki-tempo-mimir-grafana.md)
> (Observability stack reconciliation: keep Loki / Tempo / Mimir / Grafana under AGPL-3),
> accepted 2026-05-28.**
>
> ADR-0042's prohibition on Loki / Tempo / Mimir / Grafana (AGPL-3) and its mandate for
> VictoriaMetrics + ClickHouse + Jaeger + in-house Leptos portal as the canonical storage tier are
> **retired**. The five-stage observability backplane defined in ADR-0186 (Grafana Labs LGTM stack,
> fully self-hosted in oya-cells) is the canonical architecture. See ADR-0383 for the full
> license-reconciliation record and the three gates (self-hosted, network-clause satisfied,
> ops-platform lifecycle ownership) that govern the AGPL-3 components.
>
> The OTel SDK instrumentation surface, per-cell namespace pattern, per-tenant cost-attribution
> dashboards, and gen_ai semantic conventions defined here remain valid and are carried forward by
> ADR-0186.

---

## Context

Observability is a substrate concern: every axis emits metrics, logs, and traces, and every operator needs to query across them with the same vocabulary. The pack-of-19 foundation ADRs decided observability is shared but did not pin (a) the SDK + protocol — OpenTelemetry is the only credible answer, but the storage tier choice has license implications, (b) the in-house UI long-horizon vs commercial Grafana licensing decision, (c) the per-capability gen_ai semantic-convention adoption, (d) the per-cell observability namespace pattern, (e) the per-tenant cost-attribution dashboard surface.

The license dimension is sharp: Grafana / Loki / Tempo / Mimir all flipped to AGPL-3 in 2024, which is forbidden in our product surface per License Policy ADR (without legal isolation). VictoriaMetrics is Apache-2 and is the credible Prometheus-compatible storage. We must commit to either an in-house portal (Leptos / Yew Rust-stack) or commercial Grafana licensing — the AGPL-3 path is closed.

---

## Decision

We adopt **OpenTelemetry SDK** as the canonical instrumentation surface; **VictoriaMetrics** (Apache-2) as the metrics storage; **structured JSON logs via the `tracing` crate** for log emission; **per-cell observability namespace** for isolation; **per-tenant cost-attribution dashboards** at the FinOps layer; **per-capability gen_ai semantic conventions** for AI/agent telemetry; an **in-house Leptos observability portal** long-horizon; **commercial Grafana Enterprise licensing** as the Phase-1 / Phase-2 fallback if in-house portal is not ready by GA.

### OpenTelemetry SDK (Apache-2)

```rust
// crates/oya-observability
pub struct ObservabilityKit {
    pub tracer: opentelemetry::global::BoxedTracer,
    pub meter: opentelemetry::metrics::Meter,
    pub logger: tracing::Subscriber,
    pub gen_ai_attrs: GenAiAttributesProvider,  // per ADR-0011 capability binding
}
```

- **Traces.** OTLP/gRPC export to per-cell collector; W3C Trace Context propagation.
- **Metrics.** OTLP/HTTP export; OpenMetrics-compatible.
- **Logs.** Structured JSON via `tracing` crate; OTLP/HTTP export.
- **Semantic conventions.** OpenTelemetry stable + experimental gen_ai conventions.

### VictoriaMetrics for metrics storage (Apache-2)

- Prometheus-compatible; horizontally scalable; per-cell deployment.
- Long-term retention via `vmstorage` clusters (per-tenant retention policy per ADR-0034 overrides).
- Per-tenant labels enforce tenant isolation in queries.
- Replaces Prometheus + Mimir + Cortex (license + scaling concerns).

### Structured JSON logs via `tracing` crate

```rust
// every service entry
let _guard = tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer().json())
    .with(tracing_opentelemetry::layer().with_tracer(tracer))
    .with(EnvFilter::from_default_env())
    .try_init()?;
```

- Per-event JSON line.
- Per-event tenant + cell + axis + capability tags.
- Per-event audit-chain ref where the event is regulator-relevant (per ADR-0003).

### Logs storage tier — license-clean alternatives to Loki

- **Day 1.** ClickHouse (Apache-2) for log storage, with `vector.dev` (MPL-2) as the agent.
- **Long horizon.** In-house log storage (Rust + columnar layout) under `crates/oya-observability-logs-*`.

(Loki is AGPL-3 since 2024 and is forbidden in our product surface per License Policy.)

### Traces storage tier — license-clean alternatives to Tempo

- **Day 1.** Jaeger (Apache-2; CNCF Graduated) backed by ClickHouse.
- **Long horizon.** In-house trace storage under `crates/oya-observability-traces-*`.

(Tempo is AGPL-3 since 2024.)

### In-house Leptos observability portal (long-horizon)

`crates/oya-observability-portal-*` is the in-house UI:

- Built in Leptos (Rust → WASM) per platform stack policy.
- Per-cell + per-tenant + per-microservice dashboards.
- Per-capability gen_ai dashboards.
- Per-SLO burn-rate panels (per ADR-0040 metric gating).
- Per-DSR cascade tracking surface (per ADR-0038).
- Trust portal embed (per ADR-0038).

Long-horizon target: GA at W+18 to W+24.

### Commercial Grafana Enterprise as Phase-1 / Phase-2 fallback

If the in-house portal is not GA-ready by W+12, we adopt commercial Grafana Enterprise licensing (per License Policy ADR; commercial license clears the AGPL-3 issue for the period the license is held). Migration path: Grafana → in-house portal at W+18+.

### Per-cell observability namespace

Each cell (per ADR-0028) gets:

- A dedicated VictoriaMetrics namespace.
- A dedicated ClickHouse logs database.
- A dedicated Jaeger trace store.
- A dedicated portal scope (per-cell dashboards).

Cross-cell aggregation requires explicit operator action + audit-chain emission.

### Per-tenant cost-attribution dashboards

Per-tenant FinOps surface (per ADR-0028):

- Per-tenant per-microservice cost breakdown.
- Per-tenant per-resource utilization.
- Per-tenant cost anomaly detection.
- Per-tenant unit-economics (cost per active user / per workflow / per invocation).

The dashboard is exposed in the trust portal (per ADR-0038) for tenant admin visibility.

### Per-capability gen_ai semantic conventions

Per OpenTelemetry gen_ai semantic conventions (W3C draft + OTel SIG):

```
gen_ai.system            = "oya-foundry" | "openai" | "anthropic" | "google" | "in-house"
gen_ai.request.model     = "claude-opus-4-7-1m" | "gpt-5" | "gemini-3-pro" | ...
gen_ai.response.model    = ...
gen_ai.usage.input_tokens
gen_ai.usage.output_tokens
gen_ai.request.temperature
gen_ai.request.max_tokens
gen_ai.response.finish_reasons
oya.foundry.capability   = <capability_id>          # ADR-0011 binding
oya.foundry.persona_tier = "ASSIST" | "COWORKER" | "PROXY" | "AUTONOMOUS"  # ADR-0007
oya.tenant.id            = <tenant_id>
oya.cell.id              = <cell_id>
```

Every Foundry capability invocation emits these attributes; the in-house portal renders per-capability cost + latency + safety dashboards.

### Per-axis SLO catalog

Per-axis SLO catalog under `docs/SLO-CATALOG.md` (existing) is the source of truth. Each SLO declares:

- Target (e.g. 99.95% availability, P95 < 100ms latency).
- Measurement window (typically 30d rolling).
- Burn-rate alert thresholds (per ADR-0040 mathematics).
- Owner.
- Per-tier mapping (preview / stable / GA per ADR-0037).

### Anti-scope

This ADR does not own audit-chain primitives (per ADR-0003). Does not own per-rollout mechanics (per ADR-0040). Does not own per-cell HSM (per ADR-0043). Does not own per-tenant trust portal authoring (per ADR-0038, but the portal embeds observability views).

---

## Consequences

### Positive

- OpenTelemetry SDK gives uniform instrumentation across all axes.
- License-clean storage tier (VictoriaMetrics + ClickHouse + Jaeger) avoids AGPL-3 contamination of our product surface.
- Per-cell namespace makes per-tenant data isolation mechanical.
- gen_ai semantic conventions per capability give us the only credible per-agent observability story; competitors will catch up but we are in front.
- In-house Leptos portal long-horizon eliminates commercial-license dependency.

### Negative

- AGPL-3 avoidance excludes the most-popular UI (Grafana) without commercial licensing; in-house portal build is real cost.
- Multiple storage tiers (metrics + logs + traces, each with its own scaling profile) increase operational surface.
- Per-cell namespace multiplication multiplies operational deployments.
- gen_ai semconv is moving fast; we'll re-baseline annually.

### Operational

- Per-cell collector health monitored; ingest backpressure alarmed.
- Per-storage-tier capacity headroom > 30%.
- Per-axis SLO dashboard reviewed weekly.
- Per-tenant cost-attribution accuracy reviewed monthly.
- Per-quarter portal-vs-Grafana feature-parity audit (if running fallback).
- gen_ai semconv schema review per OTel SIG cadence.

---

## Alternatives considered

### Alternative A — Grafana / Loki / Tempo / Mimir (AGPL-3)

- **Pros:** mature; community.
- **Cons:** AGPL-3 forbidden in product surface per License Policy.
- **Rejected because:** license posture incompatible.

### Alternative B — Datadog (commercial SaaS)

- **Pros:** turnkey.
- **Cons:** unbounded cost at scale; data leaves our cells; KR sovereignty concerns.
- **Rejected because:** sovereignty and unit economics.

### Alternative C — Skip in-house portal; commit to commercial Grafana Enterprise long-horizon

- **Pros:** less build.
- **Cons:** perpetual commercial dependency; per-seat pricing scales poorly.
- **Rejected because:** in-house long-horizon is consistent with Rust-first sovereignty stance.

### Alternative D — Per-axis observability stack

- **Pros:** microservice-team independence.
- **Cons:** N stacks; per-stack drift; cross-microservice tracing impossible.
- **Rejected because:** cross-microservice tracing is a primary value of OTel.

---

## Open questions

1. **Q1.** In-house portal GA target — W+18 or W+24? Default: W+24; W+18 is stretch. → owner: `foundry`.
2. **Q2.** Per-tenant retention default — 90d for metrics, 30d for logs, 14d for traces? Default: yes; per-vertical override per ADR-0034. → ADR-0034.
3. **Q3.** Long-term metric retention (>1y) — VictoriaMetrics or in-house long-term store? Default: VictoriaMetrics with `vmstorage` cluster; in-house at W+24+. → owner: `foundry`.
4. **Q4.** Per-cell vs per-region collector topology — collectors per cell or one regional collector? Default: per cell for isolation; regional aggregator for cross-cell. → ADR-0028.
5. **Q5.** OTel collector version pinning — track latest stable or LTS? Default: LTS. → owner: `foundry`.

---

## References

- `docs/PRD.md` §10 (observability)
- `docs/DESIGN.md` §11 (observability), §10 (cross-microservice contracts)
- `docs/SLO-CATALOG.md` (existing source of truth for SLOs)
- OpenTelemetry spec; OpenTelemetry gen_ai semantic conventions (SIG); W3C Trace Context
- VictoriaMetrics docs; ClickHouse docs; Jaeger CNCF docs
- Grafana / Loki / Tempo / Mimir license history (AGPL-3 transition)
- ADR-0001 (cohesion), ADR-0003 (audit), ADR-0007 (Cedar + persona tier), ADR-0011 (capability registry), ADR-0028 (cloud), ADR-0036 (plugins), ADR-0037 (API stability), ADR-0038 (trust portal), ADR-0040 (progressive delivery), ADR-0043 (HSM + KMS)
