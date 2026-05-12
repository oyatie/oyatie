# Oyatie — Product PRD: Vertical Fintech

> **Status:** preview
> **Owning team:** [`teams/vertical-fintech/CHARTER.md`](../../teams/vertical-fintech/CHARTER.md)
> **Owning axis:** vertical-fintech (Axis 2)
> **Catalog reference:** `registry/catalog/oya-vertical-fintech-*.yaml`
> **Last updated:** 2026-05-09 by architecture-council

---

## 1. North Star

Oyatie Vertical Fintech is a regulated financial-services operations platform covering payment gateway (PG), open banking (account aggregation + payment initiation), KYC/KYB identity verification, AML transaction monitoring, and multi-rail payment execution — with per-region payment rail implementations supplied by regional packs (KR: 카카오페이/네이버페이/토스/계좌이체/KFTC; US: NACHA ACH/FedNow/RTP; EU: SEPA/SEPA-Inst; IN: UPI; BR: Pix; etc.). The canonical entity model is ISO 20022-aligned (PaymentInstruction, CreditTransfer, DirectDebit, AccountStatement, KycRecord, AmlAlert). The product exists within Oyatie's ecosystem — and cannot be separated from it — because the coupling of fintech workflow execution with Foundry-driven AML monitoring agents (operating under an elevated autonomy ceiling with mandatory evidence emission), the audit chain providing tamper-evident payment records, the privacy program that structurally blocks PCI/신용정보 from ad targeting at compile time, and the Corporate vertical's GL for cash-position and reconciliation is the regulatory moat that no standalone PG or BaaS provider can replicate at the same compliance depth.

---

## 2. Target Users

| Persona | What they get | What they pay for |
|---|---|---|
| Merchant / Tenant Payment Operator | Payment gateway integration (PG), payment link, recurring billing, payout management | Per-transaction metering + monthly base |
| CFO / Treasury | Multi-rail payment execution, cash position dashboard, FX exposure, sweep rules | Per-seat (treasury tier) |
| Compliance Officer | KYC/KYB workflow, AML alert queue, SAR filing, regulatory reporting, evidence portal | Per-seat (compliance tier) |
| Open Banking Developer (ISV) | Account aggregation APIs (FHIR-for-banking analogy: open-banking data model), payment initiation | Per-API-call metering |
| Risk Analyst | AML rule authoring, transaction monitoring dashboard, Foundry-assisted risk scoring | Per-seat (risk tier) |
| Fintech IT / Tenant Builder | Payment rail configuration, KYC provider adapter config, Foundry AML agent workflow authoring | Builder seat |
| Regulator / Auditor (FSC KR, OCC US, FCA UK, RBI IN) | SAR filing records, AML evidence, KYC audit trail, payment record immutability evidence | Cost of doing business |

---

## 3. In-Scope / Out-of-Scope

### 3.1 In-scope at each wave

| Wave | Capabilities | Surfaces exposed |
|---|---|---|
| Vertical-Preview | Payment gateway (PG) with KR rails (카카오페이/네이버페이/토스/계좌이체), basic KYC (identity document verification + liveness check via KR 본인확인서비스), AML transaction screening (sanction list check via OFAC + KR 금융정보분석원 FIU), ISO 20022 CreditTransfer payment instruction | REST API v1, PG checkout SDK (JS), Web UI |
| Vertical-Stable | Open banking (account aggregation per KR 마이데이터 + EU PSD2 + US Dodd-Frank 1033 draft), payment initiation (SEPA-Inst, NACHA ACH, FedNow, UPI, Pix), KYB (business entity verification via KR NTS + global), full AML rule engine with Foundry-assisted risk scoring, SAR filing workflow, multi-currency settlement, recurring billing / subscription management, dispute management | REST API stable, Open banking SDK, Webhook console |
| Public-GA | Cross-border payment orchestration (ISO 20022 pacs.008/pacs.004), embedded lending workflow (BNPL — buy-now-pay-later data model), crypto asset reconciliation (declared as seam; not directly brokered), Foundry autonomous AML triage (T2 — flag + freeze, compliance officer approves SAR), FX hedging quote (declare seam; broker API adapter per regional pack), instant payment settlement dashboard | Public OpenAPI, Embedded checkout SDK, Analytics |
| Region-Fan-Out | Per-regional-pack payment rails, local KYC identity providers, local AML regulatory filing, local open-banking schema | Per-pack launch cadence |

### 3.2 Out-of-scope (anti-scope)

- Bank charter / deposit-taking / lending origination at full banking-license depth (tenant must hold the banking license; Oyatie is the technology layer)
- Insurance underwriting / actuarial risk pricing
- Crypto exchange / spot trading (crypto asset reconciliation seam declared; no exchange or custody)
- Advertising targeting using PCI or 신용정보 data — **always and permanently blocked** (PRIVACY-PROGRAM §2.2.3 fintech override; `PCI` and `FINANCIAL_KR_신용정보` are `HARD DENY` classes)
- Cross-tenant sharing of individual payment or account data — no exceptions
- Autonomous payment execution above the autonomy ceiling without human-in-loop approval (T3 required for auto-payment execution; not granted at Preview or Stable)

