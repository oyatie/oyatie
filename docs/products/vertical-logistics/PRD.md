# Oyatie — Product PRD: Vertical Logistics

> **Status:** preview
> **Owning team:** [`teams/vertical-logistics/CHARTER.md`](../../teams/vertical-logistics/CHARTER.md)
> **Owning axis:** vertical-logistics (Axis 2)
> **Catalog reference:** `registry/catalog/oya-vertical-logistics-*.yaml`
> **Last updated:** 2026-05-09 by architecture-council

---

## 1. North Star

Oyatie Vertical Logistics is an end-to-end freight and supply-chain operations platform covering shipment lifecycle management, dock scheduling, EDI X12 transaction processing (214 Shipment Status, 990 Response to Load Tender, 997 Functional Acknowledgment), route optimization, Hours of Service (HOS) compliance, and cold-chain temperature monitoring. Its canonical entity model — Shipment, LoadTender, Route, Stop, DockAppointment, HosLog, TemperatureReading — is designed for the full logistics network: carriers, freight brokers, 3PLs, and shippers. The product exists within Oyatie's ecosystem because the coupling of a logistics execution layer with Foundry-driven route optimization agents (under the autonomy ceiling), real-time EDI exchange via the canonical eventing backbone, audit-chain immutability for regulatory HOS and cold-chain evidence, and the Corporate vertical's GL for freight cost capture is the integration value that no standalone TMS (Transportation Management System) can match. The regional pack architecture means KR 화물자동차운수사업법 compliance, US FMCSA HOS electronic logging (ELD mandate), and EU CMR e-waybill are all first-class implementations of the same seam, not special cases.

---

## 2. Target Users

| Persona | What they get | What they pay for |
|---|---|---|
| Logistics Dispatcher | Shipment creation, load tender issuance (EDI 990), carrier tracking (EDI 214), dock appointment scheduling | Per-seat subscription (dispatch tier) |
| Carrier / Driver | Route assignment, HOS log (ELD-equivalent), proof-of-delivery capture, real-time GPS updates | Per-driver seat (mobile-first) |
| 3PL Operations Manager | Multi-shipper shipment visibility, warehouse dock management, carrier performance analytics | Per-seat (3PL tier) + per-shipment metering |
| Cold-Chain Quality Manager | Temperature deviation alerts, cold-chain audit trail, ATP (Acceptance Temperature Profile) compliance | Per-seat (cold-chain add-on) |
| Freight Accountant | Freight invoice reconciliation, accessorial charge management, GL freight cost posting (Corporate vertical) | Included in Corporate GL tier |
| Logistics IT / Tenant Builder | EDI partner onboarding, route optimization model configuration, Foundry agent workflow authoring | Builder seat |
| Regulator / Auditor (FMCSA, MLTM KR, EU TEN-T) | ELD / HOS records, carrier safety ratings, temperature log evidence, CMR e-waybill | Cost of doing business |

---

## 3. In-Scope / Out-of-Scope

### 3.1 In-scope at each wave

| Wave | Capabilities | Surfaces exposed |
|---|---|---|
| Vertical-Preview | Shipment creation and tracking, load tender (EDI 990), shipment status (EDI 214), functional acknowledgment (EDI 997), dock appointment scheduling, basic route planning (Foundry route-opt agent — recommend only), KR 화물 운송 기록 | REST API v1, EDI X12 MLLP/AS2 gateway, Web UI |
| Vertical-Stable | Full EDI X12 transaction set (204 Motor Carrier Load Tender, 210 Freight Invoice, 211 Motor Carrier Bill of Lading, 214 Shipment Status, 990/997), HOS electronic log (FMCSA ELD mandate compliance for US), cold-chain temperature monitoring (IoT sensor ingestion via MQTT/HTTP), proof-of-delivery (POD) with e-signature, freight invoice reconciliation (3-way match: load tender → BOL → invoice), Foundry route optimizer (recommend + execute with planner approval), carrier performance scorecard | REST API stable, EDI AS2, Mobile (driver app), Webhook console |
| Public-GA | Multi-modal freight (FTL, LTL, intermodal, ocean, air — booking APIs), real-time GPS tracking feed, Foundry autonomous dispatch (T2 autonomy, dispatcher approval), carbon-footprint per shipment (GHG Protocol Scope 3), KR 전자운송장, EU CMR e-waybill, IATA e-Air Waybill | Public OpenAPI, Analytics dashboard, Carrier portal |
| Region-Fan-Out | Per-regional-pack HOS rules, local carrier identity, local tax-invoice (화물 적재물), local customs integration | Per-pack launch cadence |

