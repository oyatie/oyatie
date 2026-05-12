# Oyatie — Product PRD: Vertical Agriculture

> **Status:** preview (skeleton)
> **Owning team:** [`teams/vertical-agriculture/CHARTER.md`](../../teams/vertical-agriculture/CHARTER.md)
> **Owning axis:** vertical-agriculture (Axis 2)
> **Catalog reference:** `registry/catalog/oya-vertical-agriculture-*.yaml`
> **Last updated:** 2026-05-09 by architecture-council

---

## 1. North Star

Oyatie Vertical Agriculture is the farm management and agricultural supply-chain traceability platform for growers, agri-cooperatives, food processors, and retailers. It covers farm management (field recording, crop lifecycle, input application — fertilizer/pesticide), produce traceability (lot/batch tracking from seed to shelf, GS1 EPCIS aligned), and compliance management (GlobalG.A.P., KR GAP 우수농산물관리제도, USDA organic certification, EU farm-to-fork). It exists within Oyatie's ecosystem because the coupling of farm traceability with the Food vertical's supply-chain compliance (HACCP — see `vertical-food`), the Logistics vertical's cold-chain monitoring for produce transport, Foundry-driven crop advisory agents, and the Corporate vertical's GL for farm cost accounting creates the agricultural value chain platform that no standalone farm management or traceability SaaS can replicate end-to-end.

---

## 2. Target Users

| Persona | What they get | What they pay for |
|---|---|---|
| Farmer / Grower | Field recording, spray diary (pesticide/fertilizer application), harvest log, weather data integration | Per-farm subscription |
| Agronomist | Crop advisory dashboard, soil test management, pest/disease alert, Foundry crop-health recommendation | Per-seat (agronomy tier) |
| Cooperative / Packer Manager | Produce intake, lot assignment, grading, packing, EPCIS traceability event generation | Per-seat (packing tier) |
| Food Safety Manager | GAP audit dashboard, pesticide MRL check, certification evidence, GlobalG.A.P. inspection record | Per-seat (compliance tier) |
| Retailer / Buyer (upstream link) | Produce traceability lookup (QR code → EPCIS chain), provenance verification | Via API (buyer access metered per query) |

---

## 3. In-Scope / Out-of-Scope

### 3.1 In-scope at each wave

| Wave | Capabilities | Surfaces exposed |
|---|---|---|
| Vertical-Preview | Farm field record (field, crop, planting, harvest), spray diary (pesticide/fertilizer input with PHI — Pre-Harvest Interval — check), KR 농약안전정보시스템 pesticide registration check, basic lot traceability (EPCIS ObjectEvent) | REST API v1, Web UI, Mobile (field) |
| Vertical-Stable | Full GS1 EPCIS 2.0 traceability (ObjectEvent, AggregationEvent, TransformationEvent, TransactionEvent), GlobalG.A.P. audit module, KR GAP 인증 documentation, USDA organic input log, soil test management, weather station integration (IoT — MQTT), Foundry crop-advisory agent (T1 — agronomist reviews), pesticide MRL (Maximum Residue Level) compliance check per market (KR / EU / US), cold-chain integration seam (Logistics vertical), F&B procurement seam (Food vertical) | REST API stable, Buyer traceability portal, Webhook console |
| Public-GA | Carbon sequestration per farm (Verra/Gold Standard protocol), satellite imagery integration (NDVI crop health), Foundry autonomous field scouting alert (drone/satellite data — T1), EU Farm-to-Fork compliance reporting, AgriConnect marketplace (input suppliers + buyers) | Public OpenAPI, Traceability consumer portal, Analytics |

### 3.2 Out-of-scope (anti-scope)

- Precision agriculture machine control (GPS auto-steer, variable rate application hardware) — IoT sensor data ingestion is in-scope; machine control is not
- Commodity trading / futures hedging
- Livestock management at veterinary record depth (a separate vertical evaluation)
- Advertising targeting using farm or grower data — `BEHAVIORAL_TENANT_PRODUCT`; PRIVACY-PROGRAM §2.2.3 corporate default

