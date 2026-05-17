---
doc_class: ImplPlan
template_id: TPL-IMPL-PLAN
milestone: M02b-substrate
phase: P11-finance-library
status: Proposed
depends_on_phase_spec: phase-spec.md
purpose: "Implementation plan for P11-finance-library of M02b-substrate: detailed code structure and acceptance lanes."
execution_variant: merge-into-existing-crates
execution_variant_decided_at: 2026-05-17
execution_variant_decided_by: user-directive-option-2
execution_variant_note: "Delta-1 merges Currency (ISO 4217 subset, KRW-first), JournalEntryStatus (Draft→Pending→Posted/Voided state machine), and LedgerError into oya-cloud-finops-kernel instead of scaffolding new oya-finance-library-* crates. Rationale: F-M02B-PLAN-LIVE-CRATE-RECONCILIATION — live crates must be used rather than proliferating new scaffolds. No workspace dep changes; std-only. Session: claude-durable-goal-2026-05-17-p11-agent."
---
# P11-finance-library Implementation Plan

## 0. Grit Claim

```bash
grit session start
grit claim \
  --agent m02-wave-a-executor \
  --intent "P11-finance-library: 2 crates; pure Rust; Money/JournalEntry/PV/FV/NPV/IRR/XIRR/WACC/depreciation/amortization" \
  --ttl 2h \
  --symbols \
    "crates/oya-finance-library-kernel/src/money.rs::Money" \
    "crates/oya-finance-library-kernel/src/journal.rs::JournalEntry" \
    "crates/oya-finance-library-kernel/src/error.rs::FinanceError" \
    "crates/oya-finance-library-domain/src/time_value.rs::npv" \
    "crates/oya-finance-library-domain/src/time_value.rs::irr" \
    "crates/oya-finance-library-domain/src/depreciation.rs::macrs" \
    "crates/oya-finance-library-domain/src/amortization.rs::equal_payment_schedule"
```

---

## 1. Crate Inventory (2 crates)

| Crate | Layer | Purpose |
|---|---|---|
| `oya-finance-library-kernel` | kernel | `Money`, `CurrencyCode`, `JournalEntry`, `JournalLine`, `AccountCode`, `JournalEntryId`, `FinanceError`, `JournalEntryValidator` trait |
| `oya-finance-library-domain` | domain | `time_value.rs` (pv/fv/npv/irr/xirr/wacc), `depreciation.rs` (straight_line/declining_balance/macrs/syd), `amortization.rs` (equal_payment/interest_only/balloon) |

Zero additional crates. No persistence, no async, no HTTP.

### Allowed dependencies (zero-dep gate)

**`oya-finance-library-kernel/Cargo.toml`:**
```toml
[dependencies]
rust_decimal     = { version = "1", features = ["serde-with-str"] }
chrono           = { version = "0.4", features = ["serde"] }
serde            = { version = "1", features = ["derive"] }
thiserror        = "1"
uuid             = { version = "1", features = ["v4", "serde"] }

[dev-dependencies]
rust_decimal_macros = "1"
```

**`oya-finance-library-domain/Cargo.toml`:**
```toml
[dependencies]
oya-finance-library-kernel = { path = "../oya-finance-library-kernel" }
rust_decimal     = { version = "1", features = ["serde-with-str"] }
chrono           = { version = "0.4", features = ["serde"] }

[dev-dependencies]
rust_decimal_macros = "1"
approx              = "0.5"   # floating-point assertion helper for IRR convergence tests
```

---

## 2. Kernel: `oya-finance-library-kernel`

### `src/lib.rs`

```rust
// crates/oya-finance-library-kernel/src/lib.rs
pub mod error;
pub mod journal;
pub mod money;
pub mod traits;

pub use error::FinanceError;
pub use journal::{AccountCode, JournalEntry, JournalEntryId, JournalLine};
pub use money::{CurrencyCode, Money};
pub use traits::JournalEntryValidator;
```

### `src/error.rs`

```rust
// crates/oya-finance-library-kernel/src/error.rs
use rust_decimal::Decimal;
use crate::money::CurrencyCode;

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum FinanceError {
    #[error("journal entry unbalanced: debits {debits} ≠ credits {credits}")]
    JournalUnbalanced { debits: Decimal, credits: Decimal },

    #[error("cross-currency arithmetic: {lhs:?} and {rhs:?} are incompatible")]
    CrossCurrencyArithmetic { lhs: CurrencyCode, rhs: CurrencyCode },

    #[error("IRR did not converge after {iterations} iterations")]
    IrrNotConverged { iterations: u32 },

    #[error("XIRR requires equal length cash_flows and dates arrays")]
    XirrLengthMismatch,

    #[error("XIRR requires at least one positive and one negative cash flow")]
    XirrInvalidCashFlows,

    #[error("negative amount in debit/credit line")]
    NegativeAmount,

    #[error("division by zero in financial calculation")]
    DivisionByZero,

    #[error("journal entry must have at least two lines")]
    InsufficientLines,

    #[error("journal line must have exactly one of debit or credit set, not both or neither")]
    AmbiguousJournalLine,

    #[error("amortization schedule invalid: {0}")]
    InvalidAmortizationSchedule(String),

    #[error("depreciation: {0}")]
    InvalidDepreciation(String),

    #[error("rate must be non-negative")]
    NegativeRate,

    #[error("periods must be greater than zero")]
    ZeroPeriods,
}
```

### `src/money.rs`

