# Reference implementation — Multi-engine workload + PITR restore via `oya-cloud-data-sdk`

Runnable Rust program that provisions an OLTP DB, runs a writes + bounded-stale reads, posts a ledger transfer through
TigerBeetle, takes a snapshot, performs a destructive change, and restores via PITR.

## `Cargo.toml`

```toml
[package]
name = "data-end-to-end-example"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
oya-cloud-data-sdk = "0.42.0"
oya-trace = "0.42.0"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
time = { version = "0.3", features = ["macros"] }
tokio = { version = "1.43", features = ["macros", "rt-multi-thread"] }
tracing = "0.1"
tracing-subscriber = "0.3"
uuid = { version = "1.10", features = ["v4"] }
```

## `src/main.rs`

```rust
use anyhow::Result;
use oya_cloud_data_sdk::{
    ConsistencyLevel, CrdbClient, CrdbConfig, DatabaseCreateRequest, DataClient, Engine,
    LedgerAccount, LedgerClient, LedgerTransfer, PitrRestoreRequest, ReplicationFactor, Tenant,
};
use oya_trace::TraceContext;
use std::time::Duration;
use time::OffsetDateTime;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let trace = TraceContext::new_root();

    let cfg = CrdbConfig::builder()
        .endpoint("https://loopback.cloud-data.oyatie.local".parse()?)
        .tenant(Tenant::parse("oyatie.b2b.smb.acme-software")?)
        .service_account_credentials_path("/etc/oya/data/sa-creds.json")
        .request_timeout(Duration::from_secs(10))
        .build()?;

    let data = DataClient::connect(cfg.clone()).await?;
    info!("connected to cloud-data");

    // 1. Provision a database
    let db = data
        .database_create(
            DatabaseCreateRequest::builder()
                .database("acme-prod")
                .engine(Engine::Cockroachdb25)
                .region("loopback-us-east-2")
                .replication_factor(ReplicationFactor::Three)
                .pitr_window_days(30)
                .build()?,
            trace.child(),
        )
        .await?;
    info!(database_id = %db.id(), endpoint = %db.endpoint(), "database provisioned");

    let crdb: CrdbClient = data.crdb_for(&db).await?;

    // 2. DDL
    crdb.ddl_apply(
        r#"
        CREATE TABLE customers (
          tenant_id TEXT NOT NULL DEFAULT current_setting('oya.tenant_id'),
          id        UUID NOT NULL DEFAULT gen_random_uuid(),
          email     TEXT NOT NULL,
          name      TEXT NOT NULL,
          region    TEXT NOT NULL,
          created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
          PRIMARY KEY (tenant_id, id),
          UNIQUE (tenant_id, email)
        );
        CREATE TABLE orders (
          tenant_id    TEXT NOT NULL DEFAULT current_setting('oya.tenant_id'),
          id           UUID NOT NULL DEFAULT gen_random_uuid(),
          customer_id  UUID NOT NULL,
          amount_cents BIGINT NOT NULL,
          currency     TEXT NOT NULL,
          status       TEXT NOT NULL,
          created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
          PRIMARY KEY (tenant_id, id)
        );
        "#,
        trace.child(),
    )
    .await?;
    info!("schema applied");

    // 3. Strong-consistency writes
    crdb.execute(
        "INSERT INTO customers (email, name, region) VALUES \
         ('jane@example.com', 'Jane Doe', 'us'), \
         ('hans@example.de', 'Hans Müller', 'eu'), \
         ('soo@example.kr', 'Park Soo', 'kr')",
        &[],
        ConsistencyLevel::Strong,
        trace.child(),
    )
    .await?;

    let customers: Vec<(uuid::Uuid, String, String)> = crdb
        .query("SELECT id, email, region FROM customers", &[], ConsistencyLevel::Strong, trace.child())
        .await?;
    for c in &customers {
        crdb.execute(
            "INSERT INTO orders (customer_id, amount_cents, currency, status) VALUES ($1, $2, 'USD', 'completed')",
            &[&c.0, &((uuid::Uuid::new_v4().as_u128() % 100_000) as i64 + 1_000)],
            ConsistencyLevel::Strong,
            trace.child(),
        )
        .await?;
    }
    info!(customer_count = customers.len(), "writes complete");

    // 4. Bounded-stale aggregate
    let agg: Vec<(String, i64, i64)> = crdb
        .query(
            "SELECT region, COUNT(*), COALESCE(SUM(amount_cents), 0) \
             FROM orders JOIN customers ON orders.customer_id = customers.id \
             GROUP BY region",
            &[],
            ConsistencyLevel::BoundedStale(Duration::from_millis(200)),
            trace.child(),
        )
        .await?;
    for row in &agg {
        info!(region = %row.0, count = row.1, gmv_cents = row.2, "bounded-stale aggregate");
    }

    // 5. Ledger via TigerBeetle
    let ledger: LedgerClient = data.ledger_for("acme-payments").await?;
    ledger.account_upsert(LedgerAccount::new(1, /* code= */ 100), trace.child()).await?;
    ledger.account_upsert(LedgerAccount::new(2, /* code= */ 100), trace.child()).await?;
    let transfer = ledger
        .transfer_post(
            LedgerTransfer::new(/* id= */ 100, /* debit_account= */ 1, /* credit_account= */ 2, /* amount= */ 50_000, /* code= */ 200),
            trace.child(),
        )
        .await?;
    info!(
        transfer_id = transfer.id(),
        commit_latency_ms = transfer.commit_latency().as_secs_f64() * 1000.0,
        order_index = transfer.deterministic_order_index(),
        "ledger transfer posted"
    );

    // 6. Snapshot
    let snap_ts = OffsetDateTime::now_utc();
    let snap = data.snapshot_take(&db, "tutorial step 6 marker", trace.child()).await?;
    info!(snapshot_id = %snap.id(), snap_ts = %snap_ts, "snapshot taken");

    // 7. Destructive change
    crdb.execute(
        "DELETE FROM orders WHERE status = 'completed'",
        &[],
        ConsistencyLevel::Strong,
        trace.child(),
    )
    .await?;
    let remaining: Vec<(i64,)> = crdb
        .query("SELECT COUNT(*) FROM orders", &[], ConsistencyLevel::Strong, trace.child())
        .await?;
    info!(remaining = remaining[0].0, "after destructive delete");

    // 8. PITR restore to a new database
    let restored = data
        .pitr_restore(
            PitrRestoreRequest::builder()
                .source_database("acme-prod")
                .target_database("acme-prod-restored")
                .restore_to(snap_ts)
                .build()?,
            trace.child(),
        )
        .await?;
    info!(restored_id = %restored.id(), "pitr restore complete");

    let restored_crdb: CrdbClient = data.crdb_for(&restored).await?;
    let restored_count: Vec<(i64,)> = restored_crdb
        .query("SELECT COUNT(*) FROM orders", &[], ConsistencyLevel::Strong, trace.child())
        .await?;
    info!(restored_count = restored_count[0].0, "restored database row count");

    Ok(())
}
```

