---
doc_class: ImplementationPlan
ip_id: IP-014
microservice: plant-maintenance
related_adrs: [ADR-0105, ADR-0131, ADR-0244, ADR-0252, ADR-0253, ADR-0257, ADR-0263, ADR-0294, ADR-0297, ADR-0314, ADR-0315, ADR-0329, ADR-0330, ADR-0331]
journey_ref: j208-asset-lifecycle-and-maintenance
sap_submodule: Outermost adapter surface — REST + gRPC + AsyncAPI workers wrapping IP-007..012 use-cases; equivalent to SAP Gateway OData / SAP CPI integration surface
service_surface: substrate
persona: integration-engineer, mobile-engineer, frontend-engineer
status: Accepted
date: 2026-05-20
owner_team: axis-plant-maintenance + axis-erp-parity + axis-public-api
planned_enforcement_ref: oya-governance-plant-maintenance-doc-set
---

# IP-014: REST + gRPC + AsyncAPI worker surfaces for `plant-maintenance`

## A. Intent

Implements **layer-1 (delivery surfaces)** per ADR-0105 — the outermost shell that wraps IP-007..012 use-cases in:

1. **REST (HTTP/3 + HTTP/2 fallback)** via `axum` + `hyper` per ADR-0253, OpenAPI 3.2.0 contracts at `contracts/openapi-v1.yaml`.
2. **gRPC** via `tonic`, proto3 at `contracts/plant-maintenance-v1.proto`.
3. **AsyncAPI workers** via `rdkafka` consumers, AsyncAPI 3.1.0 at `contracts/asyncapi-v1.yaml`.
4. **Background cron jobs** (deadline monitor, no-show sweep, OEE rollup sweep).

Industry-precedent equivalents: SAP Gateway OData service layer / SAP CPI iflow consumers; AWS API Gateway + EventBridge consumers; Stripe REST API; Twilio webhook handlers + AMQP workers. Hyperscaler analog: AWS API Gateway HTTP API + AWS Lambda event-source mapping for SQS/Kinesis.

### A.1 Why the delivery surface is non-trivial

1. **HTTP/3 + ECH + PQC.** Per ADR-0253: HTTP/3 default, HTTP/2 + HTTP/1.1 fallback; ECH on edge; PQC hybrid in TLS. Surface MUST advertise `Alt-Svc: h3=":443"`.
2. **Cedar gate at the edge.** Every REST and gRPC route maps to a Cedar action; permit-check is at the surface (defence-in-depth) before reaching the use-case (which re-checks).
3. **Idempotency-key propagation.** REST `Idempotency-Key` header → use-case `RequestContext::idempotency_key`. Stripe-style.
4. **Tenant pin from JWT.** `Authorization: Bearer <jwt>` carries `tenant_id`; surface enforces 3-way pin (URL + JWT + body) before use-case.
5. **Worker consumer idempotency.** Kafka consumers dedupe on `event_id`; offset committed only after use-case success.
6. **Cron pacing.** Cron use-cases use leader-election (Postgres advisory locks) to ensure single-leader per cell.

## B. Acceptance criteria

- **AC-1:** REST surface covers all use-cases via OpenAPI 3.2.0 routes (≥40 endpoints).
- **AC-2:** gRPC surface covers use-cases for inter-µservice calls via proto3 service (≥30 RPCs).
- **AC-3:** AsyncAPI workers consume: `signal.equipment-running.v1`, `permit.issued.v1`, `inventory.consumed.v1`, `wo.settled.v1`, `production-planning.ddmrp.buffer-breached-red.v1`.
- **AC-4:** All routes Cedar-gated at edge; deny logged + rejected with constant-time response.
- **AC-5:** HTTP/3 default with HTTP/2 + HTTP/1.1 fallback; `Alt-Svc: h3` advertised; ECH config served per ADR-0253.
- **AC-6:** Idempotency-Key header propagated through to use-case.
- **AC-7:** 3-way tenant pin (URL `/tenants/{tenant_id}/...`, JWT `tenant_id` claim, body `tenant_id`).
- **AC-8:** Worker consumer commits offset only after use-case success.
- **AC-9:** Cron jobs use Postgres advisory lock for single-leader per cell.
- **AC-10:** RED metrics emitted per route: requests/errors/duration.