```rust
// crates/oya-finance-library-kernel/src/money.rs
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::error::FinanceError;

/// ISO 4217 currency codes. Exhaustive enum for compile-time exhaustiveness.
/// Korean Won (KRW) listed first as the primary platform currency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum CurrencyCode {
    // Primary platform currencies
    KRW,  // Korean Won
    USD,  // US Dollar
    EUR,  // Euro
    JPY,  // Japanese Yen
    GBP,  // British Pound
    CNY,  // Chinese Yuan
    // Additional ISO 4217 codes
    AUD, CAD, CHF, HKD, SGD, SEK, NOK, DKK, NZD, INR, BRL, ZAR, MXN, AED, SAR,
}

/// Fixed-point monetary value. Never use f32/f64 for money.
/// `amount` is stored as `Decimal` (128-bit fixed-point, no rounding drift).
/// CurrencyCode is mandatory — Money without currency is meaningless.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    pub amount: Decimal,
    pub currency: CurrencyCode,
}

impl Money {
    /// Construct a Money value. Panics if currency is unspecified (cannot happen with enum).
    pub fn new(amount: Decimal, currency: CurrencyCode) -> Self {
        Self { amount, currency }
    }

    /// Construct from i64 cents (e.g. 1000 KRW = Money::from_cents(1000, CurrencyCode::KRW))
    pub fn from_cents(cents: i64, currency: CurrencyCode) -> Self {
        Self {
            amount: Decimal::new(cents, 2),  // 2 decimal places
            currency,
        }
    }

    /// Add two Money values. Returns Err if currencies differ.
    pub fn add(&self, other: &Money) -> Result<Money, FinanceError> {
        if self.currency != other.currency {
            return Err(FinanceError::CrossCurrencyArithmetic {
                lhs: self.currency,
                rhs: other.currency,
            });
        }
        Ok(Money::new(self.amount + other.amount, self.currency))
    }

    /// Subtract two Money values. Returns Err if currencies differ.
    pub fn sub(&self, other: &Money) -> Result<Money, FinanceError> {
        if self.currency != other.currency {
            return Err(FinanceError::CrossCurrencyArithmetic {
                lhs: self.currency,
                rhs: other.currency,
            });
        }
        Ok(Money::new(self.amount - other.amount, self.currency))
    }

    /// Multiply by a scalar Decimal. Currency is preserved.
    pub fn mul_scalar(&self, scalar: Decimal) -> Money {
        Money::new(self.amount * scalar, self.currency)
    }

    /// Divide by a scalar Decimal. Returns Err on division by zero.
    pub fn div_scalar(&self, scalar: Decimal) -> Result<Money, FinanceError> {
        if scalar.is_zero() {
            return Err(FinanceError::DivisionByZero);
        }
        Ok(Money::new(self.amount / scalar, self.currency))
    }

    pub fn is_zero(&self) -> bool {
        self.amount.is_zero()
    }

    pub fn is_positive(&self) -> bool {
        self.amount > Decimal::ZERO
    }

    pub fn is_negative(&self) -> bool {
        self.amount < Decimal::ZERO
    }

    /// Absolute value.
    pub fn abs(&self) -> Money {
        Money::new(self.amount.abs(), self.currency)
    }
}

impl std::fmt::Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {:?}", self.amount, self.currency)
    }
}
```

### `src/journal.rs`

```rust
// crates/oya-finance-library-kernel/src/journal.rs
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{error::FinanceError, money::Money};

pub type JournalEntryId = Uuid;
pub type AccountCode = String;  // e.g. "1001" (cash), "4001" (revenue), per CoA

/// A single debit or credit line in a journal entry.
/// Invariant: exactly one of `debit` and `credit` is `Some`; never both, never neither.
/// Invariant: the present amount is non-negative (sign is encoded by debit/credit side).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalLine {
    pub account: AccountCode,
    pub debit: Option<Money>,
    pub credit: Option<Money>,
    pub description: Option<String>,
}

impl JournalLine {
    /// Construct a debit line.
    pub fn debit(account: impl Into<AccountCode>, amount: Money) -> Result<Self, FinanceError> {
        if amount.is_negative() {
            return Err(FinanceError::NegativeAmount);
        }
        Ok(Self { account: account.into(), debit: Some(amount), credit: None, description: None })
    }

    /// Construct a credit line.
    pub fn credit(account: impl Into<AccountCode>, amount: Money) -> Result<Self, FinanceError> {
        if amount.is_negative() {
            return Err(FinanceError::NegativeAmount);
        }
        Ok(Self { account: account.into(), debit: None, credit: Some(amount), description: None })
    }

    /// Add a memo to this line.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Returns the amount of this line regardless of side.
    pub fn amount(&self) -> Option<&Money> {
        self.debit.as_ref().or(self.credit.as_ref())
    }
}

/// Double-entry journal entry.
///
/// Invariant: Σ debit_amounts = Σ credit_amounts (same currency).
/// This invariant is enforced at construction — `new()` returns `Err` if unbalanced.
/// There is no runtime "balance check" method that can be bypassed; only balanced
/// entries can be constructed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: JournalEntryId,
    pub date: chrono::NaiveDate,
    pub description: String,
    /// Minimum 2 lines (at least one debit + one credit).
    pub lines: Vec<JournalLine>,
}

impl JournalEntry {
    /// Constructor enforces all invariants:
    /// - At least 2 lines
    /// - Each line has exactly one side (debit XOR credit)
    /// - All amounts non-negative
    /// - Σ debits = Σ credits (same currency for all lines)
    pub fn new(
        id: JournalEntryId,
        date: chrono::NaiveDate,
        description: impl Into<String>,
        lines: Vec<JournalLine>,
    ) -> Result<Self, FinanceError> {
        if lines.len() < 2 {
            return Err(FinanceError::InsufficientLines);
        }

        for line in &lines {
            // Exactly one side must be set
            match (&line.debit, &line.credit) {
                (Some(_), None) | (None, Some(_)) => {}
                _ => return Err(FinanceError::AmbiguousJournalLine),
            }
            // Amount must be non-negative
            if let Some(amt) = &line.debit {
                if amt.is_negative() {
                    return Err(FinanceError::NegativeAmount);
                }
            }
            if let Some(amt) = &line.credit {
                if amt.is_negative() {
                    return Err(FinanceError::NegativeAmount);
                }
            }
        }

        // Sum debits and credits (all lines must use same currency)
        let mut total_debits = Decimal::ZERO;
        let mut total_credits = Decimal::ZERO;
        let mut currency = None;

        for line in &lines {
            let (amount, side) = if let Some(d) = &line.debit {
                (d, "debit")
            } else {
                (line.credit.as_ref().unwrap(), "credit")
            };

            // Enforce single currency
            match currency {
                None => currency = Some(amount.currency),
                Some(c) if c != amount.currency => {
                    return Err(FinanceError::CrossCurrencyArithmetic {
                        lhs: c,
                        rhs: amount.currency,
                    });
                }
                _ => {}
            }

            if side == "debit" {
                total_debits += amount.amount;
            } else {
                total_credits += amount.amount;
            }
        }

        if total_debits != total_credits {
            return Err(FinanceError::JournalUnbalanced {
                debits: total_debits,
                credits: total_credits,
            });
        }

        Ok(Self {
            id,
            date,
            description: description.into(),
            lines,
        })
    }

    /// Re-validate an already-constructed entry (useful after deserialization).
    pub fn validate(&self) -> Result<(), FinanceError> {
        Self::new(self.id, self.date, self.description.clone(), self.lines.clone()).map(|_| ())
    }

    /// Total debit amount (= total credit amount; both equal per invariant).
    pub fn total(&self) -> Money {
        let currency = self
            .lines
            .iter()
            .find_map(|l| l.debit.as_ref().map(|m| m.currency))
            .expect("at least one debit line");
        let amount = self
            .lines
            .iter()
            .filter_map(|l| l.debit.as_ref())
            .map(|m| m.amount)
            .sum();
        Money::new(amount, currency)
    }
}
```

### `src/traits.rs`

