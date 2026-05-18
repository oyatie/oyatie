# IP-015 — App Composition Root + REST/gRPC Adapters

**Phase:** PHASE-01-ANALYTICS-OLAP-BOOTSTRAP
**Owner:** backend (council-analytics)
**Authority ADRs:** ADR-0193, ADR-0083 layer enum, ADR-0090 hyper canonical, ADR-0039 supply chain, ADR-0157 API gateway
**Depends on:** IP-003, IP-007, IP-008, IP-009
**Status:** Planned

## Scope

The analytics µservice's app-layer composition root binary. Wires every piece authored in the prior IPs into a single runnable µservice process. Per ADR-0083, the app layer is the outermost composition layer; it constructs concrete adapters and hands them to the inner kernel/usecase/domain layers via dependency injection.

This IP also delivers the inner layer crates (`domain`, `usecase`, `api`) as new crates per the BNF v4.1 catalog.

## Deliverables

1. Five crates per ADR-0083 layer:
   - `crates/oya-analytics-domain/` (layer: domain).
   - `crates/oya-analytics-usecase/` (layer: usecase).
   - `crates/oya-analytics-api/` (layer: api — REST + GraphQL + gRPC handlers).
   - `crates/oya-analytics-app/` (layer: app — composition root binary).
   - `crates/oya-shared-olap-clickhouse-adapter/` (layer: adapter — already from IP-003).
2. Container image at `ghcr.io/oyatie/analytics:<sha>`.
3. Helm chart `microservices/analytics/iac/helm/analytics-app/` (app deployment) — sibling to the ClickHouse cluster chart.
4. SLSA-L3 build provenance attestation per ADR-0039 (cosign-signed; OIDC-bound).
5. `/healthz` + `/readyz` endpoints per Kubernetes probe canon.
6. OpenTelemetry tracing wired per ADR-0151.
7. Per-route tracing spans + Prometheus metrics.
8. Integration test against a kind-cluster spinup verifying end-to-end flow.

## Acceptance criteria

- Container starts in dev cell, registers with the service mesh (SPIFFE identity), serves `/healthz`.
- REST endpoints from `microservices/analytics/contracts/openapi-v1.yaml` are functional.
- gRPC endpoints from `microservices/analytics/contracts/analytics.proto` are functional.
- GraphQL endpoints from `microservices/analytics/contracts/graphql-v1.sdl` are functional.
- All workspace lints pass under `cargo clippy --workspace -- -D warnings`.
- All workspace tests pass under `cargo test --workspace`.
- Distroless final image; static binary built via `release-musl.yml` pipeline.
- Image is SLSA-L3 cosign-attestable.
- OTel spans visible in Grafana Tempo via the observability µservice.

## Implementation tasks

### T1 — Domain crate

File: `crates/oya-analytics-domain/Cargo.toml`

```toml
[package]
name = "oya-analytics-domain"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true, features = ["derive"] }
chrono = { workspace = true, features = ["serde"] }
uuid = { workspace = true, features = ["v4", "serde"] }
thiserror = { workspace = true }
```

File: `crates/oya-analytics-domain/src/lib.rs` — pure types, no external dependencies.

```rust
pub mod tenant;
pub mod dashboard;
pub mod audit_log;
pub mod regulator_export;
pub mod billing;

// Public re-exports.
pub use tenant::{TenantId, TenantTier, ResidencyClass};
pub use dashboard::{WorkflowExecutionBucket, DashboardQuery};
pub use audit_log::{AuditLogEntry, Axis, AuditLogSearch};
pub use regulator_export::{RegulatorExportRequest, RegulatorExportManifest};
pub use billing::{BillingRollupRow, BillingPeriod};
```

### T2 — Usecase crate

File: `crates/oya-analytics-usecase/Cargo.toml`

```toml
[package]
name = "oya-analytics-usecase"
version = "0.1.0"

[dependencies]
oya-analytics-domain = { path = "../oya-analytics-domain" }
oya-shared-olap-client-kernel = { path = "../oya-shared-olap-client-kernel" }
async-trait = "0.1"
tracing = { workspace = true }
thiserror = { workspace = true }
```

Each use-case is a struct that holds the inner-layer port and implements its `execute` method.

```rust
pub struct GetWorkflowExecutionDashboardUseCase<C: OlapClient> {
    pub olap: C,
}

impl<C: OlapClient> GetWorkflowExecutionDashboardUseCase<C> {
    pub async fn execute(&self, principal: &Principal, query: DashboardQuery)
        -> Result<Page<WorkflowExecutionBucket>, UseCaseError>
    {
        principal.assert_can_view_dashboard(&query.tenant_id)?;
        let rows = self.olap.query_workflow_rollup(&principal.tenant_id, &query).await?;
        Ok(rows.into_page(query.cursor))
    }
}
```