---

## 4. Architecture Overview

### 4.1 Bounded Context

Flat-crates target prefix: `crates/oya-vertical-agriculture-*`.

```
crates/oya-vertical-agriculture-kernel-farm/       — Farm, Field, Crop, PlantingRecord, HarvestRecord entities
crates/oya-vertical-agriculture-kernel-input/      — InputApplication, Pesticide, Fertilizer, PhiCheck entities
crates/oya-vertical-agriculture-kernel-traceability/ — Lot, EpcisEvent, TraceabilityChain entities (GS1 EPCIS 2.0)
crates/oya-vertical-agriculture-kernel-compliance/ — GapAudit, CertificationRecord, MrlCheck entities
crates/oya-vertical-agriculture-domain-*/          — Use cases per sub-domain
crates/oya-vertical-agriculture-app-*/             — Sagas + Foundry delegation
crates/oya-vertical-agriculture-adapter-*/         — DB, IoT/MQTT, GS1 EPCIS, weather station adapters
crates/oya-vertical-agriculture-api-rest/          — REST API
crates/oya-vertical-agriculture-api-traceability/  — Buyer traceability query API (public-facing)
crates/oya-vertical-agriculture-runtime/           — Composition root
```

### 4.3 External-Facing Surfaces

| Surface | Contract location | Plane | SLO target |
|---|---|---|---|
| Agriculture REST API | `contracts/agriculture-core.openapi.yaml` | Data | 99.9% / p95 < 300ms |
| Traceability query API (buyer-facing) | `contracts/agriculture-trace.openapi.yaml` | Data | 99.5% / p95 < 1s |
| GS1 EPCIS 2.0 REST API | `contracts/agriculture-epcis.openapi.yaml` | Data | 99.5% / p95 < 500ms |

### 4.4 Internal Seams

| Seam | Trait | Consumer products |
|---|---|---|
| `HarvestLotTraceability` | `TraceabilityChainProvider` | Food vertical (HACCP lot linking), Logistics vertical (cold-chain lot ID) |
| `FarmCostGlPostable` | `GlCostPostable` | Corporate GL (farm operating cost) |
| `ProduceSearchIndexable` | `SearchIndexable` (tenant-private) | Search axis (lot/batch lookup by grower/product) |

---

## 5. Data Structures

### 5.1 Kernel Entities

```rust
// crates/oya-vertical-agriculture-kernel-farm
/// data_class: BEHAVIORAL_TENANT_PRODUCT
/// plane: data
pub struct Farm {
    pub id: FarmId,
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub name: String,
    pub owner_name: Option<String>,         // data_class: PII_IDENTIFYING if individual
    pub address: Address,                   // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub total_area_ha: Decimal,
    pub certification_status: Vec<CertificationStatus>,
    pub gap_registered: bool,               // KR GAP 등록 여부
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct HarvestRecord {
    pub id: HarvestRecordId,
    pub tenant_id: TenantId,
    pub farm_id: FarmId,
    pub field_id: FieldId,
    pub crop_id: CropId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub harvest_date: NaiveDate,
    pub quantity_kg: Decimal,
    pub lot_id: LotId,                      // links to traceability kernel
    pub grade: Option<ProduceGrade>,
    pub phi_cleared: bool,                  // Pre-Harvest Interval compliance
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// crates/oya-vertical-agriculture-kernel-traceability
/// GS1 EPCIS 2.0 aligned
/// data_class: BEHAVIORAL_TENANT_PRODUCT
pub struct EpcisEvent {
    pub id: EpcisEventId,
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub event_type: EpcisEventType,
    pub event_time: DateTime<Utc>,
    pub epc_list: Vec<Epc>,               // GS1 EPC (SGTIN/SSCC)
    pub action: EpcisAction,
    pub biz_step: String,                 // GS1 CBV business step
    pub disposition: String,              // GS1 CBV disposition
    pub read_point: Option<String>,       // GLN
    pub biz_location: Option<String>,     // GLN
    pub source_list: Vec<EpcisSource>,
    pub destination_list: Vec<EpcisDestination>,
    pub ilmd: Option<serde_json::Value>,  // Instance/Lot Master Data
    pub created_at: DateTime<Utc>,
}
pub enum EpcisEventType { ObjectEvent, AggregationEvent, TransformationEvent, TransactionEvent, AssociationEvent }
pub enum EpcisAction { Add, Observe, Delete }

// crates/oya-vertical-agriculture-kernel-input
/// Pesticide application with PHI check
/// data_class: BEHAVIORAL_TENANT_PRODUCT
pub struct InputApplication {
    pub id: InputApplicationId,
    pub tenant_id: TenantId,
    pub farm_id: FarmId,
    pub field_id: FieldId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub input_type: InputType,            // Pesticide / Fertilizer / Irrigation
    pub product_name: String,
    pub active_ingredient: Option<String>,
    pub registration_number: Option<String>, // KR 농약등록번호 / US EPA reg / EU reg
    pub application_date: NaiveDate,
    pub quantity_applied: Decimal,
    pub application_uom: UnitOfMeasure,
    pub phi_days: Option<u32>,            // Pre-Harvest Interval
    pub earliest_harvest_date: Option<NaiveDate>, // application_date + phi_days
    pub applied_by: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
pub enum InputType { Pesticide, Fertilizer, Irrigation, SoilAmendment, GrowthRegulator }
```

