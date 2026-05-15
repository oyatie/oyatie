---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-first-paying-tenant
phase: P03-accounting
impl_plan_id: IP-P03-accounting-full-scaffold
status: pending
owner: council-enterprise
blocked_by:
- impl_plan: IP-P02-payroll-full-scaffold
  reason: PayrollRunCompleted Workflow event + PayrollEntry Ontology Object Type must
    be registered before accounting Workflow consumer can be wired.
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
# IP-P03-accounting-full-scaffold: Accounting µservice — K-GAAP COA, double-entry ledger, auto-journal from PayrollRunCompleted, period-close, Typst financial statements

## Intent

Scaffolds the complete `oya-accounting-*` µservice: Postgres DDL for chart-of-accounts + ledger (append-only) + period-close + reporting with Citus sharding + RLS + outbox; Rust kernel port traits; K-GAAP double-entry domain (balance invariant, auto-journal construction from PayrollEntry Ontology reads); Typst financial statement renderer (income statement, balance sheet, cash flow); period-end close with lock enforcement; Cedar policy pack; Protobuf events; load tests. Translates Bominal ADR-0120 to `oya-finance-library-*` dependency.

---

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `crates/oya-accounting-chart-kernel/Cargo.toml` | create | deps: `async-trait`, `serde`, `uuid` |
| `crates/oya-accounting-chart-kernel/src/types.rs` | create | `AccountId(Uuid)`, `AccountCode(String)`, `AccountClass` K-GAAP hierarchy enum |
| `crates/oya-accounting-chart-kernel/src/ports.rs` | create | `ChartRepository` sealed port trait |
| `crates/oya-accounting-ledger-kernel/Cargo.toml` | create | deps + `oya-finance-library-domain` |
| `crates/oya-accounting-ledger-kernel/src/types.rs` | create | `EntryId(Uuid)`, `LineId(Uuid)`, `FiscalPeriod(year, month)` |
| `crates/oya-accounting-ledger-kernel/src/ports.rs` | create | `JournalRepository`, `TrialBalanceStore` sealed port traits |
| `crates/oya-accounting-ledger-domain/Cargo.toml` | create | deps: ledger-kernel + chart-kernel + `oya-finance-library-domain` |
| `crates/oya-accounting-ledger-domain/src/journal_entry.rs` | create | `JournalEntry` aggregate + `JournalLine` entity |
| `crates/oya-accounting-ledger-domain/src/double_entry.rs` | create | `DoubleEntryInvariant` — validates debit sum == credit sum |
| `crates/oya-accounting-ledger-domain/src/auto_journal.rs` | create | `PayrollJournalBuilder` — constructs journal from `PayrollEntry` Ontology read |
| `crates/oya-accounting-period-close-kernel/Cargo.toml` | create | deps |
| `crates/oya-accounting-period-close-kernel/src/types.rs` | create | `PeriodId(Uuid)`, `PeriodStatus` enum |
| `crates/oya-accounting-period-close-kernel/src/ports.rs` | create | `PeriodCloseRepository` sealed port trait |
| `crates/oya-accounting-period-close-domain/src/fiscal_period.rs` | create | `FiscalPeriod` aggregate + `PeriodLock` — prevents backdated entries |
| `crates/oya-accounting-period-close-domain/src/reversing_entry.rs` | create | `ReversingEntry` — auto-generates reversing entries on period open |
| `crates/oya-accounting-reporting-kernel/Cargo.toml` | create | deps |
| `crates/oya-accounting-reporting-kernel/src/ports.rs` | create | `FinancialStatementRenderer`, `StatementRepository` sealed port traits |
| `crates/oya-accounting-reporting-domain/src/financial_statement.rs` | create | `FinancialStatement` aggregate — income statement, balance sheet, cash flow |
| `crates/oya-accounting-reporting-domain/src/kgaap_rollup.rs` | create | K-GAAP account rollup rules (기업회계기준서 SME subset) |
| `crates/oya-accounting-chart-adapter/src/postgres_chart_repository.rs` | create | `PostgresChartRepository` implements `ChartRepository` |
| `crates/oya-accounting-ledger-adapter/src/postgres_journal_repository.rs` | create | `PostgresJournalRepository` implements `JournalRepository` |
| `crates/oya-accounting-ledger-adapter/src/ontology_journal_writer.rs` | create | `OntologyJournalWriter` writes `JournalEntry` Object Type to Ontology |
| `crates/oya-accounting-period-close-adapter/src/postgres_period_close_repository.rs` | create | `PostgresPeriodCloseRepository` implements `PeriodCloseRepository` |
| `crates/oya-accounting-reporting-adapter/src/typst_statement_renderer.rs` | create | `TypstStatementRenderer` implements `FinancialStatementRenderer` |
| `crates/oya-accounting-reporting-adapter/src/postgres_statement_repository.rs` | create | `PostgresStatementRepository` implements `StatementRepository` |
| `crates/oya-accounting-ledger-application/src/payroll_journal_consumer.rs` | create | Workflow event consumer: `PayrollRunCompleted` → `PayrollJournalBuilder` → `JournalRepository::save` |
| `crates/oya-accounting-app/src/main.rs` | create | DI assembly; Axum server + Workflow consumer worker |
| `migrations/accounting/001_accounting_schema.sql` | create | Full DDL (see below) |
| `contracts/accounting.openapi.yaml` | create | OpenAPI 3.1 for `/v1/journal-entries`, `/v1/trial-balance`, `/v1/period-close`, `/v1/financial-statements` |
| `proto/accounting/events.proto` | create | `PeriodClosed`, `TrialBalanceImbalanceDetected` |
| `policies/accounting/accounting.cedar` | create | Cedar policy pack |
| `typst-templates/financial-statements/income-statement.typ` | create | Typst income statement template (K-GAAP format) |
| `typst-templates/financial-statements/balance-sheet.typ` | create | Typst balance sheet template |
| `tests/load/smoke-accounting-trial-balance.js` | create | k6: p99 ≤500ms at 1k RPS |
| `tests/load/smoke-accounting-journal-write.js` | create | k6: p99 ≤150ms |
| `Cargo.toml` | update | Add all `oya-accounting-*` crates |
| `docs/standards/bounded-contexts.md` | update | Register chart/ledger/period-close/reporting BCs |