### 3.2 Out-of-scope (anti-scope)

- Ocean freight customs brokerage and HS classification at depth (declared as a seam; customs broker adapter is a regional-pack concern)
- Fleet maintenance and asset management (that is the Industrial vertical if the fleet is an industrial asset; standalone fleet-mgmt is not in-scope)
- Last-mile consumer delivery tracking UX (B2B logistics operations platform; consumer-facing delivery notifications are a notification layer on top)
- Advertising targeting using shipment or driver data — always blocked (`BEHAVIORAL_TENANT_PRODUCT` and `PII_IDENTIFYING` data classes for driver data; PRIVACY-PROGRAM §2.2.3 corporate default applies)
- Real-time PLC/safety-rated control of warehouse robotics (OPC UA read for conveyor status is in-scope; PLC write is not — same rule as Industrial vertical)

---

## 4. Architecture Overview

### 4.1 Bounded Context

Axis 2 — Vertical Logistics. Flat-crates target prefix: `crates/oya-vertical-logistics-*`.

The logistics vertical owns the shipment, EDI exchange, dock, route, HOS, and cold-chain bounded contexts. Cross-axis contracts: `oya-platform-tenant-kernel`, `oya-platform-audit-chain-kernel` (HOS/ELD + cold-chain + EDI audit), `oya-foundry-api` (route optimizer + dispatch agents), `oya-vertical-corporate-domain-gl` (freight cost posting seam), `oya-platform-regulatory-kernel` (FMCSA/MLTM/EU-TEN-T packs).

### 4.2 Layered Structure

```
crates/oya-vertical-logistics-kernel-shipment/     — Shipment, LoadTender, BillOfLading, ProofOfDelivery entities
crates/oya-vertical-logistics-kernel-route/        — Route, Stop, RouteSegment, CarrierAssignment entities
crates/oya-vertical-logistics-kernel-dock/         — DockDoor, DockAppointment, YardSlot entities
crates/oya-vertical-logistics-kernel-hos/          — HosLog, HosDutyStatus, EldEvent entities (FMCSA ELD aligned)
crates/oya-vertical-logistics-kernel-coldchain/    — TemperatureReading, ColdChainProfile, TemperatureAlert entities
crates/oya-vertical-logistics-kernel-edi/          — EdiTransaction, EdiEnvelope, EdiAcknowledgment value objects (X12 aligned)
crates/oya-vertical-logistics-domain-shipment/     — Shipment lifecycle use cases: create, tender, track, deliver, invoice
crates/oya-vertical-logistics-domain-route/        — Route planning, carrier assignment, stop sequence use cases
crates/oya-vertical-logistics-domain-hos/          — HOS computation, violation detection, ELD submission use cases
crates/oya-vertical-logistics-domain-coldchain/    — Temperature monitoring, deviation alert, ATP compliance use cases
crates/oya-vertical-logistics-app-dispatch/        — Dispatch saga, Foundry route-opt capability delegation
crates/oya-vertical-logistics-app-edi/             — EDI X12 transaction saga (send/receive/acknowledge)
crates/oya-vertical-logistics-adapter-db/          — Postgres + TimescaleDB adapters
crates/oya-vertical-logistics-adapter-edi/         — EDI X12 AS2 / MLLP gateway adapter
crates/oya-vertical-logistics-adapter-gps/         — GPS/telematics ingestion adapter (MQTT/HTTP)
crates/oya-vertical-logistics-adapter-coldchain/   — IoT temperature sensor ingestion adapter
crates/oya-vertical-logistics-api-rest/            — REST API handlers
crates/oya-vertical-logistics-api-edi/             — EDI inbound gateway (AS2 + MLLP)
crates/oya-vertical-logistics-worker-events/       — Kafka consumers (shipment events, GPS updates)
crates/oya-vertical-logistics-runtime/             — Composition root binary
```

### 4.3 External-Facing Surfaces

| Surface | Contract location | Plane | SLO target |
|---|---|---|---|
| Logistics REST API | `contracts/logistics-tms.openapi.yaml` | Data | 99.9% / p95 < 200ms |
| EDI X12 AS2 gateway | `contracts/logistics-edi-as2.yaml` | Data | 99.9% / p95 < 5s (AS2 MDN) |
| EDI X12 MLLP gateway | `contracts/logistics-edi-mllp.yaml` | Data | 99.9% / p95 < 1s |
| GPS/telematics ingest | `contracts/logistics-gps.openapi.yaml` | Data | 99.5% / p99 < 2s |
| Cold-chain telemetry ingest | `contracts/logistics-coldchain.openapi.yaml` | Data | 99.5% / p99 < 5s |
| Webhook events (shipment-delivered, temperature-alert) | `contracts/logistics-webhooks.yaml` | Data | at-least-once, ≤ 30s |
| Analytics (route performance, carrier scorecard) | internal projection API | Analytics | best-effort |

