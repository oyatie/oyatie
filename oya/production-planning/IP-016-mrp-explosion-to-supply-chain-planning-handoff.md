---
doc_class: ImplementationPlan
microservice: production-planning
status: Accepted
date: 2026-05-21
owner_team: axis-production-planning + axis-erp-parity
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0244, ADR-0252, ADR-0253, ADR-0257, ADR-0263, ADR-0294, ADR-0297, ADR-0314, ADR-0315, ADR-0316]
planned_enforcement_ref: oya-governance-production-planning-doc-set
ip_id: IP-016
journey_ref: j101
journey_slug: j101-multi-tier-supply-chain-formation
sap_submodule: PP-MRP (Material Requirements Planning)
peer_microservice: supply-chain-planning
companion_docs:
  - docs/user-journeys/j101-multi-tier-supply-chain-formation/README.md
  - microservices/production-planning/PRD.md
  - microservices/supply-chain-planning/PRD.md
inbound_citations:
  - docs/decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md
---

# IP-016: MRP Explosion → Supply-Chain-Planning Handoff (production-planning ↔ supply-chain-planning)

## A. Intent

When SAP S/4HANA `PP-MRP` runs `MD01` (single-item planning) or `MD03` (multi-level planning), it projects **dependent requirements** for every sub-component of the planned finished good and feeds these into SAP IBP-RP via the `MRP_OUT_DEPENDENT_REQUIREMENTS` BAPI. **This IP implements the equivalent in oyatie:** when `production-planning.mrp-run` completes for a finished-good `material_id`, dependent net-requirements are projected onto `supply-chain-planning` IBP-RP across multi-tier supplier networks per journey **j101-multi-tier-supply-chain-formation**.

### A.1 Concrete SAP equivalence (PP-MRP delta)

| SAP S/4HANA PP-MRP primitive | Oyatie equivalent | Concrete delta |
|---|---|---|
| `MARC` plant-material master | `production_planning.plant_material` row | Per-tenant `(tenant_id, plant_code, material_id)` PK; HLC `recorded_at`; soft-delete via `lifecycle_state` |
| `RESB` reservation table | `production_planning.mrp_reservation` event-sourced log | Append-only, idempotent on `(tenant_id, mrp_run_id, position_no)` |
| `T440P` planning data | per-cell `planning_strategy_pack` Cedar context | Strategy `10/11/20/40/52/82` mapped to oyatie strategy codes prefixed `OYA_PS_*` |
| `MD04` stock/requirements list | `GET /api/v1/material/{material_id}/stock-requirements` | Same projection but tenant-scoped + HLC-ordered |
| `MD61` planned-independent-requirements | `supply-chain-planning.demand_plan` aggregate | Owned by SCP not PP; PP only consumes |
| Lot-size key (`EX`, `FX`, `HB`, `WB`, `MB`, `PK`) | Lot-size strategy enum on `material_planning_attributes` | Mapped 1:1, plus oyatie-native `OYA_BIN_PACKING_V1` for AMR-aware lots |

### A.2 Journey leg

Per `docs/user-journeys/j101-multi-tier-supply-chain-formation/README.md`, the multi-tier supplier formation begins when a Tier-1 OEM publishes a forecast. PP-MRP's explosion is **the first cross-µservice step** in that journey: the OEM's BOM is exploded, dependent requirements emerge, and SCP receives them so Tier-2 / Tier-3 suppliers can be solicited via marketplace (settlement owned by `marketplace` per ADR-0314 — this slice never settles).

### A.3 Hyperscaler precedent

This handoff replicates the **AWS S3 → Lambda event-driven projection** pattern: a strongly-consistent write to one bounded context emits a typed event that the consumer projects eventually-consistently within a bounded latency budget. Adjacent precedent: **Stripe's `invoice.created` → `payment_intent.requires_action`** transition.

Benchmarks: SAP S/4HANA PP-MRP | Oracle Fusion Cloud Manufacturing MRP | Kinaxis RapidResponse multi-level pegging | Blue Yonder Luminate Demand-Supply | o9 Solutions IBP.

## B. Acceptance criteria

