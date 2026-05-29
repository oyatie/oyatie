---
doc_class: Benchmark
microservice: payments
benchmark_date: 2026-05-20
related_adrs: [ADR-0263, ADR-0251, ADR-0316]
doc_status: published
---

# Benchmarks — oyatie payments vs Stripe vs Adyen vs Checkout.com vs Braintree

Workloads measured: (a) sustained authorization throughput, (b) authorization latency, (c) settlement latency (auth → funds available), (d) chargeback workflow (open → submitted), (e) merchant authorization-rate uplift via multi-PSP routing, (f) annual TCO at 1 M auth/day + 100 currencies + 30 % cross-border mix.

Hardware (oyatie on-prem paid tenant_class): 6× payment-api nodes (16 vCPU EPYC 9354P, 64 GiB DDR5, 1 TiB NVMe), PostgreSQL 16.6 primary + 2 replicas (16 vCPU, 64 GiB, 3.84 TiB NVMe), Valkey 8.x 3-node, 25 GbE leaf-spine, MTU 9000. PSP outbound via dedicated TLS 1.3 keep-alive pool to api.stripe.com / checkout-api.adyen.com / api.checkout.com / api.braintreegateway.com.

## Workload (a) — sustained authorization throughput (auth requests/sec)

| Platform | Sustained (auth/sec) | Burst (auth/sec, ≤ 60 s) | Notes |
|---|---:|---:|---|
| oyatie payments (Paid, single-cell) | 5 000 | 20 000 | Multi-PSP fan-out; dominated by PSP TLS handshake reuse |
| oyatie payments (Paid, multi-AZ) | 25 000 | 100 000 | 3-AZ active-active |
| Stripe (direct) | 12 000 (published cap; higher with negotiated tier) | (no published burst cap) | Stripe Rate Limits docs |
| Adyen (direct) | (no published cap; effectively unbounded per Adyen platform) | (no published burst cap) | |
| Checkout.com (direct) | 5 000 (published default cap; higher per contract) | (no published burst cap) | |
| Braintree (direct) | (no published cap) | (no published burst cap) | |

Reading: When a tenant uses oyatie payments at Paid, they get a multi-PSP fan-out automatically. Direct integration to any single PSP is faster (no fan-out overhead) but lacks multi-PSP routing. paid tenant_class matches per-PSP throughput at the same envelope thanks to active-active.

## Workload (b) — authorization latency

| Platform | p50 (ms) | p99 (ms) | Component breakdown |
|---|---:|---:|---|
| oyatie payments (Paid, Stripe-routed) | 412 | 880 | 184 ms idempotency + Cedar; 228 ms Stripe RTT |
| oyatie payments (Paid, Adyen-routed) | 320 | 720 | 184 ms internal; 136 ms Adyen RTT |
| oyatie payments (Paid, multi-AZ) | 386 | 720 | 168 ms internal (AZ-local PG); 218 ms PSP RTT |
| Stripe (direct) | 228 | 540 | Stripe own latency |
| Adyen (direct) | 136 | 412 | Adyen own latency |
| Checkout.com (direct) | 198 | 480 | Checkout.com own latency |
| Braintree (direct) | 274 | 620 | Braintree own latency |

Reading: oyatie's overhead vs direct-PSP is ~ 180-200 ms (idempotency-key lookup + Cedar evaluation + ledger pre-post + audit-chain emit). The trade-offs we pay this for: (1) idempotency replay safety, (2) multi-PSP failover, (3) ledger consistency, (4) audit-chain non-repudiation. For most merchants the 180 ms is acceptable; for HFT-adjacent flows we offer Paid's "fast-path" Cedar pre-compile that drops to ~ 100 ms overhead.

## Workload (c) — settlement latency (auth → funds available to merchant)

| Platform | Median rail | T+0 supported? | Cross-border T+? |
|---|---|---|---|
| oyatie payments (Paid) | T+1 ACH; T+0 same-day ACH @ 1% fee | Yes (T+0 SDA); RTGS at Paid | T+2 - T+5 (cross-border SWIFT) |
| oyatie payments (Paid) | T+0 RTGS (Fedwire/TARGET2/CHAPS/BOK-Wire) | Yes (sub-5-min via RTGS for eligible pairs) | T+0 via SWIFT gpi cover (≤ 4 h p99) |
| Stripe | T+2 (US standard); T+0 instant-payouts @ 1% fee | Yes (Instant Payouts) | T+5 (international) |
| Adyen | T+1 - T+3 | Yes (Adyen Instant Payouts in EU) | T+2 - T+5 |
| Checkout.com | T+1 - T+3 | No (no instant-payout product) | T+3 - T+5 |
| Braintree | T+1 - T+3 | No | T+3 - T+5 |

