---
doc_class: ImplementationPlan
microservice: production-planning
status: Accepted
date: 2026-05-20
owner_team: axis-production-planning + axis-erp-parity
related_adrs: [ADR-0105, ADR-0131, ADR-0244, ADR-0252, ADR-0257, ADR-0263, ADR-0294, ADR-0297, ADR-0314, ADR-0315, ADR-0316]
planned_enforcement_ref: oya-governance-production-planning-doc-set
ip_id: IP-002
journey_ref: j101
journey_slug: j101-multi-tier-supply-chain-formation
sap_submodule: PP-MRP (Material Requirements Planning, transactions MD01/MD02/MD03)
tenant_class: substrate
persona: mrp-controller
---

# IP-002: Domain layer for mrp-run

## A. Intent

The MRP run is the **scheduled or interactive execution** of the materials-requirements planning algorithm: given a finished-good demand horizon, BOM revisions, plant material masters, planning strategies, and lot-size keys, project net dependent requirements with explosion across BOM levels. In SAP S/4HANA this is `MD01` (total planning), `MD02` (single-item single-level), `MD03` (single-item multi-level), and `MD04` (stock/requirements list output). Tables `MDKP` (planning header), `MDTB` (planning detail), `T440P` (planning data).

This IP implements the **domain layer** for the `mrp-run` aggregate: pure types, the explosion algorithm (BFS over BOM revision), net-requirement calculation, and planning-strategy interpretation. Lot-size strategies are dispatched via trait. No I/O.

### A.1 Why MRP is a domain concept (not just an integration step)

The net-requirements computation is **purely a function of inputs** (gross demand, on-hand stock, scheduled receipts, safety stock, lot-size rule, planning horizon). If we leak the computation into the adapter layer we cannot replay it deterministically. SAP's PP-MRP suffers from this: replaying old MRP runs requires the **full MARC/MARD snapshot**, which SAP doesn't preserve. We do, via HLC snapshots (ADR-0297).

### A.2 SAP S/4HANA mapping

| SAP entity | Oyatie aggregate / value object |
|---|---|
| `MDKP` planning header | `MrpRun` aggregate root |
| `MDTB` planning row | `MrpPlanningRow` value object |
| `MDVM` planning file entry | `PlanningFileEntry` ephemeral (re-derived) |
| `T440P` MRP control | `PlanningStrategyPack` Cedar context (per-cell) |
| Lot-size key `EX` / `FX` / `HB` / `WB` / `MB` / `PK` / `Z*` | `LotSizeStrategy` enum + trait `LotSizingStrategy` |
| `EISLO` (safety stock indicator) | `SafetyStockPolicy` enum |
| `MARA.BESKZ` (procurement type) | `ProcurementType::{InHouse, External, Both}` |

### A.3 Journey leg

`j101-multi-tier-supply-chain-formation`: a Tier-1 OEM kicks off MRP for a finished good; explosion descends through the BOM revisions; dependent requirements emerge for Tier-2/3 components; IP-016 hands these off to supply-chain-planning. **This IP owns the algorithm**.

## B. Acceptance criteria

- **AC-1:** `MrpRun::explode(&bom_revision, &planning_inputs)` produces `Vec<MrpPlanningRow>` ordered by `level_no` then `position_no`; total time ≤ 25ms for breadth-200 depth-8 BOM (in-memory benchmark).
- **AC-2:** Net requirement = `gross_requirement - on_hand_stock - scheduled_receipts + safety_stock_target`; values clamped to ≥ 0 (no negative net suggests no procurement).
- **AC-3:** Lot-size dispatch: enum + trait `LotSizingStrategy::build_planned_lots(net_required: Decimal, params: &LotSizeParams) -> Vec<PlannedLot>`. Strategies: `LotForLot`, `FixedLot`, `MonthlyBucket`, `WeeklyBucket`, `PeriodOfSupply`, `BinPacking` (oyatie native).
- **AC-4:** `MrpRun::detect_anomalies()` finds: zero on-hand + zero scheduled receipts + non-zero gross (returns `AnomalyKind::SupplyGap`); on-hand > 10× rolling demand (returns `AnomalyKind::OverstockSuspected`).
- **AC-5:** Tenant invariant: every consumed `BomRevision`, `PlantMaterial`, and demand source must share `tenant_id` with the `MrpRun`; mixed-tenant input rejected.
- **AC-6:** Determinism: same inputs → same outputs (deterministic test fixture in `tests/fixtures/deterministic-mrp/`).
- **AC-7:** Scenario isolation: `scenario_id` (Option) tags rows so `production` vs `simulation` runs never collide.
- **AC-8:** Cedar default-deny preserved at every entry; explosion is a domain function, gating is at usecase boundary.

