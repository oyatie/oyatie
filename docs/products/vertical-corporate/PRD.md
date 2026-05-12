# Oyatie — Product PRD: Vertical Corporate

> **Status:** preview
> **Owning team:** [`teams/vertical-corporate/CHARTER.md`](../../teams/vertical-corporate/CHARTER.md)
> **Owning axis:** vertical-corporate (Axis 2)
> **Catalog reference:** `registry/catalog/oya-vertical-corporate-*.yaml`
> **Last updated:** 2026-05-09 by architecture-council

---

## 1. North Star

Oyatie Vertical Corporate is the enterprise operations suite for holding groups, conglomerates, and mid-market organizations worldwide — anchored on the KR Group anchor tenant (multi-subsidiary holding structure) and designed for global fan-out from day one. It covers HR lifecycle, payroll processing, general ledger and accounts-payable/receivable, corporate mail, and unified communications — delivered not as a loosely integrated suite but as a single bounded context sharing Oyatie's canonical tenancy model, identity surface, audit chain, and Foundry agent runtime. The product can only exist as part of the Oyatie ecosystem because its value proposition — a single identity spanning HR roles, payroll actors, GL approvers, and mail recipients; a single audit trail for every payroll run, every GL journal, every document flow; and an agent runtime that authors payroll journals, reconciles GL, and drafts communications autonomously under the autonomy ceiling — collapses into commodity SaaS the moment any of those axes are unbundled.

---

## 2. Target Users

| Persona | What they get | What they pay for |
|---|---|---|
| HR Director / CHRO | Employee lifecycle management, org-chart authoring, policy compliance dashboard, headcount analytics | Per-seat SaaS subscription (HR tier) |
| Payroll Manager | End-to-end payroll run with statutory deduction automation per regional pack, audit trail per run, Foundry-authored payroll journals | Per-employee-processed metering |
| CFO / Controller | GL with subsidiary roll-up, intercompany elimination, period-close workflow, IFRS/K-IFRS/US-GAAP toggle, AP/AR ageing dashboards | Per-entity subscription |
| AP/AR Clerk | Invoice capture (OCR + Foundry extraction), three-way match, payment instruction authoring | Included in GL tier |
| Corporate IT / Tenant Builder | Workflow Studio access, Object Graph property customization, SCIM provisioning, SSO config | Builder seat |
| Executive / Board | Consolidated group financials, headcount vs budget, ESG headcount metrics, AI-authored board packs | Analytics tier add-on |
| Regulator / Auditor (NTS, MoE, 4대보험) | Evidence portal, payroll tax returns, electronic labor contracts, period-close control evidence | Cost of doing business |

---

## 3. In-Scope / Out-of-Scope

### 3.1 In-scope at each wave

| Wave | Capabilities | Surfaces exposed |
|---|---|---|
| Vertical-Preview | HR employee record, org-chart, leave management, KR payroll engine (4대보험 + 근로소득세), GL chart-of-accounts, journal entry, period-close workflow, corporate mail (SMTP/IMAP bridge), SCIM provisioning | REST API v1, Workflow Studio, Web UI |
| Vertical-Stable | Global payroll pack (US W-2/W-4, EU Working Time Directive, JP 賞与, multi-currency payroll), AP/AR with 3-way match, intercompany elimination, multi-entity consolidation, IFRS 16 lease schedules, e-invoicing per regional pack (전자세금계산서, 適格請求書, EU-eInvoice), Foundry-authored payroll journals and GL reconciliation agents, mobile app | REST API v1 stable, Webhook console, Plugin SDK |
| Public-GA | ESG headcount reporting (GRI/SASB/TCFD), board-pack generation, treasury management basics, corporate credit-card reconciliation, global entity directory (600+ jurisdictions), Foundry-driven period-close autonomous loop | Public OpenAPI, Analytics dashboards, Advertiser exclusion (no ads on this surface) |
| Region-Fan-Out | Per-regional-pack payroll engine, local tax authorities integration, local social-security rails | Per-pack launch cadence |

### 3.2 Out-of-scope (anti-scope)