---

## 4. Architecture Overview

### 4.1 Bounded Context

Axis 2 — Vertical Fintech. Flat-crates target prefix: `crates/oya-vertical-fintech-*`.

The fintech vertical owns the payment, open-banking, KYC/KYB, AML, and settlement bounded contexts. Cross-axis contracts: `oya-platform-tenant-kernel` (with `ad_targetable_blocked` forced for PCI/신용정보), `oya-platform-audit-chain-kernel` (immutable payment records + SAR audit), `oya-foundry-api` (AML risk scoring + SAR draft agents), `oya-saas-billing-rail-kernel` (payment rail seam), `oya-vertical-corporate-domain-gl` (settlement posting seam), `oya-platform-regulatory-kernel` (FSC/OCC/FCA/RBI packs).

### 4.2 Layered Structure

```
crates/oya-vertical-fintech-kernel-payment/        — PaymentInstruction, CreditTransfer, DirectDebit, PaymentStatus (ISO 20022 aligned)
crates/oya-vertical-fintech-kernel-account/        — Account, AccountStatement, Balance, Transaction (open banking aligned)
crates/oya-vertical-fintech-kernel-kyc/            — KycRecord, KycDocument, KycCheck, BiometricRef, KybRecord entities
crates/oya-vertical-fintech-kernel-aml/            — AmlAlert, AmlRule, SarRecord, SanctionMatch, TransactionRiskScore entities
crates/oya-vertical-fintech-kernel-settlement/     — SettlementBatch, SettlementEntry, NettingGroup entities
crates/oya-vertical-fintech-domain-payment/        — Payment initiation, status tracking, retry, refund use cases
crates/oya-vertical-fintech-domain-openbanking/    — Account aggregation, consent management, payment initiation (PIS) use cases
crates/oya-vertical-fintech-domain-kyc/            — KYC/KYB workflow use cases: initiate, verify, approve, periodic review
crates/oya-vertical-fintech-domain-aml/            — AML screening, rule evaluation, alert triage, SAR filing use cases
crates/oya-vertical-fintech-app-payment/           — Payment saga (acquire → route → settle); Foundry AML check delegation
crates/oya-vertical-fintech-app-kyc/               — KYC/KYB saga (document collection → verification → approval)
crates/oya-vertical-fintech-app-aml/               — AML monitoring saga; Foundry risk-scoring capability delegation
crates/oya-vertical-fintech-adapter-db/            — Postgres adapters (PCI schema; per-account shard)
crates/oya-vertical-fintech-adapter-rails/         — Per-rail adapters (KR KFTC, US NACHA/FedNow, EU SEPA, IN NPCI, BR BACEN)
crates/oya-vertical-fintech-adapter-kyc/           — KYC provider adapters (KR 본인확인서비스, Jumio, Onfido, etc.)
crates/oya-vertical-fintech-adapter-sanctions/     — Sanction list adapters (OFAC, UN, EU, KR FIU)
crates/oya-vertical-fintech-api-rest/              — REST API handlers
crates/oya-vertical-fintech-api-pg/                — Payment gateway checkout API (inbound merchant-facing)
crates/oya-vertical-fintech-worker-events/         — Kafka consumers (payment events, AML alert triggers)
crates/oya-vertical-fintech-runtime/               — Composition root binary
```

### 4.3 External-Facing Surfaces

| Surface | Contract location | Plane | SLO target |
|---|---|---|---|
| Fintech REST API (payment, KYC, AML) | `contracts/fintech-core.openapi.yaml` | Data | 99.95% / p95 < 200ms |
| PG checkout API (merchant-facing) | `contracts/fintech-pg.openapi.yaml` | Data | 99.99% / p95 < 300ms (payment authorization is P0) |
| Open banking API | `contracts/fintech-openbanking.openapi.yaml` | Data | 99.9% / p95 < 500ms |
| Webhook events (payment-completed, aml-alert) | `contracts/fintech-webhooks.yaml` | Data | at-least-once, ≤ 10s (payment SLO-critical) |
| SAR filing portal | internal portal (compliance officer only) | Control | 99.5% / p95 < 1s |
| Settlement dashboard | internal projection API | Analytics | best-effort |

### 4.4 Internal Seams

| Seam | Trait / interface | Consumer products |
|---|---|---|
| `SettlementGlPostable` | `GlCostPostable` trait | Corporate vertical GL (cash position, settlement entries) |
| `PaymentAuditEmitter` | `AuditChainEmitter` | Audit chain (immutable payment record, mandatory) |
| `AmlEvidenceEmitter` | `AuditChainEmitter` | Audit chain (SAR evidence, mandatory) |
| `KycStatusProvider` | `KycStatus` trait | Open banking (identity-gated account aggregation) |

