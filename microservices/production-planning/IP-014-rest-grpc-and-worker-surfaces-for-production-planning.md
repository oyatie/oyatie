---
doc_class: ImplementationPlan
microservice: production-planning
status: Accepted
date: 2026-05-20
owner_team: axis-production-planning + axis-erp-parity
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0244, ADR-0252, ADR-0253, ADR-0257, ADR-0263, ADR-0294, ADR-0297, ADR-0314, ADR-0315, ADR-0316]
planned_enforcement_ref: oya-governance-production-planning-doc-suite
ip_id: IP-014
journey_ref: j101
journey_slug: j101-multi-tier-supply-chain-formation
sap_submodule: PP-MRP (background job RMMRP000) + PP-SFC (UI surfaces) + PP-PI/PP-DS (planning workers)
tenant_class: substrate
persona: production-planner + integration-engineer
---

# IP-014: REST, gRPC, and worker surfaces for production-planning

## A. Intent

Exposes the production-planning usecases (IP-007..IP-012, IP-018..IP-025) through the three canonical traffic surfaces: **REST (OpenAPI 3.2.0)** for human-facing UIs and partner integrators, **gRPC (proto3)** for high-volume inter-µservice synchronous calls, and **worker (Kafka consumer)** for asynchronous event-driven workloads. Replaces SAP `RFC` (Remote Function Call) + `SAP Gateway` (OData) + ABAP background-job scheduler `SM37`/`SM36`. Oracle Fusion equivalent is OIC/REST adapters + ESS job scheduler; Dynamics 365 SCM uses OData APIs + Batch Jobs framework; NetSuite uses RESTlets + SuiteAnalytics + Scheduled Scripts.

### A.1 Why this IP is non-trivial

Three surfaces, three threat models:

1. **REST surface** — public-internet adjacent (via API gateway), must enforce: OpenAPI schema validation, ETag concurrency control, idempotency-key on writes, JWT auth from tenant IdP, rate-limit per principal, OpenAPI 3.2.0 fidelity tested in CI.
2. **gRPC surface** — internal mesh only (no public ingress), mTLS+ECH (ADR-0253), per-method Cedar context; bidirectional streaming for MRP-run subscription.
3. **Worker surface** — Kafka consumer groups; per-channel concurrency, dead-letter queue (DLQ) routing, idempotent handler contract (event_id dedupe), HLC-ordered processing per partition key.

### A.2 Concrete endpoint inventory

#### REST (OpenAPI 3.2.0 paths)
```
POST   /v1/tenants/{tenant_id}/boms                     -> Upsert BOM revision (IP-007)
POST   /v1/tenants/{tenant_id}/boms/{bom_id}/activate   -> Activate BOM (IP-007)
GET    /v1/tenants/{tenant_id}/boms/{bom_id}            -> Read BOM
POST   /v1/tenants/{tenant_id}/mrp-runs                  -> Trigger MRP run (IP-008)
GET    /v1/tenants/{tenant_id}/mrp-runs/{run_id}         -> Read MRP run status
GET    /v1/tenants/{tenant_id}/calendars/{plant}/availability  -> Capacity query (IP-009)
POST   /v1/tenants/{tenant_id}/routings                  -> Upsert routing (IP-010)
POST   /v1/tenants/{tenant_id}/routings/{key}/publish    -> Publish routing version (IP-010)
POST   /v1/tenants/{tenant_id}/production-orders         -> Create order (IP-011)
POST   /v1/tenants/{tenant_id}/production-orders/{id}/release -> Release (IP-012)
POST   /v1/tenants/{tenant_id}/production-orders/{id}/confirm -> Confirm operation (IP-011)
POST   /v1/tenants/{tenant_id}/production-orders/{id}/cancel  -> Cancel (IP-011)
POST   /v1/tenants/{tenant_id}/ddmrp/buffers             -> Author DDMRP buffer (IP-018)
POST   /v1/tenants/{tenant_id}/sop/plans                 -> Submit S&OP plan (IP-019)
POST   /v1/tenants/{tenant_id}/scheduling/runs           -> Capacity leveling (IP-021)
```