> TODO v0.2 — vertical owner to add `Field`, `Crop`, `PlantingRecord`, `GapAudit`, `MrlCheck`, `CertificationRecord` entities with full fields.

### 5.2–5.7

> TODO v0.2 — vertical owner to expand aggregate boundaries, persistence layout, event schemas, audit-chain contract, migration policy.

Key audit events: `HarvestLotCreated`, `EpcisEventRecorded`, `MrlViolationDetected`, `GapAuditCompleted`.

---

## 6. Optimization Practices

| Practice | Implementation choice |
|---|---|
| Cell routing | `(tenant_id, farm_region)` → cell |
| Sharding strategy | Per-farm shard for field records; per-tenant shard for traceability events |
| Caching tier | Redis for active PHI check results (pesticide registration cache — refreshed daily from KR 농약안전정보시스템); in-memory for MRL threshold table |
| Bulk endpoint contract | `POST /epcis/events/bulk` (GS1 EPCIS bulk event capture at packing); `POST /harvest-records/bulk` |
| Agent-driven optimization | Foundry `CropAdvisor` (spray recommendation based on weather + pest pressure — T1, agronomist reviews); Foundry `MrlComplianceChecker` (automated pre-shipment MRL check across market thresholds) |

> TODO v0.2 — vertical owner to expand.

---

## 7. Regional Pack Interactions

| Seam | Trait | Per-pack impl needed? | Tested with |
|---|---|---|---|
| Pesticide registration database | `PesticideRegistryAdapter` | Yes | `oya-pack-kr` (농약안전정보시스템 APIS), `oya-pack-us` (EPA PPIS), `oya-pack-eu` (EU Pesticides DB) |
| GAP / organic certification standard | `CertificationStandardExtension` | Yes | `oya-pack-kr` (GAP 우수농산물관리제도), `oya-pack-us` (USDA NOP organic), `oya-pack-eu` (GlobalG.A.P., EU Organic Reg 2018/848) |
| MRL (Maximum Residue Level) threshold tables | `MrlThresholdProvider` | Yes | `oya-pack-kr` (식품의약품안전처 MRL), `oya-pack-us` (EPA tolerances 40 CFR 180), `oya-pack-eu` (EU MRL Reg 396/2005) |
| Regulatory control evidence | `RegulatoryPack` | Yes | `oya-pack-kr` (농촌진흥청, 식약처), `oya-pack-us` (USDA AMS, FDA FSMA), `oya-pack-eu` (EFSA, EU farm-to-fork) |

### Regulatory Pack Declaration