### 4.5 Dependencies on Other Axes

| Contract consumed | Owner axis | Where it lives | Change-review class |
|---|---|---|---|
| `Tenant` kernel (with `ad_targetable_blocked` forced for PCI/신용정보) | SaaS platform | `oya-platform-tenant-kernel` | Cross-axis + privacy review |
| `Identity / Cedar policy` | SaaS platform | `oya-platform-identity-kernel` | Cross-axis + security |
| `Capability invocation` (AML risk scoring, SAR draft) | Foundry | `oya-foundry-api` | Foundry + fintech + compliance review |
| `Audit-chain event` (payment + SAR mandatory) | Platform | `oya-platform-audit-chain-kernel` | Audit review |
| `PaymentRail` seam | Regional pack | `oya-saas-billing-rail-kernel` | Rail + regional + fintech review |
| `RegulatoryPack` seam | Platform regulatory | `oya-platform-regulatory-kernel` | Regulatory + fintech review |
| `GlCostPostable` seam | Corporate vertical | `oya-vertical-corporate-domain-gl` | Cross-vertical review |

---

## 5. Data Structures

### 5.1 Kernel Entities

```rust
// crates/oya-vertical-fintech-kernel-payment
// ISO 20022 aligned — pacs.008 CreditTransfer / pacs.003 DirectDebit

/// data_class: PCI (card data refs encrypted); FINANCIAL_KR_신용정보 (KR account data)
/// plane: data
/// CRITICAL: ad_targetable_blocked = true (forced, cannot be raised)
pub struct PaymentInstruction {
    pub id: PaymentInstructionId,
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub instruction_type: PaymentInstructionType, // CreditTransfer, DirectDebit, InstantPayment
    pub amount: Money,                             // data_class: FINANCIAL_KR_신용정보 / PCI
    pub currency: CurrencyCode,
    pub debtor_account: EncryptedAccountRef,       // data_class: PCI / FINANCIAL_KR_신용정보 (KMS-encrypted)
    pub creditor_account: EncryptedAccountRef,     // data_class: PCI / FINANCIAL_KR_신용정보
    pub debtor_agent_bic: Option<String>,          // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub creditor_agent_bic: Option<String>,        // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub remittance_info: Option<String>,           // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub end_to_end_id: String,                     // ISO 20022 end-to-end identifier
    pub status: PaymentStatus,                     // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub rail_id: PaymentRailId,                    // which payment rail was selected
    pub rail_transaction_ref: Option<String>,      // rail's own transaction reference
    pub aml_check_status: AmlCheckStatus,          // data_class: INTERNAL_ONLY
    pub kyc_verified: bool,                        // data_class: INTERNAL_ONLY
    pub hot_key_token: Option<HotKeyToken>,        // data_class: INTERNAL_ONLY (hot-key smoothing)
    pub idempotency_key: IdempotencyKey,           // data_class: INTERNAL_ONLY
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum PaymentInstructionType {
    CreditTransfer, DirectDebit, InstantPayment, CardPayment, WalletPay, OpenBankingPIS
}
pub enum PaymentStatus {
    Pending, AmlPending, Authorized, Processing,
    Settled, Failed, Reversed, Cancelled
}
pub enum AmlCheckStatus { NotRequired, Pending, Cleared, Flagged, Blocked }
```

```rust
// crates/oya-vertical-fintech-kernel-kyc

/// data_class: PII_IDENTIFYING (name, DOB, national ID); PHI if health doc provided
/// plane: data (KYC workflow); control (KYC decision)
pub struct KycRecord {
    pub id: KycRecordId,
    pub tenant_id: TenantId,
    pub subject_type: KycSubjectType,              // Individual or Business
    pub region: RegionCode,
    pub schema_version: u32,
    pub legal_name: PersonName,                    // data_class: PII_IDENTIFYING
    pub date_of_birth: Option<NaiveDate>,          // data_class: PII_IDENTIFYING
    pub nationality: Option<CountryCode>,          // data_class: PII_QUASI_IDENTIFIER
    pub national_id: Option<NationalId>,           // data_class: PII_IDENTIFYING (RRN, SSN, etc.)
    pub address: Option<Address>,                  // data_class: PII_IDENTIFYING
    pub documents: Vec<KycDocumentRef>,            // data_class: PII_IDENTIFYING (doc references; blobs in object store KMS-encrypted)
    pub biometric_ref: Option<BiometricRef>,       // data_class: PII_IDENTIFYING (liveness check token)
    pub checks: Vec<KycCheck>,                     // data_class: INTERNAL_ONLY
    pub status: KycStatus,                         // data_class: INTERNAL_ONLY
    pub risk_level: KycRiskLevel,                  // data_class: INTERNAL_ONLY
    pub approved_by: Option<UserId>,               // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub valid_until: Option<NaiveDate>,            // KYC expiry date
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum KycSubjectType { Individual, Business }
pub enum KycStatus { Pending, InProgress, Approved, Rejected, Expired, PendingReview }
pub enum KycRiskLevel { Low, Medium, High, Prohibited }

pub struct KycCheck {
    pub check_type: KycCheckType,
    pub provider_id: KycProviderId,               // which adapter ran this check
    pub result: KycCheckResult,
    pub confidence_score: Option<f64>,
    pub run_at: DateTime<Utc>,
}
pub enum KycCheckType { DocumentVerification, FaceMatch, SanctionScreening, PepCheck, AdverseMedia }
pub enum KycCheckResult { Pass, Fail, ManualReview, Inconclusive }
```

