# Oyatie — Product PRD: Vertical Construction

> **Status:** preview (skeleton)
> **Owning team:** [`teams/vertical-construction/CHARTER.md`](../../teams/vertical-construction/CHARTER.md)
> **Owning axis:** vertical-construction (Axis 2)
> **Catalog reference:** `registry/catalog/oya-vertical-construction-*.yaml`
> **Last updated:** 2026-05-09 by architecture-council

---

## 1. North Star

Oyatie Vertical Construction is the project management and document control platform for general contractors, subcontractors, and owners in the construction and engineering (AEC — Architecture, Engineering, Construction) industry. It covers project schedule management, RFI (Request for Information) workflow, submittal management, change order management, punch list, and daily field reports. It exists within Oyatie's ecosystem because the coupling of project cost control with the Corporate vertical's GL (budget vs. actual), Foundry-driven schedule risk analysis and document classification, the audit chain for KR 건설업 reporting (건설CALS) and global ISO 19650 BIM data management, and Search-powered drawing and specification search creates the integrated construction operations stack that standalone project management SaaS (Procore-analogue) cannot offer with the same financial and compliance depth.

---

## 2. Target Users

| Persona | What they get | What they pay for |
|---|---|---|
| Project Manager | Schedule management, budget tracking, RFI/submittal dashboard, daily field report | Per-seat (PM tier) |
| Project Engineer | RFI creation and response, submittal review, drawing markup, specification search | Per-seat (engineering tier) |
| Site Supervisor | Daily field report, punch list, progress photos, workforce log | Per-seat (field tier) |
| Owner / Client | Project dashboard, cost reporting, contract milestone tracking | Per-seat (owner tier) |
| Subcontractor | RFI response, submittal submission, change order review, punch list close-out | Per-seat (sub tier; metered per project) |
| Construction IT / Tenant Builder | Project template configuration, BIM integration config, Foundry risk-analysis workflow | Builder seat |

---

## 3. In-Scope / Out-of-Scope

### 3.1 In-scope at each wave

| Wave | Capabilities | Surfaces exposed |
|---|---|---|
| Vertical-Preview | Project setup and schedule (Gantt / CPM baseline), RFI creation and response workflow, submittal log and review, daily field report, KR 건설CALS 기성실적 | REST API v1, Web UI, Mobile (field) |
| Vertical-Stable | Change order management (potential change order → RFQ → CO → contract modification), punch list, budget vs. actual (GL integration — Corporate), Foundry schedule risk analysis (float analysis, critical path alert — T1), drawing and specification search (Search axis tenant-private), BIM model reference linking (IFC file metadata), subcontractor portal | REST API stable, Subcontractor portal, Webhook console |
| Public-GA | ISO 19650 BIM data management (CDE — Common Data Environment), AI-assisted RFI response drafting (Foundry, T1), carbon footprint per project (Scope 3 embodied carbon), government compliance reporting (KR 전자조달 건설, US Davis-Bacon Act) | Public OpenAPI, Analytics dashboard |

### 3.2 Out-of-scope (anti-scope)

- Full-featured BIM authoring (Revit / ArchiCAD replacement) — BIM reference and metadata only; authoring is in Autodesk/Bentley
- Structural engineering calculation and analysis
- Construction equipment telematics at machine-control depth (OPC UA read for equipment status is the seam; machine control is not)
- Advertising using project or contractor data — `BEHAVIORAL_TENANT_PRODUCT` not ad-targetable per PRIVACY-PROGRAM §2.2.3

---

## 4. Architecture Overview

### 4.1 Bounded Context

Flat-crates target prefix: `crates/oya-vertical-construction-*`.

```
crates/oya-vertical-construction-kernel-project/    — Project, Schedule, Activity, Milestone entities
crates/oya-vertical-construction-kernel-rfi/        — RFI, RFIResponse, RFIStatus entities
crates/oya-vertical-construction-kernel-submittal/  — Submittal, SubmittalItem, ReviewCycle entities
crates/oya-vertical-construction-kernel-change/     — ChangeOrder, PotentialChangeOrder, ContractModification entities
crates/oya-vertical-construction-kernel-field/      — DailyReport, PunchListItem, ProgressPhoto entities
crates/oya-vertical-construction-domain-*/          — Use cases per sub-domain
crates/oya-vertical-construction-app-*/             — Sagas + Foundry delegation
crates/oya-vertical-construction-adapter-*/         — DB, BIM/IFC, drawing viewer adapters
crates/oya-vertical-construction-api-rest/          — REST API
crates/oya-vertical-construction-runtime/           — Composition root
```

