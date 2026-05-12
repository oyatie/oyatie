# Oyatie — Product PRD: Vertical Real Estate

> **Status:** preview (skeleton)
> **Owning team:** [`teams/vertical-real-estate/CHARTER.md`](../../teams/vertical-real-estate/CHARTER.md)
> **Owning axis:** vertical-real-estate (Axis 2)
> **Catalog reference:** `registry/catalog/oya-vertical-real-estate-*.yaml`
> **Last updated:** 2026-05-09 by architecture-council

---

## 1. North Star

Oyatie Vertical Real Estate is the property leasing and asset management platform for commercial real estate owners, REITs, property managers, and tenants. It covers the full leasing lifecycle (lease origination, CAM reconciliation, lease abstraction, renewal management), asset management (property portfolio tracking, capital expenditure planning, valuation), and facilities management (work order, preventive maintenance). It exists within Oyatie's ecosystem because the coupling of lease accounting (IFRS 16 / ASC 842 right-of-use asset computation) with the Corporate vertical's GL, Foundry-driven lease abstraction and renewal risk analysis, and the audit chain for regulatory reporting (KR 부동산투자신탁 / REITS, US SEC REIT reporting, GRESB ESG) creates the integrated real estate operations platform that no standalone REMS (Real Estate Management System) or lease accounting tool can replicate.

---

## 2. Target Users

| Persona | What they get | What they pay for |
|---|---|---|
| Asset Manager / Portfolio Manager | Property portfolio dashboard, NOI tracking, valuation, capex planning | Per-seat (asset mgmt tier) |
| Leasing Manager | Lease pipeline, lease abstraction (Foundry-assisted), CAM reconciliation, renewal management | Per-seat (leasing tier) |
| Property Accountant | IFRS 16 / ASC 842 lease schedules, GL posting (Corporate), CAM billing | Per-seat (accounting tier) |
| Facilities Manager | Preventive maintenance schedule, work order dispatch, vendor management | Per-seat (FM tier) |
| Tenant (Lessee) | Tenant portal — lease documents, CAM statements, maintenance requests, payment history | Per-seat (tenant portal — paid by landlord) |
| Real Estate IT / Tenant Builder | Lease template configuration, IFRS 16 engine config, Foundry lease-analysis workflow | Builder seat |

---

## 3. In-Scope / Out-of-Scope

### 3.1 In-scope at each wave

| Wave | Capabilities | Surfaces exposed |
|---|---|---|
| Vertical-Preview | Lease record management (commercial leases — office/retail/industrial), basic CAM reconciliation, property portfolio dashboard, KR 임대차 계약 관리 (부동산 임대계약) | REST API v1, Web UI, Tenant portal |
| Vertical-Stable | IFRS 16 / ASC 842 right-of-use asset and liability schedule computation, lease abstraction via Foundry (T1 — property accountant reviews), renewal risk scoring (Foundry), capex planning and approval workflow, preventive maintenance schedule, vendor work order management, GL integration (Corporate), CAM billing with tenant portal statements, KR 상가임대차보호법 compliance alerts | REST API stable, Tenant portal, Webhook console |
| Public-GA | GRESB ESG scoring integration, AI-assisted portfolio optimization (Foundry), cross-portfolio benchmark analytics, KR 부동산투자신탁 regulatory reporting, US SEC REIT regulatory filing support | Public OpenAPI, Analytics dashboard |

### 3.2 Out-of-scope (anti-scope)

- Residential property management at consumer depth (focus is commercial real estate; residential SMB is a potential future extension)
- Property valuation modeling (Foundry can surface comparable data; CBRE/JLL appraisal model replacement is not in-scope)
- Mortgage origination and underwriting
- Advertising targeting using tenant or lease data — `BEHAVIORAL_TENANT_PRODUCT`; PRIVACY-PROGRAM §2.2.3 corporate default blocks ads

---

## 4. Architecture Overview

### 4.1 Bounded Context

Flat-crates target prefix: `crates/oya-vertical-real-estate-*`.

```
crates/oya-vertical-real-estate-kernel-lease/     — Lease, LeaseClause, CamReconciliation, LeasePayment entities
crates/oya-vertical-real-estate-kernel-asset/     — Property, Unit, Portfolio, ValuationRecord, CapexPlan entities
crates/oya-vertical-real-estate-kernel-ifrs16/    — Rou Asset, LeaseLiability, LeasePaymentSchedule entities (IFRS 16 / ASC 842)
crates/oya-vertical-real-estate-kernel-fm/        — WorkOrder, PreventiveMaintenance, Vendor, Inspection entities
crates/oya-vertical-real-estate-domain-*/         — Use cases per sub-domain
crates/oya-vertical-real-estate-app-*/            — Sagas + Foundry delegation
crates/oya-vertical-real-estate-adapter-*/        — DB, document store adapters
crates/oya-vertical-real-estate-api-rest/         — REST API
crates/oya-vertical-real-estate-runtime/          — Composition root
```

