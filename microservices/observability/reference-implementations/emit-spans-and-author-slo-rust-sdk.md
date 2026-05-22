---
doc_class: ReferenceImplementation
microservice: observability
language: rust
related_adrs: [ADR-0130, ADR-0131, ADR-0329, ADR-0330, ADR-0331]
date: 2026-05-20
doc_status: published
---

# Reference — Emit spans + metrics + logs from a Rust µservice, then author + dryrun an SLO

This walkthrough is end-to-end runnable. By the end, your service emits OTel spans to the local OTel collector, exports Prometheus metrics, ships logs via OTLP, and you have an OpenSLO manifest dryrun-green for the last 7 d.

## Cargo.toml

```toml
[package]
name = "my-microservice-app"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-observability-sdk = { path = "../../crates/oya-observability-sdk" }
opentelemetry = { version = "0.27", features = ["trace", "metrics", "logs"] }
opentelemetry-otlp = { version = "0.27", features = ["tonic", "trace", "metrics", "logs"] }
opentelemetry_sdk = { version = "0.27", features = ["rt-tokio"] }
tracing = "0.1"
tracing-opentelemetry = "0.28"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tokio = { version = "1.42", features = ["full"] }
axum = "0.7"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
```

## src/observability.rs — initialise OTel pipeline

```rust
use opentelemetry::{global, trace::TracerProvider, KeyValue};
use opentelemetry_otlp::{ExporterBuildError, MetricsExporter, SpanExporter, LogExporter, WithExportConfig};
use opentelemetry_sdk::{
    metrics::SdkMeterProvider,
    trace::SdkTracerProvider,
    logs::SdkLoggerProvider,
    Resource,
};
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init(service_name: &'static str, tenant_id: &str, pack_id: &str) -> anyhow::Result<ShutdownGuard> {
    let resource = Resource::builder()
        .with_attributes(vec![
            KeyValue::new("service.name", service_name),
            KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
            KeyValue::new("tenant.id", tenant_id.to_string()),
            KeyValue::new("pack.id", pack_id.to_string()),
            KeyValue::new("deployment.environment", std::env::var("OYA_ENV").unwrap_or_else(|_| "dev".into())),
        ])
        .build();

    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".into());

    let span_exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint.clone())
        .with_timeout(Duration::from_secs(5))
        .build()?;

    let tracer_provider = SdkTracerProvider::builder()
        .with_batch_exporter(span_exporter)
        .with_resource(resource.clone())
        .build();
    global::set_tracer_provider(tracer_provider.clone());

    let metric_exporter = MetricsExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint.clone())
        .build()?;

    let meter_provider = SdkMeterProvider::builder()
        .with_periodic_exporter(metric_exporter)
        .with_resource(resource.clone())
        .build();
    global::set_meter_provider(meter_provider.clone());

    let log_exporter = LogExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()?;

    let logger_provider = SdkLoggerProvider::builder()
        .with_batch_exporter(log_exporter)
        .with_resource(resource)
        .build();

    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer(service_name));

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,my_microservice_app=debug")))
        .with(otel_layer)
        .init();

    Ok(ShutdownGuard {
        tracer_provider,
        meter_provider,
        logger_provider,
    })
}

pub struct ShutdownGuard {
    tracer_provider: SdkTracerProvider,
    meter_provider: SdkMeterProvider,
    logger_provider: SdkLoggerProvider,
}

impl Drop for ShutdownGuard {
    fn drop(&mut self) {
        let _ = self.tracer_provider.shutdown();
        let _ = self.meter_provider.shutdown();
        let _ = self.logger_provider.shutdown();
    }
}
```

## src/main.rs — emit a span + metric + log inside an HTTP handler