- Trading / investment management (treasury extends to cash pooling only; derivatives are out-of-scope until council decision)
- Full ERP manufacturing modules (COGS/BOM/MRP — that is the Industrial vertical)
- Benefits insurance carrier integration at carrier-network depth (benefits *administration* is in-scope; carrier *underwriting* is not)
- Consumer payroll (gig-worker self-filing apps) — corporate B2B only
- Advertising targeting using employee or payroll data — always blocked (PRIVACY-PROGRAM §2.2.3 corporate default)
- Standalone HR SaaS without the GL/Payroll coupling — the coupling is the moat

---

## 4. Architecture Overview

### 4.1 Bounded Context

Axis 2 — Vertical Corporate. Flat-crates target prefix: `crates/oya-vertical-corporate-*`.

The corporate vertical owns the HR, Payroll, GL, Mail, and Communications bounded contexts. Cross-axis contracts consumed: `oya-platform-tenant-kernel` (tenancy), `oya-platform-identity-kernel` (SSO/RBAC), `oya-platform-audit-chain-kernel` (audit emission), `oya-foundry-api` (Foundry capability invocations), `oya-platform-billing-tax-kernel` (tax-invoice formatter seam), `oya-platform-regulatory-kernel` (regulatory pack seam).

### 4.2 Layered Structure

```
crates/oya-vertical-corporate-kernel-hr/        — Employee, OrgUnit, Position, LeavePolicy entities; no I/O
crates/oya-vertical-corporate-kernel-payroll/   — PayrollRun, PayslipLine, StatutoryDeduction, TaxTable entities
crates/oya-vertical-corporate-kernel-gl/        — Account, JournalEntry, LedgerPeriod, CostCenter, Entity entities
crates/oya-vertical-corporate-kernel-mail/      — Mailbox, Message, Thread, Folder, MailPolicy entities
crates/oya-vertical-corporate-kernel-comms/     — Channel, Post, Mention, Notification entities
crates/oya-vertical-corporate-domain-hr/        — HR use cases: hire, terminate, transfer, leave-request, org-chart publish
crates/oya-vertical-corporate-domain-payroll/   — Payroll use cases: run-payroll, compute-deductions, approve-payslip, file-return
crates/oya-vertical-corporate-domain-gl/        — GL use cases: post-journal, close-period, consolidate, AP-3-way-match
crates/oya-vertical-corporate-domain-mail/      — Mail delivery, threading, policy-gate use cases
crates/oya-vertical-corporate-app-payroll/      — Payroll saga orchestration, Foundry capability delegation
crates/oya-vertical-corporate-app-gl/           — Period-close saga, intercompany elimination orchestration
crates/oya-vertical-corporate-adapter-db/       — Postgres/TimescaleDB adapters for all sub-domains
crates/oya-vertical-corporate-adapter-smtp/     — SMTP/IMAP bridge adapter
crates/oya-vertical-corporate-adapter-taxauth/  — Regional tax authority API adapters (NTS KR, IRS US, etc.)
crates/oya-vertical-corporate-api-rest/         — Inbound REST handlers for all sub-domains
crates/oya-vertical-corporate-worker-events/    — Kafka consumers (payroll-run-requested, journal-posted, etc.)
crates/oya-vertical-corporate-runtime/          — Composition root binary
```

### 4.3 External-Facing Surfaces

| Surface | Contract location | Plane | SLO target |
|---|---|---|---|
| HR REST API | `contracts/corporate-hr.openapi.yaml` | Data | 99.9% / p95 < 200ms |
| Payroll REST API | `contracts/corporate-payroll.openapi.yaml` | Data | 99.9% / p95 < 500ms |
| GL REST API | `contracts/corporate-gl.openapi.yaml` | Data | 99.9% / p95 < 300ms |
| Mail API (SMTP/IMAP bridge + REST) | `contracts/corporate-mail.openapi.yaml` | Data | 99.95% / p95 < 100ms |
| Webhook events (payslip-issued, journal-posted) | `contracts/corporate-webhooks.yaml` | Data | at-least-once, ≤ 30s |
| Analytics dashboard | internal projection API | Analytics | best-effort |

### 4.4 Internal Seams

| Seam | Trait / interface | Consumer products |
|---|---|---|
| `PayrollJournalSource` | `PayrollJournalProvider` trait | GL domain (auto-post payroll to GL) |
| `EmployeeIdentitySync` | `IdentitySync` trait | Platform identity kernel (SCIM out) |
| `CorporateMailContent` | `MailboxSearchable` trait | Search axis (tenant-private mailbox search) |
| `PeriodCloseEvidence` | `AuditChainEmitter` trait | Audit chain (control evidence) |

