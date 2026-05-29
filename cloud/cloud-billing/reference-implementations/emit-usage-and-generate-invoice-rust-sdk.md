# Reference implementation — Emit usage + generate invoice + export FOCUS via `oya-cloud-billing-sdk`

Runnable Rust program that emits a batch of usage events from a fake µservice, triggers a synthetic period close, generates an
invoice, and writes a FOCUS 1.1 export to local disk.

## `Cargo.toml`

```toml
[package]
name = "billing-end-to-end-example"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
oya-cloud-billing-sdk = "0.42.0"
oya-trace = "0.42.0"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
time = { version = "0.3", features = ["macros", "serde-well-known"] }
tokio = { version = "1.43", features = ["macros", "rt-multi-thread"] }
tracing = "0.1"
tracing-subscriber = "0.3"
uuid = { version = "1.10", features = ["v7"] }
```

## `src/main.rs`

```rust
use anyhow::Result;
use oya_cloud_billing_sdk::{
    BillingClient, BillingConfig, FocusFormat, FocusExportRequest, InvoiceMode, MeteringEvent,
    Period, Tenant,
};
use oya_trace::TraceContext;
use std::time::Duration;
use time::{Date, Month, OffsetDateTime};
use tracing::{info, warn};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let trace = TraceContext::new_root();

    let cfg = BillingConfig::builder()
        .endpoint("https://loopback.cloud-billing.oyatie.local".parse()?)
        .tenant(Tenant::parse("oyatie.b2b.smb.acme-software")?)
        .service_account_credentials_path("/etc/oya/billing/sa-creds.json")
        .request_timeout(Duration::from_secs(10))
        .metering_batch_size(500)
        .build()?;

    let client = BillingClient::connect(cfg).await?;
    info!("connected to cloud-billing");

    // 1. Emit 8,000 usage events (5,000 engineering + 3,000 product)
    let now = OffsetDateTime::now_utc();
    let mut batch = Vec::with_capacity(500);
    for i in 0..5_000_u32 {
        batch.push(
            MeteringEvent::builder()
                .event_id(Uuid::now_v7())
                .tenant("oyatie.b2b.smb.acme-software")
                .resource_kind("cloud_compute_k8s.pod_minute")
                .resource_id(format!("ns:acme-engineering/pod:webapp-{i}"))
                .quantity(1.0)
                .unit("minute")
                .timestamp(now)
                .dimension("region", "eu-west-1")
                .dimension("namespace", "ns:acme-engineering")
                .build()?,
        );
        if batch.len() >= 500 {
            client.meter_emit_batch(&batch, trace.child()).await?;
            batch.clear();
        }
    }
    for i in 0..3_000_u32 {
        batch.push(
            MeteringEvent::builder()
                .event_id(Uuid::now_v7())
                .tenant("oyatie.b2b.smb.acme-software")
                .resource_kind("cloud_compute_k8s.pod_minute")
                .resource_id(format!("ns:acme-product-discovery/pod:svc-{i}"))
                .quantity(1.0)
                .unit("minute")
                .timestamp(now)
                .dimension("region", "eu-west-1")
                .dimension("namespace", "ns:acme-product-discovery")
                .build()?,
        );
        if batch.len() >= 500 {
            client.meter_emit_batch(&batch, trace.child()).await?;
            batch.clear();
        }
    }
    if !batch.is_empty() {
        client.meter_emit_batch(&batch, trace.child()).await?;
    }
    info!("emitted 8000 events");

    // 2. Trigger period close (fast-forward, dev profile only)
    let period = Period::new(Date::from_calendar_date(2026, Month::May, 1)?, Date::from_calendar_date(2026, Month::May, 31)?);
    let close = client.period_close(period.clone(), /* fast_forward= */ true, trace.child()).await?;
    info!(
        events_aggregated = close.events_aggregated(),
        passes = close.aggregation_passes(),
        invoice_id = %close.invoice_id(),
        "period close complete"
    );

    // 3. Fetch + log the invoice
    let invoice = client.invoice_get(close.invoice_id(), trace.child()).await?;
    info!(
        currency = %invoice.currency(),
        subtotal = invoice.subtotal(),
        line_item_count = invoice.line_items().len(),
        fx_rate = invoice.fx_rate(),
        "invoice rendered"
    );

    // 4. Promote from shadow → live (dev profile; in prod this is a separate Cedar action)
    client.invoice_promote_live(close.invoice_id(), trace.child()).await?;
    info!("invoice promoted to live mode");

    // 5. FOCUS 1.1 export to local Parquet
    let export = client
        .focus_export(
            FocusExportRequest::builder()
                .period(period)
                .format(FocusFormat::Parquet)
                .destination_local("./focus-2026-05-acme.parquet")
                .build()?,
            trace.child(),
        )
        .await?;
    info!(
        rows = export.rows(),
        bytes = export.bytes(),
        focus_version = %export.focus_version(),
        "focus export complete"
    );

    // 6. Validate
    let validation = client.focus_validate_local("./focus-2026-05-acme.parquet").await?;
    if validation.schema_errors() == 0 {
        info!("focus validation OK");
    } else {
        warn!(errors = ?validation.errors(), "focus validation errors");
    }

    Ok(())
}
```

## Run it

```bash
cargo run --release
```

Expected output (trimmed):
```
INFO  connected to cloud-billing
INFO  emitted 8000 events
INFO  period close complete events_aggregated=8000 passes=3 invoice_id=inv-2026-05-acme-software
INFO  invoice rendered currency=USD subtotal=113.60 line_item_count=2 fx_rate=1.0
INFO  invoice promoted to live mode
INFO  focus export complete rows=8000 bytes=2348541 focus_version=1.1
INFO  focus validation OK
```

## SDK correctness guarantees

1. `meter_emit_batch(...)` requires every event to have `event_id` (UUID v7 enforced) and `tenant_id`. Missing fields refused at
   the client before sending.
2. `meter_emit_batch(...)` is idempotent at the bus — replaying the same batch within 5 min is deduplicated to one ledger row.
3. `period_close(...)` is atomic — either all events aggregate to an invoice or the close rolls back. Partial close is forbidden.
4. `invoice_get(...)` returns a snapshot bound to the issuance timestamp; FX rates are locked at issuance. Subsequent calls return
   the same numbers even if rates change.
5. `invoice_promote_live(...)` is Cedar-gated (`cloud_billing::Action::PromoteInvoiceLive`); refusal is `BillingError::Forbidden`.
6. `focus_export(...)` produces FOCUS 1.1 conformant output; `focus_validate_local(...)` is a strict-mode validator (no leniency).

## Tests

```bash
cargo test --features hermetic
```

The `hermetic` feature uses `oya_cloud_billing_sdk::testkit::Hermetic` to spin a single-process loopback cell with an in-memory
Kafka stub + in-memory rate card; tests finish in ≤ 90 s.

## Error budget

`BillingError::MeteringBusBackpressure` indicates the bus is throttling; retry with exponential backoff (the SDK's default
retry policy handles this). Persistent backpressure is a P2 incident — file `cloud_billing.slo.bus_backpressure`.

`BillingError::AttributionRuleMissing` indicates a usage event landed without matching a rule. Reconfigure attribution rules
before re-running close; do not paper over with a default cost center in code.
