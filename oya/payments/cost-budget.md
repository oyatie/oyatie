---
doc_class: CostBudget
template_id: TPL-COST-BUDGET
microservice: payments
status: Accepted
date: 2026-05-20
owner_team: axis-payments + ops-treasury + council-finance
related_adrs: [ADR-0174, ADR-0244, ADR-0249, ADR-0251]
companion_docs:
  - microservices/payments/PRD.md
  - microservices/payments/ARCHITECTURE.md
  - microservices/payments/capacity-model.md
  - microservices/payments/competitor-parity-matrix.md
diataxis_quadrant: reference
doc_status: published
---

# Cost Budget — payments µservice

> Per-PSP fee structure, per-100k-txn cost model, FinOps cost attribution per ADR-0174.

---

## §1. PSP fee structure (per provider, 2026-05-20 list rates)

Tenants pay PSPs directly (provider-BYOK per ADR-0255 §D-4); oyatie does not mark up PSP fees. The platform-fee oyatie charges sits **above** PSP fees and is configured per tenant via the marketplace fees policy.

### 1.1 Stripe (US base; 2026-05-20)

| Surface | Fee |
|---|---|
| Card (US, online) | 2.9% + $0.30 |
| Card (international) | 3.9% + $0.30 + 1% currency conversion |
| ACH Direct Debit | 0.8% (capped $5) |
| Card-present (in-person) | 2.7% + $0.05 |
| Subscription billing platform | + $10 / 1000 invoices |
| (platform-facilitator) | + 0.25% + $0.25 per payout |
| Identity (KYC) | + $1.50 per verification |
| Radar (fraud) | + $0.05 / charge |

### 1.2 Adyen (EU base; 2026-05-20)

Interchange-plus pricing (negotiated per tenant volume):

| Surface | Fee |
|---|---|
| EU card (interchange + ~0.40% markup + €0.10) | ~0.5-1.5% + €0.10 |
| International card | + 1.5% markup |
| SEPA Direct Debit | €0.20 flat |
| MarketPay payout | + €0.25 per payout |

### 1.3 Toss Payments (KR)

| Surface | Fee |
|---|---|
| Domestic card | 2.5% + ₩60 |
| Bank transfer | ₩50 flat |
| Wallet (Toss Pay) | 1.8% |
| Payout | ₩500 per payout |

### 1.4 KakaoPay (KR)

| Surface | Fee |
|---|---|
| Wallet | 2.5% |
| Payout | ₩500 per payout |

### 1.5 LINE Pay (JP / TW / TH)

| Surface | Fee |
|---|---|
| JP wallet | 3.5% |
| TW / TH | 2.8% |

### 1.6 WeChat Pay + Alipay (CN)

| Surface | Fee |
|---|---|
| Domestic | 0.6% (Chinese mainland tier) |
| Cross-border (Alipay+) | + 1.5% currency conversion |

### 1.7 PayPal Business (post-MVP)

| Surface | Fee |
|---|---|
| Online | 3.49% + $0.49 |
| Micropayment (<$10) | 4.99% + $0.09 |

### 1.8 Coinbase Commerce (post-MVP)

| Surface | Fee |
|---|---|
| Crypto charge | 1.0% |

## §2. Per-100k-transaction cost model (illustrative)

Assuming average sale price (ASP) of $10 USD and standard Stripe 2.9% + 30¢:

| Cost class | Per-txn | Per 100k-txn |
|---|---:|---:|
| Stripe PSP fee | $0.59 ($10 × 2.9% + $0.30) | $59,000 |
| Stripe Radar (fraud) | $0.05 | $5,000 |
| Stripe markup | $0.025 ($10 × 0.25%) (sub-merchant flows only) | $2,500 (for 100k flows) |
| Stripe Identity (per onboarded sub-merchant, amortised) | $0.015 (1.5% × $1.50, assuming 1 KYC per 100 charges) | $1,500 |
| Per-charge infra (compute + DB + audit-chain seal + observability) | $0.002 | $200 |
| Per-charge bandwidth (HTTP/3 ingress + PSP outbound) | $0.0005 | $50 |
| **Total per 100k-txn** | **~$0.69** | **~$68,250** |
| **Platform-fee oyatie charges** (varies; default 1% for marketplace flows) | $0.10 (1% × $10) | $10,000 |
| **Net per 100k-txn** | $0.41 negative cash flow | $58,250 cost to oyatie before platform fee |

Sustained at 18.75M charges/day (year-3 GA): **$12.8M/day total cost** with platform-fee revenue ~$1.875M/day → net **$10.9M/day PSP+infra cost** (paid via tenant pass-through).

## §3. FinOps cost attribution (per ADR-0174)

