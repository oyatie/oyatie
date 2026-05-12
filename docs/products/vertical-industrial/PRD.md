# Oyatie — Product PRD: Vertical Industrial

> **Status:** preview
> **Owning team:** [`teams/vertical-industrial/CHARTER.md`](../../teams/vertical-industrial/CHARTER.md)
> **Owning axis:** vertical-industrial (Axis 2)
> **Catalog reference:** `registry/catalog/oya-vertical-industrial-*.yaml`
> **Last updated:** 2026-05-09 by architecture-council

---

## 1. North Star

Oyatie Vertical Industrial is a Manufacturing Execution System (MES) and plant-operations platform aligned to ISA-95 (enterprise/control integration) and ISA-88 (batch control), with native OPC UA connectivity and SCADA historian bridging. It owns the canonical entity model for WorkOrder, ProductionRun, EquipmentUnit, ProcessValue, and QualityResult — enabling Overall Equipment Effectiveness (OEE) tracking, process-parameter capture, and quality control in discrete and process manufacturing. The product exists within Oyatie's ecosystem because the integration of a plant-floor execution layer with enterprise HR (Corporate vertical), general-ledger cost capture, Foundry-driven predictive maintenance agents, and the audit chain for regulatory compliance (FDA 21 CFR Part 11 for pharma manufacturing; MFDS for KR; EU MDR for medical devices) is the value proposition that standalone MES vendors cannot replicate. The compliance-as-code posture — ISA-95 hierarchy in the kernel, regulatory evidence emitted per operation, autonomy ceiling enforcing agent-driven process adjustments — is the industrial moat.

---

## 2. Target Users

| Persona | What they get | What they pay for |
|---|---|---|
| Plant Manager | OEE dashboard, shift summary, production vs. plan, Foundry-authored daily report | Per-site subscription |
| Production Supervisor | Work order dispatch, production run tracking, machine status, downtime capture | Per-seat (ops tier) |
| Process Engineer | Process parameter templates, recipe management, SPC (Statistical Process Control) charts, deviation alerts | Per-seat (engineering tier) |
| Quality Inspector | Incoming/in-process/final inspection results, non-conformance (NCR) creation, CAPA tracking | Per-seat (quality tier) |
| Maintenance Technician | Equipment PM schedule, work order execution, OPC UA live readings, Foundry predictive-maintenance alerts | Per-seat (maintenance tier) |
| Manufacturing IT / Tenant Builder | OPC UA namespace config, SCADA historian bridge config, ISA-95 hierarchy authoring, Foundry capability workflow authoring | Builder seat |
| Regulator / Auditor (FDA, MFDS, ISO 9001 CB) | Batch records (21 CFR Part 11), electronic signatures, audit trail, equipment calibration evidence | Cost of doing business |

---

## 3. In-Scope / Out-of-Scope

### 3.1 In-scope at each wave

| Wave | Capabilities | Surfaces exposed |
|---|---|---|
| Vertical-Preview | ISA-95 L3 hierarchy (Enterprise → Site → Area → WorkCenter → WorkUnit), work order creation and dispatch, production run tracking, OEE (Availability / Performance / Quality), downtime event capture, OPC UA data ingestion (read-only), basic quality inspection | REST API v1, Web UI (plant dashboard), OPC UA client |
| Vertical-Stable | Recipe / process-parameter management (ISA-88 batch), SPC control charts (Xbar-R, CUSUM), non-conformance (NCR) and CAPA workflow, SCADA historian bridge (OSIsoft PI / Ignition SCADA), Foundry predictive-maintenance agent, electronic batch records (EBR) with 21 CFR Part 11 electronic signatures, equipment calibration management, label printing integration | REST API stable, OPC UA read/write, SCADA bridge, Webhook console |
| Public-GA | Multi-site OEE roll-up, energy consumption tracking (ISO 50001), carbon-footprint per production run, supplier quality portal (incoming inspection), Foundry autonomous scheduling optimization (recommend-only, planner approves), MES ↔ ERP integration (standard GL cost posting via Corporate vertical) | Public OpenAPI, Analytics dashboards, ERP integration hooks |
| Region-Fan-Out | Per-regional-pack regulatory evidence (FDA, MFDS, EMA/MDR, ISO 9001 CB) | Per-pack launch cadence |