- **AC-1:** `POST /api/v1/mrp-runs` with `{material_id, plant_code, planning_horizon_days, lot_size_strategy}` executes the explosion in ≤ 30s P95 for a BOM of depth ≤ 8 and breadth ≤ 200 per level.
- **AC-2:** For every level-N dependent requirement produced, exactly one `MRP-EXPLODED` event is emitted on AsyncAPI channel `production-planning.mrp-exploded.v1` with the schema in §D-4.
- **AC-3:** `supply-chain-planning.ingest-mrp-exploded.v1` ACKs each event within 5s P95 with `correlation_id` preserved; ACK failures dead-letter to `dlq.production-planning.mrp-exploded` with operator runbook `runbooks/mrp-exploded-replay.md`.
- **AC-4:** Cedar gate `production_planning::mrp::explode` denies cross-tenant material reads (tested in `policy/mrp_explode_test.cedar`).
- **AC-5:** Audit class `EVT-PRODUCTION_PLANNING-MRP_EXPLOSION-IP_ACCEPTED` registered per ADR-0263, signed by µservice sidecar Ed25519 per ADR-0296.
- **AC-6:** Ontology projection `ontology.production_planning.mrp_dependent_requirement` reads in library-first mode per ADR-0257 amendment; `freshness_floor = "10s"`.
- **AC-7:** Marketplace settlement remains read-only per ADR-0314.
- **AC-8:** HTTP/3 + QUIC + ECH + PQC per ADR-0253 on the ingress.

## C. Verification

```bash
cargo test -p oya-production-planning-mrp -- explosion_happy_path
cargo test -p oya-production-planning-mrp -- explosion_circular_bom_detected
cargo test -p oya-production-planning-mrp -- explosion_zero_demand_no_event
cargo test -p oya-production-planning-policy -- mrp_explode_cross_tenant_denied
cargo test -p oya-production-planning-scp-integration -- handoff_correlation_preserved
cargo test -p oya-production-planning-scp-integration -- handoff_5s_p95_latency
cargo bench -p oya-production-planning-mrp -- explode_depth_8_breadth_200
```

Coverage floor: ≥ 90% line, ≥ 80% branch on `crates/oya-production-planning-mrp/`.

## D. Detailed mechanics

### D-1. API surface

**REST:**
```
POST /api/v1/mrp-runs
Content-Type: application/json
X-Oya-Tenant-Id: 01HXYZ...
X-Oya-Principal-Id: urn:oya:principal:tenant-acme:user-jdoe
Idempotency-Key: 01HXYZ...
Alt-Svc: h3=":443"; ma=2592000

{
  "material_id": "FG-12345",
  "plant_code": "PL-SEOUL-01",
  "planning_horizon_days": 90,
  "lot_size_strategy": "OYA_PS_10_LOT_FOR_LOT",
  "include_safety_stock": true,
  "scenario_id": null
}
```

**Response 202 Accepted:**
```json
{
  "mrp_run_id": "01HXYZRUN...",
  "status": "queued",
  "estimated_completion_at": "2026-05-21T03:14:15Z",
  "trace_id": "0af7651916cd43dd8448eb211c80319c"
}
```

**gRPC** (proto3, `production_planning.mrp.v1`):
```proto
service MrpService {
  rpc StartRun(StartRunRequest) returns (StartRunResponse);
  rpc GetRun(GetRunRequest) returns (MrpRun);
  rpc StreamExplosionEvents(StreamRequest) returns (stream MrpExplodedEvent);
}
```

### D-2. Workflow Studio step

`production-planning.mrp.explode` step exposes 4 decision branches:

| Branch | Trigger | Next step |
|---|---|---|
| `circular_bom_detected` | Cycle in BOM graph (Tarjan SCC > 1) | Halt → `production-planning.bom.repair-recommendation` |
| `capacity_overload` | Any work-center utilization > 120% in horizon | Fork → `production-planning.alt-routing.engage` (IP-022) |
| `make_or_buy_buy` | Component flagged buy-side | Fork → `procurement.purchase-requisition.create` (IP-021) |
| `happy_path` | Net-requirement ≥ 0 | Emit `MRP-EXPLODED` → `supply-chain-planning.ingest-mrp-exploded` |

### D-3. Data-model deltas

**New tables (PostgreSQL 16):**