## C. Verification

```bash
cargo test -p oya-plant-maintenance-rest-api -- equipment_create_route_e2e
cargo test -p oya-plant-maintenance-rest-api -- wo_release_route_idempotency_replay
cargo test -p oya-plant-maintenance-rest-api -- tenant_pin_three_way
cargo test -p oya-plant-maintenance-rest-api -- http3_advertised
cargo test -p oya-plant-maintenance-rest-api -- ech_config_served
cargo test -p oya-plant-maintenance-grpc-api -- inventory_callback_proto3_schema
cargo test -p oya-plant-maintenance-grpc-api -- skill_matrix_query_round_trip
cargo test -p oya-plant-maintenance-asyncapi-worker -- permit_issued_consumer_dedup
cargo test -p oya-plant-maintenance-asyncapi-worker -- signal_running_auto_close
cargo test -p oya-plant-maintenance-cron -- deadline_monitor_leader_election
cargo test -p oya-plant-maintenance-cron -- no_show_sweep_leader_election
cargo test -p oya-plant-maintenance-cron -- oee_sweep_leader_election
```

## D. Detailed mechanics

### D-1. REST surface (`axum`)

```rust
pub fn router(state: AppState) -> Router {
    Router::new()
        // equipment
        .route("/v1/tenants/:tenant_id/equipment",                 post(create_equipment_handler))
        .route("/v1/tenants/:tenant_id/equipment/:eq_id",          get(load_equipment_handler))
        .route("/v1/tenants/:tenant_id/equipment/:eq_id",          patch(change_equipment_handler))
        .route("/v1/tenants/:tenant_id/equipment/:eq_id/move",     post(move_equipment_handler))
        .route("/v1/tenants/:tenant_id/equipment/:eq_id/retire",   post(retire_equipment_handler))
        .route("/v1/tenants/:tenant_id/equipment/:eq_id/characteristics", post(attach_char_handler))
        // floc
        .route("/v1/tenants/:tenant_id/functional-locations",      post(create_floc_handler))
        .route("/v1/tenants/:tenant_id/functional-locations/:fl",  get(load_floc_handler))
        // maintenance plan
        .route("/v1/tenants/:tenant_id/maintenance-plans",         post(create_plan_handler))
        .route("/v1/tenants/:tenant_id/maintenance-plans/:plan_id/activate", post(activate_plan_handler))
        .route("/v1/tenants/:tenant_id/maintenance-plans/:plan_id/publish-critical", post(publish_critical_plan_handler))
        // work order
        .route("/v1/tenants/:tenant_id/work-orders",               post(create_wo_handler))
        .route("/v1/tenants/:tenant_id/work-orders/breakdown",     post(create_breakdown_wo_handler))
        .route("/v1/tenants/:tenant_id/work-orders/:wo/release",   post(release_wo_handler))
        .route("/v1/tenants/:tenant_id/work-orders/:wo/operations/:op/confirm", post(confirm_op_handler))
        .route("/v1/tenants/:tenant_id/work-orders/:wo/teco",      post(teco_handler))
        .route("/v1/tenants/:tenant_id/work-orders/:wo/close",     post(close_wo_handler))
        .route("/v1/tenants/:tenant_id/work-orders/:wo/cancel",    post(cancel_wo_handler))
        // reservation
        .route("/v1/tenants/:tenant_id/reservations",              post(reserve_components_handler))
        .route("/v1/tenants/:tenant_id/reservations/:res/movements", post(commit_movement_handler))
        .route("/v1/tenants/:tenant_id/reservations/:res/cancel",  post(cancel_reservation_handler))
        // dispatch
        .route("/v1/tenants/:tenant_id/dispatches",                post(request_dispatch_handler))
        .route("/v1/tenants/:tenant_id/dispatches/:disp/offer",    post(offer_dispatch_handler))
        .route("/v1/tenants/:tenant_id/dispatches/:disp/accept",   post(accept_offer_handler))
        .route("/v1/tenants/:tenant_id/dispatches/:disp/decline",  post(decline_offer_handler))
        .route("/v1/tenants/:tenant_id/dispatches/:disp/start",    post(start_dispatch_handler))
        .route("/v1/tenants/:tenant_id/dispatches/:disp/complete", post(complete_dispatch_handler))
        // downtime
        .route("/v1/tenants/:tenant_id/downtime-windows",          post(open_downtime_handler))
        .route("/v1/tenants/:tenant_id/downtime-windows/:dt/close", post(close_downtime_handler))
        .route("/v1/tenants/:tenant_id/oee/:floc_id",              get(oee_handler))
        // permit (handler delegates to permit µservice but exposes proxy here for UI convenience)
        .route("/v1/tenants/:tenant_id/work-orders/:wo/permit-state", get(permit_state_handler))
        .with_state(state)
        .layer(middleware::from_fn(cedar_edge_gate))
        .layer(middleware::from_fn(tenant_pin_three_way))
        .layer(middleware::from_fn(idempotency_key_propagate))
        .layer(middleware::from_fn(red_metrics))
        .layer(middleware::from_fn(altsvc_h3_advertise))
}
```

