---
ip_id: IP-001
microservice: cloud-billing
title: Domain layer — Invoice + Charge + Refund + Subscription + BillingAccount bounded contexts
wave: Wave-15B-cloud-billing-spec-sprint
date: 2026-05-21
owner: axis-cloud-billing
status: drafted
priority: P0
binding_adrs: [ADR-0330, ADR-0329, ADR-0244, ADR-0131, ADR-0145, ADR-0263, ADR-0083]
counterpart_parity: [Stripe Billing, Recurly, Zuora Billing, Chargebee]
capabilities_touched:
  - cap.cloud.billing.read_tenant_class
  - cap.cloud.billing.issue_invoice
  - cap.cloud.billing.emit_usage_event
  - cap.cloud.billing.purchase_reservation
billing_components: [revenue_share, per_seat, per_usage]
tenant_class_scope: both
---

# IP-001 — Domain layer: bounded contexts and value-typed identifiers

## §A Objective

Document the existing `oya-cloud-billing-domain` crate (`crates/oya-cloud-billing-domain/src/lib.rs`, 1,030 lines) as the canonical billing aggregate root. The kernel already encodes nine bounded contexts (BillingAccount, CloudBillingEvent, Invoice, InvoiceLineItem, BillingPeriod, Money, CurrencyCode, TaxRegistrationId, PaymentMethodRef) as value-typed Rust structs whose invariants are enforced at construction time and whose data-class classification is enforced through `oya_data_boundary_kernel::Classified<T>`. This IP closes the kernel-ahead-of-spec gap by formalizing the domain contract so downstream consumers (cloud-iam, finops-portal, payments, cloud-billing-tax-app, audit-chain) can bind to a stable surface.

## §B Scope

In scope:

- Identifier types: `BillingAccountId` (prefix `ba_`), `CloudBillingEventId` (`cbill_`), `InvoiceId` (`inv_`), `InvoiceLineItemId` (`ili_`), `TaxRegistrationId` (multi-format), `PaymentMethodRef` (`pm_`), `RateCardRef` (`rate/`).
- Value types: `CurrencyCode` (3-letter ASCII uppercase per ISO 4217 + the reserved internal credit code `OYC`), `Money` (currency + minor_units `u64`), `BillingPeriod` (epoch-seconds bounds with `start < end`).
- Aggregate roots: `BillingAccount`, `CloudBillingEvent`, `Invoice`, `InvoiceLineItem`.
- State enums: `BillingAccountState ∈ {Active, Suspended, Delinquent}`, `InvoiceState ∈ {Issued, Paid, Overdue, Void}`, `CloudBillingEventKind ∈ {ResourceCreated, ResourceTerminated, Usage, Reservation, Commitment, Credit}` (extended by proto3 to include `RevenueShare`, `RevenueShareReversal`, `SeatCount`, `Subscription`).
- Tax invoice format binding: `TaxInvoiceFormat::for_regional_pack(&str) → Result` derives format from regional pack identifier (`oya-pack-electronic-tax`, `oya-pack-qualified-tax`, `oya-pack-country-tax`, `oya-pack-market-tax`, `oya-pack-trade-tax`, `oya-pack-vat-tax`, `oya-pack-gst-tax`, `oya-pack-fiscal-tax`, `oya-pack-clearance-tax`, `oya-pack-registration-tax`).
- Schema versioning: `BILLING_ACCOUNT_SCHEMA_VERSION = 1`, `CLOUD_BILLING_EVENT_SCHEMA_VERSION = 1`, `CLOUD_INVOICE_SCHEMA_VERSION = 1`.
- Ledger primitive: `CloudBillingLedger` with O(log n) by-id maps and idempotency map.

Out of scope (covered by sibling IPs):

- Tax computation (IP-003).
- Composable billing_components semantics (IP-004).
- API request/response shapes (IP-006/007/008).
- Cedar policy bindings (IP-009).
- Audit-chain seal emission (IP-010).

## §C Architecture

### §C.1 Identifier hygiene (prefixed-token rule)

Every billing identifier is a prefixed token: `prefix + opaque_body` where `body.len() > 0` and `body` is opaque to consumers. Construction goes through a single `prefixed_id` helper that returns `Err(InvalidXxx)` on any prefix mismatch. This rule eliminates "what kind of ID is this" reflection at the API boundary and lets gRPC/REST handlers do shape-check at deserialization rather than at policy time.

