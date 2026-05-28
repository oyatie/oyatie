---
doc_class: ImplementationPlan
microservice: production-planning
status: Accepted
date: 2026-05-20
owner_team: axis-production-planning + axis-erp-parity
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0244, ADR-0252, ADR-0253, ADR-0257, ADR-0263, ADR-0294, ADR-0297, ADR-0314, ADR-0315, ADR-0316]
planned_enforcement_ref: oya-governance-production-planning-doc-set
ip_id: IP-013
journey_ref: j101
journey_slug: j101-multi-tier-supply-chain-formation
sap_submodule: Cross-cut — PP-BD-BOM / PP-BD-RTG / PP-CRP / PP-MRP / PP-SFC adapter surface (DB + outbox + consumer ports)
tenant_class: substrate
persona: platform-engineer
---

# IP-013: Adapter integrations for production-planning

## A. Intent

Implements the adapter (ADR-0105 layer 7) for production-planning. Adapters connect the pure usecase ports (IP-007..IP-012) to **concrete substrates**: PostgreSQL 16 (state + outbox), Valkey 8 (projection cache), Kafka 3.7 (AsyncAPI 3.1.0 transport), gRPC tonic (synchronous RPC), Cedar (policy decisions), HLC clock (ADR-0297), and OpenTelemetry (tracing). This IP replaces the SAP NetWeaver ABAP runtime + dialog process layer with cloud-native equivalents — there is no monolithic ABAP runtime here; adapters are thin and per-port.

### A.1 Why this IP is non-trivial

The adapter is the ONLY layer allowed to call vendor-specific code (sqlx, Valkey-compatible RESP client, rdkafka, tonic). Domain and usecase MUST remain substrate-agnostic. Concretely the adapter must:

1. **Outbox dispatcher** — drain `production_planning.outbox` rows in HLC order, publish to Kafka topic per channel, mark dispatched in same tx as Kafka ack. Failure modes: Kafka unavailable (back-pressure), envelope schema-drift (DLQ), HLC monotonicity violation (alarm).
2. **Postgres tenant pin** — every query carries `WHERE tenant_id = $1`; row-level security enabled per ADR-0244; defence-in-depth at adapter level (not only Cedar).
3. **Projection cache (Valkey)** — read-through cache for capacity-availability queries; cache key includes `policy_bundle_version` so policy bumps auto-invalidate.
4. **Cedar bundle hot-swap** — adapter listens on `cedar.bundle-published.v1` and atomically swaps active bundle without dropping in-flight requests.
5. **OpenTelemetry trace correlation** — every span carries `tenant_id`, `decision_id`, `hlc` so traces are searchable by audit cross-reference.

## B. Acceptance criteria

- **AC-1:** `pg::OrderRepositoryPg` implements `OrderRepository` port from IP-011; all queries tenant-pinned at SQL level.
- **AC-2:** `pg::CalendarRepositoryPg` implements `CalendarRepository` port from IP-009; calendar reads partition-pruned by `(tenant_id, plant_code)`.
- **AC-3:** `kafka::OutboxDispatcher` drains outbox table at ≥ 500 envelopes/sec/replica; HLC-ordered per channel; failed publish increments `outbox_failed_dispatch_total` metric.
- **AC-4:** `valkey::ProjectionCache` implements `ProjectionCache` port from IP-009; key format `pp:cap:{tenant}:{wc}:{window_hash}:{policy_version}`.
- **AC-5:** `cedar::CedarEvaluatorEngine` loads policy bundle on startup, hot-swaps on `cedar.bundle-published.v1`; current bundle version exposed as metric `cedar_bundle_version`.
- **AC-6:** `grpc::EngineeringChangeClient` implements `EngineeringChangeLoader` from IP-010; TLS+mTLS+ECH per ADR-0253; HTTP/3 transport.
- **AC-7:** Otel spans for every adapter call: span name `pp.adapter.<port>.<method>`; baggage carries `tenant_id` + `decision_id`.
- **AC-8:** Migration scripts (sqlx) for all tables; idempotent on re-run; checksum verified at startup.
- **AC-9:** Outbox table includes `event_id UUID` (idempotency at consumer side) + `hlc TEXT` + `dispatched_at TIMESTAMPTZ NULL`.
- **AC-10:** Configurable connection pools per (tenant cluster pack); per-tenant max_connections cap to prevent noisy neighbour.