### 4.5 Dependencies on Other Axes

| Contract consumed | Owner axis | Where it lives | Change-review class |
|---|---|---|---|
| `Tenant` kernel | SaaS platform | `oya-platform-tenant-kernel` | Cross-axis review |
| `Identity / Cedar policy` | SaaS platform | `oya-platform-identity-kernel` | Cross-axis + security |
| `Capability invocation` | Foundry | `oya-foundry-api` | Foundry + corporate review |
| `Audit-chain event` | Platform | `oya-platform-audit-chain-kernel` | Audit review |
| `TaxInvoiceFormatter` seam | Platform billing | `oya-platform-billing-tax-kernel` | Billing + regional-pack review |
| `RegulatoryPack` seam | Platform regulatory | `oya-platform-regulatory-kernel` | Regulatory + vertical review |
| `PaymentRail` seam | Regional pack | `oya-saas-billing-rail-kernel` | Rail + regional review |

---

## 5. Data Structures

### 5.1 Kernel Entities

```rust
// crates/oya-vertical-corporate-kernel-hr

/// data_class: PII_IDENTIFYING (name, RRN fields); PII_QUASI_IDENTIFIER (birthdate, gender)
/// plane: data
pub struct Employee {
    pub id: EmployeeId,                        // data_class: INTERNAL_ONLY
    pub tenant_id: TenantId,                   // data_class: INTERNAL_ONLY
    pub region: RegionCode,                    // data_class: INTERNAL_ONLY
    pub schema_version: u32,
    pub legal_name: PersonName,                // data_class: PII_IDENTIFYING
    pub display_name: String,                  // data_class: PII_IDENTIFYING
    pub national_id: Option<NationalId>,       // data_class: PII_IDENTIFYING (RRN in KR, SSN in US, etc.)
    pub date_of_birth: Option<NaiveDate>,      // data_class: PII_QUASI_IDENTIFIER
    pub gender: Option<Gender>,                // data_class: PII_QUASI_IDENTIFIER
    pub employment_status: EmploymentStatus,   // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub employment_type: EmploymentType,       // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub hire_date: NaiveDate,                  // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub termination_date: Option<NaiveDate>,   // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub position_id: PositionId,               // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub org_unit_id: OrgUnitId,                // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub manager_id: Option<EmployeeId>,        // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub contact_email: Email,                  // data_class: PII_IDENTIFYING
    pub contact_phone: Option<Phone>,          // data_class: PII_IDENTIFYING
    pub bank_account: Option<EncryptedBankAccount>, // data_class: PCI (encrypted at rest, KMS-shredded on DSR)
    pub local_extensions: serde_json::Value,   // data_class: per-field annotations in regional pack schema
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum EmploymentStatus { Active, OnLeave, Terminated, Suspended }
pub enum EmploymentType { FullTime, PartTime, Contract, Intern, Secondment }

/// data_class: BEHAVIORAL_TENANT_PRODUCT
/// plane: control
pub struct OrgUnit {
    pub id: OrgUnitId,
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub name: String,                         // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub code: String,                         // data_class: INTERNAL_ONLY
    pub parent_id: Option<OrgUnitId>,         // data_class: INTERNAL_ONLY
    pub cost_center_id: Option<CostCenterId>, // data_class: INTERNAL_ONLY
    pub head_count_budget: Option<u32>,       // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

```rust
// crates/oya-vertical-corporate-kernel-payroll