### 4.3 External-Facing Surfaces

| Surface | Contract location | Plane | SLO target |
|---|---|---|---|
| Real Estate REST API | `contracts/real-estate-core.openapi.yaml` | Data | 99.9% / p95 < 300ms |
| Tenant portal API | `contracts/real-estate-tenant.openapi.yaml` | Data | 99.5% / p95 < 500ms |

### 4.4 Internal Seams

| Seam | Trait | Consumer products |
|---|---|---|
| `Ifrs16GlPostable` | `GlCostPostable` | Corporate GL (ROU asset amortization, lease liability unwinding) |
| `LeaseSearchIndexable` | `SearchIndexable` (tenant-private) | Search axis (lease document search) |

---

## 5. Data Structures

### 5.1 Kernel Entities

```rust
// crates/oya-vertical-real-estate-kernel-lease
/// data_class: BEHAVIORAL_TENANT_PRODUCT (lease terms); PII_IDENTIFYING (lessee contact)
/// plane: data
pub struct Lease {
    pub id: LeaseId,
    pub tenant_id: TenantId,             // landlord organization
    pub property_id: PropertyId,
    pub unit_id: UnitId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub lessee_name: String,             // data_class: PII_IDENTIFYING (if individual) or BEHAVIORAL_TENANT_PRODUCT (if corp)
    pub lessee_contact: Option<ContactInfo>, // data_class: PII_IDENTIFYING
    pub lease_type: LeaseType,
    pub status: LeaseStatus,
    pub commencement_date: NaiveDate,    // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub expiration_date: NaiveDate,      // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub base_rent: Money,                // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub rent_escalation: Option<RentEscalation>,
    pub cam_basis: Option<CamBasis>,
    pub renewal_options: Vec<RenewalOption>,
    pub lease_document_ref: Option<DocumentRef>,
    pub ifrs16_schedule_id: Option<Ifrs16ScheduleId>,
    pub foundry_abstraction_run_id: Option<FoundryRunId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
pub enum LeaseType { GrossLease, NetLease, NnnLease, ModifiedGross, PercentageLease }
pub enum LeaseStatus { Prospect, Active, InRenewal, Expired, Terminated }

// crates/oya-vertical-real-estate-kernel-ifrs16
/// IFRS 16 / ASC 842 Right-of-Use Asset and Lease Liability
/// data_class: BEHAVIORAL_TENANT_PRODUCT
pub struct Ifrs16Schedule {
    pub id: Ifrs16ScheduleId,
    pub tenant_id: TenantId,
    pub lease_id: LeaseId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub accounting_standard: AccountingStandard, // Ifrs16 or Asc842
    pub commencement_date: NaiveDate,
    pub lease_term_months: u32,
    pub discount_rate: Decimal,          // incremental borrowing rate
    pub rou_asset_initial: Money,
    pub lease_liability_initial: Money,
    pub payment_schedule: Vec<Ifrs16PaymentLine>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
pub enum AccountingStandard { Ifrs16, Asc842 }

// crates/oya-vertical-real-estate-kernel-asset
/// data_class: BEHAVIORAL_TENANT_PRODUCT
pub struct Property {
    pub id: PropertyId,
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub name: String,
    pub property_type: PropertyType,
    pub address: Address,               // data_class: BEHAVIORAL_TENANT_PRODUCT (commercial address)
    pub total_area_sqm: Decimal,
    pub units: Vec<UnitRef>,
    pub portfolio_id: Option<PortfolioId>,
    pub acquisition_date: Option<NaiveDate>,
    pub acquisition_cost: Option<Money>,
    pub book_value: Option<Money>,
    pub latest_valuation: Option<ValuationRecord>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
pub enum PropertyType { Office, Retail, Industrial, Residential, Mixed, Land, DataCenter }
```

> TODO v0.2 — vertical owner to add `CamReconciliation`, `CapexPlan`, `WorkOrder`, `PreventiveMaintenance`, `Vendor` entities with full fields.

### 5.2–5.7

> TODO v0.2 — vertical owner to expand aggregate boundaries, persistence layout, event schemas, audit-chain contract, migration policy.

Key audit events: `LeaseExecuted`, `CamStatementIssued`, `Ifrs16SchedulePosted`, `ValuationUpdated`.

---

## 6. Optimization Practices

| Practice | Implementation choice |
|---|---|
| Cell routing | `(tenant_id, portfolio_id)` → cell |
| Sharding strategy | Per-portfolio shard for large REITs; per-tenant for smaller landlords |
| Caching tier | Redis for active lease CAM accrual state; in-memory for property unit availability |
| Bulk endpoint contract | `POST /leases/bulk-import`; `POST /ifrs16/recalculate/bulk` (portfolio-wide recompute on rate change) |
| Agent-driven optimization | Foundry `LeaseAbstractor` (clause extraction from PDF lease — T1, accountant reviews); Foundry `RenewalRiskAnalyzer` (expiry pipeline scoring — T1); Foundry `Ifrs16Recomputer` (triggered on rate change — T1, accountant approves GL posting) |

