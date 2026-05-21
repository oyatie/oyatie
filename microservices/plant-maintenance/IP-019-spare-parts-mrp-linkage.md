---
doc_class: ImplementationPlan
ip_id: IP-019
microservice: plant-maintenance
related_adrs: [ADR-0105, ADR-0131, ADR-0244, ADR-0252, ADR-0257, ADR-0263, ADR-0294, ADR-0297, ADR-0314, ADR-0315, ADR-0329, ADR-0330, ADR-0331]
journey_ref: j208-asset-lifecycle-and-maintenance
sap_submodule: PM-MRP linkage to PP-MRP (Materials Requirements Planning); SAP MD04 (stock/requirements list), MD05 (MRP list), MRP-types VB/V1/V2 + reorder-point; bridging PM reservations and production planning
service_surface: substrate
persona: darren-chen (planner), elena-volkov (stock-controller), maya-okafor (reliability), priya-singh (plant manager)
status: Accepted
date: 2026-05-20
owner_team: axis-plant-maintenance + axis-supply-chain
planned_enforcement_ref: oya-governance-plant-maintenance-doc-suite
---

# IP-019: Spare-parts MRP linkage — PM-MRP ↔ PP-MRP integration with reorder-point + DDMRP

## A. Intent

Implements the **integration seam** between plant-maintenance reservations (IP-004/010) and the supply-chain procurement-planning µservice's MRP engine. PM-reservations are *demand signals* that participate in MRP runs alongside production-order component demand. Critical spares trigger procurement requisitions when ATP-shortfalls cannot be satisfied from on-hand stock.

Mirrors SAP `MD04` (stock/requirements list) where PM reservations sit alongside `PRorder` (production) and `PldOrd` (planned order) lines; MRP-type controls per SAP `T438M` table: `VB` (manual reorder-point), `V1` (manual reorder-point with external requirements), `V2` (automatic reorder-point), `R1` (time-phased), `PD` (deterministic). Integrates with IP-018 from production-planning's DDMRP buffer logic to consume `ddmrp.buffer-breached-red.v1` events as procurement triggers for critical spares.

Industry-precedent equivalents: SAP PP/MM MRP + PM reservation cross-link; **IBM Maximo Inventory + Maximo Procurement integration**; **Infor EAM Storeroom + Inventory Replenishment**; **Oracle Fusion Inventory + Procurement Cloud**; **IFS Cloud Inventory + Purchase Order**; **Demand Driven Technologies Replenishment+** (DDMRP overlay).

### A.1 Why MRP linkage is non-trivial

1. **Critical-spare classification.** Not all reservations should fire procurement on shortfall — only those flagged `is_critical_spare = true` (typically ABC-A equipment + lead-time > on-hand-coverage).
2. **DDMRP buffer-aware sourcing.** When a spare is DDMRP-managed, PM-reservation drains the buffer; red-zone breach fires planned-order via PP DDMRP (IP-018 of production-planning).
3. **Reorder-point trigger.** Conventional MRP-type V2 spares: PM-reservation creation may push on-hand below reorder-point → automatic PR (purchase requisition) generation.
4. **Lead-time gap handling.** If lead-time > time-to-WO-execution, surfaces gap as `lead-time-gap-event`; planner may approve expedite (Cedar-gated).
5. **Critical-spares safety stock.** Per ISO 55000, critical spares maintain safety stock independently of demand forecast; the integration must respect safety-stock floor.
6. **Cross-µservice transactionality.** PM-reservation write + MRP demand-signal emit must be atomic (outbox pattern).

## B. Acceptance criteria

- **AC-1:** PM-reservation creation emits `pm-mrp-demand-added.v1` envelope with `is_critical_spare`, `lead_time_days`, `mrp_type`.
- **AC-2:** `EvaluateMrpTriggerUseCase` consumes reservation events + ATP shortfall events → emits PR-request when reorder-point crossed.
- **AC-3:** DDMRP buffer-breached-red.v1 consumer creates procurement-trigger event for critical spares.
- **AC-4:** Lead-time gap detection: if `(planned_start - now) < lead_time + safety_buffer`, emit `lead-time-gap-event`.
- **AC-5:** Cedar `expedite_procurement` permit required for lead-time gap override.
- **AC-6:** Critical-spare safety-stock floor: ATP excludes safety-stock layer; reservations against safety-stock require Cedar `safety_stock_breach` permit.
- **AC-7:** MRP demand signal idempotent on `(reservation_id, demand_seq)`.
- **AC-8:** Cross-µservice handoff to procurement-planning via gRPC + AsyncAPI.
- **AC-9:** Cross-tenant inputs rejected.
- **AC-10:** Audit events per §D-9.

## C. Verification

