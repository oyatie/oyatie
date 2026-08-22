---
ip_id: IP-002
microservice: cloud-billing
title: Kernel layer — usage record + line item + finalize_line tax-profile gate
wave: Wave-15B-cloud-billing-spec-sprint
date: 2026-05-21
owner: axis-cloud-billing
status: drafted
priority: P0
binding_adrs: [ADR-0330, ADR-0244, ADR-0131, ADR-0263, ADR-0083]
counterpart_parity: [Stripe Billing, Recurly, Zuora Billing, Chargebee]
capabilities_touched:
  - cap.cloud.billing.usage
  - cap.cloud.billing.issue_invoice
billing_components: [per_usage, per_seat, revenue_share]
tenant_class_scope: both
---

# IP-002 — Kernel layer: tax-profile-bound line finalization

## §A Objective

Document the existing `cloud-billing-kernel` crate (`crates/cloud-billing-kernel/src/lib.rs`, 185 lines) which encodes the rule that every billable line item must declare its tax-profile reference before finalization. This is the M03-P03-IP-002 minimum viable kernel — pure I/O-free, deterministic, and the canonical proof surface for "no line item ships without tax provenance."

This IP is distinct from IP-001 (`cloud-billing-domain`): the domain crate is the full aggregate-root model (account / event / invoice / line_item), while the kernel crate is the narrow algebra that closes the spec-compliance loop between usage record and finalized money amount.

## §B Scope

In scope:

- `UsageUnit` closed enum: `Cpu | GpuHour | GibStorage | GibEgress | Request | Token`.
- `UsageRecord {record_id, tenant_id, unit, quantity, timestamp_unix_ms}`.
- `LineItem {line_id, usage, unit_price_micros, tax_profile_ref}`.
- `subtotal_micros() → u128`: deterministic multiply with overflow-free 128-bit accumulator.
- `validate_usage(&UsageRecord) → Result<(), BillingError>`: kernel guard against empty record_id / empty tenant_id / zero quantity.
- `finalize_line(&LineItem) → Result<u128, BillingError>`: closes the rule "no finalize without tax_profile_ref."
- Error taxonomy: `BillingError::EmptyRecordId | EmptyTenantId | ZeroQuantity | NoTaxProfileRef { line_id }`.

Out of scope:

- Tax computation itself (delegated to IP-003 `cloud-billing-tax-app` and the per-pack tax engine).
- Currency conversion (the kernel works in `unit_price_micros`, a single-currency micro-unit; FX is upstream).
- Persistence (the kernel is I/O-free; persistence is in IP-006 REST API + IP-007 events + IP-010 audit-chain).

## §C Architecture

### §C.1 Micro-unit arithmetic

The kernel works in **micro-units** (1 micro-unit = 10^-6 of a unit of currency). `LineItem.unit_price_micros: u64` × `UsageRecord.quantity: u64` = `subtotal_micros: u128`. The 128-bit accumulator guarantees no overflow for invoices up to 2^128 - 1 micro-units (≈ 3.4 × 10^32 currency-units), far beyond any plausible enterprise commitment. This matches Stripe's internal micro-unit convention (1 millionth of the smallest currency unit, i.e. 10^-8 of a USD).

### §C.2 Why "no tax_profile_ref → fail"

The kernel's central invariant is `tax_profile_ref.is_none() → Err(NoTaxProfileRef)`. This closes a class of bugs where line items get finalized with the assumption "we'll resolve tax later," then ship to the invoice issuer in tax-naive form. Such drift causes regulatory exposure under K-FSI (which requires tax-format provenance from event ingestion onward), CSAP-KR (e-Tax-Invoice clearance), MAS-TRM (audit-trail completeness), and SOX-404 (segregation between tax computation and invoice issuance).

The tax_profile_ref is opaque to the kernel; resolution is upstream of the kernel (cloud-billing-tax µservice). What the kernel guarantees is that an unresolved tax profile cannot leak into finalize. This is the moral equivalent of TypeScript's `strictNullChecks` for tax provenance.

### §C.3 UsageUnit enum closure

The six canonical units (`cpu-hour`, `gpu-hour`, `gib-storage-month`, `gib-egress`, `request`, `token`) cover Phase-0 substrate metering. Adding a unit requires (a) extending the enum variant, (b) extending the `name()` table, (c) updating downstream rate-card-manager to accept the variant, (d) updating finops-portal aggregation. Closed-enum discipline keeps drift bounded.

The `cpu-hour` unit covers VM-second, K8s pod-minute, function-invocation-second (translated upstream); the `gpu-hour` unit covers A100/H100/MI300 wall-clock; `gib-storage-month` is the integral of bytes over time normalized to GiB-month; `gib-egress` is the data leaving Oyatie's network boundary; `request` is the unit-less API call counter; `token` is the LLM tokens-in + tokens-out unit.

### §C.4 Error taxonomy

`BillingError` is a closed-enum sum type with `.message() → String` for downstream error-response binding. It does **not** carry source-location or `Box<dyn Error>` payloads; the kernel is pure. Wrapping happens in the API layer (IP-006).

## §D Lifecycle

### §D.1 Usage event → LineItem composition