### 4.3 External-Facing Surfaces

| Surface | Contract location | Plane | SLO target |
|---|---|---|---|
| Construction REST API | `contracts/construction-core.openapi.yaml` | Data | 99.9% / p95 < 300ms |
| Subcontractor portal API | `contracts/construction-sub.openapi.yaml` | Data | 99.5% / p95 < 500ms |
| Drawing / document viewer | internal (object store presigned URL) | Data | 99.5% / p95 < 2s (large drawing retrieval) |

### 4.4 Internal Seams

| Seam | Trait | Consumer products |
|---|---|---|
| `ProjectBudgetGlPostable` | `GlCostPostable` | Corporate GL (project cost accounting, WBS cost codes) |
| `DrawingSearchIndexable` | `SearchIndexable` (tenant-private) | Search axis (drawing number, spec section search) |

---

## 5. Data Structures

### 5.1 Kernel Entities

```rust
// crates/oya-vertical-construction-kernel-rfi
/// data_class: BEHAVIORAL_TENANT_PRODUCT
/// plane: data
pub struct Rfi {
    pub id: RfiId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub number: u32,                          // sequential per project
    pub title: String,                        // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub description: String,                  // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub status: RfiStatus,
    pub priority: RfiPriority,
    pub created_by: UserId,
    pub assigned_to: Option<UserId>,
    pub due_date: Option<NaiveDate>,
    pub drawing_refs: Vec<DrawingRef>,
    pub spec_refs: Vec<SpecRef>,
    pub responses: Vec<RfiResponse>,
    pub cost_impact: Option<Money>,           // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub schedule_impact_days: Option<i32>,
    pub foundry_draft_id: Option<FoundryRunId>, // Foundry-assisted response draft (T1)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
pub enum RfiStatus { Draft, Open, Answered, Closed, Void }
pub enum RfiPriority { Low, Normal, High, Critical }

// crates/oya-vertical-construction-kernel-submittal
/// data_class: BEHAVIORAL_TENANT_PRODUCT
pub struct Submittal {
    pub id: SubmittalId,
    pub tenant_id: TenantId,
    pub project_id: ProjectId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub number: String,                       // e.g., "03-3000.01-A"
    pub title: String,
    pub spec_section: String,
    pub status: SubmittalStatus,
    pub revision: u32,
    pub submitted_by: UserId,
    pub reviewed_by: Option<UserId>,
    pub review_deadline: Option<NaiveDate>,
    pub documents: Vec<SubmittalDocRef>,
    pub review_result: Option<SubmittalReviewResult>,
    pub comments: Vec<SubmittalComment>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
pub enum SubmittalStatus { Pending, Submitted, UnderReview, Approved, ApprovedAsNoted, Rejected, Revise }
pub enum SubmittalReviewResult { Approved, ApprovedAsNoted, ReviseAndResubmit, Rejected }

// crates/oya-vertical-construction-kernel-project
/// data_class: BEHAVIORAL_TENANT_PRODUCT
pub struct Project {
    pub id: ProjectId,
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub name: String,
    pub project_number: String,
    pub status: ProjectStatus,
    pub project_type: ProjectType,
    pub owner_ref: PartyRef,
    pub gc_ref: PartyRef,
    pub contract_value: Option<Money>,
    pub start_date: Option<NaiveDate>,
    pub substantial_completion: Option<NaiveDate>,
    pub address: Option<Address>,
    pub gl_project_code: Option<String>,     // links to Corporate GL project cost code
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
pub enum ProjectStatus { Preconstruction, Active, SubstantiallyComplete, CloseOut, Complete, OnHold }
pub enum ProjectType { Commercial, Residential, Infrastructure, Industrial, Healthcare, Education }
```

> TODO v0.2 — vertical owner to add `Schedule`, `Activity`, `ChangeOrder`, `DailyReport`, `PunchListItem` entities with full fields.

### 5.2–5.7

> TODO v0.2 — vertical owner to expand.

Key audit events: `RfiAnswered`, `SubmittalApproved`, `ChangeOrderExecuted`, `PunchListClosed`.

---

## 6. Optimization Practices

