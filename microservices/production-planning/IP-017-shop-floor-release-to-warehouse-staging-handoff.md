---
doc_class: ImplementationPlan
microservice: production-planning
status: Accepted
date: 2026-05-21
owner_team: axis-production-planning + axis-erp-parity
related_adrs: [ADR-0105, ADR-0131, ADR-0244, ADR-0252, ADR-0253, ADR-0257, ADR-0263, ADR-0294, ADR-0314, ADR-0315, ADR-0316]
ip_id: IP-017
journey_ref: j101
sap_submodule: PP-SFC + EWM-RF
peer_microservice: warehouse
---

# IP-017: Shop-Floor Release → Warehouse Staging (production-planning ↔ warehouse)

## A. Intent
When a production order moves from `created` → `released` (SAP `CO02` status `REL`), warehouse EWM-RF must stage the raw-material picks against the order's reservation. This IP implements that handoff for journey **j101-multi-tier-supply-chain-formation**.

### A.1 SAP delta
| SAP S/4HANA primitive | Oyatie equivalent | Concrete delta |
|---|---|---|
| `AFKO.AUFNR` production-order header | `production_planning.production_order` row | ULID PK + tenant_id |
| Order status `REL` (released) | `lifecycle_state = "released"` | Explicit state-machine in code, not flag-bits |
| `RESB` reservation | `production_planning.order_reservation` | Event-sourced |
| EWM warehouse-task | `warehouse.staging_task` | Created by warehouse on event receipt |
| `MIGO` 261 goods-issue | `warehouse.goods_issue.v1` event | Emitted by warehouse when staging consumed |
| LE-TRA shipment | `transportation.shipment.v1` | Out of scope here |

### A.2 Journey leg
Per j101, after Tier-1 OEM's MRP run completes (IP-016) and SCP confirms supplier deliveries arrive, the production order is released — warehouse must stage materials with FIFO/FEFO honored.

### A.3 Hyperscaler precedent
This is the **Manhattan Active WMS production-replenishment integration** pattern; adjacent: **Amazon FC inbound-to-pick wave**.

Benchmarks: SAP S/4HANA PP-SFC + EWM | Oracle Fusion Manufacturing | Manhattan Active WMS | Blue Yonder WMS | Körber HighJump.

## B. Acceptance criteria
- AC-1: `PATCH /api/v1/production-orders/{id}` with `{action: "release"}` transitions state in ≤200ms P95.
- AC-2: For every reservation line, exactly one `ORDER-RELEASED-TO-FLOOR` event emitted on `production-planning.order-released.v1`.
- AC-3: Warehouse creates a `staging_task` per line within 3s P95 (ACK on `warehouse.staging-task-created.v1`).
- AC-4: Cedar gate `production_planning::order::release` denies if any reservation references material not authorized for principal's plant.
- AC-5: Audit `EVT-PRODUCTION_PLANNING-ORDER_RELEASE-IP_ACCEPTED` signed per ADR-0296.
- AC-6: FIFO/FEFO source-bin selection honored by warehouse (verified in integration test).

## C. Verification
```
cargo test -p oya-production-planning-order -- release_happy_path
cargo test -p oya-production-planning-order -- release_blocked_when_capacity_unconfirmed
cargo test -p oya-production-planning-warehouse-integration -- staging_task_created_within_3s
```

## D. Detailed mechanics

### D-1. API
```
PATCH /api/v1/production-orders/PO-01HXYZ
{ "action": "release", "release_horizon_hours": 24, "force": false }
```
Response 200:
```json
{ "production_order_id": "PO-01HXYZ", "lifecycle_state": "released", "released_at": "2026-05-21T08:00:00Z",
  "warehouse_staging_request_id": "WS-01HXYZ" }
```

### D-2. Workflow Studio branches
| Branch | Trigger | Next |
|---|---|---|
| `capacity_unconfirmed` | Work-center capacity not pegged | Block; require IP-020 finite-schedule re-confirm |
| `materials_short` | ATP says component out of stock | Fork → SCP `expedite-supplier-delivery` |
| `quality_hold_present` | Component on quality hold (IP-018 QM) | Block; surface QM notification |
| `happy_path` | All preconditions met | Emit `ORDER-RELEASED-TO-FLOOR` → warehouse staging |

