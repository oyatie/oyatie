# Oyatie — Product PRD: Vertical Legal

> **Status:** preview
> **Owning team:** [`teams/vertical-legal/CHARTER.md`](../../teams/vertical-legal/CHARTER.md)
> **Owning axis:** vertical-legal (Axis 2)
> **Catalog reference:** `registry/catalog/oya-vertical-legal-*.yaml`
> **Last updated:** 2026-05-09 by architecture-council

---

## 1. North Star

Oyatie Vertical Legal is a regulated-corpus management and legal operations platform for law firms, in-house legal departments, and compliance teams. It covers contract lifecycle management (CLM), regulated-corpus ingestion and search (case law, statutes, regulatory guidance), AI-assisted contract analysis and redlining (Foundry-powered, under autonomy ceiling), and compliance workflow management. The canonical entity model — LegalDocument, Contract, ContractClause, Matter, RegulatoryCorpus, ComplianceObligation — is designed for the dual role of (a) being the operational system for legal work product and (b) being the compliance-assistance layer for tenants across other verticals (healthcare compliance, fintech AML regulatory obligations, industrial safety regulations). The product exists within Oyatie's ecosystem because the coupling of a legal corpus search engine (backed by Oyatie Search's per-tenant-private index), Foundry agents operating under strict autonomy ceiling constraints (legal recommendations require human-lawyer review; no autonomous legal filing), the audit chain providing privilege-preserving access logs, and a privacy program that classifies all legal work product as `INTERNAL_ONLY` or `BEHAVIORAL_TENANT_PRODUCT` (never ad-targetable) is the trust architecture that no standalone legal-tech SaaS can offer with the same structural guarantees.

---

## 2. Target Users

| Persona | What they get | What they pay for |
|---|---|---|
| General Counsel / CLO | Contract portfolio dashboard, risk exposure overview, matter status, regulatory obligation tracker | Per-seat subscription (GC tier) |
| Corporate Attorney / Counsel | Contract drafting (Foundry-assisted), clause library, negotiation redline comparison, e-signature workflow | Per-seat (attorney tier) |
| Paralegal / Contract Manager | Contract intake, metadata extraction (Foundry), obligation tracking, renewal alert, executed contract archival | Per-seat (paralegal tier) |
| Compliance Officer | Regulatory obligation mapping to internal controls, compliance gap analysis, evidence collection workflow | Per-seat (compliance tier) — cross-sells with Fintech/Healthcare compliance posture |
| Legal Research Analyst | Regulated corpus search (case law, statutes, regulatory guidance), citation extraction, comparable-clause search | Per-seat (research tier) |
| Legal IT / Tenant Builder | Corpus ingestion pipeline config, clause template authoring, Foundry legal-workflow capability authoring | Builder seat |
| Outside Counsel (Guest) | Matter collaboration portal, document share (privilege-gated), e-signature | Per-matter seat (guest, metered) |

---

## 3. In-Scope / Out-of-Scope

### 3.1 In-scope at each wave

| Wave | Capabilities | Surfaces exposed |
|---|---|---|
| Vertical-Preview | Contract creation and storage, basic metadata extraction (Foundry-assisted: party names, effective date, termination date, governing law), contract search (tenant-private), e-signature integration (DocuSign / KR 전자서명법 aligned), regulated corpus ingestion (KR 법률정보 + US federal statutes initial set) | REST API v1, Web UI (CLM), Tenant-private search |
| Vertical-Stable | Full CLM lifecycle (intake → negotiation → execution → obligation tracking → renewal/termination), Foundry contract analysis (risk clause detection, deviation from playbook, comparable precedent search), redline generation (Foundry — recommend only, attorney approves), matter management (litigation + transactional), regulatory corpus expansion (KR 대법원 판례, US case law via CourtListener, EU EUR-Lex, JP 裁判所), privilege-log generation (Foundry-assisted), compliance obligation mapping with automated evidence collection | REST API stable, Webhook console, Guest portal |
| Public-GA | Cross-matter analytics (clause frequency, risk heat map by counterparty), AI contract scoring (Foundry, recommend only), regulatory-change watch (Foundry monitors corpus for new guidance, alerts compliance team), global contract clause library (benchmarked to market practice per jurisdiction), API for compliance evidence export to regulator portal | Public OpenAPI, Analytics dashboard, Evidence export |
| Region-Fan-Out | Per-regional-pack legal corpus (jurisdiction-specific statutes + case law), local e-signature law compliance, local notarization workflow | Per-pack launch cadence |

