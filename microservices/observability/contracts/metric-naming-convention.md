---
contract: metric-naming-convention
authored: 2026-05-18
canonical_authority: ADR-0064
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
related_adrs:
  - ADR-0064
  - ADR-0128
  - ADR-0139
  - ADR-0131
  - ADR-0133
overlay_consumers:
  - microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yaml
  - docs/standards/hyperscaler-invariant-conformance.md
status: canonical-base
---

# Metric naming convention — canonical-base for hyperscaler-invariant alerts

This document is the **canonical base contract** every oyatie microservice
MUST honor for the metrics that the canonical
`hyperscaler-invariants-canonical-prometheusrule.yaml` queries against.
Per ADR-0064 canonical-base-and-localization-packs, this is the seam: the
canonical PrometheusRule supplies the implementation; each microservice
supplies pure value variation (its own `<ms>` substring) by exposing
metrics under the names defined here.

## Why this contract exists

Before SWEEP-H, twenty microservices each re-declared four near-identical
PrometheusRule groups covering the four hyperscaler invariants. Roughly
2,400 lines of duplicated content existed only because the microservice
name varied across otherwise identical PromQL. Per ADR-0064 the cleanest
form is seam (this contract) + impl (one canonical PrometheusRule). A
single per-microservice surface (the metric names below) replaces the
twenty per-microservice prometheusrule INV groups.

## The metric naming surface

Every microservice MUST emit the metrics listed below. The
`<ms>` placeholder is the microservice slug (matches the regex
`^[a-z][a-z0-9-]*$`, same as `manifest.json#microservice`).

### INV-CIRCUIT-BREAKER-BULKHEAD

| Metric name (templated)                                        | Type      | Required labels                  |
|----------------------------------------------------------------|-----------|----------------------------------|
| `oya_<ms>_capability_circuit_state`                            | gauge     | `microservice`, `capability_id`, `state` (`closed`/`half_open`/`open`) |
| `oya_<ms>_capability_retry_budget_exhausted_total`             | counter   | `microservice`, `capability_id`  |

The canonical PrometheusRule alerts `OyaCapabilityCircuitOpen` and
`OyaCapabilityRetryBudgetExhausted` query these two metric families via
`{__name__=~"oya_.+_capability_circuit_state"}` and
`{__name__=~"oya_.+_capability_retry_budget_exhausted_total"}`.

### INV-SHUFFLE-SHARDING

| Metric name (templated)                                        | Type      | Required labels                  |
|----------------------------------------------------------------|-----------|----------------------------------|
| `oya_<ms>_responses_429_total`                                 | counter   | `microservice`, `tenant_id`      |

The canonical alert `OyaTenantRateLimit429Surge` aggregates by
`(microservice, tenant_id)` and fires on > 50/s sustained 5 minutes.

### INV-FOUR-REFERENCE-SIGNALS

| Metric name (templated)                                        | Type      | Required labels                  |
|----------------------------------------------------------------|-----------|----------------------------------|
| `oya_<ms>_responses_5xx_total`                                 | counter   | `microservice`                   |
| `oya_<ms>_responses_total`                                     | counter   | `microservice`                   |

Plus the Kubernetes standard `container_cpu_usage_seconds_total` joined
against `kube_pod_labels{label_app_kubernetes_io_name=<ms>}` so the
saturation alert can resolve a `microservice` label without per-µservice
config. Every microservice pod MUST carry the
`app.kubernetes.io/name=<ms>` Kubernetes label (already enforced by Helm
chart `<ms>.labels` includes).

### INV-SLO-ERROR-BUDGET

| Metric name (templated)                                        | Type      | Required labels                  |
|----------------------------------------------------------------|-----------|----------------------------------|
| `oya_<ms>_request_success_total`                               | counter   | `microservice`                   |
| `oya_<ms>_request_total`                                       | counter   | `microservice`                   |

Per-SLO burn-rate alerts remain in each microservice's own
prometheusrule (they reference per-SLI metric names that vary by
microservice). The canonical alerts `OyaErrorBudgetFastBurn1h14x` and
`OyaErrorBudgetSlowBurn6h6x` query the aggregate success/total counters
above so the conformance gate can audit by `inv:INV-SLO-ERROR-BUDGET`
label query.

## The `microservice` label

Every metric in this contract MUST carry the `microservice=<ms>` label.
This is what allows the canonical PrometheusRule to surface the offending
microservice in alert annotations via `{{ $labels.microservice }}` without
hard-coding the microservice slug into the rule template.

The Helm chart `_helpers.tpl` standard pattern is:

```yaml
{{- define "oya-<ms>.labels" -}}
app.kubernetes.io/name: {{ include "oya-<ms>.name" . }}
microservice: {{ include "oya-<ms>.name" . }}
{{- end -}}
```

