# Oyatie — Product PRD: Vertical Food

> **Status:** preview (skeleton)
> **Owning team:** [`teams/vertical-food/CHARTER.md`](../../teams/vertical-food/CHARTER.md)
> **Owning axis:** vertical-food (Axis 2)
> **Catalog reference:** `registry/catalog/oya-vertical-food-*.yaml`
> **Last updated:** 2026-05-09 by architecture-council

---

## 1. North Star

Oyatie Vertical Food is the food supply-chain compliance and operations platform for food manufacturers, processors, distributors, and food service operators. It covers HACCP (Hazard Analysis and Critical Control Points) plan management, supplier qualification, food safety audit management, allergen and label compliance, recall management, and supply-chain traceability (GS1 EPCIS 2.0 aligned, linking to the Agriculture vertical's lot records upstream and the Logistics vertical's cold-chain data in transit). It exists within Oyatie's ecosystem because the coupling of HACCP compliance records with the audit chain (FDA FSMA 204 / KR 식품위생법 / EU Food Law 178/2002 traceability requirements), Foundry-driven supplier risk scoring and non-conformance root-cause agents, the Logistics vertical's cold-chain temperature records, and the Agriculture vertical's farm-to-factory lot traceability creates the end-to-end food safety operations platform that no standalone HACCP or food ERP system can offer with the same regulatory evidence depth and cross-vertical integration.

---

## 2. Target Users

| Persona | What they get | What they pay for |
|---|---|---|
| Food Safety Manager | HACCP plan management, CCP monitoring, corrective action workflow, audit dashboard | Per-seat (food safety tier) |
| Quality Assurance Manager | Supplier qualification, incoming inspection, non-conformance (NCR), CAPA | Per-seat (QA tier) |
| Production Manager | Production batch record, ingredient traceability, allergen control, yield tracking | Per-seat (production tier) |
| Regulatory Affairs Manager | Label compliance (nutrition facts, allergen declaration), regulatory submission workflow, recall management | Per-seat (regulatory tier) |
| Supply Chain Manager | Supplier scorecard, ingredient procurement lot linking, cold-chain event integration (Logistics) | Per-seat (supply chain tier) |
| Food IT / Tenant Builder | HACCP template config, supplier portal config, Foundry supplier-risk workflow authoring | Builder seat |

---

## 3. In-Scope / Out-of-Scope

### 3.1 In-scope at each wave

| Wave | Capabilities | Surfaces exposed |
|---|---|---|
| Vertical-Preview | HACCP plan creation (hazard analysis, CCP identification, critical limits, monitoring procedures), CCP monitoring log, basic corrective action record, KR 식품위생법 영업허가 compliance checklist, GS1 EPCIS traceability event ingestion from Agriculture vertical | REST API v1, Web UI, Mobile (CCP monitoring) |
| Vertical-Stable | Supplier qualification workflow (questionnaire, audit, approval), non-conformance management (NCR + CAPA), allergen management (ingredient × allergen matrix, production schedule allergen conflict alert), nutrition facts calculation, lot traceability (end-to-end EPCIS chain — farm → factory → distribution), recall management (lot identification, regulatory notification workflow, consumer/retailer alert), cold-chain integration (Logistics vertical temperature events on food lots), Foundry supplier risk scoring (T1 — QA manager reviews), FDA FSMA 204 traceability record compliance, EU Food Law 178/2002 traceability, KR 이력추적관리 | REST API stable, Supplier portal, Webhook console |
| Public-GA | AI-driven HACCP deviation root-cause analysis (Foundry, T1), cross-batch allergen exposure trace, carbon footprint per product (Scope 3 ingredient emissions), front-of-pack label compliance for 50+ markets (per regional pack), supply chain risk map | Public OpenAPI, Analytics dashboard, Recall consumer portal |

### 3.2 Out-of-scope (anti-scope)