#### gRPC (proto3 services)
```
service ProductionPlanning {
    rpc UpsertBom(UpsertBomRequest)                       returns (UpsertBomResponse);
    rpc TriggerMrpRun(TriggerMrpRunRequest)               returns (TriggerMrpRunResponse);
    rpc SubscribeMrpRun(SubscribeMrpRunRequest)           returns (stream MrpRunEvent);
    rpc GetAvailableCapacity(GetAvailableCapacityRequest) returns (GetAvailableCapacityResponse);
    rpc UpsertRouting(UpsertRoutingRequest)               returns (UpsertRoutingResponse);
    rpc PublishRoutingVersion(PublishRoutingRequest)      returns (PublishRoutingResponse);
    rpc CreateProductionOrder(CreateOrderRequest)         returns (CreateOrderResponse);
    rpc ReleaseProductionOrder(ReleaseOrderRequest)       returns (ReleaseOrderResponse);
    rpc ConfirmOperation(ConfirmOpRequest)                returns (ConfirmOpResponse);
    rpc CancelOrder(CancelOrderRequest)                   returns (CancelOrderResponse);
}
```

#### Workers (Kafka consumers)
```
- downtime-overlay-consumer    -> plant-maintenance.downtime-window.v1 -> IngestDowntimeOverlayUseCase
- quality-hold-consumer        -> quality-management.work-center-hold.v1 -> apply hold
- ecn-published-consumer       -> engineering-change.ecn-published.v1 -> invalidate routing/bom cache
- material-master-changed      -> material-master.changed.v1 -> invalidate BOM/routing
- mrp-run-worker (cron + on-demand) -> drives ExecuteMrpRunUseCase
- ddmrp-recalc-worker (hourly cron) -> drives RecalculateDafUseCase
- s&op-cycle-worker (monthly cron)  -> drives RunSopCycleUseCase
```

## B. Acceptance criteria

- **AC-1:** Every REST path documented in `contracts/openapi/production-planning.yaml` (OpenAPI 3.2.0); CI fails on drift.
- **AC-2:** Every gRPC method in `contracts/proto/production_planning.proto` (proto3); buf-lint clean.
- **AC-3:** Every worker subscribed to its channel; consumer-group naming `pp-{handler}-v1`.
- **AC-4:** REST writes require `Idempotency-Key` header; key persisted in Postgres for 24h; replays return original response.
- **AC-5:** REST reads use `ETag` + `If-None-Match` for cache validation.
- **AC-6:** JWT auth on all REST paths; principal claim `sub` mapped to Cedar principal.
- **AC-7:** Per-principal rate limit: default 100 req/sec; tunable per tenant pack.
- **AC-8:** gRPC mTLS+ECH enforced; non-mTLS connections rejected.
- **AC-9:** Worker handlers dedupe on `event_id`; replay-safe.
- **AC-10:** Cedar evaluation in middleware (axum extractor + tonic interceptor + Kafka middleware) — usecase layer NEVER skipped.
- **AC-11:** OpenTelemetry trace spans for every request; baggage includes `tenant_id`, `decision_id`.
- **AC-12:** Worker DLQ topic `pp.dlq.{handler}.v1` for poisoned messages.

## C. Verification

```bash
cargo test -p oya-production-planning-api -- rest_upsert_bom_happy_path
cargo test -p oya-production-planning-api -- rest_etag_concurrency
cargo test -p oya-production-planning-api -- rest_idempotency_replay
cargo test -p oya-production-planning-api -- rest_rate_limit_429
cargo test -p oya-production-planning-api -- rest_jwt_required_401
cargo test -p oya-production-planning-api -- grpc_mtls_required
cargo test -p oya-production-planning-api -- grpc_streaming_mrp_run
cargo test -p oya-production-planning-api -- worker_dedupe_on_event_id
cargo test -p oya-production-planning-api -- worker_dlq_on_handler_panic
cargo test -p oya-production-planning-api -- openapi_contract_drift_check
cargo test -p oya-production-planning-api -- protobuf_buf_breaking_check
cargo test -p oya-production-planning-api -- otel_baggage_propagated
cargo test -p oya-production-planning-api --features integration -- end_to_end_release_pipeline
```

## D. Detailed mechanics

### D-1. REST handler (axum) example — release production order

```rust
#[derive(Deserialize, OpenApiSchema)]
pub struct ReleaseOrderBody { pub attempt_no: u32, pub reason: Option<String> }

#[derive(Serialize, OpenApiSchema)]
pub struct ReleaseOrderResponseBody {
    pub decision_id: Uuid,
    pub reservation_id: String,
    pub release_hlc: String,
    pub lanes_emitted: Vec<String>,
}

pub async fn release_production_order(
    State(state): State<AppState>,
    TenantPath(tenant_id): TenantPath,
    Path(order_id): Path<String>,
    Cedar(cedar_ctx): Cedar,
    IdempotencyKey(idem_key): IdempotencyKey,
    Json(body): Json<ReleaseOrderBody>,
) -> Result<(StatusCode, Json<ReleaseOrderResponseBody>), ApiError> {
    if let Some(prior) = state.idempotency.replay(&idem_key).await? {
        return Ok((StatusCode::OK, Json(prior)));
    }
    let out = state.release_uc.execute(ReleaseInput {
        tenant_id, order_id, attempt_no: body.attempt_no,
        batch_decision_id: None,
    }).await?;
    let body = ReleaseOrderResponseBody {
        decision_id: out.decision_id, reservation_id: out.reservation_id.to_string(),
        release_hlc: out.release_hlc.to_string(),
        lanes_emitted: out.lanes_emitted.iter().map(|l| l.to_string()).collect(),
    };
    state.idempotency.store(&idem_key, &body).await?;
    Ok((StatusCode::ACCEPTED, Json(body)))
}
```

