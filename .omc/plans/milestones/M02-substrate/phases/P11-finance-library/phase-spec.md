---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M02-substrate
phase: P11-finance-library
status: Proposed
acceptance_lanes: []
entry_gate: |
  M01-P05 complete; cargo check --workspace exits 0. Finance library has NO
  Postgres dependency — it is a pure computation crate; no other substrate
  phase is a hard prerequisite.
exit_gate: |
  oya-finance-library-kernel compiles with zero dependencies on any framework,
  async runtime, or database crate; Money/CurrencyCode/JournalEntry types
  compile; debits=credits invariant enforced at construction time (not audit
  time); PV/FV/NPV/IRR/XIRR/WACC/depreciation/amortization all pass Excel
  reference value round-trip tests; grit done; ICM row emitted.
depends_on:
  - milestone: M01
    phase: P05-scaffold-locks
    reason: "workspace scaffold prerequisite only"
owner_team: council-architecture
---

# P11-finance-library: Finance library substrate — pure crate, Money/CurrencyCode/JournalEntry, time-value, depreciation, amortization

## Purpose

This phase delivers `oya-finance-library-kernel` and `oya-finance-library-domain`: a pure Rust computation crate with zero persistence, zero async dependencies, and zero HTTP dependencies. Per Bominal ADR-0120, financial math must live in a shared library — not scattered across product crates and certainly not behind an HTTP API. The crate provides: `Money` + `CurrencyCode` value objects with arithmetic invariants; `JournalEntry` with the double-entry invariant (Σ debit lines = Σ credit lines) enforced at construction; time-value-of-money primitives (PV, FV, NPV, IRR, XIRR, WACC); depreciation schedules (straight-line, 150%/200% declining balance, MACRS, sum-of-years-digits); loan amortization (equal-payment, interest-only, balloon). All numeric results are validated against Excel/LibreOffice reference values in tests. Product crates (hr/payroll, medical billing, insurance, analytics) consume this via direct Rust dependency — no network hop.

---

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `finance-library` | (single concept — no BC split needed) | `crates/oya-finance-library-kernel/`, `crates/oya-finance-library-domain/` | `oya-finance-library-kernel`, `oya-finance-library-domain` |

Naming justification:

```
NAME: oya-finance-library-kernel
JUSTIFICATION:
- microservice = finance-library: shared financial math library; per Bominal ADR-0120
  "oya-kernel-finance" translated to oyatie BNF v4.1 as finance-library;
  registered in [workspace.metadata.oya.microservices]
- bc-tokens = (none): single-concept library at this layer; ADR-0056 BC-optionality
  rule — omit BC when µservice has a single concept; no split needed
- layer = kernel: Money, CurrencyCode, JournalEntry, JournalLine types + FinanceError;
  also contains pure trait definitions (JournalEntryValidator) — these are pure traits
  with no async/I/O so they belong in kernel alongside their types
- exemptions claimed: none

NAME: oya-finance-library-domain
JUSTIFICATION:
- microservice = finance-library: same µservice
- bc-tokens = (none): single-concept; ADR-0056 BC-optionality
- layer = domain: pure computation functions — PV/FV/NPV/IRR/XIRR/WACC, depreciation
  schedules, amortization schedules; all deterministic, side-effect-free; no I/O;
  no framework deps; ADR-0056 §"Layer semantics"
- exemptions claimed: none
```

### Out-of-scope

- Payroll gross-to-net calculation — owned by oya-hr-payroll-domain; uses this library.
- Chart of accounts CRUD (persistence) — owned by oya-hr-accounting-domain.
- Korean tax rate tables (MOEF brackets, 4대보험 rates) — Phase 2 surface per ADR-0120; no stub exported.
- Portfolio optimization / alpha signals — owned by external quant repo; never imported here.

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`impl-plan.md`](impl-plan.md) | Full Money/CurrencyCode/JournalEntry types + PV/FV/NPV/IRR/XIRR/WACC + depreciation + amortization + Excel reference tests | pending | `council-architecture` |