## C. Verification

```bash
cargo test -p oya-production-planning-adapter -- pg_save_load_roundtrip
cargo test -p oya-production-planning-adapter -- pg_tenant_pin_enforced
cargo test -p oya-production-planning-adapter -- outbox_dispatch_hlc_order
cargo test -p oya-production-planning-adapter -- outbox_dispatch_kafka_retry
cargo test -p oya-production-planning-adapter -- valkey_cache_invalidation_on_policy_bump
cargo test -p oya-production-planning-adapter -- cedar_bundle_hot_swap
cargo test -p oya-production-planning-adapter -- grpc_ecn_client_tls_ech
cargo test -p oya-production-planning-adapter -- otel_span_baggage_propagation
cargo test -p oya-production-planning-adapter -- migrations_idempotent_rerun
cargo test -p oya-production-planning-adapter -- per_tenant_pool_cap
cargo test -p oya-production-planning-adapter --features integration -- pg_concurrent_save_serialisable
```

## D. Detailed mechanics

### D-1. Data model (PostgreSQL 16)

```sql
-- bom-revision
CREATE TABLE production_planning.bom_revision (
    tenant_id              TEXT NOT NULL,
    bom_id                 TEXT NOT NULL,
    material_id            TEXT NOT NULL,
    plant_code             TEXT NOT NULL,
    version                INTEGER NOT NULL,
    valid_from             TIMESTAMPTZ NOT NULL,
    valid_to               TIMESTAMPTZ NOT NULL,
    components             JSONB NOT NULL,
    ecn_id                 TEXT NOT NULL,
    state                  TEXT NOT NULL CHECK (state IN ('draft','active','retired')),
    hlc                    TEXT NOT NULL,
    decision_id            UUID NOT NULL,
    created_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, bom_id, version)
) PARTITION BY HASH (tenant_id);
ALTER TABLE production_planning.bom_revision ENABLE ROW LEVEL SECURITY;

-- routing
CREATE TABLE production_planning.routing (
    tenant_id              TEXT NOT NULL,
    routing_group          TEXT NOT NULL,
    alternative_id         TEXT NOT NULL,
    version                INTEGER NOT NULL,
    plant_code             TEXT NOT NULL,
    valid_from             TIMESTAMPTZ NOT NULL,
    valid_to               TIMESTAMPTZ NOT NULL,
    lot_size_from          NUMERIC(18,4) NOT NULL,
    lot_size_to            NUMERIC(18,4) NOT NULL,
    priority               INTEGER NOT NULL,
    steps                  JSONB NOT NULL,
    ecn_id                 TEXT NOT NULL,
    hlc                    TEXT NOT NULL,
    decision_id            UUID NOT NULL,
    PRIMARY KEY (tenant_id, routing_group, alternative_id, version)
) PARTITION BY HASH (tenant_id);
ALTER TABLE production_planning.routing ENABLE ROW LEVEL SECURITY;

-- production-order
CREATE TABLE production_planning.production_order (
    tenant_id              TEXT NOT NULL,
    order_id               TEXT NOT NULL,
    state                  TEXT NOT NULL,
    target_material        TEXT NOT NULL,
    target_qty             NUMERIC(18,4) NOT NULL,
    confirmed_qty          NUMERIC(18,4) NOT NULL DEFAULT 0,
    routing_group          TEXT NOT NULL,
    routing_alternative    TEXT NOT NULL,
    bom_id                 TEXT NOT NULL,
    bom_version            INTEGER NOT NULL,
    work_center_plan       JSONB NOT NULL,
    reservation_id         TEXT,
    pegged_demand_keys     JSONB NOT NULL DEFAULT '[]',
    backflush              BOOLEAN NOT NULL DEFAULT FALSE,
    planned_finish         TIMESTAMPTZ NOT NULL,
    hlc                    TEXT NOT NULL,
    last_decision_id       UUID NOT NULL,
    PRIMARY KEY (tenant_id, order_id)
) PARTITION BY HASH (tenant_id);
ALTER TABLE production_planning.production_order ENABLE ROW LEVEL SECURITY;

-- operation-confirmation (idempotency-keyed)
CREATE TABLE production_planning.operation_confirmation (
    tenant_id              TEXT NOT NULL,
    order_id               TEXT NOT NULL,
    operation_no            INTEGER NOT NULL,
    confirm_counter         INTEGER NOT NULL,
    confirmed_qty           NUMERIC(18,4) NOT NULL,
    yield_good              NUMERIC(18,4) NOT NULL,
    yield_scrap             NUMERIC(18,4) NOT NULL,
    decision_id             UUID NOT NULL,
    hlc                     TEXT NOT NULL,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, order_id, operation_no, confirm_counter)
) PARTITION BY HASH (tenant_id);

-- outbox
CREATE TABLE production_planning.outbox (
    event_id                UUID PRIMARY KEY,
    tenant_id               TEXT NOT NULL,
    channel                 TEXT NOT NULL,
    payload                 JSONB NOT NULL,
    hlc                     TEXT NOT NULL,
    decision_id             UUID NOT NULL,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    dispatched_at           TIMESTAMPTZ
) PARTITION BY RANGE (created_at);

CREATE INDEX outbox_undispatched_idx
    ON production_planning.outbox (channel, hlc)
    WHERE dispatched_at IS NULL;
```

