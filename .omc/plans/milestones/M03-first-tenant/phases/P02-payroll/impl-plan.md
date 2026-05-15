---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-first-paying-tenant
phase: P02-payroll
impl_plan_id: IP-P02-payroll-full-scaffold
status: pending
owner: council-enterprise
blocked_by:
- impl_plan: IP-P01-hr-full-scaffold
  reason: Employee/Employment Ontology Object Types must exist before payroll adapter
    can read them.
acceptance_lanes:
- cargo-check
- cargo-build
- cargo-clippy
- cargo-nextest
- cargo-deny
- lean-a1
- lean-a2
- lean-a3
- lean-a4
- ontology-type-registry
- workflow-event-registry
- audit-chain
- jurisdiction-overlay
- k6-smoke
purpose: Auto-backfilled purpose for impl-plan.md
---
# IP-P02-payroll-full-scaffold: Payroll µservice — DDL, gross-to-net engine, 4대보험 EDI, 연말정산, Typst payslips, bank disbursement, Workflow events, load tests

## Intent

Scaffolds the complete `oya-payroll-*` µservice: Postgres DDL for all 5 BCs with Citus sharding + RLS + outbox; Rust kernel port traits; gross-to-net KR tax engine (4대보험 deductions, income tax withholding by EmploymentClassification, pro-rata for mid-month hires/terminations); 4대보험 EDI adapters (NPS/NHIS/MOEL — 취득/상실/변경/보수월액 formats); 연말정산 21-category deduction engine; Typst payslip renderer; KR bank CMS/NEMS disbursement batch; Workflow event fan-out; Ontology writes (PayrollEntry/WithholdingCert/Payslip); load tests.