### 3.2 Out-of-scope (anti-scope)

- Autonomous legal filing (court e-filing, patent filing, regulatory submission filing) — Foundry may draft; a licensed attorney must review and file
- Legal billing / time-entry (LEDES format) at depth — declared as a seam for integration with Legal ERP (e.g., Clio, NetSuite Legal)
- IP docketing at patent-portfolio management depth
- Litigation document review (eDiscovery at production-volume scale) — tenants may use Oyatie Legal for work-product management; full eDiscovery processing platform is a separate evaluation
- Advertising targeting using any legal work product, matter data, or contract terms — permanently blocked; all legal data is `INTERNAL_ONLY` or `BEHAVIORAL_TENANT_PRODUCT` — never ad-targetable

---

## 4. Architecture Overview

### 4.1 Bounded Context

Axis 2 — Vertical Legal. Flat-crates target prefix: `crates/oya-vertical-legal-*`.

The legal vertical owns contract lifecycle, matter management, regulated corpus, and compliance obligation bounded contexts. Cross-axis contracts: `oya-platform-tenant-kernel`, `oya-platform-audit-chain-kernel` (privilege log + access audit), `oya-foundry-api` (contract analysis + compliance watch agents under T1 max autonomy), `oya-search-index-kernel` (tenant-private regulated corpus index), `oya-platform-regulatory-kernel` (per-jurisdiction legal corpus packs).

### 4.2 Layered Structure

```
crates/oya-vertical-legal-kernel-contract/         — Contract, ContractClause, ContractParty, ClauseTemplate entities
crates/oya-vertical-legal-kernel-matter/           — Matter, MatterTask, MatterDocument, MatterParticipant entities
crates/oya-vertical-legal-kernel-corpus/           — LegalDocument, Statute, CaseLaw, RegulatoryGuidance, Citation entities
crates/oya-vertical-legal-kernel-compliance/       — ComplianceObligation, ControlMapping, EvidenceRecord entities
crates/oya-vertical-legal-domain-contract/         — CLM use cases: intake, draft, negotiate, execute, track-obligations, renew
crates/oya-vertical-legal-domain-matter/           — Matter lifecycle use cases
crates/oya-vertical-legal-domain-corpus/           — Corpus ingestion, citation extraction, precedent search use cases
crates/oya-vertical-legal-domain-compliance/       — Obligation mapping, evidence collection, gap analysis use cases
crates/oya-vertical-legal-app-clm/                — CLM saga, Foundry contract-analysis capability delegation
crates/oya-vertical-legal-app-corpus/             — Corpus ingestion pipeline (statute + case law + regulatory guidance)
crates/oya-vertical-legal-adapter-db/             — Postgres adapters
crates/oya-vertical-legal-adapter-esign/          — E-signature adapter (DocuSign + KR 전자서명법-aligned providers)
crates/oya-vertical-legal-adapter-corpus/         — Corpus source adapters (KR 법률정보, CourtListener, EUR-Lex, JP 裁判所)
crates/oya-vertical-legal-api-rest/               — REST API handlers
crates/oya-vertical-legal-worker-events/          — Kafka consumers (corpus-update, obligation-due, signature-completed)
crates/oya-vertical-legal-runtime/               — Composition root binary
```

### 4.3 External-Facing Surfaces

| Surface | Contract location | Plane | SLO target |
|---|---|---|---|
| Legal REST API | `contracts/legal-clm.openapi.yaml` | Data | 99.9% / p95 < 300ms |
| Regulated corpus search | internal (via Search axis tenant-private index) | Data | 99.5% / p95 < 500ms |
| Guest matter portal | `contracts/legal-guest.openapi.yaml` | Data | 99.5% / p95 < 500ms |
| Webhook events (contract-executed, obligation-due) | `contracts/legal-webhooks.yaml` | Data | at-least-once, ≤ 60s |

### 4.4 Internal Seams