```bash
cargo test -p oya-plant-maintenance-mrp-linkage -- pm_reservation_emits_demand
cargo test -p oya-plant-maintenance-mrp-linkage -- critical_spare_flag_propagated
cargo test -p oya-plant-maintenance-mrp-linkage -- reorder_point_trigger_pr_request
cargo test -p oya-plant-maintenance-mrp-linkage -- ddmrp_red_breach_consumer
cargo test -p oya-plant-maintenance-mrp-linkage -- lead_time_gap_detected
cargo test -p oya-plant-maintenance-mrp-linkage -- expedite_requires_cedar
cargo test -p oya-plant-maintenance-mrp-linkage -- safety_stock_floor_respected
cargo test -p oya-plant-maintenance-mrp-linkage -- safety_stock_breach_requires_cedar
cargo test -p oya-plant-maintenance-mrp-linkage -- mrp_demand_signal_idempotent
cargo test -p oya-plant-maintenance-mrp-linkage -- cross_tenant_rejected
```

## D. Detailed mechanics

### D-1. Data model

```sql
CREATE TABLE plant_maintenance.spare_part_master (
    tenant_id        TEXT NOT NULL,
    material_id      TEXT NOT NULL,
    plant_code       TEXT NOT NULL,
    is_critical_spare BOOLEAN NOT NULL DEFAULT FALSE,
    mrp_type         TEXT NOT NULL CHECK (mrp_type IN ('VB','V1','V2','R1','PD','NONE')),
    reorder_point    NUMERIC(14,4),
    safety_stock     NUMERIC(14,4),
    lead_time_days   INTEGER NOT NULL DEFAULT 7,
    ddmrp_buffer_profile_id TEXT,
    abc_class        TEXT CHECK (abc_class IN ('A','B','C')),
    residency_pack   TEXT NOT NULL,
    hlc              TEXT NOT NULL,
    PRIMARY KEY (tenant_id, material_id, plant_code)
) PARTITION BY HASH (tenant_id);

CREATE TABLE plant_maintenance.mrp_demand_signal (
    tenant_id      TEXT NOT NULL,
    demand_id      TEXT NOT NULL,
    reservation_id TEXT NOT NULL,
    material_id    TEXT NOT NULL,
    plant_code     TEXT NOT NULL,
    qty            NUMERIC(14,4) NOT NULL,
    required_by    TIMESTAMPTZ NOT NULL,
    demand_seq     INTEGER NOT NULL,
    state          TEXT NOT NULL CHECK (state IN ('emitted','acknowledged_by_mrp','satisfied','expedited','cancelled')),
    pr_id          TEXT,
    expedite_decision_id UUID,
    hlc            TEXT NOT NULL,
    PRIMARY KEY (tenant_id, demand_id)
) PARTITION BY HASH (tenant_id);

CREATE TABLE plant_maintenance.lead_time_gap_audit (
    tenant_id    TEXT NOT NULL,
    reservation_id TEXT NOT NULL,
    material_id  TEXT NOT NULL,
    gap_days     INTEGER NOT NULL,
    detected_at  TIMESTAMPTZ NOT NULL,
    resolution   TEXT NOT NULL CHECK (resolution IN ('expedited','wo_delayed','alternate_used','cancelled')),
    decision_id  UUID,
    hlc          TEXT NOT NULL,
    PRIMARY KEY (tenant_id, reservation_id, detected_at)
) PARTITION BY HASH (tenant_id);
```

### D-2. Rust types

```rust
#[derive(Debug, Clone)]
pub struct SparePartMaster {
    pub tenant_id:        TenantId,
    pub material_id:      MaterialId,
    pub plant_code:       PlantCode,
    pub is_critical_spare: bool,
    pub mrp_type:         MrpType,
    pub reorder_point:    Option<Decimal>,
    pub safety_stock:     Option<Decimal>,
    pub lead_time_days:   u32,
    pub ddmrp_buffer_profile_id: Option<BufferProfileId>,
    pub abc_class:        Option<AbcIndicator>,
}

#[derive(Debug, Clone)]
pub enum MrpType { VB, V1, V2, R1, PD, None }

#[derive(Debug, Clone)]
pub struct MrpDemandSignal {
    pub demand_id:      DemandId,
    pub reservation_id: ReservationId,
    pub material_id:    MaterialId,
    pub plant_code:     PlantCode,
    pub qty:            Decimal,
    pub required_by:    DateTime<Utc>,
    pub demand_seq:     u32,
    pub state:          DemandState,
    pub hlc:            Hlc,
}
```

### D-3. Lead-time gap detector

