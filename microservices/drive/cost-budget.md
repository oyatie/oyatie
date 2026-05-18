---
doc_class: CostBudget
template_id: TPL-COST-BUDGET
microservice: drive
status: Accepted
date: 2026-05-17
owner_team: axis-drive + finops + ops-sre-reliability
related_adrs: [ADR-0139, ADR-0131, ADR-0133, ADR-DRIVE-0001]
doc_status: published
---

# Cost Budget — drive µservice

## Purpose

Per-tenant + per-cell unit-economic cost envelope for the drive µservice. Used to budget infrastructure spend, alarm on cost regressions, and justify per-tier pricing.

Cost categories tracked: compute (rest + worker pods), storage (object store + Postgres metadata + Redis + Meilisearch + Tika cache), bandwidth (CDN egress + cross-cell replication), per-tenant overhead (KMS calls + audit-chain seal cost), preview render compute, virus + DLP scan compute.

## Per-tenant baseline (medium tenant)

Reference tenant: 1,000 active users, 50TB stored, 1M files, 5GB ingress + 50GB egress per day, ≤ 10k file-list/s, ≤ 200 uploads/s, ≤ 2k downloads/s, ≤ 100 sync delta/s, ≤ 50 preview renders/s. Pack-kr; Garage primary backend.

| Component | Cost driver | Unit cost (2026-05) | Monthly per tenant |
|---|---|---|---|
| Object store (Garage; 3× replication) | $/GB-mo at edge cluster | $0.012/GB-mo | $1,800 (150TB raw at 3×) |
| Object store (cold/archive; SeaweedFS) | $/GB-mo at archive cluster | $0.004/GB-mo | $200 (50TB cold) |
| Postgres (metadata; RLS-isolated; per-tenant logical shard) | per-shard | $90 | $90 |
| Redis cluster (upload session + sync cache; per-tenant prefix) | per-tenant | $25 | $25 |
| Meilisearch (per-tenant index) | per-tenant | $40 | $40 |
| Tika worker (full-text extract; throughput-bound) | shared workload | $0.05 per 1k files | $50 |
| ClamAV virus-scan worker (per scan) | shared workload | $0.002 per scan | $60 (30k scans) |
| OPSWAT MetaDefender (multi-engine; healthcare + EU packs) | per scan | $0.015 per scan | enabled per-pack; $0 for kr default |
| libvips + qpdf + LibreOffice + ffmpeg preview workers | shared workload | $0.01 per render | $300 (30k renders/day → 900k/mo) |
| OpenBao Transit (KMS calls) | per call | $0.000003 / call | $30 (10M calls/mo) |
| Audit-chain seal | per event | $0.000002 / event | $15 (7.5M events/mo) |
| CDN egress (download path) | per GB | $0.020/GB | $30 (1.5TB/mo egress) |
| Cross-cell replication (within pack) | per GB | $0.005/GB | $30 (6TB intra-pack) |
| Rest pod compute (file-store + upload + download + share-link + permissions) | reserved + autoscale | $200 baseline | $200 |
| Worker pod compute (retention + version pruner + WORM scan + sync + DLP) | reserved + autoscale | $100 baseline | $100 |
| **Sub-total per medium tenant** | | | **~$2,970/mo** |
| Tenant-pricing margin (50% gross margin target) | | | **~$4,455/mo billable** |

## Per-cell capacity (50k tenants medium-sized)

| Component | Per-cell scale | Per-cell cost/mo |
|---|---|---|
| Object store (Garage 3× replication) | 7.5EB raw (2.5EB effective) | $90M |
| Object store (cold tier; SeaweedFS) | 2.5EB cold | $10M |
| Postgres cluster (per-tenant logical shards) | 50k shards | $4.5M |
| Redis cluster | per-tenant prefix; 50k tenants | $1.25M |
| Meilisearch cluster | 50k indexes | $2M |
| Tika + ClamAV + OPSWAT + preview workers | shared; HPA on queue depth | $4M-$8M (envelope) |
| OpenBao Transit | shared HSM-backed | $1.5M |
| Audit-chain seal | shared | $0.75M |
| CDN egress | aggregate | $1.5M |
| Rest + worker pod compute | aggregate | $15M |
| **Per-cell total** | | **~$130M/mo at 50k medium tenants** |

## Cost guardrails (alarm thresholds)

| Alarm | Threshold | Action |
|---|---|---|
| Per-tenant object-store cost > 150% of baseline | 4h sustained | open incident; check for quota leak / sync amplification |
| Preview compute > 2× baseline | 1h sustained | open incident; check for preview-storm (large file flood) |
| Virus-scan worker compute > 3× baseline | 1h sustained | open incident; check for malicious-flood / OPSWAT misconfig |
| CDN egress > 200% of baseline | 4h sustained | open incident; check for share-link abuse / data exfil |
| OpenBao Transit calls > 200% of baseline | 1h sustained | open incident; check for key-rotation loop / misconfigured cache |
| Postgres connection pool > 80% | 5m sustained | scale up pool; check for permission-resolver hotspot |

## Cost categories vs reservation

| Category | Reservation | Burstable |
|---|---|---|
| Object store | committed-use (3y reserved) | n/a (object store is contract-based) |
| Postgres | reserved instances | spot for read replicas |
| Redis | reserved cluster | n/a |
| Meilisearch | reserved | n/a |
| Tika / ClamAV / OPSWAT workers | base reserved + HPA spot burst | yes |
| Preview workers | base reserved + HPA spot burst | yes |
| Rest pods | base reserved + HPA spot burst | yes |

## Cost optimisation roadmap

| Action | Target reduction | Owner | Target date |
|---|---|---|---|
| Auto-tier hot → warm → cold based on access age | -20% storage cost | axis-drive + finops | M03 |
| Preview-cache LRU tuning + thumbnail dedup | -15% preview compute | axis-drive | M03 |
| Tika extract dedup via content-address | -25% Tika compute | axis-drive | M04 |
| Garage chunk-dedup via FastCDC content-address | -30% raw storage | axis-drive | M04 |
| Cross-cell replication tuning (eventual within pack) | -20% replication bandwidth | ops-sre-reliability | M05 |

## References

- ADR-0139 (SLO-gated promotion; cost alarms gate promotion).
- ADR-0131 (per-µservice flat layout; per-µservice cost ownership).
- ADR-0133 (industry-best-practice axis-3 cost discipline).
- ADR-DRIVE-0001 (object-storage substrate selection; Garage vs MinIO vs SeaweedFS cost trade-offs).
- `microservices/drive/capacity-model.md` — drives the unit math.
- `microservices/drive/multi-region.md` — cross-region replication cost.
- AWS S3 + Garage + MinIO + SeaweedFS public pricing (2026-05).