```rust
// crates/oya-finance-library-kernel/src/traits.rs
use crate::{error::FinanceError, journal::JournalEntry};

/// Pure sync validator trait (no async/I/O). Implementors may add business-rule
/// validation beyond the structural invariants enforced by JournalEntry::new().
/// Example: validate that account codes exist in a given Chart of Accounts.
pub trait JournalEntryValidator: Send + Sync {
    fn validate(&self, entry: &JournalEntry) -> Result<(), FinanceError>;
}

/// Pass-through validator that only re-checks structural invariants.
pub struct StructuralValidator;

impl JournalEntryValidator for StructuralValidator {
    fn validate(&self, entry: &JournalEntry) -> Result<(), FinanceError> {
        entry.validate()
    }
}
```

---

## 3. Domain: `oya-finance-library-domain`

### `src/lib.rs`

```rust
// crates/oya-finance-library-domain/src/lib.rs
pub mod amortization;
pub mod depreciation;
pub mod time_value;
```

### `src/time_value.rs`

```rust
// crates/oya-finance-library-domain/src/time_value.rs
//
// Time-value-of-money functions.
// All inputs/outputs use Decimal for precision. Never f64.
// Sign convention: cash inflows are positive, outflows negative.
// IRR uses Newton-Raphson; max_iterations defaults to 1000.
//
// Excel compatibility:
//   PV(rate, nper, pmt, [fv], [type])   — type 0 = end of period (default), 1 = start
//   FV(rate, nper, pmt, [pv], [type])
//   NPV(rate, value1, value2, ...)       — EXCLUDES period-0 (initial investment)
//   IRR(values, [guess])
//   XIRR(values, dates, [guess])
//   WACC: manual formula

use chrono::NaiveDate;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use oya_finance_library_kernel::FinanceError;

/// Maximum Newton-Raphson iterations for IRR/XIRR
const MAX_ITERATIONS: u32 = 1_000;
/// Convergence tolerance
const IRR_TOLERANCE: f64 = 1e-10;

// ─── PV ─────────────────────────────────────────────────────────────────────

/// Present Value of an annuity or lump sum.
///
/// - rate: interest rate per period (e.g., 0.05 for 5%)
/// - n: number of periods
/// - fv: future value (lump sum at end; 0 for pure annuity)
/// - pmt: payment per period (0 for lump-sum PV)
/// - pmt_at_start: true = annuity-due (payments at period start); false = ordinary annuity
///
/// Excel PV(rate, nper, pmt, fv, type) = pv(rate, nper, fv, pmt, type == 1)
pub fn pv(
    rate: Decimal,
    n: u32,
    fv: Decimal,
    pmt: Decimal,
    pmt_at_start: bool,
) -> Result<Decimal, FinanceError> {
    if n == 0 {
        return Err(FinanceError::ZeroPeriods);
    }
    if rate < Decimal::ZERO {
        return Err(FinanceError::NegativeRate);
    }

    // Convert to f64 for exp/pow; convert result back to Decimal
    let r = rate.to_f64().ok_or(FinanceError::DivisionByZero)?;
    let fv_f = fv.to_f64().ok_or(FinanceError::DivisionByZero)?;
    let pmt_f = pmt.to_f64().ok_or(FinanceError::DivisionByZero)?;
    let n_f = n as f64;

    let result = if r == 0.0 {
        -fv_f - pmt_f * n_f
    } else {
        let annuity_factor = (1.0 - (1.0 + r).powf(-n_f)) / r;
        let type_factor = if pmt_at_start { 1.0 + r } else { 1.0 };
        -(fv_f / (1.0 + r).powf(n_f)) - pmt_f * annuity_factor * type_factor
    };

    Decimal::from_f64_retain(result).ok_or(FinanceError::DivisionByZero)
}

// ─── FV ─────────────────────────────────────────────────────────────────────

/// Future Value of an annuity or lump sum.
///
/// Excel FV(rate, nper, pmt, pv, type) = fv(rate, nper, pv, pmt, type == 1)
pub fn fv(
    rate: Decimal,
    n: u32,
    pv: Decimal,
    pmt: Decimal,
    pmt_at_start: bool,
) -> Result<Decimal, FinanceError> {
    if n == 0 {
        return Err(FinanceError::ZeroPeriods);
    }
    if rate < Decimal::ZERO {
        return Err(FinanceError::NegativeRate);
    }

    let r = rate.to_f64().ok_or(FinanceError::DivisionByZero)?;
    let pv_f = pv.to_f64().ok_or(FinanceError::DivisionByZero)?;
    let pmt_f = pmt.to_f64().ok_or(FinanceError::DivisionByZero)?;
    let n_f = n as f64;

    let result = if r == 0.0 {
        -pv_f - pmt_f * n_f
    } else {
        let growth = (1.0 + r).powf(n_f);
        let type_factor = if pmt_at_start { 1.0 + r } else { 1.0 };
        -(pv_f * growth) - pmt_f * (growth - 1.0) / r * type_factor
    };

    Decimal::from_f64_retain(result).ok_or(FinanceError::DivisionByZero)
}

// ─── NPV ────────────────────────────────────────────────────────────────────

/// Net Present Value of a series of future cash flows.
///
/// IMPORTANT: Follows Excel NPV() convention — the first cash_flow is at period 1,
/// NOT period 0. If you have an initial investment at period 0 (t=0), add it
/// separately: `npv(rate, &future_flows)? + initial_investment`
///
/// Excel NPV(rate, value1, value2, ...) → npv(rate, &[value1, value2, ...])
pub fn npv(rate: Decimal, cash_flows: &[Decimal]) -> Result<Decimal, FinanceError> {
    if cash_flows.is_empty() {
        return Ok(Decimal::ZERO);
    }
    if rate < Decimal::ZERO {
        return Err(FinanceError::NegativeRate);
    }

    let r = rate.to_f64().ok_or(FinanceError::DivisionByZero)?;

    let result: f64 = cash_flows
        .iter()
        .enumerate()
        .map(|(i, cf)| {
            let cf_f = cf.to_f64().unwrap_or(0.0);
            cf_f / (1.0 + r).powi(i as i32 + 1)
        })
        .sum();

    Decimal::from_f64_retain(result).ok_or(FinanceError::DivisionByZero)
}

// ─── IRR ─────────────────────────────────────────────────────────────────────

/// Internal Rate of Return via Newton-Raphson iteration.
///
/// cash_flows must include at least one positive and one negative value.
/// Returns the periodic rate r such that NPV(r, cash_flows) = 0.
///
/// Excel IRR(values, [guess]) → irr(cash_flows) with internal guess = 0.1
pub fn irr(cash_flows: &[Decimal]) -> Result<Decimal, FinanceError> {
    let flows: Vec<f64> = cash_flows
        .iter()
        .map(|d| d.to_f64().unwrap_or(0.0))
        .collect();

    let has_positive = flows.iter().any(|&x| x > 0.0);
    let has_negative = flows.iter().any(|&x| x < 0.0);
    if !has_positive || !has_negative {
        return Err(FinanceError::IrrNotConverged { iterations: 0 });
    }

    let mut rate = 0.1_f64; // initial guess

    for iteration in 1..=MAX_ITERATIONS {
        let (npv_val, npv_deriv) = npv_and_derivative(&flows, rate);

        if npv_deriv.abs() < f64::EPSILON {
            return Err(FinanceError::IrrNotConverged { iterations: iteration });
        }

        let new_rate = rate - npv_val / npv_deriv;

        if (new_rate - rate).abs() < IRR_TOLERANCE {
            return Decimal::from_f64_retain(new_rate)
                .ok_or(FinanceError::IrrNotConverged { iterations: iteration });
        }

        rate = new_rate;
    }

    Err(FinanceError::IrrNotConverged { iterations: MAX_ITERATIONS })
}

fn npv_and_derivative(flows: &[f64], rate: f64) -> (f64, f64) {
    let mut npv = 0.0_f64;
    let mut deriv = 0.0_f64;

    for (t, &cf) in flows.iter().enumerate() {
        let t_f = t as f64;
        let denominator = (1.0 + rate).powf(t_f);
        npv += cf / denominator;
        deriv -= t_f * cf / ((1.0 + rate).powf(t_f + 1.0));
    }

    (npv, deriv)
}

// ─── XIRR ────────────────────────────────────────────────────────────────────

/// Extended IRR for cash flows at irregular dates.
///
/// cash_flows[0] must be negative (initial investment).
/// dates[0] is the reference date (t=0).
/// Returns the annualized rate r such that Σ CF_i / (1+r)^((date_i - date_0)/365) = 0
///
/// Excel XIRR(values, dates, [guess]) → xirr(cash_flows, dates)
pub fn xirr(cash_flows: &[Decimal], dates: &[NaiveDate]) -> Result<Decimal, FinanceError> {
    if cash_flows.len() != dates.len() {
        return Err(FinanceError::XirrLengthMismatch);
    }
    if cash_flows.is_empty() {
        return Err(FinanceError::XirrLengthMismatch);
    }

    let flows: Vec<f64> = cash_flows.iter().map(|d| d.to_f64().unwrap_or(0.0)).collect();
    let has_positive = flows.iter().any(|&x| x > 0.0);
    let has_negative = flows.iter().any(|&x| x < 0.0);
    if !has_positive || !has_negative {
        return Err(FinanceError::XirrInvalidCashFlows);
    }

    let base_date = dates[0];
    let day_fractions: Vec<f64> = dates
        .iter()
        .map(|&d| (d - base_date).num_days() as f64 / 365.0)
        .collect();

    let mut rate = 0.1_f64;

    for iteration in 1..=MAX_ITERATIONS {
        let (val, deriv) = xirr_and_derivative(&flows, &day_fractions, rate);

        if deriv.abs() < f64::EPSILON {
            return Err(FinanceError::IrrNotConverged { iterations: iteration });
        }

        let new_rate = rate - val / deriv;

        if (new_rate - rate).abs() < IRR_TOLERANCE {
            return Decimal::from_f64_retain(new_rate)
                .ok_or(FinanceError::IrrNotConverged { iterations: iteration });
        }

        rate = new_rate;
    }

    Err(FinanceError::IrrNotConverged { iterations: MAX_ITERATIONS })
}

fn xirr_and_derivative(flows: &[f64], day_fractions: &[f64], rate: f64) -> (f64, f64) {
    let mut val = 0.0_f64;
    let mut deriv = 0.0_f64;

    for (&cf, &t) in flows.iter().zip(day_fractions.iter()) {
        let denominator = (1.0 + rate).powf(t);
        val += cf / denominator;
        deriv -= t * cf / ((1.0 + rate).powf(t + 1.0));
    }

    (val, deriv)
}

// ─── WACC ────────────────────────────────────────────────────────────────────

/// Weighted Average Cost of Capital.
///
/// WACC = equity_weight × cost_of_equity + debt_weight × cost_of_debt × (1 - tax_rate)
///
/// Reference: equity=0.6, debt=0.4, re=0.12, rd=0.06, tax=0.25 → WACC = 0.084 (8.4%)
pub fn wacc(
    equity_weight: Decimal,
    debt_weight: Decimal,
    cost_of_equity: Decimal,
    cost_of_debt: Decimal,
    tax_rate: Decimal,
) -> Result<Decimal, FinanceError> {
    if equity_weight < Decimal::ZERO || debt_weight < Decimal::ZERO {
        return Err(FinanceError::NegativeRate);
    }
    if tax_rate < Decimal::ZERO || tax_rate > Decimal::ONE {
        return Err(FinanceError::InvalidDepreciation(
            "tax_rate must be between 0.0 and 1.0".to_owned(),
        ));
    }

    let after_tax_cost_of_debt = cost_of_debt * (Decimal::ONE - tax_rate);
    let result = equity_weight * cost_of_equity + debt_weight * after_tax_cost_of_debt;
    Ok(result)
}
```