## Run it

```bash
cargo run --release
```

Expected output (trimmed):
```
INFO  connected to cloud-data
INFO  database provisioned database_id=db-… endpoint=pg://acme-prod.cloud-data.loopback.oyatie.local:26257/acme-prod
INFO  schema applied
INFO  writes complete customer_count=3
INFO  bounded-stale aggregate region=us count=1 gmv_cents=45627
INFO  bounded-stale aggregate region=eu count=1 gmv_cents=13845
INFO  bounded-stale aggregate region=kr count=1 gmv_cents=27892
INFO  ledger transfer posted transfer_id=100 commit_latency_ms=0.42 order_index=7421
INFO  snapshot taken snapshot_id=snap-… snap_ts=2026-05-20T15:32:18.214Z
INFO  after destructive delete remaining=0
INFO  pitr restore complete restored_id=db-…
INFO  restored database row count restored_count=3
```

## SDK correctness guarantees

1. `database_create(...)` is idempotent on `(tenant, database)`; replaying returns the existing database.
2. `ddl_apply(...)` is transactional — either the full schema applies or nothing.
3. `execute / query` reject queries that don't include `tenant_id` predicates (enforced by Cedar via Postgres GUC `oya.tenant_id`).
4. `ConsistencyLevel::Strong` uses leader replica; `BoundedStale(d)` uses the closest follower-read with bound `d`.
5. `snapshot_take(...)` is a CRDB consistent backup point; no application-side flush needed.
6. `pitr_restore(...)` creates a new database (cannot overwrite source); promotion requires a separate Cedar permit at paid tenant_class.

## Tests

```bash
cargo test --features hermetic
```

The `hermetic` feature uses `oya_cloud_data_sdk::testkit::Hermetic` with an in-process CRDB cluster + TigerBeetle simulator;
tests finish in ≤ 120 s.

## Error budget

`DataError::ConsistencySloBreached { observed_micros }` indicates a query took longer than the tenant_class SLO; do not retry — file
`cloud_data.slo.query_slow`. Persistent breach signals replica lag or follower-read pathology.
