---
doc_class: ImplementationPlan
microservice: production-planning
status: Accepted
date: 2026-05-20
owner_team: axis-production-planning + axis-erp-parity
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0244, ADR-0252, ADR-0253, ADR-0257, ADR-0263, ADR-0294, ADR-0297, ADR-0314, ADR-0315, ADR-0316]
planned_enforcement_ref: oya-governance-production-planning-doc-set
ip_id: IP-011
journey_ref: j101
journey_slug: j101-multi-tier-supply-chain-formation
sap_submodule: PP-SFC (Production Orders — Shop Floor Control) covering CO01/CO02/CO11N/CO15
tenant_class: substrate
persona: production-planner + shop-floor-supervisor
---

# IP-011: Usecase layer for production-order

## A. Intent

Wires the pure `ProductionOrder` aggregate from IP-005 to its orchestration ports. Production orders are the SAP S/4HANA primary unit of execution for discrete manufacturing — they bind a *planned order* (output of MRP run, IP-002/IP-008), a *routing* (IP-004/IP-010), a *BOM* (IP-001/IP-007), and a *capacity reservation* (IP-003/IP-009) into a single state machine that drives shop-floor execution. SAP transactions covered: `CO01` (create), `CO02` (change), `CO40` (convert planned), `CO11N` (operation confirm), `CO15` (order confirm), `CO13` (TECO/close). Oracle Fusion equivalent is `Work Definition` + `Work Order`; Dynamics 365 SCM uses `Production order`; NetSuite uses `Work Orders`.

### A.1 Why orchestration here is non-trivial

The order usecase enforces a **finite state machine** with the following states and legal transitions:

```
created → released → in_progress → partially_confirmed → confirmed → teco → closed
              ↘ cancelled
   created → cancelled
   released → cancelled (requires inverse reservation)
   in_progress → on_hold → in_progress (quality hold round-trip)
```

Beyond state hygiene the usecase must:

