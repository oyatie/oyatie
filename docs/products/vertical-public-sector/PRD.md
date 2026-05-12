# Oyatie — Product PRD: Vertical Public Sector

> **Status:** preview (skeleton)
> **Owning team:** [`teams/vertical-public-sector/CHARTER.md`](../../teams/vertical-public-sector/CHARTER.md)
> **Owning axis:** vertical-public-sector (Axis 2)
> **Catalog reference:** `registry/catalog/oya-vertical-public-sector-*.yaml`
> **Last updated:** 2026-05-09 by architecture-council

---

## 1. North Star

Oyatie Vertical Public Sector is the government operations and e-government services platform covering digital forms, procurement (KR 조달청 / global government procurement standards), case management, and citizen service workflows. It exists within the Oyatie ecosystem because government tenants require the strictest possible tenancy isolation, an audit chain meeting public-records-law standards, a Foundry agent runtime that operates entirely under human-in-the-loop control (T1 autonomy, no autonomous government decisions), and a privacy program enforcing the strongest regulatory defaults (public sector class forces classes 2-7, 12 always blocked from ads per PRIVACY-PROGRAM §2.2.3). The regional pack architecture allows KR 조달청, US G-Cloud / FedRAMP, EU TED / PEPPOL, and JP GEPS to be implemented in parallel as regional pack plug-ins.

---

## 2. Target Users

| Persona | What they get | What they pay for |
|---|---|---|
| Government Agency Administrator | Digital form authoring, workflow configuration, case management, procurement dashboard | Per-agency subscription |
| Civil Servant / Case Worker | Form processing, citizen request triage, procurement order management | Per-seat subscription |
| Procurement Officer | Tender publication, bid evaluation workflow, contract award, 조달청 API integration | Per-seat (procurement tier) |
| Citizen / Business (Portal User) | Self-service form submission, application status, document upload | No direct charge (agency pays) |
| Public Sector IT | Security classification config, FedRAMP/CSAP compliance dashboard, Foundry workflow authoring | Builder seat |

---

## 3. In-Scope / Out-of-Scope

### 3.1 In-scope at each wave

| Wave | Capabilities | Surfaces exposed |
|---|---|---|
| Vertical-Preview | Digital form builder (drag-and-drop + logic branching), form submission workflow, KR 조달청 나라장터 API integration (tender query + bid submission), basic case management | REST API v1, Citizen portal, Web UI |
| Vertical-Stable | Full procurement lifecycle (tender → bid → evaluation → award → contract), document management with security classification, case escalation workflow, Foundry-assisted document triage (T1 — human approves all decisions), PEPPOL BIS 3.0 e-procurement (EU), FedRAMP control evidence automation (US), KR CSAP compliance dashboard | REST API stable, Citizen portal, Webhook console |
| Public-GA | Cross-agency data sharing (consent-gated, audit-chained), open data portal (public datasets), Foundry regulatory-change monitor, global government procurement marketplace | Public OpenAPI, Open data portal |

### 3.2 Out-of-scope (anti-scope)

- Autonomous government decisions — all Foundry capabilities are T1 (recommend only); no autonomous regulatory determination
- Military / classified information handling (security classification handled at physical network level; Oyatie does not handle SCI/SAP)
- Advertising targeting using citizen or government data — **all public sector data classes 2-7, 12 always blocked** per PRIVACY-PROGRAM §2.2.3

---

## 4. Architecture Overview

### 4.1 Bounded Context

Flat-crates target prefix: `crates/oya-vertical-public-sector-*`.

```
crates/oya-vertical-public-sector-kernel-forms/       — Form, FormField, Submission, FormWorkflow entities
crates/oya-vertical-public-sector-kernel-procurement/ — Tender, Bid, Contract, ProcurementLine entities
crates/oya-vertical-public-sector-kernel-case/        — Case, CaseTask, CaseDocument, CaseParty entities
crates/oya-vertical-public-sector-domain-*/           — Use cases per sub-domain
crates/oya-vertical-public-sector-app-*/              — Sagas + Foundry delegation (T1 only)
crates/oya-vertical-public-sector-adapter-*/          — DB, 조달청 API, PEPPOL, FedRAMP adapters
crates/oya-vertical-public-sector-api-rest/           — REST API
crates/oya-vertical-public-sector-api-citizen/        — Citizen portal API (public-facing)
crates/oya-vertical-public-sector-runtime/            — Composition root
```

### 4.3 External-Facing Surfaces

| Surface | Contract location | Plane | SLO target |
|---|---|---|---|
| Public Sector REST API | `contracts/public-sector-core.openapi.yaml` | Data | 99.9% / p95 < 300ms |
| Citizen portal API | `contracts/public-sector-citizen.openapi.yaml` | Data | 99.5% / p95 < 500ms |
| PEPPOL AS4 gateway | `contracts/public-sector-peppol.yaml` | Data | 99.9% / p95 < 5s |