---

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `crates/oya-payroll-run-kernel/Cargo.toml` | create | `[package]` + deps |
| `crates/oya-payroll-run-kernel/src/types.rs` | create | `RunId(Uuid)`, `EntryId(Uuid)`, `PayPeriod(year, month)` |
| `crates/oya-payroll-run-kernel/src/ports.rs` | create | `PayrollRunRepository`, `PayrollEntryStore` sealed port traits |
| `crates/oya-payroll-run-domain/Cargo.toml` | create | deps: run-kernel + `oya-finance-library-domain` |
| `crates/oya-payroll-run-domain/src/gross_to_net.rs` | create | `GrossToNetEngine` — KR deduction computation |
| `crates/oya-payroll-run-domain/src/payroll_run.rs` | create | `PayrollRun` aggregate + `PayrollEntry` entity |
| `crates/oya-payroll-run-domain/src/pro_rata.rs` | create | Pro-rata calculation for mid-month hires/terminations |
| `crates/oya-payroll-insurance-kernel/Cargo.toml` | create | deps |
| `crates/oya-payroll-insurance-kernel/src/types.rs` | create | `InsuranceConfigId(Uuid)`, `EdiSubmissionId(Uuid)`, 4대보험 rate types |
| `crates/oya-payroll-insurance-kernel/src/ports.rs` | create | `InsuranceConfigStore`, `EdiSubmissionPort` sealed port traits |
| `crates/oya-payroll-insurance-domain/src/four_insurance_edi.rs` | create | `FourInsuranceEdi` — 취득/상실/변경/보수월액 EDI format per NHIS/NPS v5.0 schema |
| `crates/oya-payroll-insurance-domain/src/insurance_rates.rs` | create | `InsuranceRates` — NPS 9%, 건강보험 7.09%, 고용 1.8%, 산재 업종별 |
| `crates/oya-payroll-insurance-adapter/src/nps_edi_adapter.rs` | create | `NpsEdiAdapter` — formats 국민연금 취득/상실/변경 EDI files |
| `crates/oya-payroll-insurance-adapter/src/nhis_edi_adapter.rs` | create | `NhisEdiAdapter` — 건강보험 EDI files |
| `crates/oya-payroll-insurance-adapter/src/moel_edi_adapter.rs` | create | `MoelEdiAdapter` — 고용보험/산재보험 EDI files |
| `crates/oya-payroll-year-end-kernel/Cargo.toml` | create | deps |
| `crates/oya-payroll-year-end-kernel/src/types.rs` | create | `SettlementId(Uuid)`, `CertId(Uuid)`, `YearEndDeductionCategory` 21-variant enum |
| `crates/oya-payroll-year-end-kernel/src/ports.rs` | create | `YearEndRepository`, `WithholdingCertRenderer` sealed port traits |
| `crates/oya-payroll-year-end-domain/src/deduction_categories.rs` | create | All 21 categories (소득공제 14 + 세액공제 7) per 2025 tax year |
| `crates/oya-payroll-year-end-domain/src/year_end_settlement.rs` | create | `YearEndSettlement` aggregate + settlement computation |
| `crates/oya-payroll-payslip-kernel/src/ports.rs` | create | `PayslipRenderer` sealed port trait (Typst render port) |
| `crates/oya-payroll-payslip-adapter/src/typst_payslip_renderer.rs` | create | `TypstPayslipRenderer` implements `PayslipRenderer` |
| `crates/oya-payroll-disbursement-kernel/src/ports.rs` | create | `BankTransferPort`, `DisbursementStore` sealed port traits |
| `crates/oya-payroll-disbursement-adapter/src/cms_batch_adapter.rs` | create | `CmsBatchAdapter` — KR CMS 파일 형식 (KB/신한/우리/하나) |
| `crates/oya-payroll-disbursement-adapter/src/nems_batch_adapter.rs` | create | `NemsBatchAdapter` — NEMS 은행 대량이체 형식 |
| `crates/oya-payroll-app/src/main.rs` | create | DI assembly; runs payroll worker + REST API |
| `migrations/payroll/001_payroll_schema.sql` | create | Full DDL (see below) |
| `contracts/payroll.openapi.yaml` | create | OpenAPI 3.1 for `/v1/payroll-runs`, `/v1/payslips`, `/v1/edi-submissions` |
| `proto/payroll/events.proto` | create | Protobuf event schemas |
| `policies/payroll/payroll.cedar` | create | Cedar policy pack |
| `typst-templates/payslip/payslip.typ` | create | Typst payslip template (KR format; 근로소득원천징수영수증 layout) |
| `tests/load/smoke-payroll-payslip-read.js` | create | k6 smoke: p99 ≤50ms at 1k RPS |
| `tests/load/m03-payroll-3k.js` | create | k6 load: 3k employees payroll run ≤30s |
| `Cargo.toml` | update | Add all `oya-payroll-*` crates |
| `docs/standards/bounded-contexts.md` | update | Register run/insurance/year-end/payslip/disbursement BCs |

---

## Code Shape

### `crates/oya-payroll-run-domain/src/gross_to_net.rs`

```rust
/// KR gross-to-net engine
/// Statutory references (corpus.lock pinned):
///   NPS: 대한민국.사회보험.국민연금법.제88조 (employer 4.5%; employee 4.5%)
///   NHIS: 대한민국.사회보험.국민건강보험법.제69조 (employer 3.545%; employee 3.545%)
///   고용보험: 대한민국.사회보험.고용보험법.제68조 (employer 0.9%+α; employee 0.9%)
///   산재: 대한민국.사회보험.산업재해보상보험법.제13조 (employer only; 업종별)
///   소득세: 대한민국.세금.소득세법.제134조 (근로소득 간이세액표)
pub struct GrossToNetEngine {
    pub insurance_rates: InsuranceRates,
    pub tax_table: KrIncomeTaxTable,
}

impl GrossToNetEngine {
    pub fn compute(
        &self,
        gross_salary: Money,
        classification: EmploymentClassification,
        fte_pct: Decimal,
        pay_days: u8,           // for pro-rata
        total_days_in_month: u8,
        weekly_hours: Option<Decimal>,
    ) -> Result<PayrollEntryDeductions, PayrollError> {
        // 1. Determine applicable deductions by classification
        // 2. Compute 4대보험 employee share
        // 3. Compute income tax per 간이세액표 (or 사업소득 3.3% for Freelance)
        // 4. Compute local tax (지방소득세 = income_tax × 10%)
        // 5. Pro-rata if pay_days < total_days_in_month
        // 6. Return PayrollEntryDeductions { national_pension, health_insurance,
        //    employment_insurance, income_tax, local_tax, net_pay }
    }
}
```