### 4.4 Internal Seams

| Seam | Trait / interface | Consumer products |
|---|---|---|
| `FreightCostPostable` | `GlCostPostable` trait | Corporate vertical GL (freight cost per GL period) |
| `ShipmentSearchIndexable` | `SearchIndexable` (tenant-private) | Search axis (shipment lookup by BOL, reference number) |
| `HosEvidenceEmitter` | `AuditChainEmitter` | Audit chain (FMCSA HOS records, mandatory) |
| `ColdChainEvidenceEmitter` | `AuditChainEmitter` | Audit chain (temperature deviation, regulatory) |

### 4.5 Dependencies on Other Axes

| Contract consumed | Owner axis | Where it lives | Change-review class |
|---|---|---|---|
| `Tenant` kernel | SaaS platform | `oya-platform-tenant-kernel` | Cross-axis review |
| `Capability invocation` (route optimizer, dispatch) | Foundry | `oya-foundry-api` | Foundry + logistics review |
| `Audit-chain event` (HOS + cold-chain mandatory) | Platform | `oya-platform-audit-chain-kernel` | Audit review |
| `GlCostPostable` seam | Corporate vertical | `oya-vertical-corporate-domain-gl` | Cross-vertical review |
| `RegulatoryPack` seam | Platform regulatory | `oya-platform-regulatory-kernel` | Regulatory + logistics review |
| `PaymentRail` seam (freight invoice settlement) | Regional pack | `oya-saas-billing-rail-kernel` | Rail + regional review |

---

## 5. Data Structures

### 5.1 Kernel Entities

```rust
// crates/oya-vertical-logistics-kernel-shipment

/// data_class: BEHAVIORAL_TENANT_PRODUCT (shipment data; no PHI except driver PII_IDENTIFYING)
/// plane: data
pub struct Shipment {
    pub id: ShipmentId,
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub reference_number: String,                  // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub pro_number: Option<String>,                // data_class: BEHAVIORAL_TENANT_PRODUCT (carrier PRO)
    pub bol_number: Option<String>,                // data_class: BEHAVIORAL_TENANT_PRODUCT (Bill of Lading)
    pub shipper_ref: PartyRef,                     // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub consignee_ref: PartyRef,                   // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub carrier_ref: Option<CarrierRef>,           // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub status: ShipmentStatus,                    // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub mode: FreightMode,                         // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub service_type: ServiceType,                 // data_class: BEHAVIORAL_TENANT_PRODUCT (FTL, LTL, etc.)
    pub commodity: Vec<CommodityLine>,             // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub total_weight: Option<Weight>,
    pub total_pieces: Option<u32>,
    pub is_cold_chain: bool,                       // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub cold_chain_profile_id: Option<ColdChainProfileId>,
    pub pickup_date: Option<NaiveDate>,
    pub delivery_date: Option<NaiveDate>,
    pub route_id: Option<RouteId>,
    pub edi_transaction_ids: Vec<EdiTransactionId>,
    pub foundry_run_id: Option<FoundryRunId>,      // data_class: INTERNAL_ONLY
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum ShipmentStatus {
    Draft, Tendered, Accepted, InTransit, AtStop,
    OutForDelivery, Delivered, Exception, Cancelled
}
pub enum FreightMode { Truckload, LtlTruck, Intermodal, Ocean, Air, Parcel, Rail }
pub enum ServiceType { Ftl, Ltl, Volume, PartialTruckload, Dedicated }

/// EDI X12 214 / 990 / 997 transaction record
/// data_class: BEHAVIORAL_TENANT_PRODUCT
/// plane: data
pub struct EdiTransaction {
    pub id: EdiTransactionId,
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub transaction_set_id: EdiTransactionSetId,  // 214, 990, 997, 204, 210, 211
    pub interchange_control_number: String,
    pub functional_group_control_number: String,
    pub transaction_set_control_number: String,
    pub trading_partner_id: TradingPartnerId,     // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub direction: EdiDirection,
    pub raw_edi: EncryptedBlob,                   // data_class: BEHAVIORAL_TENANT_PRODUCT (encrypted)
    pub parsed_payload: serde_json::Value,        // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub shipment_id: Option<ShipmentId>,
    pub ack_status: EdiAckStatus,
    pub received_at: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum EdiTransactionSetId {
    T204, T210, T211, T214, T990, T997
}
pub enum EdiDirection { Inbound, Outbound }
pub enum EdiAckStatus { Pending, Accepted, Rejected, PartiallyAccepted }
```