```rust
// crates/oya-vertical-fintech-kernel-aml

/// data_class: FINANCIAL_KR_신용정보 (transaction data); INTERNAL_ONLY (risk scores)
/// plane: data (monitoring); control (SAR decision)
pub struct AmlAlert {
    pub id: AmlAlertId,
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub alert_type: AmlAlertType,
    pub severity: AmlAlertSeverity,
    pub payment_instruction_id: Option<PaymentInstructionId>, // data_class: FINANCIAL_KR_신용정보
    pub kyc_record_id: Option<KycRecordId>,                   // data_class: PII_IDENTIFYING
    pub triggered_rules: Vec<AmlRuleId>,                      // data_class: INTERNAL_ONLY
    pub foundry_risk_score: Option<f64>,                       // data_class: INTERNAL_ONLY (Foundry-computed)
    pub foundry_run_id: Option<FoundryRunId>,                  // data_class: INTERNAL_ONLY
    pub status: AmlAlertStatus,                                // data_class: INTERNAL_ONLY
    pub disposition: Option<AmlDisposition>,                   // data_class: INTERNAL_ONLY
    pub sar_ref: Option<SarRecordId>,                          // data_class: INTERNAL_ONLY
    pub assigned_to: Option<UserId>,                           // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum AmlAlertType { SanctionMatch, PepHit, HighRiskCountry, StructuringPattern, RapidMovement, UnusualVelocity }
pub enum AmlAlertSeverity { Low, Medium, High, Critical }
pub enum AmlAlertStatus { Open, UnderReview, Closed }
pub enum AmlDisposition { FalsePositive, TruePositive, EscalatedToSar, Dismissed }

/// SAR (Suspicious Activity Report) — immutable after filing
/// data_class: FINANCIAL_KR_신용정보; PII_IDENTIFYING
/// plane: control (SAR filing decision)
pub struct SarRecord {
    pub id: SarRecordId,
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub schema_version: u32,
    pub aml_alert_ids: Vec<AmlAlertId>,             // data_class: INTERNAL_ONLY
    pub subject_kyc_id: Option<KycRecordId>,        // data_class: PII_IDENTIFYING
    pub narrative: EncryptedBlob,                   // data_class: FINANCIAL_KR_신용정보 (KMS-encrypted)
    pub filing_ref: Option<String>,                 // regulatory filing reference number
    pub filed_to: RegulatoryAuthorityId,            // FIU, FinCEN, NCA, etc.
    pub filed_at: Option<DateTime<Utc>>,
    pub foundry_draft_run_id: Option<FoundryRunId>, // data_class: INTERNAL_ONLY (Foundry-assisted draft)
    pub filed_by: UserId,                           // human compliance officer; mandatory
    pub status: SarStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum SarStatus { Draft, PendingApproval, Filed, Rejected }
```

```rust
// crates/oya-vertical-fintech-kernel-account
// ISO 20022 / open banking aligned

/// data_class: FINANCIAL_KR_신용정보 (KR); PCI (card account)
/// plane: data
pub struct AccountStatement {
    pub id: AccountStatementId,
    pub tenant_id: TenantId,
    pub account_ref: EncryptedAccountRef,          // data_class: PCI / FINANCIAL_KR_신용정보
    pub region: RegionCode,
    pub schema_version: u32,
    pub statement_period: DateRange,               // data_class: BEHAVIORAL_TENANT_PRODUCT
    pub opening_balance: Money,                    // data_class: FINANCIAL_KR_신용정보
    pub closing_balance: Money,                    // data_class: FINANCIAL_KR_신용정보
    pub entries: Vec<AccountEntry>,                // data_class: FINANCIAL_KR_신용정보
    pub currency: CurrencyCode,
    pub consent_ref: ConsentId,                    // open banking consent that authorized this aggregation
    pub data_provider: DataProviderId,             // open banking data provider (bank)
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct AccountEntry {
    pub booking_date: NaiveDate,
    pub value_date: NaiveDate,
    pub amount: Money,                             // data_class: FINANCIAL_KR_신용정보
    pub credit_debit_indicator: CreditDebitIndicator,
    pub remittance_info: Option<String>,           // data_class: FINANCIAL_KR_신용정보
    pub entry_ref: Option<String>,
}
pub enum CreditDebitIndicator { Credit, Debit }
```

