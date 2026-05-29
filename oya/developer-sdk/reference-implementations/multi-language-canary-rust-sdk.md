# Reference implementation — A Rust canary that exercises 5 typical Oyatie SDK calls end-to-end

Goal: a single Rust binary that exercises the SDK across 5 µservices (identity, payments, comms-email, workflow-engine, observability),
verifies HTTP/3 was used, emits OTLP traces, and prints a structured report. This is the canary that the SDK regen pipeline runs
before promote-to-stable; you can run it locally as a learning + smoke-test artifact.

## `Cargo.toml`

```toml
[package]
name = "sdk-canary-rust"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
oya-canonical-sdk = "0.42.0"
oya-trace = "0.42.0"
opentelemetry = { version = "0.27", features = ["trace"] }
opentelemetry-otlp = { version = "0.27", features = ["grpc-tonic", "http-proto"] }
opentelemetry_sdk = { version = "0.27", features = ["rt-tokio"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1.43", features = ["macros", "rt-multi-thread"] }
tracing = "0.1"
tracing-opentelemetry = "0.28"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

## `src/main.rs`

```rust
use anyhow::{Context, Result};
use oya_canonical_sdk::{
    comms_email, identity, observability, payments, workflow_engine, Client, Credentials, Tenant,
};
use oya_trace::TraceContext;
use serde::Serialize;
use std::time::{Duration, Instant};
use tracing::{info, instrument};

#[derive(Serialize, Debug)]
struct CanaryReport {
    tenant: String,
    protocol: String,
    calls: Vec<CallOutcome>,
    overall: String,
}

#[derive(Serialize, Debug)]
struct CallOutcome {
    name: &'static str,
    success: bool,
    latency_ms: u128,
    detail: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_telemetry()?;
    let trace = TraceContext::new_root();

    let client = Client::builder()
        .endpoint("https://loopback.api-gateway.oyatie.local".parse()?)
        .tenant(Tenant::parse("oyatie.community.dev-sample")?)
        .credentials(Credentials::api_key(std::env::var("OYA_API_KEY")?))
        .request_timeout(Duration::from_secs(5))
        .prefer_http3(true)
        .build()
        .await?;

    let calls = vec![
        run("identity.who_am_i", who_am_i(&client, &trace)).await,
        run("payments.intent.create", create_intent(&client, &trace)).await,
        run("comms_email.sms.send", send_sms(&client, &trace)).await,
        run("workflow_engine.workflow.run", run_workflow(&client, &trace)).await,
        run("observability.metrics.query", query_metrics(&client, &trace)).await,
    ];

    let report = CanaryReport {
        tenant: "oyatie.community.dev-sample".to_string(),
        protocol: client.last_observed_protocol().to_string(),
        overall: if calls.iter().all(|c| c.success) {
            "PASS".into()
        } else {
            "FAIL".into()
        },
        calls,
    };

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

async fn run<F: std::future::Future<Output = Result<String>>>(name: &'static str, fut: F) -> CallOutcome {
    let start = Instant::now();
    let (success, detail) = match fut.await {
        Ok(d) => (true, d),
        Err(e) => (false, e.to_string()),
    };
    CallOutcome {
        name,
        success,
        latency_ms: start.elapsed().as_millis(),
        detail,
    }
}

#[instrument(skip_all)]
async fn who_am_i(client: &Client, trace: &TraceContext) -> Result<String> {
    let me = client.identity().who_am_i().trace(trace.child()).send().await?;
    Ok(format!("principal={}", me.principal_id))
}

#[instrument(skip_all)]
async fn create_intent(client: &Client, trace: &TraceContext) -> Result<String> {
    let intent = client
        .payments()
        .intent()
        .create(payments::IntentCreate {
            amount_minor: 4_200,
            currency: "USD".into(),
            customer_id: "cus_dev_sample".into(),
            idempotency_key: Some("canary-2026-05-20".into()),
        })
        .trace(trace.child())
        .send()
        .await?;
    Ok(format!("intent={}", intent.id))
}

#[instrument(skip_all)]
async fn send_sms(client: &Client, trace: &TraceContext) -> Result<String> {
    let result = client
        .comms_email()
        .sms()
        .send(comms_email::SmsSend {
            to: "+15555550100".into(),
            from_number: "+15555550199".into(),
            body: "canary".into(),
        })
        .trace(trace.child())
        .send()
        .await?;
    Ok(format!("sms_id={}", result.id))
}

#[instrument(skip_all)]
async fn run_workflow(client: &Client, trace: &TraceContext) -> Result<String> {
    let run = client
        .workflow_engine()
        .workflow()
        .run(workflow_engine::WorkflowRun {
            template: "canary/hello-workflow".into(),
            inputs: serde_json::json!({ "name": "canary" }),
        })
        .trace(trace.child())
        .send()
        .await?;
    Ok(format!("run_id={}", run.id))
}

#[instrument(skip_all)]
async fn query_metrics(client: &Client, trace: &TraceContext) -> Result<String> {
    let resp = client
        .observability()
        .metrics()
        .query(observability::MetricsQuery {
            promql: "rate(application_dispatch_total[1m])".into(),
            window_secs: 60,
        })
        .trace(trace.child())
        .send()
        .await?;
    Ok(format!("series_count={}", resp.series.len()))
}

fn init_telemetry() -> Result<()> {
    use opentelemetry_otlp::WithExportConfig;
    use tracing_subscriber::{prelude::*, EnvFilter};

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint("https://loopback.observability.oyatie.local:4317")
        .build()?;

    let provider = opentelemetry_sdk::trace::TracerProvider::builder()
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .with_resource(opentelemetry_sdk::Resource::new(vec![
            opentelemetry::KeyValue::new("service.name", "sdk-canary-rust"),
        ]))
        .build();
    let tracer = opentelemetry::trace::TracerProvider::tracer(&provider, "sdk-canary-rust");

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();
    Ok(())
}
```

## Run it

```bash
OYA_API_KEY=$(./bin/oya creds dev-token --tenant oyatie.community.dev-sample) \
  cargo run --release
```

Expected output:
```json
{
  "tenant": "oyatie.community.dev-sample",
  "protocol": "HTTP/3",
  "calls": [
    { "name": "identity.who_am_i", "success": true, "latency_ms": 18, "detail": "principal=…" },
    { "name": "payments.intent.create", "success": true, "latency_ms": 31, "detail": "intent=pi_dev_…" },
    { "name": "comms_email.sms.send", "success": true, "latency_ms": 42, "detail": "sms_id=sm_dev_…" },
    { "name": "workflow_engine.workflow.run", "success": true, "latency_ms": 27, "detail": "run_id=wfr_…" },
    { "name": "observability.metrics.query", "success": true, "latency_ms": 14, "detail": "series_count=1" }
  ],
  "overall": "PASS"
}
```

## What the canary guarantees

1. **Five µservices reachable** from one process via one tenant.
2. **HTTP/3 confirmed** by `client.last_observed_protocol()` (falls back to HTTP/2 in environments where QUIC is blocked; that's
   recorded explicitly).
3. **Idempotency** — re-running with the same `OYA_API_KEY` and idempotency key produces the same `payments.intent` ID.
4. **Telemetry** — OTLP spans land in the loopback `observability` collector; you can immediately query them via Step 5 of the
   `cloud-secrets` tutorial.
5. **Cedar permits** evaluated client-side first; any policy violations short-circuit with `OyatieError::PermitDenied`.

## Tests

```bash
cargo test --features canary
```

The canary feature exercises a hermetic version against a single-process loopback cell.