`TaxRegistrationId::new(value, format)` is the exception: the body shape depends on the regional pack the invoice was issued under. The kernel encodes the per-format body rule directly (`electronic` → 10 ASCII digits; `qualified` → `T` + 13 digits; `vat` → `vat/` + 8+ ASCII token; `gst` → 15 alphanumeric; `fiscal` → 14 digits; `clearance` → 15 digits; `registration` → 15 digits). This is the only structural validator that varies by data-context; everything else is prefix-only.

### §C.2 Money arithmetic

`Money` is `{currency: CurrencyCode, minor_units: u64}`. The kernel forbids cross-currency arithmetic at the type-system level via `checked_add` which returns `Err(InvalidInvoiceTotal)` on mismatch. There is no implicit FX inside the domain layer; FX is an explicit step in `IP-003` tax / `IP-011` settlement. `u64` minor_units bounds invoice line items at 2^64 - 1 minor units, far above any plausible enterprise invoice; overflow on `checked_add` is reported, not silently wrapped.

### §C.3 Classified<T> data-class wrapping

Every aggregate field carries a `Classified<T>` wrapper from `oya_data_boundary_kernel`. The Classified envelope binds a value to a DataClass at compile time. Cloud-billing's domain rules:

- `Public`: region, schema_version, CloudBillingEvent.data_class.
- `InternalOnly`: every identifier, billing period, money amounts, line items.
- `Financial`: tax_registration_id, BillingAccount.data_class.

This is enforced by two private helpers: `public<T>(value)` and `internal<T>(value)`. The aggregate constructor for `BillingAccount` enforces `financial_data_class` (`Err(InvalidDataClass)` if the caller passes anything else); `CloudBillingEvent` enforces `public_data_class`. This wins at the type system layer the property that "no Internal data-class leaks into a Public surface" without runtime validation.

### §C.4 Tenant-scope hygiene

Tenant id is validated by `validate_tenant_id` (must start with `ten_` per ADR-0244 reserved namespace contract; the `oyatie` system tenant uses `ten_oyatie_*`). Every aggregate carries `tenant_id` and rejects construction when:

- `BillingAccount.tenant_id` mismatches the line-item resource's embedded tenant.
- `CloudBillingEvent.tenant_id` mismatches the resource's tenant component.
- `Invoice.tenant_id` mismatches `BillingAccount.tenant_id`.

`Invoice::generate` cross-checks tenant identity three ways: against the account, against every line item's resource, and against the metering tag. Tenant misuse short-circuits before any state mutation.

### §C.5 Idempotency at the aggregate boundary

`CloudBillingEventCreate.idempotency_key` is required (non-empty string, no length cap because the kernel does not own the key allocation). `CloudBillingLedger.ingest` looks up the key in `events_by_idempotency`. On hit, the existing event is replayed (re-records through the meter, returns the original event id). This is the foundation for the gRPC `EmitUsageEventResponse.idempotent_replay` flag and the REST `Idempotency-Key` semantics.

`CloudBillingLedger.generate_invoice` enforces invoice-id idempotency by checking `invoices_by_id.contains_key` before insertion (returns `Err(DuplicateInvoice)`). Invoice id immutability is the foundation for ADR-0263 audit-chain seal stability.

### §C.6 Metering kernel handoff

`CloudBillingEvent::to_meter_event_create() → MeterEventCreate` produces a `oya_metering_domain::MeterEventCreate` whose `capability_id` is derived from `CloudBillingEventKind::capability_id()`:

| CloudBillingEventKind | capability_id |
|---|---|
| ResourceCreated, ResourceTerminated | `cap.cloud.billing.resource-lifecycle` |
| Usage | `cap.cloud.billing.usage` |
| Reservation | `cap.cloud.billing.reservation` |
| Commitment | `cap.cloud.billing.commitment` |
| Credit | `cap.cloud.billing.credit` |

Source axis is fixed to `AxisId::Cloud`; plane is fixed to `PlaneTag::Data`. Mtr id is derived deterministically from cbill id by prefix swap, which preserves the 1:1 correspondence between billing event and meter event for replay.

## §D Lifecycle

### §D.1 BillingAccount creation