---

## Acceptance Gates

### Cargo / CI gates

```bash
cargo check --workspace --all-features               # exit 0
cargo build --workspace --all-features               # exit 0
cargo clippy --workspace --all-features -- -D warnings  # exit 0
cargo nextest run --workspace --all-features         # exit 0; 0 failures
cargo deny check                                     # exit 0
cargo doc --workspace --no-deps                      # exit 0; 0 warnings
```

### Fitness lane gates

```bash
oya gate validate lean-a1 --phase P11-finance-library
oya gate validate lean-a2 --phase P11-finance-library
oya gate validate lean-a3 --phase P11-finance-library
oya gate validate lean-a4 --phase P11-finance-library
```

### Financial correctness gates (Excel reference values)

```bash
# Money arithmetic: no floating-point rounding (uses rust_decimal or i64 cents)
cargo nextest run -p oya-finance-library-domain --test money_arithmetic_precision  # exit 0

# JournalEntry debits = credits invariant
cargo nextest run -p oya-finance-library-kernel --test journal_entry_balanced      # exit 0
# Unbalanced JournalEntry construction fails
cargo nextest run -p oya-finance-library-kernel --test journal_entry_unbalanced_rejected  # exit 0

# PV/FV reference values vs Excel PMT() / PV() / FV()
cargo nextest run -p oya-finance-library-domain --test pv_fv_excel_reference       # exit 0

# NPV reference: cash_flows=[-1000, 300, 400, 500], rate=0.10 → NPV=82.64
cargo nextest run -p oya-finance-library-domain --test npv_excel_reference         # exit 0

# IRR Newton-Raphson convergence: cash_flows=[-100, 50, 60] → IRR≈12.66%
cargo nextest run -p oya-finance-library-domain --test irr_convergence             # exit 0
# IrrNotConverged error for zero-IRR edge case
cargo nextest run -p oya-finance-library-domain --test irr_not_converged_error     # exit 0

# XIRR with irregular dates
cargo nextest run -p oya-finance-library-domain --test xirr_irregular_dates        # exit 0

# WACC: equity=0.6, debt=0.4, re=0.12, rd=0.06, tax=0.25 → WACC=8.4%
cargo nextest run -p oya-finance-library-domain --test wacc_reference              # exit 0

# Depreciation: straight-line 5-year $10k asset → $2k/year
cargo nextest run -p oya-finance-library-domain --test depreciation_straight_line  # exit 0
# Declining balance 200%: year-1 = $4k, year-2 = $2.4k, ...
cargo nextest run -p oya-finance-library-domain --test depreciation_declining_200  # exit 0
# MACRS 5-year schedule reference values
cargo nextest run -p oya-finance-library-domain --test depreciation_macrs_5yr      # exit 0

# Loan amortization: $200k, 6%, 30yr → payment=$1199.10
cargo nextest run -p oya-finance-library-domain --test amortization_equal_payment  # exit 0
# Balloon payment schedule
cargo nextest run -p oya-finance-library-domain --test amortization_balloon        # exit 0
```

### Zero-dependency gate (critical: no database/async/HTTP deps)

```bash
# Verify Cargo.toml has no sqlx/diesel/tokio/axum/reqwest dependencies
cargo metadata --manifest-path crates/oya-finance-library-kernel/Cargo.toml \
  --no-deps | jq '.packages[0].dependencies[].name' \
  | grep -v -E '"(rust_decimal|chrono|serde|thiserror|uuid)"'
# Must: empty output (only allowed deps are rust_decimal, chrono, serde, thiserror, uuid)
```

---

## Clean Architecture Compliance

### Layer assignments

| Crate (BNF v4.1) | Layer | Port traits in kernel? | Impls in adapter? | Presentation-only? |
|---|---|---|---|---|
| `oya-finance-library-kernel` | `kernel` | Yes — `JournalEntryValidator` (pure trait, no async) | N/A | No |
| `oya-finance-library-domain` | `domain` | N/A — pure computation functions | N/A | No |