---

## Code Shape

### `crates/oya-accounting-ledger-domain/src/double_entry.rs`

```rust
/// K-GAAP double-entry invariant
/// Statute: 대한민국.회계.기업회계기준서 (K-GAAP SME; 한국회계기준원)
pub struct DoubleEntryInvariant;

impl DoubleEntryInvariant {
    /// Validates that sum of debit lines == sum of credit lines
    /// Returns Err(AccountingError::ImbalancedEntry) if violated
    pub fn validate(lines: &[JournalLine]) -> Result<(), AccountingError> {
        let debit_total: Decimal = lines.iter()
            .filter(|l| l.side == LedgerSide::Debit)
            .map(|l| l.amount.amount)
            .sum();
        let credit_total: Decimal = lines.iter()
            .filter(|l| l.side == LedgerSide::Credit)
            .map(|l| l.amount.amount)
            .sum();
        if debit_total != credit_total {
            return Err(AccountingError::ImbalancedEntry { debit_total, credit_total });
        }
        Ok(())
    }
}
```

### `crates/oya-accounting-ledger-domain/src/auto_journal.rs`

```rust
/// Builds K-GAAP journal entries from PayrollRunCompleted Workflow event
/// Standard journal template (per K-GAAP):
///   DR 급여비용 (salary expense)
///   DR 4대보험 사용자부담금 (employer insurance expense)
///     CR 미지급급여 (accrued salary payable)
///     CR 4대보험 예수금 (employee insurance withholding payable)
///     CR 근로소득세 예수금 (income tax withholding payable)
///     CR 4대보험 사용자부담금 예수금 (employer insurance payable)
pub struct PayrollJournalBuilder {
    chart: Arc<dyn ChartRepository>,
}

impl PayrollJournalBuilder {
    pub async fn build_from_payroll_run(
        &self,
        tenant_id: &TenantId,
        run: &PayrollRunSummary,  // read from PayrollEntry Ontology objects
        period: FiscalPeriod,
    ) -> Result<JournalEntry, AccountingError> {
        // 1. Resolve account codes from K-GAAP COA (급여비용=511, 미지급급여=253, etc.)
        // 2. Build JournalLine vec: debit salary expense + employer insurance
        // 3. Build JournalLine vec: credit payables
        // 4. Call DoubleEntryInvariant::validate
        // 5. Return JournalEntry { lines, description, period, source: PayrollRun }
    }
}
```