1. Caller submits `BillingAccountCreate {id, tenant_id, region, regional_pack, payment_method, credit_balance, state, data_class, created_at_epoch_seconds}`.
2. Kernel validates: id prefix `ba_*`, tenant_id `ten_*`, region resolves to a `RegionCode`, regional_pack starts with `oya-pack-`, payment_method `pm_*`, data_class is `Financial`.
3. On success returns `BillingAccount` with `schema_version = 1` and every field classified.

### §D.2 CloudBillingEvent ingestion

1. Caller submits `CloudBillingEventCreate {id, tenant_id, resource_id, region, metering_tag, kind, units, rate_card_ref, occurred_at_epoch_seconds, idempotency_key, data_class}`.
2. Kernel validates: id prefix `cbill_*`, tenant prefix, resource_id is a valid `ResourceId` whose embedded tenant + region match the event, metering_tag is `oya:metering:{tenant_id}:{resource_kind_label}`, rate_card_ref `rate/*`, occurred_at non-zero, data_class is `Public`.
3. `Meter::record(event.to_meter_event_create())` is invoked atomically with ledger insertion. Either both succeed or both fail.

### §D.3 Invoice generation

1. Caller submits `InvoiceGenerate {id, billing_account_id, tenant_id, regional_pack, period, line_items, subtotal, tax, total, tax_invoice_format, tax_registration_id, issued_at_epoch_seconds, due_at_epoch_seconds, data_class}`.
2. Kernel cross-checks: account identity match, tenant match, regional pack match, account state is `Active`, `due_at > issued_at >= period.end_epoch_seconds`, expected `TaxInvoiceFormat::for_regional_pack(regional_pack) == tax_invoice_format`, tax_registration_id is well-formed for the format.
3. Kernel re-computes `subtotal = Σ line_items.subtotal` and verifies caller's value matches; same for `total = subtotal + tax`.
4. State is set to `InvoiceState::Issued`; schema_version stamped; aggregate returned.

## §E Cedar Policy Bindings

This IP is the data-shape contract for the following Cedar gates (defined in IP-009; full text in `microservices/cloud-billing/policies/cloud-billing.cedar` and sibling fragments):

- `cap.cloud.billing.issue_invoice` — guards `CloudBillingLedger::generate_invoice`.
- `cap.cloud.billing.emit_usage_event` — guards `CloudBillingLedger::ingest`.
- `cap.cloud.billing.read_tenant_class` — guards principal-issuance read by cloud-iam.
- `cap.cloud.billing.purchase_reservation` — guards Reservation aggregate (extension in IP-002 kernel).
- `cap.cloud.billing.void_invoice` — guards state transition `Issued → Void`.

Resource attribute schema for the Cedar evaluator (per `policies/tenant-class-binding.cedar`):

- `resource.tenant_id` — string, derived from `BillingAccount.tenant_id` or `Invoice.tenant_id`.
- `resource.tenant_class` — string `demo_trial | paid` (snapshot at audited operation per ADR-0330 §B.12.2).
- `resource.billing_components` — set of `revenue_share | per_seat | per_usage`.

## §F Evidence

### §F.1 Source files (real, present in repo)

- `/Users/jasonlee/oyatie/crates/oya-cloud-billing-domain/src/lib.rs` (1,030 lines, 9 aggregate roots, 23 invariants, 8 unit tests).
- `/Users/jasonlee/oyatie/crates/oya-cloud-billing-domain/Cargo.toml` (workspace member, dependencies on `oya-cloud-region-domain`, `oya-cloud-resource-domain`, `oya-data-boundary-kernel`, `oya-metering-domain`).

### §F.2 Tests demonstrating invariants

- `validates_billing_account_financial_class_and_regional_pack` (lines 861–868): proves `BillingAccount::new` rejects non-Financial data classes and validates regional pack prefix.
- `ingests_cloud_billing_event_through_platform_meter` (lines 870–886): proves `CloudBillingLedger::ingest` routes through `Meter::record` with correct capability_id and source axis.
- `billing_event_idempotency_replays_original_event_and_meter_record` (lines 888–908): proves idempotency-key replay returns the original event without double-recording.
- `generates_electronic_tax_invoice_with_regional_format_and_exact_totals` (lines 910–924): proves invoice generation enforces format derivation and total recomputation.
- `ledger_rejects_duplicate_invoice_ids` (lines 926–939): proves invoice id immutability.
- `rejects_invoice_format_tax_registration_total_and_inactive_account` (lines 941–987): proves format/registration/total/state cross-checks.
- `rejects_resource_tenant_region_and_metering_tag_mismatch` (lines 989–1012): proves tenant/region/metering-tag triple-check.
- `rejects_non_public_event_metadata_and_non_financial_account_class` (lines 1014–1029): proves data-class wall.