| Seam | Trait / interface | Consumer products |
|---|---|---|
| `LegalCorpusIndexable` | `SearchIndexable` (tenant-private) | Search axis (regulated corpus full-text + vector search) |
| `ContractSearchIndexable` | `SearchIndexable` (tenant-private) | Search axis (contract portfolio search) |
| `ComplianceEvidenceExportable` | `AuditChainEmitter` | Audit chain (regulatory evidence export) |
| `PrivilegeLogEmitter` | `AuditChainEmitter` | Audit chain (attorney-client privilege access log) |

### 4.5 Dependencies on Other Axes

| Contract consumed | Owner axis | Where it lives | Change-review class |
|---|---|---|---|
| `Tenant` kernel | SaaS platform | `oya-platform-tenant-kernel` | Cross-axis review |
| `Capability invocation` (contract analysis T1 max) | Foundry | `oya-foundry-api` | Foundry + legal review |
| `Audit-chain event` (privilege log mandatory) | Platform | `oya-platform-audit-chain-kernel` | Audit review |
| `Search index lifecycle` (tenant-private corpus) | Search | `oya-search-index-kernel` | Search + legal review |

---

## 5. Data Structures

### 5.1 Kernel Entities

```rust
// crates/oya-vertical-legal-kernel-contract

/// data_class: BEHAVIORAL_TENANT_PRODUCT (contract terms are tenant work product)
/// plane: data
/// Attorney-client privilege metadata: access logs emit to audit chain
pub struct Contract {
    pub id: ContractId,
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub title: String,                             // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub contract_type: ContractType,               // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub status: ContractStatus,                    // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub parties: Vec<ContractParty>,               // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub effective_date: Option<NaiveDate>,         // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub expiration_date: Option<NaiveDate>,        // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub governing_law: Option<JurisdictionCode>,   // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub document_ref: DocumentRef,                 // data_class: BEHAVIORAL_TENANT_PRODUCT (blob in object store)
    pub clauses: Vec<ContractClauseRef>,           // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub obligations: Vec<ObligationRef>,           // links to ComplianceObligation
    pub foundry_analysis_run_id: Option<FoundryRunId>, // data_class: INTERNAL_ONLY
    pub privilege_class: PrivilegeClass,           // data_class: INTERNAL_ONLY (attorney-client / work-product / none)
    pub matter_id: Option<MatterId>,               // data_class: INTERNAL_ONLY
    pub esign_envelope_id: Option<EsignEnvelopeId>,// data_class: INTERNAL_ONLY
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum ContractStatus { Draft, InNegotiation, PendingSignature, Executed, Expired, Terminated, Archived }
pub enum ContractType { NDA, MSA, SOW, SLA, Employment, Lease, SupplyChain, License, JV, Other }
pub enum PrivilegeClass { AttorneyClient, WorkProduct, None }

/// data_class: BEHAVIORAL_TENANT_PRODUCT
pub struct ContractClause {
    pub id: ContractClauseId,
    pub contract_id: ContractId,
    pub tenant_id: TenantId,
    pub schema_version: u32,
    pub clause_type: ClauseType,                  // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub text: String,                             // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub risk_level: Option<ClauseRiskLevel>,      // data_class: INTERNAL_ONLY (Foundry-assessed)
    pub deviation_from_playbook: Option<bool>,    // data_class: INTERNAL_ONLY (Foundry-assessed)
    pub foundry_analysis: Option<serde_json::Value>, // data_class: INTERNAL_ONLY
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum ClauseType {
    Indemnification, Limitation, Warranty, Termination, Confidentiality,
    IPOwnership, Governing, Dispute, ForceMAjeure, Payment, Other
}
pub enum ClauseRiskLevel { Low, Medium, High, Critical }
```

