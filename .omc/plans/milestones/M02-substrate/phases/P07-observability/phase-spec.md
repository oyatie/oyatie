---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M02-substrate
phase: P07-observability
status: Proposed
entry_gate: |
  M01-P05 complete; oya-tenancy-kernel ships; OpenTelemetry SDK available
  as workspace dependency; VictoriaMetrics reachable in dev; cargo check exits 0.
exit_gate: |
  All observability crates compile; OTel trace exporter wired; structured JSON
  logs emit tenant_id + trace_id on every request; p50/p99/p999 histogram
  metrics exported; Grafana dashboard JSON committed; per-tenant partitioning
  verified; grit done; ICM row emitted.
depends_on:
  - milestone: M01
    phase: P05-scaffold-locks
    reason: "workspace scaffold prerequisite"
owner_team: council-architecture
---

# P07-observability: Observability substrate — OpenTelemetry SDK, VictoriaMetrics, structured JSON logs, Prometheus→Grafana, per-tenant partitioning

## Purpose

This phase delivers the complete Observability substrate per Bominal ADR-0042 (observability stack). Every µservice in the oyatie workspace wires against this substrate's ports at startup — not directly against OTel/VictoriaMetrics SDKs. The three pillars are: (1) distributed traces via OpenTelemetry SDK → OTLP exporter → Jaeger/Tempo; (2) metrics via OpenTelemetry metrics API → Prometheus scrape endpoint → VictoriaMetrics → Grafana; (3) structured JSON logs with mandatory fields `tenant_id`, `trace_id`, `span_id`, `service_name`, `severity`. Per-tenant partitioning ensures no tenant can observe another's telemetry data. This phase ships the shared `oya-observability-*` crates that every other M02 phase will depend on for instrumentation.

---

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `observability` | `traces`, `metrics`, `logs` | `crates/oya-observability-{traces,metrics,logs}-{kernel,domain,application,adapter}/`, `crates/oya-observability-worker/`, `crates/oya-observability-rest/`, `crates/oya-observability-app/` | 3×4 + 1 worker + 1 rest + 1 app = 16 crates |

Naming justification:

```
NAME: oya-observability-traces-kernel
JUSTIFICATION:
- microservice = observability: the telemetry substrate; OTel + VictoriaMetrics
- bc-tokens = traces: distributed tracing BC; distinct from metrics (counters/histograms)
  and logs (structured JSON log pipeline)
- layer = kernel: TracerPort + SpanContext types; zero I/O; OTel SDK dependency
  allowed in adapter only; kernel holds only the port trait
- exemptions claimed: none

NAME: oya-observability-metrics-kernel
JUSTIFICATION:
- microservice = observability: same µservice
- bc-tokens = metrics: Prometheus/VictoriaMetrics histogram/counter BC
- layer = kernel: MetricsPort trait (record_histogram, increment_counter, set_gauge)
- exemptions claimed: none
```

### Out-of-scope

- Log aggregation pipeline (Loki/Vector/Fluentd) — infra concern owned by oya-cloud.
- Alerting rule files (Grafana alert YAML) — owned by on-call runbook phase.
- ClickHouse analytics sink for event replay — owned by eventing substrate (P05).

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`impl-plan.md`](impl-plan.md) | Full OTel SDK wiring + VictoriaMetrics adapter + JSON log format + per-tenant partitioning + Grafana dashboard JSON | pending | `council-architecture` |

---

## Acceptance Gates

### Cargo / CI gates

```bash
cargo check --workspace --all-features               # exit 0
cargo build --workspace --all-features               # exit 0
cargo clippy --workspace --all-features -- -D warnings  # exit 0
cargo nextest run --workspace --all-features         # exit 0; 0 failures
cargo deny check                                     # exit 0
cargo doc --workspace --no-deps                      # exit 0; 0 warnings
```

### Fitness lane gates

```bash
oya gate validate lean-a1 --phase P07-observability
oya gate validate lean-a2 --phase P07-observability
oya gate validate lean-a3 --phase P07-observability
oya gate validate lean-a4 --phase P07-observability
```

### Observability correctness gates

```bash
# Structured JSON log format: mandatory fields present
cargo nextest run -p oya-observability-logs-domain --test json_log_fields_mandatory  # exit 0
# Tenant isolation: no cross-tenant trace leakage
cargo nextest run -p oya-observability-traces-application --test tenant_trace_isolation  # exit 0
# p50/p99/p999 histogram buckets exported
cargo nextest run -p oya-observability-metrics-adapter --test histogram_buckets  # exit 0
# OTel trace → OTLP exporter round-trip (against local collector)
cargo nextest run -p oya-observability-traces-adapter --test otlp_round_trip  # exit 0
```

