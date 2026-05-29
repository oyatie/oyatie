---
doc_class: CapacityModel
template_id: TPL-CAPACITY-MODEL
microservice: payments
status: Accepted
date: 2026-05-20
owner_team: axis-payments + ops-sre-reliability
related_adrs: [ADR-0145, ADR-0244, ADR-0248, ADR-0252, ADR-0253]
companion_docs:
  - microservices/payments/PRD.md
  - microservices/payments/ARCHITECTURE.md
  - microservices/payments/multi-region.md
  - microservices/payments/failure-modes.md
diataxis_quadrant: reference
doc_status: published
---

# Capacity Model — payments µservice

> Per-PSP throughput math, per-tenant quota, peak-vs-base load model, queue-theoretic sizing for charge / refund / payout / dispute / subscription. Every number is derived; no hand-waves.

---

## §1. Demand model — the load shape

### 1.1 Per-µservice consumer baseline (year-1 launch → year-3 GA)

Consumer-facing surfaces that drive payment volume:

| Surface | Year-1 daily charges | Year-3 daily charges | Driver |
|---|---:|---:|---|
| `messenger` sticker store | 200k | 5M | Per-MAU sticker-purchase rate ~5%/month |
| `shorts` creator-tip | 100k | 8M | Per-creator-cohort growth |
| `community` super-chat | 50k | 4M | Live-event spike (5x burst) |
| `cloud-billing` usage invoicing | 5k | 200k | Per-tenant monthly billing cycle (≤30k tenants) |
| `plugin-app-store` checkout | 2k | 500k | Per-developer-cohort growth |
| `marketplace` apps + agents + datasets | 1k | 1M | Marketplace-volume growth |
| `connector` escrow | 100 | 50k | Tail-light B2B contract escrow |
| **Total** | **~358k** | **~18.75M** | |

### 1.2 Burst factor

- **Sustained peak**: 3× base (sports events, Black Friday, KR Lunar New Year, JP shopping-day, IN festival-window).
- **Flash peak**: 10× base, ≤5 min duration (super-chat super-events, viral creator drops, exclusive sticker-launch).
- **24h max**: 5× base (Black Friday sustained 12h).

Year-3 sustained-peak ceiling: **~56M charges/day** = **~648/s** average; flash-peak **~6480/s**.

### 1.3 Per-event-class mix (year-3 GA)

| Event class | Share of total volume | Notes |
|---|---:|---|
| One-time charge (B2C) | 65% | Sticker / tip / super-chat / app-purchase |
| Recurring subscription charge | 20% | Per-seat SaaS + creator-sub |
| Usage-metered invoice (B2B) | 10% | Cloud-billing tenant invoicing |
| Refund | 3% | Industry-standard refund rate |
| Dispute | 0.2% | Industry-standard chargeback rate |
| Payout | 1.5% | Daily payout batches |
| Sub-merchant onboarding | 0.3% | KYC / KYB |

## §2. Per-PSP capacity ceilings

Each PSP has rate-limits that constrain our routing.

### 2.1 Stripe

| Surface | Rate-limit | Notes |
|---|---|---|
| Live mode | 100/s/account (default) | Tenant-account-level; can request raise. |
| Test mode | 25/s/account | Sandbox only. |
| Webhook | unlimited from Stripe → us | We absorb at edge + queue. |
| platform | inherits per-sub-merchant-account | Each sub-merchant has its own 100/s ceiling. |

**Implication**: a tenant with >50k charges/day needs Stripe rate-limit raise (Stripe approves on legitimate volume).

### 2.2 Adyen

| Surface | Rate-limit | Notes |
|---|---|---|
| Live mode | ~200/s/account | Higher than Stripe default. |
| Webhook | unlimited | |

### 2.3 Toss Payments (KR)

| Surface | Rate-limit | Notes |
|---|---|---|
| Live mode | ~50/s/account | KR-FSS-licensed; KRW-only. |
| Reconciliation | daily settlement file | T+1 settlement window. |

### 2.4 KakaoPay (KR)

| Surface | Rate-limit | Notes |
|---|---|---|
| Live mode | ~50/s/account | Wallet-only; KRW. |