### T3 — API crate

File: `crates/oya-analytics-api/Cargo.toml`

```toml
[package]
name = "oya-analytics-api"
version = "0.1.0"

[dependencies]
oya-analytics-domain = { path = "../oya-analytics-domain" }
oya-analytics-usecase = { path = "../oya-analytics-usecase" }
axum = { workspace = true }
tonic = { workspace = true }
async-graphql = { workspace = true }
async-graphql-axum = { workspace = true }
tracing = { workspace = true }
serde_json = { workspace = true }
```

REST handlers (one per endpoint):

```rust
// crates/oya-analytics-api/src/rest/dashboards.rs
pub async fn get_workflow_execution_dashboard(
    State(app): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<DashboardParams>,
) -> Result<Json<DashboardPage>, ApiError> {
    let principal = extract_principal(&headers)?;
    let query = DashboardQuery::from_params(&principal.tenant_id, params)?;
    let page = app.use_cases.workflow_dashboard.execute(&principal, query).await?;
    Ok(Json(page))
}
```

gRPC handlers (one per RPC):

```rust
// crates/oya-analytics-api/src/grpc/service.rs
#[tonic::async_trait]
impl AnalyticsService for AnalyticsGrpcService {
    async fn get_workflow_execution_dashboard(
        &self,
        request: Request<GetWorkflowExecutionDashboardRequest>,
    ) -> Result<Response<GetWorkflowExecutionDashboardResponse>, Status> {
        let principal = extract_principal_grpc(&request)?;
        let query = DashboardQuery::from_grpc(&principal.tenant_id, request.get_ref())?;
        let page = self.use_cases.workflow_dashboard.execute(&principal, query).await
            .map_err(to_status)?;
        Ok(Response::new(page.into()))
    }
    // ... other RPCs
}
```

GraphQL resolver (one per Query field):

```rust
// crates/oya-analytics-api/src/graphql/schema.rs
#[Object]
impl Query {
    async fn workflow_execution_dashboard(
        &self, ctx: &Context<'_>, from: DateTime<Utc>, to: DateTime<Utc>,
        first: Option<i32>, after: Option<String>,
    ) -> Result<WorkflowExecutionConnection> {
        let app = ctx.data::<AppState>()?;
        let principal = ctx.data::<Principal>()?;
        let query = DashboardQuery::from_graphql(principal, from, to, first, after)?;
        let page = app.use_cases.workflow_dashboard.execute(principal, query).await?;
        Ok(page.into())
    }
}
```

### T4 — App crate (composition root)

File: `crates/oya-analytics-app/Cargo.toml`

```toml
[package]
name = "oya-analytics-app"
version = "0.1.0"

[[bin]]
name = "oya-analytics-app"
path = "src/main.rs"

[dependencies]
oya-analytics-domain = { path = "../oya-analytics-domain" }
oya-analytics-usecase = { path = "../oya-analytics-usecase" }
oya-analytics-api = { path = "../oya-analytics-api" }
oya-shared-olap-clickhouse-adapter = { path = "../oya-shared-olap-clickhouse-adapter" }
oya-shared-olap-client-kernel = { path = "../oya-shared-olap-client-kernel" }
oya-shared-config = { path = "../oya-shared-config" }
oya-shared-observability = { path = "../oya-shared-observability" }
oya-shared-cedar = { path = "../oya-shared-cedar" }
oya-shared-cosign = { path = "../oya-shared-cosign" }
tokio = { workspace = true, features = ["rt-multi-thread", "macros", "signal"] }
axum = { workspace = true }
tonic = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
opentelemetry = { workspace = true }
opentelemetry-otlp = { workspace = true }
```