Per ADR-0174, every dollar of cost is attributed to a (tenant_id, cell_id, BC) tuple:

| Cost class | Attribution dimension |
|---|---|
| PSP fees | (tenant_id, psp, currency) — direct pass-through |
| Infra compute (Cloud Hypervisor + Kata) | (tenant_id, cell_id, BC) via per-pod request-count |
| Database (CRDB rows) | (tenant_id, cell_id, BC) — row-count attribution |
| Storage (audit-chain + dispute-evidence) | (tenant_id, cell_id, BC) — bytes-stored attribution |
| Bandwidth (ingress / egress) | (tenant_id, cell_id, BC) — bytes-transferred |
| Observability (metrics / logs / traces / audit) | (cell_id, BC) — fixed overhead, allocated by usage ratio |
| Cedar evaluation | (tenant_id) — count × per-eval-cost |

Attribution lives in `dashboards/finops-cost-attribution.md` (cross-references `microservices/finops-portal/`).

## §4. Cost-control levers

| Lever | Mechanism | Estimated savings |
|---|---|---|
| Negotiate Stripe interchange-plus | At >$10M/year tenant volume, switch from flat 2.9% + 30¢ to interchange-plus pricing | -0.3 to -0.8 percentage points per charge |
| Per-region PSP routing | EU charges via Adyen interchange-plus (rather than Stripe US-base) | -0.5 to -1.0 pp on EU volume |
| Subscription billing platform inhouse | Use our own dunning + smart-retries vs Stripe Billing | -$10/1000 invoices |
| Fraud-ML inhouse | Use library-first fraud-scoring vs Radar | -$0.05/charge |
| Webhook-handler colocation | Run webhook handlers in same cell as PSP-region routing | -0.5% per-charge bandwidth |
| Audit-chain seal-batching | 60s seal batches vs per-row seal | -90% audit-chain compute |
| Reconciliation deduplication | Daily reconciliation reuses charge-row pre-fetched | -30% reconciliation worker cost |

## §5. Budget envelope (year-1 → year-3)

| Year | Daily volume | Daily total cost | Annual cost | Net of platform fee |
|---:|---:|---:|---:|---:|
| Y1 launch | 358k | $245k | $89M | $20M oyatie-paid |
| Y2 ramp | 2.5M | $1.7M | $620M | $137M oyatie-paid |
| Y3 GA | 18.75M | $12.8M | $4.67B | $1.04B oyatie-paid |

Note: net oyatie-paid is the cost above platform-fee revenue. Most of the cost is direct pass-through to PSPs via tenant accounts; oyatie only carries the variable infra + audit + platform-fee-collection overhead.

## §6. Compliance-pack-driven additional cost

| Pack | Additional cost driver |
|---|---|
| `pack-pci-dss-l1-v4` | QSA audit ~$50-150k/year; staff training; HSM-backed key-management |
| `pack-kr-fss` | KR-FSS audit ~₩50M/year; KR-domiciled DPO; periodic audit-trail pull tooling |
| `pack-eu-psd2-sca` | SCA-step-up flows add ~5% transaction friction (cost: lower conversion) |
| `pack-us-state-mtl` | Per-US-state MTL licence ~$2-25k/state/year; surety bonds; agent-for-service |
| `pack-cn-pipl-2021` | CN-domiciled infra + DPO + regulatory liaison |
| `pack-au-aml-ctf` | AUSTRAC reporting tooling + KYB-ML enrichment |
| `pack-br-lgpd-finance` | BACEN-licence overhead |
| `pack-in-rbi` | RBI payment-aggregator licence + IN-domiciled infra |

## §7. Acceptance signals

- Per-100k-charge total cost ≤ $0.75 (variable + fixed) at year-3 GA volumes.
- PSP-fee per tenant correctly attributed; reconciliation discrepancy ≤ 0.01% per month.
- FinOps dashboard shows per-(tenant, cell, BC) breakdown updated every 24h.

## §8. References

- [`capacity-model.md`](capacity-model.md).
- [`competitor-parity-matrix.md`](competitor-parity-matrix.md).
- [`dashboards/finops-cost-attribution.md`](dashboards/finops-cost-attribution.md).
- Stripe pricing — `stripe.com/pricing`.
- Adyen pricing — `adyen.com/pricing`.
- Toss Payments pricing — `tosspayments.com/pricing`.
- [ADR-0174 — FinOps cost attribution per (tenant, cell, BC)](../../docs/decisions/ADR-0174-finops-cost-attribution.md).
- [ADR-0249 — multi-category marketplace](../../docs/decisions/ADR-0249-multi-category-marketplace-doctrine.md).