1. Upstream caller composes `UsageRecord` from a metering bus event.
2. `validate_usage(&record)` is called eagerly to fail fast on shape violations.
3. The line item is constructed with `unit_price_micros` from the rate-card-manager (opaque resolution; `rate_card_ref` carries provenance back to a specific rate card version).
4. The tax-profile resolver (cloud-billing-tax µservice, IP-003) populates `tax_profile_ref`.
5. `finalize_line(&line)` is called when the invoice issuer is ready to seal the line item.

### §D.2 Failure modes

- Empty `record_id`: caller skipped idempotency key allocation; halt with `EmptyRecordId`.
- Empty `tenant_id`: cross-tenant leak attempt or upstream bug; halt with `EmptyTenantId`.
- Zero quantity: zero-usage event slipped through the meter without `Discard` policy; halt with `ZeroQuantity`.
- Missing tax_profile_ref: tax-profile resolver failed or skipped; halt with `NoTaxProfileRef { line_id }`.

All four halt the line item; none of them attempt a partial finalize. The upstream invoice issuer treats each as a permanent error (no retry-loop on `NoTaxProfileRef`) and emits a `cloud.billing.line_item.finalize_failed` event for operator review.

## §E Cedar Policy Bindings

- `cap.cloud.billing.usage` — guards `Meter::record` from upstream callers, but the kernel itself is I/O-free and does not call Cedar directly.
- `cap.cloud.billing.issue_invoice` — guards the assembly of finalized line items into an Invoice (IP-001 aggregate root). The kernel's `finalize_line` is the structural prerequisite.

The kernel does not perform Cedar evaluation; it is the substrate algebra. Cedar evaluation happens at the gRPC service boundary (IP-009) before the kernel functions are invoked.

## §F Evidence

### §F.1 Source files

- `/Users/jasonlee/oyatie/crates/cloud-billing-kernel/src/lib.rs` (185 lines, 6 tests).
- `/Users/jasonlee/oyatie/crates/cloud-billing-kernel/Cargo.toml`.

### §F.2 Tests demonstrating invariants

- `subtotal_multiplies_quantity_and_price`: proves the 128-bit accumulator (5 × 100 = 500).
- `finalize_valid_line_returns_subtotal`: proves successful finalize round-trips the subtotal.
- `finalize_without_tax_profile_errors`: proves the central invariant — no tax profile → no finalize.
- `zero_quantity_errors`: proves quantity guard.
- `empty_record_id_errors`: proves record-id presence guard.
- `empty_tenant_id_errors`: proves tenant-id presence guard.
- `usage_unit_names_distinct`: proves enum closure (6 distinct names).

### §F.3 Downstream consumers

- `cloud-billing-tax-app` (IP-003): resolves `tax_profile_ref` from regional pack + line item kind.
- `cloud-billing-invoice-worker` Kubernetes deployment (IaC `microservices/cloud-billing/iac/oyatie-public-cloud/invoice-worker.tf`): calls `finalize_line` per line item before assembling `InvoiceGenerate`.
- `finops-portal`: reads finalized subtotals via the IP-006 OpenAPI invoice endpoint.

### §F.4 ADR anchors

- ADR-0330 §B.10.6: rate-card resolution is per-line-item, not per-account.
- ADR-0263: every state mutation (including finalize) seals into audit-chain (IP-010).
- ADR-0083 Tier 3 exemption: kernel tests use `.unwrap()` legitimately.

## §G Counterpart parity

| Counterpart | Their concept | Oyatie equivalent | Delta |
|---|---|---|---|
| Stripe Billing | `InvoiceItem.amount_decimal` (cents-with-decimals string) | `LineItem.subtotal_micros: u128` (10^-6 minor units) | Oyatie uses micro-units for arithmetic; cents at issuance time (IP-001 aggregate). Avoids float-decimal drift entirely. |
| Stripe Billing | `tax_rates: [TaxRate.id]` on InvoiceItem | `tax_profile_ref: String` on LineItem | Stripe permits multiple rates per item; oyatie binds one profile (the profile internally fans out to multiple jurisdictional rates per IP-003). |
| Recurly | `LineItem.tax_exempt: bool` + `Account.entity_use_code` | `tax_profile_ref` (either present or item halts) | Oyatie is stricter: there is no "tax exempt by default" path — exemption is a documented profile. |
| Zuora Billing | `Charge.taxable: bool` + `TaxInclusive` enum | `tax_profile_ref` declares format + jurisdiction + inclusivity | Oyatie collapses 3 Zuora fields into 1 opaque ref that the tax engine resolves. |
| Chargebee | `LineItem.tax_amount`, `tax_rate`, `tax_juris_code` | Tax fields computed by IP-003 from `tax_profile_ref`; line item carries only the ref | Same outcome, cleaner separation. |
| Stripe (Tax product) | Stripe Tax APIs computing tax via Customer.tax_ids | Tax engine (IP-003) computing tax via `tax_profile_ref` resolved from `regional_pack` | Comparable scope; oyatie binds to regional pack rather than customer-level tax ids per ADR-0064 canonical-base + localization rule. |

## §H Open questions

- Whether `UsageUnit::Token` should be split into `InputToken` and `OutputToken` for LLM cost asymmetry. Current decision: keep one unit; rate-card-manager declares per-direction price via two LineItems. Revisit if downstream tax engines require distinct provenance.
- Whether `UsageUnit` should accept user-extension variants. Current decision: closed enum, drift-bounded by design.