### D-2. Cedar extractor (axum)

```rust
#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Cedar
where AppState: FromRef<S> {
    type Rejection = ApiError;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jwt = parts.headers.get(AUTHORIZATION).ok_or(ApiError::Unauthorized)?;
        let claims = state.jwt_verifier.verify(jwt).await?;
        Ok(Cedar(CedarContext { principal: claims.sub, tenant_id: claims.tenant, ..Default::default() }))
    }
}
```

### D-3. gRPC interceptor (tonic)

```rust
pub fn cedar_interceptor(svc: BoxCloneService<Request, Response, BoxError>) -> impl Service<Request> + Clone {
    tower::ServiceBuilder::new()
        .layer_fn(|inner| CedarInterceptor { inner })
        .service(svc)
}

#[derive(Clone)]
pub struct CedarInterceptor<S> { inner: S }

impl<S: Service<Request, Response = Response, Error = BoxError> + Clone + Send + 'static>
    Service<Request> for CedarInterceptor<S>
where S::Future: Send + 'static
{
    type Response = Response; type Error = BoxError; type Future = BoxFuture<'static, Result<Response, BoxError>>;
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), BoxError>> { self.inner.poll_ready(cx) }
    fn call(&mut self, mut req: Request) -> Self::Future {
        let principal = req.metadata().get("x-oyatie-principal")
            .and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
        let tenant = req.metadata().get("x-oyatie-tenant")
            .and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
        req.extensions_mut().insert(CedarContext { principal, tenant_id: tenant, ..Default::default() });
        let fut = self.inner.call(req);
        Box::pin(fut)
    }
}
```

### D-4. Worker pattern (Kafka consumer)

```rust
pub async fn run_downtime_overlay_consumer(state: AppState, shutdown: CancellationToken)
    -> anyhow::Result<()>
{
    let consumer = state.kafka.consumer("pp-downtime-overlay-v1",
        &["plant-maintenance.downtime-window.v1"])?;
    while !shutdown.is_cancelled() {
        let msg = consumer.recv().await?;
        let event_id = msg.headers().get("x-oyatie-event-id")
            .ok_or(anyhow!("missing event_id"))?.to_owned();
        if state.dedupe.seen(&event_id).await? {
            consumer.commit_async(&msg, CommitMode::Async)?;
            continue;
        }
        let payload: DowntimeOverlayEvent = serde_json::from_slice(msg.payload().unwrap_or(&[]))?;
        match state.ingest_overlay_uc.handle(payload).await {
            Ok(_) => {
                state.dedupe.mark(&event_id).await?;
                consumer.commit_async(&msg, CommitMode::Async)?;
            }
            Err(UseCaseError::Validation(_)) => {
                state.kafka.produce("pp.dlq.downtime-overlay.v1", msg.payload().unwrap_or(&[])).await?;
                consumer.commit_async(&msg, CommitMode::Async)?;
            }
            Err(e) => {
                tracing::error!(error=?e, "transient error, leaving offset uncommitted");
            }
        }
    }
    Ok(())
}
```

### D-5. OpenAPI fragment (release)

```yaml
paths:
  /v1/tenants/{tenant_id}/production-orders/{order_id}/release:
    post:
      operationId: releaseProductionOrder
      tags: [production-planning]
      security: [{ jwt: [] }]
      parameters:
        - $ref: '#/components/parameters/TenantId'
        - $ref: '#/components/parameters/OrderId'
        - $ref: '#/components/parameters/IdempotencyKey'
      requestBody:
        required: true
        content:
          application/json:
            schema: { $ref: '#/components/schemas/ReleaseOrderBody' }
      responses:
        '202': { description: Released, content: { application/json: { schema: { $ref: '#/components/schemas/ReleaseOrderResponseBody' } } } }
        '403': { $ref: '#/components/responses/PermissionDenied' }
        '404': { $ref: '#/components/responses/NotFound' }
        '409': { $ref: '#/components/responses/Conflict' }
        '429': { $ref: '#/components/responses/RateLimited' }
```

