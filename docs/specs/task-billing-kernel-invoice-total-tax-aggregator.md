# Spec: billing-kernel-invoice-total-tax-aggregator

## Objective

Add a pure, deterministic invoice-total aggregator to `cloud-billing-kernel`.
The function accepts a slice of `LineItem` and a tax rate in basis points, and returns
the combined `subtotal_micros`, `tax_micros`, and `total_micros` for that invoice.

## Crate boundary

Crate: `cloud-billing-kernel` (path: `crates/cloud-billing-kernel`).
No new workspace members. No new dependencies. No I/O.

## Flat clean-arch mod layout (ADR-0509)

All code lives in `src/lib.rs` (single-file kernel — already established pattern in this crate).
No new modules required given the minimal surface.

## Contracts

- **No external HTTP/gRPC contracts** — pure domain function; no OpenAPI/proto changes.
- **No SLO changes** — no I/O path added.
- **No new deps** — arithmetic is standard Rust `u128` checked ops.

## Public API added

### `InvoiceTotals`

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvoiceTotals {
    pub subtotal_micros: u128,
    pub tax_micros: u128,
    pub total_micros: u128,
}
```

### `BillingError::SubtotalOverflow` (new variant)

Added to the existing `BillingError` enum. Emitted when:
- `subtotal_micros * tax_rate_basis_points` overflows `u128`, or
- any checked addition overflows `u128`.

### `aggregate_invoice`

```rust
pub fn aggregate_invoice(
    lines: &[LineItem],
    tax_rate_basis_points: u32,
) -> Result<InvoiceTotals, BillingError>
```

**Semantics:**

1. Empty `lines` → `Ok(InvoiceTotals { subtotal_micros: 0, tax_micros: 0, total_micros: 0 })`.
2. For each line, call `finalize_line(line)` to enforce tax-profile admission.
   First error is returned immediately (fail-fast).
3. Accumulate subtotals with `checked_add`; overflow → `BillingError::SubtotalOverflow`.
4. Compute tax: `tax_micros = round_half_up(subtotal_micros × bps / 10_000)`.
   Round-half-up via: `(subtotal_micros × bps_u128 + 5_000) / 10_000`.
   Overflow of the multiply → `BillingError::SubtotalOverflow`.
5. `total_micros = subtotal_micros.checked_add(tax_micros)` → overflow → `SubtotalOverflow`.

## Money-math correctness

All amounts in micros (µ-currency units, integer). No floating point.
Basis points: 1 bp = 0.01% = 1/10_000. Tax = subtotal × bps / 10_000.
Round-half-up: adding half the divisor (5_000) before integer division is the
standard integer round-half-up identity for non-negative values. This is exact
as long as `subtotal × bps` fits in `u128`; the overflow guard protects this.

## Testing strategy

Hermetic unit tests in `#[cfg(test)] mod tests` within `src/lib.rs`.

| Test | Criterion |
|------|-----------|
| `aggregate_empty_lines_returns_zero` | (a) empty → zero totals |
| `aggregate_rejects_line_without_tax_profile` | (b) missing profile → error |
| `aggregate_multi_line_subtotal_correct` | (c) multi-line sum |
| `aggregate_tax_basis_points_rounding_exact` | (d) basis-point + round-half-up exact |
| `aggregate_overflow_yields_error` | (e) overflow → SubtotalOverflow |

Additional coverage:
- Zero tax rate (bps=0).
- 100% tax rate (bps=10_000).
- Single-line exact round-half-up case.
- Invalid line (zero quantity) propagated via `finalize_line`.

## Observability / SLO

Pure domain kernel; no I/O path; not on the SLO critical path. No SLO file changes needed.

## Security

No secrets, no network, no user input paths. Pure arithmetic on caller-supplied structs.
Overflow is handled explicitly; no panics in production code paths.