### 3.2 Out-of-scope (anti-scope)

- ERP / MRP / S&OP at planning depth (work orders are received from ERP; Oyatie MES executes them; the MRP engine is not built)
- Real-time hard-PLC control (safety-rated PLC control logic, IEC 61131 programming) — OPC UA connectivity is read/write for process setpoints, not safety control
- Asset-heavy capital project management (that is the Construction vertical)
- Consumer product traceability portal (the Agriculture and Food verticals cover farm-to-table; this vertical covers the factory floor)
- Advertising targeting using production data — always blocked (`BEHAVIORAL_TENANT_PRODUCT` is not ad-targetable per PRIVACY-PROGRAM §2.2.3 industrial default)

---

## 4. Architecture Overview

### 4.1 Bounded Context

Axis 2 — Vertical Industrial. Flat-crates target prefix: `crates/oya-vertical-industrial-*`.

The industrial vertical owns the ISA-95 hierarchy, work-order execution, OEE aggregation, and process-value time-series. Cross-axis contracts: `oya-platform-tenant-kernel`, `oya-platform-audit-chain-kernel` (batch record + EBR signatures), `oya-foundry-api` (predictive maintenance + scheduling agents), `oya-vertical-corporate-domain-gl` (cost-posting seam), `oya-platform-regulatory-kernel` (FDA/MFDS/ISO packs).

### 4.2 Layered Structure

```
crates/oya-vertical-industrial-kernel-isa95/       — Enterprise, Site, Area, WorkCenter, WorkUnit, EquipmentUnit entities (ISA-95 L3)
crates/oya-vertical-industrial-kernel-execution/   — WorkOrder, ProductionRun, OperationRecord entities
crates/oya-vertical-industrial-kernel-process/     — ProcessValue, Recipe, ProcessParameter, BatchRecord entities (ISA-88)
crates/oya-vertical-industrial-kernel-quality/     — InspectionPlan, InspectionResult, NonConformance, CAPA entities
crates/oya-vertical-industrial-kernel-oee/         — OeeSnapshot, DowntimeEvent, OeeCalculation value objects
crates/oya-vertical-industrial-domain-execution/   — Work-order dispatch, production-run lifecycle, shift-report use cases
crates/oya-vertical-industrial-domain-quality/     — Inspection, NCR, CAPA use cases
crates/oya-vertical-industrial-domain-oee/         — OEE computation, downtime classification use cases
crates/oya-vertical-industrial-app-execution/      — Production saga, Foundry capability delegation (predictive maintenance, scheduling)
crates/oya-vertical-industrial-app-ebr/            — Electronic Batch Record saga (21 CFR Part 11 e-signatures)
crates/oya-vertical-industrial-adapter-db/         — Postgres + TimescaleDB adapters
crates/oya-vertical-industrial-adapter-opcua/      — OPC UA client adapter (open62541-rs wrapper)
crates/oya-vertical-industrial-adapter-scada/      — SCADA historian bridge (OSIsoft PI REST / Ignition MQTT)
crates/oya-vertical-industrial-api-rest/           — REST API handlers
crates/oya-vertical-industrial-worker-opcua/       — OPC UA subscription worker (processes incoming process values)
crates/oya-vertical-industrial-worker-events/      — Kafka consumers (work-order-requested from ERP integration, etc.)
crates/oya-vertical-industrial-runtime/            — Composition root binary
```

### 4.3 External-Facing Surfaces