```rust
// crates/oya-vertical-logistics-kernel-route

/// data_class: BEHAVIORAL_TENANT_PRODUCT
/// plane: data
pub struct Route {
    pub id: RouteId,
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub shipment_ids: Vec<ShipmentId>,
    pub carrier_ref: CarrierRef,                  // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub driver_id: Option<DriverId>,              // data_class: PII_IDENTIFYING (driver personal data)
    pub vehicle_id: Option<VehicleId>,            // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub stops: Vec<RouteStop>,
    pub planned_distance_km: Option<Decimal>,
    pub planned_duration_min: Option<u32>,
    pub status: RouteStatus,
    pub optimization_score: Option<f64>,          // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub foundry_optimization_run_id: Option<FoundryRunId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct RouteStop {
    pub sequence: u32,
    pub location: GeoLocation,                    // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub stop_type: StopType,
    pub planned_arrival: Option<DateTime<Utc>>,
    pub planned_departure: Option<DateTime<Utc>>,
    pub actual_arrival: Option<DateTime<Utc>>,
    pub actual_departure: Option<DateTime<Utc>>,
    pub dock_appointment_id: Option<DockAppointmentId>,
}

pub enum StopType { Pickup, Delivery, Fuel, Rest, CrossDock }
pub enum RouteStatus { Draft, Optimized, Dispatched, InProgress, Completed, Cancelled }
```

```rust
// crates/oya-vertical-logistics-kernel-hos

/// FMCSA ELD-aligned Hours of Service log
/// data_class: PII_IDENTIFYING (driver identity); BEHAVIORAL_TENANT_PRODUCT (duty status)
/// plane: data
pub struct HosLog {
    pub id: HosLogId,
    pub tenant_id: TenantId,
    pub driver_id: DriverId,                      // data_class: PII_IDENTIFYING
    pub region: RegionCode,
    pub schema_version: u32,
    pub log_date: NaiveDate,                      // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub vehicle_id: Option<VehicleId>,
    pub carrier_usdot: Option<String>,            // data_class: BEHAVIORAL_TENANT_PRODUCT (US DOT number)
    pub events: Vec<HosDutyEvent>,
    pub violations: Vec<HosViolation>,
    pub certifier_signature: Option<DriverSignature>,
    pub eld_malfunction: bool,
    pub regulatory_pack_id: RegulatoryPackId,     // which HOS ruleset (US, KR, EU)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct HosDutyEvent {
    pub timestamp: DateTime<Utc>,
    pub duty_status: HosDutyStatus,
    pub location: GeoLocation,                    // data_class: PII_IDENTIFYING (driver location)
    pub odometer: Option<u32>,
    pub engine_hours: Option<Decimal>,
    pub diagnostic_code: Option<String>,
}

pub enum HosDutyStatus { OffDuty, SleeperBerth, Driving, OnDutyNotDriving, PersonalConveyance, YardMoves }

pub struct HosViolation {
    pub violation_type: HosViolationType,
    pub detected_at: DateTime<Utc>,
    pub severity: ViolationSeverity,
}
pub enum HosViolationType { DrivingExceeds11Hours, OnDutyExceeds14Hours, RestBreakRequired, CycleLimit }
```

```rust
// crates/oya-vertical-logistics-kernel-coldchain

/// data_class: BEHAVIORAL_TENANT_PRODUCT
/// plane: data
pub struct TemperatureReading {
    pub id: TemperatureReadingId,
    pub tenant_id: TenantId,
    pub shipment_id: ShipmentId,
    pub sensor_id: SensorId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub temperature_celsius: Decimal,            // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub humidity_pct: Option<Decimal>,           // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub location: Option<GeoLocation>,           // data_class: BEHAVIORAL_TENANT_PRODUCT (cargo, not driver)
    pub reading_time: DateTime<Utc>,             // IoT timestamp
    pub ingested_at: DateTime<Utc>,
    pub quality: ReadingQuality,
}

pub enum ReadingQuality { Good, Estimated, BadData }

/// data_class: BEHAVIORAL_TENANT_PRODUCT
pub struct ColdChainProfile {
    pub id: ColdChainProfileId,
    pub tenant_id: TenantId,
    pub name: String,
    pub min_temp_celsius: Decimal,
    pub max_temp_celsius: Decimal,
    pub max_excursion_minutes: u32,           // total allowable excursion duration
    pub alert_threshold_celsius: Decimal,    // pre-alert before breach
    pub product_class: ColdChainProductClass,
}

pub enum ColdChainProductClass {
    Frozen,           // ≤ -18°C
    Chilled,          // 2–8°C pharmaceutical / 0–4°C food
    CoolAmbient,      // 15–25°C (pharma controlled room temp)
    Ambient,
}
```

