---
ip_id: IP-002
ip_status: ready
slice_owner: ops-finops
authored: 2026-05-18
slice: finops-portal/tenant-billing-presentation/domain
related_adrs: [ADR-0131, ADR-0174, ADR-0199]
depends_on: [IP-001]
follow_up_owner: evidence/storage-batch-followup-scope.json#finops-portal-ip-fanout
target_lines: 150
---

# IP-002 — `tenant-billing-presentation` domain slice

## Why this slice

Per ADR-0131 flat layout, every BC has a domain layer that hosts the
business-rule logic over the kernel types. For tenant-billing
presentation, the domain owns:

- Invoice composition from line items + credit applications.
- Period boundary rules (calendar month vs fiscal quarter).
- Aggregation rules (sub-cost-center rollup; per ADR-0174 chargeback
  formula).
- Validation invariants (non-negative line amounts; period non-empty;
  credit application does not exceed line total).
- Currency rules — invoices are USD canonical; locale-presented
  currencies are converted at the API layer, never in domain.

The domain layer **does not** touch the network, the database, or the
clock; it accepts inputs and produces outputs deterministically. This
is the layer where chargeback-formula correctness is unit-testable.

## Acceptance criteria

1. New crate `crates/oya-finops-portal-tenant-billing-presentation-domain/`
   depends on the kernel from IP-001 and on the shared `oya-chargeback`
   crate (per ADR-0174).
2. Public function `InvoiceComposer::compose(...)` takes line items +
   credit applications and produces a validated `TenantInvoice`:
   ```rust
   pub fn compose(
       tenant: TenantId,
       period: InvoicePeriod,
       lines: Vec<InvoiceLine>,
       credits: Vec<CreditApplication>,
   ) -> Result<TenantInvoice, DomainError>;
   ```
3. Public function `roll_up_by_cost_center(invoice: &TenantInvoice) ->
   BTreeMap<CostCenter, USD>` returns deterministic rollup; ordering
   stable for snapshot tests.
4. `DomainError` enumerates:
   - `NegativeLineAmount { line: usize, amount_cents: i64 }`.
   - `EmptyPeriod`.
   - `CreditExceedsLineTotal { credit_source: String }`.
   - `ChargebackFormulaMisuse(String)` (delegated check from
     `oya-chargeback`).
5. ≥ 6 unit tests:
   - happy path composition with mixed cost-centers.
   - rejects negative line amounts (one negative + others positive).
   - rejects empty period.
   - applies credits correctly (sums subtract; negotiated vs
     committed-use ordered deterministically by `applied-at`).
   - rolls up by cost-center matches a hand-computed table.
   - fiscal-quarter boundary handling (calendar-month period spanning
     a quarter-close marks the period `is_quarter_close = true`).
6. `cargo test -p oya-finops-portal-tenant-billing-presentation-domain`
   green; `cargo clippy -- -D warnings` green.

## File-level work plan

1. `Cargo.toml` — depends on the kernel crate, on
   `oya-chargeback` (read-only), and on `time` (no async runtime).
2. `src/lib.rs` — module exports.
3. `src/compose.rs` — `InvoiceComposer::compose` + helpers.
4. `src/rollup.rs` — `roll_up_by_cost_center` + tests.
5. `src/error.rs` — `DomainError` enum.
6. Workspace `Cargo.toml` — register.

## Domain rules (the actual business logic)

1. **Credit application ordering**: credits apply in `applied-at`
   ascending; ties broken by source (negotiated < committed-use <
   refund). Stable ordering matters because partial credits leave
   remainder that the next credit consumes.
2. **Negative-after-credit**: if credits exceed total line amount, the
   invoice clamps total to zero and records the carry-forward credit
   in `CreditApplication::carry_forward`. Carry-forward is reported in
   the next invoice's `prior_carry_forward` field.
3. **Quarter-close marker**: the domain marks the period as
   `is_quarter_close = true` if `period.end` falls on the last day of
   a fiscal quarter (calendar-year quarters: Mar 31, Jun 30, Sep 30,
   Dec 31). This is the marker IP-015 uses to trigger the regulator
   emit.
4. **No rounding in domain**: line amounts are integer cents; rounding
   to display currency happens at the API layer.

## Risk + mitigation

- **Risk**: credit-application ordering nondeterminism caused by
  timestamp ties. **Mitigation**: tie-break by source then by
  insertion index; covered by a dedicated unit test.
- **Risk**: chargeback-formula misuse against `oya-chargeback`.
  **Mitigation**: call only through the public `apply_chargeback`
  function and propagate the result; do not re-implement.

## Out-of-scope

- Persistence — usecase layer (IP-004).
- API exposure — IP-005.
- Multi-currency display — API layer (IP-005).

## References

- ADR-0174 — chargeback formula.
- ADR-0199 — cost-attribution canonical.
- ADR-0131 — per-microservice flat layout.

## Verification

- `cargo test -p oya-finops-portal-tenant-billing-presentation-domain`.
- `cargo clippy -p oya-finops-portal-tenant-billing-presentation-domain
  -- -D warnings`.