| Surface | Contract location | Plane | SLO target |
|---|---|---|---|
| Industrial REST API | `contracts/industrial-mes.openapi.yaml` | Data | 99.9% / p95 < 200ms |
| OPC UA server (namespace exposure for SCADA) | `contracts/industrial-opcua.yaml` | Data | 99.9% / p99 < 100ms (real-time process data) |
| SCADA bridge (read historian) | `contracts/industrial-scada.yaml` | Data | 99.5% / p95 < 500ms |
| Webhook events (production-run-completed, ncr-opened) | `contracts/industrial-webhooks.yaml` | Data | at-least-once, ≤ 30s |
| Analytics (OEE dashboard, SPC charts) | internal projection API | Analytics | best-effort |

### 4.4 Internal Seams

| Seam | Trait / interface | Consumer products |
|---|---|---|
| `ProductionCostPostable` | `GlCostPostable` trait | Corporate vertical GL (cost of goods produced) |
| `ProcessValueIndexable` | `SearchIndexable` (tenant-private) | Search axis (process engineer parameter lookup) |
| `EquipmentAuditEmitter` | `AuditChainEmitter` | Audit chain (EBR, 21 CFR Part 11) |

### 4.5 Dependencies on Other Axes

| Contract consumed | Owner axis | Where it lives | Change-review class |
|---|---|---|---|
| `Tenant` kernel | SaaS platform | `oya-platform-tenant-kernel` | Cross-axis review |
| `Capability invocation` (predictive maintenance, scheduling) | Foundry | `oya-foundry-api` | Foundry + industrial review |
| `Audit-chain event` (EBR mandatory) | Platform | `oya-platform-audit-chain-kernel` | Audit review |
| `GlCostPostable` seam | Corporate vertical | `oya-vertical-corporate-domain-gl` | Cross-vertical review |
| `RegulatoryPack` seam | Platform regulatory | `oya-platform-regulatory-kernel` | Regulatory + industrial review |

---

## 5. Data Structures

### 5.1 Kernel Entities

```rust
// crates/oya-vertical-industrial-kernel-isa95

/// ISA-95 Equipment hierarchy — L3 MES level
/// data_class: BEHAVIORAL_TENANT_PRODUCT
/// plane: control
pub struct EquipmentUnit {
    pub id: EquipmentUnitId,
    pub tenant_id: TenantId,
    pub site_id: SiteId,
    pub area_id: AreaId,
    pub work_center_id: WorkCenterId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub name: String,                             // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub code: String,                             // data_class: INTERNAL_ONLY
    pub equipment_class: EquipmentClass,          // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub manufacturer: Option<String>,             // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub model_number: Option<String>,             // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub serial_number: Option<String>,            // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub opcua_node_id: Option<OpcUaNodeId>,       // data_class: INTERNAL_ONLY
    pub status: EquipmentStatus,                  // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub capacity: Option<Decimal>,                // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub capacity_uom: Option<UnitOfMeasure>,      // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum EquipmentStatus { Available, Running, Scheduled, Down, Maintenance, Decommissioned }
pub enum EquipmentClass { Machine, Robot, Conveyor, Oven, Reactor, Vessel, TestStation, Other }
```

