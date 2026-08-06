---
ip_id: IP-003
microservice: cloud-billing
title: Tax computation — multi-jurisdiction TaxInvoiceFormat dispatch and tax_registration_id validation
wave: Wave-15B-cloud-billing-spec-sprint
date: 2026-05-21
owner: axis-cloud-billing
status: drafted
priority: P0
binding_adrs: [ADR-0330, ADR-0064, ADR-0244, ADR-0251, ADR-0131, ADR-0263]
counterpart_parity: [Stripe Tax, Avalara, Vertex, TaxJar, Chargebee Tax]
capabilities_touched:
  - cap.cloud.billing.issue_invoice
billing_components: [per_seat, per_usage, revenue_share]
tenant_class_scope: both
---

# IP-003 — Tax computation: regional pack → TaxInvoiceFormat dispatch

## §A Objective

Document the existing tax-format dispatch encoded in `oya-cloud-billing-domain` and the runtime invoice API surface in `oya-cloud-billing-tax-app` (276 lines). Tax computation in cloud-billing follows the **regional-pack-derives-format** rule per ADR-0064 canonical-base + localization: the regional pack identifier deterministically selects a `TaxInvoiceFormat`, which in turn imposes a shape on `tax_registration_id`. Per-jurisdiction tax rates and inclusion semantics are resolved by an upstream tax engine and surface to cloud-billing as the `tax_profile_ref` opaque string (IP-002).

This IP closes the spec gap on tax handling: cloud-billing's existing kernel encodes 7 invoice formats and 10 regional packs, but no IP documents the format/pack/registration triple.

## §B Scope

In scope:

- `TaxInvoiceFormat` closed enum (7 variants): `ElectronicTaxInvoice`, `QualifiedTaxInvoice`, `CountryEInvoice`, `GstTaxInvoice`, `FiscalDocumentInvoice`, `ClearanceQrInvoice`, `VatRegistrationInvoice`.
- Regional pack → format mapping: `TaxInvoiceFormat::for_regional_pack` table (10 packs map to 7 formats; some packs alias to `CountryEInvoice`).
- `tax_registration_id` body validation per format (10-digit ASCII, T-prefix 13-digit, 8+ ASCII token, 15-alphanumeric, 14-digit, 15-digit, 15-digit).
- Subtotal + tax + total reconciliation: `subtotal + tax = total` enforced at aggregate construction; cross-currency disallowed.
- API-surface tax fields: `CloudBillingInvoiceRecord.tax_invoice_format`, `tax_registration_id`, `tax_minor_units`.

Out of scope:

- Per-jurisdiction tax rate database (owned by upstream tax engine).
- Reverse charge mechanism for EU intra-community supply (handled by tax engine; cloud-billing accepts the resolved tax amount).
- Withholding tax (handled by IP-011 revenue attribution).

## §C Architecture

### §C.1 Regional pack canonical mapping table

| Regional pack | TaxInvoiceFormat | tax_registration_id shape | Typical jurisdictions |
|---|---|---|---|
| `oya-pack-electronic-tax` | `ElectronicTaxInvoice` | `taxid/electronic/` + 10 ASCII digits | KR e-Tax Invoice (NTS clearance) |
| `oya-pack-qualified-tax` | `QualifiedTaxInvoice` | `taxid/qualified/T` + 13 ASCII digits | JP Qualified Invoice (T-corp number) |
| `oya-pack-country-tax` | `CountryEInvoice` | `taxid/vat/` + 8+ ASCII token | EU VAT registration (varies by member state) |
| `oya-pack-market-tax` | `CountryEInvoice` | `taxid/vat/` + 8+ ASCII token | Aliased — marketplace seller VAT |
| `oya-pack-trade-tax` | `CountryEInvoice` | `taxid/vat/` + 8+ ASCII token | Aliased — cross-border trade VAT |
| `oya-pack-vat-tax` | `CountryEInvoice` | `taxid/vat/` + 8+ ASCII token | Aliased — explicit VAT |
| `oya-pack-gst-tax` | `GstTaxInvoice` | `taxid/gst/` + 15 alphanumeric | IN GST (GSTIN), AU GST, SG GST, NZ GST |
| `oya-pack-fiscal-tax` | `FiscalDocumentInvoice` | `taxid/fiscal/` + 14 ASCII digits | BR NFe (Nota Fiscal Eletrônica), IT FE |
| `oya-pack-clearance-tax` | `ClearanceQrInvoice` | `taxid/clearance/` + 15 ASCII digits | SA ZATCA clearance QR, MX CFDI, EG e-invoice |
| `oya-pack-registration-tax` | `VatRegistrationInvoice` | `taxid/registration/` + 15 ASCII digits | UK VAT, AE VAT, GCC VAT alternative |

