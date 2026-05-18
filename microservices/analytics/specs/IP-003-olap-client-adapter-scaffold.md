# IP-003 — OLAP Client Adapter Scaffold

**Phase:** PHASE-01-ANALYTICS-OLAP-BOOTSTRAP
**Owner:** backend (council-analytics)
**Authority ADRs:** ADR-0193, ADR-0083 layer enum, ADR-0145 inter-µservice communication
**Status:** Planned

## Scope

Author the ClickHouse adapter crate that implements the `oya-shared-olap-client-kernel::OlapClient` trait against a real ClickHouse cluster. The kernel was authored in the data-substrate batch; this IP wires the adapter.

The adapter is `shared` (per BNF v4.1: `oya-shared-olap-clickhouse-adapter`) because every µservice consuming OLAP shares this binding. The adapter encodes per-tenant qualified naming (`tenant_${tid}.${table}`), Cedar-policy-aware cross-tenant denial at the kernel boundary, error mapping, and connection-pool reuse.

## Deliverables

1. Crate `crates/oya-shared-olap-clickhouse-adapter/` (Rust; binds to the official `clickhouse` crate v0.14.x).
2. `ClickHouseOlapClient` impl of `OlapClient` (14 trait methods).
3. HTTP fallback adapter for resilience when the Native protocol version drifts.
4. Per-tenant SQL renderer — typed query DSL → ClickHouse SQL with `{name:Type}` bound parameters.
5. Integration test against an ephemeral ClickHouse instance.
6. Adapter-layer error mapping → `KernelError::AdapterError(...)`.
7. Connection-pool with reuse-rate ≥ 95% at steady state.
8. OpenTelemetry instrumentation per ADR-0151.

## Acceptance criteria

- All 14 trait methods implemented (`ensure_tenant_database`, `ensure_table`, `ensure_materialized_view`, `apply_quota`, `insert`, `query`, `query_streaming`, `drop_tenant_database`, `assert_same_tenant`, `exec_ddl`, `list_databases`, `health_check`, `version`, `current_user_tenant`).
- Round-trip integration test: insert 10K rows + COUNT(*) + filtered SELECT + MV registration + DROP database all pass.
- Cross-tenant query attempt returns `KernelError::CrossTenantAccessDenied` at the adapter layer before SQL is dispatched.
- ClickHouse-side QUOTA exceedance returns `KernelError::AdapterError("quota_exceeded: ...")` with the engine's reason embedded.
- Connection-pool reuse rate ≥ 95% at sustained 100 qps test load.
- Adapter round-trip latency overhead ≤ 5ms p99 above ClickHouse-native latency.

## Implementation tasks

### T1 — Cargo dependencies

File: `crates/oya-shared-olap-clickhouse-adapter/Cargo.toml`

```toml
[package]
name = "oya-shared-olap-clickhouse-adapter"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-shared-olap-client-kernel = { path = "../oya-shared-olap-client-kernel" }
clickhouse = { version = "0.14", features = ["lz4", "uuid", "time"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros", "time"] }
serde = { workspace = true, features = ["derive"] }
async-trait = "0.1"
tracing = { workspace = true }
thiserror = { workspace = true }
url = "2"
hex = "0.4"
sha2 = "0.10"

[dev-dependencies]
testcontainers = "0.20"
tokio-test = "0.4"
```

### T2 — Adapter struct

File: `crates/oya-shared-olap-clickhouse-adapter/src/lib.rs`

```rust
pub struct ClickHouseOlapClient {
    client: clickhouse::Client,
    config: ClickHouseConfig,
    pool_stats: Arc<PoolStats>,
}

pub struct ClickHouseConfig {
    pub endpoint: String,
    pub default_database: String,
    pub credentials: Credentials,           // resolved via OpenBao SecretReference
    pub max_query_duration_seconds: u64,
    pub cluster_name: Option<String>,       // for DDL ON CLUSTER
    pub max_concurrent_connections: usize,  // pool size
    pub idle_timeout_seconds: u64,
}

impl ClickHouseOlapClient {
    pub async fn connect(cfg: &ClickHouseConfig) -> Result<Self, KernelError> {
        let client = clickhouse::Client::default()
            .with_url(&cfg.endpoint)
            .with_user(&cfg.credentials.username)
            .with_password(&cfg.credentials.password)
            .with_compression(clickhouse::Compression::Lz4)
            .with_option("max_execution_time", &cfg.max_query_duration_seconds.to_string());
        // Verify connection.
        let _: u64 = client.query("SELECT 1").fetch_one().await
            .map_err(KernelError::from)?;
        Ok(Self { client, config: cfg.clone(), pool_stats: Arc::new(PoolStats::default()) })
    }
}
```

### T3 — Trait impl