```rust
// crates/oya-vertical-logistics-kernel-dock

/// data_class: BEHAVIORAL_TENANT_PRODUCT
/// plane: data
pub struct DockAppointment {
    pub id: DockAppointmentId,
    pub tenant_id: TenantId,
    pub facility_id: FacilityId,
    pub dock_door_id: DockDoorId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub shipment_id: ShipmentId,
    pub carrier_ref: CarrierRef,
    pub appointment_type: AppointmentType,       // Inbound / Outbound / Transfer
    pub scheduled_start: DateTime<Utc>,
    pub scheduled_end: DateTime<Utc>,
    pub actual_arrival: Option<DateTime<Utc>>,
    pub actual_departure: Option<DateTime<Utc>>,
    pub status: AppointmentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum AppointmentType { Inbound, Outbound, Transfer }
pub enum AppointmentStatus { Scheduled, CheckedIn, Loading, Completed, NoShow, Cancelled }
```

### 5.2 Aggregate Boundaries

| Aggregate | Root entity | Consistency boundary |
|---|---|---|
| `ShipmentAggregate` | `Shipment` + `EdiTransaction[]` | Shipment lifecycle + all EDI transactions for it; Route is a separate aggregate with shipment refs |
| `RouteAggregate` | `Route` + `RouteStop[]` | One route with all stops; stops are inline (value objects within the route) |
| `DockScheduleAggregate` | `DockDoor` + `DockAppointment[]` | Dock door schedule; time-slot conflicts enforced within this aggregate |
| `HosLogAggregate` | `HosLog` + `HosDutyEvent[]` | One driver's HOS log for one day; ELD events are inline |
| `ColdChainShipmentAggregate` | `TemperatureReading[]` for one shipment | All temperature readings for a cold-chain shipment; deviations computed from this aggregate |

### 5.3 Persistence Layout

| Aggregate | Store | Sharding key | Partition strategy | Replication | Retention |
|---|---|---|---|---|---|
| Shipment + EdiTransaction | Postgres (per-route shard) | `(tenant_id, route_region)` | Shuffle sharding on route corridor | Streaming replication × 2 | 7 years (KR 화물법, US DOT, EU CMR) |
| Route | Postgres (per-tenant shard) | `tenant_id` | Per-tenant schema | Streaming replication × 2 | 3 years |
| HosLog | Postgres (per-driver shard) | `driver_id` (shuffle) | Per-driver shuffle; immutable after certification | Streaming replication × 3 (regulatory) | 6 months minimum per FMCSA §395.8; indefinite for KR |
| TemperatureReading | TimescaleDB (per-shipment hypertable) | `(tenant_id, shipment_id, reading_time)` | Time-series hypertable | Streaming replication × 2 | 2 years hot; archive |
| DockAppointment | Postgres (per-facility shard) | `(tenant_id, facility_id)` | Per-facility | Streaming replication × 2 | 3 years |

### 5.4 Event Schemas

| Event name | Topic | Schema location | Consumer aggregates | Retention | Idempotency key |
|---|---|---|---|---|---|
| `ShipmentTendered` | `logistics.shipment.tendered` | `contracts/events/logistics-shipment.json` | EDI 990 generation, Audit chain | 30 days | `shipment_id` |
| `ShipmentStatusUpdated` | `logistics.shipment.status` | `contracts/events/logistics-shipment.json` | EDI 214 generation, Notification, Audit chain | 30 days | `(shipment_id, status, timestamp)` |
| `RouteOptimized` | `logistics.route.optimized` | `contracts/events/logistics-route.json` | Dispatch saga, Audit chain | 30 days | `route_id` |
| `HosViolationDetected` | `logistics.hos.violation` | `contracts/events/logistics-hos.json` | Compliance alert, Audit chain (mandatory) | 365 days | `(driver_id, violation_type, timestamp)` |
| `TemperatureAlert` | `logistics.coldchain.alert` | `contracts/events/logistics-coldchain.json` | Quality manager notification, Audit chain | 90 days | `(shipment_id, sensor_id, timestamp)` |
| `ShipmentDelivered` | `logistics.shipment.delivered` | `contracts/events/logistics-shipment.json` | Freight invoice trigger, GL cost posting (Corporate), Audit chain | 30 days | `shipment_id` |
| `EdiAcknowledged` | `logistics.edi.acked` | `contracts/events/logistics-edi.json` | Shipment status update, Audit chain | 30 days | `edi_transaction_id` |