### D-6. Cedar context (per-route)

Cedar evaluation happens INSIDE the usecase but the API surface enriches the context with HTTP-specific signals (`http.method`, `http.path`, `client.ip`) so policies can mention them.

### D-7. SLO targets

| Surface | Operation | p50 | p95 | p99 | Rationale |
|---|---|---|---|---|---|
| REST | `POST /production-orders/:id/release` | 30 ms | 75 ms | 150 ms | Adapter + usecase + JSON encode. |
| REST | `GET /calendars/.../availability` (cache hit) | 4 ms | 12 ms | 28 ms | Valkey read. |
| gRPC | `ReleaseProductionOrder` | 26 ms | 60 ms | 120 ms | Lower JSON overhead vs REST. |
| gRPC | `SubscribeMrpRun` (event throughput) | 800 evt/s/replica | 1200 evt/s | — | Bidirectional streaming. |
| Worker | per envelope (warm) | 6 ms | 15 ms | 35 ms | Dedupe + usecase + commit. |

### D-8. Failure modes & recovery

1. **`OpenApiDrift`** — handler request/response no longer matches schema. CI lint fails; build broken. Recovery: regenerate schema or update handler.
2. **`JwtKeyRotationLag`** — IdP rotates signing key, verifier cache stale. 401 spikes. Recovery: JWKS auto-refresh + manual reload runbook.
3. **`RateLimitFalseLockout`** — shared NAT collapses to one tenant key. Recovery: per-principal (sub claim) limiter, not per-IP.
4. **`KafkaConsumerLag`** — handler slow; partition offset lag grows. Alert at 30s lag. Recovery: scale consumer group, profile slow handler.
5. **`DlqAccumulation`** — many messages routed to DLQ. Recovery: drain DLQ via replay worker after fixing root cause; runbook `runbooks/dlq-drain.md`.
6. **`gRPCConnectionStorm`** — client reconnect loop. Recovery: jittered backoff, server-side connection cap.

### D-9. Migration notes

Source vendor surfaces:
- SAP `BAPI_PRODORD_*` RFC modules → gRPC methods listed in A.2.
- SAP Gateway OData `API_PRODUCTION_ORDER_2_SRV` → REST paths listed in A.2.
- SAP `SM36`/`SM37` background-job scheduling → worker cron + on-demand triggers.

### D-10. Audit-event class registry

| Event class | Severity | Emitter |
|---|---|---|
| `EVT-PRODUCTION_PLANNING-API-REQUEST_ACCEPTED` | informational | REST/gRPC middleware |
| `EVT-PRODUCTION_PLANNING-API-RATE_LIMITED`     | warning       | REST middleware |
| `EVT-PRODUCTION_PLANNING-API-JWT_REJECTED`     | security      | REST middleware |
| `EVT-PRODUCTION_PLANNING-API-MTLS_REJECTED`    | security      | gRPC interceptor |
| `EVT-PRODUCTION_PLANNING-WORKER-DLQ_ROUTED`    | warning       | worker |

### D-11. Ontology projection

REST/gRPC/worker surfaces themselves do NOT project — they delegate to usecase, which projects.

### D-12. Cross-µservice handoffs

| Direction | Counterparty | Surface |
|---|---|---|
| inbound  | UI                | REST (`/v1/tenants/...`) |
| inbound  | partner integrator | REST or gRPC (depending on contract) |
| inbound  | inter-µservice    | gRPC |
| inbound  | event sources     | Worker (Kafka) |
| outbound | observability     | OTLP traces + metrics |

## E. Failure-mode summary

See D-8. Each scenario maps to a runbook under `runbooks/`.

## F. Migration / rollback

Per-surface feature flag: `pp_api_rest_v1`, `pp_api_grpc_v1`, `pp_api_workers_v1`. Disabling REST keeps gRPC and workers functional (UI degrades to read-only via projection cache).

## G. References

- ADR-0105, ADR-0244, ADR-0252, ADR-0253, ADR-0263, ADR-0294, ADR-0297, ADR-0314.
- OpenAPI 3.2.0 spec.
- proto3 + buf style guide.
- Apache Kafka 3.7 consumer semantics.
- Benchmarks: SAP RFC + Gateway OData | Oracle Fusion REST + ESS | Dynamics 365 SCM OData + Batch | NetSuite RESTlet + Scheduled Scripts.

## H. Out of scope

- Domain (IP-001..IP-006), usecase (IP-007..IP-012, IP-018..IP-025), adapter (IP-013), integration tests (IP-015), MES bidirectional details (IP-024).

— end IP-014 —