### `crates/oya-accounting-chart-kernel/src/types.rs`

```rust
/// K-GAAP chart of accounts hierarchy
/// Per 기업회계기준서 (K-IFRS for SME; KASB 2024)
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AccountClass {
    /// 자산 (Assets)
    Asset,
    /// 부채 (Liabilities)
    Liability,
    /// 자본 (Equity)
    Equity,
    /// 수익 (Revenue)
    Revenue,
    /// 비용 (Expenses)
    Expense,
}

/// K-GAAP default account codes (SME subset)
/// Full template seeded in migrations/accounting/002_kgaap_coa_seed.sql
pub const KGAAP_DEFAULT_ACCOUNTS: &[(&str, &str, AccountClass)] = &[
    ("101", "현금및현금성자산", AccountClass::Asset),
    ("110", "매출채권", AccountClass::Asset),
    ("251", "매입채무", AccountClass::Liability),
    ("253", "미지급급여", AccountClass::Liability),
    ("254", "4대보험예수금", AccountClass::Liability),
    ("255", "근로소득세예수금", AccountClass::Liability),
    ("301", "자본금", AccountClass::Equity),
    ("401", "매출액", AccountClass::Revenue),
    ("511", "급여비용", AccountClass::Expense),
    ("512", "4대보험사용자부담금", AccountClass::Expense),
    // ... (full list in seed migration)
];
```

---

## Postgres DDL

### migrations/accounting/001_accounting_schema.sql