### D-3. Data model
```sql
ALTER TABLE production_planning.production_order
  ADD COLUMN released_at TIMESTAMPTZ,
  ADD COLUMN release_horizon_hours INT,
  ADD COLUMN released_by_principal_id TEXT;

CREATE TABLE production_planning.order_release_outbox (
  tenant_id UUID NOT NULL,
  event_id ULID PRIMARY KEY,
  production_order_id ULID NOT NULL,
  payload_jsonb JSONB NOT NULL,
  signature BYTEA NOT NULL,
  dispatch_state TEXT NOT NULL DEFAULT 'pending'
);
```

### D-4. AsyncAPI event `production-planning.order-released.v1`
```yaml
payload:
  type: object
  required: [event_id, tenant_id, production_order_id, reservations]
  properties:
    event_id: {type: string, format: ulid}
    tenant_id: {type: string, format: uuid}
    production_order_id: {type: string, format: ulid}
    material_id: {type: string}
    plant_code: {type: string}
    released_at: {type: string, format: date-time}
    hlc: {type: string}
    correlation_id: {type: string}
    audit_event_class: {const: "EVT-PRODUCTION_PLANNING-ORDER_RELEASE-IP_ACCEPTED"}
    reservations:
      type: array
      items:
        type: object
        required: [reservation_id, component_id, quantity_required, uom, source_strategy]
        properties:
          reservation_id: {type: string, format: ulid}
          component_id: {type: string}
          quantity_required: {type: string}
          uom: {type: string}
          source_strategy: {type: string, enum: [FIFO, FEFO, LIFO, NEAREST_BIN]}
          required_by: {type: string, format: date-time}
```

### D-5. Cedar fragment
```cedar
@id("production_planning::order::release::v1")
permit (
  principal in ProductionPlanning::Operator::"role-floor-supervisor",
  action == ProductionPlanning::Action::"release_order",
  resource in ProductionPlanning::ProductionOrder::?
) when {
  context.tenant_id == resource.tenant_id &&
  resource.plant_code in context.principal.authorized_plants &&
  resource.lifecycle_state == "created" &&
  resource.capacity_confirmed == true &&
  resource.quality_holds_count == 0
};
forbid (principal, action == ProductionPlanning::Action::"release_order", resource)
when { context.policy_bundle_version < "2026-05-21" || resource.lifecycle_state != "created" };
```

### D-6. Ontology projection
`ontology.production_planning.released_order` joins `production_order` ⨝ `order_reservation` ⨝ `plant_material`; library-first per ADR-0257; `freshness_floor=2s`.

### D-7. SLO
- P95 release latency ≤ 200ms, P99 ≤ 500ms
- Warehouse-ACK P95 ≤ 3s, P99 ≤ 8s
- Error budget 0.1% / 30d

### D-8. Telemetry
Metrics: `production_planning_order_release_latency_seconds`, `production_planning_order_release_total{outcome}`, `warehouse_staging_task_ack_lag_seconds{tenant_id}`.
Trace: `production-planning.order.release` → `warehouse.staging.create`.

## E. Failure modes
- **E-1 capacity_unconfirmed:** Block release, return 409 with `failure_reason=capacity_unconfirmed`; operator must run IP-020 finite-schedule.
- **E-2 material_short:** Emit `ORDER-RELEASE-BLOCKED-SHORT`; SCP expedite triggered.
- **E-3 quality_hold:** Block release; surface QM notification ID; runbook `runbooks/quality-hold-clearance.md`.
- **E-4 warehouse partition:** Outbox accumulates; backpressure on release endpoint at 5k pending rows.
- **E-5 FIFO mis-pick:** Warehouse honors strategy; mismatch surfaces as `staging_task_anomaly`; runbook covers root-cause walk.
- **E-6 release after order cancelled:** State-machine guard rejects with 409; audit `ORDER_RELEASE_REJECTED_INVALID_STATE`.

## F. Migration
Phase 1 contracts; Phase 2 Cedar soak; Phase 3 outbox+dispatcher; Phase 4 warehouse `staging_task_listener` consumer; Phase 5 feature flag flip. Rollback: feature flag → false.

## G. References

- ADR-0105 (13-layer), ADR-0130 (SLO promotion), ADR-0131 (flat-layout), ADR-0244 (tenant scoping), ADR-0252 (HLC), ADR-0253 (HTTP/3+QUIC), ADR-0257 (ontology read), ADR-0263 (observability), ADR-0294 (Cedar soak ≥60s), ADR-0314 (marketplace settlement boundary), ADR-0315 (ERP/SAP parity), ADR-0316 (tenant-class)
- SAP Help Portal entity references cited inline for mapping only

## H. Out-of-scope
Backflushing (IP-018 confirmation), kitting (IP-025 warehouse-side VAS), QM clearance (IP-018 QM domain).
— end IP-017 —