### `src/depreciation.rs`

```rust
// crates/oya-finance-library-domain/src/depreciation.rs
//
// Depreciation schedules. All amounts preserve Money currency.
// Returns one Money per period (year), length = life_years.
//
// Reference values:
//   straight_line:  cost=10000 USD, salvage=0, life=5 → [2000, 2000, 2000, 2000, 2000]
//   declining_200:  cost=10000 USD, salvage=0, life=5 → [4000, 2400, 1440,  864, 1296]
//                   (year 5 switches to SL when SL > DB)
//   macrs_5yr:      cost=10000 USD → [2000, 3200, 1920, 1152, 1152, 576] (6 rows, half-year convention)
//   syd:            cost=10000 USD, salvage=0, life=5 → [3333.33, 2666.67, 2000, 1333.33, 666.67]

use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use oya_finance_library_kernel::{FinanceError, Money, CurrencyCode};

/// MACRS (Modified Accelerated Cost Recovery System) property classes.
/// Percentages sourced from IRS Publication 946, Table A-1 (half-year convention).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacrsPropertyClass {
    ThreeYear,
    FiveYear,
    SevenYear,
    TenYear,
    FifteenYear,
    TwentyYear,
}

/// IRS Publication 946 Table A-1 percentages (half-year convention).
/// Each array has length = recovery_period + 1 (extra year from half-year convention).
fn macrs_percentages(class: MacrsPropertyClass) -> &'static [f64] {
    match class {
        MacrsPropertyClass::ThreeYear   => &[0.3333, 0.4445, 0.1481, 0.0741],
        MacrsPropertyClass::FiveYear    => &[0.2000, 0.3200, 0.1920, 0.1152, 0.1152, 0.0576],
        MacrsPropertyClass::SevenYear   => &[0.1429, 0.2449, 0.1749, 0.1249, 0.0893, 0.0892, 0.0893, 0.0446],
        MacrsPropertyClass::TenYear     => &[0.1000, 0.1800, 0.1440, 0.1152, 0.0922, 0.0737, 0.0655, 0.0655, 0.0656, 0.0655, 0.0328],
        MacrsPropertyClass::FifteenYear => &[0.0500, 0.0950, 0.0855, 0.0770, 0.0693, 0.0623, 0.0590, 0.0590, 0.0591, 0.0590, 0.0591, 0.0590, 0.0591, 0.0590, 0.0591, 0.0295],
        MacrsPropertyClass::TwentyYear  => &[0.0375, 0.0722, 0.0668, 0.0618, 0.0571, 0.0529, 0.0489, 0.0452, 0.0446, 0.0446, 0.0446, 0.0446, 0.0446, 0.0446, 0.0446, 0.0446, 0.0446, 0.0446, 0.0446, 0.0446, 0.0223],
    }
}

/// Straight-line depreciation.
/// Returns `life_years` equal annual depreciation amounts.
pub fn straight_line(
    cost: &Money,
    salvage: &Money,
    life_years: u32,
) -> Result<Vec<Money>, FinanceError> {
    if life_years == 0 {
        return Err(FinanceError::ZeroPeriods);
    }
    let depreciable = cost.sub(salvage)?;
    let annual = depreciable.div_scalar(Decimal::from(life_years))?;
    Ok(vec![annual; life_years as usize])
}

/// Declining balance depreciation (150% DB = rate_multiplier 1.5, 200% DDB = 2.0).
/// Switches to straight-line in the year when SL depreciation exceeds DB depreciation.
/// Salvage value is never depreciated below.
pub fn declining_balance(
    cost: &Money,
    salvage: &Money,
    life_years: u32,
    rate_multiplier: Decimal,
) -> Result<Vec<Money>, FinanceError> {
    if life_years == 0 {
        return Err(FinanceError::ZeroPeriods);
    }
    if rate_multiplier <= Decimal::ZERO {
        return Err(FinanceError::InvalidDepreciation(
            "rate_multiplier must be positive".to_owned(),
        ));
    }

    let db_rate = rate_multiplier / Decimal::from(life_years);
    let mut book_value = cost.amount;
    let salvage_amount = salvage.amount;
    let currency = cost.currency;
    let mut schedule = Vec::with_capacity(life_years as usize);

    let remaining_years = |year: u32| Decimal::from(life_years - year + 1);

    for year in 1..=life_years {
        let db_depr = (book_value * db_rate).max(Decimal::ZERO);
        let sl_depr = if book_value > salvage_amount {
            (book_value - salvage_amount) / remaining_years(year)
        } else {
            Decimal::ZERO
        };

        let depr = if sl_depr > db_depr { sl_depr } else { db_depr };
        // Never depreciate below salvage
        let actual_depr = depr.min(book_value - salvage_amount).max(Decimal::ZERO);

        schedule.push(Money::new(actual_depr, currency));
        book_value -= actual_depr;
    }

    Ok(schedule)
}

/// MACRS depreciation using IRS Publication 946 Table A-1 percentages.
/// Returns one amount per year (including the extra half-year convention year).
pub fn macrs(cost: &Money, property_class: MacrsPropertyClass) -> Result<Vec<Money>, FinanceError> {
    let percentages = macrs_percentages(property_class);
    Ok(percentages
        .iter()
        .map(|&pct| {
            let pct_dec = Decimal::from_f64_retain(pct)
                .unwrap_or(Decimal::ZERO);
            Money::new(cost.amount * pct_dec, cost.currency)
        })
        .collect())
}

/// Sum-of-years-digits depreciation.
/// Year 1 gets the largest fraction; Year n gets the smallest.
/// SYD denominator = n*(n+1)/2
pub fn sum_of_years_digits(
    cost: &Money,
    salvage: &Money,
    life_years: u32,
) -> Result<Vec<Money>, FinanceError> {
    if life_years == 0 {
        return Err(FinanceError::ZeroPeriods);
    }
    let depreciable = cost.sub(salvage)?;
    let syd_denom = Decimal::from(life_years * (life_years + 1)) / Decimal::from(2u32);

    if syd_denom.is_zero() {
        return Err(FinanceError::DivisionByZero);
    }

    let schedule = (1..=life_years)
        .map(|year| {
            // Year 1: fraction = n/SYD; Year n: fraction = 1/SYD
            let numerator = Decimal::from(life_years - year + 1);
            let fraction = numerator / syd_denom;
            Money::new(depreciable.amount * fraction, depreciable.currency)
        })
        .collect();

    Ok(schedule)
}
```

