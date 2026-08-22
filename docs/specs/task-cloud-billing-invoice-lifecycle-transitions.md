# Spec: Cloud Billing Invoice Lifecycle Transitions

| Field | Value |
|-------|-------|
| Task slug | `cloud-billing-invoice-lifecycle-transitions` |
| Vertical | cloud |
| Crate | `cloud-billing-domain` |
| Branch | `feat/task-cloud-billing-invoice-lifecycle-transitions-2026-05-28` |
| Stage | SPEC |

## Objective

Extend the `cloud-billing-domain` kernel with two additive domain
operations inside the single existing `src/lib.rs`:

1. **Invoice lifecycle state-transition** — a typed, guarded method on
   `CloudBillingLedger` that enforces the legal transition graph over the
   existing `InvoiceState` machine, with explicit `CloudBillingError` variants
   for illegal moves and missing invoices.
2. **Validated credit-note path** — a `CreditNoteCreate` input type and ledger
   method that emits a negative-amount `InvoiceLineItem` against an existing
   invoice, reusing `sum_line_items` / `Money` invariants, and rejecting
   over-credit or targeting a `Void` invoice.

Scope is pure in-crate domain logic. No new workspace member, no root
`Cargo.toml` edit, no adapter or REST work.

## Vertical and Crate Context

`cloud-billing-domain` currently owns:

- `InvoiceState` enum: `Issued | Paid | Overdue | Void`
- `Invoice` struct with `state: Classified<InvoiceState>` (always starts as
  `Issued` on `Invoice::generate`)
- `CloudBillingLedger` with `invoices_by_id: BTreeMap<InvoiceId, Invoice>`,
  `generate_invoice`, `invoices()`
- `Money` with `checked_add` (private), `CurrencyCode` (3-char ASCII uppercase)
- `sum_line_items` (private) — computes subtotal from a line-item slice
- `InvoiceLineItem` / `InvoiceLineItemCreate` with `subtotal: Money`
- `CloudBillingError` enum — existing variants (no `InvoiceNotFound`,
  `IllegalInvoiceTransition`, `CreditNoteOverCredit`, `CreditNoteTargetVoid`)
- Data-boundary helpers: `internal()`, `public()`, `audit()`, `financial_data_class()`

All new code is additive inside the existing single `src/lib.rs` per the
flat-clean-arch / single-crate-per-service doctrine (ADR-0509).

## Transition Graph

```
Issued  ──► Paid
Issued  ──► Overdue
Issued  ──► Void
Overdue ──► Paid
Overdue ──► Void
```

Terminal states `Paid` and `Void` have no outbound edges. Any attempt to
transition out of a terminal state, or a same-state no-op, returns
`CloudBillingError::IllegalInvoiceTransition { from, to }`.

## Error Taxonomy

New `CloudBillingError` variants added by this slice:

| Variant | When raised |
|---------|-------------|
| `InvoiceNotFound` | `transition_invoice` or `apply_credit_note` called with an id not present in `invoices_by_id` |
| `IllegalInvoiceTransition { from: InvoiceState, to: InvoiceState }` | Requested transition is outside the legal graph (including same-state, and any move from `Paid` or `Void`) |
| `CreditNoteOverCredit` | `credit_minor_units > invoice.subtotal.value.minor_units` |
| `CreditNoteTargetVoid` | `apply_credit_note` targets a `Void` invoice |

Existing variants reused by this slice: `InvalidInvoiceId`,
`InvalidInvoiceLineItemId`, `InvalidInvoiceTotal`, `InvalidInvoiceLineItem`,
`TenantMismatch`, `InvalidDataClass`, `InvalidCurrencyCode`,
`InvalidResourceId`.

## Module Layout (flat clean-arch, mods inside `src/lib.rs`)