### 4.4 Internal Seams

> TODO v0.2 — vertical owner to expand.

---

## 5. Data Structures

### 5.1 Kernel Entities

```rust
// crates/oya-vertical-public-sector-kernel-forms
/// data_class: PII_IDENTIFYING (citizen submitter fields); BEHAVIORAL_TENANT_PRODUCT (form structure)
/// plane: data
/// PUBLIC SECTOR OVERRIDE: classes 2-7, 12 always blocked from ads (PRIVACY-PROGRAM §2.2.3)
pub struct FormSubmission {
    pub id: FormSubmissionId,
    pub tenant_id: TenantId,              // government agency tenant
    pub region: RegionCode,
    pub schema_version: u32,
    pub form_id: FormId,
    pub submitter_id: Option<CitizenId>,  // data_class: PII_IDENTIFYING
    pub submitter_name: Option<String>,   // data_class: PII_IDENTIFYING
    pub submitter_national_id: Option<NationalId>, // data_class: PII_IDENTIFYING
    pub fields: serde_json::Value,        // data_class: PII_IDENTIFYING (may contain citizen PII)
    pub status: SubmissionStatus,         // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub case_id: Option<CaseId>,
    pub submitted_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
pub enum SubmissionStatus { Received, UnderReview, Approved, Rejected, AwaitingInfo }

// crates/oya-vertical-public-sector-kernel-procurement
/// data_class: BEHAVIORAL_TENANT_PRODUCT (tender public), PII_IDENTIFYING (bidder contact)
/// plane: data
pub struct Tender {
    pub id: TenderId,
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub reference_number: String,         // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub title: String,                    // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub procurement_type: ProcurementType,
    pub estimated_value: Option<Money>,   // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub currency: CurrencyCode,
    pub submission_deadline: DateTime<Utc>,
    pub status: TenderStatus,
    pub naras_ref: Option<String>,        // KR 나라장터 tender number
    pub peppol_ref: Option<String>,       // EU PEPPOL tender reference
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
pub enum ProcurementType { OpenTender, RestrictedTender, NegotiatedProcedure, DirectAward }
pub enum TenderStatus { Draft, Published, Closed, Evaluated, Awarded, Cancelled }

// crates/oya-vertical-public-sector-kernel-case
/// data_class: PII_IDENTIFYING (citizen in the case); BEHAVIORAL_TENANT_PRODUCT (case metadata)
pub struct Case {
    pub id: CaseId,
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub case_number: String,              // data_class: INTERNAL_ONLY
    pub case_type: CaseType,             // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub status: CaseStatus,
    pub subject_citizen_id: Option<CitizenId>, // data_class: PII_IDENTIFYING
    pub assigned_to: Option<UserId>,
    pub priority: CasePriority,
    pub documents: Vec<CaseDocumentRef>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

> TODO v0.2 — vertical owner to add `Form`, `FormField`, `Bid`, `ProcurementContract`, `CaseTask` entities with full field enumeration.

### 5.2–5.7

> TODO v0.2 — vertical owner to expand aggregate boundaries, persistence layout, event schemas, index touchpoints, audit-chain contract, migration policy.

Key audit events: `FormSubmitted`, `TenderPublished`, `BidReceived`, `ContractAwarded`, `CaseEscalated`.

**Mandatory audit chain entries**: every citizen-facing action emits an audit record per 정보공개법 / FOIA (US) / EU transparency regulation.

---

## 6. Optimization Practices

| Practice | Implementation choice |
|---|---|
| Cell routing | `(tenant_id, agency_region)` → cell; data residency is government-mandated |
| Sharding strategy | Per-agency shard; citizen submission data per-agency isolated |
| Caching tier | Redis for form definition cache; in-memory for tender catalog; no citizen PII in cache |
| Bulk endpoint contract | `POST /submissions/bulk`; `POST /tenders/bulk-publish` |
| Agent-driven optimization | Foundry `SubmissionTriager` (document classification, T1 — human caseworker approves); Foundry `ProcurementAnalyzer` (bid anomaly detection, T1) |

> TODO v0.2 — vertical owner to expand.

---

## 7. Regional Pack Interactions

| Seam | Trait | Per-pack impl needed? | Tested with |
|---|---|---|---|
| Procurement platform adapter | `ProcurementPlatformAdapter` | Yes | `oya-pack-kr` (조달청 나라장터 G2B API), `oya-pack-us` (SAM.gov + GSA APIs), `oya-pack-eu` (TED / PEPPOL BIS 3.0) |
| Citizen identity verification | `CitizenIdentityProvider` | Yes | `oya-pack-kr` (정부24 / 민원24 / 공동인증서), `oya-pack-us` (Login.gov / ID.me), `oya-pack-eu` (eIDAS eID) |
| e-Invoice / e-Procurement format | `WaybillFormatter` | Yes | `oya-pack-kr` (국가 전자조달 XML), `oya-pack-eu` (PEPPOL UBL 2.1) |
| Security classification | `SecurityClassificationPolicy` | Yes | `oya-pack-kr` (국가정보원 보안등급), `oya-pack-us` (FedRAMP Low/Moderate/High) |
| Regulatory control evidence | `RegulatoryPack` | Yes | `oya-pack-kr` (CSAP, K-ISMS-P, NIS), `oya-pack-us` (FedRAMP, FISMA), `oya-pack-eu` (GAIA-X, EU AI Act public-sector provisions) |

### Regulatory Pack Declaration

```yaml
regulatory_packs:
  - oya-pack-kr   # CSAP, K-ISMS-P, NIS, 조달청, 정보공개법, PIPA
  - oya-pack-us   # FedRAMP, FISMA, FOIA, FAR/DFARS, SAM.gov
  - oya-pack-eu   # GAIA-X, PEPPOL, EU AI Act (public sector), TED
  - oya-pack-jp   # ISMAP, GEPS, e-Gov