…and any OpenTelemetry / Prometheus client code in the data plane must
emit the same label on every counter/gauge/histogram listed above.

## Implementation surface (per microservice)

A microservice ships **one** thing to satisfy the canonical PrometheusRule:
the metric emission described above. No per-microservice PrometheusRule
re-declaration of these four INV groups is required (or permitted). The
microservice's own `prometheusrule.yaml` is reserved for per-SLO
burn-rate alerts and per-microservice operational alerts
(e.g. `NotesE2ELeakageDetected`, `MeetLiveStreamEgressUnauthorized`)
that do not generalize across microservices.

### Canonical Rust implementation (added 2026-05-18 per PERF-143-002)

The canonical Rust surface every microservice integrates against is the
**shared metric emission trait kernel** + **Prometheus reference adapter**:

- `crates/oya-shared-hyperscaler-metrics-kernel` — declares
  `HyperscalerMetrics` trait with one method per canonical metric family
  (`record_capability_circuit_state`, `record_capability_retry_budget_exhausted`,
  `record_responses_429`, `record_responses_5xx`, `record_responses_total`,
  `record_request_success`, `record_request_total`). Pure trait (no I/O,
  no Prometheus dep); port-in-kernel per ADR-0056.
- `crates/oya-shared-hyperscaler-metrics-adapter-prometheus` — reference
  impl against `prometheus 0.13.x`. Registers all seven families against
  a `prometheus::Registry` at startup; binds the `microservice` label
  once (foot-gun closed); enforces canonical name shape via
  `MetricFamily::canonical_set()`.

Per ADR-0064 canonical-base-and-localization-packs, the kernel is the
**seam** (the canonical surface every µservice integrates against); the
prometheus adapter is the canonical **impl**; per-pack adapters
(e.g. an OpenTelemetry-only deployment in pack-eu Sovereign-Cloud
overlays) replace the prometheus adapter without touching the kernel.

The composition root of each µservice's `app` crate constructs ONE
adapter and shares it as `Arc<dyn HyperscalerMetrics>` across the BCs;
see `microservices/messenger/IP-NEW-hyperscaler-metric-emission.md` for
the canonical wiring pattern (pilot µservice, M02-P14).

## Cardinality discipline (ADR-0151)

Per `docs/standards/request-id-canonical.md` + ADR-0151, the following
labels MUST NEVER appear on any Prometheus / Mimir metric. They are
high-cardinality (one-series-per-request or per-entity); emitting them
on metrics blows up the TSDB. They MAY appear in Tempo span attributes
and Loki log fields only.

Six canonical high-cardinality labels:

- `request_id` — per-request correlation id (ULID) per ADR-0151.
- `user_id` — per-user identifier.
- `session_id` — per-session identifier.
- `document_id` — per-document identifier (drive, docs, notes).
- `message_id` — per-message identifier (messenger, mail).
- `channel_id` — per-channel identifier (messenger).

Every µservice's `ServiceMonitor.metricRelabelings` MUST emit a
`labeldrop` action for each label above that the µservice's runtime
plausibly emits. Compliance enforced by the
`oya gate validate metric-cardinality` lane.

## CI enforcement

A Rust check (planned: `oya-check-canonical-base-cohesion`) verifies:

1. `microservices/<ms>/iac/helm/<chart>/templates/prometheusrule.yaml`
   contains **no** group named `*-hyperscaler-overlay-circuit-breaker`,
   `*-hyperscaler-overlay-tenant-rate-limit`,
   `*-hyperscaler-overlay-reference-signals`, or
   `*-hyperscaler-overlay-error-budget-burn`.
2. `microservices/<ms>/manifest.json#hyperscaler_inv_coverage` references
   the canonical PrometheusRule path
   `microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yaml`
   and the canonical alert names
   (`OyaCapabilityCircuitOpen`, `OyaTenantRateLimit429Surge`,
   `OyaSaturationCpuOver70pct`, `OyaErrorBudgetFastBurn1h14x`).
3. The crates in `microservices/<ms>/src/crates/` that produce HTTP/gRPC
   responses emit `oya_<ms>_responses_total`, `oya_<ms>_responses_5xx_total`,
   `oya_<ms>_responses_429_total` (verified by codegen template or static
   scan of metric registration).

Until that check lands, the canonical-base discipline is enforced by
review.

## Authority

- ADR-0064 — canonical-base-and-localization-packs.
- ADR-0128 — hyperscaler architecture invariants (the four INV-* ids).
- ADR-0139 — agentic SLO-gated promotion (consumer of these signals).
- ADR-0131 — per-microservice flat layout.
- ADR-0133 — industry best-practice + hyperscaler conformance program.
- `docs/standards/hyperscaler-invariant-conformance.md` — normative
  conformance declaration for every microservice.