### `crates/oya-payroll-year-end-domain/src/deduction_categories.rs`

```rust
/// 연말정산 21-category deduction model — 2025 tax year
/// Statute: 대한민국.세금.소득세법.제50조~제59조의4
/// corpus_sha: <read from corpus.lock>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum YearEndDeductionCategory {
    // 소득공제 (income deductions) — 14 categories
    BasicDeduction,          // 기본공제
    AdditionalDeduction,     // 추가공제
    PensionInsurance,        // 연금보험료공제
    SpecialDeductionInsurance,   // 특별소득공제 - 건강보험
    SpecialDeductionHousing,     // 특별소득공제 - 주택자금
    PersonalPension,         // 개인연금저축
    SocialInsurance,         // 소기업소상공인공제
    CreditCard,              // 신용카드등사용금액
    HousingFund,             // 주택마련저축
    RetirementPension,       // 퇴직연금
    StockOwnership,          // 우리사주조합출자
    EmployeeStockPurchase,   // 우리사주조합 인출금
    LongTermCareInsurance,   // 장기요양보험료
    Other,                   // 기타소득공제
    // 세액공제 (tax credits) — 7 categories
    TaxCreditPersonal,       // 근로소득세액공제
    TaxCreditChildren,       // 자녀세액공제
    TaxCreditPension,        // 연금계좌세액공제
    TaxCreditSpecial,        // 특별세액공제 (보험/의료비/교육비/기부금)
    TaxCreditStandardized,   // 표준세액공제
    TaxCreditPartTime,       // 중소기업취업자세액공제
    TaxCreditForeign,        // 외국납부세액공제
}
```

---

## Postgres DDL

### migrations/payroll/001_payroll_schema.sql (key tables)