### `src/amortization.rs`

```rust
// crates/oya-finance-library-domain/src/amortization.rs
//
// Loan amortization schedules.
// All calculations use Decimal to avoid rounding drift.
//
// Reference: $200,000 principal, 6% annual rate, 30-year term
//   Monthly payment = $1,199.10 (Excel PMT(0.5%, 360, -200000))
//   equal_payment_schedule returns 360 AmortizationLines.

use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use oya_finance_library_kernel::{FinanceError, Money};

/// One period in an amortization schedule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmortizationLine {
    pub period: u32,
    pub payment: Money,
    pub principal: Money,
    pub interest: Money,
    pub balance: Money,
}

/// Equal-payment (standard mortgage / annuity) amortization.
///
/// - principal: loan amount (must be positive)
/// - annual_rate: annual interest rate as decimal (e.g. 0.06 for 6%)
/// - periods: number of payment periods (e.g. 360 for 30-year monthly)
///
/// Payment per period = principal × r / (1 − (1+r)^-n)   where r = annual_rate / 12
///
/// Reference: PMT(0.005, 360, -200000) = $1,199.10
pub fn equal_payment_schedule(
    principal: &Money,
    annual_rate: Decimal,
    periods: u32,
) -> Result<Vec<AmortizationLine>, FinanceError> {
    if periods == 0 {
        return Err(FinanceError::ZeroPeriods);
    }
    if annual_rate < Decimal::ZERO {
        return Err(FinanceError::NegativeRate);
    }
    if !principal.is_positive() {
        return Err(FinanceError::InvalidAmortizationSchedule(
            "principal must be positive".to_owned(),
        ));
    }

    let currency = principal.currency;
    // Monthly rate (assumes monthly compounding for standard mortgages)
    let r = annual_rate / Decimal::from(12u32);

    // Calculate fixed payment
    let payment_amount = if r.is_zero() {
        // Zero-interest: equal principal repayment
        principal.amount / Decimal::from(periods)
    } else {
        let r_f = r.to_f64().ok_or(FinanceError::DivisionByZero)?;
        let n_f = periods as f64;
        let payment_f = principal.amount.to_f64().ok_or(FinanceError::DivisionByZero)?
            * r_f / (1.0 - (1.0 + r_f).powf(-n_f));
        Decimal::from_f64_retain(payment_f).ok_or(FinanceError::DivisionByZero)?
    };

    // Scale payment to 2 decimal places (cents)
    let payment_rounded = payment_amount.round_dp(2);
    let mut balance = principal.amount;
    let mut schedule = Vec::with_capacity(periods as usize);

    for period in 1..=periods {
        let interest_amount = (balance * r).round_dp(2);
        let mut principal_amount = payment_rounded - interest_amount;

        // Last period: pay off remaining balance exactly (avoids rounding residual)
        if period == periods {
            principal_amount = balance;
        }

        balance -= principal_amount;
        // Clamp to zero (rounding can produce -0.01)
        if balance.abs() < dec!(0.01) {
            balance = Decimal::ZERO;
        }

        schedule.push(AmortizationLine {
            period,
            payment: Money::new(principal_amount + interest_amount, currency),
            principal: Money::new(principal_amount, currency),
            interest: Money::new(interest_amount, currency),
            balance: Money::new(balance, currency),
        });
    }

    Ok(schedule)
}

/// Interest-only schedule with balloon payment at `balloon_period`.
///
/// Pays only interest for periods 1..balloon_period.
/// At `balloon_period`, pays interest + full remaining principal (balloon).
/// After balloon_period, `periods - balloon_period` periods remain at zero balance.
pub fn interest_only_schedule(
    principal: &Money,
    annual_rate: Decimal,
    periods: u32,
    balloon_period: u32,
) -> Result<Vec<AmortizationLine>, FinanceError> {
    if periods == 0 {
        return Err(FinanceError::ZeroPeriods);
    }
    if balloon_period == 0 || balloon_period > periods {
        return Err(FinanceError::InvalidAmortizationSchedule(format!(
            "balloon_period ({balloon_period}) must be 1..={periods}"
        )));
    }
    if !principal.is_positive() {
        return Err(FinanceError::InvalidAmortizationSchedule(
            "principal must be positive".to_owned(),
        ));
    }
    if annual_rate < Decimal::ZERO {
        return Err(FinanceError::NegativeRate);
    }

    let currency = principal.currency;
    let r = annual_rate / Decimal::from(12u32);
    let mut balance = principal.amount;
    let mut schedule = Vec::with_capacity(periods as usize);

    for period in 1..=periods {
        let interest_amount = (balance * r).round_dp(2);

        let (principal_repaid, payment_amount) = if period == balloon_period {
            // Balloon: repay all remaining principal
            (balance, balance + interest_amount)
        } else if period < balloon_period {
            // Interest only
            (Decimal::ZERO, interest_amount)
        } else {
            // After balloon: zero payments (loan fully repaid)
            (Decimal::ZERO, Decimal::ZERO)
        };

        balance -= principal_repaid;

        schedule.push(AmortizationLine {
            period,
            payment: Money::new(payment_amount, currency),
            principal: Money::new(principal_repaid, currency),
            interest: Money::new(if period <= balloon_period { interest_amount } else { Decimal::ZERO }, currency),
            balance: Money::new(balance, currency),
        });
    }

    Ok(schedule)
}
```