### 2.5 LINE Pay (JP / TW / TH)

| Surface | Rate-limit | Notes |
|---|---|---|
| Live mode | ~100/s/account | Per-region. |

### 2.6 WeChat Pay + Alipay (CN)

| Surface | Rate-limit | Notes |
|---|---|---|
| Live mode | ~50/s/account each | CN-only; PIPL data-residency. |

### 2.7 Aggregate ceiling

Aggregate PSP capacity across all tenants (assuming each tenant has rate-limit raises for its volume):

| Tier | Per-PSP-baseline-ceiling | Tenant-fan-out | Aggregate |
|---|---:|---:|---:|
| Stripe | 100/s | × 5,000 active tenants = 500,000/s | 500k/s |
| Adyen | 200/s | × 500 EU tenants = 100,000/s | 100k/s |
| Toss / KakaoPay | 100/s combined | × 200 KR tenants = 20,000/s | 20k/s |
| LINE Pay | 100/s | × 50 JP/TW/TH tenants = 5,000/s | 5k/s |
| WeChat / Alipay | 100/s combined | × 30 CN tenants = 3,000/s | 3k/s |
| **Aggregate** | | | **~628k/s** |

Far exceeds Year-3 sustained-peak demand of 648/s; we have ~1000× headroom at the PSP layer. Our bottleneck will be internal (DB writes + audit-chain seal latency), not PSP.

## §3. Internal bottleneck math

### 3.1 Charge-API write path

**Steps** per Charge::Create:

1. Cedar evaluation (library-first, p99 ≤2ms).
2. Ontology read (tenant + user; library-first, p99 ≤5ms).
3. Fraud-scoring (library-first, p99 ≤20ms).
4. PSP-adapter call (HTTPS round-trip, p99 ≤200ms for Stripe US-region).
5. DB write (charges row + audit-chain row in same transaction, p99 ≤15ms on RF-3 CRDB).
6. Audit-chain seal (per ADR-0028 Merkle-append, p99 ≤10ms async).
7. Domain-event emit (AsyncAPI; p99 ≤5ms async).

**Critical-path latency budget**: 2 + 5 + 20 + 200 + 15 = **242ms p99**; SLO target ≤500ms p99 → 258ms headroom.

### 3.2 DB write throughput (CRDB RF-3)

Per-cell CRDB write capacity ~25k/s sustained, ~50k/s burst (RF-3 multi-AZ, NVMe storage).

| Cell tier | Cells | Aggregate write capacity |
|---|---:|---:|
| Tier-1 (regulated finance) | 6 (KR, EU, US, JP, AU, BR) | 150k/s sustained |
| Tier-2 (default product) | 12 (per major region pair) | 300k/s sustained |
| **Total** | 18 | **450k/s sustained**, **900k/s burst** |

At 6480/s flash-peak, we use 1.4% of sustained capacity. Comfortable headroom.

### 3.3 Audit-chain seal lag

Merkle-append per ADR-0028:

- Append per row: O(log N) hash operations.
- Per-day seal batch: ~50M rows → 256 batches × 64MB Merkle-root reanchor.
- Seal-publish to `governance` µservice: every 60s.

Worst-case seal lag: **60s after row write**. SLO: seal-lag ≤90s p99 (set in [`slos/`](slos/)).

### 3.4 Per-tenant quota

Per ADR-0244 every action is tenant-scoped. Default per-tenant quotas (overridable in tenant manifest):

| Action | Quota | Burst |
|---|---:|---:|
| `Charge::Create` | 100/s per tenant | 500/s for 60s |
| `Refund::Create` | 50/s per tenant | 200/s for 60s |
| `Payout::Schedule` | 10/min per tenant | 50/min |
| `SubMerchant::Onboard` | 5/min per tenant | 20/min |
| `Dispute::SubmitEvidence` | 10/min per tenant | 30/min |

Tenant quotas enforced via Cedar gate `payments.quota.cedar` (token-bucket per tenant-id).

## §4. Queue theory — webhook ingest

Webhooks from PSPs are bursty (PSPs retry aggressively during their own incidents).

### 4.1 Steady-state arrival rate

