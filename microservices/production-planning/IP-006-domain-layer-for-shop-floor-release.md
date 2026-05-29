---
doc_class: ImplementationPlan
microservice: production-planning
status: Accepted
date: 2026-05-20
owner_team: axis-production-planning + axis-erp-parity
related_adrs: [ADR-0105, ADR-0131, ADR-0244, ADR-0252, ADR-0263, ADR-0294, ADR-0297, ADR-0314, ADR-0315, ADR-0316]
planned_enforcement_ref: oya-governance-production-planning-doc-set
ip_id: IP-006
journey_ref: j101
journey_slug: j101-multi-tier-supply-chain-formation
sap_submodule: PP-REM (Repetitive Manufacturing) + PP-SFC (release transactions CO02/MFBF)
tenant_class: substrate
persona: shop-floor-supervisor
---

# IP-006: Domain layer for shop-floor-release

## A. Intent

Shop-floor release is the **gating moment** when a production order transitions from `Created` to `Released` and goods movements become legal. SAP's `CO02` transaction performs availability checks, work-center availability, capacity check, and PRT availability before the release flips. Errors at release time are the most expensive class of PP failure because they cascade into downstream warehouse, quality, and costing modules.

This IP implements the **domain layer** for `shop-floor-release`: the `ReleaseEvaluation` aggregate that captures the pre-release checks (material availability, capacity availability, PRT availability, quality clearance) and the `ReleaseDecision` value object with a typed outcome.

### A.1 SAP equivalence delta

| SAP | Oyatie |
|---|---|
| `CO02` release | `ProductionOrder::release()` (IP-005) — preconditioned by this eval |
| availability check `ATP` | `MaterialAvailabilityCheck` value object |
| capacity check `CRP` | `CapacityAvailabilityCheck` value object |
| `MFBF` repetitive manufacturing backflush | `BackflushPlan` value object |
| status checks `STAT` | folded into `ReleaseGate` enum |

### A.2 Journey leg

`j101`: after planned-order conversion and material/capacity provisioning, this evaluation must pass before the order can release.

## B. Acceptance criteria

- **AC-1:** `ReleaseEvaluation::evaluate()` returns `ReleaseDecision::Pass` or `Fail { reasons: Vec<FailureReason> }`.
- **AC-2:** All four sub-checks (material, capacity, PRT, quality clearance) run independently and accumulate failures (no short-circuit).
- **AC-3:** `MaterialAvailabilityCheck::missing_quantities()` returns per-component shortage.
- **AC-4:** `CapacityAvailabilityCheck::overload_intervals()` returns intervals where required hours exceed available.
- **AC-5:** Backflush plan: for `RoutingUsage::Production` with control-key `Pp01ProcessNoCostInfo`, generates per-operation backflush component list.
- **AC-6:** Tenant invariant across all four checks.
- **AC-7:** Cedar default-deny on every entry.
- **AC-8:** Outcome is event-sourced: `EVT-PRODUCTION_PLANNING-SHOP_FLOOR_RELEASE-IP_ACCEPTED`.

## C. Verification

```bash
cargo test -p oya-production-planning-release-domain -- happy_path_passes
cargo test -p oya-production-planning-release-domain -- material_shortage_accumulates_failures
cargo test -p oya-production-planning-release-domain -- capacity_overload_accumulates_failures
cargo test -p oya-production-planning-release-domain -- prt_conflict_accumulates_failures
cargo test -p oya-production-planning-release-domain -- quality_hold_blocks_release
cargo test -p oya-production-planning-release-domain -- backflush_plan_for_rem_orders
cargo test -p oya-production-planning-release-domain -- cross_tenant_input_rejected
cargo test -p oya-production-planning-release-domain -- all_four_checks_run_no_short_circuit
cargo bench -p oya-production-planning-release-domain -- evaluate_order_50_components_20_ops
```

## D. Detailed mechanics