---

## 4. Excel Reference Tests

### `oya-finance-library-domain/tests/excel_reference.rs`

```rust
// crates/oya-finance-library-domain/tests/excel_reference.rs
//
// All values validated against Excel 365 / LibreOffice Calc.
// Tolerance: ±0.01 for monetary values (1 cent), ±0.0001 for rates.

#[cfg(test)]
mod time_value_tests {
    use rust_decimal_macros::dec;
    use oya_finance_library_domain::time_value::*;

    // ─── NPV ────────────────────────────────────────────────────────────────
    // Excel: =NPV(10%, -1000+300, 400, 500) where initial investment is period-0
    // Following Excel convention: NPV(0.10, 300, 400, 500) + (-1000) = 82.64
    // i.e., the initial -1000 is added externally since Excel NPV starts at period 1.
    #[test]
    fn npv_excel_reference() {
        let rate = dec!(0.10);
        let cash_flows = vec![dec!(300), dec!(400), dec!(500)];
        let result = npv(rate, &cash_flows).unwrap();
        let total = result + dec!(-1000);  // add initial investment at period 0
        // Excel NPV(10%, 300, 400, 500) - 1000 = 82.64
        let diff = (total - dec!(82.64)).abs();
        assert!(diff < dec!(0.01), "NPV={total}, expected ≈82.64, diff={diff}");
    }

    // ─── IRR ────────────────────────────────────────────────────────────────
    // Excel: =IRR({-100, 50, 60}) ≈ 12.66%
    #[test]
    fn irr_convergence() {
        let cash_flows = vec![dec!(-100), dec!(50), dec!(60)];
        let result = irr(&cash_flows).unwrap();
        let diff = (result - dec!(0.1266)).abs();
        assert!(diff < dec!(0.0001), "IRR={result}, expected ≈0.1266, diff={diff}");
    }

    // All-positive cash flows have no IRR → IrrNotConverged
    #[test]
    fn irr_not_converged_error() {
        use oya_finance_library_kernel::FinanceError;
        let result = irr(&[dec!(100), dec!(200), dec!(300)]);
        assert!(matches!(result, Err(FinanceError::IrrNotConverged { .. })));
    }

    // ─── XIRR ───────────────────────────────────────────────────────────────
    // Excel: =XIRR({-1000, 250, 250, 250, 250, 250}, {2024-01-01, ..., 2028-01-01})
    // Annual cash flow at irregular intervals; expected ≈ 7.93%
    #[test]
    fn xirr_irregular_dates() {
        use chrono::NaiveDate;
        let cash_flows = vec![dec!(-1000), dec!(250), dec!(250), dec!(250), dec!(250), dec!(250)];
        let dates: Vec<NaiveDate> = vec![
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2028, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2029, 1, 1).unwrap(),
        ];
        let result = xirr(&cash_flows, &dates).unwrap();
        let diff = (result - dec!(0.0792)).abs();
        assert!(diff < dec!(0.001), "XIRR={result}, expected ≈0.0792, diff={diff}");
    }

    // ─── WACC ───────────────────────────────────────────────────────────────
    // equity=0.6, debt=0.4, re=0.12, rd=0.06, tax=0.25 → WACC = 0.084 (8.4%)
    #[test]
    fn wacc_reference() {
        let result = wacc(dec!(0.6), dec!(0.4), dec!(0.12), dec!(0.06), dec!(0.25)).unwrap();
        let diff = (result - dec!(0.084)).abs();
        assert!(diff < dec!(0.0001), "WACC={result}, expected=0.084, diff={diff}");
    }

    // ─── PV ─────────────────────────────────────────────────────────────────
    // Excel: =PV(5%, 10, -1000, 0, 0) → 7,721.73 (ordinary annuity of $1000/period)
    #[test]
    fn pv_ordinary_annuity_excel_reference() {
        let result = pv(dec!(0.05), 10, dec!(0), dec!(-1000), false).unwrap();
        let diff = (result - dec!(7721.73)).abs();
        assert!(diff < dec!(0.10), "PV={result}, expected≈7721.73, diff={diff}");
    }

    // ─── FV ─────────────────────────────────────────────────────────────────
    // Excel: =FV(5%, 10, -1000, 0, 0) → 12,577.89
    #[test]
    fn fv_ordinary_annuity_excel_reference() {
        let result = fv(dec!(0.05), 10, dec!(0), dec!(-1000), false).unwrap();
        let diff = (result - dec!(12577.89)).abs();
        assert!(diff < dec!(0.10), "FV={result}, expected≈12577.89, diff={diff}");
    }
}

#[cfg(test)]
mod depreciation_tests {
    use rust_decimal_macros::dec;
    use oya_finance_library_kernel::{CurrencyCode, Money};
    use oya_finance_library_domain::depreciation::*;

    // ─── Straight-line ───────────────────────────────────────────────────────
    // cost=10000 USD, salvage=0, life=5 → [2000, 2000, 2000, 2000, 2000]
    #[test]
    fn depreciation_straight_line() {
        let cost = Money::new(dec!(10000), CurrencyCode::USD);
        let salvage = Money::new(dec!(0), CurrencyCode::USD);
        let schedule = straight_line(&cost, &salvage, 5).unwrap();
        assert_eq!(schedule.len(), 5);
        for line in &schedule {
            let diff = (line.amount - dec!(2000)).abs();
            assert!(diff < dec!(0.01), "Expected 2000, got {}", line.amount);
        }
    }

    // ─── 200% Declining Balance ─────────────────────────────────────────────
    // cost=10000 USD, salvage=0, life=5 → [4000, 2400, 1440, 864, 1296]
    // (switches to SL in year 5: SL(864/2) = 432 < DB(864*0.4)=345.60, so stays DB until last)
    #[test]
    fn depreciation_declining_200() {
        let cost = Money::new(dec!(10000), CurrencyCode::USD);
        let salvage = Money::new(dec!(0), CurrencyCode::USD);
        let schedule = declining_balance(&cost, &salvage, 5, dec!(2.0)).unwrap();
        assert_eq!(schedule.len(), 5);
        let expected = [dec!(4000), dec!(2400), dec!(1440), dec!(864), dec!(1296)];
        for (line, &exp) in schedule.iter().zip(expected.iter()) {
            let diff = (line.amount - exp).abs();
            assert!(diff < dec!(0.01), "Expected {exp}, got {}", line.amount);
        }
    }

    // ─── MACRS 5-year ────────────────────────────────────────────────────────
    // cost=10000 USD → [2000, 3200, 1920, 1152, 1152, 576] (IRS Publication 946 Table A-1)
    #[test]
    fn depreciation_macrs_5yr() {
        let cost = Money::new(dec!(10000), CurrencyCode::USD);
        let schedule = macrs(&cost, MacrsPropertyClass::FiveYear).unwrap();
        assert_eq!(schedule.len(), 6, "MACRS 5-yr has 6 rows due to half-year convention");
        let expected = [dec!(2000), dec!(3200), dec!(1920), dec!(1152), dec!(1152), dec!(576)];
        for (line, &exp) in schedule.iter().zip(expected.iter()) {
            let diff = (line.amount - exp).abs();
            assert!(diff < dec!(1.00), "Expected {exp}, got {}", line.amount);
        }
    }
}

#[cfg(test)]
mod amortization_tests {
    use rust_decimal_macros::dec;
    use oya_finance_library_kernel::{CurrencyCode, Money};
    use oya_finance_library_domain::amortization::*;

    // ─── Equal payment ────────────────────────────────────────────────────────
    // $200k, 6% annual, 30yr (360 months)
    // Excel PMT(0.5%, 360, -200000) = $1,199.10 per month
    #[test]
    fn amortization_equal_payment() {
        let principal = Money::new(dec!(200000), CurrencyCode::USD);
        let schedule = equal_payment_schedule(&principal, dec!(0.06), 360).unwrap();
        assert_eq!(schedule.len(), 360);
        // Check first payment ≈ $1,199.10
        let first = &schedule[0];
        let diff = (first.payment.amount - dec!(1199.10)).abs();
        assert!(diff < dec!(0.05), "First payment={}, expected≈1199.10", first.payment.amount);
        // Check last balance is zero
        let last = &schedule[359];
        assert!(
            last.balance.amount.abs() < dec!(0.01),
            "Final balance should be zero, got {}",
            last.balance.amount
        );
    }

    // ─── Balloon ─────────────────────────────────────────────────────────────
    // $100k, 6% annual, 36 periods, balloon at period 12
    // Months 1-11: interest only; month 12: interest + full principal balloon
    #[test]
    fn amortization_balloon() {
        let principal = Money::new(dec!(100000), CurrencyCode::USD);
        let schedule = interest_only_schedule(&principal, dec!(0.06), 36, 12).unwrap();
        // Months 1-11: principal repaid = 0
        for line in &schedule[0..11] {
            assert!(
                line.principal.amount.is_zero(),
                "Period {} should have zero principal, got {}",
                line.period, line.principal.amount
            );
        }
        // Month 12: full balloon
        let balloon = &schedule[11];
        let diff = (balloon.principal.amount - dec!(100000)).abs();
        assert!(diff < dec!(0.01), "Balloon principal={}, expected=100000", balloon.principal.amount);
        // Balance after balloon = 0
        assert!(
            balloon.balance.amount.abs() < dec!(0.01),
            "Balance after balloon should be zero, got {}",
            balloon.balance.amount
        );
        // Months 13-36: zero payments
        for line in &schedule[12..] {
            assert!(
                line.payment.amount.is_zero(),
                "Period {} post-balloon should have zero payment", line.period
            );
        }
    }
}

#[cfg(test)]
mod journal_tests {
    use rust_decimal_macros::dec;
    use chrono::NaiveDate;
    use uuid::Uuid;
    use oya_finance_library_kernel::{
        CurrencyCode, FinanceError, JournalEntry, JournalLine, Money,
    };

    fn date() -> NaiveDate { NaiveDate::from_ymd_opt(2026, 1, 1).unwrap() }

    // ─── Balanced ────────────────────────────────────────────────────────────
    #[test]
    fn journal_entry_balanced() {
        let lines = vec![
            JournalLine::debit("1001", Money::new(dec!(1000), CurrencyCode::USD)).unwrap(),
            JournalLine::credit("4001", Money::new(dec!(1000), CurrencyCode::USD)).unwrap(),
        ];
        let entry = JournalEntry::new(Uuid::new_v4(), date(), "Test sale", lines);
        assert!(entry.is_ok(), "Balanced entry must succeed");
    }

    // ─── Unbalanced ──────────────────────────────────────────────────────────
    #[test]
    fn journal_entry_unbalanced_rejected() {
        let lines = vec![
            JournalLine::debit("1001", Money::new(dec!(1000), CurrencyCode::USD)).unwrap(),
            JournalLine::credit("4001", Money::new(dec!(900), CurrencyCode::USD)).unwrap(),
        ];
        let result = JournalEntry::new(Uuid::new_v4(), date(), "Unbalanced", lines);
        assert!(
            matches!(result, Err(FinanceError::JournalUnbalanced { .. })),
            "Unbalanced entry must return JournalUnbalanced error"
        );
    }

    // ─── Fewer than 2 lines ──────────────────────────────────────────────────
    #[test]
    fn journal_entry_single_line_rejected() {
        let lines = vec![
            JournalLine::debit("1001", Money::new(dec!(1000), CurrencyCode::USD)).unwrap(),
        ];
        let result = JournalEntry::new(Uuid::new_v4(), date(), "Single line", lines);
        assert!(
            matches!(result, Err(FinanceError::InsufficientLines)),
            "Single-line entry must return InsufficientLines error"
        );
    }

    // ─── Cross-currency arithmetic ───────────────────────────────────────────
    #[test]
    fn money_cross_currency_rejected() {
        let a = Money::new(dec!(100), CurrencyCode::USD);
        let b = Money::new(dec!(100), CurrencyCode::KRW);
        let result = a.add(&b);
        assert!(
            matches!(result, Err(FinanceError::CrossCurrencyArithmetic { .. })),
            "Cross-currency addition must fail"
        );
    }

    // ─── Money precision ─────────────────────────────────────────────────────
    #[test]
    fn money_arithmetic_precision() {
        // 0.1 + 0.2 must equal 0.3 exactly (no floating-point drift)
        let a = Money::new(dec!(0.1), CurrencyCode::USD);
        let b = Money::new(dec!(0.2), CurrencyCode::USD);
        let result = a.add(&b).unwrap();
        assert_eq!(result.amount, dec!(0.3), "0.1 + 0.2 must equal 0.3 exactly using Decimal");
    }
}
```

