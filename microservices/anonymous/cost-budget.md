---
doc_class: CostBudget
template_id: TPL-COST-BUDGET
microservice: anonymous
status: Accepted
date: 2026-05-17
owner_team: axis-anonymous + axis-cost-engineering
related_adrs: [ADR-0139, ADR-ANON-0001, ADR-ANON-0002]
review_cadence: monthly
doc_status: published
---

# Cost Budget: anonymous µservice

## Steady-state monthly cost envelope (per cell)

| Component | Baseline / 1M MAU | Notes |
|---|---|---|
| Postgres 16 (posts + votes + attestation-bindings, multi-AZ) | $1,800 | db.m6gd.large × 3 |
| Valkey 8.1 (Redis wire-compat) cluster (feed cache + vote counter, multi-AZ) | $900 | cache.r6g.large × 6 nodes |
| Meilisearch 0.10 (hashtag search) | $400 | 2 × 4 vCPU / 8 GiB |
| Postgres backup (30-day rolling, encrypted) | $250 | (small because retention is short) |
| Object storage (T2 attachments, when enabled; default off) | $200 | per pack avg |
| Cloudflare (TLS + WAF; no Cloudflare Insights/Analytics) | $400 | enterprise tier; pass-through mode |
| Kubernetes compute (REST + workers, ~50 pods baseline) | $2,400 | m6g.large × 12 worker nodes |
| Foundry-runtime classifier inference (T2; ~1B verdicts/month) | $200 | batched 100 per call |
| Observability ingest (per-pack OTLP egress to observability µservice) | $300 | metrics + traces; logs sampled |
| OpenBao secrets HA | $150 | per-cell |
| OPSWAT MetaDefender API (T2 attachments; off by default) | $0–$300 | per scanned attachment; off by default |
| NCMEC reporter (operational; included in compute) | $0 | inline |
| **Total per 1M MAU** | **~$7,000–$7,300/mo** | excluding tenant-overhead allocation |

## Per-pack overlay

| Pack | Multiplier | Driver |
|---|---|---|
| pack-kr | 1.05x | KR PIPC reporting + 통신비밀보호법 disclosure tracking overhead |
| pack-eu | 1.10x | EU DSA transparency report + EU AI Act Art. 50 logging |
| pack-uk | 1.08x | UK OSA Ofcom reporting + IPA 2016 disclosure tracking |
| pack-us | 1.00x | baseline |
| pack-us-healthcare | 1.20x | minimal but stringent; not commonly used for this tier |
| pack-jp | 1.05x | 通신의 비밀 + APPI reporting |
| Others | 1.00x–1.05x | per-pack regulator overhead |

## Per-request cost approximation

| Operation | Marginal cost (¢) | Driver |
|---|---|---|
| Post-create | 0.0008 | Postgres insert + fanout queue |
| Vote-action | 0.0001 | Valkey increment + Postgres flush batched |
| Feed-render (top 50) | 0.0030 | Valkey read + Cedar evaluation |
| Affinity-attestation verify | 0.0200 | BBS+ verify CPU-bound |
| Abuse-classifier inference | 0.0200 | batched; per-post |
| Hard-delete (initial) | 0.0010 | Postgres delete + tombstone |
| Hard-delete (propagation: invalidate caches + search index + fanout) | 0.0050 | per affected replica |
| Legal-process disclosure E2E | $2.00–$50.00 | varies with court-order complexity + reviewer time + chain-of-custody packaging |

## Capacity → cost growth

| MAU | Estimated steady-state cost |
|---|---|
| 100k | $1,200/mo |
| 1M | $7,000/mo |
| 10M | $50,000/mo |
| 100M | $400,000/mo (with horizontal-scale + multi-cell) |

## Cost controls

- HPA caps at 100 REST replicas to bound autoscale runaway.
- Per-tenant storage quota: 10 GB default; admin opt-in raises.
- Per-tenant compute quota: 1k QPS default.
- Retention default 30 days bounds long-tail storage cost.
- Feed cache TTL 5 min reduces Postgres read amplification.
- Foundry-runtime classifier batched 100 to amortize inference cost.

## Budget review cadence

Monthly; council-architecture + axis-cost-engineering + axis-anonymous joint review. Variance > 15% triggers ADR successor-IP.