```rust
// crates/oya-vertical-industrial-kernel-execution

/// ISA-95 Work Order (L3 production request)
/// data_class: BEHAVIORAL_TENANT_PRODUCT
/// plane: data
pub struct WorkOrder {
    pub id: WorkOrderId,
    pub tenant_id: TenantId,
    pub site_id: SiteId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub erp_order_ref: Option<ExternalRef>,       // data_class: BEHAVIORAL_TENANT_PRODUCT (ERP integration)
    pub product_id: ProductId,                    // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub recipe_id: Option<RecipeId>,              // data_class: BEHAVIORAL_TENANT_PRODUCT (ISA-88 recipe)
    pub planned_quantity: Decimal,                // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub planned_uom: UnitOfMeasure,
    pub planned_start: DateTime<Utc>,             // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub planned_end: DateTime<Utc>,               // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub priority: WorkOrderPriority,              // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub status: WorkOrderStatus,                  // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub work_center_id: WorkCenterId,             // data_class: INTERNAL_ONLY
    pub foundry_run_id: Option<FoundryRunId>,     // data_class: INTERNAL_ONLY (scheduling agent)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum WorkOrderStatus {
    Planned, Released, InProgress, Paused, Completed, Cancelled
}
pub enum WorkOrderPriority { Low, Normal, High, Urgent }

/// ISA-95 Production Run (actual execution record)
/// data_class: BEHAVIORAL_TENANT_PRODUCT
/// plane: data
pub struct ProductionRun {
    pub id: ProductionRunId,
    pub tenant_id: TenantId,
    pub work_order_id: WorkOrderId,               // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub equipment_unit_id: EquipmentUnitId,       // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub region: RegionCode,
    pub schema_version: u32,
    pub batch_number: Option<BatchNumber>,        // data_class: BEHAVIORAL_TENANT_PRODUCT (ISA-88 batch id)
    pub lot_number: Option<LotNumber>,            // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub actual_start: Option<DateTime<Utc>>,
    pub actual_end: Option<DateTime<Utc>>,
    pub actual_quantity: Option<Decimal>,         // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub good_quantity: Option<Decimal>,           // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub scrap_quantity: Option<Decimal>,          // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub status: ProductionRunStatus,              // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub operator_ids: Vec<UserId>,                // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub ebr_ref: Option<EbrRef>,                  // data_class: INTERNAL_ONLY (electronic batch record)
    pub oee_snapshot: Option<OeeSnapshot>,        // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum ProductionRunStatus { Active, Paused, Completed, Aborted }
```

```rust
// crates/oya-vertical-industrial-kernel-process

/// ISA-88 Process Parameter / Recipe value
/// data_class: BEHAVIORAL_TENANT_PRODUCT
/// plane: data
pub struct ProcessValue {
    pub id: ProcessValueId,
    pub tenant_id: TenantId,
    pub equipment_unit_id: EquipmentUnitId,
    pub production_run_id: Option<ProductionRunId>,
    pub region: RegionCode,
    pub schema_version: u32,
    pub parameter_name: String,                   // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub opcua_node_id: Option<OpcUaNodeId>,       // data_class: INTERNAL_ONLY
    pub value: ProcessValueReading,               // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub uom: Option<UnitOfMeasure>,
    pub quality: OpcUaQuality,                    // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub timestamp: DateTime<Utc>,                 // OPC UA server timestamp
    pub ingested_at: DateTime<Utc>,
}

pub enum ProcessValueReading {
    Float(f64),
    Int(i64),
    Bool(bool),
    String(String),
}

pub enum OpcUaQuality { Good, Uncertain, Bad }
```

```rust
// crates/oya-vertical-industrial-kernel-quality

/// Quality Inspection Result
/// data_class: BEHAVIORAL_TENANT_PRODUCT
/// plane: data
pub struct InspectionResult {
    pub id: InspectionResultId,
    pub tenant_id: TenantId,
    pub production_run_id: ProductionRunId,
    pub inspection_plan_id: InspectionPlanId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub characteristic: String,                  // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub measured_value: Option<Decimal>,          // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub measured_uom: Option<UnitOfMeasure>,
    pub attribute_result: Option<AttributeResult>,// data_class: BEHAVIORAL_TENANT_PRODUCT
    pub usl: Option<Decimal>,                    // upper spec limit
    pub lsl: Option<Decimal>,                    // lower spec limit
    pub conformant: bool,                        // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub inspector_id: UserId,                    // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub inspected_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum AttributeResult { Pass, Fail, Marginal }

/// OEE Snapshot (computed value object)
/// data_class: BEHAVIORAL_TENANT_PRODUCT
pub struct OeeSnapshot {
    pub availability: f64,    // (0.0 - 1.0) actual_run / planned_run
    pub performance: f64,     // (0.0 - 1.0) actual_output / ideal_output
    pub quality: f64,         // (0.0 - 1.0) good_qty / total_qty
    pub oee: f64,             // availability × performance × quality
    pub computed_at: DateTime<Utc>,
}
```