> TODO v0.2 — vertical owner to expand.

---

## 7. Regional Pack Interactions

| Seam | Trait | Per-pack impl needed? | Tested with |
|---|---|---|---|
| Lease law compliance | `LeaseLawExtension` | Yes | `oya-pack-kr` (상가임대차보호법, 주택임대차보호법, 부동산 실거래가 신고), `oya-pack-us` (state commercial lease law, ASC 842), `oya-pack-eu` (IFRS 16, EU commercial tenancy per country) |
| REIT regulatory reporting | `RegulatoryPack` | Yes | `oya-pack-kr` (부동산투자신탁법, 금융위원회), `oya-pack-us` (SEC REIT reporting, IRS Rev. Proc.) |
| Tax-invoice (rent payment) | `TaxInvoiceFormatter` | Yes | `oya-pack-kr` (부동산 임대업 부가가치세), `oya-pack-eu` (VAT on commercial rent) |

### Regulatory Pack Declaration

```yaml
regulatory_packs:
  - oya-pack-kr   # 상가임대차보호법, 부동산투자신탁, 부동산 실거래가 신고, PIPA
  - oya-pack-us   # ASC 842, SEC REIT, IRS, state commercial lease laws
  - oya-pack-eu   # IFRS 16, GRESB, EU commercial tenancy directives
```

---

## 8. In-House vs External Dependency Posture

> TODO v0.2 — vertical owner to expand. Key: `tokio`/`axum`/`sqlx`/`serde`/`rustls` (kernel-grade); IFRS 16 schedule computation in-house (`oya-vertical-real-estate-kernel-ifrs16`); PDF lease parsing via Foundry capability (not kernel-level dep); `rust_decimal` for monetary arithmetic.

---

## 9. Success Metrics

| Metric | Vertical-Preview target | Vertical-Stable target | Public-GA target |
|---|---|---|---|
| Leases under management | ≥ 100 (design partner REIT) | ≥ 10,000 | ≥ 500,000 |
| IFRS 16 schedule computation P99 | < 1s per lease | < 500ms | < 200ms |
| Foundry lease abstraction accuracy (key dates/rent) | ≥ 85% | ≥ 95% | ≥ 98% |
| CAM reconciliation cycle time (median) | < 30 days | < 15 days | < 7 days |
| Audit chain completeness | 100% | 100% | 100% |

---

## 10. Risks + Mitigations

| Risk | Severity | Mitigation | Owner |
|---|---|---|---|
| IFRS 16 computation error (wrong ROU asset) | Critical | Dual computation path: Foundry + deterministic in-house IFRS 16 engine; results compared before GL posting; auditor can replay schedule | Lease accounting domain |
| Lessee PII in cross-tenant data | High | Individual lessee data is `PII_IDENTIFYING`; tenant-private only; DSR cascade within 30 days | Privacy |
| Lease expiration not flagged in time | High | Automated renewal pipeline alert at 12/6/3 month marks; Foundry `RenewalRiskAnalyzer` proactive alert | Leasing domain |
| KR 상가임대차보호법 compliance alert missed | Medium | Regulatory-change watch lane; KR pack versioned; lease termination blocked if protection period active | KR pack + Leasing domain |

> TODO v0.2 — vertical owner to expand.

---

## 11. Open Questions

- IFRS 16 variable lease payments (indexed to CPI): in-scope for Stable or GA?
- GRESB ESG data submission: direct API integration or manual upload?
- KR 부동산 실거래가 신고: automated filing on lease execution or manual?

---

## 12. Decision Log

| Decision | Date | Rationale | ADR ref |
|---|---|---|---|
| IFRS 16 engine built in-house | 2026-05-09 | IFRS 16 computation is deterministic; in-house ensures auditability and no vendor lock-in | — |
| Foundry lease abstraction at T1 | 2026-05-09 | Lease terms have legal and financial consequences; accountant must review | ADR-0050 |
| Flat-crates: `crates/oya-vertical-real-estate-*` | 2026-05-09 | Per ADR-0015 | ADR-0015 |

---

## 13. Sources Scanned

- `docs/PRD.md`, `docs/DESIGN.md` §1, §4, §12
- `docs/PRIVACY-PROGRAM.md` §2.2.3
- IFRS 16 (Leases); ASC 842 (US GAAP Leases); KR 상가임대차보호법

---

## Doc-Catalog Row

```
| `vertical-real-estate` | `vertical-2` | leasing/IFRS-16/asset-mgmt/CAM/facilities | monthly | PRD.md, DESIGN.md §12 |
```
