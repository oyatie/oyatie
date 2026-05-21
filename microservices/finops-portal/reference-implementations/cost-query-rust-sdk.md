---
doc_class: ReferenceImplementation
microservice: finops-portal
language: Rust + Bash
date: 2026-05-20
doc_status: published
---

# Reference implementation — Query cost data + author a dashboard via the finops-portal Rust SDK

A runnable example that:

1. Authenticates as a tenant `finops_admin` principal.
2. Queries the trailing 30 days of cost attributed to a cost center.
3. Renders a forecast for the next 30 days.
4. Lists active anomalies for the tenant.
5. Exports the cost data in FOCUS-spec format.

## Cargo.toml

```toml
[package]
name = "finops-portal-example"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-finops-portal-client = { path = "../../../../crates/oya-finops-portal-client" }
oya-audit-chain-client = { path = "../../../../crates/oya-audit-chain-client" }
oya-cedar-client = { path = "../../../../crates/oya-cedar-client" }
tokio = { version = "1.40", features = ["rt-multi-thread", "macros"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
rust_decimal = "1.36"
chrono = { version = "0.4", features = ["serde"] }
```

## src/main.rs

```rust
use anyhow::Result;
use chrono::{Duration, Utc};
use oya_finops_portal_client::{
    FinopsPortalClient, FinopsPortalConfig,
    CostQuery, CostQueryFilter, CostAttribution,
    ForecastRequest, ForecastHorizon, ForecastModel,
    AnomalyListRequest, AnomalySeverity,
    FocusExportRequest, FocusExportFormat,
};
use oya_cedar_client::CedarPrincipal;
use rust_decimal::Decimal;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Construct the client bound to a finops_admin Cedar principal.
    let principal = CedarPrincipal::from_env("FINOPS_ADMIN_JWT")?;
    let client = FinopsPortalClient::connect(FinopsPortalConfig {
        cell_endpoint: std::env::var("FINOPS_PORTAL_ENDPOINT")?,
        tenant_id: "acme-corp".into(),
        principal,
        request_timeout: std::time::Duration::from_secs(30),
    }).await?;

    // 2. Query trailing 30 days of cost attributed to a cost center.
    let now = Utc::now();
    let thirty_days_ago = now - Duration::days(30);

    let cost_query = CostQuery::builder()
        .since(thirty_days_ago)
        .until(now)
        .filter(CostQueryFilter::CostCenterId("engineering-platform".into()))
        .group_by(vec!["day".into(), "service_name".into()])
        .order_by_desc("daily_cost_usd")
        .limit(100)
        .build()?;

    let cost_rows = client.cost_query(&cost_query).await?;
    println!("Cost attribution for engineering-platform (last 30 d):");
    let mut total = Decimal::ZERO;
    for row in &cost_rows {
        println!(
            "  {} | {} | {} USD",
            row.day, row.service_name, row.daily_cost_usd
        );
        total += row.daily_cost_usd;
    }
    println!("Total trailing 30 d: {} USD", total);

    // 3. Render a forecast for the next 30 days.
    let forecast = client.forecast(ForecastRequest {
        cost_center_id: Some("engineering-platform".into()),
        horizon: ForecastHorizon::Days30,
        model: ForecastModel::AutoSelect,
        confidence_quantiles: vec![10, 50, 90],
    }).await?;
    println!("Forecast for next 30 days:");
    println!(
        "  Selected model: {:?}, MAPE on 30-d holdout: {:.1} %",
        forecast.selected_model, forecast.mape_30d_holdout
    );
    for f in &forecast.forecast {
        println!(
            "  {} | p10={:>8} | p50={:>8} | p90={:>8} USD",
            f.date, f.p10_cost_usd, f.p50_cost_usd, f.p90_cost_usd
        );
    }

    // 4. List active anomalies for the tenant.
    let anomalies = client.anomaly_list(AnomalyListRequest {
        since: Utc::now() - Duration::days(30),
        until: Utc::now(),
        min_severity: Some(AnomalySeverity::Medium),
    }).await?;
    println!("Active anomalies (medium+) in last 30 days:");
    for a in &anomalies {
        println!(
            "  {} | center={} | service={} | severity={:?} | residual={} USD",
            a.detected_at,
            a.cost_center_id.as_deref().unwrap_or("(unallocated)"),
            a.service_name,
            a.severity,
            a.residual_usd
        );
    }

    // 5. Trigger a FOCUS export of last 30 days.
    let export = client.focus_export(FocusExportRequest {
        since: thirty_days_ago,
        until: now,
        format: FocusExportFormat::Parquet,
        s3_output_uri: Some("s3://acme-corp-finops-exports/2026-05-20.parquet".into()),
    }).await?;
    println!(
        "FOCUS export started: job_id={}, status={:?}, eta_seconds={}",
        export.job_id, export.status, export.estimated_seconds
    );

    Ok(())
}
```

