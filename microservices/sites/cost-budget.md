---
doc_class: CostBudget
template_id: TPL-COST-BUDGET
microservice: sites
status: Accepted
date: 2026-05-17
owner_team: axis-sites + ops-finops
related_adrs: [ADR-0117, ADR-0131, ADR-SITES-0003]
doc_status: published
---

# Cost Budget — sites µservice

## Purpose

Model unit-economics of sites delivery so axis-sites + ops-finops + council-product can price plans, set per-tenant quotas, and pre-empt cost cliffs. Aligns with `capacity-model.md`.

## Cost-of-goods drivers

| Driver | Cost source | Scale | Notes |
|---|---|---|---|
| Postgres OCPU + storage | OCI Bare Metal | per-cell | 3-replica HA; 16-64 OCPU primary; 2-16 TB block |
| Redis OCPU + RAM | OCI cluster | per-cell | cluster mode; 3-15 shards × 16-64 GB |
| Meilisearch OCPU + RAM | OCI VM | per-cell | 3-12 instances × 16-64 GB |
| S3 storage (published artifacts) | OCI Object Storage | per-cell | per-tenant prefix; WORM for public |
| S3 GET requests | OCI Object Storage | per-cell | only on CDN miss (cache-hit > 95% target) |
| CDN bandwidth | per ADR-SITES-0003 substrate | per-cell | per-pack edge nodes |
| CDN cache invalidations | per ADR-SITES-0003 | per-tenant | signed purge; bounded per-tenant |
| libvips image-optimize compute | K8s worker pods | per-cell | streaming; per-job memory bound |
| ACME (Let's Encrypt) | free (Let's Encrypt) | per-tenant | unlimited; rate-limit per ADR-SITES-0004 |
| LLM inference (T2 AI-page-build) | per foundry-runtime | per-tenant | only on T2 capability use |
| Loro CRDT relay compute | K8s pods | per-cell | per active co-editing session |
| Network egress | OCI | per-pack | CDN absorbs majority; tenant-tier-bound |

## Per-tenant cost envelope (medium tenant: 1k pages, 100k monthly visitors)

| Component | Monthly cost (USD, OCI list price) |
|---|---|
| Postgres footprint share | $8 |
| Redis cache share | $2 |
| Meilisearch share | $4 |
| S3 storage (1 GB/site × ~ 5k assets) | $0.50 |
| S3 GETs (cache-miss; ~ 5% of 100k visitors × 5 pages) | $0.05 |
| CDN bandwidth (~ 10 GB at ~ $0.05/GB) | $0.50 |
| libvips compute share | $1 |
| Loro CRDT relay share | $0.50 |
| OBSERVABILITY share (metrics + traces) | $1 |
| **Total (excluding AI tier)** | **~$17.55** |
| AI tier (T2; 30 generations/mo) | $5 |

Pricing target: starter $29/mo; pro $99/mo; business $299/mo; enterprise per-quote.

## Per-pack overlays

| Pack | Cost adjustment | Driver |
|---|---|---|
| pack-eu | +15% | EU residency premium (Frankfurt OCI), stricter retention, ePrivacy consent infra |
| pack-us-healthcare | +40% | HIPAA-eligible infra; longer audit retention (6y); BAA overhead |
| pack-kr | baseline | OCI ap-seoul-1 |
| pack-jp | +10% | jp-tokyo-1 list price |
| pack-sg | +5% | ap-singapore-1 |
| pack-au | +10% | ap-sydney-1 |
| pack-in | +5% | ap-mumbai-1 |
| pack-br | +15% | sa-saopaulo-1 limited capacity |
| pack-ae / pack-ksa | +20% | Middle-East data centres + Hijri overlay compute |

## Anti-patterns + guardrails

| Anti-pattern | Detection | Guardrail |
|---|---|---|
| Tenant uploads enormous images that explode storage + CDN egress | image-size scan at upload | refuse > 100 MB / > 50MP per ADR-SITES-0007; libvips-pipeline emits responsive variants |
| Tenant publishes a million pages and overwhelms search reindex | publish-queue depth alarm | per-tenant publish-rate-limit |
| Tenant abuses T2 AI-page-build (LLM cost explosion) | daily T2-call counter | per-tenant T2 daily cap; ADR-SITES-0006 |
| Bot/scraper hammers anonymous reads | per-IP rate alarm | Cedar `public-read.cedar` per-IP throttle |
| Cert-renewal storm hits Let's Encrypt rate-limit | ACME renewal cluster alarm | multi-account pool + 30d-pre-expiry renewal window |
| Cross-pack data egress | LEAN `oya-check-cross-pack-replication-prohibition` | refused at PR-time |

## Finops cadence

| Cadence | Action | Owner |
|---|---|---|
| Daily | Per-cell cost report; per-tenant top-10 cost | ops-finops |
| Weekly | Per-pack cost envelope vs budget | ops-finops + axis-sites |
| Monthly | Tenant top-cost outliers → tenant-success review | ops-finops + council-product |
| Quarterly | Re-pricing review (LLM + CDN + S3 storage tier negotiation) | ops-finops |
| Annually | Tenant DPA renewal aligned with cost negotiation | council-privacy + ops-finops |

## References

- ADR-0117: cloud-native infrastructure.
- ADR-0131: per-microservice layout.
- ADR-SITES-0003: CDN substrate.
- ADR-SITES-0006: AI-page-build (LLM-cost-bound).
- ADR-SITES-0007: image pipeline (storage + compute-bound).
- `capacity-model.md`, `multi-region.md`.
- OCI list pricing — `oracle.com/cloud/price-list/`.
- Let's Encrypt rate limits — `letsencrypt.org/docs/rate-limits/`.