```sql
CREATE TABLE production_planning.mrp_run (
  tenant_id            UUID NOT NULL,
  mrp_run_id           ULID PRIMARY KEY,
  material_id          TEXT NOT NULL,
  plant_code           TEXT NOT NULL,
  planning_horizon_days INT NOT NULL CHECK (planning_horizon_days BETWEEN 1 AND 730),
  lot_size_strategy    TEXT NOT NULL,
  status               TEXT NOT NULL CHECK (status IN ('queued','running','succeeded','failed','cancelled')),
  hlc_started_at       BYTEA NOT NULL,
  hlc_completed_at     BYTEA,
  principal_id         TEXT NOT NULL,
  policy_bundle_version TEXT NOT NULL,
  correlation_id       TEXT NOT NULL,
  cedar_decision_id    TEXT NOT NULL
) PARTITION BY HASH (tenant_id);

CREATE INDEX mrp_run_tenant_material_idx
  ON production_planning.mrp_run (tenant_id, material_id, hlc_started_at DESC);

CREATE TABLE production_planning.mrp_dependent_requirement (
  tenant_id        UUID NOT NULL,
  mrp_run_id       ULID NOT NULL,
  position_no      INT NOT NULL,
  level_no         INT NOT NULL CHECK (level_no BETWEEN 0 AND 32),
  component_id     TEXT NOT NULL,
  parent_component_id TEXT,
  quantity_required NUMERIC(20,6) NOT NULL,
  uom              TEXT NOT NULL,
  required_at      TIMESTAMPTZ NOT NULL,
  source           TEXT NOT NULL CHECK (source IN ('forecast','sales_order','reservation','safety_stock')),
  PRIMARY KEY (tenant_id, mrp_run_id, position_no)
) PARTITION BY HASH (tenant_id);

CREATE TABLE production_planning.mrp_exploded_outbox (
  tenant_id    UUID NOT NULL,
  event_id     ULID PRIMARY KEY,
  mrp_run_id   ULID NOT NULL,
  payload_jsonb JSONB NOT NULL,
  signature    BYTEA NOT NULL,
  dispatch_state TEXT NOT NULL DEFAULT 'pending',
  dispatched_at TIMESTAMPTZ,
  retries      INT NOT NULL DEFAULT 0,
  next_retry_at TIMESTAMPTZ
);
```

**Rollback:** `DROP TABLE` in reverse dependency order; outbox messages preserved in S3 cold archive for 90 days per ADR-0276 portability.

### D-4. AsyncAPI event shape

```yaml
asyncapi: 3.1.0
channels:
  production-planning.mrp-exploded.v1:
    address: production-planning.mrp-exploded.v1
    messages:
      mrpExploded:
        payload:
          type: object
          required: [event_id, tenant_id, mrp_run_id, material_id, dependent_requirements]
          properties:
            event_id: {type: string, format: ulid}
            tenant_id: {type: string, format: uuid}
            mrp_run_id: {type: string, format: ulid}
            material_id: {type: string, maxLength: 64}
            plant_code: {type: string, maxLength: 16}
            occurred_at: {type: string, format: date-time}
            hlc: {type: string, description: "HLC encoded base32"}
            correlation_id: {type: string}
            causation_id: {type: string}
            policy_bundle_version: {type: string}
            cedar_decision_id: {type: string}
            audit_event_class: {const: "EVT-PRODUCTION_PLANNING-MRP_EXPLOSION-IP_ACCEPTED"}
            dependent_requirements:
              type: array
              maxItems: 50000
              items:
                type: object
                required: [position_no, level_no, component_id, quantity, uom, required_at, source]
                properties:
                  position_no: {type: integer, minimum: 1}
                  level_no: {type: integer, minimum: 0, maximum: 32}
                  component_id: {type: string}
                  parent_component_id: {type: string}
                  quantity: {type: string, description: "decimal as string"}
                  uom: {type: string, enum: [EA, KG, L, M, M2, M3, BOX, PALLET]}
                  required_at: {type: string, format: date-time}
                  source: {type: string, enum: [forecast, sales_order, reservation, safety_stock]}
            home_cell: {type: string}
            dr_cell: {type: string}
            signature: {type: string, format: byte}
```

### D-5. Cedar policy fragment