### 5.2 Aggregate Boundaries

| Aggregate | Root entity | Consistency boundary |
|---|---|---|
| `PaymentInstructionAggregate` | `PaymentInstruction` | Single payment lifecycle; AML check status is inline |
| `KycAggregate` | `KycRecord` + `KycCheck[]` | KYC workflow for one subject; document refs are to object store |
| `AmlAlertAggregate` | `AmlAlert` | One alert lifecycle; SAR is a separate aggregate created from the alert |
| `SarAggregate` | `SarRecord` | Immutable after filing; append-only |
| `SettlementBatchAggregate` | `SettlementBatch` + `SettlementEntry[]` | One netting cycle; entries are inline |
| `AccountStatementAggregate` | `AccountStatement` + `AccountEntry[]` | One statement period per account; consent-gated |

### 5.3 Persistence Layout

| Aggregate | Store | Sharding key | Partition strategy | Replication | Retention |
|---|---|---|---|---|---|
| PaymentInstruction | Postgres (per-account shard with hot-key smoothing) | `account_hot_key_token` → distributed across shards | Hot-key smoothing: high-volume accounts distributed across N shards | Streaming replication × 3 | 7 years (KR 전자금융거래법; FINCEN 5 years; EU PSD2 5 years) |
| KycRecord + KycCheck | Postgres (per-tenant shard) | `tenant_id` | Per-tenant schema | Streaming replication × 3 | 5 years after relationship end (FATF guidance) |
| AmlAlert | Postgres (per-tenant shard) | `tenant_id` | Per-tenant schema | Streaming replication × 3 | 5 years (KR 특정금융정보법; FinCEN) |
| SarRecord | Postgres (append-only, per-tenant) | `tenant_id` | Append-only partition; immutable | Streaming replication × 3 | 5 years from filing (KR FIU; FinCEN) |
| AccountStatement | Postgres (per-account shard) | `encrypted_account_ref_hash` | Per-account shuffle sharding | Streaming replication × 2 | 13 months (EU PSD2 access-to-account minimum) + tenant policy |
| SettlementBatch | Postgres (per-rail shard) | `(tenant_id, rail_id, settlement_date)` | Per-rail per-date | Streaming replication × 2 | 7 years |

### 5.4 Event Schemas

| Event name | Topic | Schema location | Consumer aggregates | Retention | Idempotency key |
|---|---|---|---|---|---|
| `PaymentInitiated` | `fintech.payment.initiated` | `contracts/events/fintech-payment.json` | AML check (async), Audit chain | 30 days | `idempotency_key` |
| `PaymentSettled` | `fintech.payment.settled` | `contracts/events/fintech-payment.json` | Settlement batch, GL posting (Corporate), Audit chain | 30 days | `payment_instruction_id` |
| `PaymentFailed` | `fintech.payment.failed` | `contracts/events/fintech-payment.json` | Retry saga, Merchant notification, Audit chain | 30 days | `payment_instruction_id` |
| `AmlAlertRaised` | `fintech.aml.alert_raised` | `contracts/events/fintech-aml.json` | Compliance queue, Foundry risk-scoring, Audit chain | 365 days | `aml_alert_id` |
| `SarFiled` | `fintech.aml.sar_filed` | `contracts/events/fintech-aml.json` | Audit chain (mandatory, immutable), Regulatory evidence pack | 7 years | `sar_record_id` |
| `KycApproved` | `fintech.kyc.approved` | `contracts/events/fintech-kyc.json` | Payment enablement, Open banking consent unlock, Audit chain | 365 days | `kyc_record_id` |
| `KycExpired` | `fintech.kyc.expired` | `contracts/events/fintech-kyc.json` | Payment suspension, Compliance alert, Audit chain | 365 days | `kyc_record_id` |
| `OpenBankingConsentGranted` | `fintech.openbanking.consent_granted` | `contracts/events/fintech-openbanking.json` | Account aggregation trigger, Audit chain | 365 days | `(kyc_record_id, consent_ref)` |

### 5.5 Index / Search-Index Touchpoints

| Entity field | Index | Class allowed | Cascade-on-DSR? |
|---|---|---|---|
| `PaymentInstruction.end_to_end_id` | tenant-private search (payment lookup) | `BEHAVIORAL_TENANT_PRODUCT` | No (regulatory retention; pseudonymized if DSR) |
| `KycRecord.legal_name` | tenant-private KYC directory | `PII_IDENTIFYING` — tenant-private only | Yes — DSR cascade on KYC subject |
| `AmlAlert.alert_type` + status | tenant-private compliance dashboard | `INTERNAL_ONLY` | No (regulatory retention) |

**Structural enforcement:** `PCI` and `FINANCIAL_KR_신용정보` data classes are HARD DENY for ads and analytics across tenant boundaries. The `oya-platform-ads-gate` rejects any record with these classes at the eventing backbone level. No policy-only control; structurally enforced.

### 5.6 Audit-Chain Emission Contract