### D-2. Rust types

```rust
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ProductionOrderRow {
    pub tenant_id: String,
    pub order_id: String,
    pub state: String,
    pub target_material: String,
    pub target_qty: rust_decimal::Decimal,
    pub confirmed_qty: rust_decimal::Decimal,
    pub routing_group: String,
    pub routing_alternative: String,
    pub bom_id: String,
    pub bom_version: i32,
    pub work_center_plan: sqlx::types::Json<WorkCenterPlan>,
    pub reservation_id: Option<String>,
    pub pegged_demand_keys: sqlx::types::Json<Vec<PeggedDemandKey>>,
    pub backflush: bool,
    pub planned_finish: chrono::DateTime<chrono::Utc>,
    pub hlc: String,
    pub last_decision_id: uuid::Uuid,
}

impl TryFrom<ProductionOrderRow> for ProductionOrder { /* … */ }
impl From<&ProductionOrder> for ProductionOrderRow { /* … */ }
```

### D-3. Outbox dispatcher (Kafka)

```rust
pub struct OutboxDispatcher {
    pool: PgPool,
    producer: FutureProducer,
    metrics: Arc<DispatcherMetrics>,
}

impl OutboxDispatcher {
    pub async fn run(&self, shutdown: CancellationToken) -> anyhow::Result<()> {
        let mut interval = tokio::time::interval(Duration::from_millis(50));
        loop {
            tokio::select! { _ = shutdown.cancelled() => break, _ = interval.tick() => () }
            let batch = sqlx::query_as!(OutboxRow,
                r#"SELECT event_id, tenant_id, channel, payload, hlc, decision_id
                   FROM production_planning.outbox
                   WHERE dispatched_at IS NULL
                   ORDER BY hlc ASC
                   LIMIT 500
                   FOR UPDATE SKIP LOCKED"#)
                .fetch_all(&self.pool).await?;
            for row in batch {
                let record = FutureRecord::to(&row.channel)
                    .key(&row.tenant_id)
                    .headers(otel_headers(&row))
                    .payload(&serde_json::to_vec(&row.payload)?);
                match self.producer.send(record, Duration::from_secs(5)).await {
                    Ok(_) => {
                        sqlx::query!("UPDATE production_planning.outbox SET dispatched_at = now()
                                       WHERE event_id = $1", row.event_id)
                            .execute(&self.pool).await?;
                        self.metrics.dispatched.inc();
                    }
                    Err((e, _)) => {
                        self.metrics.failed.inc();
                        tracing::warn!(error=?e, event_id=?row.event_id, "kafka dispatch failed");
                    }
                }
            }
        }
        Ok(())
    }
}
```