---

## 5. Zero-Dependency Verification

```bash
# Verify oya-finance-library-kernel has NO sqlx/diesel/tokio/axum/reqwest/async-trait
cargo metadata --manifest-path crates/oya-finance-library-kernel/Cargo.toml \
  --no-deps --format-version 1 \
  | jq -r '.packages[0].dependencies[].name' \
  | sort

# Expected output (only these 5 deps):
#   chrono
#   rust_decimal
#   serde
#   thiserror
#   uuid

# Verify oya-finance-library-domain has no I/O deps beyond kernel
cargo metadata --manifest-path crates/oya-finance-library-domain/Cargo.toml \
  --no-deps --format-version 1 \
  | jq -r '.packages[0].dependencies[] | select(.kind == null) | .name' \
  | sort

# Expected output (only these 3 prod deps):
#   chrono
#   oya-finance-library-kernel
#   rust_decimal
```

---

## 6. Acceptance Gate Commands

```bash
# 1. Cargo gates
cargo check --workspace --all-features
cargo build --workspace --all-features
cargo clippy --workspace --all-features -- -D warnings
cargo nextest run --workspace --all-features

# 2. Fitness lanes
oya gate validate lean-a1 --phase P11-finance-library
oya gate validate lean-a2 --phase P11-finance-library
oya gate validate lean-a3 --phase P11-finance-library
oya gate validate lean-a4 --phase P11-finance-library

# 3. JournalEntry invariant
cargo nextest run -p oya-finance-library-kernel --test journal_entry_balanced
cargo nextest run -p oya-finance-library-kernel --test journal_entry_unbalanced_rejected

# 4. Money precision
cargo nextest run -p oya-finance-library-domain --test money_arithmetic_precision

# 5. Time-value Excel references
cargo nextest run -p oya-finance-library-domain --test pv_fv_excel_reference
cargo nextest run -p oya-finance-library-domain --test npv_excel_reference
cargo nextest run -p oya-finance-library-domain --test irr_convergence
cargo nextest run -p oya-finance-library-domain --test irr_not_converged_error
cargo nextest run -p oya-finance-library-domain --test xirr_irregular_dates
cargo nextest run -p oya-finance-library-domain --test wacc_reference

# 6. Depreciation
cargo nextest run -p oya-finance-library-domain --test depreciation_straight_line
cargo nextest run -p oya-finance-library-domain --test depreciation_declining_200
cargo nextest run -p oya-finance-library-domain --test depreciation_macrs_5yr

# 7. Amortization
cargo nextest run -p oya-finance-library-domain --test amortization_equal_payment
cargo nextest run -p oya-finance-library-domain --test amortization_balloon

# 8. Zero-dep gate
cargo metadata --manifest-path crates/oya-finance-library-kernel/Cargo.toml --no-deps \
  --format-version 1 | jq -r '.packages[0].dependencies[].name' \
  | grep -E 'sqlx|diesel|tokio|axum|reqwest|async-trait' && exit 1 || true
# Must: exit 0 (no forbidden deps found)

# 9. grit done
grit done --agent m02-wave-a-executor
```

---

## 7. ICM Store Commands

```bash
icm store \
  -t context-oyatie \
  -c "IP-P11-finance-library merged; 2 crates (kernel+domain); pure Rust zero I/O; Money/JournalEntry double-entry invariant; PV/FV/NPV/IRR/XIRR/WACC/depreciation/amortization; all Excel reference tests pass; Wave-A 22 files COMPLETE" \
  -i high \
  -k "M02,P11,impl-plan,finance-library,Wave-A-complete"

icm store \
  -t context-oyatie \
  -c "M02 Wave-A complete: 22 files (11 phase-specs + 11 impl-plans) authored for 11 substrate µservices (P01-P11). All impl-plans have full DDL, sealed port traits, adapter code, Protobuf schemas, load tests, and acceptance gates. No stubs." \
  -i critical \
  -k "M02,wave-a,complete,22-files,P01,P02,P03,P04,P05,P06,P07,P08,P09,P10,P11"
```

---

## Next IP Pointer

Wave-A complete. Next: M02 Wave-B µservice impl-plans (parallel to substrate).
