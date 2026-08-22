---
purpose: "Cross-cutting observability standard. Mandates OpenTelemetry as the canonical emission fabric, names the structured-logging schema, codifies the audit-chain `EVT-*` emission contract, requires Prometheus 3.11+ (post-3.5-EOL)."
doc_status: published
---

---
doc_class: Standard
shape: ~
length_cap: 250
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Cross-cutting observability standard. Mandates OpenTelemetry as the canonical
  emission fabric, names the structured-logging schema, codifies the audit-chain
  `EVT-*` emission contract, requires Prometheus 3.11+ (post-3.5-EOL), enables
  Honeycomb-style exemplars on traces/metrics correlation, and sets retention
  defaults. Operates within `decision-principles.json` DP-08 (audit-chain
  emission on every cross-axis flow).
canonical_authority: /specs/decision-principles.json + /specs/forbidden-operations.json
planned_enforcement_ref: governance-otel-emit
companion_docs:
  - docs/standards/error-handling.md
  - docs/standards/on-call.md
  - docs/standards/data-class.md
  - docs/standards/release-management.md
  - docs/SLO-CATALOG.md
related_adrs:
  - ADR-0053
  - ADR-0052
  - ADR-0054
---

# Observability

## Doctrinal authority — [decision-principles.json](../../specs/decision-principles.json) + [forbidden-operations.json](../../specs/forbidden-operations.json)

Per [`decision-principles.json`](../../specs/decision-principles.json) DP-08 — "Audit-chain
emission on every cross-axis flow" — observability is **not** an optional
operational concern: cross-pillar data movement without emission is a
defect, not an optimization. This standard names the three pillars
(traces, metrics, logs) plus the audit-chain emission contract.

## 1. OpenTelemetry mandatory

Every `oya-*` service binary emits telemetry via the **OpenTelemetry SDK**
(Rust crate family: `opentelemetry`, `opentelemetry-otlp`, `tracing-opentelemetry`,
`opentelemetry_sdk`). OTLP (gRPC or HTTP) is the wire format.

- **Collector**: `opentelemetry-collector-releases` ≥ **v0.151.0** per
  [`.omc/scratch/lts-versions-verified-2026-05-12.md`](../../.omc/scratch/lts-versions-verified-2026-05-12.md).
- **Deployment pattern**: agent (per-host or per-pod sidecar) + gateway
  (centralized aggregator). Hierarchical is acceptable for high-volume
  axes; pure-gateway is forbidden because per-host fan-in becomes a SPOF.
- **Backend split** (env-configurable, reversible):
  - Open-source path: Grafana stack (Tempo / Mimir / Loki).
  - Honeycomb for high-cardinality query.
  - Datadog only when the regional pack mandates a vendor SaaS.

Lane: `governance-otel-emit` validates every service has an OTLP
exporter wired and emits the three pillars.