### §C.2 Why pack drives format

ADR-0064 canonical-base + localization mandates that "every µservice = canonical global base + localization overlay (seam OR adapter OR pack, per-concern)." For cloud-billing, the localization overlay is the **regional pack**, and the canonical base is the format-agnostic invoice aggregate. Binding tax format to the pack rather than to the tenant or the customer means:

- Sovereign deployments (on-prem KR-CSAP, GCC, BR) ship with a fixed pack and a single TaxInvoiceFormat — no runtime tax-format selection that could be misconfigured by the operator.
- Marketplace sellers (revenue_share component) get the seller's home-country pack via cloud-marketplace integration; the seller's `tax_registration_id` is validated against the pack-determined format.
- Cross-border invoicing (e.g. a US tenant invoicing a JP customer) uses the buyer-jurisdiction pack, not the seller-jurisdiction pack — matching the OECD digital-services VAT/GST rule.

### §C.3 Subtotal/tax/total invariant

`Invoice::generate` (lines 469–527 of `oya-cloud-billing-domain/src/lib.rs`) enforces:

```rust
let computed_subtotal = sum_line_items(&line_items)?;
if computed_subtotal != input.subtotal
    || input.subtotal.checked_add(&input.tax)? != input.total
{
    return Err(CloudBillingError::InvalidInvoiceTotal);
}
```

This is the **tax-naive subtotal** rule: cloud-billing's domain crate treats the tax amount as an opaque value computed upstream and verifies only the additive consistency. The cross-currency check inside `Money::checked_add` ensures `subtotal.currency == tax.currency == total.currency`.

### §C.4 Two crate split: domain vs tax-app

The two crates serve distinct roles:

- `oya-cloud-billing-domain` (IP-001): tax-naive aggregate root with format+registration shape validation.
- `oya-cloud-billing-tax-app` (this IP): API-surface adapter that proves runtime alignment with `contracts/openapi/cloud/cloud-billing-invoice-v1.yaml`.

The tax-app crate's `generate_cloud_billing_invoice_from_api` function validates the API request preconditions in a defined order:

1. Request ID present (else 401 missing_request_id).
2. Tenant scope match across header / request / account (else 403 tenant_mismatch).
3. Idempotency key present (else 422 missing_idempotency_key).
4. Shape consistency: id non-empty, account.id non-empty, line items non-empty, currencies match, `subtotal + tax = total` (else 400 invalid_invoice_request).
5. Account state active (else 409 billing_account_not_active).
6. Success: return `CloudBillingInvoiceRecord` with `state = "issued"` and stamped schema_version.

### §C.5 ASC 606 / IFRS 15 revenue recognition tie-in

cloud-billing's invoice is the recognition trigger for the per-line-item revenue. Recognition timing:

- **Point-in-time recognition** for `Usage` / `Reservation` line items: recognized at invoice issuance.
- **Over-time recognition** for `Subscription` / `Commitment` line items: recognized ratably over the billing period (period_start to period_end).
- **Performance-obligation-deferred recognition** for prepaid credits: recognized as consumed.

The recognition timing is not encoded in the domain crate; it is computed by the FOCUS / ERP export surface (IP-015 counterpart parity). The kernel guarantees the invariants that recognition relies on (period bounds well-formed, totals reconstructable, immutable invoice id).

## §D Lifecycle

### §D.1 Pack-driven format derivation at invoice generation

1. Tenant has a `BillingAccount.regional_pack` (set at account creation, immutable).
2. Invoice generator calls `TaxInvoiceFormat::for_regional_pack(&account.regional_pack)`.
3. Result is bound into `InvoiceGenerate.tax_invoice_format`.
4. `Invoice::generate` cross-checks the input format against the derivation — mismatch returns `InvalidTaxInvoiceFormat`.
5. `tax_registration_id` is validated against the format's body rule (lines 314–349 of domain crate).

### §D.2 Cross-border invoice scenario (US tenant → JP customer)

1. cloud-marketplace ascertains the buyer jurisdiction is JP.
2. cloud-billing-tax µservice resolves the buyer-jurisdiction pack: `oya-pack-qualified-tax`.
3. New BillingAccount opened for the buyer with `regional_pack = oya-pack-qualified-tax`.
4. Tax engine computes JCT (Japanese Consumption Tax) per OECD digital-services rule.
5. cloud-billing issues the invoice with `tax_invoice_format = QualifiedTaxInvoice` and the buyer's T-corp number as `tax_registration_id`.

### §D.3 Failure modes

