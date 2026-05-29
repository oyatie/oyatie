---
doc_class: FAQ
microservice: payments
persona: payments-engineer + fintech-platform-engineer
date: 2026-05-20
doc_status: published
---

# Payments Engineer FAQ — payments

## Why multi-PSP routing instead of Stripe-only?

Per ADR-XXX-payments-multi-psp. Three drivers:

1. **Outage resilience**: Stripe had 6 P1-class outages in 2024-2025 (Stripe's own status page). Adyen had 3. Checkout.com had 2. A single-PSP integration means every Stripe outage is a tenant outage. Multi-PSP routing fails over within 90 s.
2. **Regional optimization**: Stripe's interchange optimization is best in US + EU. Adyen wins in NL + AU + ZAR-denominated payments + SEPA Instant. Checkout.com wins in ME + LATAM card networks. Per-region routing improves merchant authorization rate by 0.5-2 % (verified on our 2026-Q1 internal A/B).
3. **Fee arbitrage**: PSP fees vary by card BIN. With multi-PSP, the routing policy chooses the lowest-cost PSP for the specific BIN. Average fee saving 3-7 bps fleet-wide.

The trade-off: tokenization is PSP-specific (a Stripe token cannot be presented to Adyen). We tokenize the card with EVERY active PSP at first-save (network-token mode where available; PSP-specific token otherwise). Storage cost: ~ 1.4× single-PSP.

## Why double-entry ledger instead of a flat transaction table?

Per ADR-XXX-payments-ledger. Three drivers:

1. **Audit traceability**: every cent has a debit-account and a credit-account. PSP fees, taxes, FX margin, chargebacks all post as separate ledger entries with explicit accounts. Reconciliation reduces from "find the missing dollar in 14 join tables" to "find which posting set doesn't sum to zero" — a SQL-1-liner.
2. **GAAP / IFRS compliance**: regulated tenants need GAAP-compliant accounting export. A flat transaction table requires reconstruction; a double-entry ledger is GAAP-shaped natively.
3. **Multi-currency**: each posting is in a single currency. FX conversion is itself a posting (debit account X, credit account Y, with `fx_rate` and `fx_source` attributes). No silent currency rounding.

Common newcomer mistake: trying to update an existing posting on refund. Don't. Refunds emit NEW postings that reverse the original — every posting is immutable. This is core to the audit-chain integration (every posting → audit event; immutable history).

## Why `Decimal128` instead of f64 / Decimal / BigDecimal?

`Decimal128` per ICC IEEE 754-2008 § 5 provides 34 significant decimal digits + a configurable scale per currency (JPY = 0, USD = 2, BHD/KWD/JOD = 3, BTC = 8). f64 cannot represent `0.10 + 0.20` exactly (gives `0.30000000000000004`). f64 in finance code is an immediate audit fail.

Rust crate: `rust_decimal` 1.36 with the `serde-with-arbitrary-precision` feature. We pin to 1.36 because earlier versions had a `Decimal::round_dp` precision bug that affected our fee-rounding tests.

The PostgreSQL column type is `NUMERIC(38, 9)` — 38 digits of precision + 9 decimal places — fits Decimal128 with headroom.

## When does idempotency-key replay kick in?

Always, when:

1. The client provides an `Idempotency-Key` header on POST endpoints (`/charges`, `/refunds`, `/payouts`).
2. The key has been seen within the 24-h replay window AND the request body matches the original.

Behavior:

- **Same key + same body**: returns the cached response from the original request (no new side-effects).
- **Same key + different body**: returns 422 with `idempotency_key_mismatch` error. The client is bugged.
- **Same key + different tenant**: returns 422; idempotency keys are scoped to (tenant, principal, key) tuple.

If a Stripe call is in-flight when a retry comes in (key seen, response not yet computed), the retry waits up to 30 s for the original to complete, then returns the same response. This handles network-retry-storms gracefully.

## What's the per-tenant fee schedule? How is it different from Stripe application fees?

Per ADR-XXX-payments-fee-schedule. A "fee schedule" is a configurable transaction-cost model:

- Card-present: `2.4% + 5¢`
- Card-not-present: `2.9% + 30¢`
- ACH origination: `0.5% + 5¢ (cap $5)`
- Wire send: `$15 flat`
- FX margin: `35 bps` (configurable 0-200 bps)
- Cross-border surcharge: `1.0%`
- Chargeback fee: `$15`

The tenant configures their fee schedule via `oya payments fee-schedule update`. We compute the gross-to-net transaction internally; the merchant sees only the net settlement on their statement.

Stripe application fees let you take a cut from a sub-merchant. Our per-tenant fee schedule is broader — it covers ALL cost components including FX margin, chargeback, and cross-border surcharges, and applies BEFORE the Stripe cut (if is in use).

The ADR enumerates the 14 fee components we model. Most tenants use 4-6; advanced tenants use 12+.

## How does the ISO 20022 integration work? Do we send / receive messages?

Per ADR-XXX-payments-iso20022. Three message-direction patterns:

1. **Outbound `pacs.008` (credit transfer)** — tenant initiates a wire to a beneficiary. We construct the `pacs.008.001.10` XML message, sign with the tenant's correspondent-bank-issued BIC key, and submit via SWIFT FIN (legacy) or SWIFT FINplus (ISO 20022 native). For SEPA Credit Transfer Instant, we submit via the SCT Inst scheme rule book v2026.
2. **Inbound `camt.054` (debit/credit advice)** — the correspondent bank notifies us of an incoming payment. We parse the XML, match the `EndToEndId` against pending invoices, post the ledger entry, emit `payments.payment.received` event.
3. **Inbound `pacs.002` (status report)** — the correspondent bank confirms or rejects an outbound payment. We update the transaction status (`processing` → `settled` or `failed`) and emit the corresponding audit event.

ISO 20022 is verbose (each `pacs.008` is 4-12 KiB of XML). We store the raw XML in SeaweedFS-S3 with a pointer in PostgreSQL; the parsed structured form lives in the ledger. The raw XML is required for audit-trail-reconstruction under SWIFT gpi rules.

The full ISO 20022 schema set (~ 350 message types) is overkill for our purpose; we support 8 message types covering 99% of fintech-grade flows.

## What happens when an FX rate goes stale?

Per the `fx-rate-stale.md` runbook. Definitions:

- Stale: rate > 60 s old. Hard-fail charges that require fresh rate (real-time conversion).
- Very stale: rate > 5 min old. Page on-call.

If the primary FX source (Refinitiv) goes stale:

1. Routing policy fails over to the secondary (OANDA Live). The secondary's rates are typically within ±2 bps of Refinitiv.
2. If both are stale (rare; double-outage), charges requiring real-time conversion return 503 `fx_unavailable`. The tenant can opt to accept the last-cached rate with explicit `accept_stale_rate=true` header (rare; for some B2B flows that prefer settling at a slightly-off rate vs not settling at all).

The stale-rate behavior is configurable per tenant in the fee schedule.

## How do we handle GDPR Art. 17 (right to erasure) for payment history?

Per ADR-XXX-payments-data-retention. Payment history has competing legal requirements:

- GDPR Art. 17: right to erasure on subject request.
- Anti-money-laundering: 5+ years retention required (varies by jurisdiction; EU AMLD5 = 5 y, UK = 5 y, US BSA = 5 y).
- Tax: 6-10 y retention required.
- SEC 17a-4(f): 6 y for broker-dealers (US).

The retention requirement WINS over GDPR Art. 17 per Art. 17(3)(b) ("processing is necessary for compliance with a legal obligation"). We pseudonymize the PII (name, email, phone) on erasure request, but retain the transaction history with a `pii_redacted_at` timestamp and a `pii_redaction_reason=gdpr_art_17_request` attribute. The Cedar policy denies any access that would expose pseudonymized PII; only legal-hold + court-order paths can re-derive the PII from the audit-chain emit history.

The pseudonymization itself emits an audit event (`payments.customer.pii_redacted`).

## What's the chargeback evidence-upload flow? Why does it take 14-21 days?

Per the card network rules:

- **Visa**: merchant has 21 d after notification to upload evidence (per Visa Core Rules Section 11).
- **Mastercard**: 14 d (per Mastercard CIP Manual Chapter 4).
- **Amex**: 20 d (per Amex Merchant Operating Guide).
- **Discover**: 14 d.

The chargeback workflow:

1. Network notifies acquirer (Stripe/Adyen).
2. Acquirer notifies us via webhook (typically within 1-4 h).
3. We open a `payments.chargeback.opened` event + workflow ticket.
4. Merchant has the network-specific window to upload evidence (we surface a countdown in the dashboard).
5. Evidence submitted; we forward to acquirer; acquirer forwards to network.
6. Network decides (45-90 d typical).

Evidence categories per network:

- Transaction record (auth log + capture log).
- Shipping confirmation (for physical goods).
- Customer-acknowledged delivery (e-signature, photo).
- Recurring-subscription proof (for subscription chargebacks).
- Refund proof (if a refund was issued and the chargeback is duplicative).
- Communication log (customer-service tickets).

Win rate at our paid tenant_class: ~ 48 % (industry average is 38-42 %). The delta comes from automated evidence assembly + 3DS2 SCA strong-customer-authentication evidence inclusion.

## What's the difference between this µservice and `finops-portal`?

- `payments`: handles money movement TO/FROM tenants' customers (charges, refunds, payouts to sellers, chargebacks). Acquirer / merchant of record perspective.
- `finops-portal`: handles cloud-cost transparency to the tenant (what does this tenant's compute / storage / network cost?). Internal billing perspective.

A SaaS-billing tenant uses `payments` to charge their customers + `finops-portal` to understand their own oyatie infrastructure cost. The two never share schemas; both emit to audit-chain.

## How does this differ from `cloud-billing` (the cloud-product-billing µservice)?

- `cloud-billing`: the bill that OYATIE sends to ITS tenants (oyatie as merchant, tenant as customer).
- `payments`: the bill that THE TENANT sends to THEIR customers (tenant as merchant, customer as customer).

`cloud-billing` charges go THROUGH `payments` (oyatie's own merchant account in `payments`). So `payments` is the substrate; `cloud-billing` is one specific (high-volume) tenant.