### D-4. Valkey projection cache

```rust
pub struct ValkeyProjectionCache { pool: bb8::Pool<ValkeyConnectionManager>, policy_version: ArcSwap<String> }

#[async_trait]
impl ProjectionCache for ValkeyProjectionCache {
    async fn get(&self, key: &ProjectionCacheKey) -> Result<Option<CachedProjection>, CacheError> {
        let mut conn = self.pool.get().await?;
        let stored: Option<Vec<u8>> = conn.get(self.key(key)).await?;
        Ok(stored.map(|b| rmp_serde::from_slice(&b)).transpose()?)
    }
    async fn set(&self, key: &ProjectionCacheKey, intervals: &[CapacityInterval]) -> Result<(), CacheError> {
        let mut conn = self.pool.get().await?;
        let blob = rmp_serde::to_vec(intervals)?;
        conn.set_ex::<_,_,()>(self.key(key), blob, 300).await?;
        Ok(())
    }
    async fn invalidate(&self, wc: &WorkCenterId) -> Result<(), CacheError> {
        let mut conn = self.pool.get().await?;
        let pattern = format!("pp:cap:*:{}:*:*", wc);
        let mut iter = conn.scan_match::<_, String>(&pattern).await?;
        while let Some(k) = iter.next_item().await { let _: () = conn.del(k).await?; }
        Ok(())
    }
}

impl ValkeyProjectionCache {
    fn key(&self, k: &ProjectionCacheKey) -> String {
        format!("pp:cap:{}:{}:{}:{}", k.tenant, k.work_center, k.window.hash(), self.policy_version.load())
    }
}
```

### D-5. Cedar adapter

```rust
pub struct CedarEvaluatorEngine {
    bundle: ArcSwap<PolicySet>,
    schema: cedar_policy::Schema,
    bundle_consumer: KafkaConsumer,
    metrics: Arc<CedarMetrics>,
}

#[async_trait]
impl CedarEvaluator for CedarEvaluatorEngine {
    async fn evaluate(&self, req: CedarRequest) -> Result<CedarDecision, CedarError> {
        let auth = Authorizer::new();
        let ps = self.bundle.load();
        let resp = auth.is_authorized(&req.into_cedar()?, &ps, &req.entities()?);
        Ok(CedarDecision::from_response(resp))
    }
}

impl CedarEvaluatorEngine {
    pub async fn run_bundle_listener(&self, shutdown: CancellationToken) -> anyhow::Result<()> {
        while !shutdown.is_cancelled() {
            let msg = self.bundle_consumer.recv().await?;
            let envelope: BundlePublished = serde_json::from_slice(msg.payload())?;
            let ps = PolicySet::from_str(&envelope.policy_text)?;
            self.bundle.store(Arc::new(ps));
            self.metrics.bundle_version.set(envelope.version as f64);
        }
        Ok(())
    }
}
```

### D-6. gRPC client (HTTP/3 + QUIC + ECH per ADR-0253)