```rust
use axum::{extract::Path, http::StatusCode, response::Json, routing::get, Router};
use opentelemetry::{global, metrics::Counter, KeyValue};
use serde::Serialize;
use std::sync::OnceLock;
use tracing::{info, instrument};

mod observability;

static REQUEST_COUNTER: OnceLock<Counter<u64>> = OnceLock::new();

#[derive(Serialize)]
struct InvoiceResponse {
    invoice_id: String,
    amount_cents: u64,
}

#[instrument(skip_all, fields(tenant_id = %tenant_id, invoice_id = %invoice_id))]
async fn get_invoice(
    Path((tenant_id, invoice_id)): Path<(String, String)>,
) -> Result<Json<InvoiceResponse>, StatusCode> {
    let counter = REQUEST_COUNTER.get_or_init(|| {
        global::meter("my-microservice-app").u64_counter("http.requests").build()
    });
    counter.add(
        1,
        &[
            KeyValue::new("http.route", "/tenant/:tenant/invoice/:invoice"),
            KeyValue::new("http.status_code", 200),
            KeyValue::new("tenant.id", tenant_id.clone()),
        ],
    );

    info!(
        tenant_id = %tenant_id,
        invoice_id = %invoice_id,
        "loaded invoice",
    );

    Ok(Json(InvoiceResponse {
        invoice_id,
        amount_cents: 12_500,
    }))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _guard = observability::init(
        "my-microservice-app",
        &std::env::var("OYA_TENANT_ID").unwrap_or_else(|_| "tenant_oyatie_dev_001".into()),
        &std::env::var("OYA_PACK_ID").unwrap_or_else(|_| "base".into()),
    )?;

    let app = Router::new().route("/tenant/:tenant/invoice/:invoice", get(get_invoice));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    info!("listening on 0.0.0.0:8080");
    axum::serve(listener, app).await?;

    Ok(())
}
```

## Run locally + verify

```sh
docker run -d --name otelcol -p 4317:4317 -p 4318:4318 -p 8888:8888 \
    -v $PWD/otel-collector-config.yaml:/etc/otelcol-contrib/config.yaml \
    otel/opentelemetry-collector-contrib:0.105.0

OYA_ENV=dev OYA_TENANT_ID=tenant_oyatie_dev_001 OYA_PACK_ID=base \
    OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 \
    cargo run --release

curl http://localhost:8080/tenant/tenant_oyatie_dev_001/invoice/inv_42

docker logs otelcol --tail 50 | grep -i "service.name=my-microservice-app"
```

You should see span batches arriving at the collector within 5 s.

## slos/availability.openslo.yaml

```yaml
apiVersion: openslo/v1
kind: SLO
metadata:
  name: my-microservice-app-availability
  displayName: my-microservice-app availability
spec:
  service: my-microservice-app
  description: |
    Tenants expect 99.9 % of /tenant/:tenant/invoice/:invoice requests to return 2xx.
    A breach indicates the invoice-read path is degraded, blocking finance teams from billing operations.
  indicator:
    metadata:
      name: http-2xx-ratio
    spec:
      ratioMetric:
        counter: true
        good:
          metricSource:
            metricSourceRef: prometheus
            spec:
              query: |
                sum(rate(http_requests_total{service="my-microservice-app",http_status_code=~"2.."}[5m]))
        total:
          metricSource:
            metricSourceRef: prometheus
            spec:
              query: |
                sum(rate(http_requests_total{service="my-microservice-app"}[5m]))
  objectives:
    - target: 0.999
      window: 30d
  alertPolicies:
    - alertPolicyRef: multi-window-burn-rate-fast
    - alertPolicyRef: multi-window-burn-rate-slow
  annotations:
    runbook: https://github.com/oyatie/oyatie/blob/dev/microservices/my-microservice-app/runbooks/availability-breach.md
    dashboards: https://grafana.dev.oyatie.io/d/my-microservice-app-overview
```

## Dryrun the SLO

```sh
cargo run -p oya-dev-cli -- observability dryrun-slo \
    --ms my-microservice-app \
    --slo availability.openslo.yaml \
    --window 7d
```

Expected output:

```
SLO: my-microservice-app-availability
Window: 7 d
Objective: 0.999
Actual SLI: 0.9994
Status: GREEN
Error budget consumed: 16 %
```

## Check promotion eligibility

```sh
cargo run -p oya-dev-cli -- observability check-promotion-eligibility \
    --ms my-microservice-app \
    --from dev --to staging
```

Once green, push your branch + open a PR. The ADR-0130 gate will auto-lift.

## Notes

- The `tenant.id` + `pack.id` resource attributes are MANDATORY per ADR-0130; without them the SLO engine cannot partition SLI calculation per-tenant and your promotion gate will refuse to lift.
- The OTel collector pipeline in production is configured by the substrate team; you do not need to manage it. Locally you use the `otel-collector-config.yaml` provided in `microservices/observability/iac/local-dev/`.
- Cardinality budget: stay under 1000 distinct values per metric label. The collector will enforce per-tenant series caps and drop overflow signals.
- The OpenSLO `alertPolicies` references are resolved by the SLO engine against the substrate-managed multi-window burn-rate policies. You should NOT define your own custom alert policies unless you have an approved exception.