### Types declared in kernel

```rust
// oya-finance-library-kernel/src/money.rs
use rust_decimal::Decimal;

/// ISO 4217 currency code. Exhaustive enum for compile-time exhaustiveness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CurrencyCode { KRW, USD, EUR, JPY, GBP, /* ... all ISO 4217 codes */ }

/// Fixed-point monetary value. Never use f32/f64 for money.
/// Amount is stored as Decimal (128-bit fixed-point) to avoid rounding drift.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Money {
    pub amount: Decimal,
    pub currency: CurrencyCode,
}

impl Money {
    pub fn new(amount: Decimal, currency: CurrencyCode) -> Self;
    pub fn add(&self, other: &Money) -> Result<Money, FinanceError>;  // rejects cross-currency
    pub fn sub(&self, other: &Money) -> Result<Money, FinanceError>;
    pub fn mul_scalar(&self, scalar: Decimal) -> Money;
    pub fn is_zero(&self) -> bool;
    pub fn is_positive(&self) -> bool;
    pub fn is_negative(&self) -> bool;
}

// oya-finance-library-kernel/src/journal.rs
/// Double-entry journal entry. Invariant: Σ debit_amounts = Σ credit_amounts.
/// Enforced at construction; `new()` returns Err if unbalanced.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JournalEntry {
    pub id: JournalEntryId,
    pub date: chrono::NaiveDate,
    pub description: String,
    pub lines: Vec<JournalLine>,   // minimum 2 lines (one debit + one credit)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JournalLine {
    pub account: AccountCode,
    pub debit: Option<Money>,
    pub credit: Option<Money>,
    // Invariant: exactly one of debit/credit is Some; never both, never neither.
}

impl JournalEntry {
    /// Constructor enforces: non-empty, each line has exactly one side,
    /// amounts non-negative, Σ debits = Σ credits (same currency).
    pub fn new(id: JournalEntryId, date: chrono::NaiveDate, description: String,
        lines: Vec<JournalLine>) -> Result<Self, FinanceError>;
    pub fn validate(&self) -> Result<(), FinanceError>;
}

pub trait JournalEntryValidator: Send + Sync {
    fn validate(&self, entry: &JournalEntry) -> Result<(), FinanceError>;
}

// oya-finance-library-kernel/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum FinanceError {
    #[error("journal entry unbalanced: debits {debits} ≠ credits {credits}")]
    JournalUnbalanced { debits: Decimal, credits: Decimal },
    #[error("cross-currency arithmetic: {lhs} and {rhs} are incompatible")]
    CrossCurrencyArithmetic { lhs: CurrencyCode, rhs: CurrencyCode },
    #[error("IRR did not converge after {iterations} iterations")]
    IrrNotConverged { iterations: u32 },
    #[error("negative amount in debit/credit line")]
    NegativeAmount,
    #[error("division by zero in financial calculation")]
    DivisionByZero,
    #[error("amortization schedule invalid: {0}")]
    InvalidAmortizationSchedule(String),
}
```

### Domain computation module signatures