```cedar
// policy/mrp-explode.cedar — soak ≥60s per ADR-0294
@id("production_planning::mrp::explode::v1")
@soak_started_at("2026-05-21T00:00:00Z")
permit (
  principal in ProductionPlanning::Operator::"role-mrp-planner",
  action == ProductionPlanning::Action::"mrp_explode",
  resource in ProductionPlanning::Material::?
) when {
  context.tenant_id == resource.tenant_id &&
  context.principal.tenant_id == resource.tenant_id &&
  context.audience_type == "enterprise_b2b" &&
  context.transport.http3_advertised == true &&
  context.transport.tls_min_version >= "1.3" &&
  resource.lifecycle_state == "active" &&
  resource.plant_code in context.principal.authorized_plants
};

forbid (
  principal,
  action == ProductionPlanning::Action::"mrp_explode",
  resource
) when {
  context.tenant_id != resource.tenant_id ||
  context.policy_bundle_version < "2026-05-21" ||
  context.principal.is_sanctioned == true
};
```

### D-6. Ontology projection (Palantir-equivalent)

```yaml
# ontology projection definition
projection: ontology.production_planning.mrp_dependent_requirement
mode: library_first  # per ADR-0257 amendment
freshness_floor: 10s
join:
  - source: production_planning.mrp_dependent_requirement
    on: (tenant_id, mrp_run_id)
  - source: production_planning.mrp_run
    on: (tenant_id, mrp_run_id)
  - source: production_planning.plant_material
    on: (tenant_id, plant_code, material_id)
exposed_fields:
  material_id: production_planning.mrp_dependent_requirement.component_id
  required_quantity: production_planning.mrp_dependent_requirement.quantity_required
  required_at: production_planning.mrp_dependent_requirement.required_at
  source_tier: production_planning.mrp_run.material_id  # tier-1 = root, tier-N = depth
  cedar_gate: production_planning::mrp::explode
```

### D-7. SAP S/4HANA → Oyatie ontology mapping

| SAP entity (table.field) | Oyatie ontology fact | Notes |
|---|---|---|
| `MARC.MATNR` | `material_id` | Tenant-scoped key |
| `MARC.WERKS` | `plant_code` | Renamed for clarity |
| `MARC.DISLS` (lot-size) | `lot_size_strategy` enum | EX→OYA_PS_10, FX→OYA_PS_11, etc. |
| `MARC.EISBE` (safety stock) | `safety_stock_quantity` | Decimal as string |
| `RESB.BDMNG` | `dependent_requirement.quantity_required` | |
| `RESB.BDTER` | `dependent_requirement.required_at` | TIMESTAMPTZ, not SAP's date+time split |
| `STKO.STLNR` (BOM header) | `bom_revision.bom_id` | |
| `STPO.IDNRK` (BOM item) | `bom_position.component_id` | |
| `MDKP.PLNUM` | `mrp_run.mrp_run_id` | ULID instead of SAP number-range |

### D-8. SLO

```yaml
apiVersion: openslo/v1
kind: SLO
metadata:
  name: production-planning.mrp-explosion
spec:
  service: production-planning
  description: MRP explosion completes and emits MRP-EXPLODED within budget.
  budgetingMethod: Occurrences
  objectives:
    - displayName: explosion-latency-p95-30s
      target: 0.99
      sli:
        ratioMetric:
          counter: true
          good:
            metricSource: prometheus
            spec:
              query: 'sum(rate(production_planning_mrp_explode_duration_seconds_bucket{le="30"}[5m]))'
          total:
            metricSource: prometheus
            spec:
              query: 'sum(rate(production_planning_mrp_explode_duration_seconds_count[5m]))'
    - displayName: handoff-ack-p95-5s
      target: 0.99
```

### D-9. Telemetry

Metrics:
- `production_planning_mrp_explode_duration_seconds` histogram (le: 1, 5, 10, 30, 60, 120)
- `production_planning_mrp_dependent_requirements_emitted_total{level_no,source}` counter
- `production_planning_mrp_outbox_lag_seconds{tenant_id}` gauge
- `production_planning_mrp_cedar_decisions_total{decision}` counter

Trace span shape: `production-planning.mrp.explode` with attributes `tenant_id`, `mrp_run_id`, `material_id`, `bom_depth`, `bom_breadth`, `cedar_decision_id`, `home_cell`.