Reading: oyatie Paid leads on cross-border settlement via SWIFT gpi cover; the others all rely on correspondent-bank chains that take days. T+0 RTGS support is unique at Paid among the comparator set.

## Workload (d) — chargeback workflow (open → evidence submitted, p99 wall-clock)

| Platform | Workflow latency (p99) | Win rate (fleet-average) |
|---|---:|---:|
| oyatie payments (Paid) | 18 h | 48 % |
| oyatie payments (Paid, with automated evidence assembly) | 6 h | 54 % |
| Stripe Disputes (direct merchant) | (merchant-driven; varies) | 35-42 % industry avg |
| Adyen Disputes (direct merchant) | (merchant-driven; varies) | 38-45 % industry avg |

Reading: our paid tenant_class matches industry leaders; Paid's automated 3DS2 evidence + shipping-document assembly pushes win rate above the field by 6-12 %.

## Workload (e) — merchant authorization-rate uplift via multi-PSP routing

Internal A/B over Q1-2026 on 5 enterprise tenants (≥ 100k auth/month each):

| Tenant segment | Single-PSP auth-rate | oyatie multi-PSP auth-rate | Uplift |
|---|---:|---:|---:|
| US-only B2C | 93.4 % | 95.1 % | +1.7 % |
| US + EU B2C | 91.2 % | 94.8 % | +3.6 % |
| Global B2C (incl. APAC, LATAM) | 88.7 % | 93.4 % | +4.7 % |
| US B2B | 96.1 % | 97.0 % | +0.9 % |
| Subscription / recurring | 90.4 % | 94.2 % | +3.8 % |

Reading: multi-PSP routing's biggest gains come from cross-region traffic where regional PSP optimization differs. US-only flows see ~ 1.5-2 % uplift; global flows see 4-5 %.

## Workload (f) — annual TCO at 1 M auth/day, 100 currencies, 30 % cross-border

Assumptions: 1 M auth/day = ~ 12 auth/sec average, ~ 100 auth/sec peak. 30 % cross-border. Average ticket size $50 USD-equivalent.

| Platform | Per-tx fee | FX margin | Chargeback cost | Hardware/licence | Ops | Total (USD/year) |
|---:|---:|---:|---:|---:|---:|---:|
| oyatie payments (Paid, self-routed) | 4.7M (passed-through PSP fees) | 1.4M revenue (40 bps margin returned to merchant) | 360k | 480k (hardware + 3 PSP integrations) | 372k (3 SRE × 0.4 FTE) | 4.5M net (after FX margin recapture) |
| Stripe (direct) | 4.7M (2.9% + 30¢ blended for US; varies international) | 0 (Stripe captures the margin) | 360k | 0 (managed) | 124k (1 SRE × 0.2 FTE) | 5.2M |
| Adyen (direct) | 4.4M (lower interchange-optimized) | 0 (Adyen captures the margin) | 360k | 0 (managed) | 124k | 4.9M |
| Stripe (via Stripe Treasury, RTGS-eligible) | 4.7M | 0 | 360k | 240k (Stripe Treasury entitlement) | 124k | 5.4M |
| Adyen + custom multi-PSP routing (DIY) | 4.4M | 0.8M (DIY FX margin) | 360k | 1.2M (engineering 3 PSP integrations) | 620k (5 SRE × 0.4 FTE) | 5.8M |

Reading: oyatie Paid is net-cheaper than direct-Stripe by ~ 700 k USD/year on a 1M-auth/day tenant when FX margin recapture is counted. The bigger win is the SaaS-like cost shape: oyatie's overhead is mostly amortised hardware + ops; PSP fees pass through 1:1.

Caveats:

- These numbers assume 40 bps tenant-configured FX margin. Tenants who set higher margins (60-100 bps) recapture more; tenants in jurisdictions that mandate margin caps (some EU pairs under CBPR2) recapture less.
- Stripe / Adyen / Checkout fees pass through at oyatie's negotiated wholesale rate; the listed per-tx fee is the published merchant rate.
- Ops cost scales with the number of PSP integrations active. Most enterprise tenants run 2-3 PSPs; the listed 3-SRE × 0.4 FTE assumes 4 PSPs.

## Reproducibility

The benchmark harness is at `benchmarks/paymentsbench/`. Run with:

```sh
cargo run -p oya-dev-cli -- benchmarks payments \
    --workload sustained-auth-1m-per-day \
    --tier paid \
    --psp-mix stripe:0.5,adyen:0.3,checkout:0.2 \
    --output ./benchmark-results.json
```

Cloud-PSP comparators require valid sandbox keys for each PSP. Results live at `benchmarks/results/payments/<date>.csv` and are re-run weekly to detect drift in either direction.
