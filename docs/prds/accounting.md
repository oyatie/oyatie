---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-accounting
microservice: accounting
status: Accepted
sales_segment: Enterprise
tier: B2B
milestone_first_ship: M03-first-paying-tenant
bominal_source:
  - ADR-0120  # platform-finance library (translated: finance-library-*)
  - ADR-0018  # tenancy RLS posture
  - ADR-0028  # audit chain Merkle/Ed25519
  - ADR-0119  # data tier assignment matrix
doc_status: published
---

# PRD-accounting: Accounting µservice

---

## Purpose

The Accounting µservice provides K-GAAP double-entry bookkeeping for Korean
corporate tenants. It is the authoritative ledger: receives journal entry events
from Payroll, Procurement, and other µservices via Workflow; maintains the
chart of accounts; produces financial statements; and supports period-end closing.

Inherits from Bominal ADR-0120 (platform-finance library), translated to oyatie
glossary: `platform → shared`; the finance library becomes `finance-library-*`
(shared substrate) consumed by the Accounting µservice's infrastructure layer.
No Bominal overrides beyond glossary translation.

---

## Tenant Value

- **K-GAAP double-entry ledger**: every financial event double-posted; trial
  balance always in equilibrium; no manual reconciliation.
- **Automated journal entries from Payroll**: payroll run completion triggers
  journal entries (salary expense, withholding tax payable, insurance payable)
  without HR staff involvement.
- **Financial statements**: income statement, balance sheet, cash flow statement
  generated on demand; period-end closing enforced.
- **Audit-grade trail**: every journal entry cryptographically sealed;
  admissible for tax authority examination.
- **Multi-currency ready**: KRW primary; USD/JPY/EUR support for multinational
  tenants (post-M03).

---

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | Accountant | define chart of accounts (assets, liabilities, equity, revenue, expenses) per K-GAAP hierarchy | all journal entries post to the correct account codes | `chart-of-accounts` | Must |
| FR-02 | System | receive payroll journal entries automatically when `PayrollRunCompleted` fires | no manual data entry from payroll to accounting | `ledger` | Must |
| FR-03 | Accountant | post manual journal entries with debit/credit pairs; system validates balance | trial balance stays in equilibrium | `ledger` | Must |
| FR-04 | Accountant | run period-end closing (월마감/연마감); lock prior periods | financial data integrity enforced; no backdated entries after close | `period-close` | Must |
| FR-05 | CFO | generate income statement, balance sheet, cash flow statement for any closed period | GAAP-compliant reports for board / tax filing | `reporting` | Must |
| FR-06 | Tax accountant | export 법인세 신고 data (corporate tax return support) | tax return preparation reduced from days to hours | `reporting` | Should |
| FR-07 | Auditor | access full journal entry audit chain per period | tax authority examination satisfied without reconstruction | `audit` | Must |

---

## Non-Functional Requirements

### Performance
- P99 journal entry write: ≤200 ms (includes double-entry validation).
- P99 trial balance query: ≤500 ms for 100k entries.
- Period-end close for 1 month, 10k entries: ≤60 s.

### Security
- JWT `tenant_id` enforced; ledger data never cross-tenant (ADR-0018).
- Period-close lock: only authorized accountants (Cedar policy) can close periods.
- Journal entries immutable after period close; amendment creates reversing entry.

### Audit + Compliance
- Every journal entry Merkle/Ed25519 sealed per (tenant_id, fiscal_period) per ADR-0028.
- K-GAAP: Korean Generally Accepted Accounting Principles; chart of accounts
  per 기업회계기준서 (K-IFRS subset for SME).
- 법인세법: 5-year record retention; immutable ledger satisfies requirement.
- Jurisdiction overlay `KR` per ADR-0127.

### Availability + SLO
- 99.9% monthly. Period-close windows (month-end, year-end) treated as high priority.
- RTO ≤30 s; RPO ≤5 s.

---

## Bounded Contexts

| BC name | Crate family (BNF v4.1) | Purpose | Key entities |
|---|---|---|---|
| `chart-of-accounts` | `accounting-chart-{domain,application,infrastructure,rest}` | COA definition; K-GAAP hierarchy; account code registry | `Account`, `AccountClass` |
| `ledger` | `accounting-ledger-{domain,application,infrastructure,rest}` | Double-entry journal entries; trial balance; general ledger | `JournalEntry`, `JournalLine` |
| `period-close` | `accounting-period-close-{domain,application,infrastructure}` | Period opening/closing; lock enforcement; reversing entries | `FiscalPeriod`, `PeriodLock` |
| `reporting` | `accounting-reporting-{domain,application,infrastructure,rest}` | Financial statement generation; export formats | `FinancialStatement` |

```
NAME: accounting-ledger-domain
JUSTIFICATION:
- microservice = accounting: Accounting µservice; flat catalog; ADR-0056 v4.1
- bc-tokens = ledger: accounting has multiple BCs (chart-of-accounts/ledger/period-close/reporting); ledger BC owns JournalEntry entity + double-entry validation rules; ADR-0056 v4.1 BC-optionality
- layer = domain: JournalEntry entity + debit/credit balance invariant + JournalRepository port-trait; no I/O; ADR-0056 §"Layer semantics"
- exemptions: none
```

Shared finance library dependency:
```
NAME: finance-library-domain  (shared substrate consumed by accounting infrastructure)
JUSTIFICATION:
- microservice = finance-library: shared substrate for monetary types, currency arithmetic,
  rounding rules; consumed by accounting + payroll + payments; flat catalog; ADR-0056 v4.1
- bc-tokens: omitted — single concept (monetary/currency utilities) at this layer
- layer = domain: Money value type + currency arithmetic + rounding rules; no I/O
- exemptions: none
```

