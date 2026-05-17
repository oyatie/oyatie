---
doc_class: MultiRegion
template_id: TPL-MULTI-REGION
microservice: community
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-community + ops-sre
related_adrs: [ADR-0056, ADR-0105, ADR-0117, ADR-0126, ADR-0131]
doc_status: published
---

# Multi-Region: community µservice

## Topology

Per-region deployment. Each tenant pinned to one region by `jurisdiction_code` per `policy/data-residency.md`. No global cluster — data residency obligations forbid it.

### Regions (initial)

| Region | Code | Pack overlay | Status |
|---|---|---|---|
| US East (Virginia) | us-east-1 | pack-us, pack-us-healthcare | Live |
| US West (Oregon) | us-west-2 | pack-us, pack-us-healthcare | Live |
| EU Central (Frankfurt) | eu-central-1 | pack-eu | Live |
| EU West (Ireland) | eu-west-1 | pack-eu | Live |
| APAC Tokyo | ap-northeast-1 | pack-jp | Live |
| APAC Seoul | ap-northeast-2 | pack-kr | Live |
| APAC Singapore | ap-southeast-1 | pack-sg | Pilot |
| APAC Sydney | ap-southeast-2 | pack-au | Pilot |
| South America São Paulo | sa-east-1 | pack-br | Backlog |
| Middle East UAE | me-central-1 | pack-ae | Backlog |
| KSA Riyadh | me-south-1 | pack-ksa | Backlog |
| India Mumbai | ap-south-1 | pack-in | Backlog |

## Per-Region Stack

Each region runs full stack:

- Postgres (Citus + Patroni) primary + sync replica
- Elasticsearch cluster (3 master + N data)
- Redis cluster
- S3 bucket per region (KB attachments)
- ClamAV inline scanner
- Worker fleet (reindex / guardrails-bridge / audit-chain-seal)
- REST + SDK gateway
- OpenBao secret backend (regional)

## Failover Within Region

- Postgres: Patroni auto-promote replica on primary failure (RTO 60 s).
- Elasticsearch: replica-shard promotion (auto, RTO 30 s).
- Redis: cluster rebalance on node loss (RTO 60 s).
- S3: cross-AZ already (provider-managed).

## Cross-Region Replication

**Default: OFF.** Tenant opt-in only.

When opt-in (`tenant.cross_region_replication == true`):

| Component | Mechanism | Async lag |
|---|---|---|
| Postgres | Logical replication slot | ≤ 60 s |
| Elasticsearch | CCR (cross-cluster replication) | ≤ 60 s |
| S3 | Cross-Region Replication (CRR) | ≤ 15 min |
| Redis | Not replicated cross-region (cache only; rebuilt) | — |
| Audit-chain | Per-region seal; cross-region witness chain | ≤ 60 s |

Cross-border transfer rules per `policy/data-residency.md`. Cross-region replication for KR / UAE / KSA / IN tenants requires explicit DSR-equivalent consent.

## Regional Failover

If a region is unavailable:

- Default tenants (no cross-region opt-in): service unavailable in that region until restoration.
- Opt-in tenants: route to secondary region via DNS failover (`active-passive`); RTO 30 min; RPO ≤ 60 s.
- Tenant must accept the cross-border legal posture before opt-in.

## Promotion Pipeline per Region

Per ADR-0130, each region's promotion-eligibility verdict is independent. A community release pointer at `release/community/<region>/<env>` advances only when the per-region eligibility is GREEN.

| Env | Ref | Promotion gate |
|---|---|---|
| dev | `release/community/<region>/dev` | Smoke + unit |
| staging | `release/community/<region>/staging` | Burn-rate green over 6 h window |
| production | `release/community/<region>/production` | Burn-rate green over 3 d window + manual two-eyes for first region; subsequent regions auto |

Canary cohort weighting per region: 1 % → 10 % → 50 % → 100 %.

## DNS + Routing

- Per-region public hostname: `community.<region>.oyatie.io`.
- Tenant-pinned hostname: `<tenant>.community.oyatie.io` (CNAME to tenant's region).
- Anycast for read paths when tenant opt-in.

## Capacity per Region

Each region sized to handle 110 % of its tenant population's peak (head-room for regional failover ingress on opt-in tenants).

## Drill Cadence

- **Quarterly**: regional failover drill on staging (real flip).
- **Annually**: regional outage simulation in production (cell-by-cell brownout test).

## Cost

Per-region overhead vs. single-region: ~ 2.2× (1× per region; HA replicas; CCR for opt-in tenants). Tenant-borne via per-region pricing tier.