| Operation | Emits topic | Required fields |
|---|---|---|
| Payment instruction created | `audit.fintech.payment.initiated` | `payment_instruction_id`, `rail_id`, `amount_hash` (not plaintext), `aml_check_status`, `idempotency_key` |
| Payment settled | `audit.fintech.payment.settled` | `payment_instruction_id`, `settlement_ref`, `rail_transaction_ref`, `settled_at` |
| AML alert raised | `audit.fintech.aml.alert_raised` | `aml_alert_id`, `alert_type`, `severity`, `triggered_rules`, `payment_ref` (pseudonymized) |
| SAR filed | `audit.fintech.sar.filed` | `sar_record_id`, `filed_to`, `filed_at`, `filed_by`, `alert_ids` — **immutable record; append-only** |
| KYC decision | `audit.fintech.kyc.decision` | `kyc_record_id` (pseudonymized), `decision`, `decided_by`, `checks_run`, `risk_level` |
| PCI data accessed | `audit.fintech.pci.access` | `accessor_id`, `payment_instruction_id`, `access_type`, `purpose` |
| Open banking consent decision | `audit.fintech.openbanking.consent` | `consent_ref`, `subject_id` (pseudonymized), `granted_scopes`, `data_provider` |

### 5.7 Schema Migration Policy

- PaymentInstruction is append-only after settlement; no destructive migrations.
- SarRecord is permanently immutable after `SarStatus::Filed`.
- KYC document blobs stored in object store; metadata schema is mutable within additive rules.
- Hot-key smoothing configuration (shard count) is a configuration change, not a schema migration.

---

## 6. Optimization Practices

| Practice | Implementation choice |
|---|---|
| Cell routing | `(tenant_id, rail_id)` → cell; PG merchants on high-volume rails get dedicated cells |
| Sharding strategy | Per-account shard with hot-key smoothing for PaymentInstruction (high-volume merchant accounts distributed across N sub-shards using `account_hot_key_token`); per-tenant for KYC/AML |
| Caching tier | Redis for idempotency-key dedup (24-hour TTL); in-memory for sanction list (refreshed hourly); no PCI/신용정보 in Redis without per-key DEK |
| Bulk endpoint contract | `POST /payments/bulk` (batch payment initiation); `POST /kyc/checks/bulk` (batch KYC re-screening on periodic review); `POST /aml/screen/bulk` (batch transaction screening) |
| Pagination | Cursor on `(created_at, payment_instruction_id)` for payment list; `_since` filter; AML alert list cursor on `(created_at, aml_alert_id)` |
| Idempotency | `Idempotency-Key` header required on all payment initiation calls; 24-hour dedup; AML alert dedup on `(payment_instruction_id, rule_id)` |
| Batch dispatch | Foundry `AmlRiskScorer` runs per-alert batch (async from payment initiation); Foundry `SarDraftWriter` runs on compliance officer request; sanction list refresh as Foundry-scheduled capability |
| Backpressure | PG checkout API rate-limited per merchant (configurable); AML queue depth monitored; if queue depth > threshold, synchronous AML check blocks payment authorization |
| Hot-path benchmarks | `payment_authorize` criterion < 100ms P99 (PG checkout is latency-critical); `aml_sanction_screen` < 50ms (pre-settlement); `kyc_status_check` < 20ms |
| Agent-driven optimization | Foundry `AmlRiskScorer` (ML risk score on transaction features, under autonomy ceiling with compliance officer approval for SAR); Foundry `SarDraftWriter` (narrative draft from alert context — human files); Foundry `SanctionListRefresher` (automated list update) |
| FinOps unit-economics | Per-transaction metering (PG); per-KYC-check metering; per-AML-screen metering; Foundry capability invocations metered separately |
| Build-cache / CI affected-graph | `oya-vertical-fintech-kernel-payment` → full rebuild; `adapter-rails` → per-rail integration tests; `adapter-sanctions` → sanction list conformance test |

---

## 7. Regional Pack Interactions

