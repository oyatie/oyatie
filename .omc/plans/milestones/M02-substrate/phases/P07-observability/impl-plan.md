---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-substrate
phase: P07-observability
impl_plan_id: IP-P07-observability-substrate
status: pending
owner: council-architecture
blocked_by: []
acceptance_lanes:
- cargo-check
- cargo-build
- cargo-clippy
- cargo-nextest
- cargo-deny
- lean-a1
- lean-a2
- lean-a3
- lean-a4
purpose: "Delivers the complete Observability substrate: 16 crates across 3 BCs (traces, metrics, logs), OpenTelemetry SDK wiring (traces + metrics + logs), VictoriaMetrics Prometheus-compatible adapter."
---
# IP-P07-observability-substrate: Scaffold 16 observability crates with OTel SDK, VictoriaMetrics, structured JSON logs

## Intent

Delivers the complete Observability substrate: 16 crates across 3 BCs (traces, metrics, logs), OpenTelemetry SDK wiring (traces + metrics + logs), VictoriaMetrics Prometheus-compatible adapter, structured JSON log format with mandatory fields (tenant_id, trace_id, span_id, service_name, severity), per-tenant trace isolation, Grafana dashboard JSON, load test p99≤50ms on metrics endpoint.

---

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `crates/oya-observability-traces-kernel/Cargo.toml` | create | TracerPort trait; SpanContext types |
| `crates/oya-observability-traces-kernel/src/ports.rs` | create | TracerPort sealed trait |
| `crates/oya-observability-traces-kernel/src/types.rs` | create | SpanContext, TraceId, SpanId, BoxedSpan, Carrier trait |
| `crates/oya-observability-metrics-kernel/Cargo.toml` | create | MetricsPort trait |
| `crates/oya-observability-metrics-kernel/src/ports.rs` | create | MetricsPort sealed trait |
| `crates/oya-observability-logs-kernel/Cargo.toml` | create | StructuredLogPort trait; LogRecord struct |
| `crates/oya-observability-logs-kernel/src/ports.rs` | create | StructuredLogPort sealed trait |
| `crates/oya-observability-logs-kernel/src/types.rs` | create | LogRecord, Severity enum, mandatory fields enforcement |
| `crates/oya-observability-traces-domain/src/span.rs` | create | Span abstraction; propagation helpers |
| `crates/oya-observability-metrics-domain/src/histogram.rs` | create | Histogram bucket definitions: p50/p99/p999 standard buckets |
| `crates/oya-observability-logs-domain/src/formatter.rs` | create | JSON log formatting; mandatory fields validation |
| `crates/oya-observability-traces-application/src/middleware.rs` | create | axum/tonic middleware: extract trace context from headers |
| `crates/oya-observability-metrics-application/src/recorder.rs` | create | MetricsRecorder use-case; standard metric names |
| `crates/oya-observability-logs-application/src/logger.rs` | create | StructuredLogger use-case |
| `crates/oya-observability-traces-adapter/src/otel.rs` | create | OtelTracerAdapter: opentelemetry SDK + OTLP exporter |
| `crates/oya-observability-metrics-adapter/src/victoria.rs` | create | VictoriaMetricsAdapter: prometheus_client crate; scrape endpoint |
| `crates/oya-observability-logs-adapter/src/json.rs` | create | JsonLogAdapter: tracing-subscriber + JSON formatter |
| `crates/oya-observability-worker/src/metrics_pusher.rs` | create | optional push gateway worker for VictoriaMetrics |
| `crates/oya-observability-rest/src/routes.rs` | create | GET /metrics (Prometheus scrape), GET /health |
| `crates/oya-observability-app/src/main.rs` | create | composition root; init OTel global tracer + meter |
| `dashboards/grafana/observability-overview.json` | create | Grafana dashboard JSON: p50/p99/p999 per-service panels |
| `tests/load/smoke-observability-metrics.js` | create | k6 smoke test |
| `Cargo.toml` | update | add all 16 observability crates |