### 5.2 Aggregate Boundaries

| Aggregate | Root entity | Consistency boundary |
|---|---|---|
| `EquipmentAggregate` | `EquipmentUnit` | Equipment metadata + calibration records; ISA-95 hierarchy linkage via references |
| `WorkOrderAggregate` | `WorkOrder` | Work order status lifecycle; production runs are separate aggregates with WO-ref |
| `ProductionRunAggregate` | `ProductionRun` + `ProcessValue` (inline references) | One production run + its OEE snapshot; downtime events attached to the run |
| `QualityInspectionAggregate` | `InspectionResult[]` for one production run | All inspection results for a run; NCR is a separate aggregate created from QI |
| `EbrAggregate` | `ElectronicBatchRecord` + `ESignature[]` | Immutable after final signature; 21 CFR Part 11 compliance |

### 5.3 Persistence Layout

| Aggregate | Store | Sharding key | Partition strategy | Replication | Retention |
|---|---|---|---|---|---|
| EquipmentUnit / WorkOrder | Postgres (per-site shard) | `(tenant_id, site_id)` | Per-site schema | Streaming replication × 2 | Indefinite (active) + 10 years (decommissioned) |
| ProductionRun | Postgres (per-site shard) | `(tenant_id, site_id)` | Per-site schema | Streaming replication × 2 | 10 years (FDA 21 CFR Part 11 batch record) |
| ProcessValue | TimescaleDB (per-site hypertable) | `(tenant_id, equipment_unit_id, timestamp)` | Time-series hypertable, 1-hour chunk | Streaming replication × 2 | 2 years hot; archive to object store |
| InspectionResult | Postgres (per-site shard) | `(tenant_id, site_id)` | Per-site schema | Streaming replication × 2 | 10 years (ISO 9001 / FDA) |
| EBR (Electronic Batch Record) | Postgres (immutable append-only) | `(tenant_id, batch_number)` | Append-only partitioned table | Streaming replication × 3 | Per-regulatory: FDA 21 CFR Part 211.68 = batch record life + 1 year or 3 years (whichever longer) |

### 5.4 Event Schemas

| Event name | Topic | Schema location | Consumer aggregates | Retention | Idempotency key |
|---|---|---|---|---|---|
| `WorkOrderReleased` | `industrial.workorder.released` | `contracts/events/industrial-execution.json` | ProductionRun, Scheduling agent (Foundry), Audit chain | 30 days | `work_order_id` |
| `ProductionRunCompleted` | `industrial.production.run.completed` | `contracts/events/industrial-execution.json` | OEE aggregate, Quality inspection trigger, GL cost posting (Corporate), Audit chain | 30 days | `production_run_id` |
| `ProcessValueAlert` | `industrial.process.alert` | `contracts/events/industrial-process.json` | Predictive maintenance (Foundry), Operator notification | 7 days | `(equipment_unit_id, timestamp)` |
| `NonConformanceOpened` | `industrial.quality.ncr.opened` | `contracts/events/industrial-quality.json` | CAPA workflow, Audit chain | 90 days | `ncr_id` |
| `EbrSigned` | `industrial.ebr.signed` | `contracts/events/industrial-ebr.json` | Audit chain (mandatory), Regulatory evidence pack | 365 days | `(ebr_id, signer_id)` |
| `EquipmentDown` | `industrial.equipment.down` | `contracts/events/industrial-maintenance.json` | Predictive maintenance (Foundry), Maintenance scheduler, OEE update | 30 days | `(equipment_unit_id, timestamp)` |