---

## Integration via Workflow + Ontology

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `PayrollRunCompleted` | `payroll` | `ledger` | Auto-post payroll journal entries (salary expense + tax/insurance payables) |
| `EmployeeTerminated` | `hr` | `ledger` | Post severance accrual reversal |
| `ProcurementInvoiceApproved` | `procurement` (M04+) | `ledger` | Post accounts payable entry |

### Workflow events produced

| Event type | Trigger | Consumed by | State machine |
|---|---|---|---|
| `PeriodClosed` | Period-end close finalized | `reporting`, `payroll` (next period gate) | `period-close-sm` |
| `TrialBalanceImbalanceDetected` | Journal entry violates balance | `alert` | `ledger-integrity-sm` |

### Ontology writes

| Object Type | Link Type | Written by BC | Audit trail |
|---|---|---|---|
| `JournalEntry` | `PostedToAccount` → `Account` | `ledger` | Ed25519 per entry |
| `FiscalPeriod` | `HasStatus` → `PeriodStatus` | `period-close` | Ed25519 on close |

### Ontology reads

| Object Type | Read by BC | Query shape |
|---|---|---|
| `PayrollEntry` | `ledger` | `filter(tenant_id, run_id)` — to generate journal lines |
| `Department` | `ledger` | `filter(tenant_id)` — cost-center allocation |

---

## Competitive Benchmark

| Competitor | Product | Parity dimensions | Primary source |
|---|---|---|---|
| 더존비즈온 | iCUBE 회계 | K-GAAP chart of accounts; 법인세 export; period-close workflow | https://www.douzone.com |
| SAP | S/4HANA Finance | Double-entry engine; period-close orchestration; IFRS/GAAP dual reporting | https://www.sap.com |
| QuickBooks Online | QBO Advanced | Ease of use; auto journal entry from connected apps; reporting depth | https://quickbooks.intuit.com |
| Xero | Xero Accounting | API design quality; event-driven journal posting; audit trail | https://developer.xero.com |

Key parity gaps:
1. **K-GAAP COA default template**: 더존 iCUBE ships a pre-built KR corporate COA; oyatie must provide equivalent out-of-box.
2. **법인세 신고 export**: corporate tax return data export format per 국세청 standard.
3. **Reversing entry automation**: SAP/Xero auto-generate reversing entries for accruals; oyatie must support on period-open.

---

## Performance Targets

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Journal entry write | 20 ms | 150 ms | 300 ms | Includes double-entry validation |
| Trial balance query (100k entries) | 100 ms | 500 ms | 1 s | |
| Financial statement gen | 500 ms | 5 s | 15 s | Full P&L for 12 months |
| Period-close 10k entries | — | 60 s | — | Serialized; lock enforced |
| Audit chain seal | — | 1 s | — | Per (tenant_id, fiscal_period); ADR-0028 |

Error budget: 0.1% monthly. SLO burn-rate alarm: 5×.

---

## Horizontal Scalability

**State strategy**: `postgres` — ledger in append-only Postgres table; `tenant_id`
+ `fiscal_period` composite partition key; Postgres RLS; ClickHouse replica for
reporting queries at scale.

**Active-active compatibility**: `single-writer-compatible` — journal posting
serialized per (tenant, period) to enforce balance invariant.

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Max journal entries per tenant/month | 100,000 | 10,000,000 | ClickHouse replica lag > 5 s |
| Max QPS (entry writes) | 500 | 5,000 | CPU > 70% |
| Max QPS (report reads) | 1,000 | 10,000 | Memory > 80% |

Scale-out: read-heavy reporting layer (REST) HPA on CPU >70%; ledger write layer
single-writer per tenant shard. ClickHouse replica for analytics reads.
Cross-region: M03 KR only.

---

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | Double-entry invariant holds: all journal entries balance; imbalanced entry rejected | `cargo nextest run -p accounting-ledger-domain --test double_entry_invariant` |
| AC-02 | Auto-journal from `PayrollRunCompleted` event; entries posted correctly | integration test `test_payroll_to_accounting_journal` |
| AC-03 | Period-close locks prior period; backdated entry rejected after close | `cargo nextest run -p accounting-period-close-domain` |
| AC-04 | Income statement and balance sheet generated for closed period | `cargo nextest run -p accounting-reporting-domain` |
| AC-05 | LEAN-A2: no direct imports from payroll/hr/procurement | `oya gate validate lean-a2 --ms accounting` exits 0 |
| AC-06 | Trial balance query p99 ≤500 ms at 1k RPS | k6 smoke; `http_req_duration{p(99)}<500` |
| AC-07 | Audit chain sealed per (tenant, fiscal_period) | `oya gate validate audit-chain --ms accounting` |

---

## Open Questions

| # | Question | Owner | Target |
|---|---|---|---|
| 1 | K-IFRS vs K-GAAP SME: which standard as default for M03 tenants? | council-product | M03/P01 |
| 2 | ClickHouse replica: M03 or deferred to M04? | council-architecture | ADR-#### |

---

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| Bominal ADR-0120 | Platform-finance library | inherited — translated to `finance-library-*` |
| Bominal ADR-0018 | Tenancy RLS posture | inherited |
| Bominal ADR-0028 | Audit chain Merkle/Ed25519 | inherited |
| Bominal ADR-0119 | Data tier assignment matrix | inherited |
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0103 | Workflow hexagonal | integration plane |
| ADR-0106 | Ontology architecture | information plane |