```rust
pub struct EngineeringChangeClient { client: engineering_change_grpc::EngineeringChangeClient<Channel> }

impl EngineeringChangeClient {
    pub async fn connect(endpoint: &str, tls: &TlsConfig) -> Result<Self, ConnError> {
        let channel = Channel::from_static(endpoint)
            .tls_config(tls.client_config()?)?
            .http2_keep_alive_interval(Duration::from_secs(30))
            // HTTP/3 + QUIC enabled via custom connector (omitted for brevity)
            .connect().await?;
        Ok(Self { client: engineering_change_grpc::EngineeringChangeClient::new(channel) })
    }
}

#[async_trait]
impl EngineeringChangeLoader for EngineeringChangeClient {
    async fn load(&self, tenant: &TenantId, ecn: &EcnId) -> Result<EngineeringChange, EcnError> {
        let req = tonic::Request::new(LoadEcnRequest {
            tenant_id: tenant.to_string(),
            ecn_id: ecn.to_string(),
        });
        let resp = self.client.clone().load_ecn(req).await?;
        Ok(resp.into_inner().try_into()?)
    }
}
```

### D-7. Cedar context wiring

The adapter constructs Cedar requests with full provenance — adapters NEVER strip tenant or HLC fields from the request.

### D-8. OpenTelemetry baggage

```rust
fn otel_headers(row: &OutboxRow) -> OwnedHeaders {
    OwnedHeaders::new()
        .insert(Header { key: "x-oyatie-tenant",      value: Some(row.tenant_id.as_bytes()) })
        .insert(Header { key: "x-oyatie-hlc",          value: Some(row.hlc.as_bytes()) })
        .insert(Header { key: "x-oyatie-decision-id",  value: Some(row.decision_id.as_bytes()) })
        .insert(Header { key: "traceparent",           value: Some(current_traceparent().as_bytes()) })
}
```

### D-9. SLO targets (adapter floor)

| Operation | p50 | p95 | p99 | Rationale |
|---|---|---|---|---|
| `pg::OrderRepositoryPg::save` | 5 ms | 12 ms | 28 ms | Single INSERT…ON CONFLICT inside outer tx. |
| `pg::OrderRepositoryPg::load_for_update` | 3 ms | 8 ms | 18 ms | Hash-partitioned + index lookup; row-lock. |
| `kafka::OutboxDispatcher` per envelope | 4 ms | 9 ms | 22 ms | Kafka producer latency floor. |
| `valkey::ProjectionCache::get` | 0.4 ms | 1.2 ms | 3 ms | Single RTT inside DC. |
| `cedar::CedarEvaluatorEngine::evaluate` | 0.8 ms | 2.5 ms | 6 ms | In-process; bundle size ~5k rules. |

### D-10. Failure modes & recovery

1. **`KafkaUnavailable`** — dispatcher retries with exponential backoff; outbox accumulates; alert at 10k undispatched. Recovery: Kafka restored → automatic drain. Runbook `runbooks/kafka-unavailable.md`.
2. **`PgPoolExhausted`** — usecase calls fail-fast with `RepoError::PoolTimeout`; per-tenant cap triggers fairness. Recovery: scale pool / tighten tenant cap.
3. **`ValkeyFailure`** — cache port returns `CacheError::Unavailable`; usecase degrades to direct repo read. Recovery: Valkey restored → cache repopulates.
4. **`CedarBundleParseError`** — hot-swap rejected; current bundle retained; alert fires. Recovery: bundle publisher republishes corrected bundle.
5. **`OutboxOrphan`** — envelope dispatched but `dispatched_at` write failed. Idempotency on consumer side prevents double-process. Sweep job marks orphans. Runbook `runbooks/outbox-orphan.md`.
6. **`HlcMonotonicityViolation`** — outbox row arrives with HLC < prior dispatched. Alarm fires; dispatcher SKIPS the row and marks `quarantined_at`. Manual recovery via runbook `runbooks/hlc-monotonicity-violation.md`.

### D-11. Migration notes