### 5.5 Index / Search-Index Touchpoints

| Entity field | Index | Class allowed | Cascade-on-DSR? |
|---|---|---|---|
| `WorkOrder.product_id` + description | tenant-private production search | `BEHAVIORAL_TENANT_PRODUCT` | No (production records have regulatory retention) |
| `EquipmentUnit.name` | tenant-private equipment directory | `BEHAVIORAL_TENANT_PRODUCT` | No |
| `NonConformance.description` | tenant-private quality search | `BEHAVIORAL_TENANT_PRODUCT` | No |

### 5.6 Audit-Chain Emission Contract

| Operation | Emits topic | Required fields |
|---|---|---|
| Electronic batch record signed | `audit.industrial.ebr.signed` | `ebr_id`, `batch_number`, `signer_id`, `signature_reason`, `timestamp` (21 CFR Part 11 §11.50) |
| Work order status change | `audit.industrial.workorder.status` | `work_order_id`, `old_status`, `new_status`, `changed_by`, `timestamp` |
| Process parameter deviation | `audit.industrial.process.deviation` | `production_run_id`, `parameter_name`, `measured_value`, `usl`, `lsl`, `deviation_magnitude` |
| Non-conformance created | `audit.industrial.ncr.created` | `ncr_id`, `production_run_id`, `characteristic`, `created_by` |
| Recipe change (pre-production) | `audit.industrial.recipe.changed` | `recipe_id`, `version`, `changed_by`, `change_summary` |

### 5.7 Schema Migration Policy

- Production and EBR schema changes require staging validation + non-zero-downtime migration strategy.
- EBR tables are append-only; no destructive migrations on EBR schema in production.
- TimescaleDB hypertable chunk parameters (chunk interval, compression) can be tuned without migration; declared as configuration, not schema.

---

## 6. Optimization Practices

| Practice | Implementation choice |
|---|---|
| Cell routing | `(tenant_id, site_id)` → cell; one factory site is one cell unit for blast-radius control |
| Sharding strategy | Per-site Postgres shard for transactional data; TimescaleDB hypertable per site for process values |
| Caching tier | In-memory LRU for recipe / process-parameter templates (low-churn); Redis for live OEE state (updated every shift); no caching of EBR records |
| Bulk endpoint contract | `POST /process-values/bulk` (OPC UA subscription batch ingest, up to 10,000 values/call); `POST /inspection-results/bulk` (QC batch upload) |
| Pagination | Cursor-based on `(timestamp, id)` for process values; page size max 1,000 for process-value queries; OEE aggregates paginated by shift |
| Idempotency | `Idempotency-Key` on work-order creation and EBR signing; OPC UA subscription values deduplicated on `(node_id, timestamp)` |
| Batch dispatch | Foundry `PredictiveMaintenance` capability runs per-equipment-unit batch every hour; Foundry `ProductionScheduleOptimizer` runs per-site batch per shift |
| Backpressure | OPC UA subscription worker applies Kafka consumer-group backpressure; process-value ingest rate-limited to 10K/s per site with admission control |
| Hot-path benchmarks | `opcua_value_ingest` criterion gate: < 5ms per value; `oee_compute_shift` < 500ms; `work_order_dispatch` < 100ms |
| Agent-driven optimization | Foundry `PredictiveMaintenance` (anomaly detection on process values → pre-emptive maintenance work order); Foundry `ProductionScheduleOptimizer` (recommend-only, planner approves) |
| FinOps unit-economics | Per-equipment-unit-month metering; per-process-value-ingest metering (TimescaleDB storage cost); Foundry capability invocations metered separately |
| Build-cache / CI affected-graph | `oya-vertical-industrial-kernel-isa95` → full rebuild; `adapter-opcua` → targeted rebuild + OPC UA conformance test |

---

## 7. Regional Pack Interactions