```
src/lib.rs
  // --- existing (unchanged) ---
  pub enum   InvoiceState          { Issued, Paid, Overdue, Void }
  pub struct Invoice               { ... state: Classified<InvoiceState> ... }
  pub enum   CloudBillingError     { ... existing variants ... }
  pub struct CloudBillingLedger    { ... invoices_by_id ... }
  impl       CloudBillingLedger    { ingest, generate_invoice, invoices }
  fn         sum_line_items        (private)
  // Money::checked_add            (private)

  // --- new (additive) ---
  // CloudBillingError variants:
  //   InvoiceNotFound
  //   IllegalInvoiceTransition { from: InvoiceState, to: InvoiceState }
  //   CreditNoteOverCredit
  //   CreditNoteTargetVoid

  pub struct CreditNoteCreate      {
    invoice_id, line_item_id, resource_id,
    description, units, credit_minor_units,
    currency, data_class
  }

  impl Money
    fn checked_sub(&self, other: &Self) -> Result<Self, CloudBillingError>  // private

  impl CloudBillingLedger
    pub fn transition_invoice(
      &mut self,
      id: &InvoiceId,
      target: InvoiceState,
    ) -> Result<&Invoice, CloudBillingError>

    pub fn apply_credit_note(
      &mut self,
      tenant_id: &str,
      input: CreditNoteCreate,
    ) -> Result<&Invoice, CloudBillingError>
```

## Contracts

### OpenAPI 3.2.0 fragment (domain-level; REST adapter is out of scope)

```yaml
# Lifecycle transition — domain operation, not a REST endpoint in this slice.
# Shown as a conceptual contract for future REST adapter consumption.
#
# POST /v1/invoices/{invoice_id}/transitions
requestBody:
  required: true
  content:
    application/json:
      schema:
        type: object
        required: [target_state]
        properties:
          target_state:
            type: string
            enum: [Paid, Overdue, Void]
responses:
  '200':
    description: Invoice state updated
    content:
      application/json:
        schema:
          $ref: '#/components/schemas/Invoice'
  '404':
    description: Invoice not found (InvoiceNotFound)
  '409':
    description: Illegal state transition (IllegalInvoiceTransition)

# Credit note — domain operation, not a REST endpoint in this slice.
# POST /v1/invoices/{invoice_id}/credit-notes
requestBody:
  required: true
  content:
    application/json:
      schema:
        type: object
        required:
          [line_item_id, resource_id, description, units,
           credit_minor_units, currency, data_class]
        properties:
          line_item_id:       { type: string }
          resource_id:        { type: string }
          description:        { type: string, maxLength: 160 }
          units:              { type: array, items: { type: object } }
          credit_minor_units: { type: integer, format: int64, minimum: 1 }
          currency:           { type: string, pattern: '^[A-Z]{3}$' }
          data_class:         { type: string, enum: [Financial] }
responses:
  '200':
    description: Credit note applied; updated invoice returned
  '404':
    description: Invoice not found (InvoiceNotFound)
  '409':
    description: Over-credit (CreditNoteOverCredit) or target Void (CreditNoteTargetVoid)
  '422':
    description: Validation error (currency mismatch, invalid line-item, etc.)
```

### proto3 fragment (informational; gRPC adapter is out of scope)

```proto
syntax = "proto3";
package oya.cloud.billing.v1;

enum InvoiceState {
  INVOICE_STATE_UNSPECIFIED = 0;
  INVOICE_STATE_ISSUED      = 1;
  INVOICE_STATE_PAID        = 2;
  INVOICE_STATE_OVERDUE     = 3;
  INVOICE_STATE_VOID        = 4;
}

message TransitionInvoiceRequest {
  string        invoice_id   = 1;
  InvoiceState  target_state = 2;
}
message TransitionInvoiceResponse {
  Invoice invoice = 1;
}

message ApplyCreditNoteRequest {
  string invoice_id         = 1;
  string line_item_id       = 2;
  string resource_id        = 3;
  string description        = 4;
  repeated MeterUnit units  = 5;
  uint64 credit_minor_units = 6;
  string currency           = 7;
  string data_class         = 8;
}
message ApplyCreditNoteResponse {
  Invoice invoice = 1;
}

service CloudBillingService {
  rpc TransitionInvoice (TransitionInvoiceRequest)  returns (TransitionInvoiceResponse);
  rpc ApplyCreditNote   (ApplyCreditNoteRequest)    returns (ApplyCreditNoteResponse);
}
```

