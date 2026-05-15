---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M03-first-paying-tenant
phase: P03-accounting
status: Proposed
acceptance_lanes: []
entry_gate: 'M03/P02-payroll complete; PayrollRunCompleted Workflow event registered;

  PayrollEntry Object Type registered in Ontology;

  oya-finance-library-domain ships (M02-P11);

  oya-workflow-engine-kernel ships.

  '
exit_gate: "All IP acceptance gates green; `cargo nextest run -p oya-accounting-*`\
  \ 0 failures;\ndouble-entry invariant test green; auto-journal from PayrollRunCompleted\
  \ green;\nperiod-close lock enforced; financial statements generated for closed\
  \ period;\n`oya gate validate lean-a2 --ms accounting` exits 0;\n`oya gate validate\
  \ audit-chain --ms accounting` exits 0;\nk6 smoke trial balance p99 \u2264500ms\
  \ at 1k RPS;\ngrit done on all P03 symbols; ICM phase-handoff row emitted.\n"
depends_on:
- milestone: M03
  phase: P02-payroll
  reason: Accounting primary event source is PayrollRunCompleted; PayrollEntry Ontology
    reads needed for journal-line generation.
- milestone: M02
  phase: P11-finance-library
  reason: oya-finance-library-domain provides Money/KRW arithmetic + double-entry
    balance invariant primitives.
parallel_wave: 3
owner_team: council-enterprise
purpose: Auto-backfilled purpose for phase-spec.md
---
# P03-accounting: Accounting µservice — K-GAAP double-entry ledger, auto-journal from Payroll, financial statements

## Purpose

Delivers the `oya-accounting-*` µservice: K-GAAP double-entry bookkeeping with
automatic journal entry generation from Payroll events via Workflow, chart of
accounts CRUD, period-end closing (월마감/연마감), and financial statement generation
(income statement, balance sheet, cash flow — Typst PDF). Consumes `PayrollRunCompleted`
from Workflow; writes `JournalEntry` / `FiscalPeriod` to Ontology; emits `PeriodClosed`
back to Workflow for next-period gate.

Translates Bominal ADR-0120 (platform-finance library) to oyatie glossary:
`platform → shared`; the finance library is `oya-finance-library-*` consumed via
dependency, not re-implemented here.

---

## Scope

### In-scope

| µservice | Bounded Contexts | Crate family (BNF v4.1) |
|---|---|---|
| `accounting` | `chart` | `oya-accounting-chart-{kernel,domain,application,adapter,rest}` |
| `accounting` | `ledger` | `oya-accounting-ledger-{kernel,domain,application,adapter,rest}` |
| `accounting` | `period-close` | `oya-accounting-period-close-{kernel,domain,application,adapter}` |
| `accounting` | `reporting` | `oya-accounting-reporting-{kernel,domain,application,adapter,rest}` |
| `accounting` | `app` | `oya-accounting-app` |

Naming justifications:

```
NAME: oya-accounting-chart-kernel
JUSTIFICATION:
- microservice = accounting: Accounting µservice; registered; ADR-0056 v4.1
- bc-tokens = chart: accounting has multiple BCs (chart/ledger/period-close/reporting); chart BC owns Account entity + AccountClass + K-GAAP COA hierarchy + ChartRepository port-trait; ADR-0056 v4.1 BC-optionality
- layer = kernel: pure AccountId/AccountCode value types + ChartRepository port declaration; zero logic; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-accounting-ledger-kernel
JUSTIFICATION:
- microservice = accounting; bc-tokens = ledger: ledger BC owns JournalEntry aggregate + JournalLine + double-entry balance invariant + JournalRepository port-trait; ADR-0056 v4.1 BC-optionality
- layer = kernel: pure EntryId + JournalRepository port declaration; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-accounting-ledger-domain
JUSTIFICATION:
- microservice = accounting; bc-tokens = ledger; layer = domain: double-entry balance invariant logic; auto-journal construction from PayrollEntry reads; JournalEntry aggregate + debit/credit pair validation; calls through JournalRepository; no I/O; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-accounting-period-close-kernel
JUSTIFICATION:
- microservice = accounting; bc-tokens = period-close: period-close BC owns FiscalPeriod entity + PeriodLock + PeriodCloseRepository port-trait; ADR-0056 v4.1 BC-optionality
- layer = kernel: pure FiscalPeriodId + PeriodCloseRepository port declaration; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-accounting-reporting-kernel
JUSTIFICATION:
- microservice = accounting; bc-tokens = reporting: reporting BC owns FinancialStatement entity + Typst report renderer port + StatementRepository port-trait; ADR-0056 v4.1 BC-optionality
- layer = kernel: pure StatementId + FinancialStatementRenderer port declaration; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-accounting-app
JUSTIFICATION:
- microservice = accounting; bc-tokens: OMITTED — composition-root; ADR-0056 §"BC optionality"
- layer = app: main.rs + DI wiring; ADR-0056 §"Layer semantics"
- exemptions: none
```

### Out-of-scope

- ClickHouse replica for high-scale analytics — deferred to M04 per PRD-accounting open question #2.
- 법인세 신고 export (corporate tax return format per 국세청) — deferred to M04 (PRD FR-06 priority: Should).
- Multi-currency (USD/JPY/EUR) — post-M03; KRW primary only at M03.
- Procurement/invoice accounting (`ProcurementInvoiceApproved` handler) — deferred to M04.

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`impl-plan.md`](impl-plan.md) | Full accounting µservice: K-GAAP COA DDL, double-entry ledger domain, auto-journal Workflow consumer, period-close enforcement, Typst financial statements, ClickHouse read-replica schema stub, Ontology writes, load tests | pending | council-enterprise |