```sql
CREATE SCHEMA IF NOT EXISTS payroll;

-- Payroll runs
CREATE TABLE payroll.runs (
    run_id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       uuid NOT NULL,
    pay_year        int NOT NULL,
    pay_month       int NOT NULL CHECK (pay_month BETWEEN 1 AND 12),
    status          text NOT NULL DEFAULT 'draft'
                    CHECK (status IN ('draft','calculating','calculated','approved','disbursed','sealed')),
    employee_count  int NOT NULL DEFAULT 0,
    total_gross     numeric(20,2) NOT NULL DEFAULT 0,
    total_net       numeric(20,2) NOT NULL DEFAULT 0,
    total_insurance numeric(20,2) NOT NULL DEFAULT 0,
    total_tax       numeric(20,2) NOT NULL DEFAULT 0,
    sealed_at       timestamptz NULL,   -- Ed25519 seal timestamp
    audit_hash      bytea NULL,         -- Merkle root per (tenant_id, year, month)
    jurisdiction_code text NOT NULL DEFAULT 'KR',
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE payroll.runs ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON payroll.runs
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE UNIQUE INDEX idx_run_period ON payroll.runs (tenant_id, pay_year, pay_month)
    WHERE status NOT IN ('draft');
-- SELECT create_distributed_table('payroll.runs', 'tenant_id');

-- Payroll entries (one per employee per run)
CREATE TABLE payroll.entries (
    entry_id            uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           uuid NOT NULL,
    run_id              uuid NOT NULL REFERENCES payroll.runs(run_id),
    employee_id         uuid NOT NULL,      -- FK to hr.employees (cross-schema; enforced in application)
    employment_id       uuid NOT NULL,      -- FK to hr.employments
    classification      text NOT NULL,      -- EmploymentClassification snapshot
    gross_salary        numeric(15,2) NOT NULL,
    pay_days            int NOT NULL,
    -- 4대보험 employee share
    national_pension    numeric(12,2) NOT NULL DEFAULT 0,
    health_insurance    numeric(12,2) NOT NULL DEFAULT 0,
    employment_insurance numeric(12,2) NOT NULL DEFAULT 0,
    -- Tax
    income_tax          numeric(12,2) NOT NULL DEFAULT 0,
    local_income_tax    numeric(12,2) NOT NULL DEFAULT 0,
    -- Net
    net_pay             numeric(15,2) NOT NULL,
    -- Employer share (for accounting journal entry)
    employer_national_pension   numeric(12,2) NOT NULL DEFAULT 0,
    employer_health_insurance   numeric(12,2) NOT NULL DEFAULT 0,
    employer_employment_insurance numeric(12,2) NOT NULL DEFAULT 0,
    employer_workers_comp       numeric(12,2) NOT NULL DEFAULT 0,
    created_at          timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE payroll.entries ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON payroll.entries
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE INDEX idx_entry_run ON payroll.entries (tenant_id, run_id);
CREATE UNIQUE INDEX idx_entry_emp_run ON payroll.entries (tenant_id, run_id, employee_id);

-- 4대보험 EDI submissions
CREATE TABLE payroll.edi_submissions (
    submission_id   uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       uuid NOT NULL,
    run_id          uuid NOT NULL REFERENCES payroll.runs(run_id),
    insurer         text NOT NULL CHECK (insurer IN ('nps','nhis','moel_employment','moel_workers_comp')),
    edi_type        text NOT NULL CHECK (edi_type IN ('취득','상실','변경','보수월액')),
    file_content    bytea NOT NULL,         -- EDI file content (immutable; resubmission creates new row)
    submitted_at    timestamptz NULL,
    ack_received_at timestamptz NULL,
    ack_code        text NULL,
    ack_message     text NULL,
    status          text NOT NULL DEFAULT 'pending'
                    CHECK (status IN ('pending','submitted','acknowledged','rejected')),
    created_at      timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE payroll.edi_submissions ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON payroll.edi_submissions
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

-- Year-end settlements
CREATE TABLE payroll.year_end_settlements (
    settlement_id   uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       uuid NOT NULL,
    employee_id     uuid NOT NULL,
    settlement_year int NOT NULL,
    total_income    numeric(15,2) NOT NULL,
    total_deductions numeric(15,2) NOT NULL DEFAULT 0,
    tax_base        numeric(15,2) NOT NULL,
    computed_tax    numeric(15,2) NOT NULL,
    withheld_tax    numeric(15,2) NOT NULL,
    refund_amount   numeric(15,2) GENERATED ALWAYS AS (withheld_tax - computed_tax) STORED,
    status          text NOT NULL DEFAULT 'draft'
                    CHECK (status IN ('draft','confirmed','sealed')),
    sealed_at       timestamptz NULL,
    audit_hash      bytea NULL,
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE payroll.year_end_settlements ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON payroll.year_end_settlements
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE UNIQUE INDEX idx_yes_emp_year ON payroll.year_end_settlements (tenant_id, employee_id, settlement_year);

-- Payslips (metadata; content rendered on demand via Typst)
CREATE TABLE payroll.payslips (
    payslip_id      uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       uuid NOT NULL,
    entry_id        uuid NOT NULL REFERENCES payroll.entries(entry_id),
    employee_id     uuid NOT NULL,
    pay_year        int NOT NULL,
    pay_month       int NOT NULL,
    pdf_object_key  text NULL,      -- OCI Object Storage key for rendered PDF
    delivered_at    timestamptz NULL,
    created_at      timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE payroll.payslips ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON payroll.payslips
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

-- Disbursement batches
CREATE TABLE payroll.disbursement_batches (
    batch_id        uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       uuid NOT NULL,
    run_id          uuid NOT NULL REFERENCES payroll.runs(run_id),
    bank_code       text NOT NULL,      -- 은행 코드: 004=KB, 088=신한, 020=우리, 081=하나
    batch_format    text NOT NULL CHECK (batch_format IN ('cms','nems')),
    file_content    bytea NOT NULL,     -- bank transfer file (immutable)
    total_amount    numeric(20,2) NOT NULL,
    entry_count     int NOT NULL,
    submitted_at    timestamptz NULL,
    status          text NOT NULL DEFAULT 'pending'
                    CHECK (status IN ('pending','submitted','confirmed','failed')),
    created_at      timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE payroll.disbursement_batches ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON payroll.disbursement_batches
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

-- Outbox
CREATE TABLE payroll.outbox (
    outbox_id   bigserial PRIMARY KEY,
    tenant_id   uuid NOT NULL,
    topic       text NOT NULL,
    key         text NOT NULL,
    payload     jsonb NOT NULL,
    published_at timestamptz NULL,
    created_at  timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_payroll_outbox_unpublished ON payroll.outbox (created_at)
    WHERE published_at IS NULL;
```

