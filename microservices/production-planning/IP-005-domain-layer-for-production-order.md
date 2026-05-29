---
doc_class: ImplementationPlan
microservice: production-planning
status: Accepted
date: 2026-05-20
owner_team: axis-production-planning + axis-erp-parity
related_adrs: [ADR-0105, ADR-0131, ADR-0244, ADR-0252, ADR-0257, ADR-0263, ADR-0294, ADR-0297, ADR-0314, ADR-0315, ADR-0316]
planned_enforcement_ref: oya-governance-production-planning-doc-set
ip_id: IP-005
journey_ref: j101
journey_slug: j101-multi-tier-supply-chain-formation
sap_submodule: PP-SFC (Shop Floor Control, transactions CO01/CO02/CO11N/CO15)
tenant_class: substrate
persona: shop-floor-supervisor
---

# IP-005: Domain layer for production-order

## A. Intent

A production order is the **executable instruction to make N units of a finished good**, generated from a planned order via MRP conversion (`CO40`/`CO41` in SAP). It references a frozen BOM revision, a released routing, planned start/finish HLCs, and accumulates confirmations (yield, scrap, time) until completion. In SAP S/4HANA this lives in `AUFK` (order header), `AFKO` (header data PP-specific), `AFPO` (item), `AFVC` (operation), `AFVV` (operation values), `RESB` (reservations), `AFRU` (confirmations).

This IP implements the **domain layer** for `production-order`: aggregate root `ProductionOrder`, lifecycle state machine, confirmation accumulator, variance derivation. No I/O.

### A.1 SAP equivalence delta

| SAP entity | Oyatie aggregate / value object |
|---|---|
| `AUFK` order header | `ProductionOrder` aggregate root (combined with AFKO) |
| `AFKO` PP-specific data | folded into `ProductionOrder` |
| `AFPO` order item | `ProductionOrderItem` (1:1 for discrete; 1:N for co-products) |
| `AFVC` operation | `ProductionOrderOperation` (snapshotted from `Routing` at release) |
| `AFRU` confirmation | `Confirmation` event (event-sourced log inside aggregate) |
| `RESB` reservation | NOT here — handled by `warehouse.outbound-reservation` |
| Order status `CRTD/REL/PCNF/CNF/TECO` | `ProductionOrderState` enum |

### A.2 Journey leg

`j101`: MRP-emitted planned order → converted to production order (this aggregate) → released to shop floor (IP-006) → confirmations stream in → goods receipt → costing settlement.

## B. Acceptance criteria

- **AC-1:** `ProductionOrder::convert_from_planned(planned, bom_rev, routing_rev)` snapshots the BOM and routing revisions at conversion time; later changes to BOM/routing do NOT affect this order.
- **AC-2:** State machine transitions enforced via typed methods (no raw state writes); illegal transitions return `OrderError::IllegalTransition { from, to }`.
- **AC-3:** Confirmation accumulator: `record_confirmation(op_no, qty_good, qty_scrap, time_actual)` updates cumulative `qty_delivered` and `qty_scrapped` invariants.
- **AC-4:** `is_overdelivered()` returns true if `qty_delivered > order_qty * (1 + overdelivery_tolerance_pct/100)`.
- **AC-5:** `is_partial_confirmed()` returns true if 0 < `qty_delivered` < `order_qty` and at least one operation has `partial_confirmed=true`.
- **AC-6:** Variance derivation: `cost_variance()` returns planned vs actual (per BOM consumption + per routing time × work-center rate) — pure function over confirmations.
- **AC-7:** Tenant invariant: `order.tenant_id == bom_rev.tenant_id == routing_rev.tenant_id`.
- **AC-8:** Cedar default-deny preserved on every public method entry.

## C. Verification

```bash
cargo test -p oya-production-planning-order-domain -- convert_from_planned_snapshots_bom_routing
cargo test -p oya-production-planning-order-domain -- state_machine_legal_transitions
cargo test -p oya-production-planning-order-domain -- state_machine_illegal_rejected
cargo test -p oya-production-planning-order-domain -- confirmation_cumulates
cargo test -p oya-production-planning-order-domain -- overdelivery_tolerance_pct
cargo test -p oya-production-planning-order-domain -- partial_confirm_flagged
cargo test -p oya-production-planning-order-domain -- cost_variance_pure_function
cargo test -p oya-production-planning-order-domain -- cross_tenant_inputs_rejected
cargo test -p oya-production-planning-order-domain -- teco_freezes_aggregate
cargo bench -p oya-production-planning-order-domain -- accumulate_500_confirmations
```

Coverage ≥ 95% line, ≥ 90% branch.

## D. Detailed mechanics