---

## Crate Naming

```
NAME: oya-observability-traces-kernel
JUSTIFICATION:
- microservice = observability: telemetry substrate; OTel + VictoriaMetrics
- bc-tokens = traces: distributed tracing BC
- layer = kernel: TracerPort trait + SpanContext types; zero I/O
- exemptions claimed: none
```

---

## Code Shape

### `crates/oya-observability-logs-kernel/src/types.rs`

```rust
use uuid::Uuid;

/// Severity levels aligned with OpenTelemetry LogRecord severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Severity { Trace, Debug, Info, Warn, Error, Fatal }

/// Mandatory fields on every log record per oyatie structured-log standard.
/// Any log emission that omits service_name fails at compile time (non-optional field).
#[derive(Debug, serde::Serialize)]
pub struct LogRecord {
    /// Optional: set from RLS context when processing a tenant-scoped request.
    pub tenant_id:    Option<Uuid>,
    /// Optional: populated from OTel trace context propagation.
    pub trace_id:     Option<[u8; 16]>,
    pub span_id:      Option<[u8; 8]>,
    /// MANDATORY: the BNF v4.1 crate name of the emitting service.
    pub service_name: &'static str,
    pub severity:     Severity,
    pub message:      String,
    pub fields:       Vec<(String, serde_json::Value)>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp:    chrono::DateTime<chrono::Utc>,
}

impl LogRecord {
    pub fn new(service_name: &'static str, severity: Severity, message: impl Into<String>) -> Self {
        Self {
            tenant_id: None, trace_id: None, span_id: None,
            service_name, severity, message: message.into(),
            fields: Vec::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn with_tenant(mut self, tenant_id: Uuid) -> Self { self.tenant_id = Some(tenant_id); self }
    pub fn with_trace(mut self, trace_id: [u8; 16], span_id: [u8; 8]) -> Self {
        self.trace_id = Some(trace_id); self.span_id = Some(span_id); self
    }
    pub fn with_field(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.fields.push((key.into(), value.into())); self
    }
}
```

### `crates/oya-observability-metrics-domain/src/histogram.rs`

```rust
/// Standard histogram bucket boundaries for p50/p99/p999 targets.
/// Latency buckets in milliseconds matching Bominal ADR-0107 targets.
pub const LATENCY_BUCKETS_MS: &[f64] = &[
    1.0, 2.0, 5.0, 10.0, 25.0, 50.0, 100.0, 200.0, 500.0, 1000.0, 2000.0, 5000.0,
];

/// Standard metric names. Use these constants to avoid typos.
pub mod metric_names {
    pub const HTTP_REQUEST_DURATION_MS: &str = "oyatie_http_request_duration_ms";
    pub const DB_QUERY_DURATION_MS: &str = "oyatie_db_query_duration_ms";
    pub const KAFKA_PUBLISH_DURATION_MS: &str = "oyatie_kafka_publish_duration_ms";
    pub const OUTBOX_QUEUE_DEPTH: &str = "oyatie_outbox_queue_depth";
    pub const ACTIVE_SESSIONS: &str = "oyatie_active_sessions";
    pub const AUDIT_SEAL_DURATION_MS: &str = "oyatie_audit_seal_duration_ms";
}
```

### `crates/oya-observability-traces-adapter/src/otel.rs`