### D-2. gRPC surface (`tonic`)

```proto
syntax = "proto3";
package oya.plant_maintenance.v1;

service PlantMaintenance {
  rpc CreateEquipment(CreateEquipmentRequest) returns (EquipmentRef);
  rpc LoadEquipment(LoadEquipmentRequest)     returns (Equipment);
  rpc MoveEquipment(MoveEquipmentRequest)     returns (EquipmentRef);

  rpc CreateMaintenancePlan(CreatePlanRequest) returns (PlanRef);
  rpc PublishCriticalPlan(PublishCriticalRequest) returns (PlanRef);
  rpc OnCompletion(OnCompletionRequest)       returns (OnCompletionRef);

  rpc CreateWorkOrder(CreateWoRequest)        returns (WoRef);
  rpc ReleaseWorkOrder(ReleaseWoRequest)      returns (WoRef);
  rpc ConfirmOperation(ConfirmOperationRequest) returns (ConfirmRef);
  rpc Teco(TecoRequest)                       returns (WoRef);
  rpc Close(CloseRequest)                     returns (WoRef);

  rpc ReserveComponents(ReserveRequest)       returns (ReservationRef);
  rpc CommitGoodsMovement(MovementRequest)    returns (MovementRef);
  rpc CancelReservation(CancelReservationRequest) returns (Empty);

  rpc RequestDispatch(RequestDispatchRequest) returns (DispatchRef);
  rpc OfferDispatch(OfferDispatchRequest)     returns (OfferRef);
  rpc AcceptOffer(AcceptRequest)              returns (Empty);

  rpc OpenDowntime(OpenDowntimeRequest)       returns (DtRef);
  rpc CloseDowntime(CloseDowntimeRequest)     returns (DowntimeClosedRef);
  rpc Oee(OeeRequest)                         returns (OeeBreakdown);
}
```

### D-3. AsyncAPI worker pattern

```rust
pub async fn run_signal_running_consumer(ctx: AppContext, mut rx: KafkaConsumer<SignalRunningEvent>) {
    let uc = AutoCloseOnSignalUseCase::new(/* deps */);
    while let Some(msg) = rx.next().await {
        let event_id = msg.payload.event_id;
        let result = uc.execute(msg.payload, RequestContext::from_kafka(&msg)).await;
        match result {
            Ok(_) => msg.commit_offset().await,
            Err(UseCaseError::CrossTenant) => {
                ctx.audit.security_log(event_id, "cross_tenant_in_async").await;
                msg.commit_offset().await;     // poison-message — don't reprocess
            }
            Err(UseCaseError::Transient) => { /* skip commit, retry */ }
            Err(e) => { ctx.dlq.send(msg.into_dlq(e)).await; msg.commit_offset().await; }
        }
    }
}
```