File: `crates/oya-analytics-app/src/main.rs`

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Load config from env + ConfigMap.
    let cfg = AppConfig::load()?;

    // 2. Initialize tracing per ADR-0151.
    let _otel_guard = init_tracing(&cfg.otel)?;

    // 3. Construct concrete adapters.
    let olap = ClickHouseOlapClient::connect(&cfg.clickhouse).await?;
    let cedar = CedarPolicyEvaluator::load(&cfg.cedar.policy_paths)?;
    let cosign = CosignSigner::from_openbao(&cfg.cosign.openbao_ref).await?;
    let audit_chain = AuditChainPublisher::connect(&cfg.audit_chain).await?;

    // 4. Wire usecases.
    let use_cases = UseCases::new(olap.clone(), cedar.clone(), audit_chain.clone());

    // 5. Construct API state.
    let app_state = AppState { use_cases, cedar, cosign, audit_chain };

    // 6. Start REST + GraphQL on :8080.
    let rest_app = oya_analytics_api::rest::router(app_state.clone());
    let graphql_app = oya_analytics_api::graphql::router(app_state.clone());
    tokio::spawn(serve_http(rest_app.merge(graphql_app), cfg.http.addr));

    // 7. Start gRPC on :9090.
    tokio::spawn(serve_grpc(oya_analytics_api::grpc::service(app_state.clone()), cfg.grpc.addr));

    // 8. Wait for shutdown signal.
    tokio::signal::ctrl_c().await?;
    tracing::info!("shutting down");
    Ok(())
}
```

### T5 — Helm chart

File: `microservices/analytics/iac/helm/analytics-app/Chart.yaml`

```yaml
apiVersion: v2
name: analytics-app
description: Analytics µservice composition root.
version: 0.1.0
appVersion: "0.1.0"
```

File: `microservices/analytics/iac/helm/analytics-app/values.yaml`

```yaml
image:
  repository: ghcr.io/oyatie/analytics
  tag: latest
  pullPolicy: IfNotPresent
replicas: 3
resources:
  requests: { cpu: "500m", memory: "512Mi" }
  limits: { cpu: "2", memory: "2Gi" }
service:
  rest:
    port: 8080
  grpc:
    port: 9090
serviceMonitor:
  enabled: true
otel:
  endpoint: http://otel-collector.observability.svc.cluster.local:4317
```

### T6 — SLSA-L3 attestation

In `.github/workflows/release-analytics.yml`:

```yaml
permissions:
  id-token: write
  contents: read
  attestations: write
jobs:
  build-and-sign:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - run: cargo build --release -p oya-analytics-app
      - run: cosign sign --yes ghcr.io/oyatie/analytics:${{ github.sha }}
      - uses: actions/attest-build-provenance@v1
        with:
          subject-path: target/release/oya-analytics-app
```

### T7 — Integration test (kind cluster spinup)

File: `crates/oya-analytics-app/tests/e2e.rs`

```rust
#[tokio::test]
#[ignore = "requires kind cluster"]
async fn test_e2e_dashboard_query() {
    let cluster = KindCluster::start_with_chart(
        "microservices/analytics/iac/helm/clickhouse-analytics/",
        "microservices/analytics/iac/helm/analytics-app/",
    ).await;
    seed_tenant(&cluster, "test_tenant").await;
    seed_events(&cluster, "test_tenant", 1000).await;

    let response = reqwest::Client::new()
        .get(&format!("{}/v1/dashboards/workflow-execution?from=2026-04-01T00:00:00Z&to=2026-05-01T00:00:00Z", cluster.gateway_url()))
        .bearer_auth(cluster.test_token("test_tenant"))
        .send()
        .await.unwrap();
    assert_eq!(response.status(), 200);
    let page: DashboardPage = response.json().await.unwrap();
    assert!(!page.data.is_empty());
}
```

## Out of scope

- Tenant-facing UI surface (lives in application µservice).
- Per-cell autoscaling (deferred — phase 2).

## Failure modes

| Mode | Detection | Mitigation |
|---|---|---|
| ClickHouse connection lost on startup | `/readyz` returns 503 | k8s waits; pod stays NotReady; alert if > 5 min |
| Cosign key unavailable | startup fails | container restart; alert |
| Cedar policy load fails | startup fails | revert to last-known-good policy via Flux GitOps |
| Mesh sidecar not ready | request fails | mesh fail-open or fail-closed per ADR-0148 |

## SLO commitment (downstream IP-014)

- App availability: 99.95% per cell.
- Cold-start time: < 30 s (container start to `/readyz` 200).

## Rollback

- Per ADR-0159, every µservice is feature-flag-gated. Rollback = unset flag; old behavior persists.
- Helm rollback: `helm rollback analytics-app <revision>`.

## Evidence emission

- Every request emits an OTel span with `peer.service=analytics`.
- Every Cedar denial emits an audit event.
- Container startup emits `oya.analytics.app.started.v1`.

## References

- ADR-0083 layer enum.
- ADR-0090 hyper canonical (Hyper HTTP runtime).
- ADR-0039 supply chain (SLSA-L3).
- ADR-0151 X-Request-Id + OTel tracing.
- ADR-0157 API gateway.
- `microservices/analytics/contracts/openapi-v1.yaml`, `analytics.proto`, `graphql-v1.sdl`.
