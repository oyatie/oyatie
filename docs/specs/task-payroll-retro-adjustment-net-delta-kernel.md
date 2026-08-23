# Spec: payroll-retro-adjustment-net-delta-kernel

## Summary
Pure kernel function `evaluate_retro_adjustment` that computes signed per-payee net
retro deltas between an original-period `PayeeVarianceTotal`-style baseline and
corrected current totals. Distinct from variance (BPS-swing anomaly gating) and
from journal/GL posting.

## Crate
`payroll-run-domain` (flat clean-arch, ADR-0509)

## Public API

### Schema Version
```rust
pub const RETRO_ADJUSTMENT_SCHEMA_VERSION: u32 = 1;
```

### Input
```rust
pub struct RetroAdjustmentInput {
    pub run_id: String,                              // must have prun_ prefix
    pub run_ref: String,                             // must have audit/ prefix
    pub original_period_totals: Vec<PayeeVarianceTotal>,  // FINANCIAL
    pub corrected_period_totals: Vec<PayeeVarianceTotal>, // FINANCIAL
    pub evidence_refs: Vec<String>,                  // each must have audit/ prefix; non-empty
}
```

### Payee Classification
```rust
pub enum RetroPayeeClass {
    Added,     // payee in corrected but not in original
    Removed,   // payee in original but not in corrected
    Changed,   // payee in both; corrected != original
    Unchanged, // payee in both; corrected == original (delta = 0)
}
```

### Output Lines
```rust
pub struct RetroDeltaLine {
    pub payee_id: Classified<PayeeId>,               // INTERNAL_ONLY
    pub original_amount: Classified<MoneyAmount>,    // FINANCIAL; amount_minor=0 for Added payees
    pub corrected_amount: Classified<MoneyAmount>,   // FINANCIAL; amount_minor=0 for Removed payees
    pub delta_amount: Classified<MoneyAmount>,       // FINANCIAL; signed (corrected - original)
    pub payee_class: Classified<RetroPayeeClass>,    // INTERNAL_ONLY
}
```

### Verdict
```rust
pub struct RetroAdjustmentVerdict {
    pub run_id: Classified<PayrollRunId>,            // INTERNAL_ONLY
    pub lines: Classified<Vec<RetroDeltaLine>>,      // FINANCIAL
    pub run_net_delta: Classified<MoneyAmount>,      // FINANCIAL; sum of delta_amount.amount_minor
    pub balanced: Classified<bool>,                  // PUBLIC; true iff sum(corrected) - sum(original) == run_net_delta
    pub evidence_digest: Classified<EvidenceDigest>, // INTERNAL_ONLY
    pub schema_version: Classified<u32>,             // PUBLIC
}
```

### Errors Added
```rust
PayrollDomainError::CurrencyMismatch    // original and corrected amounts use different currencies for same payee
PayrollDomainError::InvalidRunRef       // run_ref fails audit/ prefix + path safety check
PayrollDomainError::RetroEvidenceRequired  // evidence_refs is empty
```

## Behaviour

### Currency invariant
Each payee must appear with the same currency in both original and corrected totals.
If a payee appears in both with different currencies, return `Err(CurrencyMismatch)`.
Added payees inherit the corrected currency; Removed payees inherit the original currency.

### Delta computation
- Added: delta = corrected_amount; original synthetic = MoneyAmount{0, corrected.currency}
- Removed: delta = -original_amount; corrected synthetic = MoneyAmount{0, original.currency}
- Changed: delta = corrected - original (signed minor units)
- Unchanged: delta = 0 (line still emitted)

### run_net_delta
Currency is taken from the first line's delta currency (all must be same-currency).
`amount_minor` = sum of all delta line `delta_amount.amount_minor`.

### balanced flag
`true` iff `run_net_delta.amount_minor == sum(corrected minor) - sum(original minor)`.

### Evidence digest
Same XOR-fold-over-bytes algorithm as `evaluate_payroll_variance`: XOR all bytes
of all evidence_ref strings into a 32-byte buffer (position mod 32), hex-encode,
prepend `sha256:`.

### Determinism
No I/O. Same input → same output. Lines ordered: original_period payees first
(in input order), then added payees (corrected-only, in input order).

### Validation
- `run_id`: must have `prun_` prefix, alphanumeric+`_-` suffix
- `run_ref`: must have `audit/` prefix, path-safe
- `evidence_refs`: must be non-empty; each must have `audit/` prefix and be path-safe
- Each `payee_id` in both lists: `payee_` prefix
- Currency codes: 3-char ASCII

## Acceptance Tests (7)

| # | Scenario | Expected |
|---|----------|----------|
| a | Pure delta for matched payee (Changed) | delta = corrected - original |
| b | Currency mismatch on same payee | Err(CurrencyMismatch) |
| c | Newly added payee | delta = corrected amount, payee_class = Added |
| d | Removed payee | delta = -original amount, payee_class = Removed |
| e | Zero-delta payee | retained with delta=0, payee_class = Unchanged |
| f | run_net_delta = sum of line deltas | assertion on MoneyAmount.amount_minor |
| g | Invalid run_id / invalid evidence_ref | typed error returned |
