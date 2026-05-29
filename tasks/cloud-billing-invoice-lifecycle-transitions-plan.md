# Plan: cloud-billing-invoice-lifecycle-transitions

Vertical: cloud
Crate: oya-cloud-billing-domain
Branch: feat/task-cloud-billing-invoice-lifecycle-transitions-2026-05-28

## Objective

Extend the existing `oya-cloud-billing-domain` crate (single `src/lib.rs`) with
two additive domain operations:

1. A guarded invoice-lifecycle state-transition method on `CloudBillingLedger`
   that enforces the legal transition graph and returns typed errors for every
   illegal move.
2. A validated credit-note path that emits a negative-amount
   `InvoiceLineItem` against an existing invoice, reusing `sum_line_items` /
   `Money` invariants and rejecting over-credit or targeting a `Void` invoice.

Pure in-crate domain extension. No new workspace member, no root `Cargo.toml`
edit, no adapter or REST work.

## Subtasks

### [ST1] transition_invoice + CloudBillingError variants

Add to `src/lib.rs`:

- `CloudBillingError::InvoiceNotFound` — invoice id not present in the ledger.
- `CloudBillingError::IllegalInvoiceTransition { from: InvoiceState, to: InvoiceState }` —
  the requested (from, to) pair is outside the legal graph.
- `CloudBillingLedger::transition_invoice(id: &InvoiceId, target: InvoiceState) -> Result<&Invoice, CloudBillingError>` —
  looks up the invoice, enforces the legal transition graph, mutates
  `invoice.state` in-place, returns a reference to the updated invoice.

Legal transition graph (exhaustive; all other (from, to) pairs are illegal):

| From    | To     |
|---------|--------|
| Issued  | Paid   |
| Issued  | Overdue|
| Issued  | Void   |
| Overdue | Paid   |
| Overdue | Void   |

Terminal states `Paid` and `Void` reject all outbound transitions.
Idempotency: if `from == to` the method returns `IllegalInvoiceTransition`
(callers must not silently re-apply the same state).

Acceptance:
- `cargo check -p oya-cloud-billing-domain --all-targets` passes.
- Unit tests (in `#[cfg(test)] mod tests`) cover every legal transition (5 cases)
  and every illegal transition including Paid->*, Void->*, and same-state
  (at minimum 7 illegal cases), asserting the exact `CloudBillingError` variant.
- `cargo nextest run -p oya-cloud-billing-domain` green.

### [ST2] CreditNoteCreate + apply_credit_note

Add to `src/lib.rs`:

- `CreditNoteCreate` struct:
  - `invoice_id: String` — target invoice id (must be `inv_` prefixed).
  - `line_item_id: String` — id for the new credit line item (`ili_` prefixed).
  - `resource_id: String` — resource the credit corrects.
  - `description: String` — non-empty, ≤160 chars.
  - `units: Vec<MeterUnit>` — non-empty.
  - `credit_minor_units: u64` — amount to credit (positive; the method negates).
  - `currency: String` — 3-char ISO code; must match invoice currency.
  - `data_class: DataClass` — must be `Financial`.
- `CloudBillingError::CreditNoteOverCredit` — credit would push the invoice
  subtotal below zero (i.e., `credit_minor_units > invoice.subtotal.minor_units`).
- `CloudBillingError::CreditNoteTargetVoid` — credit note against a `Void`
  invoice is rejected.
- `CloudBillingLedger::apply_credit_note(tenant_id: &str, input: CreditNoteCreate) -> Result<&Invoice, CloudBillingError>` —
  validates input, looks up the invoice, enforces Void-rejection and
  over-credit guard, appends a new negative-signed `InvoiceLineItem` (subtotal
  `minor_units` stored as-is in `Money`; negative semantics conveyed by a
  leading description marker `"[CREDIT] "`), updates `invoice.subtotal` in-place
  using `Money::checked_sub`, returns a reference to the updated invoice.

`Money::checked_sub` (private helper, analogous to `checked_add`): same
currency guard, panics-free, returns `InvalidInvoiceTotal` on underflow.

Currency consistency: `input.currency` must equal `invoice.subtotal.value.currency.value`
(returns `InvalidInvoiceTotal` on mismatch).

Acceptance:
- Unit tests cover: valid credit reducing the subtotal, rejection when
  `credit_minor_units > subtotal.minor_units` (over-credit), rejection against a
  `Void` invoice, currency mismatch rejection.
- `Money` currency consistency enforced on `checked_sub`.
- `cargo nextest run -p oya-cloud-billing-domain` green.

### [ST3] Lane-namespaced docs

Create:
- `docs/specs/task-cloud-billing-invoice-lifecycle-transitions.md` — full spec
  (objective, vertical, contracts, mod layout, testing strategy, boundaries).
- `tasks/cloud-billing-invoice-lifecycle-transitions-plan.md` — this file.

Do not edit:
- Root `Cargo.toml` (no new workspace member).
- Any crate other than `oya-cloud-billing-domain`.
- `crates/oya-cloud-billing-domain/slos/*.openslo.yaml` (SLO is unchanged).

Acceptance:
- Both lane-namespaced docs exist.
- `cargo check -p oya-cloud-billing-domain --all-targets` green.
- `cargo nextest run -p oya-cloud-billing-domain` green.

## Acceptance (overall)

```
cargo check -p oya-cloud-billing-domain --all-targets   # zero errors
cargo nextest run -p oya-cloud-billing-domain            # all tests pass
```

No changes outside `crates/oya-cloud-billing-domain/src/lib.rs`,
`docs/specs/task-cloud-billing-invoice-lifecycle-transitions.md`, and
`tasks/cloud-billing-invoice-lifecycle-transitions-plan.md`.