| Practice | Implementation choice |
|---|---|
| Cell routing | `(tenant_id, project_id)` → cell |
| Sharding strategy | Per-project shard (projects are natural isolation boundaries) |
| Caching tier | Redis for RFI/submittal log state; in-memory for drawing index (high-read); large drawing files served from Object Store via presigned URL |
| Bulk endpoint contract | `POST /rfis/bulk-import`; `POST /submittals/bulk-import` (migration from legacy systems) |
| Agent-driven optimization | Foundry `ScheduleRiskAnalyzer` (float analysis, critical path, T1); Foundry `RfiResponseDrafter` (T1, engineer reviews); Foundry `BudgetVarianceAlert` (real-time budget vs. actual monitoring) |

> TODO v0.2 — vertical owner to expand.

---

## 7. Regional Pack Interactions

| Seam | Trait | Per-pack impl needed? | Tested with |
|---|---|---|---|
| Government construction reporting | `ConstructionReportAdapter` | Yes | `oya-pack-kr` (건설CALS, 국토교통부 건설공사정보관리시스템), `oya-pack-us` (AIA G702/G703 pay application, Davis-Bacon Act compliance) |
| BIM standard | `BimDataModelExtension` | Yes | `oya-pack-kr` (국토교통부 BIM 가이드라인 2.0), `oya-pack-eu` (ISO 19650 + EU BIM mandate) |
| Regulatory control evidence | `RegulatoryPack` | Yes | `oya-pack-kr` (건설업 면허, 안전보건공단, 건축법) |

### Regulatory Pack Declaration

```yaml
regulatory_packs:
  - oya-pack-kr   # 건설업 면허관리, 건설CALS, 안전보건관리법, 건축법
  - oya-pack-us   # OSHA 1926 (construction safety), Davis-Bacon, AIA contracts
  - oya-pack-eu   # EU BIM mandate, ISO 19650, CE marking for construction products
```

---

## 8. In-House vs External Dependency Posture

> TODO v0.2 — vertical owner to expand. Key: `tokio`/`axum`/`sqlx`/`serde`/`rustls` (kernel-grade); IFC/BIM parsing via `ifc-rs` or in-house (evaluation pending); drawing viewer via browser-native PDF.js (Apache-2).

---

## 9. Success Metrics

| Metric | Vertical-Preview target | Vertical-Stable target | Public-GA target |
|---|---|---|---|
| Active projects | ≥ 5 (design partner) | ≥ 500 | ≥ 10,000 |
| RFI response cycle time (median) | < 10 days | < 7 days | < 5 days |
| Submittal review cycle time (median) | < 14 days | < 10 days | < 7 days |
| Budget vs actual accuracy | baseline | ±2% | ±1% |
| Audit chain completeness | 100% | 100% | 100% |

---

## 10. Risks + Mitigations

| Risk | Severity | Mitigation | Owner |
|---|---|---|---|
| Unanswered RFI causing project delay | High | Automated escalation at 50% of due-date; Foundry `ScheduleRiskAnalyzer` flags float-impacting open RFIs | Construction domain |
| Drawing file version confusion (wrong revision issued) | High | Immutable drawing revision records; superseded revisions archived not deleted; version watermark on PDF export | Construction domain |
| Budget overrun not surfaced in time | High | Real-time GL integration (Corporate vertical); Foundry `BudgetVarianceAlert` triggers at 80% committed | Construction + Corporate |
| KR 건설CALS filing deadline missed | Medium | Regulatory-change watch lane; automated filing trigger on milestone events | KR pack + Construction |

> TODO v0.2 — vertical owner to expand.

---

## 11. Open Questions

- BIM authoring integration (Autodesk BIM 360 / Procore API): direct sync or import-only?
- Change order cost impact: does it auto-propose GL journal (via Corporate domain) or require manual CFO approval?
- Subcontractor payment application (AIA G702): in-scope for Stable or separate vertical seam?

---

## 12. Decision Log

| Decision | Date | Rationale | ADR ref |
|---|---|---|---|
| Foundry schedule analysis at T1 | 2026-05-09 | Schedule changes have contractual and financial implications; PM must approve | ADR-0050 |
| Per-project shard strategy | 2026-05-09 | Projects are natural isolation boundaries; cross-project queries are analytics-only | — |
| Flat-crates: `crates/oya-vertical-construction-*` | 2026-05-09 | Per ADR-0015 | ADR-0015 |

---

## 13. Sources Scanned

- `docs/PRD.md`, `docs/DESIGN.md` §1, §4, §12
- `docs/PRIVACY-PROGRAM.md` §2.2.3

---

## Doc-Catalog Row

```
| `vertical-construction` | `vertical-2` | project-mgmt/RFI/submittal/change-order/건설CALS | monthly | PRD.md, DESIGN.md §12 |
```