```rust
pub fn detect_lead_time_gap(reservation: &Reservation, master: &SparePartMaster,
                             now: DateTime<Utc>, atp: Decimal) -> Option<LeadTimeGap>
{
    let demand_qty: Decimal = reservation.items.iter().map(|i| i.planned_qty).sum();
    if atp >= demand_qty { return None; }
    let time_to_execution_days = (reservation.required_by() - now).num_days();
    let lead_time_with_safety = master.lead_time_days as i64 + 2; // 2-day safety buffer
    if time_to_execution_days < lead_time_with_safety {
        Some(LeadTimeGap {
            material_id: master.material_id.clone(),
            short_by: demand_qty - atp,
            gap_days: (lead_time_with_safety - time_to_execution_days) as u32,
        })
    } else { None }
}
```

### D-4. MRP-trigger use-case

```rust
#[async_trait]
impl UseCase for EvaluateMrpTriggerUseCase<R, SP, PROC, OUT> {
    type Input = MrpTriggerInput;
    type Output = MrpDecision;

    async fn execute(&self, input: Self::Input, ctx: RequestContext) -> Result<MrpDecision, UseCaseError> {
        if input.tenant_id != ctx.tenant_id { return Err(UseCaseError::CrossTenant); }
        let master = self.master_repo.load(&input.tenant_id, &input.material_id, &input.plant_code).await?
            .ok_or(UseCaseError::SpareMasterMissing)?;
        let atp_after_reservation = self.inv.atp(&input.tenant_id, &input.material_id, &input.plant_code).await?;

        let needs_trigger = match master.mrp_type {
            MrpType::V1 | MrpType::V2 => master.reorder_point.map_or(false, |rop| atp_after_reservation < rop),
            MrpType::R1 | MrpType::PD => input.demand_qty > Decimal::ZERO,   // deterministic always re-plans
            MrpType::VB => false,    // manual only — surface advisory
            MrpType::None => false,
        };
        if needs_trigger {
            let pr = self.procurement.request_pr(PrRequest {
                tenant_id: input.tenant_id.clone(),
                material_id: input.material_id.clone(),
                plant_code: input.plant_code.clone(),
                qty: input.demand_qty * Decimal::from(2),  // round up to reorder-quantity in real impl
                required_by: input.required_by,
                is_critical_spare: master.is_critical_spare,
                source_reservation_id: input.reservation_id.clone(),
            }).await?;
            Ok(MrpDecision::PrCreated(pr))
        } else {
            Ok(MrpDecision::Sufficient)
        }
    }
}
```

### D-5. DDMRP red-breach consumer

```rust
pub async fn run_ddmrp_red_breach_consumer(ctx: AppContext, mut rx: KafkaConsumer<DdmrpBufferBreachedRedEvent>) {
    let uc = EvaluateMrpTriggerUseCase::new(/*deps*/);
    while let Some(msg) = rx.next().await {
        let mat = msg.payload.material_id.clone();
        let plant = msg.payload.plant_code.clone();
        // Check if any PM critical-spare master matches
        if let Ok(Some(master)) = ctx.master_repo.load(&msg.payload.tenant_id, &mat, &plant).await {
            if master.is_critical_spare {
                let _ = uc.execute(MrpTriggerInput {
                    tenant_id: msg.payload.tenant_id.clone(),
                    material_id: mat, plant_code: plant,
                    reservation_id: ReservationId::synthetic_from_ddmrp(&msg.payload),
                    demand_qty: msg.payload.suggested_replenishment_qty,
                    required_by: msg.payload.replenishment_required_by,
                }, RequestContext::from_kafka(&msg)).await;
            }
        }
        msg.commit_offset().await;
    }
}
```

### D-6. Cedar context (expedite)

```jsonc
{
  "principal": "oyatie::tenant::acme::user::maintenance-planner-3",
  "action":    "plant_maintenance::mrp::expedite_procurement",
  "resource":  "plant_maintenance::mrp_demand::DMD-2026-887611",
  "context": {
    "tenant_id": "acme",
    "material_id": "MAT-BRG-9F",
    "is_critical_spare": true,
    "abc_class": "A",
    "gap_days": 4,
    "expedite_cost_estimate": "12500.00",
    "data_class": "operational",
    "policy_bundle_version": "2026.05.20-r3",
    "residency_pack": "global",
    "byok_mode": "platform_default"
  }
}
```

### D-7. AsyncAPI envelopes

| Channel | Trigger | Consumers |
|---|---|---|
| `plant-maintenance.mrp.demand-added.v1` | reservation create | procurement-planning, analytics |
| `plant-maintenance.mrp.demand-acknowledged.v1` | PR raised | analytics |
| `plant-maintenance.mrp.lead-time-gap.v1` | gap detected | planner UI, alerting |
| `plant-maintenance.mrp.expedited.v1` | Cedar permit + PR raised | finops (expedite cost) |
| `plant-maintenance.mrp.safety-stock-breach.v1` | reservation depletes safety stock | safety-authority alert |
| (inbound) `production-planning.ddmrp.buffer-breached-red.v1` | DDMRP red zone | trigger MRP-eval |
| (inbound) `procurement-planning.pr-created.v1` | PR raised | acknowledge demand |
| (inbound) `procurement-planning.pr-delivered.v1` | parts arrive | reservation reconcile |