## E. Failure modes & recovery

### E-1. Circular BOM detected (Tarjan SCC > 1)
**Detection:** During DFS traversal, Tarjan's SCC finds a strongly-connected component of size > 1.
**Behaviour:** Halt explosion, emit `MRP-EXPLOSION-FAILED` with `failure_reason=circular_bom`, attach the offending cycle path. No partial emission to SCP.
**Recovery:** Runbook `runbooks/circular-bom-repair.md` walks the planner through identifying the offending BOM revision and creating an ECN.

### E-2. Network partition to SCP (outbox lag > 30s for 2+ min)
**Detection:** `production_planning_mrp_outbox_lag_seconds` exceeds 30 for >120s.
**Behaviour:** Outbox accumulates; at-least-once dispatch with idempotency-key absorbs duplicates on SCP side; max 10k rows in outbox before backpressure on `POST /api/v1/mrp-runs` (returns 429).
**Recovery:** Once partition heals, dispatcher drains in HLC order; replay-window 5min, dead-letter to `dlq.production-planning.mrp-exploded` for messages >24h with operator review.

### E-3. SCP rejects payload (schema violation)
**Detection:** SCP's `ingest-mrp-exploded` returns 422 with `validation_errors`.
**Behaviour:** Outbox row marked `dispatch_state=poison`; emit `MRP-EXPLODED-POISON` audit event with the schema error; do not retry.
**Recovery:** Operator inspects the poison-pill row, fixes the schema (most likely a version mismatch — both µservices must roll forward), re-runs MRP.

### E-4. Worker crash mid-explosion (level-12 of a level-20 BOM)
**Detection:** Worker heartbeat lost > 30s.
**Behaviour:** Another worker claims the run via Postgres `SELECT ... FOR UPDATE SKIP LOCKED`; explosion resumes from the last committed level (level-12) using the `mrp_dependent_requirement` rows already persisted.
**Recovery:** Idempotency on `(tenant_id, mrp_run_id, position_no)` ensures no duplicate inserts; the worker resumes the DFS at the boundary.

### E-5. Cedar fragment misdeploy (false-positive deny spike)
**Detection:** `production_planning_mrp_cedar_decisions_total{decision="deny"}` rate exceeds 10× P50 over 60s.
**Behaviour:** Per ADR-0294 60s soak, the fragment deploy halts before global rollout.
**Recovery:** Emergency rollback via `runbooks/cedar-fragment-emergency-rollback.md`.

### E-6. Plant code reassigned mid-run
**Detection:** `plant_material.lifecycle_state` flips to `retired` after the run started.
**Behaviour:** Run completes with the snapshot taken at start (HLC `hlc_started_at`); audit event annotated `plant_lifecycle_snapshot=true`.
**Recovery:** Operator decides whether to re-run with the new plant configuration.

## F. Migration

Phase 1: contracts + Cedar fragment soak (≥60s); Phase 2: outbox + dispatcher behind feature flag `production_planning_mrp_explode_enabled`; Phase 3: SCP's `ingest-mrp-exploded` consumer; Phase 4: flag flip to true on canary cell, then global.

Rollback: feature flag → false; outbox messages drained but not dispatched; SCP-side consumer idempotency absorbs any in-flight duplicates.

## G. References

- ADR-0105, ADR-0130, ADR-0131, ADR-0244, ADR-0252, ADR-0253, ADR-0257, ADR-0263, ADR-0294, ADR-0314, ADR-0315, ADR-0316
- `docs/user-journeys/j101-multi-tier-supply-chain-formation/README.md`
- SAP Help Portal: MRP (PP-MRP) — `https://help.sap.com/docs/SAP_S4HANA_ON-PREMISE/...` (cited for entity mapping only, not for code)
- Tarjan's SCC algorithm (Tarjan, 1972) for circular-BOM detection.

## H. Out-of-scope

- BOM repair UX (handled by `production-planning.bom-revision` IP-019).
- Marketplace supplier solicitation (owned by `marketplace` per ADR-0314).
- Tax / landed-cost computation (owned by `treasury` + `global-trade` IP-030).
- Self-modification (owned by `foundry` per ADR-0247).

— end IP-016 —