### D-1. Aggregate root

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ProductionOrder {
    tenant_id: TenantId,
    order_id: ProductionOrderId,             // ULID, externally rendered as O-YYYYMMDD-NNNNNN
    material_id: MaterialId,
    plant_code: PlantCode,
    order_qty: Decimal,
    qty_uom: UnitOfMeasure,
    bom_revision_snapshot: BomRevision,      // FROZEN snapshot
    routing_snapshot: Routing,               // FROZEN snapshot
    operations: Vec<ProductionOrderOperation>,
    planned_start: Hlc,
    planned_finish: Hlc,
    actual_start: Option<Hlc>,
    actual_finish: Option<Hlc>,
    state: ProductionOrderState,
    qty_delivered: Decimal,
    qty_scrapped: Decimal,
    overdelivery_tolerance_pct: Decimal,     // 0..=100
    underdelivery_tolerance_pct: Decimal,    // 0..=100
    confirmations: Vec<Confirmation>,        // event-sourced log
    scenario_id: Option<ScenarioId>,
    mrp_run_origin: Option<MrpRunId>,        // traceable back to MRP
    principal_id: PrincipalId,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProductionOrderState {
    Created,        // CRTD — converted from planned, not yet released
    Released,       // REL — released to shop floor; goods movement allowed
    PartiallyConfirmed, // PCNF
    Confirmed,      // CNF
    TechnicallyComplete, // TECO — frozen, no further confirmations
    Closed,         // CLSD — settled to costing
    Deleted,        // DLT — soft delete, audit retained
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProductionOrderOperation {
    step_no: StepNo,
    work_center_id: WorkCenterId,
    planned_start: Hlc,
    planned_finish: Hlc,
    planned_setup: Duration,
    planned_processing: Duration,
    planned_teardown: Duration,
    actual_setup: Option<Duration>,
    actual_processing: Option<Duration>,
    actual_teardown: Option<Duration>,
    qty_good: Decimal,
    qty_scrap: Decimal,
    partial_confirmed: bool,
    final_confirmed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Confirmation {
    confirmation_id: ConfirmationId,
    op_step_no: StepNo,
    qty_good: Decimal,
    qty_scrap: Decimal,
    scrap_reason_code: Option<ScrapReasonCode>,
    time_actual: Duration,
    posted_at: Hlc,
    posted_by: PrincipalId,
    final_for_op: bool,
}
```

### D-2. State machine

```rust
impl ProductionOrder {
    pub fn release(&mut self) -> Result<(), OrderError> {
        match self.state {
            ProductionOrderState::Created => {
                self.state = ProductionOrderState::Released;
                self.emit_event(OrderEvent::Released { at: Hlc::now() });
                Ok(())
            }
            _ => Err(OrderError::IllegalTransition { from: self.state, to: ProductionOrderState::Released }),
        }
    }

    pub fn technically_complete(&mut self) -> Result<(), OrderError> {
        match self.state {
            ProductionOrderState::Released
            | ProductionOrderState::PartiallyConfirmed
            | ProductionOrderState::Confirmed => {
                self.state = ProductionOrderState::TechnicallyComplete;
                self.emit_event(OrderEvent::TechnicallyComplete { at: Hlc::now() });
                Ok(())
            }
            _ => Err(OrderError::IllegalTransition { from: self.state, to: ProductionOrderState::TechnicallyComplete }),
        }
    }
}
```

Legal transitions (graph):

```text
Created --release-->  Released
Released --confirm--> PartiallyConfirmed
Released --confirm--> Confirmed
PartiallyConfirmed --confirm--> Confirmed
PartiallyConfirmed --teco--> TechnicallyComplete
Confirmed --teco--> TechnicallyComplete
TechnicallyComplete --close--> Closed
* --delete--> Deleted (only if no goods movements posted)
```

### D-3. Confirmation accumulator

```rust
impl ProductionOrder {
    pub fn record_confirmation(&mut self, c: Confirmation) -> Result<(), OrderError> {
        if matches!(self.state, ProductionOrderState::TechnicallyComplete
                              | ProductionOrderState::Closed
                              | ProductionOrderState::Deleted) {
            return Err(OrderError::FrozenAggregate);
        }
        if c.qty_good < Decimal::ZERO || c.qty_scrap < Decimal::ZERO {
            return Err(OrderError::NegativeQuantity);
        }
        let op = self.operations.iter_mut()
            .find(|o| o.step_no == c.op_step_no)
            .ok_or(OrderError::OperationMissing { step_no: c.op_step_no })?;
        op.qty_good += c.qty_good;
        op.qty_scrap += c.qty_scrap;
        op.actual_processing = op.actual_processing.map(|d| d + c.time_actual).or(Some(c.time_actual));
        if c.final_for_op { op.final_confirmed = true; } else { op.partial_confirmed = true; }
        if self.operations.iter().any(|o| !o.final_confirmed) {
            self.state = ProductionOrderState::PartiallyConfirmed;
        } else {
            self.state = ProductionOrderState::Confirmed;
        }
        self.qty_delivered = self.operations.iter().map(|o| o.qty_good).sum();
        self.qty_scrapped = self.operations.iter().map(|o| o.qty_scrap).sum();
        self.confirmations.push(c);
        Ok(())
    }
}
```

### D-4. Cost variance (pure function)

```rust
impl ProductionOrder {
    pub fn cost_variance(&self, rates: &WorkCenterRates) -> CostVariance {
        let planned_time_cost = self.operations.iter().map(|o| {
            rates.rate_for(&o.work_center_id) * (o.planned_setup + o.planned_processing + o.planned_teardown).as_decimal_hours()
        }).sum::<Decimal>();
        let actual_time_cost = self.operations.iter().map(|o| {
            let actual = o.actual_setup.unwrap_or_default() + o.actual_processing.unwrap_or_default() + o.actual_teardown.unwrap_or_default();
            rates.rate_for(&o.work_center_id) * actual.as_decimal_hours()
        }).sum::<Decimal>();
        CostVariance { planned: planned_time_cost, actual: actual_time_cost, variance: actual_time_cost - planned_time_cost }
    }
}
```

### D-5. Typed errors

```rust
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum OrderError {
    #[error("illegal transition from {from:?} to {to:?}")]
    IllegalTransition { from: ProductionOrderState, to: ProductionOrderState },
    #[error("aggregate is frozen (TECO/Closed/Deleted)")]
    FrozenAggregate,
    #[error("operation missing: step_no={step_no}")]
    OperationMissing { step_no: StepNo },
    #[error("negative quantity in confirmation")]
    NegativeQuantity,
    #[error("cross-tenant: {a} vs {b}")] CrossTenant,
    #[error("overdelivery exceeds tolerance pct {tol}")] OverdeliveryExceeded { tol: Decimal },
}
```

### D-6. Audit-event class

`EVT-PRODUCTION_PLANNING-PRODUCTION_ORDER-IP_ACCEPTED` per ADR-0263; envelope adds correlation back to MRP run via `mrp_run_origin`.

### D-7. SLO contribution

In-process: confirmation processing ≤ 500µs P95 per event. `cost_variance` ≤ 1ms P95 for 50-operation order.

### D-8. Cross-µservice handoffs

| Direction | Counterparty | Mode |
|---|---|---|
| outbound | `warehouse.outbound-reservation` (IP-017) | AsyncAPI `production-order-released.v1` |
| outbound | `quality-management.inspection-lot` | AsyncAPI on first confirmation per operation with inspection point |
| outbound | `costing.settlement` (substrate) | AsyncAPI on `Closed` transition |
| inbound | `plant-maintenance.equipment-availability` | gRPC read |

## E. Failure modes & recovery

### E-1. Illegal state transition attempted
**Detection:** `OrderError::IllegalTransition`.
**Behaviour:** Mutator rejects.
**Recovery:** Operator inspects current state and follows the legal graph; runbook `runbooks/order-state-transition.md`.

### E-2. Overdelivery beyond tolerance
**Detection:** `OrderError::OverdeliveryExceeded`.
**Behaviour:** Confirmation rejected; partial delivery still accepted.
**Recovery:** Engineer adjusts tolerance pct or splits into a new order.

### E-3. Confirmation after TECO
**Detection:** `OrderError::FrozenAggregate`.
**Behaviour:** Rejected.
**Recovery:** Either revoke TECO (privileged), or post a counter-confirmation against a new order.

### E-4. Cross-tenant BOM/routing snapshot
**Detection:** `OrderError::CrossTenant`.
**Behaviour:** Conversion aborted.
**Recovery:** Operator inspects tenant context.

## F. Migration

Phase 1: domain (this IP).
Phase 2 (IP-011): usecase.
Phase 3 (IP-013): adapter + outbox.
Phase 4 (IP-006 release / IP-017 warehouse handoff): downstream surfaces.

Rollback: feature flag `production_planning_production_order_v1` → false.

## G. References

- ADR-0105, ADR-0244, ADR-0252, ADR-0263, ADR-0294, ADR-0297, ADR-0315.
- SAP Help: PP-SFC `CO01`/`CO02`/`CO11N`/`CO15`; tables `AUFK`/`AFKO`/`AFPO`/`AFVC`/`AFRU`.
- Benchmarks: SAP S/4HANA PP-SFC | Oracle Fusion Cloud Manufacturing Work Orders | Microsoft Dynamics 365 SCM Production Orders | NetSuite Work Orders | SAP DM (Digital Manufacturing).

## H. Out-of-scope

- Goods movements (owned by `warehouse`).
- Costing settlement (owned by `costing`).
- Quality inspection (owned by `quality-management`).
- Persistence (IP-013).

— end IP-005 —