```yaml
regulatory_packs:
  - oya-pack-kr   # 농촌진흥청, 식품의약품안전처, GAP, 농약관리법, PIPA
  - oya-pack-us   # USDA NOP, FDA FSMA Produce Safety Rule, EPA
  - oya-pack-eu   # GlobalG.A.P., EU Organic 2018/848, EFSA, Farm-to-Fork
```

---

## 8. In-House vs External Dependency Posture

> TODO v0.2 — vertical owner to expand. Key: `tokio`/`axum`/`sqlx`/`serde`/`rustls` (kernel-grade); GS1 EPCIS 2.0 parsing in-house; `rumqttc` for IoT weather station (Apache-2); pesticide registry adapters in-house per regional pack.

---

## 9. Success Metrics

| Metric | Vertical-Preview target | Vertical-Stable target | Public-GA target |
|---|---|---|---|
| Farms under management | ≥ 10 (design partner cooperative) | ≥ 1,000 | ≥ 100,000 |
| PHI compliance check accuracy | 100% | 100% | 100% |
| EPCIS traceability event P99 | < 500ms | < 200ms | < 100ms |
| MRL violation detection before shipment | ≥ 95% | ≥ 99% | ≥ 99.9% |
| Traceability chain completeness (seed to shelf) | baseline | ≥ 90% | ≥ 99% |

---

## 10. Risks + Mitigations

| Risk | Severity | Mitigation | Owner |
|---|---|---|---|
| PHI (Pre-Harvest Interval) computation error (unsafe produce shipped) | Critical | PHI check is deterministic kernel computation; cross-validated against pesticide registry; harvest blocked if PHI not cleared | Agriculture domain + Food safety |
| MRL violation reaching export market (regulatory recall) | Critical | Pre-shipment MRL check via Foundry `MrlComplianceChecker`; lot held if any MRL check fails; audit record per check | Agriculture domain + Regional pack |
| EPCIS traceability chain gap (unlinked lot) | High | Lot creation requires parent EPCIS ObjectEvent; gap detection in TraceabilityChain aggregate | Agriculture domain |
| Pesticide registration data stale (daily refresh missed) | High | Freshness check on pesticide cache; stale-data alert; PHI check reverts to conservative default if cache > 48h stale | SRE + KR pack |

> TODO v0.2 — vertical owner to expand.

---

## 11. Open Questions

- Livestock traceability (cattle/pork/poultry) — same vertical or separate vertical-livestock evaluation?
- Drone/satellite NDVI data ingestion: direct S3-compatible object store or third-party imagery platform (Planet/Maxar)?
- Carbon credit protocol (Verra VCS / Gold Standard): automated credit calculation or manual attestation?

---

## 12. Decision Log

| Decision | Date | Rationale | ADR ref |
|---|---|---|---|
| GS1 EPCIS 2.0 as canonical traceability standard | 2026-05-09 | GS1 EPCIS is the global standard for food supply-chain traceability; adopted by EU Farm-to-Fork, FDA FSMA 204 | — |
| PHI check in kernel (not external API call) | 2026-05-09 | PHI check is safety-critical; deterministic in-house computation + regional pack registry data | — |
| Foundry crop advisory at T1 | 2026-05-09 | Spray recommendations have PHI and MRL implications; agronomist must approve | ADR-0050 |
| Flat-crates: `crates/oya-vertical-agriculture-*` | 2026-05-09 | Per ADR-0015 | ADR-0015 |

---

## 13. Sources Scanned

- `docs/PRD.md`, `docs/DESIGN.md` §1, §4, §12
- `docs/PRIVACY-PROGRAM.md` §2.2.3
- GS1 EPCIS 2.0 standard; EU Farm-to-Fork Strategy; FDA FSMA 204 (food traceability rule); KR 농약관리법

---

## Doc-Catalog Row

```
| `vertical-agriculture` | `vertical-2` | traceability/farm-mgmt/GAP/EPCIS/PHI-MRL | monthly | PRD.md, DESIGN.md §12 |
```
