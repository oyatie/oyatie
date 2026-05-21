# `marketplace` µservice — Benchmark vs Stripe Connect, Shopify Plus, Amazon Marketplace, Salesforce AppExchange

> Measured 2026-04-29 to 2026-05-17 across 5 mixed workloads (publish, purchase, escrow, payout, dispute) over 3 trial windows.
> Numbers reflect the paid `marketplace` revenue_share + per_usage path vs comparable vendor enterprise plans.

## Listing categories supported

| Surface | Plugins | Apps | Workflows | Agents | ML Models | Datasets | One ledger across categories? |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `marketplace` (paid) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Stripe Connect | n/a | n/a | n/a | n/a | n/a | n/a | n/a (payment rail, not categories) |
| Shopify Plus | ❌ | ✅ (apps) | ❌ | ❌ | ❌ | ❌ | n/a |
| Amazon Marketplace | partial (via separate stores) | ❌ | ❌ | ❌ | partial (AWS Marketplace) | partial (Data Exchange) | ❌ (separate ledgers) |
| Salesforce AppExchange | ❌ | ✅ | ❌ | partial (agents) | ❌ | ❌ | n/a |

## Settlement latency (purchase → seller payable in escrow)

| Surface | p50 | p95 | p99 |
| --- | --- | --- | --- |
| `marketplace` (paid) | **180 ms** | **420 ms** | **820 ms** |
| Stripe Connect (Express) | 280 ms | 640 ms | 1.2 s |
| Shopify Plus (cart → order paid) | 1.4 s | 3.8 s | 8.1 s |
| Amazon Marketplace (purchase → MWS event) | 12 s | 38 s | 95 s |
| AppExchange (purchase → ISV ledger) | 4.2 s | 12 s | 28 s |

## Payout cadence

| Surface | Demo/trial path | Paid default | Paid high-volume policy | Real-time payout? |
| --- | --- | --- | --- | --- |
| `marketplace` | no live payout | 7 d / 1 d by policy | hourly/realtime by policy | ✅ paid policy |
| Stripe Connect | 7 d (rolling) | 2 d (Standard) | 1 d (Custom) | ✅ Instant Payouts ($0.50/txn) |
| Shopify Plus | 3-7 d | 2 d | 2 d | ❌ |
| Amazon Marketplace | 14 d (most categories) | 14 d | 7 d (some) | ❌ |
| AppExchange | quarterly | monthly | monthly | ❌ |

## Platform fee at $1M/yr listing seller GMV

| Surface | Effective platform fee | Tax handling | KYC included | Dispute team |
| --- | --- | --- | --- | --- |
| `marketplace` (paid) | **5 % + $0.20/tx ≈ 5.6 %** | platform (110 jurisdictions) | ✅ | ✅ |
| Stripe Connect (Custom) | 2.9 % + $0.30 card + 0.25 % + $2 routing ≈ 3.5 % | + Stripe Tax 0.5 % | partial | external |
| Shopify Plus | 0 % platform (just 2.9 % payment) + Shopify $2,000/mo | partial (Shopify Tax) | ✅ | partial |
| Amazon Marketplace | 8-17 % category-dependent + $39.99/mo | platform | ✅ | ✅ |
| AppExchange | 25 % ISV revenue share | partial | ✅ | partial |

## Refund / dispute flow

| Surface | Auto-rules for low-value? | Buyer-evidence + seller-evidence? | Cedar-gated stages? | Audit chain |
| --- | --- | --- | --- | --- |
| `marketplace` | ✅ | ✅ | ✅ | ✅ BLAKE3 chain |
| Stripe Connect | partial (Radar rules) | partial | ❌ | append-only |
| Shopify Plus | ❌ | ❌ | ❌ | append-only |
| Amazon Marketplace | ✅ (A-to-z) | ✅ | ❌ | append-only |
| AppExchange | ❌ | n/a | ❌ | partial |

## TCO at $5M GMV/yr seller, 50,000 transactions/yr

| Surface | Annual platform fee | Tax engine | KYC | Dispute platform | Total | Net to seller |
| --- | --- | --- | --- | --- | --- | --- |
| `marketplace` (paid) | $280,000 (5.6 %) | included | included | included | $280,000 | $4,720,000 |
| Stripe Connect + Stripe Tax + Persona | $175,000 (3.5 %) | $25,000 | $18,000 (Persona) | $30,000 (external) | $248,000 | $4,752,000 |
| Shopify Plus | $0 platform + $24,000 SaaS + $145,000 payments | $12,000 | $18,000 | external | $199,000 | $4,801,000 |
| Amazon Marketplace | $725,000 (14.5 % avg) | included | included | included | $725,000 | $4,275,000 |
| AppExchange | $1,250,000 (25 %) | included | included | included | $1,250,000 | $3,750,000 |

At $5M GMV the cheapest is Shopify Plus, but Shopify covers only the `apps` category and doesn't ledger across categories. For a
multi-category seller, `marketplace` (paid) is competitive with Stripe Connect-built-yourself and dramatically cheaper than Amazon
or AppExchange.

## Governance + compliance

| Surface | SOC 2 | GDPR | HIPAA | PCI DSS | EU AI Act ready | DAC7 / MTR? |
| --- | --- | --- | --- | --- | --- | --- |
| `marketplace` (paid) | ✅ | ✅ | ✅ | ✅ v4.0 | ✅ (regulated pack) | ✅ |
| Stripe Connect | ✅ | ✅ | partial (BAA) | ✅ | ❌ | partial |
| Shopify Plus | ✅ | ✅ | ❌ | ✅ | ❌ | partial |
| Amazon Marketplace | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ |
| AppExchange | ✅ | ✅ | ✅ (HIPAA-compliant orgs) | ✅ | ❌ | n/a |

## Where `marketplace` wins

1. Multi-category from day 1.
2. One settlement ledger across categories.
3. Cedar-gated dispute stages.
4. BLAKE3 audit chain.
5. EU AI Act readiness as a pack overlay.
6. provider-credential BYOK at the listing (ADR-0255 §D-4).

## Where vendors win

1. Stripe Connect has the most mature global payment-rail coverage; we ride on top of `payments`+Stripe under the hood for many rails.
2. Shopify Plus has the lowest sticker price for app-category single-purpose sellers.
3. Amazon Marketplace has unparalleled buyer reach.
4. AppExchange has the most mature ISV ecosystem in CRM.

## Reproducibility

```bash
make benchmarks.marketplace.run \
  VENDORS="marketplace,stripe-connect,shopify-plus,amazon-marketplace,appexchange" \
  WORKLOADS="publish,purchase,escrow,payout,dispute" \
  TRIALS=3
```

Evidence: `.foundry/evidence/benchmarks/marketplace/2026-05-17T15:11:33Z/`.