| PSP | Webhook events/s (year-3 GA, base) | Burst |
|---|---:|---:|
| Stripe | 800/s | 4000/s |
| Adyen | 200/s | 1000/s |
| Toss / KakaoPay | 100/s | 500/s |
| LINE Pay | 50/s | 250/s |
| WeChat / Alipay | 30/s | 150/s |
| **Total** | **1180/s** | **5900/s** |

### 4.2 Service time

p99 service time (webhook → audit-chain row): ~30ms (mostly DB write + idempotency lookup).

### 4.3 M/M/c sizing

For utilisation ρ ≤ 0.7 at burst (5900/s × 0.03s = 177 service-units / 0.7 = 253 servers needed).

We provision **128 webhook-handler replicas** per cell (3 cells per region active concurrently → 384 total). Burst HPA up to 256 replicas per cell during PSP-incident-storms.

Reference: [Erlang-C calculator + queue theory chapter in Janert "Feedback Control for Computer Systems"].

## §5. Per-region distribution

Per [`multi-region.md`](multi-region.md) cell distribution:

| Region | Cells | Year-3 daily volume share | Notes |
|---|---:|---:|---|
| US-East / US-West / US-Central | 6 | 35% | Tier-2 default + Tier-1 regulated. |
| EU-West / EU-Central | 4 | 25% | EU PSD2 cells. |
| KR | 2 | 18% | Tier-1 KR-FSS cells; KRW-only flows; Toss / KakaoPay routed in-region. |
| JP | 1 | 8% | LINE Pay region. |
| SG / AU / IN / BR / AE / KSA | 5 | 12% | Tier-2 default per region. |
| CN | 1 | 2% | CN-PIPL Tier-1; WeChat Pay / Alipay only. |

## §6. Cost-capacity frontier

Per [`cost-budget.md`](cost-budget.md), cost scales **linearly** with volume above 1M charges/day (PSP fees dominate). Below 1M/day, fixed infra ~$8k/month/cell dominates.

| Daily volume | Variable cost (PSP fees @ 2.9% + 30¢) | Fixed infra cost | Total/day | Per-charge |
|---:|---:|---:|---:|---:|
| 100k | $87k (assuming $10 ASP) | $0.5k | $87.5k | $0.875 |
| 1M | $870k | $1.5k | $871.5k | $0.871 |
| 10M | $8.7M | $5k | $8.705M | $0.870 |
| 18.75M (year-3) | $16.3M | $8k | $16.31M | $0.870 |

At year-3 GA, fixed infra is **<0.1% of total cost**. The cost lever is PSP-fee-negotiation, not infra.

## §7. Scale-out path

| Bottleneck candidate | Mitigation |
|---|---|
| Per-tenant Stripe rate-limit | Request raise; sharding via sub-merchants. |
| Per-cell CRDB write capacity | Horizontal cell-shard expansion per ADR-0248. |
| Audit-chain seal lag | Parallel-seal-shard by tenant_id-prefix; 16-shard parallelism. |
| Webhook-handler queue depth | HPA on queue-depth; cross-cell rebalance. |
| Dispute-evidence storage | Object-storage scales to PB; no internal bottleneck. |
| Subscription dunning-cron | Per-tenant cron-shard. |

## §8. Acceptance signals

- p99 charge-API latency ≤500ms at year-3 sustained-peak.
- Per-tenant quota correctness (Cedar token-bucket holds ≤±5% accuracy at 100/s).
- Webhook-queue depth p99 <5s at PSP-burst-storm scenarios.
- Audit-chain seal lag p99 <90s at year-3 volume.

## §9. References

- [`ARCHITECTURE.md`](ARCHITECTURE.md).
- [`cost-budget.md`](cost-budget.md).
- [`multi-region.md`](multi-region.md).
- [`failure-modes.md`](failure-modes.md).
- [`slos/charge-api-latency.openslo.yaml`](slos/charge-api-latency.openslo.yaml).
- Stripe API rate-limits — `stripe.com/docs/rate-limits`.
- Adyen API rate-limits — `docs.adyen.com/development-resources/error-handling`.
- Janert, "Feedback Control for Computer Systems" (queue-theoretic sizing).
- Google SRE Workbook ch. 5 (multi-window burn-rate).