```rust
#[async_trait]
impl OlapClient for ClickHouseOlapClient {
    async fn ensure_tenant_database(&self, tid: &TenantId) -> Result<(), KernelError> {
        let sql = format!("CREATE DATABASE IF NOT EXISTS tenant_{} ON CLUSTER {}",
            tid.as_str(), self.config.cluster_name.as_deref().unwrap_or("oya-cell"));
        self.exec_ddl_raw(&sql).await
    }

    async fn ensure_table(&self, tid: &TenantId, ddl: &TableDdl) -> Result<(), KernelError> {
        self.assert_same_tenant_for_table(tid, &ddl.qualified_name())?;
        let sql = ddl.render_create_if_not_exists(&self.config.cluster_name);
        self.exec_ddl_raw(&sql).await
    }

    async fn apply_quota(&self, tid: &TenantId, profile: &QuotaProfile) -> Result<(), KernelError> {
        let sql = profile.render_create_quota_sql(tid, &self.config.cluster_name);
        self.exec_ddl_raw(&sql).await
    }

    async fn query<T: serde::de::DeserializeOwned + Send + Sync>(
        &self,
        principal: &Principal,
        q: &ParameterizedQuery,
    ) -> Result<Vec<T>, KernelError> {
        // Cross-tenant guard.
        self.assert_same_tenant_for_query(principal, &q.qualified_tables())?;
        let mut req = self.client.query(&q.sql);
        for (name, value) in q.bound_params() {
            req = match value {
                Value::String(s) => req.bind(name, s),
                Value::Int(i) => req.bind(name, i),
                Value::DateTime(t) => req.bind(name, t),
                // ...
            };
        }
        self.pool_stats.record_request();
        let result: Vec<T> = req.fetch_all().await.map_err(KernelError::from)?;
        Ok(result)
    }

    // ... 11 more methods
}
```

### T4 — SQL renderer

File: `crates/oya-shared-olap-clikhouse-adapter/src/sql_render.rs`

```rust
pub struct ParameterizedQuery {
    pub sql: String,
    pub params: HashMap<String, Value>,
}

impl ParameterizedQuery {
    pub fn new(sql: &str, params: HashMap<String, Value>) -> Self {
        // Validate the SQL has only `{name:Type}` placeholders, no raw interpolation.
        for cap in REGEX_PLACEHOLDER.captures_iter(sql) {
            let name = &cap[1];
            if !params.contains_key(name) {
                panic!("missing parameter binding: {name}");
            }
        }
        Self { sql: sql.into(), params }
    }

    pub fn qualified_tables(&self) -> Vec<QualifiedTable> {
        // Static-analyze the SQL for table references; return a list of (db, table) tuples.
        // Used by cross-tenant guard.
        parse_table_refs(&self.sql)
    }
}
```

The renderer enforces:

- Tables are always qualified as `tenant_${tid}.${table}`.
- Parameters are always emitted as `{name:Type}` placeholders.
- `ON CLUSTER {cluster}` is wrapped for DDL (CREATE / DROP / ALTER) when running against the replicated cluster.
- Raw SQL never enters the renderer; the type system rejects it.

### T5 — Error mapping

File: `crates/oya-shared-olap-clickhouse-adapter/src/error.rs`

```rust
impl From<clickhouse::error::Error> for KernelError {
    fn from(e: clickhouse::error::Error) -> Self {
        let s = e.to_string();
        // ClickHouse exposes the code via the "Code: N" pattern in the error string.
        if let Some(code) = parse_error_code(&s) {
            return match code {
                81  => KernelError::AdapterError("database not found".into()),
                60  => KernelError::AdapterError("table not found".into()),
                192 => KernelError::AdapterError("auth failure".into()),
                201 => KernelError::AdapterError(format!("quota_exceeded: {s}")),
                204 => KernelError::AdapterError(format!("read_only_query: {s}")),
                _   => KernelError::AdapterError(s),
            };
        }
        KernelError::AdapterError(s)
    }
}
```

### T6 — Cross-tenant guard

File: `crates/oya-shared-olap-clickhouse-adapter/src/tenant_guard.rs`

```rust
impl ClickHouseOlapClient {
    fn assert_same_tenant_for_query(&self, principal: &Principal, tables: &[QualifiedTable])
        -> Result<(), KernelError>
    {
        for t in tables {
            if t.database.starts_with("tenant_") {
                let table_tid = t.database.trim_start_matches("tenant_");
                if table_tid != principal.tenant_id && !principal.is_internal_admin() {
                    return Err(KernelError::CrossTenantAccessDenied {
                        principal_tid: principal.tenant_id.clone(),
                        attempted_tid: table_tid.into(),
                    });
                }
            }
            if t.database.starts_with("fleet_internal") && !principal.is_internal_admin() {
                return Err(KernelError::CrossTenantAccessDenied {
                    principal_tid: principal.tenant_id.clone(),
                    attempted_tid: "fleet_internal".into(),
                });
            }
        }
        Ok(())
    }
}
```

This is the kernel-layer defense (per ADR-AN-003 §"Defense-in-depth stack"). Cedar at the gateway is the first defense; this is the second.

### T7 — Connection pool

The `clickhouse::Client` is internally a connection pool. We wrap it in our own `PoolStats` to track reuse:

```rust
pub struct PoolStats {
    pub total_requests: AtomicU64,
    pub new_connections: AtomicU64,
}

impl PoolStats {
    pub fn reuse_rate(&self) -> f64 {
        let total = self.total_requests.load(Ordering::Relaxed) as f64;
        let new = self.new_connections.load(Ordering::Relaxed) as f64;
        if total == 0.0 { return 1.0; }
        (total - new) / total
    }
}
```

The reuse-rate is exported as a Prometheus gauge.

### T8 — HTTP fallback

When the Native protocol version drifts (rare; on a ClickHouse minor upgrade), the adapter falls back to HTTP:

```rust
pub async fn query_http_fallback<T: DeserializeOwned>(&self, sql: &str) -> Result<Vec<T>, KernelError> {
    let url = format!("{}?query={}", self.config.endpoint_http(), urlencoding::encode(sql));
    let response = reqwest::get(&url).await.map_err(|e| KernelError::AdapterError(e.to_string()))?;
    parse_tab_separated_with_names(response.text().await?)
}
```

Used only when the Native client returns a protocol-mismatch error.

### T9 — OpenTelemetry instrumentation

Every adapter call emits an OTel span:

```rust
#[tracing::instrument(skip(self, q), fields(
    peer.service = "clickhouse",
    db.name = %q.first_database(),
    oyatie.tenant_id = %principal.tenant_id,
    oyatie.request_id = ?current_request_id(),
))]
async fn query<T>(...) -> Result<...> { ... }
```

### T10 — Integration test

File: `crates/oya-shared-olap-clickhouse-adapter/tests/integration.rs`

```rust
#[tokio::test]
async fn test_round_trip() {
    let cli = testcontainers::clients::Cli::default();
    let ch = cli.run(testcontainers_clickhouse::ClickHouse::default());
    let port = ch.get_host_port_ipv4(9000);
    let adapter = ClickHouseOlapClient::connect(&ClickHouseConfig {
        endpoint: format!("tcp://localhost:{port}"),
        // ...
    }).await.unwrap();

    adapter.ensure_tenant_database(&TenantId::new("test")).await.unwrap();
    adapter.exec_ddl_raw("CREATE TABLE tenant_test.events (id UInt64) ENGINE = MergeTree() ORDER BY id").await.unwrap();
    adapter.exec_ddl_raw("INSERT INTO tenant_test.events SELECT number FROM numbers(10000)").await.unwrap();

    let principal = Principal::tenant("test");
    let rows: Vec<u64> = adapter.query(&principal, &ParameterizedQuery::new(
        "SELECT count() FROM tenant_test.events", HashMap::new()
    )).await.unwrap();
    assert_eq!(rows[0], 10000);
}

#[tokio::test]
async fn test_cross_tenant_denied() {
    let adapter = setup_adapter().await;
    let principal = Principal::tenant("ten_acme");
    let q = ParameterizedQuery::new("SELECT * FROM tenant_ten_bryan.events", HashMap::new());
    let err = adapter.query::<u64>(&principal, &q).await.expect_err("should deny");
    assert!(matches!(err, KernelError::CrossTenantAccessDenied { .. }));
}

#[tokio::test]
async fn test_quota_exceeded_maps_correctly() {
    let adapter = setup_adapter_with_tight_quota().await;
    let principal = Principal::tenant("test");
    for _ in 0..200 {
        let _ = adapter.query::<u64>(&principal, &ParameterizedQuery::new("SELECT 1", HashMap::new())).await;
    }
    let err = adapter.query::<u64>(&principal, &ParameterizedQuery::new("SELECT 1", HashMap::new())).await.expect_err("quota");
    assert!(matches!(err, KernelError::AdapterError(s) if s.contains("quota_exceeded")));
}
```

## Out of scope

- Phase-2 in-house `oya-olap-warehouse-server` adapter (separate phase per ADR-0193 §"In-house roadmap").
- Per-tenant credential isolation at the connection-pool level (cluster admin user is used; tenant scoping is at the DB/role level).

## Failure modes

| Mode | Detection | Mitigation |
|---|---|---|
| Cluster down on connect | adapter init fails | retry with backoff; `/readyz` 503 |
| Protocol version mismatch | error code 51 | HTTP fallback |
| Connection pool exhausted | request times out | pool size tunable; alert |
| Cross-tenant attempt | adapter denies | audit event; 403 |

## SLO commitment (downstream IP-014)

- Adapter round-trip latency overhead ≤ 5ms p99 above native.
- Connection pool reuse ≥ 95% at steady state.

## Rollback

- Adapter is consumed only by the analytics µservice's app + bootstrap controllers.
- Version pin in their `Cargo.toml` allows downgrade.

## Evidence emission

- Per query: OTel span.
- Per cross-tenant denial: `oya.analytics.adapter.cross_tenant_denied.v1` audit event.
- Per HTTP fallback: counter `oya_analytics_adapter_http_fallback_total`.

## References

- ADR-0193 (engine choice).
- ADR-0083 layer enum.
- ADR-AN-003-row-level-tenant-isolation (defense-in-depth).
- oya-shared-olap-client-kernel crate.
- clickhouse-rs docs: https://docs.rs/clickhouse/latest/clickhouse/