## C. Verification

```bash
cargo test -p oya-production-planning-mrp-domain -- explode_breadth_200_depth_8
cargo test -p oya-production-planning-mrp-domain -- net_requirement_clamps_to_zero
cargo test -p oya-production-planning-mrp-domain -- lot_for_lot_emits_one_lot_per_period
cargo test -p oya-production-planning-mrp-domain -- fixed_lot_sums_to_or_exceeds_demand
cargo test -p oya-production-planning-mrp-domain -- monthly_bucket_aggregates_within_month
cargo test -p oya-production-planning-mrp-domain -- bin_packing_minimizes_partial_pallets
cargo test -p oya-production-planning-mrp-domain -- detect_supply_gap_when_zero_supply
cargo test -p oya-production-planning-mrp-domain -- determinism_reference_fixture
cargo test -p oya-production-planning-mrp-domain -- cross_tenant_input_rejected
cargo bench -p oya-production-planning-mrp-domain -- explode_depth_8_breadth_200
```

Coverage ≥ 95% line, ≥ 90% branch. Property-based tests with `proptest` for invariants.

## D. Detailed mechanics

### D-1. Aggregate root

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct MrpRun {
    tenant_id: TenantId,
    mrp_run_id: MrpRunId,                     // ULID
    material_id: MaterialId,                  // root finished-good
    plant_code: PlantCode,
    planning_horizon: PlanningHorizon,        // start..=end (HLC)
    lot_size_strategy: LotSizeStrategy,
    safety_stock_policy: SafetyStockPolicy,
    scenario_id: Option<ScenarioId>,
    status: MrpRunStatus,
    started_at: Hlc,
    completed_at: Option<Hlc>,
    rows: Vec<MrpPlanningRow>,
    anomalies: Vec<MrpAnomaly>,
    principal_id: PrincipalId,
    policy_bundle_version: PolicyBundleVersion,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MrpPlanningRow {
    position_no: PositionNo,
    level_no: u8,                             // 0 = root
    parent_component_id: Option<MaterialId>,
    component_id: MaterialId,
    required_at: Hlc,
    gross_requirement: Decimal,
    on_hand_stock: Decimal,
    scheduled_receipts: Decimal,
    safety_stock_target: Decimal,
    net_requirement: Decimal,                 // clamped >= 0
    procurement_type: ProcurementType,
    planned_lots: Vec<PlannedLot>,
}
```

### D-2. Explosion algorithm (BFS, deterministic)

```rust
impl MrpRun {
    pub fn explode(
        &mut self,
        bom: &BomRevision,
        plant_materials: &HashMap<MaterialId, PlantMaterial>,
        demands: &[Demand],
    ) -> Result<(), MrpError> {
        if bom.tenant_id() != self.tenant_id { return Err(MrpError::CrossTenant); }

        let mut by_parent: HashMap<MaterialId, Vec<&BomPosition>> = HashMap::new();
        for pos in bom.positions() {
            let parent = pos.parent_component_id().clone().unwrap_or(self.material_id.clone());
            by_parent.entry(parent).or_default().push(pos);
        }

        let mut queue: VecDeque<(MaterialId, Decimal, u8)> =
            VecDeque::from(vec![(self.material_id.clone(), self.aggregate_demand(demands)?, 0u8)]);

        let mut next_position_no: u32 = 1;
        while let Some((parent_id, parent_qty, level)) = queue.pop_front() {
            if level > 32 { return Err(MrpError::ExplosionDepthExceeded { depth: level }); }
            let Some(children) = by_parent.get(&parent_id) else { continue; };
            for child in children {
                let gross_qty = parent_qty * child.quantity_per_assembly()
                    * (Decimal::ONE + child.scrap_pct() / Decimal::ONE_HUNDRED);
                let pm = plant_materials.get(child.component_id())
                    .ok_or_else(|| MrpError::PlantMaterialMissing { material_id: child.component_id().clone() })?;
                let net = self.net_requirement(gross_qty, pm)?;
                let row = MrpPlanningRow {
                    position_no: PositionNo(next_position_no),
                    level_no: level + 1,
                    parent_component_id: Some(parent_id.clone()),
                    component_id: child.component_id().clone(),
                    required_at: self.compute_required_at(level + 1, pm)?,
                    gross_requirement: gross_qty,
                    on_hand_stock: pm.on_hand(),
                    scheduled_receipts: pm.scheduled_receipts(),
                    safety_stock_target: pm.safety_stock_target(),
                    net_requirement: net,
                    procurement_type: pm.procurement_type(),
                    planned_lots: self.lot_size_strategy.build_planned_lots(net, &pm.lot_params())?,
                };
                next_position_no += 1;
                self.rows.push(row);
                if net > Decimal::ZERO && pm.procurement_type() != ProcurementType::External {
                    queue.push_back((child.component_id().clone(), net, level + 1));
                }
            }
        }
        Ok(())
    }
}
```

### D-3. Lot-size strategies (trait + enum dispatch)

```rust
pub trait LotSizingStrategy {
    fn build_planned_lots(
        &self,
        net_required: Decimal,
        params: &LotSizeParams,
    ) -> Result<Vec<PlannedLot>, MrpError>;
}

pub struct LotForLot;
pub struct FixedLot { pub size: Decimal, pub min: Decimal, pub max: Decimal, pub rounding: Decimal }
pub struct MonthlyBucket;
pub struct WeeklyBucket;
pub struct PeriodOfSupply { pub days: u16 }
pub struct BinPacking { pub bin_capacity: Decimal }   // oyatie-native FFD heuristic
```

Strategy choice maps to SAP keys: `EX`→LotForLot, `FX`→FixedLot, `MB`→MonthlyBucket, `WB`→WeeklyBucket, `PK`→PeriodOfSupply, `Z01-Z99`→customer-defined; `OYA_BIN_PACKING_V1`→BinPacking.

### D-4. Typed errors

```rust
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum MrpError {
    #[error("cross-tenant input")]
    CrossTenant,
    #[error("plant material missing for material_id={material_id}")]
    PlantMaterialMissing { material_id: MaterialId },
    #[error("explosion depth {depth} exceeds limit 32")]
    ExplosionDepthExceeded { depth: u8 },
    #[error("planning horizon end {end} must be > start {start}")]
    InvalidHorizon { start: Hlc, end: Hlc },
    #[error("invalid lot size")]
    InvalidLotSize,
    #[error("aggregate demand exceeds plant capacity multiplier 100")]
    DemandSanityExceeded,
}
```

### D-5. Anomaly detection

```rust
impl MrpRun {
    pub fn detect_anomalies(&mut self) {
        for row in &self.rows {
            if row.gross_requirement > Decimal::ZERO
                && row.on_hand_stock == Decimal::ZERO
                && row.scheduled_receipts == Decimal::ZERO
            {
                self.anomalies.push(MrpAnomaly::SupplyGap {
                    component_id: row.component_id.clone(),
                    required_at: row.required_at,
                });
            }
            if row.on_hand_stock > row.gross_requirement * Decimal::from(10) {
                self.anomalies.push(MrpAnomaly::OverstockSuspected {
                    component_id: row.component_id.clone(),
                    multiple: row.on_hand_stock / row.gross_requirement.max(Decimal::ONE),
                });
            }
        }
    }
}
```

### D-6. Determinism contract

Inputs (BOM, demands, plant materials, lot params, HLC `started_at`) → output rows. Fixture: `tests/fixtures/deterministic-mrp/case-1-acme/` is the canonical JSON snapshot of inputs + ordered output. Any algorithm or rounding change requires `MIGR-MRP-DETERMINISM-<YYYY-MM-DD>.md`.

### D-7. SLO contribution

In-process compute target: ≤ 25ms for 200-position depth-8 BOM. Feeds the IP-016 30s P95 budget by reserving ≥ 99% of latency to I/O + outbox + SCP dispatch.

### D-8. Audit-event class

`EVT-PRODUCTION_PLANNING-MRP_RUN-IP_ACCEPTED` per ADR-0263; envelope adds correlation/causation/principal at adapter layer.

### D-9. Cross-µservice handoff (read-only)

| Consumer | Read mode | Purpose |
|---|---|---|
| `supply-chain-planning` | AsyncAPI consumer (IP-016 emits) | Tier-N supplier dependent requirement projection |
| `procurement` | ontology projection `ontology.production_planning.mrp_planning_row` | Purchase requisition derivation |
| `costing` | ontology projection | Cost-roll-up using planned lots |
| `marketplace` | read-only ontology view | Settlement OWNED elsewhere per ADR-0314 |

## E. Failure modes & recovery

### E-1. Explosion depth > 32 (pathological BOM)

**Detection:** `MrpError::ExplosionDepthExceeded`.
**Behaviour:** Run halted with status `failed`; partial rows discarded (event-sourced — never persisted).
**Recovery:** BOM revisit; runbook `runbooks/mrp-depth-exceeded.md`.

### E-2. Missing PlantMaterial for a referenced component

**Detection:** `MrpError::PlantMaterialMissing { material_id }`.
**Behaviour:** Run halted; offending `material_id` returned.
**Recovery:** Operator creates the missing `MARC` equivalent (`plant_material` row in PP usecase); re-runs.

### E-3. Cross-tenant input

**Detection:** `MrpError::CrossTenant`.
**Behaviour:** Aggregate never advances. Security audit `EVT-SECURITY-CROSS_TENANT_ATTEMPT` emitted at usecase layer.
**Recovery:** Operator inspects principal token vs requested resource tenants.

### E-4. Determinism regression (deterministic fixture diff)

**Detection:** `cargo test determinism_reference_fixture` fails on a PR.
**Behaviour:** CI red; merge blocked.
**Recovery:** Either fix algorithm OR explicitly re-baseline via `MIGR-MRP-DETERMINISM-<YYYY-MM-DD>.md`.

## F. Migration

Phase 1: domain layer (this IP).
Phase 2 (IP-008): usecase wires to ports.
Phase 3 (IP-013): adapter persists run + rows via outbox.
Phase 4 (IP-016): SCP handoff event live.

Rollback: feature flag `production_planning_mrp_v1` to false.

## G. References

- ADR-0105, ADR-0244, ADR-0252 (HLC default), ADR-0263 (audit registry), ADR-0294 (Cedar soak), ADR-0297 (HLC defaults), ADR-0315 (SAP parity), ADR-0316 (audit anchoring).
- Orlicky, J. (1975). "Material Requirements Planning." McGraw-Hill — algorithmic origin.
- SAP Help: PP-MRP (transactions MD01/MD02/MD03/MD04, tables MDKP/MDTB).
- Benchmarks: SAP S/4HANA PP-MRP | Oracle Fusion MRP | Kinaxis RapidResponse pegging | Blue Yonder Luminate | o9 Solutions IBP.

## H. Out-of-scope

- Persistence (IP-013).
- Outbox dispatch (IP-013 + IP-016).
- Cross-µservice handoff (IP-016).
- Routing/capacity (IP-003 + IP-004 + IP-019 + IP-020).
- Marketplace settlement (owned by `marketplace` per ADR-0314).

— end IP-002 —