---

## Protobuf Event Schemas

```proto
syntax = "proto3";
package payroll.events;

// Kafka topic: payroll.{tenant_id}.PayrollRunCompleted
message PayrollRunCompleted {
    string tenant_id = 1;
    string run_id = 2;
    int32  pay_year = 3;
    int32  pay_month = 4;
    int32  employee_count = 5;
    string total_gross = 6;      // numeric string; KRW
    string total_net = 7;
    string audit_hash = 8;       // Merkle root hex
    int64  occurred_at_ms = 9;
}

// Kafka topic: payroll.{tenant_id}.PayrollFinalizedForEmployee
message PayrollFinalizedForEmployee {
    string tenant_id = 1;
    string run_id = 2;
    string employee_id = 3;
    string entry_id = 4;
    string net_pay = 5;          // KRW numeric string
    int64  occurred_at_ms = 6;
}

// Kafka topic: payroll.{tenant_id}.EdiSubmissionQueued
message EdiSubmissionQueued {
    string tenant_id = 1;
    string run_id = 2;
    string submission_id = 3;
    string insurer = 4;          // nps | nhis | moel_employment | moel_workers_comp
    string edi_type = 5;         // 취득 | 상실 | 변경 | 보수월액
    int64  occurred_at_ms = 6;
}

// Kafka topic: payroll.{tenant_id}.YearEndSettlementCompleted
message YearEndSettlementCompleted {
    string tenant_id = 1;
    string settlement_id = 2;
    int32  settlement_year = 3;
    int32  employee_count = 4;
    string audit_hash = 5;
    int64  occurred_at_ms = 6;
}

// Kafka topic: payroll.{tenant_id}.DisbursementBatchReady
message DisbursementBatchReady {
    string tenant_id = 1;
    string batch_id = 2;
    string run_id = 3;
    string bank_code = 4;
    string total_amount = 5;
    int32  entry_count = 6;
    int64  occurred_at_ms = 7;
}
```

---

## Cedar Policy Pack

```cedar
// payroll/payroll.cedar
entity Tenant;
entity PayrollAdmin in [Tenant] = { organization_id: String };
entity EmployeeUser in [Tenant] = { employee_id: String };
entity Auditor in [Tenant];

// Payroll admin can run payroll and configure EDI for their org
permit (
    principal is PayrollAdmin,
    action in [Action::"RunPayroll", Action::"ApproveRun", Action::"ConfigureEdi",
               Action::"SubmitEdi", Action::"RunYearEnd", Action::"ExportDisbursement"],
    resource
) when {
    context.tenant_id == principal.tenant_id
};

// Employee can read own payslip only
permit (
    principal is EmployeeUser,
    action == Action::"ReadPayslip",
    resource
) when {
    resource.employee_id == principal.employee_id
};

// No cross-tenant access (ADR-0018)
forbid (principal, action, resource) when {
    context.tenant_id != resource.tenant_id
};
```

---

## Acceptance Gates

```bash
cargo check -p oya-payroll-app --all-features  # exit 0
cargo nextest run -p oya-payroll-run-domain --test payroll_run_1000  # exit 0; ≤30s
cargo nextest run -p oya-payroll-insurance-domain --test edi_format_compliance  # exit 0; 더존 reference match
cargo nextest run -p oya-payroll-year-end-domain --test year_end_all_categories  # exit 0; all 21 categories
oya gate validate lean-a2 --ms payroll   # no imports from hr/accounting/connect
oya gate validate audit-chain --ms payroll
k6 run tests/load/smoke-payroll-payslip-read.js  # p(99)<50
```

---

## Test Plan

