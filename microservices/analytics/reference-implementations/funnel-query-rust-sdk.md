---
doc_class: ReferenceImplementation
microservice: analytics
language: Rust + SQL
date: 2026-05-20
doc_status: published
---

# Reference implementation — Funnel query via the analytics Rust SDK

A runnable example that issues a 5-stage funnel query against an oyatie analytics cell using the `oya-analytics-client` crate (target API; once it lands per IP-007 + IP-008). Until the SDK lands, this reference doubles as the contract specification for the SDK and the SQL-direct equivalent.

## Cargo.toml

```toml
[package]
name = "funnel-example"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-analytics-client = { path = "../../crates/oya-analytics-client" }
oya-cedar-client = { path = "../../crates/oya-cedar-client" }
tokio = { version = "1.40", features = ["rt-multi-thread", "macros"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
chrono = { version = "0.4", features = ["serde"] }
```

## src/main.rs

```rust
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use oya_analytics_client::{AnalyticsClient, AnalyticsClientConfig, Query, QueryParam};
use oya_cedar_client::CedarPrincipal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct FunnelRow {
    stage: u8,
    users_reached: u64,
    conversion_pct: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Construct an analytics client bound to a Cedar principal.
    //    The principal carries the tenant_id + role; the gateway enforces
    //    `analytics::tenant_db::select` on every query.
    let principal = CedarPrincipal::from_env("ANALYTICS_PRINCIPAL_JWT")?;
    let config = AnalyticsClientConfig {
        cell_endpoint: std::env::var("ANALYTICS_CELL")?,
        tenant_id: std::env::var("OYA_TENANT_ID")?,
        principal,
        request_timeout: std::time::Duration::from_secs(30),
        max_retries: 2,
    };
    let client = AnalyticsClient::connect(config).await?;

    // 2. Build the funnel query. The SDK exposes a typed builder for the
    //    common ClickHouse aggregates we wrap (windowFunnel, retention,
    //    uniqExact, etc).
    let now = Utc::now();
    let seven_days_ago = now - Duration::days(7);

    let query = Query::builder()
        .name("workflow_funnel_7d")
        .sql(r#"
            SELECT
                stage,
                users_reached,
                round(users_reached * 100.0 / first_value(users_reached) OVER (), 2) AS conversion_pct
            FROM (
                WITH funnel AS (
                    SELECT
                        windowFunnel(86400)(
                            event_time,
                            event_name = 'workflow_started',
                            event_name = 'form_submitted',
                            event_name = 'payment_intent_created',
                            event_name = 'payment_succeeded',
                            event_name = 'workflow_completed'
                        ) AS level
                    FROM {{tenant_db}}.workflow_event
                    WHERE event_time >= {{start}} AND event_time < {{end}}
                    GROUP BY user_id
                )
                SELECT 1 AS stage, sumIf(1, level >= 1) AS users_reached FROM funnel UNION ALL
                SELECT 2, sumIf(1, level >= 2) FROM funnel UNION ALL
                SELECT 3, sumIf(1, level >= 3) FROM funnel UNION ALL
                SELECT 4, sumIf(1, level >= 4) FROM funnel UNION ALL
                SELECT 5, sumIf(1, level >= 5) FROM funnel
            )
            ORDER BY stage
        "#)
        .param("start", QueryParam::Timestamp(seven_days_ago))
        .param("end", QueryParam::Timestamp(now))
        .build()?;

    // 3. Execute the query. The SDK handles:
    //    - Per-tenant database resolution (`{{tenant_db}}` → `tenant_<tenant_id>`).
    //    - Cedar pre-flight (rejected here before bytes hit the wire if denied).
    //    - Quota check + automatic retry-with-backoff on `quota_exceeded` (up to max_retries).
    //    - Audit-chain emission (one event per query).
    let rows: Vec<FunnelRow> = client.execute_typed::<FunnelRow>(&query).await?;

    // 4. Render the funnel.
    println!("Funnel — last 7 days");
    println!("{:>5} | {:>16} | {:>14}", "stage", "users_reached", "conversion_pct");
    for row in &rows {
        println!(
            "{:>5} | {:>16} | {:>13}%",
            row.stage, row.users_reached, row.conversion_pct
        );
    }

    Ok(())
}
```

## Expected output (against a paid tenant_class cell with 50 k synthetic users emitted per `tutorials/build-funnel-query.md`):

```
Funnel — last 7 days
stage |    users_reached |  conversion_pct
    1 |            50000 |          100.0%
    2 |            42500 |           85.0%
    3 |            35000 |           70.0%
    4 |            27500 |           55.0%
    5 |            24000 |           48.0%
```

## Audit chain emission

After `client.execute_typed()` returns, an `analytics_query_executed` event lands in the audit chain:

```json
{
  "event_class": "analytics_query_executed",
  "tenant_id": "drill-acme",
  "principal_id": "u-12345",
  "query_name": "workflow_funnel_7d",
  "query_hash": "sha256:a1b2c3d4...",
  "duration_ms": 84,
  "read_rows": 178500,
  "read_bytes": 42_300_000,
  "result_rows": 5,
  "cedar_decision": "allow",
  "cedar_policy_id": "analytics-tenant-select-v1",
  "signature": "ed25519:..."
}
```

## Direct SQL alternative (until the SDK lands)

Until `oya-analytics-client` ships (IP-008), the same query can be issued via the HTTP gateway:

```sh
curl -X POST https://analytics.drill-syd-1.oyatie.local/v1/query \
    -H "Authorization: Bearer $ANALYTICS_PRINCIPAL_JWT" \
    -H "X-Oya-Tenant-Id: drill-acme" \
    -H "Content-Type: application/json" \
    -d '{
        "name": "workflow_funnel_7d",
        "sql": "SELECT ...",
        "params": {
            "start": "2026-05-13T00:00:00Z",
            "end":   "2026-05-20T00:00:00Z"
        }
    }'
```

Response shape:

```json
{
  "rows": [
    {"stage": 1, "users_reached": 50000, "conversion_pct": 100.0},
    {"stage": 2, "users_reached": 42500, "conversion_pct": 85.0},
    {"stage": 3, "users_reached": 35000, "conversion_pct": 70.0},
    {"stage": 4, "users_reached": 27500, "conversion_pct": 55.0},
    {"stage": 5, "users_reached": 24000, "conversion_pct": 48.0}
  ],
  "stats": {
    "duration_ms": 84,
    "read_rows": 178500,
    "read_bytes": 42300000
  }
}
```

## Error handling — what to retry

The SDK distinguishes retryable from fatal errors:

| Error class | Retry? | Action |
|---|---|---|
| `cedar_denied` | No | The principal lacks the permission. Fix at IAM, not at runtime. |
| `quota_exceeded` | Yes (auto, exponential backoff) | The tenant has burnt its hourly quota. Backoff and retry; surface to user if all retries fail. |
| `query_timeout` | No (would just timeout again) | The query is pathological; the tenant should refactor (MV, narrower window, indexed predicate). |
| `cell_unavailable` | Yes (with circuit-breaker) | The cell is down; the SDK fails the request after 3 retries and the circuit-breaker opens for 30 s. |
| `schema_mismatch` | No | The query references a table/column that doesn't exist in this tenant's database. Fix at the query level. |

## Where this file lives in the µservice

`microservices/analytics/reference-implementations/funnel-query-rust-sdk.md` (this file).

The runnable Cargo project will land at `microservices/analytics/reference-implementations/funnel-example/` once IP-007 + IP-008 land the SDK. Until then, this file is the contract; CI's `analytics-reference-impl-compiles` lane is a stub waiting for the SDK.