### D-1. Aggregate

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseEvaluation {
    tenant_id: TenantId,
    order_id: ProductionOrderId,
    material_check: MaterialAvailabilityCheck,
    capacity_check: CapacityAvailabilityCheck,
    prt_check: PrtAvailabilityCheck,
    quality_clearance: QualityClearanceCheck,
    evaluated_at: Hlc,
    principal_id: PrincipalId,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReleaseDecision {
    Pass { backflush_plan: Option<BackflushPlan> },
    Fail { reasons: Vec<FailureReason> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum FailureReason {
    MaterialShortage { component_id: MaterialId, shortage: Decimal },
    CapacityOverload { work_center_id: WorkCenterId, interval: Range<Hlc>, deficit_hours: Decimal },
    PrtConflict { prt_id: PrtId, conflicting_window: Range<Hlc> },
    QualityHold { hold_id: QualityHoldId, reason: String },
    CrossTenantContext,
    ScenarioRunNotConvertible,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackflushPlan {
    operations: Vec<BackflushOperation>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct BackflushOperation {
    step_no: StepNo,
    components: Vec<BackflushComponent>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct BackflushComponent {
    component_id: MaterialId,
    quantity_per_unit: Decimal,
    consumption_point: ConsumptionPoint,    // OnReceipt | OnConfirmation | EndOfRun
}
```

### D-2. Evaluation orchestration

```rust
impl ReleaseEvaluation {
    pub fn evaluate(&self) -> ReleaseDecision {
        let mut reasons = Vec::new();
        if let Err(es) = self.material_check.evaluate() { reasons.extend(es); }
        if let Err(es) = self.capacity_check.evaluate() { reasons.extend(es); }
        if let Err(es) = self.prt_check.evaluate() { reasons.extend(es); }
        if let Err(es) = self.quality_clearance.evaluate() { reasons.extend(es); }
        if reasons.is_empty() {
            let bp = self.derive_backflush_plan();
            ReleaseDecision::Pass { backflush_plan: bp }
        } else {
            ReleaseDecision::Fail { reasons }
        }
    }
}
```

### D-3. Material availability check

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct MaterialAvailabilityCheck {
    required_components: Vec<RequiredComponent>,
    availability_view: AvailabilitySnapshot,
}
impl MaterialAvailabilityCheck {
    pub fn evaluate(&self) -> Result<(), Vec<FailureReason>> {
        let mut fails = Vec::new();
        for rc in &self.required_components {
            let avail = self.availability_view.get(&rc.component_id);
            if avail < rc.required_qty {
                fails.push(FailureReason::MaterialShortage {
                    component_id: rc.component_id.clone(),
                    shortage: rc.required_qty - avail,
                });
            }
        }
        if fails.is_empty() { Ok(()) } else { Err(fails) }
    }
}
```

### D-4. Capacity availability check

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct CapacityAvailabilityCheck {
    operation_requirements: Vec<OperationRequirement>,
    capacity_projection: HashMap<WorkCenterId, Vec<CapacityInterval>>,
}
impl CapacityAvailabilityCheck {
    pub fn evaluate(&self) -> Result<(), Vec<FailureReason>> {
        let mut fails = Vec::new();
        for op in &self.operation_requirements {
            let avail = self.capacity_projection.get(&op.work_center_id)
                .map(|ivs| ivs.iter().filter(|iv| iv.overlaps(&op.window)).map(|iv| iv.available_minutes).sum::<Decimal>())
                .unwrap_or(Decimal::ZERO);
            let required = (op.setup + op.processing + op.teardown).as_decimal_minutes();
            if avail < required {
                fails.push(FailureReason::CapacityOverload {
                    work_center_id: op.work_center_id.clone(),
                    interval: op.window.clone(),
                    deficit_hours: ((required - avail) / Decimal::from(60)),
                });
            }
        }
        if fails.is_empty() { Ok(()) } else { Err(fails) }
    }
}
```

### D-5. PRT availability check

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct PrtAvailabilityCheck {
    operation_prt_refs: Vec<(StepNo, PrtRef, Range<Hlc>)>,
    prt_reservations: HashMap<PrtId, Vec<Range<Hlc>>>,
}
impl PrtAvailabilityCheck {
    pub fn evaluate(&self) -> Result<(), Vec<FailureReason>> {
        let mut fails = Vec::new();
        for (_step, prt_ref, window) in &self.operation_prt_refs {
            let res = self.prt_reservations.get(&prt_ref.prt_id);
            if let Some(existing) = res {
                if existing.iter().any(|r| ranges_overlap(r, window)) {
                    fails.push(FailureReason::PrtConflict {
                        prt_id: prt_ref.prt_id.clone(),
                        conflicting_window: window.clone(),
                    });
                }
            }
        }
        if fails.is_empty() { Ok(()) } else { Err(fails) }
    }
}
```

### D-6. Quality clearance check

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct QualityClearanceCheck {
    holds: Vec<QualityHold>,
    material_id: MaterialId,
}
impl QualityClearanceCheck {
    pub fn evaluate(&self) -> Result<(), Vec<FailureReason>> {
        let active: Vec<_> = self.holds.iter()
            .filter(|h| h.material_id == self.material_id && h.is_active())
            .collect();
        if active.is_empty() { Ok(()) }
        else {
            Err(active.into_iter().map(|h| FailureReason::QualityHold {
                hold_id: h.hold_id.clone(), reason: h.reason.clone(),
            }).collect())
        }
    }
}
```

### D-7. Typed errors

```rust
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ReleaseError {
    #[error("cross-tenant context")] CrossTenant,
    #[error("scenario order cannot release to shop floor")] ScenarioNotConvertible,
    #[error("evaluation already finalised")] EvaluationFinalised,
}
```

### D-8. Audit-event class

`EVT-PRODUCTION_PLANNING-SHOP_FLOOR_RELEASE-IP_ACCEPTED` per ADR-0263.

### D-9. SLO contribution

In-process: full evaluation ≤ 8ms P95 for a 50-component, 20-operation order.

### D-10. Cross-µservice handoffs

| Direction | Counterparty | Mode |
|---|---|---|
| inbound | `warehouse` ATP snapshot | gRPC pull |
| inbound | `quality-management` active holds | gRPC pull |
| inbound | `plant-maintenance` PRT reservations | gRPC pull |
| outbound (on Pass) | `production-order` (IP-005) | in-process state transition |
| outbound (on Pass) | `warehouse.outbound-reservation` (IP-017) | AsyncAPI `production-order-released.v1` |

## E. Failure modes & recovery

### E-1. Material shortage at release
**Detection:** `FailureReason::MaterialShortage`.
**Behaviour:** Release blocked; partial shortages enumerated.
**Recovery:** Procurement is triggered for short components; runbook `runbooks/release-material-shortage.md`.

### E-2. Capacity overload across multiple work-centers
**Detection:** Multiple `FailureReason::CapacityOverload` rows.
**Behaviour:** Release blocked.
**Recovery:** IP-022 alt-routing branch engages; or shift overtime overlay added (IP-009 capacity-calendar).

### E-3. PRT conflict
**Detection:** `FailureReason::PrtConflict`.
**Behaviour:** Release blocked.
**Recovery:** Operator reschedules conflicting reservation or reroutes through alt PRT (IP-021).

### E-4. Quality hold active
**Detection:** `FailureReason::QualityHold`.
**Behaviour:** Release blocked indefinitely until QM clears the hold.
**Recovery:** Quality manager closes the notification (IP-QM); release re-evaluated.

## F. Migration

Phase 1: domain (this IP).
Phase 2 (IP-012): usecase orchestrator.
Phase 3 (IP-013): adapter pulls inputs via gRPC; outbox dispatches result.

Rollback: feature flag `production_planning_shop_floor_release_v1` → false.

## G. References

- ADR-0105, ADR-0244, ADR-0252, ADR-0263, ADR-0294, ADR-0297, ADR-0315.
- SAP Help: PP-SFC `CO02` release; PP-REM `MFBF` backflush.
- Benchmarks: SAP S/4HANA PP-SFC/PP-REM | Oracle Fusion Work Order Release | Microsoft Dynamics 365 SCM Released Production Orders | SAP DM (Digital Manufacturing).

## H. Out-of-scope

- Material reservation execution (warehouse).
- PRT calibration cycle (plant-maintenance).
- Quality notification creation (quality-management).
- Persistence (IP-013).

— end IP-006 —
