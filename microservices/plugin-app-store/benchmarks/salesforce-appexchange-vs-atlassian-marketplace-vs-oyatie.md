---
doc_class: Benchmark
microservice: plugin-app-store
benchmark_date: 2026-05-20
related_adrs: [ADR-0316, ADR-0249, ADR-0251]
doc_status: published
---

# Benchmarks — oyatie plugin-app-store vs Salesforce AppExchange vs Atlassian Marketplace vs GitHub Marketplace vs ServiceNow Store vs HubSpot Marketplace

Workloads measured: (a) listing-search latency, (b) install throughput, (c) AI-curated recommendation quality, (d) revenue-share + payment processing, (e) annual TCO at 100k listings + 50M installs/month.

Hardware (oyatie on-prem tenant_class paid tier): 12× marketplace API pods (16 vCPU AMD EPYC 9354P, 64 GiB DDR5), 6× publish workers, 6× security scanners, 2× GPU for AI recommendations, PostgreSQL + Elasticsearch + SeaweedFS.

## Workload (a) — listing search latency

| Engine | p99 (ms; simple query) | p99 (ms; faceted) |
|---|---:|---:|
| oyatie (tenant_class paid) | 200 | 480 |
| oyatie (tenant_class paid) | 100 | 240 |
| Salesforce AppExchange | 380 | 720 |
| Atlassian Marketplace | 280 | 580 |
| GitHub Marketplace | 220 | 460 |
| ServiceNow Store | 480 | 920 |
| HubSpot Marketplace | 320 | 680 |
| OpenAI GPT Store | 280 | n/a |
| Hugging Face Hub | 180 | 380 |

Reading: oyatie paid leads + Hugging Face Hub is competitive (open-source ES tuning). The major-vendor marketplaces are slower due to multi-tenant search queue contention.

## Workload (b) — install throughput

| Engine | Installs/sec sustained |
|---|---:|
| oyatie (tenant_class paid) | 200 |
| oyatie (tenant_class paid) | 1 200 |
| Salesforce AppExchange | 600 |
| Atlassian Marketplace | 400 |
| GitHub Marketplace | 800 |
| ServiceNow Store | 300 |
| HubSpot Marketplace | 350 |

Reading: oyatie paid leads. The parallelised artifact-download path + CDN-fronted distribution gives the edge. GitHub Marketplace is competitive due to GitHub's CDN scale.

## Workload (c) — AI-curated recommendation quality (precision @ 10)

| Engine | Precision @ 10 | Notes |
|---|---:|---|
| oyatie (tenant_class paid, Llama-3.1-70B + collaborative filtering) | 0.71 | Tenant-fine-tuned |
| Salesforce AppExchange (Einstein Recommender) | 0.62 | Tenant-aware |
| Atlassian Marketplace | 0.54 | Basic CF |
| GitHub Marketplace | 0.48 | Tag-based |
| ServiceNow Store | 0.51 | |
| HubSpot Marketplace | 0.49 | |
| OpenAI GPT Store | n/a | Browse-only |
| Hugging Face Hub | 0.56 | Trending-based |

Reading: oyatie paid leads. Tenant-fine-tuning is the key — recommendations reflect what tenants similar to you have installed AND found useful (retention-weighted, not just install-count).

## Workload (d) — payment processing latency (Stripe Checkout → subscription active)

| Engine | p99 (s) |
|---|---:|
| oyatie (tenant_class paid, Stripe Connect) | 3.8 |
| oyatie (tenant_class paid, multi-PSP failover) | 2.4 |
| Salesforce AppExchange (Salesforce CPQ) | 5.2 |
| Atlassian Marketplace (Atlassian Billing) | 4.4 |
| GitHub Marketplace (GitHub Billing) | 6.0 |
| ServiceNow Store | 7.2 |
| HubSpot Marketplace | 4.8 |

Reading: oyatie paid leads. Multi-PSP failover (Stripe primary, Adyen secondary) reduces tail latency.

## Workload (e) — annual TCO at 100k listings + 50M installs/month

| Platform | Hardware (USD) | Licence + fees (USD) | Tax + PSP fees (USD) | Ops (USD) | Total (USD) |
|---:|---:|---:|---:|---:|---:|
| oyatie plugin-app-store (tenant_class paid on-prem) | 740 000 | 0 | 120 000 (Avalara + Stripe platform fees) | 372 000 (3 SRE × 0.4 FTE) | 1 232 000 |
| Salesforce AppExchange (Independent ISV) | 0 | (revenue-share-only model; ISVs pay no platform fee; Salesforce gets 15-25 % per listing) | (bundled) | n/a | n/a (we operate the marketplace; not pay-to-operate) |
| Atlassian Marketplace (vendor operator) | n/a (not operable by 3rd party) | n/a | n/a | n/a | n/a |
| GitHub Marketplace | n/a (not operable) | n/a | n/a | n/a | n/a |
| ServiceNow Store | n/a (not operable) | n/a | n/a | n/a | n/a |
| HubSpot Marketplace | n/a (not operable) | n/a | n/a | n/a | n/a |
| OpenAI GPT Store | n/a (not operable) | n/a | n/a | n/a | n/a |
| Hugging Face Hub (self-host alternative) | 320 000 | 0 (HF Spaces) | 0 | 248 000 | 568 000 |

Reading: most competitors are not operable by third parties — they are vendor-controlled. oyatie's value proposition is "host your own multi-category marketplace" — relevant for enterprises wanting an internal app store or tenants wanting to build their own ecosystem on the oyatie substrate.

## Workload (f) — sovereign-pack feature parity (oyatie-exclusive)

"Operate a KR-PIPA-compliant marketplace with pack-resident listings + dual-control publish + KR-PIPA pack overlay."

| Engine | Support |
|---|---|
| oyatie (compliance_pack-bound paid) | Yes |
| All competitors | No (vendor-controlled; no sovereign-residency option) |

Unique differentiator.

## Reproducibility

Benchmark harness at `benchmarks/plugin-app-store/`. Re-run weekly.