tenant_class_overrides:
  ad_targetable_blocked: true    # classes 2-7, 12 always blocked; hardest public-sector default
```

---

## 8. In-House vs External Dependency Posture

> TODO v0.2 — vertical owner to expand. Key: `tokio`/`axum`/`sqlx`/`serde`/`rustls` (kernel-grade); 조달청 G2B API client (in-house adapter); PEPPOL AS4 gateway (in-house adapter in `oya-vertical-public-sector-adapter-peppol`); form builder rendering (in-house Rust/WASM).

---

## 9. Success Metrics

| Metric | Vertical-Preview target | Vertical-Stable target | Public-GA target |
|---|---|---|---|
| Government agency tenants | ≥ 1 (KR design partner) | ≥ 10 agencies | ≥ 100 agencies across 3+ countries |
| Form submission processing P99 | < 2s | < 1s | < 500ms |
| Procurement tender publication (end-to-end) | < 1 business day | < 4 hours | < 1 hour |
| Audit chain completeness (all citizen-facing actions) | 100% | 100% | 100% |
| FedRAMP / CSAP evidence automation coverage | baseline | ≥ 50% automated | ≥ 80% automated |

---

## 10. Risks + Mitigations

| Risk | Severity | Mitigation | Owner |
|---|---|---|---|
| Citizen PII in cross-agency data share (without consent) | Catastrophic | Cross-agency share requires explicit consent + audit-chain record per share; structural enforcement at eventing backbone | Privacy + Architecture |
| Foundry autonomous government decision (T1 breached) | Critical | Cedar policy enforces T1-only for all public-sector capabilities; no T2+ grants without council approval | Foundry + Governance |
| Procurement bid data exposure before deadline | Critical | Bid submissions encrypted at rest with DEK per tender; decryption only after deadline timestamp; Cedar policy enforces | Security |
| CSAP / FedRAMP compliance evidence gap | High | Foundry `ComplianceEvidenceCollector` automates control evidence; weekly evidence freshness check | Compliance + KR/US pack |
| 조달청 API breaking change | Medium | Adapter pattern isolates 조달청 dependency; versioned adapter with contract test | KR pack + Public sector domain |

> TODO v0.2 — vertical owner to expand risk register.

---

## 11. Open Questions

- KR 마이데이터 for citizens (공공 마이데이터) — integration scope at Stable?
- FedRAMP IL2 vs IL4 vs IL5 target: which impact level for initial US launch?
- Cross-agency data federation: ABAC policy model or full re-consent per share?

---

## 12. Decision Log

| Decision | Date | Rationale | ADR ref |
|---|---|---|---|
| Foundry autonomy ceiling T1-only for public sector | 2026-05-09 | Autonomous government decisions are legally prohibited in most jurisdictions | ADR-0050 |
| All public-sector data classes 2-7, 12 blocked from ads | 2026-05-09 | PRIVACY-PROGRAM §2.2.3; 공공정보법 + 정보공개법 | PRIVACY-PROGRAM §2.2.3 |
| Flat-crates: `crates/oya-vertical-public-sector-*` | 2026-05-09 | Per ADR-0015 | ADR-0015 |

---

## 13. Sources Scanned

- `docs/PRD.md`, `docs/DESIGN.md` §1, §4, §12
- `docs/PRIVACY-PROGRAM.md` §2.2.3 (public sector override)

---

## Doc-Catalog Row

```
| `vertical-public-sector` | `vertical-2` | forms/조달청/global-gov-procurement/case-mgmt | monthly | PRD.md, DESIGN.md §12, PRIVACY-PROGRAM.md §2.2.3 |
```