| Seam | Trait | Per-pack impl needed? | Tested with which packs? |
|---|---|---|---|
| Payment rail adapter | `PaymentRail` | Yes — one impl per rail | `oya-pack-kr` (KFTC 계좌이체/카카오페이/네이버페이/토스 + 마이데이터), `oya-pack-us` (NACHA ACH + FedNow/RTP + Visa/Mastercard), `oya-pack-eu` (SEPA CT + SEPA-Inst + EU open banking PSD2), `oya-pack-in` (NPCI UPI + RTGS + IMPS), `oya-pack-br` (BACEN Pix) |
| KYC identity provider adapter | `KycIdentityProvider` | Yes | `oya-pack-kr` (본인확인서비스 — PASS/카카오/NICE), `oya-pack-us` (Jumio / Onfido / Socure), `oya-pack-eu` (eIDAS + national eID), `oya-pack-in` (Aadhaar eKYC via UIDAI) |
| Open banking data provider | `OpenBankingProvider` | Yes | `oya-pack-kr` (마이데이터 금융위원회 API), `oya-pack-eu` (PSD2 AISP — Plaid / TrueLayer / Salt Edge adapters), `oya-pack-us` (Dodd-Frank 1033 draft / Plaid), `oya-pack-br` (Open Finance BR per BACEN) |
| AML regulatory filing | `AmlFilingAdapter` | Yes — per financial intelligence unit | `oya-pack-kr` (KoFIU 금융정보분석원 SAR e-filing), `oya-pack-us` (FinCEN BSA e-filing), `oya-pack-eu` (national FIU per EU AMLD) |
| Sanction list provider | `SanctionListProvider` | Yes | `oya-pack-kr` (KoFIU + UN + OFAC), `oya-pack-us` (OFAC SDN + OFAC non-SDN), `oya-pack-eu` (EU consolidated sanctions + UN) |
| Regulatory control evidence | `RegulatoryPack` | Yes | `oya-pack-kr` (금융위원회, FSC, 특정금융정보법), `oya-pack-us` (OCC, FinCEN, CFPB, Reg E), `oya-pack-eu` (EBA, PSD2, EU AMLD V/VI, DORA) |

### Regulatory Pack Declaration

```yaml
# registry/catalog/oya-vertical-fintech-runtime.yaml
regulatory_packs:
  - oya-pack-kr   # FSC, 금융위원회, KoFIU, 전자금융거래법, 특정금융정보법, 신용정보법
  - oya-pack-us   # OCC, FinCEN BSA/AML, CFPB, NACHA, FedNow, Reg E, PCI-DSS
  - oya-pack-eu   # EBA, PSD2, EU AMLD VI, GDPR, DORA, SEPA
  - oya-pack-in   # RBI, PMLA, NPCI, UIDAI (Aadhaar eKYC)
  - oya-pack-br   # BACEN, LGPD, Open Finance BR, Pix
tenant_class_overrides:
  ad_targetable_blocked: true   # forced for PCI + FINANCIAL_KR_신용정보; cannot be raised
  search_index_class: INTERNAL_ONLY   # payment/account data never cross-tenant searchable
```

---

## 8. In-House vs External Dependency Posture

| External dep | Maturity tier | License | In-house alternative considered? | Decision |
|---|---|---|---|---|
| `tokio`, `axum`, `sqlx`, `serde`, `rustls` | kernel-grade | MIT / Apache-2 | No | Use |
| `rust_decimal` (monetary arithmetic, no float rounding) | stable | MIT | In-house — critical path; rust_decimal is the standard | Use |
| `iso_currency` (ISO 4217 currency codes) | stable | MIT | In-house enum considered | Use |
| `jose-jwt` / `pasetors` (JWT/PASETO for open banking tokens) | stable | MIT / Apache-2 | In-house token library considered | Use pasetors (PASETO v4); avoid JWT for new surfaces |
| Per-rail SDKs (KFTC, NACHA, NPCI APIs) | external APIs (no Rust SDK) | Proprietary API (no code dep) | In-house HTTP clients wrapping official APIs | Build in-house per-rail adapters in `oya-vertical-fintech-adapter-rails` |
| KYC provider SDKs (Jumio / Onfido / NICE KR) | external APIs | Proprietary | In-house adapter wrapping vendor REST APIs | Adapter pattern in `oya-vertical-fintech-adapter-kyc` |
| `luhn` (card number validation) | trivial | MIT | In-house trivial | Use (10 lines; license clear) |
| HSM integration (KCMVP for KR PCI) | hardware | Vendor SDK (proprietary) | KMS/HSM integration in `oya-platform-secrets-*` layer; fintech adapter calls KMS only | Use KMS seam; no direct HSM dep in fintech kernel |

---

## 9. Success Metrics

| Metric | Vertical-Preview target | Vertical-Stable target | Public-GA target |
|---|---|---|---|
| Payment authorization latency P99 | < 500ms | < 200ms | < 100ms |
| Payment success rate (excluding user errors) | ≥ 98% | ≥ 99.5% | ≥ 99.9% |
| AML alert false-positive rate | baseline (measure only) | < 30% | < 15% (Foundry ML improvement) |
| SAR filing cycle time (alert → filed) | < 30 business days | < 15 days | < 7 days |
| KYC approval cycle time (documents → decision) | < 24 hours | < 4 hours | < 1 hour (Foundry-assisted) |
| Audit-chain completeness (payment + SAR events) | 100% | 100% | 100% |
| PCI/신용정보 leak to ads/analytics | 0 (hard zero, structurally enforced) | 0 | 0 |
| Open banking consent grant → account data < latency | < 30s | < 10s | < 5s |
| Settlement reconciliation accuracy | 100% | 100% | 100% |
| Cross-axis contract violations | 0 | 0 | 0 |

---

## 10. Risks + Mitigations