```rust
// crates/oya-vertical-legal-kernel-corpus

/// Regulatory corpus entry — statutes, case law, regulatory guidance
/// data_class: PUBLIC (published legal materials) or BEHAVIORAL_TENANT_PRODUCT (tenant annotations)
/// plane: data (corpus); control (index lifecycle)
pub struct LegalDocument {
    pub id: LegalDocumentId,
    pub tenant_id: TenantId,                      // corpus is per-tenant or system-wide PUBLIC
    pub region: RegionCode,
    pub schema_version: u32,
    pub document_class: LegalDocumentClass,       // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub jurisdiction: JurisdictionCode,           // data_class: PUBLIC
    pub citation: String,                         // data_class: PUBLIC (e.g., "대법원 2023다12345")
    pub title: String,                            // data_class: PUBLIC
    pub effective_date: Option<NaiveDate>,        // data_class: PUBLIC
    pub source_url: Option<Url>,                  // data_class: PUBLIC
    pub full_text: Option<EncryptedBlob>,         // data_class: PUBLIC (indexed in search)
    pub summary: Option<String>,                  // data_class: PUBLIC (Foundry-generated summary)
    pub embedding_ref: Option<EmbeddingRef>,      // data_class: INTERNAL_ONLY (vector embedding pointer)
    pub citations: Vec<LegalDocumentId>,          // data_class: PUBLIC (citation graph)
    pub annotations: Vec<TenantAnnotation>,       // data_class: BEHAVIORAL_TENANT_PRODUCT (tenant-specific notes)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum LegalDocumentClass { Statute, Regulation, CaseLaw, RegulatoryGuidance, Treaty, Other }
```

```rust
// crates/oya-vertical-legal-kernel-compliance

/// data_class: BEHAVIORAL_TENANT_PRODUCT
/// plane: data
pub struct ComplianceObligation {
    pub id: ComplianceObligationId,
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub source: ObligationSource,                 // Contract, Regulation, InternalPolicy
    pub source_ref: Option<LegalDocumentId>,      // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub contract_ref: Option<ContractId>,         // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub description: String,                      // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub obligation_type: ObligationType,          // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub due_date: Option<NaiveDate>,              // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub recurrence: Option<ObligationRecurrence>, // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub status: ObligationStatus,                 // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub assigned_to: Option<UserId>,              // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub evidence_refs: Vec<EvidenceRef>,          // data_class: INTERNAL_ONLY
    pub foundry_run_id: Option<FoundryRunId>,     // data_class: INTERNAL_ONLY
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum ObligationSource { Contract, Regulation, InternalPolicy }
pub enum ObligationType { Reporting, Payment, Delivery, Consent, Disclosure, Review, Certification }
pub enum ObligationStatus { Active, InProgress, Completed, Overdue, Waived }
```

```rust
// crates/oya-vertical-legal-kernel-matter

/// data_class: BEHAVIORAL_TENANT_PRODUCT (may include PII if litigation involves individuals)
/// plane: data
pub struct Matter {
    pub id: MatterId,
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub matter_number: String,                    // data_class: INTERNAL_ONLY
    pub title: String,                            // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub matter_type: MatterType,                  // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub status: MatterStatus,                     // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub jurisdiction: Option<JurisdictionCode>,   // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub privilege_class: PrivilegeClass,          // data_class: INTERNAL_ONLY
    pub participants: Vec<MatterParticipant>,     // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub documents: Vec<MatterDocumentRef>,        // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub contracts: Vec<ContractId>,               // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub opposing_party: Option<PartyRef>,         // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub outside_counsel_ref: Option<TenantRef>,   // data_class: INTERNAL_ONLY (guest tenant)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum MatterType { Litigation, Arbitration, Regulatory, Transactional, Advisory, IP, Employment }
pub enum MatterStatus { Open, OnHold, Closed, Archived }
```

### 5.2 Aggregate Boundaries

| Aggregate | Root entity | Consistency boundary |
|---|---|---|
| `ContractAggregate` | `Contract` + `ContractClause[]` + `ContractParty[]` | Full contract lifecycle; clauses are inline (part of the contract); obligations are separate aggregates |
| `MatterAggregate` | `Matter` + `MatterTask[]` + `MatterDocument[]` | Full matter lifecycle; privilege class enforced at aggregate level |
| `LegalDocumentAggregate` | `LegalDocument` | Public corpus document with tenant annotations; search embedding is a derived projection |
| `ComplianceObligationAggregate` | `ComplianceObligation` + `EvidenceRecord[]` | Obligation lifecycle with evidence; evidence records are inline |

### 5.3 Persistence Layout