### Load test gate

```bash
k6 run tests/load/smoke-observability-metrics.js --env BASE_URL=http://localhost:8085
# Pass: metrics endpoint p99 ≤50ms; 0 errors at 10k scrapes/min
```

---

## Clean Architecture Compliance

### Layer assignments

| Crate (BNF v4.1) | Layer | Port traits in kernel? | Impls in adapter? | Presentation-only? |
|---|---|---|---|---|
| `oya-observability-traces-kernel` | `kernel` | Yes — `TracerPort` | N/A | No |
| `oya-observability-metrics-kernel` | `kernel` | Yes — `MetricsPort` | N/A | No |
| `oya-observability-logs-kernel` | `kernel` | Yes — `StructuredLogPort` | N/A | No |
| `oya-observability-traces-adapter` | `adapter` | N/A | Yes — `OtelTracerAdapter` | No |
| `oya-observability-metrics-adapter` | `adapter` | N/A | Yes — `VictoriaMetricsAdapter` | No |
| `oya-observability-logs-adapter` | `adapter` | N/A | Yes — `JsonLogAdapter` | No |
| `oya-observability-app` | `app` | N/A | Unrestricted inward | No |

### Port traits declared in kernel

```rust
// oya-observability-traces-kernel/src/ports.rs
#[doc(hidden)]
mod sealed { pub trait Sealed {} }

pub trait TracerPort: Send + Sync + sealed::Sealed {
    fn start_span(&self, name: &'static str, ctx: SpanContext) -> BoxedSpan;
    fn inject_context(&self, span: &BoxedSpan, carrier: &mut dyn Carrier);
    fn extract_context(&self, carrier: &dyn Carrier) -> SpanContext;
}

// oya-observability-metrics-kernel/src/ports.rs
pub trait MetricsPort: Send + Sync + sealed::Sealed {
    fn record_histogram(&self, name: &'static str, value: f64,
        labels: &[(&'static str, &str)]);
    fn increment_counter(&self, name: &'static str, labels: &[(&'static str, &str)]);
    fn set_gauge(&self, name: &'static str, value: f64, labels: &[(&'static str, &str)]);
}

// oya-observability-logs-kernel/src/ports.rs
/// Mandatory fields on every log record: tenant_id, trace_id, span_id, service_name, severity.
pub trait StructuredLogPort: Send + Sync + sealed::Sealed {
    fn log(&self, record: LogRecord);
}

#[derive(Debug)]
pub struct LogRecord {
    pub tenant_id: Option<TenantId>,
    pub trace_id: Option<TraceId>,
    pub span_id: Option<SpanId>,
    pub service_name: &'static str,
    pub severity: Severity,
    pub message: String,
    pub fields: Vec<(String, serde_json::Value)>,
}
```

### CI lanes that must green before phase exit gate

| Lane | Command | Expected |
|---|---|---|
| `dependency-direction` | `oya gate validate lean-a1 --phase P07-observability` | exit 0 |
| `statelessness` | `oya gate validate statelessness --phase P07-observability` | exit 0 |

### New BCs registered in this phase

| BC name | Owner µservice | Registration PR |
|---|---|---|
| `traces` | `observability` | pending |
| `metrics` | `observability` | pending |
| `logs` | `observability` | pending |

---

## Grit Claim Symbols

```
crates/oya-observability-traces-kernel/src/ports.rs::TracerPort
crates/oya-observability-metrics-kernel/src/ports.rs::MetricsPort
crates/oya-observability-logs-kernel/src/ports.rs::StructuredLogPort
crates/oya-observability-traces-adapter/src/otel.rs::OtelTracerAdapter
crates/oya-observability-metrics-adapter/src/victoria.rs::VictoriaMetricsAdapter
dashboards/grafana/observability-overview.json::GrafanaDashboard
```

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "Phase P07-observability started; scope: 16 crates (traces/metrics/logs BCs); OTel SDK + VictoriaMetrics + structured JSON logs; per-tenant partitioning" \
  -i high \
  -k "M02,P07,phase-start,observability"

icm store \
  -t context-oyatie \
  -c "Phase P07-observability complete; OTel wired; VictoriaMetrics adapter green; JSON log fields verified; per-tenant isolation tested; Grafana dashboard committed; next: P08-kms" \
  -i high \
  -k "M02,P07,phase-complete,observability"
```

---

## References

- Bominal ADRs inherited: ADR-0042 (observability stack)
- oyatie ADRs: ADR-0056 (BNF v4.1)
- depends_on: M01-P05
- unblocks: all Wave-B product phases (instrument via oya-observability-*-kernel ports)