## Expected output (against a paid with per_seat billing_component-tenant_class cell with 30 d of synthetic cost data)

```
Cost attribution for engineering-platform (last 30 d):
  2026-05-20 | k8s-compute | 384.50 USD
  2026-05-20 | postgres-managed | 122.40 USD
  2026-05-20 | seaweedfs-s3 | 84.10 USD
  ...
Total trailing 30 d: 18420.32 USD
Forecast for next 30 days:
  Selected model: Ensemble, MAPE on 30-d holdout: 5.9 %
  2026-05-21 | p10=  520.20 | p50=  612.40 | p90=  704.60 USD
  2026-05-22 | p10=  528.10 | p50=  624.80 | p90=  721.50 USD
  ...
Active anomalies (medium+) in last 30 days:
  2026-05-12T08:14:32Z | center=engineering-platform | service=k8s-compute | severity=High | residual=1850.40 USD
  2026-05-18T22:08:14Z | center=sales | service=postgres-managed | severity=Medium | residual=412.20 USD
FOCUS export started: job_id=fex_acme_2026_05_20_001, status=Running, eta_seconds=240
```

## HTTP alternative (curl)

```sh
# Cost query
curl -X POST https://finops-portal.prod-syd-1.oyatie.local/v1/cost/query \
    -H "Authorization: Bearer $FINOPS_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "since": "2026-04-20T00:00:00Z",
        "until": "2026-05-20T00:00:00Z",
        "filter": {"cost_center_id": "engineering-platform"},
        "group_by": ["day", "service_name"],
        "order_by_desc": "daily_cost_usd",
        "limit": 100
    }'

# Forecast
curl -X POST https://finops-portal.prod-syd-1.oyatie.local/v1/forecast \
    -H "Authorization: Bearer $FINOPS_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "cost_center_id": "engineering-platform",
        "horizon": "Days30",
        "model": "AutoSelect",
        "confidence_quantiles": [10, 50, 90]
    }'

# Anomaly list
curl -G https://finops-portal.prod-syd-1.oyatie.local/v1/anomalies \
    -H "Authorization: Bearer $FINOPS_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    --data-urlencode "since=2026-04-20T00:00:00Z" \
    --data-urlencode "until=2026-05-20T00:00:00Z" \
    --data-urlencode "min_severity=Medium"

# FOCUS export
curl -X POST https://finops-portal.prod-syd-1.oyatie.local/v1/exports/focus \
    -H "Authorization: Bearer $FINOPS_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Content-Type: application/json" \
    -d '{
        "since": "2026-04-20T00:00:00Z",
        "until": "2026-05-20T00:00:00Z",
        "format": "Parquet",
        "s3_output_uri": "s3://acme-corp-finops-exports/2026-05-20.parquet"
    }'
```

## Error handling

| Error class | HTTP | Retry? | Action |
|---|---|---|---|
| `cedar_denied` | 403 | No | Principal lacks `finops_portal::dashboard::read` |
| `query_budget_exceeded` | 429 | Yes (auto, backoff) | Daily query budget burnt; backoff |
| `forecast_insufficient_history` | 422 | No | Tenant has < 30 d history; wait for data accumulation |
| `cost_data_ingest_lagging` | 503 | Yes (auto) | Last cost-ingest stale > SLO; transient |
| `export_concurrent_limit` | 429 | Yes (auto) | Max 3 concurrent FOCUS exports per tenant |
| `clickhouse_unavailable` | 503 | Yes (auto, circuit-break) | Backend down; SDK opens circuit-breaker for 30 s |

## Audit-chain events emitted

| Operation | Event class |
|---|---|
| `cost_query` | `finops_portal.cost.queried` |
| `forecast` | `finops_portal.forecast.requested` |
| `anomaly_list` | `finops_portal.anomaly.viewed` |
| `focus_export` (started) | `finops_portal.export.started` |
| `focus_export` (completed) | `finops_portal.export.completed` |
| Budget threshold cross (auto) | `finops_portal.budget_threshold_crossed` |
| Anomaly detected (auto) | `finops_portal.cost_anomaly_detected` |

## Where this file lives

`microservices/finops-portal/reference-implementations/cost-query-rust-sdk.md` (this file). The runnable Cargo project lands at `microservices/finops-portal/reference-implementations/cost-query-example/` once `oya-finops-portal-client` ships.