| Aggregate | Store | Sharding key | Partition strategy | Replication | Retention |
|---|---|---|---|---|---|
| Contract + ContractClause | Postgres (per-tenant shard) | `tenant_id` | Per-tenant schema | Streaming replication × 2 | Per-tenant retention policy (default 10 years after expiration) |
| Matter | Postgres (per-tenant shard) | `tenant_id` | Per-tenant schema | Streaming replication × 2 | 10 years after close (litigation hold may extend) |
| LegalDocument (PUBLIC corpus) | Postgres (shared corpus shard) + Search index | `jurisdiction` | Per-jurisdiction partition | Streaming replication × 2 | Indefinite (public law doesn't expire) |
| ComplianceObligation + Evidence | Postgres (per-tenant shard) | `tenant_id` | Per-tenant schema | Streaming replication × 2 | 7 years (general compliance record retention) |

### 5.4 Event Schemas

| Event name | Topic | Schema location | Consumer aggregates | Retention | Idempotency key |
|---|---|---|---|---|---|
| `ContractExecuted` | `legal.contract.executed` | `contracts/events/legal-clm.json` | Obligation tracking, Matter update, Audit chain | 90 days | `contract_id` |
| `ObligationDue` | `legal.obligation.due` | `contracts/events/legal-compliance.json` | Compliance alert, Matter task, Audit chain | 30 days | `(obligation_id, due_date)` |
| `CorpusDocumentIngested` | `legal.corpus.ingested` | `contracts/events/legal-corpus.json` | Search index (tenant-private), Foundry regulatory-change alert | 30 days | `legal_document_id` |
| `ContractAnalysisCompleted` | `legal.contract.analyzed` | `contracts/events/legal-clm.json` | Contract clause risk update, Attorney notification | 30 days | `(contract_id, foundry_run_id)` |
| `PrivilegeDocumentAccessed` | `legal.privilege.accessed` | `contracts/events/legal-audit.json` | Audit chain (mandatory), Privilege log | 7 years | `(document_ref, accessor_id, timestamp)` |

### 5.5 Index / Search-Index Touchpoints

| Entity field | Index | Class allowed | Cascade-on-DSR? |
|---|---|---|---|
| `LegalDocument.full_text` (PUBLIC corpus) | tenant-private corpus search index | `PUBLIC` (statute/case law text) | No (public law) |
| `Contract.title` + `clause.text` | tenant-private contract search | `BEHAVIORAL_TENANT_PRODUCT` | Yes — DSR cascade on contract work product if data subject requests |
| `Matter.title` + `matter_type` | tenant-private matter search | `BEHAVIORAL_TENANT_PRODUCT` | Yes (litigation matters may name individuals) |

### 5.6 Audit-Chain Emission Contract

| Operation | Emits topic | Required fields |
|---|---|---|
| Privilege document accessed | `audit.legal.privilege_accessed` | `document_ref`, `accessor_id`, `matter_id`, `privilege_class`, `access_type`, `timestamp` |
| Contract executed (e-signature complete) | `audit.legal.contract_executed` | `contract_id`, `parties`, `esign_envelope_id`, `executed_at` |
| Compliance obligation evidence collected | `audit.legal.evidence_collected` | `obligation_id`, `evidence_ref`, `collected_by`, `collected_at` |
| Foundry contract analysis run | `audit.legal.contract_analyzed` | `contract_id`, `foundry_run_id`, `model_id`, `clauses_analyzed`, `risk_flags` |
| Legal corpus document regulatory change | `audit.legal.corpus_change_detected` | `legal_document_id`, `jurisdiction`, `change_summary`, `detected_at` |

### 5.7 Schema Migration Policy

- Contract and Matter data are never destructively deleted (litigation hold may apply); migrations are additive.
- Privilege class field is immutable after creation; no downgrade from `AttorneyClient` to `None`.
- Corpus schema is stable and additive; new LegalDocumentClass values are additive enum extensions.

---

## 6. Optimization Practices

| Practice | Implementation choice |
|---|---|
| Cell routing | `tenant_id` → cell; legal cells co-located with tenant's primary region for data residency |
| Sharding strategy | Per-tenant Postgres shard for Contract/Matter/Obligation; shared corpus shard for PUBLIC legal documents |
| Caching tier | In-memory LRU for clause template library (low-churn); Redis for regulated corpus search result cache (1-hour TTL); no privilege-class documents in Redis |
| Bulk endpoint contract | `POST /contracts/bulk-analyze` (Foundry batch contract analysis); `POST /corpus/ingest/bulk` (statute/case law bulk ingestion) |
| Pagination | Cursor on `(created_at, contract_id)`; `_since` filter for obligation due-date queries |
| Idempotency | `Idempotency-Key` on contract creation and e-signature envelope submission |
| Batch dispatch | Foundry `ContractAnalyzer` runs per-contract batch (asynchronous after intake); Foundry `RegulatoryWatcher` runs corpus diff batch daily |
| Backpressure | Corpus ingestion pipeline rate-limited to avoid overwhelming search index; Foundry analysis queue depth monitored |
| Hot-path benchmarks | `contract_search` criterion < 200ms; `clause_risk_lookup` < 50ms; `corpus_search_query` < 500ms |
| Agent-driven optimization | Foundry `ContractAnalyzer` (clause extraction, risk detection, playbook deviation — recommend only, T1); Foundry `RegulatoryWatcher` (corpus change detection, compliance alert); Foundry `ObligationTracker` (proactive due-date alerts) |
| FinOps unit-economics | Per-contract-under-management; per-Foundry-analysis-run; corpus storage per-GB-month |
| Build-cache / CI affected-graph | `oya-vertical-legal-kernel-contract` → full rebuild; `adapter-corpus` → corpus ingestion integration tests |

---

## 7. Regional Pack Interactions

| Seam | Trait | Per-pack impl needed? | Tested with which packs? |
|---|---|---|---|
| Legal corpus source adapter | `LegalCorpusSource` | Yes — per jurisdiction | `oya-pack-kr` (법제처 국가법령정보, 대법원 판례 APIS), `oya-pack-us` (CourtListener + Congress.gov), `oya-pack-eu` (EUR-Lex + national corpus per country), `oya-pack-jp` (裁判所 判例集 + e-Gov) |
| E-signature law compliance | `EsignLawAdapter` | Yes | `oya-pack-kr` (전자서명법 — 공인전자서명 + 간편전자서명), `oya-pack-us` (ESIGN Act + UETA), `oya-pack-eu` (eIDAS eSignature — QES/AES/SES tiers) |
| Regulatory control evidence | `RegulatoryPack` | Yes | All packs (compliance evidence export is cross-vertical) |
| Identity-provider adapter (for attorney authentication) | `IdentityProvider` | Yes | `oya-pack-kr` (변호사 자격확인 — 대한변호사협회 API), `oya-pack-us` (bar number validation — state bar APIs), `oya-pack-eu` (lawyer federation eIDAS) |

### Regulatory Pack Declaration

```yaml
# registry/catalog/oya-vertical-legal-runtime.yaml
regulatory_packs:
  - oya-pack-kr   # 전자서명법, 법원 판례, 법제처 법령, 변호사법
  - oya-pack-us   # ESIGN Act, UETA, Uniform Electronic Records
  - oya-pack-eu   # eIDAS eSignature (QES/AES/SES), EUR-Lex
  - oya-pack-jp   # 電子署名法, 裁判所, e-Gov
```

---

## 8. In-House vs External Dependency Posture

| External dep | Maturity tier | License | In-house alternative considered? | Decision |
|---|---|---|---|---|
| `tokio`, `axum`, `sqlx`, `serde`, `rustls` | kernel-grade | MIT / Apache-2 | No | Use |
| `pulldown-cmark` (Markdown rendering for clause text) | stable | MIT | In-house trivial | Use |
| `docx-rs` (DOCX generation for contract export) | stable | MIT | In-house DOCX considered | Use |
| `pdf-rs` / `lopdf` (PDF export of executed contracts) | stable | MIT | In-house considered | Use lopdf |
| DocuSign API (e-signature) | external API | Proprietary (no code dep) | In-house e-signature considered — complex trust-chain requirements | Adapter pattern; DocuSign as first impl; KR 공인전자서명 as KR-pack adapter |
| CourtListener API (US case law) | external API | CC Attribution | No alternatives for US case law at scale | Adapter in `oya-vertical-legal-adapter-corpus` |
| EUR-Lex SPARQL/REST API (EU law) | external API | EC Open Data | No alternatives | Adapter |

---

## 9. Success Metrics

| Metric | Vertical-Preview target | Vertical-Stable target | Public-GA target |
|---|---|---|---|
| Contracts under management | ≥ 500 (design-partner legal dept) | ≥ 50,000 | ≥ 1,000,000 |
| Foundry contract analysis adoption | ≥ 30% of new contracts | ≥ 70% | ≥ 90% |
| Clause risk detection precision (attorney-validated) | ≥ 75% | ≥ 85% | ≥ 90% |
| Obligation due-date alert lead time | ≥ 30 days prior | ≥ 45 days | ≥ 60 days (configurable) |
| Privilege access audit completeness | 100% | 100% | 100% |
| Corpus ingestion freshness (statute/case law lag) | < 7 days from publication | < 48 hours | < 24 hours |
| Contract search P99 | < 500ms | < 300ms | < 200ms |
| DSR / legal subject data fulfillment | < 30 days | < 15 days | < 7 days |
| Cross-axis contract violations | 0 | 0 | 0 |

---

## 10. Risks + Mitigations

| Risk | Severity | Mitigation | Owner |
|---|---|---|---|
| Foundry hallucination in contract analysis (wrong risk assessment) | High | Autonomy ceiling T1 = recommend only; attorney must review every Foundry-generated analysis before acting; Foundry output never auto-posted to contract record without attorney approval | Foundry + Legal domain |
| Privilege waiver risk (privileged document accessed by unauthorized party) | Critical | `PrivilegeClass` field enforced at aggregate level; Cedar policy gates document access by role; every access emits mandatory audit record; no privilege document in cross-tenant search index | Privacy + Security + Legal domain |
| Corpus stale data (regulatory change not picked up) | High | Foundry `RegulatoryWatcher` runs daily corpus diff; freshness SLO tracked; alert if corpus source API unavailable > 24 hours | Legal domain + SRE |
| E-signature non-repudiation failure (KR 전자서명법) | Critical | KR-pack e-signature adapter uses 공인전자서명 (qualified) by default; timestamp authority (TSA) binding per signature; audit chain record per execution | Security + KR pack |
| Legal work product leak across tenant boundary | Critical | All contract/matter data is `BEHAVIORAL_TENANT_PRODUCT`; tenant-private search index only; no cross-tenant corpus sharing of work product | Privacy + Architecture |
| Obligation tracking failure (missed compliance deadline) | High | Multi-layer alert (email + Oyatie notification + Foundry proactive alert); dead-letter queue for failed obligation-due events; manual override capture | Legal domain + SRE |

---

## 11. Open Questions

- KR 전자서명법 qualified signature (공인전자서명) — is KICA (한국인터넷진흥원) certification required for all contracts or only specific regulated document types?
- eDiscovery processing scope — should Oyatie Legal offer a basic document review workflow (Vertical-Stable) or defer entirely to integration with Relativity/Logikcull?
- Legal billing (LEDES time-entry) — in-scope seam integration at Vertical-Stable or deferred to vertical owner?
- Litigation hold enforcement — should Oyatie Legal enforce litigation hold by blocking DSR cascade on flagged matter documents?
- Attorney licensing verification — active API integration with bar associations (KR 대한변호사협회, state bars US) at Preview or Stable?

---

## 12. Decision Log

| Decision | Date | Rationale | ADR ref |
|---|---|---|---|
| Foundry contract analysis at T1 (recommend only, attorney approves) | 2026-05-09 | Unauthorized practice of law (UPL) risk; attorney must be in the loop for any legal conclusion | ADR-0050 |
| Privilege class field immutable after creation | 2026-05-09 | Attorney-client privilege cannot be waived inadvertently by a software downgrade | — |
| PUBLIC corpus documents stored in shared shard | 2026-05-09 | Statutes and case law are public; no per-tenant isolation needed; shared shard reduces storage cost | — |
| In-house corpus ingestion adapters (no third-party legal data vendor lock-in) | 2026-05-09 | LexisNexis/Westlaw lock-in avoided; direct government API access where available | — |
| Flat-crates: `crates/oya-vertical-legal-*` | 2026-05-09 | Per ADR-0015 | ADR-0015 |

---

## 13. Sources Scanned

- `docs/PRD.md`, `docs/DESIGN.md` §1, §4, §10, §12
- `docs/PRIVACY-PROGRAM.md` §2.2.1, §2.2.3
- KR 전자서명법; EU eIDAS Regulation; US ESIGN Act; FATF guidance on legal sector AML

---

## Doc-Catalog Row

```
| `vertical-legal` | `vertical-2` | CLM/regulated-corpus/contract-analysis/compliance-assist | monthly | PRD.md, DESIGN.md §12, PRIVACY-PROGRAM.md §2.2.3 |
```
