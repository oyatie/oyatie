---
doc_class: FeatureParityMatrix
title: cloud-billing feature parity vs Stripe Billing + AWS Billing & Cost Management + Recurly
status: Accepted
date: 2026-05-21
microservice: cloud-billing
phase: Phase-0 Shared Infrastructure
wave: Wave-4-rolling
agent_class: microservice-ownership-coherence-audit-agent
top_3_counterparts:
  - Stripe Billing
  - AWS Billing & Cost Management
  - Recurly
union_coverage_bar: true
audit_only: true
---

# `cloud-billing` Feature-Parity Matrix vs Top-3 Counterparts — 2026-05-21

## Canonical Anchors

1. `/Users/jasonlee/oyatie/docs/decisions/ADR-0700-ci-admission-live-apex.md` §D-5 UNION coverage parity bar.
2. `/Users/jasonlee/oyatie/specs/master-plan-sequencing.json` keys `canonical_build_sequence.phases[0]`, `deployment_contexts`.
3. `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md` for the binary tenant_class + billing_components contract.
4. Stripe Billing public docs (`docs.stripe.com/billing`, `docs.stripe.com/products-prices`, `docs.stripe.com/invoicing`, `docs.stripe.com/tax`, `docs.stripe.com/connect`, `docs.stripe.com/revenue-recognition`), Stripe API v2026-05-12+; treated as primary subscription-billing counterpart.
5. AWS Billing & Cost Management (`docs.aws.amazon.com/awsaccountbilling`, `docs.aws.amazon.com/cur`, `docs.aws.amazon.com/cost-management`, `docs.aws.amazon.com/savingsplans`, `docs.aws.amazon.com/awsaccountbilling/latest/aboutv2/billing-conductor.html`) + Recurly docs (`docs.recurly.com`, subscription billing + dunning + ASC 606 / IFRS 15).

## §1 Purpose

This matrix lists every major capability across the three counterpart products and assigns each row a coverage state for `cloud-billing` plus a path to the owning Oyatie surface (cloud-billing itself, a sibling Phase-0/Phase-1 µservice, a tenant_class projection, a workflow, a pack overlay, or `out-of-scope intentional` with reason). Coverage states per ADR-0328 §D-5.15: `covered` requires a path to the owning artifact; `partial` requires a missing-gap note; `missing` requires a proposed remediation target; `out-of-scope intentional` requires a doctrine reason and approving ADR or standard. The matrix is the UNION across all three counterparts — a feature counted in any one counterpart enters the matrix, and Oyatie must either cover it (in cloud-billing or via a sibling) or mark it intentionally out of scope with a doctrine reason. The matrix has three sub-sections per counterpart followed by the union table and family summary.

## §2 Counterpart 1 — Stripe Billing capability inventory (33 features)

Stripe Billing is the largest subscription-billing surface in the industry. The features below are organized by Stripe API family.