```sql
CREATE SCHEMA IF NOT EXISTS accounting;

CREATE TYPE accounting.account_class AS ENUM ('asset','liability','equity','revenue','expense');
CREATE TYPE accounting.ledger_side AS ENUM ('debit','credit');
CREATE TYPE accounting.period_status AS ENUM ('open','closed','locked');

-- Chart of accounts
CREATE TABLE accounting.accounts (
    account_id      uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       uuid NOT NULL,
    account_code    text NOT NULL,
    account_name    text NOT NULL,
    account_class   accounting.account_class NOT NULL,
    parent_account_id uuid NULL REFERENCES accounting.accounts(account_id),
    is_system       bool NOT NULL DEFAULT false,  -- true = K-GAAP default; not deletable
    jurisdiction_code text NOT NULL DEFAULT 'KR',
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE accounting.accounts ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON accounting.accounts
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE UNIQUE INDEX idx_account_code ON accounting.accounts (tenant_id, account_code);
-- SELECT create_distributed_table('accounting.accounts', 'tenant_id');

-- Fiscal periods
CREATE TABLE accounting.fiscal_periods (
    period_id       uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       uuid NOT NULL,
    period_year     int NOT NULL,
    period_month    int NOT NULL CHECK (period_month BETWEEN 1 AND 12),
    status          accounting.period_status NOT NULL DEFAULT 'open',
    closed_at       timestamptz NULL,
    closed_by       uuid NULL,   -- TenantUser who closed the period
    audit_hash      bytea NULL,  -- Ed25519 seal on close
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE accounting.fiscal_periods ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON accounting.fiscal_periods
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE UNIQUE INDEX idx_period ON accounting.fiscal_periods (tenant_id, period_year, period_month);

-- Journal entries (append-only ledger)
-- K-GAAP: every entry double-posted; imbalanced entries rejected at domain layer
CREATE TABLE accounting.journal_entries (
    entry_id        uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       uuid NOT NULL,
    period_id       uuid NOT NULL REFERENCES accounting.fiscal_periods(period_id),
    description     text NOT NULL,
    source_type     text NOT NULL CHECK (source_type IN ('payroll','manual','system','reversing')),
    source_id       uuid NULL,   -- run_id for payroll source
    is_reversing    bool NOT NULL DEFAULT false,
    reversal_of_entry_id uuid NULL REFERENCES accounting.journal_entries(entry_id),
    audit_hash      bytea NULL,  -- Ed25519 seal
    created_by      uuid NOT NULL,
    created_at      timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE accounting.journal_entries ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON accounting.journal_entries
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE INDEX idx_je_period ON accounting.journal_entries (tenant_id, period_id);
CREATE INDEX idx_je_source ON accounting.journal_entries (tenant_id, source_type, source_id)
    WHERE source_id IS NOT NULL;
-- Append-only enforcement: no UPDATE or DELETE permitted after audit_hash is set
-- Enforced via trigger: prevent_journal_mutation

-- Journal lines (debit/credit pairs)
CREATE TABLE accounting.journal_lines (
    line_id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       uuid NOT NULL,
    entry_id        uuid NOT NULL REFERENCES accounting.journal_entries(entry_id),
    account_id      uuid NOT NULL REFERENCES accounting.accounts(account_id),
    side            accounting.ledger_side NOT NULL,
    amount          numeric(20,2) NOT NULL CHECK (amount > 0),
    description     text NULL,
    created_at      timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE accounting.journal_lines ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON accounting.journal_lines
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE INDEX idx_jl_entry ON accounting.journal_lines (tenant_id, entry_id);
CREATE INDEX idx_jl_account ON accounting.journal_lines (tenant_id, account_id);

-- Outbox
CREATE TABLE accounting.outbox (
    outbox_id   bigserial PRIMARY KEY,
    tenant_id   uuid NOT NULL,
    topic       text NOT NULL,
    key         text NOT NULL,
    payload     jsonb NOT NULL,
    published_at timestamptz NULL,
    created_at  timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_accounting_outbox_unpublished ON accounting.outbox (created_at)
    WHERE published_at IS NULL;

-- Prevent mutation of sealed journal entries
CREATE OR REPLACE FUNCTION accounting.prevent_journal_mutation()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    IF OLD.audit_hash IS NOT NULL THEN
        RAISE EXCEPTION 'Cannot modify sealed journal entry %', OLD.entry_id;
    END IF;
    RETURN NEW;
END;
$$;
CREATE TRIGGER prevent_journal_mutation_trigger
    BEFORE UPDATE OR DELETE ON accounting.journal_entries
    FOR EACH ROW EXECUTE FUNCTION accounting.prevent_journal_mutation();
```

---

## Protobuf Event Schemas

```proto
syntax = "proto3";
package accounting.events;

// Kafka topic: accounting.{tenant_id}.PeriodClosed
message PeriodClosed {
    string tenant_id = 1;
    string period_id = 2;
    int32  period_year = 3;
    int32  period_month = 4;
    string closed_by = 5;
    string audit_hash = 6;   // Ed25519 seal hex
    int64  occurred_at_ms = 7;
}

// Kafka topic: accounting.{tenant_id}.TrialBalanceImbalanceDetected
message TrialBalanceImbalanceDetected {
    string tenant_id = 1;
    string entry_id = 2;
    string debit_total = 3;
    string credit_total = 4;
    int64  occurred_at_ms = 5;
}
```

---

## Cedar Policy Pack

```cedar
// accounting/accounting.cedar
entity Tenant;
entity Accountant in [Tenant];
entity Auditor in [Tenant];

// Accountant can post journal entries and close periods
permit (
    principal is Accountant,
    action in [Action::"PostJournalEntry", Action::"ClosePeriod",
               Action::"OpenPeriod", Action::"GenerateStatement"],
    resource
) when { context.tenant_id == principal.tenant_id };

// Sealed periods: no backdated entries
forbid (
    principal,
    action == Action::"PostJournalEntry",
    resource
) when {
    resource.period_status == "locked" ||
    resource.period_status == "closed"
};

// No cross-tenant access
forbid (principal, action, resource) when {
    context.tenant_id != resource.tenant_id
};
```

---

## Acceptance Gates