### D-4. Cron jobs with Postgres advisory lock

```rust
pub async fn cron_deadline_monitor(state: AppState) {
    let mut ticker = tokio::time::interval(Duration::from_secs(60));
    loop {
        ticker.tick().await;
        let acquired = sqlx::query!("SELECT pg_try_advisory_lock(7100) AS got")
            .fetch_one(&state.pool).await.map(|r| r.got.unwrap_or(false)).unwrap_or(false);
        if !acquired { continue; }
        let _ = state.deadline_sweep_uc.tick(Utc::now()).await;
        let _ = sqlx::query!("SELECT pg_advisory_unlock(7100)").execute(&state.pool).await;
    }
}

pub async fn cron_no_show_sweep(state: AppState)   { /* lock id 7101, 5-min cadence */ }
pub async fn cron_oee_rollup_sweep(state: AppState){ /* lock id 7102, 5-min cadence */ }
```

### D-5. HTTP/3 + ECH + PQC config

```rust
pub fn tls_config(cert_path: &Path, key_path: &Path, ech_config: &Path) -> rustls::ServerConfig {
    let mut cfg = rustls::ServerConfig::builder_with_provider(rustls::crypto::aws_lc_rs::default_provider().into())
        .with_protocol_versions(&[&rustls::version::TLS13]).unwrap()
        .with_no_client_auth()
        .with_single_cert(load_certs(cert_path), load_key(key_path)).unwrap();
    cfg.alpn_protocols = vec![b"h3".to_vec(), b"h2".to_vec(), b"http/1.1".to_vec()];
    cfg.send_half_rtt_data = false;
    cfg.ech_config = Some(load_ech_config(ech_config));
    // PQC hybrid X25519MLKEM768 + ed25519+ml_dsa_65 enabled via aws-lc-rs provider
    cfg
}
```

### D-6. Cedar edge gate middleware

```rust
async fn cedar_edge_gate<B>(req: Request<B>, next: Next<B>) -> impl IntoResponse {
    let ctx = req.extensions().get::<RequestContext>().cloned().unwrap();
    let route_meta = req.extensions().get::<RouteMeta>().cloned().unwrap();
    let decision = state().cedar.evaluate(AuthzRequest::from_route(&ctx, &route_meta)).await;
    match decision {
        Ok(d) if d.is_permit() => next.run(req).await,
        Ok(_) => StatusCode::FORBIDDEN.into_response(),         // constant-time
        Err(_) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}
```

### D-7. AsyncAPI worker catalog

| Consumer | Topic | Idempotency key | Calls use-case |
|---|---|---|---|
| `signal-running-consumer` | `signal.equipment-running.v1` | `(tenant, equipment_id, signal_hlc)` | `AutoCloseOnSignalUseCase` |
| `permit-issued-consumer` | `pm-wcm.permit.issued.v1` | `(tenant, wo_id, permit_id)` | `HandlePermitIssuedUseCase` |
| `inventory-consumed-consumer` | `inventory.consumed.v1` | `(tenant, reservation_id, item_no, mv_seq)` | (audit only — usecase already wrote) |
| `wo-settled-consumer` | `oya-cloud-finops.wo.settled.v1` | `(tenant, wo_id, settled_hlc)` | unblocks `CloseWorkOrderUseCase` |
| `ddmrp-breach-consumer` | `production-planning.ddmrp.buffer-breached-red.v1` | `(tenant, part, breach_hlc)` | trigger spare-part procurement gate |
| `dlq-replay-consumer` | `plant-maintenance.dlq.v1` | (varies) | per-use-case manual replay |

### D-8. SLO targets