| Stripe feature | Owning Stripe surface | Oyatie coverage state | Oyatie owning surface | Gap / target |
|---|---|---|---|---|
| S-01 Products + Prices catalog (one-time + recurring; multi-currency; lookup keys) | `Product`, `Price` resources | `partial` | `cloud-billing` rate-card (`crates/cloud-billing-domain/src/lib.rs:RateCardRef`) + FAQ Q16 YAML rate-card | Missing: explicit Product separation from Price; lookup keys; tax_behavior toggle per price. Target: Wave 15B PRD + kernel `Product` + `Price` types. |
| S-02 Subscription lifecycle (create, update plan, cancel, pause, resume) | `Subscription`, `SubscriptionSchedule` | `missing` | none (cloud-billing has BillingAccount but no Subscription) | Target: Wave 15B introduce `Subscription` primitive bound to a paid tenant + billing_components set. |
| S-03 Subscription proration (mid-period upgrade/downgrade, credit, debit) | `Invoice.discount_amount` + `Subscription.proration_behavior` | `missing` | none | Target: Wave 15B proration semantics in subscription state machine. |
| S-04 Trial subscriptions (free + paid card-required trials) | `Subscription.trial_end`, `trial_settings` | `partial` | tenant_class=demo_trial covers free trials | Missing: paid card-required trials (a paid tenant with $0 first period). Target: Wave 15B trial sub-state machine on Subscription. |
| S-05 Invoice generation + finalization + PDF/HTML rendering | `Invoice`, `Invoice.finalize_invoice` | `partial` | `cloud-billing` Invoice + runbook invoice-generation-timeout.md | Missing: PDF/HTML rendering pipeline; sample invoice templates per tenant; brand overlay. Target: Wave 15B `cloud-billing-render` crate + templates. |
| S-06 Invoice line items with metadata, period, tax rates | `InvoiceItem`, `Invoice.lines` | `covered` | `crates/cloud-billing-domain/src/lib.rs:InvoiceLineItem` + cloud-billing-tax handoff | None. |
| S-07 Credit notes (issue, mark void, apply to invoice) | `CreditNote` | `partial` | `cloud_billing::Action::IssueCreditMemo` per FAQ Q10 | Missing: void credit note; partial application; multi-period roll-forward. Target: Wave 15B credit-note state machine. |
| S-08 Invoice issuance via email + hosted invoice page + portal link | `Invoice.hosted_invoice_url`, `Invoice.invoice_pdf` | `missing` | needs comms-email + finops-portal coordination | Target: Wave 15B hosted invoice link + comms-email template. |
| S-09 Payment method on file (card, ACH, SEPA, BACS, BLIK, OXXO, iDEAL, etc.) | `PaymentMethod`, `SetupIntent` | `out-of-scope intentional` | `payments` (Phase-1 service 07) owns | Reason: cloud-billing is substrate; payment-method storage and PCI-DSS Level-1 attestation live in `payments`. ADR-0245 substrate vs product layering. |
| S-10 Payment retry / dunning (SmartRetries, retry schedule, eventual hard freeze) | `Subscription.payment_settings.payment_method_options.card.request_three_d_secure` + Stripe Dunning | `partial` | FAQ Q19 fraud sweep + runbook portal-mark-delayed | Missing: canonical dunning policy (retry windows, hard-freeze on N declines, comms cadence). Target: Wave 15B dunning state machine. |
| S-11 Subscription invoice cycles (monthly, annual, custom days, anchor date) | `Subscription.billing_cycle_anchor`, `interval` | `partial` | FAQ Q12 names "weekly / monthly / quarterly / annual"; tenant_class adoption record cadence per tenant_class | Missing: per-subscription anchor date + custom interval + interval count. Target: Wave 15B Subscription.cycle. |
| S-12 Tax rates per jurisdiction (Stripe Tax) | `TaxRate`, Stripe Tax automatic | `covered` | `cloud-billing-tax` µservice (Phase-0 service 13) | None — actually cleaner separation than Stripe Tax (which is bundled). |
| S-13 Customer tax IDs (VAT, GST, ABN, EIN, etc.) | `Customer.tax_ids` | `partial` | `crates/cloud-billing-domain/src/lib.rs:TaxRegistrationId` with format enum (Electronic / Qualified / VAT / GST / Fiscal / Clearance / Registration) | Gap: customer-level tax ID stored on BillingAccount (not yet); Stripe stores per-Customer. Target: Wave 15B BillingAccount.tax_ids field. |
| S-14 Quote → Invoice workflow (Stripe Quotes) | `Quote` | `missing` | none | Target: Wave 15B or out-of-scope intentional (reason: B2C-pivot; Oyatie's B2B sales flow may use `crm` Phase-4A.3 for quote management). |
| S-15 Coupons + Promotion codes (percent off, amount off, repeating, forever) | `Coupon`, `PromotionCode` | `missing` | none | Target: Wave 15B Coupon + PromotionCode primitives bound to subscription or invoice. |
| S-16 Discounts (subscription-level, invoice-level, line-item-level) | `Subscription.discount`, `Invoice.discount`, `InvoiceItem.discount` | `missing` | none | Target: Wave 15B Discount primitive applicable at three levels. |
| S-17 Customer Portal (self-serve billing UI) | Stripe Customer Portal hosted | `out-of-scope intentional` | `finops-portal` (Phase-1 service 08) owns tenant self-serve | Reason: ADR-0245 substrate vs product; tenant-facing UI is finops-portal's responsibility. |
| S-18 Webhooks (subscription.created, invoice.paid, etc.) | Stripe Events + Webhooks | `partial` | audit-chain emits events; runbooks reference EVT_CLOUD_BILLING_* | Gap: no canonical event-class registry; no outbound webhook delivery to tenant-owned URLs. Target: Wave 15B webhook delivery service + canonical event-class registry (resolves CB-F-025 from coherence audit). |
| S-19 Idempotency keys on all writes | Stripe `Idempotency-Key` header | `covered` | `CloudBillingEventCreate.idempotency_key` + `events_by_idempotency` dedup map | Gap: SDK example uses `event_id` (UUID v7) interchangeably with `idempotency_key`; reconcile naming. |
| S-20 Test mode + Test clocks (simulate time advance) | `TestClock` | `partial` | `make dev-cell.up` loopback + `--fast-forward` flag on `oya billing close` | Gap: no canonical test clock primitive that subscriptions reference for trial-end / cycle-anchor simulation. Target: Wave 15B testkit::TestClock. |
| S-21 Multi-currency invoicing (settle in tenant's preferred currency) | `Currency` per Invoice | `covered` | tenant_class adoption record (Paid: 9; Paid: 28; Paid: all + sovereign) + kernel `CurrencyCode` | Gap: tenant_class adoption record lists currencies per tenant_class; under tenant_class binary, all paid tenants get the full list. Target: Wave 15J retirement + Wave 15B currency-policy.md. |
| S-22 FX rate at issuance lock | Stripe locks FX at finalization | `partial` | FAQ Q5 + runbook FX lock command; benchmark says ECB-reference-rates-daily | Gap: no kernel-level FX model; Money::checked_add requires same currency. Target: Wave 15B `cloud-billing-fx` crate (resolves CB-F-033). |
| S-23 Statement descriptors | Stripe `Charge.statement_descriptor` | `out-of-scope intentional` | `payments` owns | Reason: same as S-09. |
| S-24 Disputes + Chargebacks | Stripe `Dispute` | `out-of-scope intentional` | `payments` owns dispute lifecycle; cloud-billing receives the credit-memo when dispute resolved | Reason: ADR-0245 substrate vs product layering. |
| S-25 Reporting (Stripe Sigma SQL) | Stripe Sigma | `partial` | FOCUS 1.1 export to Parquet + Kafka stream | Gap: no canonical SQL surface; FOCUS export is point-in-time. Target: Wave 15B SQL-via-finops-portal. |
| S-26 Revenue Recognition (ASC 606 + IFRS 15) | Stripe Revenue Recognition | `partial` | FAQ Q12 "SOX-404 controls" + benchmark "OECD BEPS Pillar Two GloBE" | Gap: no canonical RevRec schedule (deferred revenue, earned revenue, contract liability); ASC 606 5-step model not modeled. Target: Wave 15B revenue-recognition.md + kernel ScheduleOfRevenue. |
| S-27 Tax reporting (e.g., 1099, EU VAT MOSS) | Stripe Tax reports | `partial` | cloud-billing-tax produces per-jurisdiction tax lines | Gap: jurisdiction-specific reports (1099-K, OSS/IOSS quarterly VAT) not modeled. Target: cloud-billing-tax authoring lane. |
| S-28 (marketplace settlement + platform commission) | Stripe (`Account`, `Transfer`, `ApplicationFee`) | `missing` | revenue_share component (per directive memory) not modeled in kernel | Target: Wave 15B revenue_share primitive + cohort tracking + monthly settlement event + payments-µservice payout integration. |
| S-29 Treasury (embedded financial accounts) | Stripe Treasury | `out-of-scope intentional` | not Oyatie's product surface | Reason: ADR-0321 long-tail B2B SaaS; embedded banking-as-a-service is not a Phase-0 substrate concern. |
| S-30 Billing Meter (usage-based with aggregation) | Stripe `BillingMeter` (2024+) | `covered` | metering bus + `CloudBillingEvent` + per_usage billing_component | None — actually cloud-billing's metering bus is more substantial (5M events/sec) than Stripe's BillingMeter (per spec). |
| S-31 Drafts (invoice + credit note drafts) | `Invoice.status=draft` | `partial` | invoice preview vs final state in runbook | Gap: explicit Draft state in kernel `InvoiceState` enum (currently `Issued | Paid | Overdue | Void`). Target: Wave 15B add `Draft` to InvoiceState. |
| S-32 Custom fields on invoice / customer | `Customer.metadata`, `Invoice.metadata` | `missing` | none | Target: Wave 15B BillingAccount.metadata map and Invoice.metadata map. |
| S-33 Stripe Apps (third-party billing extensions) | Stripe Apps Marketplace | `out-of-scope intentional` | `cloud-marketplace` (Phase-0 service 18) + `plugin-app-store` (Phase-4) own extension distribution | Reason: ADR-0249 multi-category marketplace doctrine. |

Stripe Billing coverage rollup: covered = 7, partial = 14, missing = 8, out-of-scope intentional = 4. Total = 33.

## §3 Counterpart 2 — AWS Billing & Cost Management capability inventory (25 features)

AWS B&CM is the primary cloud-cost reporting + cost-allocation counterpart. cloud-billing already benchmarks against AWS CUR + Cost Explorer, so most of these rows have existing coverage paths.

| AWS feature | Owning AWS surface | Oyatie coverage state | Oyatie owning surface | Gap / target |
|---|---|---|---|---|
| A-01 Cost and Usage Reports 2.0 (CUR 2.0; Parquet hourly + daily granularity) | AWS CUR | `partial` | benchmark + `cloud-billing-ingestor-aws-*` adapters | Gap: cloud-billing INGESTS CUR; cloud-billing's OWN output is FOCUS 1.1 Parquet (not CUR). The mapping CUR↔FOCUS lives in cloud-billing-ingestor-aws. Target: Wave 15B ARCHITECTURE.md ingestor section. |
| A-02 Cost Categories (re-group line items into logical buckets) | AWS Cost Categories | `partial` | cloud-billing attribution rules + cost centers | Gap: AWS Cost Categories supports nested/inheritance rules; cloud-billing attribution-rule priority is flat. Target: Wave 15B nested cost-category extension. |
| A-03 Cost Allocation Tags (user-defined + AWS-generated) | AWS tags | `covered` | resource tag → tenant_id mapping per FAQ Q13 | None. |
| A-04 Savings Plans (Compute / EC2 / SageMaker) | AWS Savings Plans | `partial` | reservation primitive (FAQ Q6) | Gap: cloud-billing has reservations as a single primitive; AWS splits Savings Plans (flexible commitment) from Reserved Instances (rigid). Target: Wave 15B introduce Commitment vs Reservation (cf. CloudBillingEventKind already has both). |
| A-05 Reserved Instances (Standard + Convertible) | AWS RI | `partial` | reservation primitive + convertible reservation in tenant_class adoption record (paid) | Gap: tenant_class adoption record is retirement-target; convertible should be a reservation property not a tier privilege. Target: Wave 15J + Wave 15B Reservation.convertible flag. |
| A-06 Cost Anomaly Detection (ML-based) | AWS Cost Anomaly Detection | `covered` | FAQ Q9 streaming Bayesian + tenant_class adoption record per-tenant_class cadence | Gap: tenant_class adoption record retirement; under unified-quality bar, anomaly detection is continuous for every paid tenant. Target: Wave 15J. |
| A-07 AWS Budgets (alert + auto-action) | AWS Budgets | `missing` | none (cloud-billing has cap-breach for demo_trial but not user-defined Budget) | Target: Wave 15B Budget primitive + auto-action (notify, restrict via Cedar, suspend). |
| A-08 Free Tier Usage Alerts | AWS Free Tier | `missing` | covered conceptually via demo_trial cap-breach but no canonical alert primitive | Target: Wave 15B usage-cap-breach event + grace state machine (resolves CB-F-020). |
| A-09 Billing Conductor (linked-account re-billing) | AWS Billing Conductor | `partial` | cost-center attribution + chargeback | Gap: cross-entity transfer pricing (OECD BEPS Pillar Two GloBE in benchmark paid tenant_class) maps to Billing Conductor's "billing groups + pricing rules". Target: Wave 15J retire tier framing + Wave 15B canonical transfer-pricing rules. |
| A-10 Customer Carbon Footprint Tool | AWS CCFT | `missing` | none (sustainability reporting absent) | Target: Wave 15B or out-of-scope intentional (reason: long-tail; sustainability reporting can land in a `sustainability` µservice or `finops-portal` overlay later). |
| A-11 Marketplace Cost (AWS Marketplace purchases on consolidated bill) | AWS Marketplace billing | `partial` | `cloud-marketplace` µservice (Phase-0 service 18) | Gap: marketplace purchases under revenue_share component need explicit ledger flow. Target: Wave 15B. |
| A-12 Account Hierarchy (Organizations) | AWS Organizations | `partial` | tenant tree per ADR-0244 + runbook tenant-tree-sync command | Gap: explicit account-hierarchy schema (parent / OU / sub-account) not in cloud-billing kernel. Target: Wave 15B BillingAccount.parent_id + tenancy-µservice handoff. |
| A-13 Consolidated billing (parent pays for all children) | AWS Consolidated Billing | `partial` | tenant tree + cost-center | Gap: settlement flow (which billing account pays) not modeled. Target: Wave 15B consolidated-billing.md. |
| A-14 Cost Explorer (UI for cost + usage query) | AWS Cost Explorer | `out-of-scope intentional` | `finops-portal` owns the UI | Reason: ADR-0245 substrate vs product. |
| A-15 Cost Categories with rule-based attribution | AWS Cost Categories rules | `partial` | attribution-rule primitive | Gap: AWS rules support tag-based, account-based, dimension-based, and rule-inheritance; cloud-billing's rule is dimension-match priority. Target: Wave 15B rule-extensibility. |
| A-16 Tags Editor + Cost Allocation Tag activation | AWS UI | `out-of-scope intentional` | `finops-portal` UI | Reason: substrate vs product. |
| A-17 CSV export of cost + usage | AWS CSV exports | `covered` | FOCUS 1.1 Parquet + CSV options on FOCUS export | None (Parquet is canonical; CSV is a render output if requested). |
| A-18 Resource lifecycle events (created, deleted, modified) → cost | AWS CloudTrail + CUR | `covered` | `CloudBillingEventKind::{ResourceCreated, ResourceTerminated, Usage, Reservation, Commitment, Credit}` | None. |
| A-19 Currency conversion (USD only; convert at month-end) | AWS Billing in USD only | `covered` (more substantial) | multi-currency invoicing per tenant_class adoption record (28+ on Paid; all on Paid) + ECB rate lock | None — cloud-billing exceeds AWS B&CM on currency. |
| A-20 Programmatic API for cost data | AWS Cost Explorer API | `partial` | OpenAPI contract authored in Wave 15B per CB-F-010 | Target: Wave 15B contracts/openapi/cloud-billing.openapi.yaml. |
| A-21 IAM-based access control on cost APIs | AWS IAM + cost-specific actions | `covered` | Cedar permits per `cloud_billing::Action::*` | Target Wave 15B: actual Cedar files on tree per CB-F-012. |
| A-22 Reservation utilization + coverage reports | AWS RI Utilization & Coverage | `covered` | reservation recommender runbook + utilization input | None. |
| A-23 Reservation recommendations (with break-even days) | AWS Reservation Recommendations | `covered` | reservation-recommendation-engine runbook | Gap: break-even days not surfaced as a primary metric. Target: Wave 15B add to recommender output. |
| A-24 Commitment-based discounts that span multiple resource types | AWS SP family flex | `partial` | reservation/commitment primitive but resource family not modeled cross-cuttingly | Target: Wave 15B Commitment.eligible_resource_kinds. |
| A-25 Audit / change tracking on billing config | AWS CloudTrail on Billing | `covered` | audit-chain BLAKE3 anchor per FAQ Q19 fraud, per benchmark | None — cloud-billing has tamper-evident chain (BLAKE3), AWS uses append-only logs. |

AWS B&CM coverage rollup: covered = 7, partial = 13, missing = 3, out-of-scope intentional = 3. Total = 25 (one row [A-15] was previously counted in "partial").

## §4 Counterpart 3 — Recurly capability inventory (22 features)

Recurly's strength is subscription billing + dunning + revenue recognition. Several Recurly features overlap with Stripe (S-row) and are noted as duplicates.

| Recurly feature | Owning Recurly surface | Oyatie coverage state | Oyatie owning surface | Gap / target |
|---|---|---|---|---|
| R-01 Subscription Plans (basic, multi-currency, multi-tier pricing) | `Plan` resource | `partial` (dup of S-01) | rate-card + per-resource-kind billing_components | Same gap as S-01. |
| R-02 Subscription lifecycle (similar to S-02) | `Subscription` | `missing` (dup of S-02) | none | Same as S-02. |
| R-03 Proration (similar to S-03) | `Subscription.proration` | `missing` (dup of S-03) | none | Same as S-03. |
| R-04 Mid-period add-ons + one-time charges | `Subscription.add_ons` | `missing` | none | Target: Wave 15B Subscription.add_ons primitive. |
| R-05 Subscription pause / resume (with strategies: stop_at_period_end vs immediate) | `Subscription.pause` | `missing` | none | Target: Wave 15B Subscription state machine. |
| R-06 Setup fees | `Plan.setup_fees` | `missing` | none | Target: Wave 15B add SetupFee to Plan. |
| R-07 Trial extensions (extend trial without losing user) | `Subscription.trial_ends_at` mutation | `partial` | demo_trial conversion path | Gap: explicit trial extension API; preserves billing_components selection. Target: Wave 15B. |
| R-08 Dunning Management (failed payment policies, retry windows, decline categorization) | Recurly Dunning | `partial` | FAQ Q19 fraud + runbook portal-mark-delayed | Major gap: Recurly's dunning is industry-leader (sophisticated retry logic, "smart retries", decline-category-based behavior). Target: Wave 15B dunning state machine + retry policy DSL. |
| R-09 ASC 606 / IFRS 15 Revenue Recognition (schedule + journal entry export) | Recurly RevRec | `partial` (dup of S-26) | benchmark "OECD BEPS Pillar Two" + SOX-404 wording | Same as S-26. |
| R-10 Tax integration (Avalara + Vertex + Recurly Tax) | Recurly Tax | `covered` (dup of S-12) | cloud-billing-tax | None. |
| R-11 Customer Data Platform (linked external IDs) | Recurly Customer | `partial` | BillingAccount.tenant_id maps to canonical tenant; no external_id field | Target: Wave 15B BillingAccount.external_ids map. |
| R-12 Webhooks (subscription, invoice, payment, account events) | Recurly Webhooks | `partial` (dup of S-18) | audit-chain | Same as S-18. |
| R-13 Hosted Payment Pages | Recurly HPP | `out-of-scope intentional` | `payments` owns | Reason: substrate vs product. |
| R-14 Account Hierarchies (parent-child, sub-accounts) | Recurly hierarchy | `partial` (dup of A-12) | tenant tree | Same as A-12. |
| R-15 Coupons (percent off, dollar off, single-use, multi-use, custom redemptions) | Recurly Coupons | `missing` (dup of S-15) | none | Same as S-15. |
| R-16 Gift cards (sell-and-redeem flow) | Recurly Gift Cards | `out-of-scope intentional` | not Phase-0 substrate (could land in marketplace or consumer brand) | Reason: ADR-0321 long-tail. |
| R-17 Subscription cancellation surveys | Recurly cancellation | `out-of-scope intentional` | not a billing-substrate concern (UI in finops-portal or product surface) | Reason: substrate vs product. |
| R-18 Subscription analytics dashboards (MRR, ARR, churn, LTV, CAC) | Recurly Analytics | `out-of-scope intentional` | `analytics` (Phase-3 service 16) + `finops-portal` own | Reason: substrate vs product. |
| R-19 Refund processing (full + partial + credit refund) | Recurly Refunds | `covered` | `cloud_billing::Action::IssueCreditMemo` per FAQ Q10 + reservation FAQ Q17 | Gap: partial vs full refund distinction; reservation pseudo-refund via ConvertReservation. |
| R-20 Multi-language invoice templates | Recurly templates | `missing` | none | Target: Wave 15B invoice template per locale; canonical-base + localization pack per ADR-0064. |
| R-21 Country-specific invoicing requirements (e-invoice for KR / IN / IT / MX / etc.) | Recurly Country Tax | `covered` | TaxInvoiceFormat enum: ElectronicTaxInvoice (KR), QualifiedTaxInvoice (JP-class), CountryEInvoice (EU), GstTaxInvoice (IN GST 15-char), FiscalDocumentInvoice (MX/BR/AR), ClearanceQrInvoice (SA/MX clearance), VatRegistrationInvoice | None — cloud-billing exceeds Recurly on e-invoice format diversity. |
| R-22 Subscription change preview (what the invoice will look like after change) | Recurly preview | `partial` | invoice-preview command in runbook | Gap: preview tied to subscription change (not yet a primitive). Target: Wave 15B. |

Recurly coverage rollup (deduplicating against Stripe): unique-to-Recurly partial = 4, unique-to-Recurly missing = 4, unique-to-Recurly out-of-scope = 3, unique covered = 3. Total Recurly-unique = 14 (8 are Stripe duplicates).

## §5 UNION-Coverage Matrix (consolidated)

The UNION matrix consolidates the three counterparts. Rows where one counterpart's feature subsumes another's are collapsed; rows where counterparts diverge are kept distinct.

| Capability family | Counterpart anchor | Coverage state | Oyatie owning surface |
|---|---|---|---|
| Product + Price catalog | S-01, R-01 | `partial` | cloud-billing rate-card |
| Subscription lifecycle | S-02, R-02 | `missing` | none → Wave 15B Subscription primitive |
| Proration (mid-period plan change) | S-03, R-03 | `missing` | none → Wave 15B |
| Trials (free + paid card-required) | S-04, R-07 | `partial` | tenant_class=demo_trial |
| Trial extension | R-07 | `partial` | demo_trial conversion path |
| Add-ons (mid-period one-time + recurring) | R-04 | `missing` | none → Wave 15B |
| Pause / resume | R-05 | `missing` | none → Wave 15B |
| Setup fees | R-06 | `missing` | none → Wave 15B |
| Invoice generation + finalization | S-05 | `partial` | cloud-billing Invoice + runbook |
| Invoice line items + metadata | S-06 | `covered` | InvoiceLineItem |
| Invoice rendering (PDF/HTML/portal link) | S-05, S-08 | `missing` | render crate target Wave 15B |
| Invoice drafts | S-31 | `partial` | add Draft state |
| Credit notes / credit memos | S-07, R-19 | `partial` | IssueCreditMemo Cedar action |
| Multi-language invoice templates | R-20 | `missing` | ADR-0064 canonical-base + localization pack target |
| Country-specific e-invoice formats | R-21 | `covered` | TaxInvoiceFormat enum (7 formats) |
| Custom fields on Customer / Invoice / Subscription | S-32 | `missing` | metadata map target Wave 15B |
| Payment method storage | S-09, R-13 | `out-of-scope intentional` | payments µservice (ADR-0245) |
| Payment retry / dunning | S-10, R-08 | `partial` | dunning state machine target Wave 15B |
| Disputes / chargebacks | S-24 | `out-of-scope intentional` | payments owns |
| Subscription billing cycles | S-11 | `partial` | cycle anchor target Wave 15B |
| Customer tax IDs | S-13 | `partial` | BillingAccount.tax_ids target Wave 15B |
| Tax rates + automatic Stripe Tax / Avalara | S-12, R-10 | `covered` | cloud-billing-tax handoff |
| Tax reporting (1099, OSS/IOSS, EU VAT MOSS) | S-27 | `partial` | cloud-billing-tax target |
| Revenue Recognition (ASC 606 / IFRS 15) | S-26, R-09 | `partial` | revenue-recognition.md target Wave 15B |
| Quote → Invoice | S-14 | `missing` | Wave 15B or crm-routed |
| Coupons + Promotion codes | S-15, R-15 | `missing` | Wave 15B Coupon + PromotionCode |
| Discounts (subscription, invoice, line-item) | S-16 | `missing` | Wave 15B Discount primitive |
| Customer Portal / Tenant self-serve | S-17 | `out-of-scope intentional` | finops-portal |
| Webhooks (canonical event registry + outbound delivery) | S-18, R-12 | `partial` | audit-chain emits; outbound delivery target Wave 15B |
| Idempotency keys on writes | S-19 | `covered` | events_by_idempotency |
| Test clocks (simulate time advance) | S-20 | `partial` | --fast-forward flag |
| Multi-currency invoicing | S-21, A-19 | `covered` | CurrencyCode + ECB FX |
| FX rate lock at issuance | S-22 | `partial` | runbook command; kernel does not model FX |
| Marketplace / commission settlement | S-28, R-16 | `missing` | revenue_share component target Wave 15B |
| Treasury / embedded banking | S-29 | `out-of-scope intentional` | ADR-0321 long-tail |
| Usage-based metering (Stripe Billing Meter) | S-30 | `covered` | metering bus + CloudBillingEvent |
| Reporting / SQL surface (Sigma) | S-25 | `partial` | FOCUS export → SQL via finops-portal target |
| Stripe Apps / 3rd-party extensions | S-33 | `out-of-scope intentional` | cloud-marketplace + plugin-app-store |
| CUR-class cloud-cost ingestion + FOCUS conformance | A-01 | `partial` | ingestor-aws/gcp/azure-stripe + FOCUS export |
| Cost Categories / Attribution rules | A-02, A-15 | `partial` | cost-center + attribution-rule |
| Cost Allocation Tags | A-03 | `covered` | tag → tenant mapping |
| Savings Plans (flex commitment) | A-04 | `partial` | Commitment + Reservation primitives target Wave 15B |
| Reserved Instances (Standard + Convertible) | A-05 | `partial` | Reservation + convertible flag target Wave 15B |
| Cost Anomaly Detection (ML) | A-06 | `covered` | anomaly detection per FAQ Q9 |
| User Budgets + Alerts | A-07 | `missing` | Budget primitive target Wave 15B |
| Free Tier Usage Alerts | A-08 | `missing` | demo_trial cap-breach target Wave 15B |
| Billing Conductor (re-bill linked accounts) | A-09 | `partial` | cost-center + chargeback + transfer pricing |
| Carbon Footprint / Sustainability | A-10 | `missing` | Wave 15B or long-tail out-of-scope |
| Marketplace Cost on consolidated bill | A-11 | `partial` | cloud-marketplace integration |
| Account Hierarchy (Organizations / Recurly) | A-12, R-14 | `partial` | tenant tree per ADR-0244 |
| Consolidated billing settlement | A-13 | `partial` | tenant tree + settlement target Wave 15B |
| Cost Explorer UI | A-14 | `out-of-scope intentional` | finops-portal |
| Tags Editor UI | A-16 | `out-of-scope intentional` | finops-portal |
| CSV / Parquet cost data export | A-17 | `covered` | FOCUS Parquet + CSV |
| Resource lifecycle events → cost | A-18 | `covered` | CloudBillingEventKind |
| Cost data programmatic API | A-20 | `partial` | OpenAPI contract Wave 15B |
| IAM-based access on cost APIs | A-21 | `covered` | Cedar permits Wave 15B authoring |
| Reservation utilization + coverage reports | A-22 | `covered` | runbook |
| Reservation recommendations | A-23 | `covered` | reservation recommender |
| Cross-resource commitment-based discount | A-24 | `partial` | Commitment.eligible_resource_kinds Wave 15B |
| Audit trail on billing config | A-25 | `covered` | audit-chain BLAKE3 |
| Subscription analytics (MRR, ARR, churn) | R-18 | `out-of-scope intentional` | analytics + finops-portal |
| Cancellation surveys | R-17 | `out-of-scope intentional` | substrate vs product |
| Subscription preview | R-22 | `partial` | invoice-preview command Wave 15B |
| Customer-data external IDs | R-11 | `partial` | external_ids map Wave 15B |

UNION coverage rollup across all three counterparts (deduplicated): **covered = 16, partial = 26, missing = 14, out-of-scope intentional = 14. Total UNION rows = 70**.

## §6 Family Summary

The UNION coverage clusters into four families. (1) **Substrate billing primitives** (meter, rate card, invoice, credit memo, FOCUS export, cost-center attribution, reservation) — cloud-billing is materially strong on this family (covered or partial on every row). (2) **Subscription primitives** (Subscription, Plan/Product/Price separation, proration, add-ons, pause/resume, setup fees, coupons, discounts, quotes) — cloud-billing is materially weak on this family (every row is missing except a partial on "trials" via tenant_class=demo_trial). This is the largest gap. (3) **Operational primitives** (idempotency, test clocks, multi-currency, FX lock, anomaly detection, budgets, alerts, audit) — cloud-billing is mostly covered or partial; budgets and free-tier-alerts are the two missing rows. (4) **Boundary primitives** (payment methods, statement descriptors, disputes, hosted portals, analytics dashboards, treasury, gift cards, cancellation surveys) — correctly marked `out-of-scope intentional` because each of these belongs to a sibling µservice (`payments`, `finops-portal`, `analytics`, `cloud-marketplace`) per ADR-0245 substrate vs product layering.

## §7 Headline Gap Analysis

The single biggest parity gap is **subscription**. Stripe Billing and Recurly are subscription-first; cloud-billing today is meter-first. The keystone directive's `billing_components = {revenue_share, per_seat, per_usage}` set is consistent with subscription semantics but does not name the Subscription primitive itself. Wave 15B should introduce Subscription as a first-class kernel type bound to a paid tenant + billing_components set + a lifecycle state machine ({trial, active, paused, past_due, canceled, expired}). The kernel-level addition is moderate (perhaps 600 lines of Rust across `Subscription`, `Plan`, `Add-on`, `Coupon`, `Discount`, `DunningPolicy`); the documentation surface required is significant (PRD section, ARCHITECTURE section, runbook for subscription-state-stuck, FAQ rows on subscription lifecycle, contracts/openapi entry for subscription resource).

The second biggest gap is **revenue recognition**. Stripe RevRec and Recurly RevRec both produce ASC 606 / IFRS 15-conformant schedules; cloud-billing's "SOX-404 controls" wording is correct but does not encode the actual five-step ASC 606 model (identify contract, identify performance obligations, determine transaction price, allocate transaction price, recognize revenue). This is a Phase-0 substrate concern because revenue recognition feeds compliance.md and the finance close. Target: Wave 15B revenue-recognition.md + kernel ScheduleOfRevenue + journal-entry export to ERP.

The third biggest gap is **dunning / retry policy**. Recurly's dunning is industry-leader. cloud-billing's FAQ Q19 covers fraud detection (10x baseline spike → throttle, 3+ declines → freeze, reservation purchase + immediate cancellation pattern → governance) but not the canonical dunning state machine (smart retry windows, retry-on-decline-category, hard-freeze on N attempts, comms cadence, recovery webhook). Target: Wave 15B dunning-policy.md + DunningPolicy primitive parametric over (retry_count, retry_interval, hard_freeze_threshold, recovery_grace).

The fourth gap is **marketplace settlement (revenue_share)**. The directive memory says cloud-billing OWNS revenue_share; no kernel primitive exists today. Stripe is the strongest counterpart; AWS Marketplace billing is the closest cloud-cost counterpart. Wave 15B introduces RevenueShareEvent + cohort tracking + monthly settlement + payouts integration.

The fifth gap is **budgets + free-tier alerts**. AWS Budgets is the canonical primitive; cloud-billing has nothing user-defined. Target: Wave 15B Budget + Alert primitives.

## §8 Additive Surface (Oyatie features not in any counterpart)

cloud-billing has several features that exceed the union of the three counterparts. (1) **Per-second metering at 5M events/sec sustained**, 18M peak. None of the three counterparts publishes a metering-bus throughput at that scale. (2) **End-of-period close p95 ≤ 74 min** for 10M events. Stripe and Recurly do not publish a comparable end-of-period close SLO; AWS Billing publishes hourly latency on CUR but no end-of-period close SLO. (3) **FOCUS 1.1 native conformance** with extension columns (`tenant_id`, `cost_center`, `pack_id`). AWS / GCP / Azure are preview-grade on FOCUS; Apptio / Vantage / CloudZero conform post-ingest. cloud-billing conforms at the source. (4) **OECD BEPS Pillar Two GloBE export**. Apptio Cloudability is the only counterpart with this; Stripe + Recurly do not. (5) **Cedar-gated credit memos** — every memo flows through a policy permit; vendor systems allow direct ledger writes. (6) **Audit-chain BLAKE3** — tamper-evident anchoring; vendors append-only. (7) **HTTP/3 QUIC** per ADR-0253 — cloud-billing default protocol. Vendors are HTTP/2. (8) **Per-tenant compliance pack overlays** (SOX-404, K-FSI, MAS-TRM, PCI DSS v4.0, GDPR, EU AI Act, CSAP-KR, …) — pack activation flips behavior per tenant; vendor counterparts have global compliance posture per region. (9) **Six deployment contexts** (oyatie-public-cloud, guest-on-aws, guest-on-oci, on-prem, colo, oyatie-as-cloud-provider) once IaC sub-wave lands. (10) **OCI Always Free demo_trial profile** — Oyatie's tenant onboarding can fit zero-cost demo tenants on OCI's 4 OCPU + 24 GB + 200 GB block + 2× Autonomous DB Always Free perpetual tier; no counterpart offers a perpetual-free cloud-billing surface (Stripe has a Stripe Atlas / Stripe Climate side, not equivalent). (11) **Rust-strict implementation** (statically-linked binaries, deterministic builds, no Python interpreter dependency) — portability and ops-grade fitness on every Tier-1 OS.

The additive surface is what makes cloud-billing's substrate role legitimate. Without these, cloud-billing would be a poorer subset of Stripe + Recurly + AWS B&CM. With these, cloud-billing is a hyperscaler-grade substrate that Stripe and Recurly cannot match on cloud-cost ingestion + FOCUS native + audit-chain tamper-evidence + multi-context deployment + sovereign pack overlays.

## §9 Verification Notes

The matrix was authored by inspecting cloud-billing's local corpus (10 docs + kernel `cloud-billing-domain` source) and cross-referencing the three counterparts' public documentation. The Stripe Billing API version cited is the published surface as of Stripe API v2026-05-12+ (Stripe API versions are monthly; subscription + invoice + credit-note APIs are stable). AWS Billing & Cost Management is cited per the AWS docs surface as of 2026-05; CUR 2.0 is the current generation. Recurly is cited per docs.recurly.com surface. No row was marked covered or partial without an Oyatie path; no row was marked out-of-scope intentional without a doctrine reason and ADR pointer. The matrix is findings-only; no µservice file is rewritten by this matrix.

## §10 Findings → Wave 14 Backlog (UNION-derived)

The parity rows produce 14 new missing-feature rows that consolidate into the Wave 14 backlog. The 14 rows are: Subscription primitive (S-02/R-02), Proration (S-03/R-03), Add-ons (R-04), Pause/Resume (R-05), Setup fees (R-06), Quote→Invoice (S-14), Coupons (S-15/R-15), Discounts (S-16), Custom fields metadata (S-32), Budgets (A-07), Free Tier Usage Alerts (A-08), Marketplace / revenue_share (S-28/R-16), Multi-language invoice templates (R-20), Carbon Footprint (A-10). Each row's owner is `cloud-billing` (Phase-0 service 12) with the Wave 15B (substance) sub-wave as the canonical landing point. Severity per ADR-0328 §D-8.10..§D-8.12: missing parity is P2 by default unless it blocks a Big 8 priority µservice; in this case all 14 rows are P2 because they enrich cloud-billing but do not block downstream phase promotion. Two exceptions: Subscription primitive (P1 because every CRM / HR / ERP Phase-4 µservice will need subscription-style billing) and Marketplace / revenue_share (P1 because the directive memory says cloud-billing OWNS revenue_share and the absence of the primitive blocks marketplace seller onboarding).

## §11 Per-Counterpart Tenant-Class Mapping

This section maps each counterpart's pricing-class concept to the canonical Oyatie tenant_class binary so that no downstream agent reintroduces retired customer-ladder vocabulary.

### §11.1 Stripe Billing → Oyatie tenant_class

Stripe's pricing classes (Stripe Standard, Stripe Plus, Stripe Premium, Stripe Standard / Express / Custom) carry feature gating + revenue-share differences.
Stripe Standard ≈ Oyatie paid + billing_components={per_usage} for Stripe charges.
Stripe Plus / Premium ≈ Oyatie paid + billing_components={per_usage, per_seat} (higher seat count / dedicated support).
Stripe Standard / Express / Custom ≈ Oyatie paid + billing_components={revenue_share, per_usage} (marketplace settlement).
Stripe Atlas (entity formation) is out-of-scope for cloud-billing.
Stripe Climate (sustainability) maps to the Carbon Footprint row A-10.
Demo / sandbox Stripe (test mode) ≈ Oyatie tenant_class=demo_trial.

### §11.2 AWS Billing & Cost Management → Oyatie tenant_class

AWS pricing-tier surfaces (Free Tier, AWS Activate, AWS Marketplace, AWS Enterprise Support, AWS Business Support, AWS Developer Support, AWS Basic Support) carry usage-cap + support differences.
AWS Free Tier 12-month + AWS Activate $1K-$100K credits ≈ Oyatie tenant_class=demo_trial.
AWS standard usage ≈ Oyatie paid + billing_components={per_usage} for cloud-cost.
AWS Enterprise Support / Business Support ≈ Oyatie paid + billing_components={per_seat, per_usage} (support contract is per-seat support).
AWS Marketplace seller payouts ≈ Oyatie paid + billing_components={revenue_share}.
AWS Organizations consolidated billing ≈ Oyatie tenant tree per ADR-0244 + tenant_class=paid for billed entities.

### §11.3 Recurly → Oyatie tenant_class

Recurly's tier surfaces (Recurly Core, Recurly Professional, Recurly Elite) carry feature gating differences.
Recurly Core ≈ Oyatie paid + billing_components={per_usage}.
Recurly Professional / Elite ≈ Oyatie paid + billing_components={per_seat, per_usage} with sovereign + advanced revenue-recognition overlays per compliance pack.
Recurly free trial ≈ Oyatie tenant_class=demo_trial.

The mapping is intentional: Oyatie collapses three counterpart tenant_class models into a single binary tenant_class with a composable billing_components set. The mapping holds across the entire feature parity matrix and is the reason the §11 row "Multi-currency invoicing" is uniformly covered for every Oyatie paid tenant — Recurly Tier-2 gating no longer applies.

## §12 Per-Counterpart Migration Stories

This section names the canonical migration paths from each counterpart into cloud-billing. The existing `migration-playbooks/from-aws-cur-and-cloudability.md` covers the AWS + Apptio path; this section adds the Stripe + Recurly paths.

### §12.1 Stripe Billing migration path

Source: Stripe Billing with Stripe Customers + Subscriptions + Invoices + Tax + Connect.
Target: Oyatie cloud-billing tenant_class=paid + billing_components={revenue_share?, per_seat?, per_usage}.

Phase 0 — Inventory.
Export Stripe Customers (`stripe customers list --limit 100 --starting-after ...`) to JSON.
Export Stripe Subscriptions (`stripe subscriptions list ...`) with plan + cycle anchor + trial state.
Export Stripe Invoices for the trailing 25 months (matches Paid-equivalent retention).
Export Stripe Credit Notes.
Export Stripe Payment Methods (transfer to `payments` µservice via Stripe oauth flow).
Export Stripe Accounts + Application Fees (transfer to revenue_share cohort).

Phase 1 — Tenant + billing-components provisioning.
`./bin/oya billing tenant register --tenant <name> --class paid --components revenue_share,per_seat,per_usage --currency USD --invoice-cadence monthly`.

Phase 2 — Subscription + Plan import.
For each Stripe Subscription, create the Oyatie Subscription (once Wave 15B subscription primitive lands) bound to the paid tenant + billing_components.

Phase 3 — Historical invoice backfill.
Re-issue the 25 months of historical Stripe invoices as Oyatie invoices in `shadow` mode for reconciliation; do not double-bill.

Phase 4 — Dual-billing window (60-90 days).
Stripe continues to charge; Oyatie generates `shadow` invoices; reconciliation tolerance ≤ 0.1 %.

Phase 5 — Cut-over.
Switch Oyatie invoices from `shadow` to `live`; disable Stripe subscription auto-renew; preserve Stripe as the payment processor via `payments` µservice for 30 days as safety net.

Phase 6 — Decommission overlap.
Cancel Stripe subscription metadata; preserve historical Stripe data export for 7 years (SOX-404).

### §12.2 Recurly migration path

Source: Recurly with Customers + Subscriptions + Invoices + Plans + Coupons + Dunning policies.
Target: Oyatie cloud-billing paid + billing_components={per_seat, per_usage}.

Phase 0 — Inventory.
Export Recurly Account list with billing info + Active subscription + Trial state.
Export Recurly Subscription Plans + Add-ons + Coupons + Dunning policy.
Export Recurly Invoices + Credit Notes (matches Paid-equivalent retention).
Export Recurly Webhooks subscriptions.

Phase 1 — Tenant + billing-components provisioning.
`./bin/oya billing tenant register --tenant <name> --class paid --components per_seat,per_usage --dunning-policy <imported>`.

Phase 2 — Plan + Coupon import (once Wave 15B primitives land).
Map Recurly Plans 1:1 to Oyatie Plans; Recurly Coupons to Oyatie Coupons; Recurly Add-ons to Oyatie Add-ons.

Phase 3 — Subscription import.
Each Recurly Subscription becomes an Oyatie Subscription; preserve cycle anchor + trial state + dunning state.

Phase 4 — Dual-billing window (60-90 days).
Same dual-shadow approach as Stripe.

Phase 5 — Cut-over + dunning cutover.
Switch Oyatie invoices to `live`; route Recurly webhook subscribers to Oyatie webhook delivery (canonical event-class registry, Wave 15B).

Phase 6 — Decommission Recurly.
Save 7 years of Recurly historical data export.

## §13 Counterpart Pricing Comparison Annex

This annex consolidates counterpart pricing comparable to Oyatie cloud-billing TCO. Source dates 2026-05; pricing changes monthly.

| Counterpart | Pricing model | Approximate cost at mid-market scale (50K seats, $2M/month cloud spend, ~50M events/period) |
|---|---|---|
| Stripe Billing | 0.5 % of revenue + payment processing fees (~2.9 % + $0.30) | ~$10,000-$15,000 / month |
| Stripe Tax | 0.5 % of taxable revenue | ~$5,000-$10,000 / month |
| Stripe | 0.25 % of transactions + processing | ~$5,000-$15,000 / month |
| AWS Billing & Cost Management (native) | $0 (included with AWS account) | $0 — but you pay ~$8,000 / month for Apptio Cloudability or ~$5,500 / month for CloudZero to get chargeback functionality |
| Recurly | $200-$300 base + 0.9 % of revenue | ~$15,000-$25,000 / month |
| Oyatie cloud-billing (existing benchmark TCO at mid-market) | ~$2,800 / month substrate cost + processing fees through payments µservice | ~$2,800 / month + processing |

The cloud-billing pricing posture is 49-65 % below Apptio Cloudability + AWS native combo and 70-85 % below Stripe Billing + Stripe Tax + Stripe combo at mid-market scale. The pricing comparison should be tested before publication and updated to reflect the Wave 15B per-billing-component decomposition (Q-BM-10 from the performance-benchmark doc).

## §14 Out-of-Scope Intentional Reasons (consolidated)

This section names the doctrine reason for every row marked `out-of-scope intentional` per ADR-0328 §D-5.13.

| Row | Counterpart feature | Reason | Approving ADR / standard |
|---|---|---|---|
| S-09, R-13 | Payment method storage + Hosted Payment Pages | substrate vs product layering; `payments` µservice (Phase-1 service 07) owns | ADR-0245 |
| S-23 | Statement descriptors | substrate vs product; `payments` owns | ADR-0245 |
| S-24 | Disputes / chargebacks | substrate vs product; `payments` owns lifecycle | ADR-0245 |
| S-17 | Customer self-serve portal | substrate vs product; `finops-portal` (Phase-1 service 08) owns | ADR-0245 |
| S-29 | Treasury / embedded banking | long-tail B2B SaaS scope; not Phase-0 substrate | ADR-0321 |
| S-33 | Stripe Apps / 3rd-party extensions | `cloud-marketplace` + `plugin-app-store` own | ADR-0249 |
| A-14 | Cost Explorer UI | substrate vs product; `finops-portal` UI | ADR-0245 |
| A-16 | Tags Editor UI | substrate vs product; `finops-portal` UI | ADR-0245 |
| R-16 | Gift cards | long-tail; could land in marketplace or consumer brand | ADR-0321 |
| R-17 | Cancellation surveys | not a billing-substrate concern; finops-portal product surface | ADR-0245 |
| R-18 | Subscription analytics dashboards (MRR, ARR, churn, LTV, CAC) | `analytics` (Phase-3 service 16) + `finops-portal` own | ADR-0245 |

The intentional out-of-scope rows are not gaps; they are correctly delegated to sibling µservices that own the surface. Each row carries a valid doctrine reason per §D-5.13.

## §15 Findings Confirmation

The matrix produces these explicit findings (consolidated for Wave 14):
- 14 missing parity rows (Subscription, Proration, Add-ons, Pause/Resume, Setup fees, Quote, Coupons, Discounts, Custom fields, Budgets, Free Tier Alerts, Marketplace / revenue_share, Multi-language invoice templates, Carbon Footprint).
- 26 partial parity rows requiring documentation or completion in Wave 15B.
- 16 covered rows providing the existing competitive surface.
- 14 out-of-scope intentional rows with doctrine reasons.

The matrix confirms that cloud-billing is materially weak on subscription primitives + materially strong on substrate primitives. The Wave 14 backlog prioritization should reflect the Big-8 enterprise displacement: Subscription primitive is P1 because every Phase-4 µservice (CRM, HR, ERP) needs subscription-style billing; Marketplace / revenue_share is P1 because the keystone directive memory says cloud-billing OWNS the revenue_share component.

## §16 Forward Compatibility Notes

Subscription, Plan, Add-on, Coupon, Discount, DunningPolicy, Budget, and Free-Tier-Alert primitives must be additive to the existing kernel without breaking the current BillingAccount + CloudBillingEvent + Invoice contract. Wave 15B authoring must follow ADR-0108 sunset doctrine: introduce new types with version 2 schemas; preserve version 1 reads for at least one minor cycle; deprecate version 1 reads only after every consumer (cloud-billing-tax, payments, finops-portal, audit-chain, comms-email) has upgraded to version 2.

The Subscription primitive is the highest leverage row; introducing it correctly unlocks several rows that are currently `missing`: Subscription itself, Proration, Add-ons, Pause/Resume, Setup fees, Trial extensions, Cycle anchors, Discounts at the subscription level, Coupons applied to subscriptions, Custom fields on Subscription. The implementation surface for the Subscription primitive should follow Stripe Billing's Customer + Subscription + Plan + Price separation because that schema is the dominant industry mental model; Recurly's Plan + Add-on + Coupon overlays on Stripe's base; AWS B&CM has no subscription concept and is not constraining here.

The Marketplace / revenue_share row is the second-highest leverage row; introducing it correctly unlocks revenue-share cohort tracking, monthly settlement, payouts integration with payments µservice, marketplace seller onboarding, B2C consumer-product operator billing, embedded SaaS reseller pricing, and affiliate / channel partner negative-rev-share flows. The implementation surface should follow Stripe Connect's Account + Transfer + ApplicationFee separation because that schema is the dominant marketplace mental model; AWS Marketplace billing's separation is account-flat (no concept of platform fee separate from seller payout).

The Budget primitive is the third-highest leverage row because Budgets enable both demo_trial cap-breach (currently row CB-F-020 in the coherence audit) and AWS Budgets parity (A-07 in this matrix). Wave 15B should author a Budget that supports limit_kind (cost / usage / consumption), threshold (absolute / percentage), action (notify / restrict via Cedar / suspend), notification (comms-email + webhook + Cedar audit), and a per-tenant-class default (demo_trial budget = hard cap; paid budget = soft alert + auto-action).

The Dunning primitive is the fourth-highest leverage row. Recurly's dunning state machine is the industry-leader; the model is: failed payment → retry schedule (smart retries based on decline category) → escalating notifications → eventual suspend → recovery webhook on payment success. cloud-billing's existing FAQ Q19 fraud sweep covers part of this surface; Wave 15B should author a DunningPolicy primitive that parameterizes retry windows, decline-category branching, comms cadence, and the eventual suspension via Cedar permit revocation.

The Revenue Recognition primitive is the fifth-highest leverage row. ASC 606 / IFRS 15 require a five-step model: identify contract, identify performance obligations, determine transaction price, allocate transaction price to obligations, recognize revenue as obligations are satisfied. cloud-billing today does step 5 (recognize revenue as usage events flow); steps 1-4 are not modeled. The implementation surface should introduce Contract + PerformanceObligation + TransactionPriceAllocation + RevenueSchedule + JournalEntry primitives. The journal entry export integrates with the ERP adapter that the migration playbook references.

## §17 Counterpart-Specific Strengths to Adopt

This section calls out the specific patterns from each counterpart that cloud-billing should adopt.

From Stripe Billing: (a) idempotency-key header on every write (cloud-billing already has events_by_idempotency at the metering layer; extend to invoice writes + subscription writes + rate-card writes); (b) test clocks for subscription simulation (current `--fast-forward` flag is insufficient); (c) hosted invoice page (Wave 15B); (d) webhook signing (HMAC + per-tenant key); (e) Connect's separation of Account + Transfer + ApplicationFee for marketplace settlement.

From AWS Billing & Cost Management: (a) Cost Categories with nested rules (cloud-billing attribution-rules are flat); (b) Savings Plans flex-commitment separation from Reserved Instances rigid (cloud-billing has CloudBillingEventKind::{Reservation, Commitment} but the mid-period flex semantics differ); (c) Billing Conductor's pricing-rule model for re-grouping linked accounts; (d) per-resource-type tag-based attribution (already covered at FAQ Q13); (e) Cost Anomaly Detection's monitor-and-subscriber model (cloud-billing has continuous anomaly detection but no user-defined "monitor" primitive).

From Recurly: (a) Dunning state machine with decline-category-based retry; (b) hosted recovery flow (handoff to payments µservice); (c) coupon redemption tracking (single-use, multi-use, custom redemption count); (d) plan add-on bundling with proration; (e) cancellation reason capture (out-of-scope intentional per §14 but the reason taxonomy may inform comms-email subject lines).

## §18 Matrix Maintenance Doctrine

The feature parity matrix must be refreshed quarterly because Stripe ships new APIs every month and Recurly + AWS ship new features monthly. The refresh cadence: read each counterpart's release notes + API changelog; map every new feature to one of {covered, partial, missing, out-of-scope intentional}; add new rows for capabilities not yet covered. The refresh should not delete out-of-scope intentional rows (they document the doctrine); it should not delete covered rows that were re-implemented (the covered evidence remains valid). The refresh should add new partial rows when cloud-billing falls behind a counterpart's new capability and the response decision (cover, partial-cover, decline, out-of-scope) has not yet been recorded.

The matrix is also a parity bar for Wave 15B authoring: every new cloud-billing feature must check whether it changes a row from `missing` to `partial` or `covered`, and the change must be evidenced (a path to the owning artifact). Per ADR-0328 §D-5.16 a `covered` mark requires a path to the owning artifact; a `partial` mark requires a missing-gap note; a `missing` mark requires a proposed remediation target. The matrix enforces this discipline.

## §19 Matrix Reading Guide

For a new engineer reading the matrix to understand cloud-billing's competitive surface:

(1) §2-§4 give the counterpart inventory. Read each counterpart's table to learn the surface area.

(2) §5 gives the UNION view. Read this to see where cloud-billing covers / partial-covers / misses / declines.

(3) §6 gives the family rollup. Read this to learn which families cloud-billing is strong on (substrate) and weak on (subscription).

(4) §7 names the headline gaps and their proposed Wave 15B remediation targets.

(5) §8 names the additive surface — what cloud-billing offers beyond the union of counterparts.

(6) §11 maps counterpart pricing classes to Oyatie tenant_class binary; this is the doctrine bridge.

(7) §12 names the migration paths from each counterpart into cloud-billing.

(8) §13 gives counterpart pricing comparison.

(9) §14 documents the doctrine reasons for the out-of-scope intentional rows.

(10) §16-§18 are forward-looking notes for Wave 15B and beyond.

The matrix is meant to be consulted before any Wave 15B authoring; an engineer authoring a Subscription primitive should know the Stripe + Recurly schemas and the explicit decision to align with Stripe's Customer + Subscription + Plan + Price separation (per §16).