```bash
cargo check -p oya-accounting-app --all-features  # exit 0
cargo nextest run -p oya-accounting-ledger-domain --test double_entry_invariant  # exit 0
cargo nextest run --test test_payroll_to_accounting_journal  # exit 0; entries correct
cargo nextest run -p oya-accounting-period-close-domain  # exit 0; lock enforced
cargo nextest run -p oya-accounting-reporting-domain  # exit 0; income statement + balance sheet
oya gate validate lean-a2 --ms accounting  # no imports from payroll/hr
oya gate validate audit-chain --ms accounting
k6 run tests/load/smoke-accounting-trial-balance.js  # p(99)<500
```

---

## Test Plan

| Test | Verifies |
|---|---|
| `test_double_entry_invariant` | Balanced entry accepted; imbalanced entry → `ImbalancedEntry` error |
| `test_payroll_to_accounting_journal` | `PayrollRunCompleted` event → correct debit/credit lines (급여비용 DR, 미지급급여 CR, etc.) |
| `test_period_close_lock` | Journal entry to closed period → `PeriodLocked` error |
| `test_backdated_entry_rejected` | Entry with date in closed period rejected |
| `test_trial_balance_equilibrium` | Sum of all debits == sum of all credits across all entries in open period |
| `test_income_statement_generation` | Revenue - Expense = Net Income; matches expected fixture |
| `test_balance_sheet_generation` | Assets = Liabilities + Equity; K-GAAP format |
| `test_kgaap_coa_default_seed` | Default K-GAAP accounts seeded correctly; 급여비용=511 etc. |

### Load tests

```javascript
// tests/load/smoke-accounting-trial-balance.js
export const options = {
  vus: 100,
  duration: '60s',
  thresholds: {
    http_req_duration: ['p(99)<500'],  // PRD: trial balance p99 ≤500ms (100k entries)
    http_req_failed: ['rate<0.001'],
  },
};
```

---

## Grit Symbol-Locks

```bash
grit claim \
  --agent ip-p03-accounting \
  --intent "P03-accounting: K-GAAP double-entry ledger, auto-journal from PayrollRunCompleted, period-close, Typst financial statements" \
  --ttl 3600 \
  crates/oya-accounting-ledger-kernel/src/ports.rs::JournalRepository \
  crates/oya-accounting-ledger-domain/src/double_entry.rs::DoubleEntryInvariant \
  crates/oya-accounting-ledger-domain/src/auto_journal.rs::PayrollJournalBuilder \
  crates/oya-accounting-period-close-domain/src/fiscal_period.rs::PeriodLock \
  crates/oya-accounting-reporting-kernel/src/ports.rs::FinancialStatementRenderer \
  migrations/accounting/001_accounting_schema.sql::accounting.journal_entries \
  proto/accounting/events.proto::PeriodClosed
```

---

## ICM Rows to Emit

```bash
icm store \
  -t context-oyatie \
  -c "IP-P03-accounting-full-scaffold merged; K-GAAP COA + double-entry ledger + append-only sealed entries + auto-journal from PayrollRunCompleted + period-close lock + Typst financial statements; LEAN lanes green; next: IP-P04-connect-pro-mail" \
  -i high \
  -k "M03,P03,IP-P03-accounting-full-scaffold,accounting,kgaap,double-entry"
```

---

## Halt Conditions

1. `test_double_entry_invariant` fails because `oya-finance-library-domain::Money` arithmetic has rounding discrepancy — do not patch the test; investigate `Money::add` in finance-library-domain; escalate if root cause is in the library crate.
2. `test_payroll_to_accounting_journal` K-GAAP account code mapping fails — the default COA seed migration must run before the test; ensure migration order is correct.
3. LEAN-A2 violation — accounting imports `oya-payroll-*` directly; fix: consume `PayrollRunCompleted` via Workflow event consumer, read `PayrollEntry` via `oya-ontology-entity-kernel::ObjectStore` only.

---

## Next IP Pointer

`phases/P04-connect-pro-mail/impl-plan.md`

---

## Cross-References

- PRD: `docs/prds/accounting.md`
- Bominal ADR-0120 (platform-finance library → oya-finance-library-*), ADR-0018 (RLS), ADR-0028 (audit chain)
- ADR-0056 (BNF v4.1)