```rust
use opentelemetry::trace::{Tracer, TracerProvider};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::SdkTracerProvider;
use oya_observability_traces_kernel::ports::TracerPort;
use oya_observability_traces_kernel::types::{BoxedSpan, Carrier, SpanContext};

pub struct OtelTracerAdapter {
    tracer: opentelemetry_sdk::trace::Tracer,
}

impl OtelTracerAdapter {
    pub fn init(service_name: &'static str, otlp_endpoint: &str) -> anyhow::Result<Self> {
        let provider = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(otlp_endpoint)
            .build()?;
        let provider = SdkTracerProvider::builder()
            .with_batch_exporter(provider)
            .with_resource(opentelemetry_sdk::Resource::builder()
                .with_service_name(service_name)
                .build())
            .build();
        let tracer = provider.tracer(service_name);
        Ok(Self { tracer })
    }
}

impl TracerPort for OtelTracerAdapter {
    fn start_span(&self, name: &'static str, ctx: SpanContext) -> BoxedSpan {
        use opentelemetry::trace::Tracer;
        let parent = opentelemetry::Context::current(); // propagate from incoming context
        Box::new(self.tracer.start_with_context(name, &parent))
    }

    fn inject_context(&self, span: &BoxedSpan, carrier: &mut dyn Carrier) {
        opentelemetry::global::get_text_map_propagator(|p| {
            p.inject_context(&opentelemetry::Context::current(), &mut CarrierAdapter(carrier));
        });
    }

    fn extract_context(&self, carrier: &dyn Carrier) -> SpanContext {
        let ctx = opentelemetry::global::get_text_map_propagator(|p| {
            p.extract(&CarrierAdapter(carrier))
        });
        SpanContext::from_otel(ctx)
    }
}

struct CarrierAdapter<'a>(&'a mut dyn Carrier);
impl<'a> opentelemetry::propagation::Injector for CarrierAdapter<'a> {
    fn set(&mut self, key: &str, value: String) { self.0.set(key, value); }
}
impl<'a> opentelemetry::propagation::Extractor for CarrierAdapter<'a> {
    fn get(&self, key: &str) -> Option<&str> { self.0.get(key) }
    fn keys(&self) -> Vec<&str> { vec![] }
}
```

### `dashboards/grafana/observability-overview.json`

```json
{
  "title": "oyatie Observability Overview",
  "uid": "oyatie-obs-overview",
  "version": 1,
  "panels": [
    {
      "title": "HTTP Request Latency (p50/p99/p999)",
      "type": "graph",
      "targets": [
        { "expr": "histogram_quantile(0.50, rate(oyatie_http_request_duration_ms_bucket[5m]))", "legendFormat": "p50" },
        { "expr": "histogram_quantile(0.99, rate(oyatie_http_request_duration_ms_bucket[5m]))", "legendFormat": "p99" },
        { "expr": "histogram_quantile(0.999, rate(oyatie_http_request_duration_ms_bucket[5m]))", "legendFormat": "p999" }
      ],
      "alert": {
        "name": "p99 > 200ms", "frequency": "1m",
        "conditions": [{ "evaluator": { "type": "gt", "params": [200] }, "query": { "model": { "expr": "histogram_quantile(0.99, ...)" } } }]
      }
    },
    {
      "title": "Outbox Queue Depth",
      "type": "stat",
      "targets": [{ "expr": "oyatie_outbox_queue_depth", "legendFormat": "{{microservice}}" }]
    },
    {
      "title": "Active Sessions",
      "type": "stat",
      "targets": [{ "expr": "oyatie_active_sessions", "legendFormat": "sessions" }]
    },
    {
      "title": "Audit Seal Latency (ms)",
      "type": "graph",
      "targets": [{ "expr": "oyatie_audit_seal_duration_ms", "legendFormat": "seal_ms" }],
      "alert": { "name": "seal > 1000ms", "conditions": [{ "evaluator": { "type": "gt", "params": [1000] } }] }
    }
  ]
}
```

### `tests/load/smoke-observability-metrics.js`

```javascript
import http from 'k6/http';
import { check } from 'k6';

export const options = {
  vus: 50, duration: '60s',
  thresholds: {
    http_req_duration: ['p(99)<50'],   // metrics endpoint ≤50ms
    http_req_failed: ['rate<0.001'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8085';

export default function () {
  const res = http.get(`${BASE_URL}/metrics`);
  check(res, {
    'metrics 200': (r) => r.status === 200,
    'contains oyatie metric': (r) => r.body.includes('oyatie_http_request_duration_ms'),
  });
}
```

---

## Acceptance Gates