/// data_class: FINANCIAL_KR_신용정보 (KR bank/wage data); PCI (bank account refs)
/// plane: data
pub struct PayrollRun {
    pub id: PayrollRunId,
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub pay_period: PayPeriod,                // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub entity_id: EntityId,                  // data_class: INTERNAL_ONLY
    pub status: PayrollRunStatus,             // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub currency: CurrencyCode,               // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub total_gross: Money,                   // data_class: FINANCIAL_KR_신용정보 (KR) / BEHAVIORAL_TENANT_PRODUCT (other)
    pub total_net: Money,                     // data_class: FINANCIAL_KR_신용정보 (KR)
    pub total_employer_contributions: Money,  // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub regulatory_pack_id: RegulatoryPackId, // data_class: INTERNAL_ONLY
    pub foundry_run_id: Option<FoundryRunId>, // data_class: INTERNAL_ONLY (Foundry job that authored journals)
    pub approved_by: Option<UserId>,          // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub approved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum PayrollRunStatus { Draft, Computed, PendingApproval, Approved, Disbursed, Filed }

/// data_class: FINANCIAL_KR_신용정보 (wage breakdown); PII_IDENTIFYING (employee ref)
/// plane: data
pub struct PayslipLine {
    pub id: PayslipLineId,
    pub payroll_run_id: PayrollRunId,
    pub employee_id: EmployeeId,              // data_class: PII_IDENTIFYING
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub line_type: PayslipLineType,           // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub description: String,                  // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub amount: Money,                        // data_class: FINANCIAL_KR_신용정보 (KR)
    pub statutory_code: Option<StatutoryCode>,// data_class: INTERNAL_ONLY (links to regulatory pack)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum PayslipLineType {
    BasicWage, Allowance, Overtime, Bonus,
    IncomeTaxWithholding, NationalPension, HealthInsurance,
    EmploymentInsurance, WorkplaceSafety, EmployerNationalPension,
    NetPay,
}
```

```rust
// crates/oya-vertical-corporate-kernel-gl

/// data_class: BEHAVIORAL_TENANT_PRODUCT (accounting amounts are not PHI/PCI)
/// plane: data
pub struct JournalEntry {
    pub id: JournalEntryId,
    pub tenant_id: TenantId,
    pub entity_id: EntityId,                  // legal entity within the group
    pub region: RegionCode,
    pub schema_version: u32,
    pub period: LedgerPeriod,                 // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub reference: String,                    // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub description: String,                  // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub currency: CurrencyCode,
    pub lines: Vec<JournalLine>,              // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub status: JournalStatus,
    pub source: JournalSource,                // Payroll, AP, AR, Manual, Foundry
    pub foundry_run_id: Option<FoundryRunId>, // data_class: INTERNAL_ONLY
    pub audit_ref: AuditChainRef,             // data_class: INTERNAL_ONLY
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct JournalLine {
    pub account_id: AccountId,               // data_class: INTERNAL_ONLY
    pub cost_center_id: Option<CostCenterId>,// data_class: INTERNAL_ONLY
    pub debit: Option<Money>,                // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub credit: Option<Money>,               // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub description: String,                 // data_class: BEHAVIORAL_TENANT_PRODUCT
}

pub enum JournalStatus { Draft, Posted, Reversed }
pub enum JournalSource { Payroll, AP, AR, Manual, Foundry, Migration }

/// plane: control
pub struct Account {
    pub id: AccountId,
    pub tenant_id: TenantId,
    pub entity_id: EntityId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub code: String,                        // data_class: INTERNAL_ONLY
    pub name: String,                        // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub account_type: AccountType,           // Asset, Liability, Equity, Revenue, Expense
    pub normal_balance: NormalBalance,       // Debit / Credit
    pub currency: CurrencyCode,
    pub is_intercompany: bool,
    pub is_control_account: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

```rust
// crates/oya-vertical-corporate-kernel-mail

/// data_class: PII_IDENTIFYING (from/to); BEHAVIORAL_TENANT_PRODUCT (body content)
/// plane: data
pub struct Message {
    pub id: MessageId,
    pub tenant_id: TenantId,
    pub mailbox_id: MailboxId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub thread_id: ThreadId,
    pub from: EmailAddress,                  // data_class: PII_IDENTIFYING
    pub to: Vec<EmailAddress>,               // data_class: PII_IDENTIFYING
    pub cc: Vec<EmailAddress>,               // data_class: PII_IDENTIFYING
    pub subject: String,                     // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub body_text: Option<EncryptedBlob>,    // data_class: BEHAVIORAL_TENANT_PRODUCT (encrypted at rest)
    pub body_html: Option<EncryptedBlob>,    // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub attachments: Vec<AttachmentRef>,     // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub headers: Vec<MailHeader>,            // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub received_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### 5.2 Aggregate Boundaries

| Aggregate | Root entity | Consistency boundary |
|---|---|---|
| `EmployeeAggregate` | `Employee` | All HR state per employee; leave balances are inside this aggregate |
| `OrgUnitAggregate` | `OrgUnit` | Org-chart subtree; reassignment crosses aggregate via domain event |
| `PayrollRunAggregate` | `PayrollRun` + `PayslipLine[]` | Entire payroll run for one entity/period; lines are part of the same aggregate |
| `LedgerPeriodAggregate` | `LedgerPeriod` + `JournalEntry[]` | A closed period is immutable; cross-entity via intercompany event |
| `MailboxAggregate` | `Mailbox` + `Message[]` + `Thread[]` | Per-user or per-group mailbox; folder structure is inside |

### 5.3 Persistence Layout

| Aggregate | Store | Sharding key | Partition strategy | Replication | Retention |
|---|---|---|---|---|---|
| Employee | Postgres (per-tenant shard) | `tenant_id` | Per-tenant schema | Streaming replication × 2 | 7 years (KR 근로기준법) |
| PayrollRun + PayslipLine | Postgres (per-tenant shard) | `tenant_id` | Per-tenant schema | Streaming replication × 2 | 7 years (KR 소득세법) |
| JournalEntry | Postgres (per-entity shard) | `(tenant_id, entity_id)` | Per-entity schema | Streaming replication × 2 | 10 years (K-IFRS / US-GAAP) |
| Message | Postgres + Object Store (body blobs) | `(tenant_id, mailbox_id)` | Shuffle sharding | Streaming replication × 2 | Per-tenant retention policy (default 5 years) |
| OrgUnit / Account | Postgres (per-tenant shard) | `tenant_id` | Per-tenant schema | Streaming replication × 2 | Indefinite (active) + 7 years (archived) |

### 5.4 Event Schemas

| Event name | Topic | Schema location | Consumer aggregates | Retention | Idempotency key |
|---|---|---|---|---|---|
| `PayrollRunApproved` | `corporate.payroll.run.approved` | `contracts/events/corporate-payroll.json` | GL (auto-post journals), Audit chain | 30 days | `payroll_run_id` |
| `PayslipIssued` | `corporate.payroll.payslip.issued` | `contracts/events/corporate-payroll.json` | Employee notification, Audit chain | 30 days | `payslip_line_id` |
| `JournalPosted` | `corporate.gl.journal.posted` | `contracts/events/corporate-gl.json` | Analytics projection, Audit chain | 30 days | `journal_entry_id` |
| `PeriodClosed` | `corporate.gl.period.closed` | `contracts/events/corporate-gl.json` | Consolidation aggregate, Audit chain | 90 days | `(entity_id, period)` |
| `EmployeeHired` | `corporate.hr.employee.hired` | `contracts/events/corporate-hr.json` | Identity SCIM sync, Mailbox provision, Audit chain | 30 days | `employee_id` |
| `EmployeeTerminated` | `corporate.hr.employee.terminated` | `contracts/events/corporate-hr.json` | Identity de-provision, Mailbox archive, Payroll cut-off, Audit chain | 90 days | `employee_id` |
| `MessageReceived` | `corporate.mail.message.received` | `contracts/events/corporate-mail.json` | Search index (tenant-private), Notification | 7 days | `message_id` |

### 5.5 Index / Search-Index Touchpoints

| Entity field | Index | Class allowed | Cascade-on-DSR? |
|---|---|---|---|
| `Message.subject` | tenant-private search index | `BEHAVIORAL_TENANT_PRODUCT` (tenant-searchable only) | Yes — DSR cascade deletes index entries |
| `Employee.display_name` | tenant-private directory | `PII_IDENTIFYING` (tenant-searchable only, never cross-tenant) | Yes |
| `JournalEntry.description` | tenant-private GL search | `BEHAVIORAL_TENANT_PRODUCT` | No (financial records retained per regulatory requirement) |

### 5.6 Audit-Chain Emission Contract

| Operation | Emits topic | Required fields |
|---|---|---|
| Payroll run approved | `audit.corporate.payroll.approved` | `payroll_run_id`, `approved_by`, `entity_id`, `period`, `total_gross`, `regulatory_pack_id` |
| Payslip line computed | `audit.corporate.payslip.computed` | `employee_id` (pseudonymized), `line_type`, `amount`, `statutory_code` |
| GL period closed | `audit.corporate.gl.period_closed` | `entity_id`, `period`, `closed_by`, `trial_balance_hash` |
| Employee data exported | `audit.corporate.hr.data_exported` | `employee_id`, `exported_by`, `data_classes`, `export_format`, `dsr_ref` |
| Break-glass access | `audit.corporate.break_glass` | `accessor_id`, `employee_id`, `reason`, `duration` |
| Tax return filed | `audit.corporate.payroll.tax_filed` | `filing_ref`, `tax_authority_id`, `period`, `amount`, `regulatory_pack_id` |

### 5.7 Schema Migration Policy

- All migrations via Postgres `flyway`-style versioned scripts in `crates/oya-vertical-corporate-adapter-db/migrations/`.
- Every migration is reversible (down-migration required) until the period is closed.
- Dry-run gate: migration must succeed on staging replica before landing on main.
- Payroll schema changes require audit-chain evidence of zero data loss on staging.

---

## 6. Optimization Practices

| Practice | Implementation choice |
|---|---|
| Cell routing | `tenant_id` → cell assignment; holding-group multi-entity tenants route all entities to the same cell unless tenant opts into cross-cell for DR |
| Sharding strategy | Per-tenant Postgres schema; per-entity GL shard within a tenant; per-mailbox shuffle sharding for mail |
| Caching tier | In-memory LRU for chart-of-accounts (low-churn); Redis for payroll computation intermediate state; CDN for static GL report PDFs |
| Bulk endpoint contract | `POST /payroll/runs/{id}/compute-bulk` (batch employee payslip computation); `POST /gl/journals/bulk` (mass journal import) |
| Pagination | Cursor-based on `(created_at, id)` for all list endpoints; max page 200 items; filter by `period`, `entity_id`, `status` |
| Idempotency | `Idempotency-Key` header on all payroll-run and journal-post mutations; 24-hour dedup window |
| Batch dispatch | Payroll run computation is dispatched as a batch of per-employee Foundry capability invocations; GL consolidation as a Foundry batch saga |
| Backpressure | Worker pool for payroll computation reads `RUNNING` count from Postgres; halts intake if > threshold; Kafka consumer lag monitored |
| Hot-path benchmarks | `payslip_compute` criterion benchmark gate: < 50ms per employee at P99; `journal_post` < 20ms |
| Agent-driven optimization | Foundry `PayrollReconciler` capability runs nightly, reconciling bank disbursements against payslip net amounts; Foundry `GLPeriodCloser` drives period-close checklist |
| FinOps unit-economics | Per-employee-processed payroll metering; per-entity GL metering; mail storage per-GB-month; Foundry capability invocations metered separately |
| Build-cache / CI affected-graph | `oya-vertical-corporate-kernel-*` → affected graph touches `domain-*`, `app-*`, `adapter-*`, `api-*`, `worker-*`, `runtime` |

---

## 7. Regional Pack Interactions

| Seam | Trait | Per-pack impl needed? | Tested with which packs? |
|---|---|---|---|
| Payroll statutory deduction computation | `StatutoryDeductionEngine` | Yes — every payroll-capable region needs a deduction pack | `oya-pack-kr` (4대보험 + 근로소득세), `oya-pack-us` (FICA + Federal/State withholding), `oya-pack-jp` (社会保険 + 所得税), `oya-pack-eu` (country-level) |
| Tax-invoice formatter | `TaxInvoiceFormatter` | Yes | `oya-pack-kr` (전자세금계산서 + NTS API), `oya-pack-jp` (適格請求書), `oya-pack-eu` (Factur-X / Peppol BIS) |
| Payment-rails adapter | `PaymentRail` | Yes — payroll disbursement + AP payment | `oya-pack-kr` (계좌이체 / 카카오페이), `oya-pack-us` (NACHA ACH), `oya-pack-jp` (総合振込), `oya-pack-eu` (SEPA Credit Transfer) |
| Identity-provider adapter | `IdentityProvider` | Yes — employee SSO | `oya-pack-kr` (본인확인서비스), `oya-pack-us` (Login.gov for public entities; any OIDC for private) |
| Regulatory control evidence | `RegulatoryPack` | Yes | `oya-pack-kr` (NTS, NHIS, NLIS, MoE), `oya-pack-us` (IRS, SSA, DOL), `oya-pack-eu` (per-country) |
| Industry data model — labor classification | `LocalIndustryExtension` (HR kernel) | Yes | `oya-pack-kr` (통상임금, 연차수당, 퇴직금), `oya-pack-us` (FLSA exempt/nonexempt, 1099/W-2), `oya-pack-jp` (賞与, 年次有給休暇) |
| Calendar / fiscal year | `LocaleFormatter` | Yes | All onboarded packs |
| Corporate mail content safety | `ContentSafetyRules` | Yes (KR 정보통신망법) | `oya-pack-kr`, `oya-pack-eu` (DSA) |

### Regulatory Pack Declaration

```yaml
# registry/catalog/oya-vertical-corporate-runtime.yaml
regulatory_packs:
  - oya-pack-kr   # PIPA, 근로기준법, 소득세법, NTS, NHIS, NLIS
  - oya-pack-us   # FLSA, IRS, SSA, CCPA/CPRA
  - oya-pack-jp   # APPI, 労働基準法, 社会保険
  - oya-pack-eu   # GDPR, EU Working Time Directive, per-country tax
```

---

## 8. In-House vs External Dependency Posture

| External dep | Maturity tier | License | In-house alternative considered? | Decision |
|---|---|---|---|---|
| `tokio` (async runtime) | kernel-grade | MIT | No — OS-level quality | Use |
| `axum` (HTTP server) | kernel-grade | MIT | No | Use |
| `sqlx` (Postgres driver) | kernel-grade | MIT / Apache-2 | No | Use |
| `serde` + `serde_json` | kernel-grade | MIT / Apache-2 | No | Use |
| `rustls` (TLS) | kernel-grade | Apache-2 / MIT / ISC | No | Use |
| `lettre` (SMTP client) | stable | MIT / Apache-2 | Evaluated in-house SMTP — lettre sufficient | Use; ADR pending |
| `calamine` (Excel parsing for payroll upload) | stable | MIT / Apache-2 | In-house considered — calamine well-maintained | Use |
| `chrono` / `time` (date/time) | kernel-grade | MIT / Apache-2 | No | Use |
| `rust_decimal` (money arithmetic) | stable | MIT | In-house considered — rust_decimal covers all monetary rounding modes | Use |
| `imap` (IMAP client for mail bridge) | mature | MIT | In-house bridge possible — IMAP crate sufficient for Phase 1 | Use; revisit at stable |
| NTS e-세금계산서 API client | KR-specific | Proprietary API (no code dependency) | In-house HTTP client wrapping NTS OpenAPI | Build in-house via `oya-vertical-corporate-adapter-taxauth` |
| `lopdf` (PDF generation for payslips) | stable | MIT | In-house template renderer considered | Use lopdf for now; revisit |

---

## 9. Success Metrics

| Metric | Vertical-Preview target | Vertical-Stable target | Public-GA target |
|---|---|---|---|
| Design-partner tenants live on payroll | ≥ 1 (KR Group anchor) | ≥ 3 holding groups | ≥ 20 enterprise tenants |
| Payroll run P99 compute time (per employee) | < 200ms | < 100ms | < 50ms |
| GL period-close cycle time (median) | < 5 business days (manual assist) | < 2 business days | < 1 business day (Foundry-driven) |
| Statutory deduction accuracy (vs manual recompute) | 100% match | 100% match | 100% match |
| Audit-chain emission completeness | 100% of payroll + GL events | 100% | 100% |
| Foundry capability runs (payroll + GL agents) | ≥ 100/week | ≥ 1,000/week | ≥ 10,000/week |
| SCIM provisioning latency (hire→access) | < 30 min | < 5 min | < 1 min |
| DSR cascade completion time | < 72 hours | < 30 hours | < 24 hours |
| Mail delivery SLO (intra-tenant) | 99.5% | 99.9% | 99.95% |
| Cross-axis contract violations | 0 | 0 | 0 |
| Foundation-bypass count (trend) | decreasing | 0 new | 0 |

---

## 10. Risks + Mitigations

| Risk | Severity | Mitigation | Owner |
|---|---|---|---|
| Statutory deduction computation error (miscalculated wages → regulatory penalty) | Critical | Dual computation path: Foundry agent + deterministic in-house engine; results compared before approval; immutable audit trail; NTS API confirmation | Payroll domain + Regulatory |
| Employee PII leak into search/ads axis | Catastrophic | `PII_IDENTIFYING` data_class blocks search-index ingestion and ads targeting structurally (PRIVACY-PROGRAM §2.2.3); SCIM sync uses pseudonymized IDs | Privacy + Architecture |
| Bank account data (PCI) leak | Catastrophic | `oya-vertical-corporate-kernel-payroll` stores bank accounts as KMS-encrypted blobs; DEK per tenant; KMS shred on DSR | Security + KMS |
| Payroll run race condition (double disbursal) | High | Idempotency-key on disbursement API; outbox pattern for PayrollRunDisbursed event; bank ACK checked before status flip | Payroll domain |
| Period-close regression (GL doesn't balance) | High | Trial balance assertion on every period-close saga step; automated reconciliation report by Foundry GLPeriodCloser | GL domain + Foundry |
| Korean labor law change (통상임금 scope change, 주52시간 amendments) | High | Regulatory-change watch lane (ADR-0050); KR pack versioned; affected payroll runs flagged for re-computation | Regulatory + KR pack |
| Mail content retention violates GDPR / PIPA | Medium | Per-tenant retention policy enforced by TTL on Object Store; DSR cascade deletes mail blobs with proof-of-erasure | Privacy + Mail domain |
| Intercompany elimination failure at consolidation | Medium | LedgerPeriodAggregate enforces that intercompany journal pairs sum to zero before close; Foundry consolidation agent validates | GL domain |
| Large holding group onboarding performance (100+ entities) | Medium | Per-entity GL shard; batch onboarding saga; Foundry-driven COA migration agent | Corporate ops + Foundry |
| Foundry GL autonomy ceiling exceeded (agent posts unauthorized journal) | High | Cedar policy gates `journal.post` capability at autonomy tier T2; human approval required for amounts > threshold | Foundry + Cedar |

---

## 11. Open Questions

- Council decision needed: which subsidiary structure is the canonical "KR Group anchor" tenant for Vertical-Preview — flat subsidiary list or multi-level holding?
- Tax treaty handling for intercompany transfer pricing — scope of GL vs. separate tax module?
- KR 퇴직연금 (severance pension) — IRP vs. DB/DC scheme selection per employee: in-scope for Vertical-Preview or deferred to Stable?
- Mail encryption at rest: per-message DEK vs. per-mailbox DEK? Trade-off between DSR granularity and key management overhead.
- Corporate communications (채팅) — Slack-parity feature set or headliner-only for Vertical-Preview?

---

## 12. Decision Log

| Decision | Date | Rationale | ADR ref |
|---|---|---|---|
| KR Group as anchor design-partner tenant | 2026-05-09 | Deepest existing corporate domain knowledge; KR pack first | — |
| Payroll and GL in same vertical bounded context | 2026-05-09 | Payroll-to-GL journal is a first-class integration seam; unbundling creates integration tax | — |
| Mail via SMTP/IMAP bridge (not proprietary protocol) | 2026-05-09 | Corporate mail interoperability with external systems required; bridge pattern preserves correctness | — |
| Flat-crates target: `crates/oya-vertical-corporate-*` | 2026-05-09 | Per ADR-0015 flat-crates mandate | ADR-0015 |
| Statutory deduction via regional pack, not hardcoded | 2026-05-09 | Global fan-out requires pluggable deduction engine; KR hardcoding would block US/JP/EU | DESIGN.md §12 |

---

## 13. Sources Scanned

- `docs/PRD.md` — north star, wave sequencing, cohesion thesis
- `docs/DESIGN.md` §1, §4, §10, §12 — bounded context, regional pack architecture
- `docs/PRIVACY-PROGRAM.md` §2.2.1, §2.2.3 — data class taxonomy, tenant-class overrides
- `docs/GLOSSARY.md` §7 — regulatory terms
- ADR-0015 (flat crates), ADR-0003 (audit chain), ADR-0017 (plane separation), ADR-0050 (AI/ML governance)

---

## Doc-Catalog Row

```
| `vertical-corporate` | `vertical-2` | HR/payroll/GL/mail/comms; KR Group anchor + global | monthly | PRD.md, DESIGN.md §12, PRIVACY-PROGRAM.md §2.2.3, GLOSSARY.md §7 |
```