## Credit-Note Invariants

1. **No over-credit**: `credit_minor_units` must not exceed
   `invoice.subtotal.value.minor_units`. Violators return `CreditNoteOverCredit`.
2. **No credit against Void**: an invoice in state `Void` cannot receive a
   credit note. Violators return `CreditNoteTargetVoid`.
3. **Currency consistency**: `input.currency` must equal the invoice's existing
   subtotal currency code. Mismatch returns `InvalidInvoiceTotal`.
4. **Description prefix**: the stored `InvoiceLineItem.description` is prefixed
   with `"[CREDIT] "` to distinguish credit entries from charge entries.
5. **Positive input**: `credit_minor_units > 0` is enforced via
   `InvalidInvoiceLineItem` (same guard as the existing subtotal != 0 check).
6. **Subtotal update**: after the credit line item is appended,
   `invoice.subtotal` is decremented by `credit_minor_units` via the new
   private `Money::checked_sub`.
7. **Data class**: `input.data_class` must be `Financial` (same guard as
   `InvoiceLineItemCreate`).

## Testing Strategy

All tests live in `#[cfg(test)] mod tests` inside `src/lib.rs` (existing
pattern). The existing `account_create`, `invoice_generate`, `invoice_line_item`,
and `units` fixtures are reused.

### ST1 — transition tests (minimum coverage)

| Test name | Scenario |
|-----------|----------|
| `transition_issued_to_paid` | Legal: Issued → Paid |
| `transition_issued_to_overdue` | Legal: Issued → Overdue |
| `transition_issued_to_void` | Legal: Issued → Void |
| `transition_overdue_to_paid` | Legal: Overdue → Paid (chain via Issued→Overdue first) |
| `transition_overdue_to_void` | Legal: Overdue → Void |
| `transition_paid_rejects_all` | Illegal: Paid → Issued / Paid → Overdue / Paid → Void each return `IllegalInvoiceTransition` |
| `transition_void_rejects_all` | Illegal: Void → Issued / Void → Paid / Void → Overdue each return `IllegalInvoiceTransition` |
| `transition_same_state_rejected` | Illegal: Issued → Issued returns `IllegalInvoiceTransition` |
| `transition_invoice_not_found` | Unknown `InvoiceId` returns `InvoiceNotFound` |

### ST2 — credit-note tests (minimum coverage)

| Test name | Scenario |
|-----------|----------|
| `credit_note_reduces_subtotal` | Valid credit; subtotal decremented, line item appended |
| `credit_note_over_credit_rejected` | `credit_minor_units > subtotal` returns `CreditNoteOverCredit` |
| `credit_note_against_void_rejected` | Invoice in Void state returns `CreditNoteTargetVoid` |
| `credit_note_currency_mismatch_rejected` | Mismatched currency returns `InvalidInvoiceTotal` |

## Boundaries

- **In scope**: `crates/cloud-billing-domain/src/lib.rs` (additive changes
  only), `docs/specs/task-cloud-billing-invoice-lifecycle-transitions.md`,
  `tasks/cloud-billing-invoice-lifecycle-transitions-plan.md`.
- **Out of scope**: root `Cargo.toml`, any other crate, REST/gRPC adapters,
  OpenSLO files (`slos/*.openslo.yaml`), any existing test modifications.
- **SLO**: the crate's existing OpenSLO file is untouched; this slice adds no
  new SLO targets.