| Test | Verifies |
|---|---|
| `test_gross_to_net_regular` | Regular employee; all 4대보험; 근로소득세; local tax; correct net |
| `test_gross_to_net_freelance` | Freelance; 사업소득 3.3% only; no 4대보험 employer share |
| `test_gross_to_net_part_time` | Pro-rata by hours; ≥15h/wk severance threshold |
| `test_pro_rata_mid_month_hire` | Employee hired on 15th; pay_days=17; net correct |
| `edi_format_compliance` | 취득/상실/변경/보수월액 EDI byte-exact match against 더존 iCUBE reference fixture |
| `year_end_all_categories` | All 21 deduction categories computed; total deductions correct |
| `test_disbursement_cms_format` | CMS batch file byte-exact against KB bank reference |
| `test_payroll_run_1000` | 1,000 employees in ≤30s; 0 errors; all entries correct |
| `test_payroll_accounting_workflow` | `PayrollRunCompleted` event routed to accounting consumer |
| `test_audit_chain_sealed` | Ed25519 seal per (tenant_id, year, month); tamper detection |

### Load test

```javascript
// tests/load/smoke-payroll-payslip-read.js
export const options = {
  vus: 100,
  duration: '60s',
  thresholds: {
    http_req_duration: ['p(99)<50'],   // PRD: payslip read p99 ≤50ms
    http_req_failed: ['rate<0.001'],
  },
};
```

```javascript
// tests/load/m03-payroll-3k.js — 3k-person shape (P08 acceptance)
export const options = {
  vus: 10,
  iterations: 1,
  thresholds: {
    'payroll_run_duration': ['value<30000'],  // ≤30s for 3k employees
  },
};
```

---

## Grit Symbol-Locks

```bash
grit session start ip-p02-payroll-full-scaffold

grit claim \
  --agent ip-p02-payroll \
  --intent "P02-payroll: scaffold gross-to-net engine, 4대보험 EDI adapters, 연말정산 21 categories, Typst payslips, bank disbursement batch" \
  --ttl 3600 \
  crates/oya-payroll-run-kernel/src/ports.rs::PayrollRunRepository \
  crates/oya-payroll-run-domain/src/gross_to_net.rs::GrossToNetEngine \
  crates/oya-payroll-insurance-domain/src/four_insurance_edi.rs::FourInsuranceEdi \
  crates/oya-payroll-year-end-domain/src/deduction_categories.rs::YearEndDeductionCategory \
  crates/oya-payroll-disbursement-kernel/src/ports.rs::BankTransferPort \
  migrations/payroll/001_payroll_schema.sql::payroll.entries \
  proto/payroll/events.proto::PayrollRunCompleted \
  contracts/payroll.openapi.yaml::runPayroll
```

---

## ICM Rows to Emit

```bash
icm store \
  -t context-oyatie \
  -c "IP-P02-payroll-full-scaffold merged; GrossToNetEngine (KR tax); 4대보험 EDI adapters (NPS/NHIS/MOEL); 연말정산 21 categories; Typst payslips; CMS/NEMS bank batch; Workflow events: PayrollRunCompleted/PayrollFinalizedForEmployee/EdiSubmissionQueued/YearEndSettlementCompleted/DisbursementBatchReady; LEAN lanes green; next: IP-P03-accounting" \
  -i high \
  -k "M03,P02,IP-P02-payroll-full-scaffold,payroll,4대보험,연말정산"
```

---

## Halt Conditions

1. 4대보험 EDI format compliance test fails after 3 attempts — byte-level format mismatch against 더존 iCUBE reference; requires EDI spec document review; escalate to payroll domain expert.
2. `test_payroll_run_1000` consistently exceeds 30s — parallelism design issue; escalate to architect for worker-pool sizing.
3. LEAN-A2 violation on `oya-payroll-run-application` importing `oya-hr-*` — design boundary error; payroll must read Employee/Employment via Ontology ObjectStore port, not HR crates directly.

---

## Next IP Pointer

`phases/P03-accounting/impl-plan.md`

---

## Cross-References

- Phase spec: `phase-spec.md`
- PRD: `docs/prds/payroll.md`
- Bominal ADR-0210 (M3 scope), ADR-0126 (8-class enum), ADR-0028 (audit chain)
- ADR-0056 (BNF v4.1)