| Seam | Trait | Per-pack impl needed? | Tested with which packs? |
|---|---|---|---|
| Regulatory control evidence (21 CFR Part 11 / MFDS / ISO) | `RegulatoryPack` | Yes | `oya-pack-kr` (MFDS 의료기기법 GMP, 식품안전관리인증원 HACCP-lite), `oya-pack-us` (FDA 21 CFR Part 11, 21 CFR Part 211, ISO 9001 CB), `oya-pack-eu` (EU MDR 2017/745, ISO 13485, CE marking evidence) |
| Local engineering standards | `LocalIndustryExtension` (industrial kernel) | Yes | `oya-pack-kr` (KS standards, KOSHA safety), `oya-pack-us` (ANSI/ISA standards, OSHA), `oya-pack-eu` (EN standards, CE marking, ATEX) |
| OPC UA server configuration | `OpcUaServerConfig` (regional endpoint format) | No — OPC UA is a global standard | All packs (OPC UA Foundation standard) |
| Calendar / shift schedule | `LocaleFormatter` | Yes (national holidays affect planned_run calculation for OEE) | All onboarded packs |

### Regulatory Pack Declaration

```yaml
# registry/catalog/oya-vertical-industrial-runtime.yaml
regulatory_packs:
  - oya-pack-kr   # MFDS, KOSSHA, KS standards
  - oya-pack-us   # FDA 21 CFR Part 11/211, ISO 9001, OSHA
  - oya-pack-eu   # EU MDR, ISO 13485, ATEX, EN standards
```

---

## 8. In-House vs External Dependency Posture

| External dep | Maturity tier | License | In-house alternative considered? | Decision |
|---|---|---|---|---|
| `tokio`, `axum`, `sqlx`, `serde`, `rustls` | kernel-grade | MIT / Apache-2 | No | Use |
| `open62541-rs` (OPC UA client, Rust binding to open62541) | maturing | MPL-2 (open62541) | In-house OPC UA client is a 6-month+ effort; open62541 is IEC 62541 certified | Use; ADR required (MPL-2 confirmed OK) |
| TimescaleDB (Postgres time-series extension) | stable | Apache-2 OSS | Pure Postgres partitioning insufficient for 10K+ values/s per site | Use TimescaleDB OSS |
| `rumqttc` (MQTT client for SCADA/Ignition bridge) | stable | Apache-2 | In-house MQTT client considered — rumqttc covers v3.1 and v5 | Use |
| `statrs` (statistical distributions for SPC) | stable | MIT | In-house SPC computation considered — statrs covers Xbar-R, CUSUM | Use |
| `printpdf` (label/EBR PDF generation) | stable | MIT | In-house PDF considered — printpdf suitable for EBR records | Use |

---

## 9. Success Metrics

| Metric | Vertical-Preview target | Vertical-Stable target | Public-GA target |
|---|---|---|---|
| Active equipment units under management | ≥ 50 (design-partner site) | ≥ 500 | ≥ 10,000 |
| OEE computation latency (per shift close) | < 5s | < 1s | < 500ms |
| Process value ingest rate | ≥ 1,000 values/s per site | ≥ 5,000 values/s | ≥ 10,000 values/s |
| EBR signature audit-chain completeness | 100% | 100% | 100% |
| Foundry PredictiveMaintenance alert precision | ≥ 70% | ≥ 85% | ≥ 90% |
| Work order dispatch P99 | < 500ms | < 200ms | < 100ms |
| NCR → CAPA closure cycle time (median) | < 30 days | < 15 days | < 10 days |
| Planned vs actual production variance (Foundry scheduling) | baseline (measure only) | < 10% variance | < 5% variance |
| Cross-axis contract violations | 0 | 0 | 0 |

---

## 10. Risks + Mitigations