Sources: [OpenTelemetry Collector docs](https://opentelemetry.io/docs/collector/),
[Honeycomb — OpenTelemetry](https://www.honeycomb.io/platform/opentelemetry),
[Better Stack — OTel Best Practices](https://betterstack.com/community/guides/observability/opentelemetry-best-practices/).

## 2. Three pillars

### 2.1 Traces

- W3C Trace Context propagation (`traceparent`, `tracestate`) on every
  HTTP / gRPC / message-queue boundary.
- Span naming: `<verb>.<noun>` lowercase (`foundry.capability.invoke`,
  `audit.event.emit`).
- Sampling: head-based 100% for Sev-1 paths; tail-based via
  `processor.tail_sampling` for high-volume routes (target: keep all
  errors, sample 1% of success).
- Retention: 7–14 days raw at high resolution.

### 2.2 Metrics

- Cardinality budget: ≤ 50 unique label combinations per metric per
  service instance; metrics violating this MUST refactor (per Honeycomb
  guidance — high cardinality belongs in traces, not metrics).
- Naming: `<axis>_<service>_<noun>_<unit>` (e.g., `foundry_runtime_invocations_total`,
  `foundry_runtime_invocation_duration_seconds`).
- Histogram buckets for latency: `0.005, 0.01, 0.025, 0.05, 0.1, 0.25,
  0.5, 1, 2.5, 5, 10` seconds.
- Backend: **Prometheus 3.11+** (the `release-management.md` and on-call
  alerting depend on the 3.11 multi-window-burn-rate features).
  **Prometheus 3.5 LTS expires 2026-07-31** — migration is mandatory.
- Retention: high-res 7d; downsampled 90d.

### 2.3 Logs

- Structured JSON only (no free-text log lines in production).
- Library: `tracing` + `tracing-subscriber` + `tracing-bunyan-formatter`
  OR `tracing-stackdriver` for cloud sinks.
- Levels: `ERROR`, `WARN`, `INFO`, `DEBUG`, `TRACE` (only `INFO`+ in
  production by default; `DEBUG` enabled via runtime flag for short
  windows).

## 3. Structured logging schema

Every log line carries:

| Field | Type | Required | Note |
|---|---|---|---|
| `timestamp` | ISO-8601 UTC | YES | nanosecond precision |
| `severity` | string | YES | `ERROR`/`WARN`/`INFO`/`DEBUG`/`TRACE` |
| `service` | string | YES | crate package name (`intelligence-runtime-rag`) |
| `version` | string | YES | git SHA short |
| `trace_id` | hex | when in span | W3C trace context |
| `span_id` | hex | when in span | W3C span context |
| `tenant_id` | string | when tenant-scoped | per tenancy plane |
| `actor_id` | string | when actor-scoped | user or agent |
| `data_class` | string | when handling fielded data | per data-class.md |
| `message` | string | YES | human-readable |
| `error.class` | string | on errors | thiserror variant name |
| `error.source` | string | on errors | chained error display |
| `audit.evt_id` | ULID | on audit emission | `EVT-*` correlation |

No secrets in `message` or any field; redacted by the `tracing-subscriber`
filter layer at edge. Lane: `governance-log-schema` validates
the JSON shape on golden fixtures.

## 4. Audit-chain `EVT-*` emission contract

Per [`decision-principles.json`](../../specs/decision-principles.json) DP-08 and ADR-0003, every
cross-pillar / cross-axis data movement emits an `EVT-*` record into the
audit chain. The chain is hash-linked and replayable per
DOC-CATALOG.md §4 `audit-chain-replay` lane.

Required emission topics (non-exhaustive):

| Topic | Trigger | Owner |
|---|---|---|
| `EVT-CAPABILITY-INVOKED` | Foundry capability execution | axis-foundry |
| `EVT-DATA-EGRESS` | Data leaves an axis boundary | per axis |
| `EVT-DSR-EXECUTED` | DSR (data subject request) ran | council-privacy |
| `EVT-CONSTITUTION-AMENDED` | Constitution merge | council-architecture |
| `EVT-INCIDENT-OPENED` / `-CLOSED` | Sev-1/2 lifecycle | ops-sre-reliability |
| `EVT-POSTMORTEM-PUBLISHED` | Postmortem merged | ops-sre-reliability |
| `EVT-DOC-UPDATED` | Canonical doc change (DOC-CATALOG.md §3.4) | doc-owner |
| `EVT-RELEASE-DEPLOYED` | Production deploy succeeds | ops-sre-reliability |
| `EVT-ERROR-*` | Cross-pillar errors (per `error-handling.md` §7) | per crate |
| `EVT-AUTONOMY-UPLIFT` | Capability tier change | capability-reviewer |
| `EVT-TOOL-INVOKED` | Agent-harness `Bash` invocation (telemetry hook) | agent-harness |

Every `EVT-*` record carries `evt_id` (ULID), `timestamp`, `service`,
`tenant_id` (if applicable), `actor_id`, `trace_id`, and a topic-specific
payload. Records are JSON, hash-chained, persisted under audit shards.

Lane: `governance-audit-emission` validates that every
cross-pillar code path emits the required topic.

## 5. Exemplars (trace ↔ metric correlation)

Per Honeycomb-style observability:

- Every histogram metric emission attaches an **exemplar** linking the
  bucket to an exemplary trace ID (W3C `trace_id`).
- Grafana / Tempo / Mimir surface the exemplar marker on dashboards;
  clicking traverses to the trace.
- Cost: low if cardinality is bounded; high if trace_id is leaked into
  label sets (do NOT label-on-trace-id).

The lane `governance-exemplar-coverage` checks that latency
histograms on hot paths emit exemplars.

## 6. Retention defaults

| Pillar | Hot retention | Cold retention | Notes |
|---|---|---|---|
| Traces | 7–14 days | (per regional pack) | tail-sampled 1% success retained 30d |
| Metrics (high-res) | 7 days | 90 days downsampled | per `prometheus.yml` rules |
| Logs | 14 days | 365 days cold | per privacy program (PII auto-redacted) |
| Audit chain (`EVT-*`) | indefinite | indefinite | hash-chained; immutable |

Regional packs (e.g., Korea per `regional-packs/` ) may override per
PIPC / MFDS / KCC retention rules.

## 7. SLO derivation

SLOs in [`docs/SLO-CATALOG.md`](../SLO-CATALOG.md) are derived from
emitted metrics. The four golden signals (Google SRE) apply per service:

1. **Latency** — `<service>_<op>_duration_seconds` p50/p95/p99.
2. **Traffic** — `<service>_<op>_total` request rate.
3. **Errors** — `<service>_<op>_errors_total` / total.
4. **Saturation** — utilization of bounded resources (queue depth,
   connection-pool usage, GC heap).

Burn-rate alerts derive from §6 in
[`on-call.md`](on-call.md).

Source: [Splunk — Four Golden Signals](https://www.splunk.com/en_us/blog/learn/sre-metrics-four-golden-signals-of-monitoring.html).

## 8. Privacy and redaction

- No PII / PHI in metric labels (cardinality + privacy risk).
- Logs: redaction layer at the `tracing-subscriber` boundary maps field
  names to redaction rules; per data class per
  [`data-class.md`](data-class.md).
- Traces: span attributes scrubbed by the OTel SDK redaction processor
  before export.
- Audit chain: PII fields are hashed with a tenant-scoped salt; the raw
  values live in the source pillar only and are recoverable only through
  the DSR pathway.

## 9. Anti-patterns

1. **Free-text log lines in production** — switch to structured JSON.
2. **High-cardinality metric labels** (`user_id`, `tenant_id`,
   `trace_id`) — move to traces.
3. **Sampling that drops errors** — head-sample at 100% on error paths.
4. **Audit-chain emission omitted at a cross-pillar boundary** —
   refused by the lane.
5. **Custom in-house exporters when OTLP fits** — choose OTLP first.

## 10. Sources scanned

- [OpenTelemetry Collector docs](https://opentelemetry.io/docs/collector/).
- [Honeycomb — OpenTelemetry](https://www.honeycomb.io/platform/opentelemetry).
- [Better Stack — OTel Best Practices](https://betterstack.com/community/guides/observability/opentelemetry-best-practices/).
- [Markaicode — Full Stack Observability 2025](https://markaicode.com/2025-observability-opentelemetry-grafana-11-full-stack-monitoring/).
- [Google SRE Workbook — Alerting on SLOs](https://sre.google/workbook/alerting-on-slos/).
- [Splunk — Four Golden Signals](https://www.splunk.com/en_us/blog/learn/sre-metrics-four-golden-signals-of-monitoring.html).
- [Prometheus — Release Cycle](https://prometheus.io/docs/introduction/release-cycle/) (3.5 LTS EOS 2026-07-31).
- ADR-0003 (audit chain).