- Restaurant point-of-sale (that is the Hospitality vertical's F&B module)
- Consumer recipe and nutrition app (B2B food safety platform only)
- Pesticide MRL checking at farm level (that is the Agriculture vertical's domain; Food vertical consumes the MRL-cleared lot record via traceability chain)
- Advertising targeting using food production or supplier data — `BEHAVIORAL_TENANT_PRODUCT`; PRIVACY-PROGRAM §2.2.3 corporate default

---

## 4. Architecture Overview

### 4.1 Bounded Context

Flat-crates target prefix: `crates/oya-vertical-food-*`.

```
crates/oya-vertical-food-kernel-haccp/          — HaccpPlan, HazardAnalysis, CriticalControlPoint, CcpLog entities
crates/oya-vertical-food-kernel-supplier/       — Supplier, SupplierQualification, SupplierAudit, IngredientSpec entities
crates/oya-vertical-food-kernel-traceability/   — FoodLot, EpcisEventLink, TraceabilityChain entities (links to Agriculture EPCIS)
crates/oya-vertical-food-kernel-label/          — ProductSpec, NutritionFacts, AllergenDeclaration, LabelVersion entities
crates/oya-vertical-food-kernel-recall/         — RecallEvent, AffectedLot, RegulatoryNotification entities
crates/oya-vertical-food-domain-*/              — Use cases per sub-domain
crates/oya-vertical-food-app-*/                 — Sagas + Foundry delegation
crates/oya-vertical-food-adapter-*/             — DB, GS1 EPCIS, cold-chain event adapters
crates/oya-vertical-food-api-rest/              — REST API
crates/oya-vertical-food-api-traceability/      — Consumer-facing traceability query (recall portal)
crates/oya-vertical-food-runtime/               — Composition root
```

### 4.3 External-Facing Surfaces

| Surface | Contract location | Plane | SLO target |
|---|---|---|---|
| Food Safety REST API | `contracts/food-core.openapi.yaml` | Data | 99.9% / p95 < 300ms |
| Supplier portal API | `contracts/food-supplier.openapi.yaml` | Data | 99.5% / p95 < 500ms |
| Recall / traceability public API | `contracts/food-recall.openapi.yaml` | Data | 99.9% / p95 < 1s (recall is time-critical) |
| GS1 EPCIS 2.0 REST API | `contracts/food-epcis.openapi.yaml` | Data | 99.5% / p95 < 500ms |

### 4.4 Internal Seams

| Seam | Trait | Consumer products |
|---|---|---|
| `FoodLotTraceabilityConsumer` | `TraceabilityChainConsumer` | Agriculture vertical (upstream lot records); Logistics vertical (cold-chain events) |
| `FoodCostGlPostable` | `GlCostPostable` | Corporate GL (COGS, ingredient cost) |
| `RecallAlertEmitter` | `AuditChainEmitter` | Audit chain (recall notification mandatory); Logistics (shipment hold) |

---

## 5. Data Structures

### 5.1 Kernel Entities

```rust
// crates/oya-vertical-food-kernel-haccp
/// HACCP CCP monitoring log — FDA 21 CFR Part 120/123 / CODEX HACCP aligned
/// data_class: BEHAVIORAL_TENANT_PRODUCT
/// plane: data
pub struct CcpLog {
    pub id: CcpLogId,
    pub tenant_id: TenantId,
    pub haccp_plan_id: HaccpPlanId,
    pub ccp_id: CcpId,
    pub production_run_id: Option<ProductionRunId>,
    pub region: RegionCode,
    pub schema_version: u32,
    pub monitored_at: DateTime<Utc>,
    pub monitored_by: UserId,
    pub measured_value: Decimal,         // data_class: BEHAVIORAL_TENANT_PRODUCT (e.g., temperature °C, pH)
    pub measurement_uom: UnitOfMeasure,
    pub critical_limit_min: Option<Decimal>,
    pub critical_limit_max: Option<Decimal>,
    pub in_control: bool,               // measured_value within critical limits
    pub corrective_action_id: Option<CorrectiveActionId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct CriticalControlPoint {
    pub id: CcpId,
    pub tenant_id: TenantId,
    pub haccp_plan_id: HaccpPlanId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub step_name: String,               // process step (e.g., "Pasteurization")
    pub hazard_type: HazardType,
    pub critical_limit_description: String,
    pub monitoring_method: String,
    pub monitoring_frequency: String,
    pub corrective_action_procedure: String,
    pub verification_procedure: String,
    pub record_keeping: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
pub enum HazardType { Biological, Chemical, Physical, Radiological }

// crates/oya-vertical-food-kernel-traceability
/// Food lot — FSMA 204 / EU 178/2002 / KR 이력추적관리 aligned
/// data_class: BEHAVIORAL_TENANT_PRODUCT
pub struct FoodLot {
    pub id: FoodLotId,
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub lot_number: String,
    pub product_id: ProductId,
    pub production_date: NaiveDate,
    pub best_before: Option<NaiveDate>,
    pub use_by: Option<NaiveDate>,
    pub quantity_produced: Decimal,
    pub quantity_uom: UnitOfMeasure,
    pub ingredient_lot_links: Vec<FoodLotLink>,  // upstream lot references (Agriculture/Supplier)
    pub epcis_event_ids: Vec<EpcisEventId>,
    pub cold_chain_shipment_ids: Vec<ShipmentId>, // Logistics vertical cold-chain refs
    pub recall_status: RecallStatus,
    pub traceable_kyc: TraceableKyc,              // KR 이력추적관리번호
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
pub enum RecallStatus { None, UnderInvestigation, Recalled }

pub struct FoodLotLink {
    pub upstream_lot_id: String,          // may reference Agriculture vertical lot or supplier lot
    pub upstream_source: UpstreamSource,
    pub ingredient_name: String,
    pub quantity_used: Decimal,
    pub quantity_uom: UnitOfMeasure,
}
pub enum UpstreamSource { AgricultureVertical, ExternalSupplier, ManufacturingLot }

// crates/oya-vertical-food-kernel-recall
/// data_class: BEHAVIORAL_TENANT_PRODUCT (lot info); PII_IDENTIFYING (consumer contact if voluntary recall)
pub struct RecallEvent {
    pub id: RecallEventId,
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub recall_class: RecallClass,        // FDA Class I/II/III or KR 회수등급
    pub reason: String,
    pub affected_lots: Vec<FoodLotId>,
    pub status: RecallStatus2,
    pub initiated_at: DateTime<Utc>,
    pub regulatory_notifications: Vec<RegulatoryNotification>,
    pub public_announcement_at: Option<DateTime<Utc>>,
    pub foundry_run_id: Option<FoundryRunId>, // Foundry root-cause analysis draft
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
pub enum RecallClass { ClassI, ClassII, ClassIII } // FDA classification
pub enum RecallStatus2 { Initiated, Active, Completed, Terminated }

// crates/oya-vertical-food-kernel-label
/// data_class: BEHAVIORAL_TENANT_PRODUCT
pub struct AllergenDeclaration {
    pub id: AllergenDeclarationId,
    pub tenant_id: TenantId,
    pub product_id: ProductId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub allergens_present: Vec<AllergenCode>, // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub allergens_may_contain: Vec<AllergenCode>,
    pub declaration_basis: AllergenDeclarationBasis,
    pub valid_from: NaiveDate,
    pub regulatory_standard: AllergenStandard, // EU 1169/2011, US FALCPA/FASTER, KR 식품표시기준
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
pub enum AllergenCode {
    Gluten, Crustaceans, Eggs, Fish, Peanuts, Soybeans,
    Milk, Nuts, Celery, Mustard, Sesame, Sulphites, Lupin, Molluscs,
    // KR additions:
    Wheat, Buckwheat, Beef, Chicken, Pork, Peach, Tomato, Shrimp, Crab, Squid, Mackerel, Clam, Oyster, Abalone, Mussel,
}
pub enum AllergenStandard { Eu11692011, UsFalcpaFaster, KrFoodLabelingStandard, CodexAlimentarius }
```

> TODO v0.2 — vertical owner to add `HaccpPlan`, `HazardAnalysis`, `Supplier`, `SupplierQualification`, `NutritionFacts`, `LabelVersion`, `CorrectiveAction` entities with full fields.

### 5.2–5.7

> TODO v0.2 — vertical owner to expand aggregate boundaries, persistence layout, event schemas, audit-chain contract, migration policy.

Key audit events: `CcpDeviationDetected`, `CorrectiveActionTaken`, `RecallInitiated`, `SupplierDisqualified`, `LotTraceabilityChainComplete`.

**Mandatory audit events**: every recall notification emits an immutable audit record with regulatory authority, notified party, lot list, and timestamp.

---

## 6. Optimization Practices

| Practice | Implementation choice |
|---|---|
| Cell routing | `(tenant_id, production_site_id)` → cell |
| Sharding strategy | Per-site shard for HACCP/CCP logs; per-tenant for supplier and recall records |
| Caching tier | Redis for allergen matrix (product × ingredient × allergen — high-read in production scheduling); in-memory for HACCP plan templates |
| Bulk endpoint contract | `POST /ccp-logs/bulk` (automated CCP monitoring sensor batch upload); `POST /food-lots/bulk-trace` (EPCIS chain bulk query) |
| Agent-driven optimization | Foundry `SupplierRiskScorer` (supplier qualification risk assessment — T1, QA manager reviews); Foundry `RecallRootCauseAnalyzer` (root-cause hypothesis from HACCP + lot trace — T1); Foundry `AllergenConflictDetector` (production schedule allergen cross-contact alert — T1) |

> TODO v0.2 — vertical owner to expand.

---

## 7. Regional Pack Interactions

| Seam | Trait | Per-pack impl needed? | Tested with |
|---|---|---|---|
| HACCP regulatory standard | `HaccpStandardExtension` | Yes | `oya-pack-kr` (식품위생법 HACCP 기준, 식약처), `oya-pack-us` (FDA FSMA HARPC / 21 CFR Part 120/123, USDA FSIS), `oya-pack-eu` (EU Reg 852/2004, EFSA) |
| Food traceability regulation | `TraceabilityRegulatoryExtension` | Yes | `oya-pack-kr` (이력추적관리 — 식품이력추적관리시스템), `oya-pack-us` (FDA FSMA 204 KDE requirements), `oya-pack-eu` (EU Food Law 178/2002 one-step-up one-step-down) |
| Allergen declaration standard | `AllergenStandardExtension` | Yes | `oya-pack-kr` (식품표시기준 — 23 major allergens), `oya-pack-us` (FALCPA + FASTER Act — 9 major allergens), `oya-pack-eu` (EU 1169/2011 — 14 major allergens) |
| Recall notification authority | `RecallNotificationAdapter` | Yes | `oya-pack-kr` (식품의약품안전처 식품이력추적), `oya-pack-us` (FDA CFSAN recall portal), `oya-pack-eu` (RASFF — Rapid Alert System for Food and Feed) |
| Regulatory control evidence | `RegulatoryPack` | Yes | `oya-pack-kr` (식약처, HACCP 인증원), `oya-pack-us` (FDA, USDA FSIS), `oya-pack-eu` (EFSA, EU RASFF) |

### Regulatory Pack Declaration

```yaml
regulatory_packs:
  - oya-pack-kr   # 식품위생법, 식품이력추적관리, HACCP 인증원, 식약처, PIPA
  - oya-pack-us   # FDA FSMA 204, HARPC, USDA FSIS, FALCPA/FASTER
  - oya-pack-eu   # EU Reg 852/2004, EU Food Law 178/2002, EU 1169/2011, RASFF, EFSA
  - oya-pack-au   # Food Standards Australia New Zealand (FSANZ), HACCP AU
```

---

## 8. In-House vs External Dependency Posture

> TODO v0.2 — vertical owner to expand. Key: `tokio`/`axum`/`sqlx`/`serde`/`rustls` (kernel-grade); GS1 EPCIS 2.0 in-house (shared kernel with Agriculture vertical via `oya-vertical-agriculture-kernel-traceability`); allergen matrix computation in-house; nutrition facts calculation in-house.

---

## 9. Success Metrics

| Metric | Vertical-Preview target | Vertical-Stable target | Public-GA target |
|---|---|---|---|
| Food facilities under HACCP management | ≥ 1 (design partner food manufacturer) | ≥ 50 | ≥ 1,000 |
| CCP monitoring log completeness | 100% | 100% | 100% |
| Lot traceability chain completeness (factory → retail) | ≥ 80% | ≥ 95% | ≥ 99% |
| Recall lot identification time | < 4 hours | < 1 hour | < 15 min |
| Allergen conflict detection rate | ≥ 95% | ≥ 99% | ≥ 99.9% |
| Audit chain completeness (HACCP deviations + recall) | 100% | 100% | 100% |
| Foundry supplier risk scoring adoption | ≥ 20% of supplier reviews | ≥ 70% | ≥ 90% |

---

## 10. Risks + Mitigations

| Risk | Severity | Mitigation | Owner |
|---|---|---|---|
| CCP deviation not detected (food safety failure reaches consumer) | Catastrophic | Real-time CCP log monitoring with IoT sensor ingest; automated corrective action trigger on deviation; mandatory audit record per CCP check | Food safety domain + SRE |
| Allergen undeclared (consumer anaphylaxis risk) | Catastrophic | Allergen matrix enforced at production scheduling; label allergen declaration cross-validated against ingredient BOM; allergen conflict alert blocks production dispatch without override | Food safety domain |
| Recall lot identification failure (incomplete traceability chain) | Critical | FSMA 204 KDE requirements checked at lot creation; missing upstream lot link is a blocking validation error | Traceability domain |
| RASFF notification not filed in time (EU 178/2002 72-hour requirement) | Critical | Recall event triggers automatic 72-hour countdown alert; RASFF adapter in EU pack files notification; audit record per filing | EU pack + Recall domain |
| Cold-chain temperature excursion linked to HACCP deviation | High | Cold-chain events from Logistics vertical auto-linked to FoodLot; temperature deviation flagged in HACCP CCP log | Logistics seam + Food safety |
| Supplier adulterant not detected by qualification process | High | Foundry `SupplierRiskScorer` uses adverse event database + audit history; manual audit required for Class I risk suppliers regardless of score | QA domain + Foundry |

> TODO v0.2 — vertical owner to expand.

---

## 11. Open Questions

- Shared GS1 EPCIS kernel with Agriculture vertical: one shared crate (`oya-vertical-agriculture-kernel-traceability`) or a platform-level `oya-platform-epcis-kernel`? (Current plan: Agriculture owns; Food depends — review at Vertical-Stable.)
- Nutrition facts engine: build in-house or license USDA FoodData Central data + calculation library?
- USDA FSIS meat/poultry HACCP (9 CFR Part 417) — is this a separate regulatory pack extension or base US-pack coverage?
- KR 축산물이력제 (livestock traceability) — handled in Agriculture or Food vertical?

---

## 12. Decision Log

| Decision | Date | Rationale | ADR ref |
|---|---|---|---|
| HACCP plan in kernel (not external template vendor) | 2026-05-09 | HACCP is a regulated plan; in-house ensures tamper-evidence and audit completeness | — |
| Allergen matrix in-house computation | 2026-05-09 | Allergen underdeclaration is a product-liability risk; in-house with full data lineage | — |
| Foundry root-cause analysis at T1 | 2026-05-09 | Food safety corrective actions have regulatory consequence; food safety manager must decide | ADR-0050 |
| Recall records immutable after initiation | 2026-05-09 | FDA + EU + KR regulations prohibit alteration of recall records | — |
| Flat-crates: `crates/oya-vertical-food-*` | 2026-05-09 | Per ADR-0015 | ADR-0015 |

---

## 13. Sources Scanned

- `docs/PRD.md`, `docs/DESIGN.md` §1, §4, §12
- `docs/PRIVACY-PROGRAM.md` §2.2.3
- Codex HACCP 1969 rev. 2020; FDA FSMA 204 (food traceability); EU Food Law 178/2002; KR 식품위생법; EU 1169/2011 (allergen labeling); GS1 EPCIS 2.0 standard

---

## Doc-Catalog Row

```
| `vertical-food` | `vertical-2` | supply-chain-compliance/HACCP/traceability/allergen/recall | monthly | PRD.md, DESIGN.md §12, PRIVACY-PROGRAM.md §2.2.3 |
```
