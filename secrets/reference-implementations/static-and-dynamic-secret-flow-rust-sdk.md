# Reference implementation — Static + dynamic secret flow with `cloud-secrets-sdk`

Runnable Rust program that authenticates a workload, reads a static secret, issues a 15-minute dynamic Postgres credential,
runs a query, and revokes the lease.

## `Cargo.toml`

```toml
[package]
name = "secrets-flow-example"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
cloud-secrets-sdk = "0.42.0"
trace = "0.42.0"
serde = { version = "1", features = ["derive"] }
tokio = { version = "1.43", features = ["macros", "rt-multi-thread"] }
tokio-postgres = { version = "0.7", features = ["with-tls-rustls", "with-uuid-1"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

## `src/main.rs`

```rust
use anyhow::{Context, Result};
use cloud_secrets_sdk::{
    DynamicLease, SecretsClient, SecretsConfig, StaticReadOptions, Tenant, WorkloadIdentity,
};
use trace::TraceContext;
use std::time::Duration;
use tokio_postgres::NoTls;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let trace = TraceContext::new_root();

    let cfg = SecretsConfig::builder()
        .endpoint("https://loopback.cloud-secrets.oyatie.local".parse()?)
        .tenant(Tenant::parse("oyatie.b2b.smb.acme-software")?)
        .workload_identity(WorkloadIdentity::auto_detect()?) // K8s SA, AppRole, or static
        .request_timeout(Duration::from_secs(5))
        .build()?;

    let client = SecretsClient::connect(cfg).await?;
    info!("connected to cloud-secrets");

    // 1. Static secret read
    let api_key = client
        .read_static(
            "kv/prod/openai-api-key",
            StaticReadOptions::default().with_lease(Duration::from_secs(15 * 60)),
            trace.child(),
        )
        .await
        .context("static read failed")?;
    info!(
        version = api_key.version(),
        lease_expires = %api_key.lease_expires_at(),
        bytes = api_key.value().len(),
        "static secret read"
    );

    // 2. Dynamic Postgres credential issuance
    let lease: DynamicLease = client
        .issue_dynamic(
            "postgres-prod-primary",
            "app-readonly",
            Duration::from_secs(15 * 60),
            trace.child(),
        )
        .await
        .context("dynamic issue failed")?;
    info!(
        lease_id = %lease.id(),
        user = %lease.username(),
        expires = %lease.expires_at(),
        "dynamic credential issued"
    );

    // 3. Use the credential — note the SDK gives you a connection URI directly
    let conn_str = lease.postgres_connection_uri()?;
    let (pg, conn) = tokio_postgres::connect(&conn_str, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            warn!(error = ?e, "postgres conn err");
        }
    });

    let row = pg
        .query_one(
            "SELECT count(*)::bigint AS n FROM orders WHERE created_at > now() - interval '24 hours'",
            &[],
        )
        .await?;
    let count: i64 = row.get("n");
    info!(orders_last_24h = count, "query result");

    // 4. Explicit revoke (best practice; otherwise auto-revokes at lease expiry)
    client
        .revoke_dynamic(lease.id(), trace.child())
        .await
        .context("revoke failed")?;
    info!(lease_id = %lease.id(), "lease revoked");

    Ok(())
}
```

## Run it

```bash
cargo run
```

Expected stdout:
```
INFO  connected to cloud-secrets
INFO  static secret read version=3 lease_expires=2026-05-20T08:46:00Z bytes=51
INFO  dynamic credential issued lease_id=le-… user=dyn_4f8c2b7a9e1d expires=2026-05-20T08:46:00Z
INFO  query result orders_last_24h=4218
INFO  lease revoked lease_id=le-…
```

## SDK correctness guarantees

1. `WorkloadIdentity::auto_detect()` chooses between Kubernetes service-account JWT, AppRole secret-id, AWS IAM signature, or a
   static developer token (dev-cell only); production refuses static tokens.
2. `read_static` returns a `Secret` whose `.value()` is in a `secrecy::Secret<Vec<u8>>` wrapper — `Debug` is redacted, `Display` panics.
3. `issue_dynamic` returns a `DynamicLease` with a typed `username` + `password` and a `postgres_connection_uri()` helper that
   inlines SSL parameters correctly.
4. The SDK auto-renews static-secret leases up to the configured ceiling and re-fetches values when the version on disk changes.
5. `revoke_dynamic` is idempotent on lease ID; double-calls are silent.
6. Every API call carries a `traceparent` propagated from `TraceContext`.

## Production tightenings

- `SecretsConfig::require_attested_workload(true)` — refuse non-attested workloads.
- `SecretsConfig::byok_required(true)` — refuse to read secrets whose backing KEK isn't tenant-owned (paid tenant_class).
- `SecretsConfig::pack_expectation(["soc2-type-ii-v2017"])` — refuse if the tenant doesn't have the named pack active.
- `WorkloadIdentity::kubernetes_with_token_audience("vault.acme.io")` — explicit SA token audience.

## Tests

```bash
cargo test --features hermetic-pg
```

The hermetic test feature launches a containerized Postgres + a single-process `cloud-secrets` loopback cell.