### D-8. SLO targets

| Operation | p50 | p95 | p99 |
|---|---|---|---|
| `pm-mrp-demand-added` emit | 4 ms | 10 ms | 22 ms |
| `EvaluateMrpTriggerUseCase` | 35 ms | 80 ms | 160 ms |
| Lead-time gap detect | 6 ms | 14 ms | 28 ms |
| DDMRP red-breach consumer step | 25 ms | 60 ms | 120 ms |
| End-to-end reservation → PR raised | 250 ms | 580 ms | 1.2 s |

### D-9. Audit-event registry

| Event class | Severity | Emitter |
|---|---|---|
| `EVT-PLANT_MAINTENANCE-MRP-DEMAND_EMITTED` | informational | usecase |
| `EVT-PLANT_MAINTENANCE-MRP-PR_RAISED` | informational | usecase |
| `EVT-PLANT_MAINTENANCE-MRP-LEAD_TIME_GAP` | warning | usecase |
| `EVT-PLANT_MAINTENANCE-MRP-EXPEDITED` | informational | usecase |
| `EVT-PLANT_MAINTENANCE-MRP-SAFETY_STOCK_BREACHED` | warning | usecase |
| `EVT-PLANT_MAINTENANCE-MRP-DDMRP_RED_CONSUMED` | informational | consumer |
| `EVT-PLANT_MAINTENANCE-MRP-CROSS_TENANT_REJECTED` | security | usecase |

### D-10. Failure modes & recovery

1. **`PrCreationFailed`** — procurement gRPC down. Demand signal persists in `emitted`; cron retry every 60s. Runbook `runbooks/pr-creation-failed.md`.
2. **`LeadTimeGapUnresolved`** — gap detected, no resolution chosen. Auto-alert at gap >2 days; planner UI prompt. Runbook `runbooks/lead-time-gap-stuck.md`.
3. **`SafetyStockBreached`** — reservation forced through despite floor. Cedar permit must capture decision_id; ops audit. Runbook `runbooks/safety-stock-breach.md`.
4. **`DdmrpProducerLag`** — DDMRP red events delayed. PM falls back to its own reorder-point check. Runbook `runbooks/ddmrp-lag.md`.
5. **`MasterDataStale`** — spare-part master row stale (e.g., lead-time changed). Cron refresh every 6h from `inventory-management`. Runbook `runbooks/master-data-stale.md`.
6. **`ExpediteCostBudgetExceeded`** — expedite cost > tenant cap. Cedar denies; escalate to plant manager. Runbook `runbooks/expedite-budget.md`.

### D-11. Migration notes

Sources: SAP `MARC` (material plant data) + `MARD` (storage location stock) + `T438M` (MRP type) for spare-part master; SAP `MD04/MD05` outputs joined to PM-reservation rows for the integration seam.

### D-12. Cross-µservice handoffs

| Direction | Counterparty | Surface |
|---|---|---|
| outbound | `procurement-planning` | gRPC `procurement.v1.RequestPr` + AsyncAPI `mrp.demand-added.v1` |
| outbound | `oya-cloud-finops` | AsyncAPI `mrp.expedited.v1` (cost capture) |
| inbound | `production-planning` | AsyncAPI `ddmrp.buffer-breached-red.v1` |
| inbound | `inventory-management` | gRPC `inventory.v1.AtpAndSafetyStock` |
| outbound | `audit-chain` | per ADR-0263 |
| outbound | `ontology` | spare-part-master delta on update |

## E. Failure-mode summary

See D-10.

## F. Migration / rollback

Per-MRP-type feature flag (e.g., `plant_maintenance_mrp_v2_v1`). DDMRP consumer can be paused per tenant.

## G. References

- ADR-0105, ADR-0244, ADR-0252, ADR-0263, ADR-0294, ADR-0297, ADR-0314..0316.
- SAP `MD04/MD05/T438M/MARC` documentation.
- IP-018 of production-planning (DDMRP buffer profile).
- IP-004/010 (reservation domain + use-case).
- Ptak & Smith, *Demand Driven Material Requirements Planning* (3rd ed., 2018).

## H. Out of scope

- Procurement-order placement (lives in procurement-planning), inventory master (inventory-management), DDMRP buffer engine (production-planning IP-018).

— end IP-019 —