| Surface | p50 | p95 | p99 |
|---|---|---|---|
| REST POST /work-orders (no parts) | 35 ms | 80 ms | 160 ms |
| REST POST /work-orders (saga) | 95 ms | 220 ms | 460 ms |
| REST GET /equipment/:id | 8 ms | 20 ms | 42 ms |
| REST GET /oee/:floc | 15 ms | 35 ms | 72 ms |
| gRPC CreateWorkOrder | 32 ms | 75 ms | 150 ms |
| gRPC OnCompletion | 38 ms | 88 ms | 175 ms |
| Worker consume → use-case → commit | 30 ms | 70 ms | 140 ms |
| Cron tick acquisition | 2 ms | 5 ms | 10 ms |

### D-9. Audit-event registry (surface-level)

| Event class | Severity | Emitter |
|---|---|---|
| `EVT-PLANT_MAINTENANCE-SURFACE-EDGE_DENY` | warning | rest/grpc |
| `EVT-PLANT_MAINTENANCE-SURFACE-TENANT_PIN_MISMATCH` | security | rest/grpc |
| `EVT-PLANT_MAINTENANCE-SURFACE-IDEMPOTENT_REPLAY` | informational | rest |
| `EVT-PLANT_MAINTENANCE-SURFACE-RATE_LIMITED` | informational | rest |
| `EVT-PLANT_MAINTENANCE-SURFACE-WORKER_DLQ` | warning | worker |
| `EVT-PLANT_MAINTENANCE-SURFACE-CRON_LEADER_LOST` | informational | cron |
| `EVT-PLANT_MAINTENANCE-SURFACE-HTTP3_FALLBACK_USED` | informational | rest |

### D-10. Failure modes & recovery

1. **`EdgeCedarBundleStale`** — edge gate using stale bundle. Default-deny applied; UI shows "policy refresh in progress". Runbook `runbooks/edge-cedar-stale.md`.
2. **`TenantPinMismatch`** — URL says tenant X, JWT says tenant Y. Reject with 403 (constant-time); security audit captured. Runbook `runbooks/tenant-pin-mismatch.md`.
3. **`HTTP3Blocked`** — corporate firewall blocks QUIC. Fall back to HTTP/2 transparently; metric `pm_http3_blocked_total` increments. Runbook `runbooks/http3-blocked.md`.
4. **`WorkerLagSpike`** — consumer falls behind. Scale-out automatically (HPA); alert at lag > 5 min. Runbook `runbooks/worker-lag.md`.
5. **`CronLeaderFlap`** — advisory lock contention. Diagnostic metric exposes flap-rate; threshold > 3/min triggers investigation. Runbook `runbooks/cron-leader-flap.md`.
6. **`DlqOverflow`** — DLQ accumulates faster than replay. Alert; integration engineer triages root cause. Runbook `runbooks/dlq-overflow.md`.

### D-11. Migration notes

REST surface follows API versioning ADR-0258: `/v1/...` for current; `/v2/...` for next major. Deprecation cadence is 6 months minimum.

### D-12. Cross-µservice handoffs

The surface is the entry/exit point of all cross-µservice traffic listed in IPs 001-013.

## E. Failure-mode summary

See D-10.

## F. Migration / rollback

Per-route feature flag (`plant_maintenance_route_<name>_v1`). Worker consumers can be paused per-topic.

## G. References

- ADR-0105, ADR-0253 (HTTP/3 + ECH + PQC), ADR-0258 (API versioning), ADR-0263, ADR-0294, ADR-0295 (SPIFFE), ADR-0297, ADR-0314..0316.
- OpenAPI 3.2.0 spec; AsyncAPI 3.1.0 spec; proto3 spec.
- `axum`, `tonic`, `rdkafka` library docs.
- HTTP/3 RFC 9114; QUIC RFC 9000; ECH draft-ietf-tls-esni-22; PQC X25519MLKEM768 draft.

## H. Out of scope

- Use-cases (IP-007..012), adapters (IP-013), integration tests (IP-015).

— end IP-014 —