### 5.5 Index / Search-Index Touchpoints

| Entity field | Index | Class allowed | Cascade-on-DSR? |
|---|---|---|---|
| `Shipment.reference_number`, `bol_number` | tenant-private search | `BEHAVIORAL_TENANT_PRODUCT` | No (regulatory retention) |
| `Shipment.consignee_ref.name` | tenant-private search | `BEHAVIORAL_TENANT_PRODUCT` | No |
| `HosLog.driver_id` lookup | tenant-private driver directory | `PII_IDENTIFYING` (tenant-private only) | Yes — DSR cascade on driver data |

**Note:** Driver `HosLog` data carries `PII_IDENTIFYING` because it links to an identified driver. DSR cascade applies for the driver's personal data fields. HOS records required for regulatory retention are pseudonymized but retained.

### 5.6 Audit-Chain Emission Contract

| Operation | Emits topic | Required fields |
|---|---|---|
| HOS log certified by driver | `audit.logistics.hos.certified` | `driver_id` (pseudonymized), `log_date`, `certifier_signature_hash`, `regulatory_pack_id` |
| HOS violation detected | `audit.logistics.hos.violation` | `driver_id` (pseudonymized), `violation_type`, `severity`, `detected_at`, `carrier_usdot` |
| Cold-chain temperature excursion | `audit.logistics.coldchain.excursion` | `shipment_id`, `sensor_id`, `max_temp`, `excursion_start`, `excursion_duration_min`, `product_class` |
| EDI 214 shipment status transmitted | `audit.logistics.edi.214_sent` | `edi_transaction_id`, `trading_partner_id`, `shipment_id`, `status_code` |
| Proof of delivery captured | `audit.logistics.shipment.pod` | `shipment_id`, `driver_id` (pseudonymized), `delivery_time`, `signature_hash` |

### 5.7 Schema Migration Policy

- HOS log schema is append-only after driver certification; no destructive migrations.
- EdiTransaction raw EDI is stored encrypted; schema changes to parsed_payload are additive.
- TimescaleDB chunk schema changes (cold-chain readings) use Timescale's migration tooling.

---

## 6. Optimization Practices

| Practice | Implementation choice |
|---|---|
| Cell routing | `(tenant_id, route_region)` → cell; logistics cells are aligned to regional transport corridors |
| Sharding strategy | Per-route-region shard for Shipment; per-driver shuffle sharding for HOS; per-shipment hypertable for cold-chain readings |
| Caching tier | Redis for active route state (updated by GPS ingest); in-memory for dock-door availability schedule (per-facility); no driver PII in Redis |
| Bulk endpoint contract | `POST /shipments/bulk` (mass shipment creation for import); `POST /temperature-readings/bulk` (IoT batch ingest); `POST /hos-events/bulk` (ELD bulk upload) |
| Pagination | Cursor on `(created_at, shipment_id)` for shipments; `_since` filter for status queries; EDI transaction list paginated by `received_at` |
| Idempotency | `Idempotency-Key` on shipment creation and EDI send; HOS events deduplicated on `(driver_id, timestamp, duty_status)` |
| Batch dispatch | Foundry `RouteOptimizer` runs per-depot batch at shift-start and on-demand; EDI 997 acknowledgments batched per functional group |
| Backpressure | GPS ingest applies per-vehicle rate limit (max 1 update/30s); cold-chain MQTT ingest circuit-breaker on sensor flood; Kafka consumer lag monitored |
| Hot-path benchmarks | `shipment_create` criterion < 100ms; `edi_214_generate` < 200ms; `temperature_reading_ingest` < 10ms per reading |
| Agent-driven optimization | Foundry `RouteOptimizer` (multi-stop VRP with time-windows; recommend + planner approval at T1 Preview, T2 Stable); Foundry `ColdChainMonitor` (real-time excursion prediction from trend analysis) |
| FinOps unit-economics | Per-shipment metering; per-driver HOS log day; per-temperature-reading (IoT storage); Foundry optimization invocations metered |
| Build-cache / CI affected-graph | `oya-vertical-logistics-kernel-shipment` → full rebuild; `adapter-edi` → EDI conformance test suite; `adapter-coldchain` → temperature simulation test |