### §F.3 Downstream contracts

- proto3 service definitions: `/Users/jasonlee/oyatie/microservices/cloud-billing/contracts/proto/cloud-billing.proto` (services: TenantClassApi, BillingAccountApi, MeteringApi, InvoiceApi, ReservationApi, SettlementApi, SubscriptionApi, SeatCountApi, FxLockApi, DunningApi, ExportApi).
- OpenAPI 3.2.0 surface: `/Users/jasonlee/oyatie/contracts/openapi/cloud/cloud-billing-invoice-v1.yaml` (544 lines, invoice generate endpoint).
- AsyncAPI 3.1.0 envelope: `/Users/jasonlee/oyatie/contracts/asyncapi/cloud/cloud-billing-events-v1.yaml` (CloudEvents 1.0 + Protobuf payload).

### §F.4 ADR anchors

- ADR-0330 §B.10: cloud-billing is source-of-truth for tenant_class + billing_components.
- ADR-0244: tenant scoping primitive — every billing aggregate carries `tenant_id`.
- ADR-0263: every state mutation produces an audit-chain seal.
- ADR-0145: direct gRPC + 3 invariants (no Workflow+Ontology adapter).
- ADR-0083 Tier 3: tests legitimately use `.unwrap()` under `cfg(test)`.

## §G Counterpart parity

| Counterpart | Their concept | Oyatie equivalent | Delta |
|---|---|---|---|
| Stripe Billing | `Customer` + `Invoice` + `InvoiceItem` + `Charge` | `BillingAccount` + `Invoice` + `InvoiceLineItem` + `CloudBillingEvent` | Stripe is customer-rooted; oyatie is tenant-rooted (multi-account-per-tenant supported via `BillingAccountId`). |
| Stripe Billing | `idempotency_key` HTTP header on every POST | `idempotency_key` on `CloudBillingEventCreate`; `Idempotency-Key` header on REST invoice endpoint; gRPC takes it on event create | Behavioral parity (replay returns original record). |
| Recurly | `Account` + `Invoice` + `LineItem` | `BillingAccount` + `Invoice` + `InvoiceLineItem` | Recurly's `Account.code` ≅ `BillingAccount.id` (prefix `ba_`). |
| Recurly | `currency` per Account; multi-currency Account requires explicit setup | `CurrencyCode` per `Money`; no implicit FX | Oyatie is stricter — cross-currency arithmetic compile-fails. |
| Zuora Billing | `Subscription` aggregate + `RatePlan` + `RatePlanCharge` | `Subscription` proto + `rate_card_ref` | Subscription primitive lives in proto + IP-002 kernel extension; rate-card ref is opaque (rate-card-manager µservice owns the body). |
| Zuora Billing | Closed-period invoice immutability | `CloudBillingLedger::generate_invoice` returns `DuplicateInvoice` on re-issue | Behavioral parity. |
| Chargebee | `Customer` (with `taxability` field) | `BillingAccount` + `regional_pack` (which derives `TaxInvoiceFormat`) | Oyatie binds taxability to regional pack, not customer-level field. This is by design — sovereign deployments require pack-driven tax behavior. |
| Chargebee | Plan-based subscription with metered components | `Subscription` + `billing_components: {per_seat, per_usage, revenue_share}` (subset) | Oyatie's tri-component composition is strictly more expressive than Chargebee's metered-plan model. |

## §H Open questions

- Whether the kernel should expose a `Refund` aggregate distinct from "negative `CloudBillingEvent` of kind Credit". Current decision: Credit kind covers refunds; an explicit Refund aggregate would duplicate state. Revisit if SOX-404 auditors require a typed refund record.
- Whether `BillingAccountState::Delinquent` should auto-transition from `Active` after N days of unpaid invoices. Current decision: transition is performed by `dunning-policy-worker` (not in kernel) and reflected via account update event.