Source vendor surface: SAP NetWeaver ABAP DBA (Oracle/HANA), SAP Gateway (REST), SAP PI/PO (B2B messaging). Replaced by: Postgres 16, Tonic gRPC, Kafka 3.7. ABAP includes are not portable — IP-013 starts greenfield; lift-shift migration uses the adapter through usecase per ADR-0247.

### D-12. Configuration (TOML)

```toml
[production_planning.adapter.pg]
url = "postgres://pp:***@pg-primary/oyatie?sslmode=verify-full"
max_connections_per_tenant = 32
statement_timeout_ms = 10000

[production_planning.adapter.kafka]
brokers = ["kafka-0:9093", "kafka-1:9093", "kafka-2:9093"]
acks = "all"
compression = "lz4"

[production_planning.adapter.valkey]
endpoints = ["valkey-0:6379","valkey-1:6379"]
pool_size_per_replica = 64

[production_planning.adapter.cedar]
bundle_topic = "cedar.bundle-published.v1"
warmup_path  = "/etc/oyatie/cedar/production-planning.cedar"

[production_planning.adapter.grpc.engineering_change]
endpoint = "https://engineering-change.svc.cluster.local:8443"
tls.ca   = "/etc/oyatie/pki/ca.pem"
tls.cert = "/etc/oyatie/pki/pp.pem"
tls.key  = "/etc/oyatie/pki/pp.key"
ech.public_name = "engineering-change.oyatie"
```

### D-13. Audit-event class registry

| Event class | Severity | Emitter |
|---|---|---|
| `EVT-PRODUCTION_PLANNING-ADAPTER-OUTBOX_DISPATCHED` | informational | dispatcher |
| `EVT-PRODUCTION_PLANNING-ADAPTER-OUTBOX_FAILED` | warning | dispatcher |
| `EVT-PRODUCTION_PLANNING-ADAPTER-CEDAR_BUNDLE_HOT_SWAPPED` | informational | cedar |
| `EVT-PRODUCTION_PLANNING-ADAPTER-CEDAR_BUNDLE_REJECTED` | warning | cedar |
| `EVT-PRODUCTION_PLANNING-ADAPTER-HLC_MONOTONICITY_VIOLATION` | warning | dispatcher |

### D-14. Cross-µservice handoffs

| Direction | Counterparty | Channel |
|---|---|---|
| outbound | all consumers | Kafka topics enumerated in IP-011/IP-012 |
| inbound  | `engineering-change` | gRPC |
| inbound  | `material-master`    | gRPC |
| inbound  | `policy-substrate` (Cedar bundles) | AsyncAPI `cedar.bundle-published.v1` |
| outbound | `audit-substrate`    | AsyncAPI `audit-events.v1` |
| outbound | `observability`      | OTLP traces + metrics |

## E. Failure-mode summary

See D-10. Each scenario maps to a runbook under `runbooks/`.

## F. Migration / rollback

Per-port feature flags: `pp_adapter_pg_v1`, `pp_adapter_kafka_v1`, `pp_adapter_valkey_v1`. Disable individual ports while leaving usecase functional with degraded fallback paths (D-10).

## G. References

- ADR-0105, ADR-0244, ADR-0252, ADR-0253, ADR-0263, ADR-0294, ADR-0297.
- PostgreSQL 16 docs (partitioning, RLS, FOR UPDATE SKIP LOCKED).
- Apache Kafka 3.7 producer semantics + idempotence (`enable.idempotence=true` per ADR-0294).
- Cedar policy language 4.x (Amazon AWS open-source).
- Benchmarks: SAP HANA + NetWeaver runtime | Oracle Fusion ESS/OSS adapters | Microsoft Dataverse data platform | NetSuite SuiteScript adapter surface.

## H. Out of scope

- Domain (IP-001..IP-006), usecase (IP-007..IP-012), REST/gRPC ingress (IP-014), integration tests (IP-015), DDMRP buffer logic (IP-018), MES (IP-024).

— end IP-013 —