---

## 7. Regional Pack Interactions

| Seam | Trait | Per-pack impl needed? | Tested with which packs? |
|---|---|---|---|
| HOS ruleset (driving hours regulations) | `HosRuleEngine` | Yes — every trucking-capable region | `oya-pack-us` (FMCSA 49 CFR Part 395 ELD mandate, property/passenger carrier rules), `oya-pack-kr` (화물자동차운수사업법, 운수종사자 자격), `oya-pack-eu` (EU Regulation 561/2006 + AETR, digital tachograph) |
| Carrier identity + safety rating | `CarrierIdentityProvider` | Yes | `oya-pack-us` (FMCSA SAFER DB, USDOT number), `oya-pack-kr` (국토교통부 화물운송 허가), `oya-pack-eu` (EU operator license + tachograph card) |
| Electronic waybill / BOL format | `WaybillFormatter` | Yes | `oya-pack-kr` (전자운송장 + KTNET), `oya-pack-us` (ANSI X12 211 BOL), `oya-pack-eu` (CMR e-waybill per UN/CEFACT) |
| Customs declaration | `CustomsAdapter` (declared seam, not yet impl) | Yes — regional customs APIs | `oya-pack-us` (ACE, Importer Security Filing), `oya-pack-eu` (ICS2 / NCTS), `oya-pack-kr` (관세청 UNI-PASS) |
| Regulatory control evidence | `RegulatoryPack` | Yes | `oya-pack-kr` (MLTM, 한국도로공사), `oya-pack-us` (FMCSA, DOT), `oya-pack-eu` (EC 561/2006 enforcement) |
| Calendar / working hours | `LocaleFormatter` | Yes | All onboarded packs |

### Regulatory Pack Declaration

```yaml
# registry/catalog/oya-vertical-logistics-runtime.yaml
regulatory_packs:
  - oya-pack-kr   # 화물자동차운수사업법, 운수종사자, MLTM
  - oya-pack-us   # FMCSA 49 CFR Part 395 ELD, USDOT, PHMSA (hazmat)
  - oya-pack-eu   # EU Reg 561/2006, AETR, CMR e-waybill, digital tachograph
```

---

## 8. In-House vs External Dependency Posture

| External dep | Maturity tier | License | In-house alternative considered? | Decision |
|---|---|---|---|---|
| `tokio`, `axum`, `sqlx`, `serde`, `rustls` | kernel-grade | MIT / Apache-2 | No | Use |
| `x12-rs` (EDI X12 parser) | early-stable | MIT | In-house EDI parser strongly considered for correctness | Build in-house in `oya-vertical-logistics-kernel-edi`; x12-rs as reference; own the parser for full control of transaction set coverage |
| TimescaleDB | stable | Apache-2 | Pure Postgres insufficient for GPS + temperature at scale | Use |
| `rumqttc` (MQTT for IoT sensor ingest) | stable | Apache-2 | In-house MQTT client considered | Use |
| `geo` (geospatial types + distance) | stable | MIT / Apache-2 | In-house geo types considered | Use for distance computation; PostGIS for spatial queries in adapter |
| OR-Tools (Google, vehicle routing) | mature | Apache-2 | In-house VRP solver considered — OR-Tools is the industry benchmark | Use OR-Tools via FFI wrapper in Foundry route-opt capability; route opt stays in Foundry layer, not kernel |
| AS2 gateway (`portier`-compatible) | no mature Rust crate | — | No mature AS2 crate exists | Build in-house AS2 adapter in `oya-vertical-logistics-api-edi` |

---

## 9. Success Metrics

| Metric | Vertical-Preview target | Vertical-Stable target | Public-GA target |
|---|---|---|---|
| Active shipments tracked | ≥ 1,000 (design-partner 3PL) | ≥ 100,000 | ≥ 1,000,000 |
| EDI 214 generation latency P99 | < 1s | < 500ms | < 200ms |
| EDI 997 acknowledgment round-trip | < 30s | < 10s | < 5s |
| Cold-chain temperature alert latency (excursion → alert) | < 5 min | < 2 min | < 1 min |
| HOS audit-chain completeness | 100% | 100% | 100% |
| Foundry route optimization acceptance rate (planner agrees) | ≥ 60% | ≥ 75% | ≥ 85% |
| Route optimization cost improvement vs baseline | ≥ 5% | ≥ 12% | ≥ 18% |
| Driver DSR fulfillment time | < 30 days | < 10 days | < 5 days |
| Cold-chain compliance rate (no excursion at delivery) | ≥ 95% | ≥ 98% | ≥ 99% |
| Cross-axis contract violations | 0 | 0 | 0 |