```bash
cargo check -p oya-observability-traces-kernel --all-features   # exit 0
cargo check -p oya-observability-metrics-adapter --all-features  # exit 0
cargo clippy --workspace --all-features -- -D warnings            # exit 0
cargo nextest run --workspace --all-features                      # exit 0
# JSON log fields test
cargo nextest run -p oya-observability-logs-domain --test json_log_fields_mandatory  # exit 0
# Tenant isolation
cargo nextest run -p oya-observability-traces-application --test tenant_trace_isolation  # exit 0
# Histogram buckets
cargo nextest run -p oya-observability-metrics-adapter --test histogram_buckets  # exit 0
# Load test
k6 run tests/load/smoke-observability-metrics.js --env BASE_URL=http://localhost:8085
```

---

## Test Plan

### Unit tests

| Test name | What it verifies |
|---|---|
| `test_log_record_mandatory_service_name` | LogRecord without service_name fails to compile |
| `test_log_record_json_format` | Serialized JSON contains all mandatory fields |
| `test_severity_ordering` | Severity::Error > Severity::Info |
| `test_histogram_bucket_boundaries` | LATENCY_BUCKETS_MS covers p50/p99/p999 range |
| `test_metric_name_constants` | No typos in metric name constants |
| `test_otel_trace_inject_extract` | inject context → extract context → same TraceId |

### Integration tests

| Test name | What it verifies |
|---|---|
| `integration_otlp_span_exported` | Span started → OTLP collector receives it |
| `integration_prometheus_scrape_endpoint` | GET /metrics returns valid Prometheus text format |
| `integration_tenant_isolation_trace` | Trace from tenant A not visible in tenant B query |

---

## Clean Architecture Compliance

| Crate | Layer | Imports (layers only) | Forbidden imports |
|---|---|---|---|
| `oya-observability-traces-kernel` | `kernel` | `opentelemetry` API (external) | all project layers |
| `oya-observability-metrics-kernel` | `kernel` | nothing project-internal | all project layers |
| `oya-observability-traces-adapter` | `adapter` | `traces-application`, `traces-kernel`; `opentelemetry-otlp` (external) | presentation |
| `oya-observability-metrics-adapter` | `adapter` | `metrics-application`, `metrics-kernel`; `prometheus_client` (external) | presentation |
| `oya-observability-app` | `app` | all | none |

---

## Load Test

```bash
k6 run tests/load/smoke-observability-metrics.js --env BASE_URL=http://localhost:8085
# Pass: p99 ≤50ms; 0 errors at 50 VUs/60s

vegeta attack -rate=10000/s -duration=30s -targets=<(echo "GET http://localhost:8085/metrics") | vegeta report
# Pass: p99 ≤50ms; success_rate=100%
```

---

## Grit Symbol-Locks

```bash
grit claim \
  --agent m02-wave-a-executor \
  --intent "IP-P07-observability: 16 crates + OTel + VictoriaMetrics + JSON logs + Grafana" \
  --ttl 7200 \
  crates/oya-observability-traces-kernel/src/ports.rs::TracerPort \
  crates/oya-observability-metrics-kernel/src/ports.rs::MetricsPort \
  crates/oya-observability-logs-kernel/src/ports.rs::StructuredLogPort \
  dashboards/grafana/observability-overview.json::GrafanaDashboard
```

---

## ICM Rows to Emit

```bash
icm store \
  -t context-oyatie \
  -c "IP-P07-observability merged; 16 crates; OTel SDK wired; VictoriaMetrics adapter; JSON logs mandatory fields; per-tenant isolation; Grafana dashboard; next: P08-kms/impl-plan" \
  -i high \
  -k "M02,P07,IP-P07,observability"
```

---

## Next IP Pointer

`phases/P08-kms/impl-plan.md`

---

## Cross-References

- Phase spec: `phase-spec.md`
- Bominal ADR-0042 (observability stack)
- opentelemetry-rust: https://github.com/open-telemetry/opentelemetry-rust