```rust
// oya-finance-library-domain/src/time_value.rs
/// Present Value of annuity or lump sum. rate per period; n = periods.
pub fn pv(rate: Decimal, n: u32, fv: Decimal, pmt: Decimal, pmt_at_start: bool)
    -> Result<Decimal, FinanceError>;

pub fn fv(rate: Decimal, n: u32, pv: Decimal, pmt: Decimal, pmt_at_start: bool)
    -> Result<Decimal, FinanceError>;

/// Net Present Value of irregular cash flows.
pub fn npv(rate: Decimal, cash_flows: &[Decimal]) -> Result<Decimal, FinanceError>;

/// Internal Rate of Return via Newton-Raphson. Returns IrrNotConverged if no solution.
pub fn irr(cash_flows: &[Decimal]) -> Result<Decimal, FinanceError>;

/// XIRR: IRR for cash flows at irregular dates.
pub fn xirr(cash_flows: &[Decimal], dates: &[chrono::NaiveDate]) -> Result<Decimal, FinanceError>;

/// Weighted Average Cost of Capital.
pub fn wacc(equity_weight: Decimal, debt_weight: Decimal,
    cost_of_equity: Decimal, cost_of_debt: Decimal, tax_rate: Decimal)
    -> Result<Decimal, FinanceError>;

// oya-finance-library-domain/src/depreciation.rs
pub fn straight_line(cost: &Money, salvage: &Money, life_years: u32)
    -> Result<Vec<Money>, FinanceError>;

pub fn declining_balance(cost: &Money, salvage: &Money, life_years: u32, rate_multiplier: Decimal)
    -> Result<Vec<Money>, FinanceError>;  // rate_multiplier = 1.5 (150DB) or 2.0 (200DB/DDB)

pub fn macrs(cost: &Money, property_class: MacrsPropertyClass)
    -> Result<Vec<Money>, FinanceError>;

pub fn sum_of_years_digits(cost: &Money, salvage: &Money, life_years: u32)
    -> Result<Vec<Money>, FinanceError>;

// oya-finance-library-domain/src/amortization.rs
#[derive(Debug, Clone)]
pub struct AmortizationLine {
    pub period: u32,
    pub payment: Money,
    pub principal: Money,
    pub interest: Money,
    pub balance: Money,
}

pub fn equal_payment_schedule(principal: &Money, annual_rate: Decimal, periods: u32)
    -> Result<Vec<AmortizationLine>, FinanceError>;

pub fn interest_only_schedule(principal: &Money, annual_rate: Decimal, periods: u32,
    balloon_period: u32) -> Result<Vec<AmortizationLine>, FinanceError>;
```

### CI lanes that must green before phase exit gate

| Lane | Command | Expected |
|---|---|---|
| `dependency-direction` | `oya gate validate lean-a1 --phase P11-finance-library` | exit 0 |
| `cross-product-refusal` | `oya gate validate lean-a2 --phase P11-finance-library` | exit 0 |
| `layer-correctness` | `oya gate validate layer-correctness --phase P11-finance-library` | exit 0 |

### New BCs registered in this phase

None — `finance-library` uses BC-optionality (single concept); no BC token in crate names.

---

## Grit Claim Symbols

```
crates/oya-finance-library-kernel/src/money.rs::Money
crates/oya-finance-library-kernel/src/journal.rs::JournalEntry
crates/oya-finance-library-kernel/src/error.rs::FinanceError
crates/oya-finance-library-domain/src/time_value.rs::npv
crates/oya-finance-library-domain/src/time_value.rs::irr
crates/oya-finance-library-domain/src/depreciation.rs::macrs
crates/oya-finance-library-domain/src/amortization.rs::equal_payment_schedule
```

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "Phase P11-finance-library started; scope: 2 crates (kernel + domain); pure Rust no persistence; Bominal ADR-0120 translation; Money/JournalEntry/PV/FV/NPV/IRR/XIRR/WACC/depreciation/amortization" \
  -i high \
  -k "M02,P11,phase-start,finance-library"

icm store \
  -t context-oyatie \
  -c "Phase P11-finance-library complete; all Excel reference tests pass; JournalEntry invariant enforced at construction; zero database/async deps verified; Wave-A complete" \
  -i high \
  -k "M02,P11,phase-complete,finance-library"
```

---

## References

- Bominal ADRs inherited: ADR-0120 (platform finance library)
- oyatie ADRs: ADR-0056 (BNF v4.1)
- depends_on: M01-P05 only
- unblocks: oya-hr-payroll-domain (gross-to-net), oya-medical-billing-domain (amortization), oya-accounting-domain (journal entries), Wave-B product phases consuming financial primitives