---

## 10. Risks + Mitigations

| Risk | Severity | Mitigation | Owner |
|---|---|---|---|
| HOS ELD data falsification (driver under pressure) | Critical | ELD events immutable after GPS-timestamp anchor; anomaly detection (speed vs duty status mismatch) via Foundry HOS monitor; FMCSA-equivalent random audit sampling | Compliance + Foundry |
| Cold-chain excursion not detected in time (food safety / pharma) | Critical | Real-time MQTT ingest with < 5 min alert SLO at Stable; redundant sensor reading (two sensors per trailer); automatic reject flag on POD if unresolved excursion | Cold-chain domain + SRE |
| EDI partner trading-partner configuration error (wrong ISA06/ISA08) | High | AS2 / MLLP gateway validates ISA envelope before processing; 997 rejection routed to operations alert queue | EDI domain + Partner ops |
| Driver PII (location history) exposure | High | HosLog driver location is `PII_IDENTIFYING`; tenant-private only; DSR cascade within 30 days of request; no cross-tenant aggregation of driver location | Privacy |
| Route optimizer recommending infeasible route (HOS violation) | High | RouteOptimizer Foundry capability validates output against HOS ruleset before proposing; planner UI shows HOS compliance score for each proposed route | Foundry + HOS domain |
| GPS ingest storm (rogue telematics device flooding endpoint) | Medium | Per-device rate limit (1 update/30s); device authentication (mTLS + device certificate); circuit breaker on ingest worker | SRE + Security |
| Customs clearance delay causing cold-chain breach | Medium | Cold-chain alert escalation includes customs hold status; pre-clearance ISF filing seam (US) | Regional pack + Cold-chain domain |
| KR 화물법 amendment (driver rest requirements) | Medium | Regulatory-change watch lane; KR pack versioned; HOS rule engine is pluggable | KR pack + Compliance |

---

## 11. Open Questions

- OR-Tools VRP integration: deploy as a Foundry capability binary (recommended) or as a sidecar to the logistics runtime? Capacity / licensing discussion needed.
- Hazmat (PHMSA / UN ADR / IATA DGR) — in-scope for Vertical-Stable or separate regulatory add-on?
- Ocean freight booking API (INTTRA / CargoSphere / GT Nexus) — direct integration or buyer's-choice adapter?
- Cross-border truck GPS tracking: data residency of driver location data when crossing KR ↔ CN or KR ↔ JP ferry? Which regional pack governs?
- Driver app: native mobile (Tauri/React Native) or PWA? Decision affects mobile HOS log UX significantly.

---

## 12. Decision Log

| Decision | Date | Rationale | ADR ref |
|---|---|---|---|
| EDI X12 parser built in-house | 2026-05-09 | No mature Rust X12 crate with full transaction set coverage; logistics correctness depends on parser | — |
| HOS ruleset via regional pack (not hardcoded) | 2026-05-09 | FMCSA (US), EU 561/2006, KR 화물법 all have different hours rules; pluggable engine required | DESIGN.md §12 |
| OR-Tools VRP via Foundry capability (not in kernel) | 2026-05-09 | Route optimization is a Foundry-delegated capability; kernel stays pure | — |
| AS2 gateway built in-house | 2026-05-09 | No mature AS2 Rust crate; AS2 is critical for EDI trading partner connectivity | — |
| Cold-chain readings in TimescaleDB hypertable | 2026-05-09 | IoT-scale ingest (1K+ readings/min per tenant at Stable) requires time-series optimization | — |
| Flat-crates: `crates/oya-vertical-logistics-*` | 2026-05-09 | Per ADR-0015 | ADR-0015 |

---

## 13. Sources Scanned

- `docs/PRD.md`, `docs/DESIGN.md` §1, §4, §10, §12
- `docs/PRIVACY-PROGRAM.md` §2.2.1, §2.2.3
- FMCSA 49 CFR Part 395 (ELD mandate); EDI X12 214/990/997/204/210/211 transaction sets; EU Regulation 561/2006 (driving hours); KR 화물자동차운수사업법

---

## Doc-Catalog Row

```
| `vertical-logistics` | `vertical-2` | TMS/EDI X12/HOS-ELD/cold-chain/route-opt | monthly | PRD.md, DESIGN.md §12, PRIVACY-PROGRAM.md §2.2.3 |
```
