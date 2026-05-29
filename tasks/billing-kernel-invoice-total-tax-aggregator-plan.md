# billing-kernel-invoice-total-tax-aggregator — Plan

## Objective

Add `aggregate_invoice(lines, tax_rate_basis_points)` to `oya-cloud-billing-kernel`.
Pure deterministic kernel slice; no I/O; no new deps.

## Requirements Analysis

### Core function signature
```rust
pub fn aggregate_invoice(
    lines: &[LineItem],
    tax_rate_basis_points: u32,
) -> Result<InvoiceTotals, BillingError>
```

### InvoiceTotals return type
```rust
pub struct InvoiceTotals {
    pub subtotal_micros: u128,
    pub tax_micros: u128,
    pub total_micros: u128,
}
```

### Algorithm
1. **Empty line set** → return `Ok(InvoiceTotals::zero())` (zero rule chosen for composability;
   callers enforce business minimum-line policy above the kernel layer).
2. **Per-line finalization** → call `finalize_line(line)` for each; first error stops the whole
   aggregate (fail-fast). This reuses the existing tax-profile admission gate.
3. **Subtotal accumulation** → checked u128 addition; overflow → `BillingError::SubtotalOverflow`.
4. **Tax calculation** → `tax_micros = round_half_up(subtotal_micros * bps / 10_000)`.
   - Implemented with 256-bit-safe u128 arithmetic:
     `(subtotal * bps + 5_000) / 10_000` — adding half-divisor before integer division
     achieves round-half-up exactly when `subtotal * bps` does not overflow u128.
   - Guard: `subtotal_micros * u128::from(bps)` must not overflow; if it does →
     `BillingError::SubtotalOverflow`.
5. **Total** → `subtotal_micros.checked_add(tax_micros)` → overflow → `BillingError::SubtotalOverflow`.

### Edge cases
- Empty lines: return zero totals.
- Any line with `tax_profile_ref = None`: propagate `BillingError::NoTaxProfileRef`.
- `tax_rate_basis_points = 0`: tax_micros = 0, total = subtotal.
- `tax_rate_basis_points = 10_000`: tax = subtotal (100% tax rate).
- `tax_rate_basis_points > 10_000`: allowed (caller's policy domain; kernel is pure math).
- Single line with large quantity × price: subtotal u128 overflow → `SubtotalOverflow`.
- Multi-line overflow on sum: `SubtotalOverflow`.

### New BillingError variant
```rust
SubtotalOverflow
```

## Ordered Subtasks

1. [x] Write plan doc (this file).
2. [ ] Write spec doc `docs/specs/task-billing-kernel-invoice-total-tax-aggregator.md`.
3. [ ] Write failing tests (RED phase).
4. [ ] Implement `InvoiceTotals`, `BillingError::SubtotalOverflow`, `aggregate_invoice` (GREEN phase).
5. [ ] Self-review (correctness / arch / security / perf / cloud-native).
6. [ ] Simplify (guard clauses, naming, dead code).
7. [ ] Final `cargo nextest run -p oya-cloud-billing-kernel` green.
8. [ ] Commit + push + open PR.

## Acceptance Criteria

| # | Criterion | Test name |
|---|-----------|-----------|
| a | Empty line set returns zero totals | `aggregate_empty_lines_returns_zero` |
| b | Any line missing tax_profile_ref fails whole aggregate | `aggregate_rejects_line_without_tax_profile` |
| c | Multi-line subtotal sums correctly | `aggregate_multi_line_subtotal_correct` |
| d | Basis-point tax + rounding exact on representative cases | `aggregate_tax_basis_points_rounding_exact` |
| e | Overflow yields SubtotalOverflow error | `aggregate_overflow_yields_error` |