---

## Acceptance Gates

### Cargo / CI gates

```bash
cargo check --workspace --all-features                                           # exit 0
cargo build -p oya-accounting-app --all-features                                 # exit 0
cargo clippy -p oya-accounting-ledger-domain -p oya-accounting-period-close-domain -- -D warnings  # exit 0
cargo nextest run -p oya-accounting-ledger-domain --test double_entry_invariant  # exit 0; imbalanced entry rejected
cargo nextest run -p oya-accounting-period-close-domain                          # exit 0; period lock enforced
cargo nextest run -p oya-accounting-reporting-domain                             # exit 0; income statement + balance sheet generated
cargo nextest run -p oya-accounting-chart-domain                                 # exit 0; K-GAAP COA default loaded
cargo deny check                                                                 # exit 0
```

### Integration gates

```bash
# Auto-journal from PayrollRunCompleted Workflow event
cargo nextest run --test test_payroll_to_accounting_journal  # exit 0; entries match salary + tax + insurance payables
```

### Fitness lane gates

```bash
oya gate validate lean-a2 --ms accounting        # no imports from payroll/hr/procurement
oya gate validate lean-a1 --ms accounting        # layer ordering
oya gate validate port-location --ms accounting  # port traits in kernel
oya gate validate shardability --ms accounting   # tenant_id + fiscal_period composite partition key
oya gate validate audit-chain --ms accounting    # Ed25519 seal per (tenant_id, fiscal_period)
oya gate validate jurisdiction-overlay --ms accounting  # jurisdiction_code=KR
```

### Performance gates

```bash
# k6: trial balance query p99 ≤500ms at 1k RPS (100k entries)
k6 run tests/load/smoke-accounting-trial-balance.js --env BASE_URL=http://localhost:8083
# Pass: http_req_duration{p(99)}<500; error rate <0.1%

# k6: journal entry write p99 ≤150ms
k6 run tests/load/smoke-accounting-journal-write.js --env BASE_URL=http://localhost:8083
```

---

## Clean Architecture Compliance

### Layer assignments

| Crate | Layer | Port traits in kernel? | Impls in adapter? |
|---|---|---|---|
| `oya-accounting-chart-kernel` | `kernel` | Yes — `ChartRepository` | N/A |
| `oya-accounting-chart-domain` | `domain` | N/A | N/A |
| `oya-accounting-chart-adapter` | `adapter` | N/A | Yes — `PostgresChartRepository` |
| `oya-accounting-ledger-kernel` | `kernel` | Yes — `JournalRepository`, `TrialBalanceStore` | N/A |
| `oya-accounting-ledger-domain` | `domain` | N/A | N/A |
| `oya-accounting-ledger-adapter` | `adapter` | N/A | Yes — `PostgresJournalRepository`, `OntologyJournalWriter` |
| `oya-accounting-period-close-kernel` | `kernel` | Yes — `PeriodCloseRepository` | N/A |
| `oya-accounting-period-close-adapter` | `adapter` | N/A | Yes — `PostgresPeriodCloseRepository` |
| `oya-accounting-reporting-kernel` | `kernel` | Yes — `FinancialStatementRenderer`, `StatementRepository` | N/A |
| `oya-accounting-reporting-adapter` | `adapter` | N/A | Yes — `TypstStatementRenderer`, `PostgresStatementRepository` |
| `oya-accounting-app` | `app` | N/A | Unrestricted inward |

Cross-product: accounting NEVER imports `oya-payroll-*` or `oya-hr-*`.
PayrollEntry reads via `oya-ontology-entity-kernel::ObjectStore` port only.

### New BCs registered in this phase

| BC name | Owner µservice | Registration PR |
|---|---|---|
| `chart` | `accounting` | pending |
| `ledger` | `accounting` | pending |
| `period-close` | `accounting` | pending |
| `reporting` | `accounting` | pending |

---

## Grit Claim Symbols

```
crates/oya-accounting-ledger-kernel/src/ports.rs::JournalRepository
crates/oya-accounting-ledger-domain/src/journal_entry.rs::JournalEntry
crates/oya-accounting-ledger-domain/src/double_entry.rs::DoubleEntryInvariant
crates/oya-accounting-period-close-domain/src/period_lock.rs::PeriodLock
crates/oya-accounting-reporting-kernel/src/ports.rs::FinancialStatementRenderer
contracts/accounting.openapi.yaml::postJournalEntry
contracts/accounting.openapi.yaml::closePeriod
docs/standards/bounded-contexts.md::accounting.ledger
```

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "Phase P03-accounting started; depends on P02-payroll + M02 finance-library; scope: K-GAAP double-entry, auto-journal from PayrollRunCompleted, period-close, Typst financial statements" \
  -i high \
  -k "M03,P03,phase-start,accounting"

icm store \
  -t context-oyatie \
  -c "Phase P03-accounting complete; K-GAAP COA + double-entry ledger + auto-journal from Payroll + period-close + financial statements shipped; PeriodClosed Workflow event; next: P04-connect-pro-mail" \
  -i high \
  -k "M03,P03,phase-complete,accounting"
```

---

## References

- PRD: `docs/prds/accounting.md`
- Bominal ADRs inherited: ADR-0120 (platform-finance library → oya-finance-library-*), ADR-0018 (tenancy RLS), ADR-0028 (audit chain), ADR-0119 (data tier)
- oyatie ADRs: ADR-0056 (BNF v4.1)