| Risk | Severity | Mitigation | Owner |
|---|---|---|---|
| PCI data in plaintext in audit log or event payload | Catastrophic | All PCI fields are `EncryptedAccountRef` (KMS-encrypted); plaintext PAN never in any event; luhn-only validation (no storage); CI lint checks for PCI field class annotations | Security + Privacy |
| AML rule miss (transaction clears that should be blocked) | Critical | Foundry AmlRiskScorer as second layer; sanction list refreshed hourly; manual compliance review queue; regulatory-change watch lane for new sanctions lists | AML domain + Compliance |
| Payment double-spend (idempotency failure) | Critical | Idempotency-Key required + 24-hour dedup in Redis; outbox pattern for PaymentSettled event; bank ack checked before status flip | Payment domain |
| Hot-key problem (high-volume merchant on single shard) | High | Hot-key smoothing: `account_hot_key_token` distributes high-volume accounts across N sub-shards at write time; weighted consistent hashing | Infrastructure + Payment domain |
| KYC document blob leak (object store misconfiguration) | Catastrophic | KYC document blobs stored in tenant-isolated object store prefix with KMS-DEK per document; pre-signed URL with 15-min TTL for access; no public bucket ACLs | Security |
| SAR filing deadline missed (regulatory penalty) | High | SAR workflow has automated deadline tracking (30-day filing window); escalation alerts at 20-day mark; Foundry `SarDraftWriter` accelerates narrative | AML domain + Foundry |
| Open banking consent expiry (stale data access) | Medium | ConsentRef checked on every account aggregation call; expired consent triggers automatic suspension; ConsumerExpired event triggers DSR cascade | Open banking domain + Privacy |
| KR 신용정보법 amendment (마이데이터 scope change) | High | Regulatory-change watch lane; KR pack versioned; open banking model is pluggable per provider | KR pack + Compliance |
| Foundry AML agent autonomy ceiling exceeded | High | Cedar policy gates `aml.freeze_account` and `aml.file_sar` at T3 (not granted at Preview/Stable); human compliance officer approves all SAR filings | Foundry + Compliance |
| DORA incident response requirement (EU fintech tenants) | Medium | Incident response runbook aligned to DORA ICT risk framework; evidence generated per incident for EU-pack regulatory reporting | EU pack + SRE |

---

## 11. Open Questions

- KR 마이데이터 open banking API — direct integration with Financial Services Commission API hub or via aggregator (Yodlee-KR / 뱅크샐러드)? Affects `oya-pack-kr` open-banking provider adapter.
- BNPL (Buy Now Pay Later) data model — in-scope for Vertical-Stable or separate credit vertical?
- Crypto asset reconciliation seam — which crypto custodian APIs to adapter-wrap first (Fireblocks / BitGo)?
- DORA ICT risk reporting — which tenant data is in-scope for DORA incident notification? Need EU pack scoping.
- PG 3DS2 (3-D Secure 2) authentication — in-house implementation or rely on card network SDK?

---

## 12. Decision Log

| Decision | Date | Rationale | ADR ref |
|---|---|---|---|
| ISO 20022 as canonical payment data model | 2026-05-09 | Global standard for financial messaging; adopted by SWIFT, FedNow, SEPA, NPCI UPI | — |
| PCI + 신용정보 forced `ad_targetable_blocked` (cannot be raised) | 2026-05-09 | PRIVACY-PROGRAM §2.2.3 fintech override; PCI-DSS + KR 신용정보법 mandate | PRIVACY-PROGRAM §2.2.3 |
| Hot-key smoothing for high-volume merchant accounts | 2026-05-09 | Single-shard bottleneck on high-volume merchant (e.g., large e-commerce tenant) is a P0 scaling risk | DESIGN.md §9 |
| Foundry AML agents at T2 max (flag + freeze with compliance approval) | 2026-05-09 | Autonomous SAR filing without human review violates FATF guidance and most AML regulations | ADR-0050; ADR-0022 |
| SAR records append-only, permanently immutable after filing | 2026-05-09 | Regulatory requirement; FinCEN + KoFIU prohibit alteration of filed SARs | — |
| Flat-crates: `crates/oya-vertical-fintech-*` | 2026-05-09 | Per ADR-0015 | ADR-0015 |

---

## 13. Sources Scanned

- `docs/PRD.md`, `docs/DESIGN.md` §1, §4, §10, §12
- `docs/PRIVACY-PROGRAM.md` §2.2.1, §2.2.3 (PCI/신용정보 HARD DENY)
- ISO 20022 (pacs.008/pacs.004/camt.053); FATF 40 Recommendations; KR 특정금융정보법; PCI-DSS v4; EU PSD2; DORA

---

## Doc-Catalog Row

```
| `vertical-fintech` | `vertical-2` | PG/open-banking/KYC-KYB/AML/ISO-20022/multi-rail; PCI-hard-deny | monthly | PRD.md, DESIGN.md §12, PRIVACY-PROGRAM.md §2.2.3 |
```