- Pack unknown: `InvalidRegionalPack` (account creation fails).
- Format mismatch: `InvalidTaxInvoiceFormat` (invoice generation fails).
- Registration shape wrong: `InvalidTaxRegistrationId` (invoice generation fails).
- Total reconciliation off: `InvalidInvoiceTotal` (invoice generation fails).
- Cross-currency arithmetic: `InvalidInvoiceTotal` (via `Money::checked_add`).

## §E Cedar Policy Bindings

- `cap.cloud.billing.issue_invoice` — guards invoice issuance per `cloud-billing.cedar`.
- `cap.cloud.billing.settlement.sovereign_invoice` — guards sovereign-pack invoice issuance (on-prem, colo, guest-on-oci) per `settlement-gates.cedar`.

Context attributes used by Cedar:

- `context.regional_pack` — used by sovereign-invoice gate to confirm pack alignment with deployment context.
- `context.sovereign_pack_active` — boolean from tenancy.

## §F Evidence

### §F.1 Source files

- `/Users/jasonlee/oyatie/crates/oya-cloud-billing-domain/src/lib.rs` lines 94–103 (`TaxInvoiceFormat` enum), 314–349 (`TaxRegistrationId::new`), 406–422 (`TaxInvoiceFormat::for_regional_pack`), 469–527 (`Invoice::generate` cross-checks).
- `/Users/jasonlee/oyatie/crates/oya-cloud-billing-tax-app/src/lib.rs` lines 162–253 (`generate_cloud_billing_invoice_from_api`).
- `/Users/jasonlee/oyatie/contracts/openapi/cloud/cloud-billing-invoice-v1.yaml` (544 lines; the runtime contract this tax-app crate mirrors).

### §F.2 Tests demonstrating tax invariants

- `generates_electronic_tax_invoice_with_regional_format_and_exact_totals` (domain crate test): proves KR e-Tax pack derives ElectronicTaxInvoice format and 10-digit registration body works.
- `rejects_invoice_format_tax_registration_total_and_inactive_account` (domain crate test): proves three failure paths — wrong format for pack, wrong registration body shape, off-by-one total.

### §F.3 Integration test surface

- `crates/oya-cloud-billing-tax-app/tests/cloud_billing_invoice_api.rs`: HTTP-level test against the OpenAPI contract — proves status codes (201/400/401/403/409/422) round-trip.

### §F.4 ADR anchors

- ADR-0064 canonical-base + localization: pack drives format.
- ADR-0244: tenant scoping (tax_registration_id is a tenant-scoped FINANCIAL data-class field).
- ADR-0251 compliance packs: each tax format aligns with a compliance pack family.
- ADR-0330 §B.10.6: cloud-billing-tax is an upstream consumer that returns the tax_profile_ref into the kernel.

## §G Counterpart parity

| Counterpart | Their concept | Oyatie equivalent | Delta |
|---|---|---|---|
| Stripe Tax | `automatic_tax: true` on InvoiceItem; rates computed by Stripe Tax | Regional pack drives format; tax amount computed by upstream `cloud-billing-tax` µservice and passed in | Stripe couples computation + invoicing in one product; oyatie separates kernel from tax engine for ADR-0064 canonical-base purity. |
| Stripe Tax | Customer.tax_ids[] (multiple per customer) | `BillingAccount.tax_registration_id` (one per account; multi-jurisdiction tenants use multiple accounts) | Oyatie binds tax id to billing account, not to tenant root — supports B2B reseller patterns. |
| Avalara AvaTax | Per-product tax codes + nexus determination | `tax_profile_ref` opaque string resolved by tax engine | Oyatie wraps Avalara-style determination at the boundary; kernel is engine-agnostic. |
| Vertex | "Tax Decision Engine" with rule sets per jurisdiction | Pack-driven format + downstream tax engine resolution | Same architectural shape — both separate decision from invoicing. |
| TaxJar | Sales tax calculation API (US-centric) | `oya-pack-vat-tax` aliases CountryEInvoice; US sales tax computed by upstream engine | TaxJar is US-only; oyatie's pack model supports global out-of-the-box. |
| Chargebee Tax | `Customer.taxability ∈ {taxable, exempt}` | Pack-determined; exemption is via dedicated tax_profile_ref | Oyatie is stricter — exemption is provenance-tracked, not a boolean. |
| Stripe (revenue recognition) | ASC 606 revenue recognition module bundled with Billing | Recognition timing computed at FOCUS/ERP export (IP-015) | Oyatie separates concerns; tax-app stays minimal. |

## §H Open questions

- Whether to add a `WithholdingTaxInvoice` format for India / Brazil withholding scenarios. Current decision: handled via line-item-level adjustment in tax engine, not a separate format.
- Whether `oya-pack-clearance-tax` should split into per-country variants (SA-ZATCA vs MX-CFDI). Current decision: keep one pack name with country-specific tax_profile_ref bodies; revisit if regulators diverge.