1. **Atomically reserve capacity** at release time — capacity is consumed against IP-003 `WorkCenterCapacity` and any conflict (overlap with another order's reservation) MUST abort the release, not the create.
2. **Bind to planned-order pegging** — a converted production order carries `pegged_demand_keys: Vec<PeggedDemandKey>` so MRP downstream can update its pegging graph (consumes IP-008's `planned-order.converted.v1` event).
3. **Confirm partial quantities idempotently** — `ConfirmOperationUseCase` must accept duplicate confirm payloads (same `confirm_counter`) without double-counting; this is critical because MES (IP-024) retries on its own cadence.
4. **Emit downstream propagation events** — release triggers `shop-floor-release.requested.v1` consumed by warehouse staging (IP-017 hand-off); confirm triggers `production-order.operation-confirmed.v1` consumed by inventory (yield posting) and costing (variance posting).
5. **Maintain Cedar context fidelity** — every transition includes `decision_id` so the audit trail reconstructs full provenance.

## B. Acceptance criteria

- **AC-1:** `CreateProductionOrderUseCase::execute` Cedar-gated on `production_planning::production_order::create`; default deny preserved; idempotency on `(tenant_id, idempotency_key)`.
- **AC-2:** `ReleaseProductionOrderUseCase::execute` atomically (i) acquires capacity reservation, (ii) appends `shop-floor-release.requested.v1`, (iii) updates order state to `released`. ROLLBACK on any leg failure.
- **AC-3:** `ConfirmOperationUseCase::execute(confirm_payload)` idempotent on `(tenant_id, order_id, operation_no, confirm_counter)`. Replaying same counter returns the original decision_id.
- **AC-4:** `ConvertPlannedOrderUseCase::execute(planned_order_id)` reads pegging from MRP outbox (IP-002), creates production order with carried pegging keys.
- **AC-5:** `CancelOrderUseCase::execute` only legal in states `{created, released}`; in `released` inverse-releases the capacity reservation and emits `shop-floor-release.cancelled.v1`.
- **AC-6:** State machine illegal transitions raise `IllegalStateTransition { from, to, order_id }` typed error — never panics.
- **AC-7:** Audit emission per ADR-0263 for every state-changing transition.
- **AC-8:** HLC stamping per ADR-0297; outbox envelopes carry `hlc` field per AsyncAPI 3.1.0 schema.
- **AC-9:** Cross-tenant defence-in-depth — load-by-id verifies `tenant_id` from persisted row matches principal context.
- **AC-10:** Partial confirm: cumulative `confirmed_qty` ≤ `target_qty` enforced; exceeding triggers `OverdeliveryAttempt` typed error.

## C. Verification

```bash
cargo test -p oya-production-planning-order-usecase -- create_happy_path
cargo test -p oya-production-planning-order-usecase -- create_idempotent_same_key
cargo test -p oya-production-planning-order-usecase -- release_atomic_reservation
cargo test -p oya-production-planning-order-usecase -- release_rolls_back_on_outbox_fail
cargo test -p oya-production-planning-order-usecase -- confirm_partial_quantities
cargo test -p oya-production-planning-order-usecase -- confirm_idempotent_replay
cargo test -p oya-production-planning-order-usecase -- confirm_overdelivery_rejected
cargo test -p oya-production-planning-order-usecase -- convert_planned_carries_pegging
cargo test -p oya-production-planning-order-usecase -- cancel_released_inverse_reserves
cargo test -p oya-production-planning-order-usecase -- illegal_state_transition_typed_error
cargo test -p oya-production-planning-order-usecase -- cross_tenant_load_rejected
cargo test -p oya-production-planning-order-usecase -- hlc_ordering_late_confirm
cargo test -p oya-production-planning-order-contract -- asyncapi_release_envelope_schema
cargo test -p oya-production-planning-order-contract -- asyncapi_confirm_envelope_schema
```

## D. Detailed mechanics

### D-1. Use-case structs

```rust
pub struct CreateProductionOrderUseCase<R, C, O, A>      { /* repo, cedar, outbox, audit */ }
pub struct ReleaseProductionOrderUseCase<R, C, K, O, A>  { /* + capacity reservation port K */ }
pub struct ConfirmOperationUseCase<R, C, Y, O, A>        { /* + yield-posting port Y */ }
pub struct ConvertPlannedOrderUseCase<R, C, P, O, A>     { /* + planned-order reader P */ }
pub struct CancelOrderUseCase<R, C, K, O, A>             { /* + capacity inverse port K */ }
```

### D-2. State-machine guard

```rust
fn assert_transition(from: OrderState, to: OrderState) -> Result<(), IllegalStateTransition> {
    use OrderState::*;
    let ok = matches!((from, to),
        (Created, Released) | (Created, Cancelled) |
        (Released, InProgress) | (Released, Cancelled) |
        (InProgress, OnHold) | (OnHold, InProgress) |
        (InProgress, PartiallyConfirmed) | (PartiallyConfirmed, PartiallyConfirmed) |
        (PartiallyConfirmed, Confirmed) | (InProgress, Confirmed) |
        (Confirmed, Teco) | (Teco, Closed)
    );
    if ok { Ok(()) } else { Err(IllegalStateTransition { from, to }) }
}
```

### D-3. Release use-case (atomic three-leg commit)

```rust
impl<R, C, K, O, A> ReleaseProductionOrderUseCase<R, C, K, O, A>
where R: OrderRepository, C: CedarEvaluator, K: CapacityReservationPort,
      O: OutboxDispatcher, A: AuditEmitter,
{
    pub async fn execute(&self, input: ReleaseInput) -> Result<ReleaseOutput, UseCaseError> {
        let decision = self.cedar.evaluate(cedar_req_release(&input)).await?;
        if !decision.is_permit() { return Err(UseCaseError::PermissionDenied { reason: decision.reasons() }); }

        let tx = self.repo.begin_tx().await?;
        let mut order = self.repo.load_for_update(&tx, &input.tenant_id, &input.order_id).await?
            .ok_or(UseCaseError::NotFound)?;
        assert_transition(order.state(), OrderState::Released)?;

        // Leg 1: capacity reservation (idempotent on order_id)
        let reservation = self.capacity.reserve(
            &tx, &input.tenant_id, &order.work_center_plan(), &decision).await?;

        // Leg 2: outbox shop-floor-release request
        let env = shop_floor_release_requested_event(&order, &reservation, &decision);
        self.outbox.append(&tx, &env).await?;

        // Leg 3: state update
        order.transition_to_released(reservation.id(), Hlc::now())?;
        self.repo.save(&tx, &order).await?;

        // Audit
        self.audit.emit(&tx, AuditEntry::release(&order, &decision)).await?;

        tx.commit().await?;
        Ok(ReleaseOutput { decision_id: decision.decision_id, reservation_id: reservation.id(), hlc: order.hlc() })
    }
}
```

### D-4. Idempotent confirm

```rust
pub async fn confirm_operation(&self, payload: ConfirmPayload) -> Result<ConfirmOutput, UseCaseError> {
    let dedupe_key = (payload.tenant_id.clone(), payload.order_id.clone(),
                      payload.operation_no, payload.confirm_counter);
    if let Some(prior) = self.repo.find_confirm_by_dedupe(&dedupe_key).await? {
        return Ok(ConfirmOutput::from_prior(prior));  // idempotent replay
    }
    let decision = self.cedar.evaluate(cedar_req_confirm(&payload)).await?;
    if !decision.is_permit() { return Err(UseCaseError::PermissionDenied { reason: decision.reasons() }); }

    let tx = self.repo.begin_tx().await?;
    let mut order = self.repo.load_for_update(&tx, &payload.tenant_id, &payload.order_id).await?
        .ok_or(UseCaseError::NotFound)?;
    if payload.confirmed_qty + order.confirmed_qty_for(payload.operation_no) > order.target_qty_for(payload.operation_no) {
        return Err(UseCaseError::OverdeliveryAttempt);
    }
    order.apply_confirm(payload.clone(), decision.decision_id, Hlc::now())?;
    self.repo.save_confirm_record(&tx, &payload, decision.decision_id).await?;
    self.repo.save(&tx, &order).await?;
    self.outbox.append(&tx, &operation_confirmed_event(&order, &payload, &decision)).await?;
    self.audit.emit(&tx, AuditEntry::confirm(&order, &payload, &decision)).await?;
    tx.commit().await?;
    Ok(ConfirmOutput { decision_id: decision.decision_id, hlc: order.hlc(), order_state: order.state() })
}
```

### D-5. Port traits

```rust
#[async_trait]
pub trait OrderRepository {
    async fn begin_tx(&self) -> Result<RepoTx, RepoError>;
    async fn load_for_update(&self, tx: &RepoTx, tenant: &TenantId, id: &OrderId)
        -> Result<Option<ProductionOrder>, RepoError>;
    async fn save(&self, tx: &RepoTx, order: &ProductionOrder) -> Result<(), RepoError>;
    async fn save_confirm_record(&self, tx: &RepoTx, payload: &ConfirmPayload, decision: DecisionId)
        -> Result<(), RepoError>;
    async fn find_confirm_by_dedupe(&self, key: &ConfirmDedupeKey)
        -> Result<Option<ConfirmRecord>, RepoError>;
}

#[async_trait]
pub trait CapacityReservationPort {
    async fn reserve(&self, tx: &RepoTx, tenant: &TenantId, plan: &WorkCenterPlan, decision: &CedarDecision)
        -> Result<CapacityReservation, CapacityError>;
    async fn inverse(&self, tx: &RepoTx, tenant: &TenantId, reservation_id: &ReservationId)
        -> Result<(), CapacityError>;
}
```

### D-6. Cedar context

```jsonc
{
  "principal": "oyatie::tenant::acme::user::supervisor-7",
  "action":    "production_planning::production_order::release",
  "resource":  "production_planning::production_order::PO-FG-0001-9001",
  "context": {
    "tenant_id": "acme", "plant_code": "P01", "order_state_from": "created",
    "data_class": "operational", "source_system_id": "production_planning",
    "policy_bundle_version": "2026.05.20-r3", "residency_pack": "global+kr",
    "byok_mode": "platform_default"
  }
}
```

### D-7. Outbox envelopes (AsyncAPI 3.1.0)

| Channel | Trigger | Consumers |
|---|---|---|
| `production-planning.shop-floor-release.requested.v1` | release | `warehouse`, `manufacturing-execution-system`, `quality-management` |
| `production-planning.production-order.operation-confirmed.v1` | partial/final confirm | `inventory`, `costing`, `mrp-run`, `mes` |
| `production-planning.production-order.cancelled.v1` | cancel | `warehouse`, `mes`, `mrp-run` |
| `production-planning.production-order.teco.v1` | TECO | `costing` (cost-collector variance close) |
| `production-planning.production-order.closed.v1` | close | `costing`, `archival` |

### D-8. Audit-event class registry (per ADR-0263)

| Event class | Severity | Emitter | Sink |
|---|---|---|---|
| `EVT-PRODUCTION_PLANNING-PRODUCTION_ORDER-CREATED`        | informational | usecase | `audit-events.v1` |
| `EVT-PRODUCTION_PLANNING-PRODUCTION_ORDER-RELEASED`       | informational | usecase | `audit-events.v1` |
| `EVT-PRODUCTION_PLANNING-PRODUCTION_ORDER-CONFIRMED`      | informational | usecase | `audit-events.v1` |
| `EVT-PRODUCTION_PLANNING-PRODUCTION_ORDER-OVERDELIVERY_REJECTED` | warning | usecase | `audit-events.v1` |
| `EVT-PRODUCTION_PLANNING-PRODUCTION_ORDER-ILLEGAL_TRANSITION`    | warning | usecase | `audit-events.v1` |
| `EVT-PRODUCTION_PLANNING-PRODUCTION_ORDER-PERMISSION_DENIED`     | security | usecase | `audit-events.security.v1` |

### D-9. SLO targets

| Operation | p50 | p95 | p99 | Rationale |
|---|---|---|---|---|
| `Create` | 14 ms | 32 ms | 70 ms | Cedar + 2 DB roundtrips + outbox. |
| `Release` | 24 ms | 55 ms | 110 ms | Three-leg atomic commit; capacity check adds one DB read. |
| `Confirm` (cold) | 18 ms | 40 ms | 85 ms | Idempotency probe + state update. |
| `Confirm` (idempotent replay) | 4 ms | 9 ms | 20 ms | Dedupe-table lookup only. |
| `Convert` | 28 ms | 60 ms | 120 ms | Reads planned-order from MRP outbox + creates order + outbox emit. |
| `Cancel` | 22 ms | 48 ms | 95 ms | Inverse-reservation leg adds DB roundtrip. |

### D-10. Failure modes & recovery

1. **`CapacityReservationConflict`** — capacity port refuses release because another order already reserved the same slot. Tx rolls back; order remains `created`. Caller retries after reshuffle; runbook `runbooks/capacity-conflict.md`.
2. **`OutboxAppendFailure`** — Postgres outbox insert fails. Tx rolls back. Idempotency key preserved; safe retry. Operator alarm fires after 3 consecutive failures.
3. **`OverdeliveryAttempt`** — confirmed qty exceeds target. Typed error to caller; no state mutation; MES UI prompts supervisor to open a yield-variance ticket. Runbook `runbooks/overdelivery.md`.
4. **`IllegalStateTransition`** — typically a stale UI or replayed event. Audit emits `EVT-…-ILLEGAL_TRANSITION`. Caller refreshes order state.
5. **`MesDuplicateConfirm`** — same `confirm_counter` arrives twice. Idempotency returns original `decision_id`; no double-post to costing.
6. **`CrossTenantLoad`** — defence-in-depth tripwire; security audit emitted; principal session revoked per `runbooks/cross-tenant-leak-suspected.md`.

### D-11. Migration notes

Source vendor surface: SAP S/4HANA tables `AUFK` (order header), `AFKO` (order header — production), `AFPO` (order item), `AFRU` (operation confirmation), `AFFL` (operation sequence). Lift-shift migration jobs run *through* this usecase (per ADR-0247 self-modification doctrine) so all state transitions are Cedar-audited identically to user-driven transitions. Greenfield tenants start with empty order space; first order seeded by IP-018 DDMRP buffer breach or IP-008 MRP-run conversion.

### D-12. Ontology projection (library-first)

```rust
pub fn project_order_to_ontology(o: &ProductionOrder) -> OntologyDelta {
    OntologyDelta::new()
        .upsert_node(NodeRef::production_order(o.tenant_id(), o.order_id()))
        .upsert_edge(Edge::for_material(o.id(), o.target_material()))
        .upsert_edge(Edge::uses_routing(o.id(), o.routing_key()))
        .upsert_edge(Edge::uses_bom(o.id(), o.bom_key()))
        .upsert_edges(o.pegged_demand_keys().iter().map(|d|
            Edge::pegged_to(o.id(), d.clone())))
        .upsert_edges(o.confirms().iter().map(|c|
            Edge::confirmed_operation(o.id(), c.operation_no(), c.confirmed_qty(), c.hlc())))
        .with_state(o.state())
        .with_hlc(o.hlc())
}
```

### D-13. Cross-µservice handoffs

| Direction | Counterparty | Channel |
|---|---|---|
| inbound  | `mrp-run` (IP-002/IP-008) | gRPC `mrp_run.v1.LoadPlannedOrder` |
| inbound  | `quality-management` | AsyncAPI `quality-hold.requested.v1` (drives `on_hold` transition) |
| outbound | `warehouse` (IP-017) | AsyncAPI `shop-floor-release.requested.v1` |
| outbound | `manufacturing-execution-system` (IP-024) | AsyncAPI `production-order.released.v1` |
| outbound | `costing` | AsyncAPI `production-order.operation-confirmed.v1` (variance posting) |
| outbound | `inventory` | AsyncAPI `production-order.operation-confirmed.v1` (yield posting) |

## E. Failure-mode summary

See D-10. Each scenario maps to a named runbook under `runbooks/`.

## F. Migration / rollback

Feature flag: `production_planning_order_usecase_v1`. Rollback: flag → false; UI shows read-only order browser; existing in-flight orders frozen until flag re-enabled (state is durable in Postgres so no data loss).

## G. References

- ADR-0105, ADR-0244, ADR-0263, ADR-0294, ADR-0297, ADR-0314, ADR-0315, ADR-0316.
- SAP Help: PP-SFC transactions `CO01`, `CO02`, `CO11N`, `CO15`, `CO13`, `CO40`; tables `AUFK`, `AFKO`, `AFPO`, `AFRU`, `AFFL`.
- Benchmarks: SAP PP-SFC | Oracle Fusion Cloud Manufacturing (Work Order) | Microsoft Dynamics 365 SCM (Production order) | NetSuite Work Orders | Infor CloudSuite Industrial Production Orders.

## H. Out of scope

- Domain (IP-005), adapter (IP-013), REST surface (IP-014), MES bidirectional (IP-024), capacity leveling (IP-021), production-line balancing (IP-025).

— end IP-011 —
