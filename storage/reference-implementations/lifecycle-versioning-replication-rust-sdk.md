# Reference implementation — Bucket + lifecycle + versioning + cross-region replication via `cloud-storage-sdk`

Runnable Rust program that provisions a WORM-locked bucket, uploads a versioned object, configures lifecycle, enables cross-region
replication, and exercises the bucket via S3 SDK round-trip.

## `Cargo.toml`

```toml
[package]
name = "storage-end-to-end-example"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
aws-config = "1.5"
aws-credential-types = "1.2"
aws-sdk-s3 = "1.55"
cloud-storage-sdk = "0.42.0"
trace = "0.42.0"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1.43", features = ["macros", "rt-multi-thread"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

## `src/main.rs`

```rust
use anyhow::Result;
use cloud_storage_sdk::{
    BucketCreateRequest, LifecycleRule, LifecyclePolicyRequest, ObjectLockMode, ObjectPutRequest,
    ReplicationRequest, StorageClass, StorageClient, StorageConfig, Tenant,
};
use trace::TraceContext;
use std::time::Duration;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let trace = TraceContext::new_root();

    let cfg = StorageConfig::builder()
        .endpoint("https://loopback.cloud-storage.oyatie.local".parse()?)
        .tenant(Tenant::parse("oyatie.b2b.smb.acme-software")?)
        .service_account_credentials_path("/etc/oya/storage/sa-creds.json")
        .request_timeout(Duration::from_secs(30))
        .build()?;

    let client = StorageClient::connect(cfg).await?;
    info!("connected to cloud-storage");

    // 1. Bucket with WORM compliance + versioning
    let bucket = client
        .bucket_create(
            BucketCreateRequest::builder()
                .bucket("acme-financial-records-demo")
                .region("loopback-us-east-2")
                .storage_class(StorageClass::Hot)
                .enable_versioning(true)
                .object_lock_default(ObjectLockMode::Compliance { retention_years: 7 })
                .enable_encryption(true)
                .kms_cmk("acme-financial")
                .enable_inventory_daily_parquet(true)
                .build()?,
            trace.child(),
        )
        .await?;
    info!(bucket_id = %bucket.id(), endpoint_s3 = %bucket.endpoint_s3(), "bucket provisioned");

    // 2. Versioned PUT
    let payload_v1 = serde_json::to_vec(&serde_json::json!({
        "trade_id": "TRD-2026-05-20-001",
        "instrument": "AAPL",
        "price_usd": 248.50
    }))?;
    let put_v1 = client
        .object_put(
            ObjectPutRequest::builder()
                .bucket("acme-financial-records-demo")
                .key("trades/2026/05/20/TRD-001.json")
                .body(payload_v1.clone())
                .content_type("application/json")
                .metadata("regulatory_class", "FINRA-OATS")
                .build()?,
            trace.child(),
        )
        .await?;
    info!(version_id_v1 = %put_v1.version_id(), "put v1");

    // 3. Versioned PUT v2 (correction)
    let payload_v2 = serde_json::to_vec(&serde_json::json!({
        "trade_id": "TRD-2026-05-20-001",
        "instrument": "AAPL",
        "price_usd": 248.55,
        "correction_note": "T+0 fix"
    }))?;
    let put_v2 = client
        .object_put(
            ObjectPutRequest::builder()
                .bucket("acme-financial-records-demo")
                .key("trades/2026/05/20/TRD-001.json")
                .body(payload_v2)
                .content_type("application/json")
                .metadata("regulatory_class", "FINRA-OATS")
                .build()?,
            trace.child(),
        )
        .await?;
    info!(version_id_v2 = %put_v2.version_id(), "put v2");

    // 4. Confirm WORM refuses delete on v1
    match client
        .object_delete("acme-financial-records-demo", "trades/2026/05/20/TRD-001.json", Some(put_v1.version_id()), trace.child())
        .await
    {
        Err(e) if e.is_object_lock_protected() => {
            info!("expected object-lock refusal on v1");
        }
        other => panic!("expected ObjectLockProtected; got {:?}", other),
    }

    // 5. Lifecycle policy
    client
        .lifecycle_policy_apply(
            LifecyclePolicyRequest::builder()
                .bucket("acme-financial-records-demo")
                .rule(
                    LifecycleRule::builder()
                        .name("trade-records-lifecycle")
                        .filter_prefix("trades/")
                        .transition_warm_after_days(30)
                        .transition_cold_after_days(180)
                        .transition_archive_after_days(730)
                        .non_current_version_expiration_after_days(2555)
                        .respect_object_lock(true)
                        .build()?,
                )
                .build()?,
            trace.child(),
        )
        .await?;
    info!("lifecycle policy applied");

    // 6. Cross-region replication (dev profile auto-creates the replica bucket)
    client
        .replication_enable(
            ReplicationRequest::builder()
                .source_bucket("acme-financial-records-demo")
                .source_region("loopback-us-east-2")
                .target_bucket("acme-financial-records-demo-eu")
                .target_region("loopback-eu-west-1")
                .replicate_versions_all(true)
                .replicate_deletes(false)
                .replicate_object_lock(true)
                .target_storage_class_override(Some(StorageClass::Warm))
                .build()?,
            trace.child(),
        )
        .await?;
    info!("cross-region replication enabled");

    // 7. Wait for replication + verify
    tokio::time::sleep(Duration::from_secs(8)).await;
    let head_eu = client
        .object_head_in_region(
            "acme-financial-records-demo-eu",
            "trades/2026/05/20/TRD-001.json",
            Some(put_v2.version_id()),
            "loopback-eu-west-1",
            trace.child(),
        )
        .await?;
    info!(
        eu_storage_class = ?head_eu.storage_class(),
        eu_version_id = %head_eu.version_id(),
        eu_object_lock = ?head_eu.object_lock_mode(),
        "replica verified"
    );

    // 8. Round-trip via standard S3 SDK
    let s3_creds = client
        .s3_credential_issue(Duration::from_secs(3600), /* read_only= */ true, Some("trades/"), trace.child())
        .await?;
    let aws_cfg = aws_config::from_env()
        .endpoint_url(bucket.endpoint_s3())
        .credentials_provider(aws_credential_types::Credentials::new(
            s3_creds.access_key().to_string(),
            s3_creds.secret_key().to_string(),
            Some(s3_creds.session_token().to_string()),
            None,
            "cloud-storage-issued",
        ))
        .load()
        .await;
    let s3 = aws_sdk_s3::Client::new(&aws_cfg);
    let resp = s3
        .get_object()
        .bucket("acme-financial-records-demo")
        .key("trades/2026/05/20/TRD-001.json")
        .send()
        .await?;
    let body = resp.body.collect().await?.to_vec();
    let parsed: serde_json::Value = serde_json::from_slice(&body)?;
    info!(
        s3_round_trip_price_usd = parsed["price_usd"].as_f64(),
        s3_round_trip_correction = %parsed["correction_note"].as_str().unwrap_or(""),
        "s3-api round-trip OK"
    );

    Ok(())
}
```

## Run it

```bash
cargo run --release
```

Expected output (trimmed):
```
INFO  connected to cloud-storage
INFO  bucket provisioned bucket_id=bkt-… endpoint_s3=https://s3.cloud-storage.loopback.oyatie.local
INFO  put v1 version_id_v1=01HKM5K2P7R3D6Q9NTYABCXFEH
INFO  put v2 version_id_v2=01HKM5N8XWY4Z7P3R9SQGCDF2T
INFO  expected object-lock refusal on v1
INFO  lifecycle policy applied
INFO  cross-region replication enabled
INFO  replica verified eu_storage_class=Some(Warm) eu_version_id=01HKM5N8XWY4Z7P3R9SQGCDF2T eu_object_lock=Some(Compliance{retention_years: 7})
INFO  s3-api round-trip OK s3_round_trip_price_usd=Some(248.55) s3_round_trip_correction="T+0 fix"
```

## SDK correctness guarantees

1. `bucket_create(...)` rejects names that collide with existing tenant buckets or invalid S3 naming rules; idempotent on `(tenant, name)`.
2. `object_put(...)` injects AAD `(tenant_id, bucket, key)` automatically; raw API calls without AAD are refused by the server.
3. `object_delete(...)` returns `StorageError::ObjectLockProtected` when retention has not expired; the SDK explicitly distinguishes
   the error variant so callers can branch.
4. `lifecycle_policy_apply(...)` is atomic — all rules apply or none.
5. `replication_enable(...)` creates the target bucket if missing with matching properties (versioning, encryption, object-lock).
6. `s3_credential_issue(...)` issues short-lived AWS-Sig-v4-compatible credentials scoped to the tenant + prefix.

## Tests

```bash
cargo test --features hermetic
```

The `hermetic` feature uses `cloud_storage_sdk::testkit::Hermetic` with an in-process MinIO + cross-region simulator; tests
finish in ≤ 75 s.

## Error budget

`StorageError::GetLatencySloBreached { took_ms }` indicates the GET took longer than the tenant_class SLO. Do not retry — file
`cloud_storage.slo.get_slow`. Persistent breach signals NVMe cache miss pathology; on-call rotation engages.