| Risk | Severity | Mitigation | Owner |
|---|---|---|---|
| OPC UA write command causing unsafe process condition | Catastrophic | OPC UA write is disabled by default; requires explicit tenant opt-in per node-id; Cedar policy gates write capability at autonomy ceiling T2; Safety-instrumented systems (SIS) are out-of-scope and explicitly excluded from OPC UA namespace | Industrial domain + Security |
| 21 CFR Part 11 EBR non-compliance (missing audit trail) | Critical | EBR aggregate is append-only; every e-signature emits mandatory audit-chain record; FDA mock audit run quarterly | Compliance + Industrial domain |
| TimescaleDB process-value retention cost explosion | High | Per-site storage FinOps alert; automatic compression after 24 hours; archive to object store after 90 days; tenant-level retention policy | FinOps + Infrastructure |
| OPC UA subscription data storm (misconfigured sampling rate) | High | Per-site ingestion rate limit (10K values/s); adaptive sampling rate negotiation with OPC UA server; circuit breaker on ingest worker | Industrial domain + SRE |
| Recipe deviation in regulated pharma production (FDA GMP violation) | Catastrophic | ProcessValueAlert triggers mandatory deviation log; EBR flags deviation; Foundry cannot auto-correct recipe parameters without T3 autonomy (not granted for GMP) | Industrial domain + Compliance |
| SCADA historian bridge failure (missed downtime event) | High | SCADA bridge uses dead-reckoning fallback: if OPC UA heartbeat missed > 5 min, auto-create DowntimeEvent; manual reconciliation UI | Industrial domain + SRE |

---

## 11. Open Questions

- OPC UA write permission model: per-node-id Cedar policy or per-equipment-class Cedar policy? Trade-off between granularity and policy maintenance burden.
- ISA-88 batch recipe management — in-scope for Vertical-Preview or deferred to Stable? (Currently scoped to Stable.)
- Foundry scheduling optimizer autonomy level: T2 (recommend + execute with approval) or T1 (recommend only)? Industrial safety argues for T1 at Preview.
- Carbon footprint per production run — ISO 14064 scope? Integration with EU Carbon Border Adjustment Mechanism (CBAM) reporting?
- Multi-site OEE roll-up — does cross-site aggregation require tenant consent or is it within-tenant-ok?

---

## 12. Decision Log

| Decision | Date | Rationale | ADR ref |
|---|---|---|---|
| ISA-95 L3 as the canonical hierarchy model | 2026-05-09 | Industry standard; interoperates with ERP (ISA-95 L4) and SCADA (ISA-95 L1/L2) | — |
| OPC UA as primary connectivity protocol (not MQTT only) | 2026-05-09 | OPC UA is IEC 62541 standard; MQTT is supplemental for SCADA bridge | — |
| TimescaleDB for process values (not pure Postgres) | 2026-05-09 | 10K+ values/s per site requires time-series optimization; TimescaleDB OSS Apache-2 | — |
| Foundry agents at T1 autonomy for Preview (recommend only) | 2026-05-09 | Industrial safety constraint; operator must approve process adjustments | ADR-0050 |
| EBR tables append-only | 2026-05-09 | FDA 21 CFR Part 11 §11.10(e) requires accurate and complete records not subject to alteration | — |
| Flat-crates: `crates/oya-vertical-industrial-*` | 2026-05-09 | Per ADR-0015 | ADR-0015 |

---

## 13. Sources Scanned

- `docs/PRD.md`, `docs/DESIGN.md` §1, §4, §10, §12
- `docs/PRIVACY-PROGRAM.md` §2.2.1, §2.2.3
- ISA-95 Part 1-6 (enterprise/control integration); ISA-88 (batch control); OPC UA specification (IEC 62541)
- FDA 21 CFR Part 11 (electronic records); FDA 21 CFR Part 211 (pharmaceutical GMP)

---

## Doc-Catalog Row

```
| `vertical-industrial` | `vertical-2` | MES/OEE/ISA-95/OPC UA/SCADA; 21 CFR Part 11 EBR | monthly | PRD.md, DESIGN.md §12, PRIVACY-PROGRAM.md §2.2.3 |
```
