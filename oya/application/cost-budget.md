---
doc_class: CostBudget
title: Cost Budget + FinOps Posture
microservice: application
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-finops + axis-application + ops-sre-reliability
deciders: ops-finops, axis-application, ops-sre-reliability, council-architecture
related_adrs: [ADR-0117, ADR-0123, ADR-0131]
related_artifacts:
  - microservices/application/capacity-model.md
  - microservices/application/multi-region.md
review_cadence: monthly + on every capacity-model revision
doc_status: published
---

# Cost Budget + FinOps Posture (application µservice)

## Purpose

Track the Application Shell's monthly cloud cost across CDN, compute,
session store, Postgres, and ancillary services. Surface budget breach via
the `oya-check-cost-budget` LEAN lane. Numbers cite OCI public pricing
(2026-05-17); Cloudflare overlay cited where used. Verify-at-deploy markers
called out where vendor pricing may have moved.

## Cost Categories

| Category | What | Pricing reference |
|---|---|---|
| CDN (per-pack OCI CDN + optional Cloudflare overlay) | Static asset distribution (WASM + CSS + fonts); per-tenant shell HTML origin shield | OCI CDN `oracle.com/cloud/networking/cdn/pricing/`; Cloudflare Workers/CDN `cloudflare.com/plans/` |
| Compute (OKE node pool) | shell-routing + auth-gateway + module-loader + frontend-bundle-serve REST + worker pods | `oracle.com/cloud/compute/pricing/` |
| Block storage (PV) | Postgres data volume; admin / audit tables | `oracle.com/cloud/storage/block-volume/pricing/` |
| Object storage | Bundle archive (versioned); audit log cold-tier | `oracle.com/cloud/storage/object-storage/pricing/` |
| Postgres-flex (Citus) | Tenant-partitioned shell-state DB | `oracle.com/database/postgresql/pricing/` |
| Valkey (Sentinel/Cluster) | Session store | self-hosted on OKE; node cost only |
| Network egress | CDN origin pull; cross-AZ traffic; auditor exports | `oracle.com/cloud/networking/pricing/` |
| KMS | Per-pack signing keys (Ed25519); session HMAC keys | `oracle.com/security/key-management/pricing/` |
| Load balancer | Per-pack Istio gateway + public ingress LB | `oracle.com/cloud/networking/load-balancing/pricing/` |
| Observability-on-application | OTel ingest + per-app SLO ingest into observability µservice | recursive (charged in observability) |

## Per-Component Monthly Cost (XS tier, single pack-kr, M03 launch)

Per `capacity-model.md` worked example: 20 tenants × 5 000 average concurrent
sessions × 50 routes/user/day × Application Shell footprint.

| Component | Replicas × instance | Compute / mo | Storage / mo | Network / mo | Total / mo |
|---|---|---|---|---|---|
| shell-routing-rest | 4 × VM.Standard.E4 4-core | $290 | – | – | $290 |
| tenant-context-rest | 2 × VM.Standard.E4 2-core | $72 | – | – | $72 |
| auth-gateway-rest | 4 × VM.Standard.E4 4-core | $290 | – | – | $290 |
| auth-gateway-worker | 2 × VM.Standard.E4 2-core | $72 | – | – | $72 |
| module-loader-rest | 2 × VM.Standard.E4 2-core | $72 | – | – | $72 |
| frontend-bundle-serve-worker (CDN purge consumer) | 2 × VM.Standard.E4 2-core | $72 | – | – | $72 |
| Composition-root `*-app` binaries | 2 × VM.Standard.E4 4-core | $145 | – | – | $145 |
| Postgres + Citus (3-node HA) | 3 × VM.Standard.E4 8-core | $435 | $250 PV | – | $685 |
| Valkey Cluster (3 master + 3 replica) | 6 × VM.Standard.E4 2-core | $216 | $30 PV (AOF/RDB) | – | $246 |
| Object storage (bundle archive + audit cold) | – | – | $40 hot (200 GB) + $20 cold (5 TB) | – | $60 |
| KMS keys (per-pack Ed25519 + session HMAC) | – | – | – | – | $15 |
| OCI CDN (pack-kr POPs) | – | – | – | $200 egress (~1.5 TB) | $200 |
| Cloudflare overlay (n/a pack-kr; placeholder for pack-eu activation) | – | – | – | – | $0 (KR) / ~$200 (EU when activated) |
| Load balancer (per-pack ingress) | – | – | – | $30 | $30 |
| OTel egress to observability µservice (intra-cluster) | – | – | – | $10 | $10 |
| **Total pack-kr XS** | – | **$1664** | **$340** | **$240** | **~$2260 / mo** |

Note: pack-kr is single-region (no DR pair); DR-pair packs roughly 1.6 ×
the XS cost.

## Per-Tenant Unit Economics

| Tier | Tenants per pack | Avg concurrent sessions/tenant | Cost per tenant per mo |
|---|---|---|---|
| XS (M03 launch) | 20 | 5 000 | ~$113 |
| S | 100 | 5 000 | ~$50 |
| M | 1 000 | 5 000 | ~$25 |
| L (scale-out triggers) | 5 000 | 5 000 | ~$15 |

Cost-per-tenant drops as fixed-cost components (Postgres minimum, KMS,
ingress LB) amortise. Verify with `oya gate validate unit-economics
--ms application`.

## Budget Alarms

| Alarm | Threshold | Action |
|---|---|---|
| Monthly compute > forecast | >+20% | ops-finops review; right-size HPA min |
| CDN egress > forecast | >+50% | investigate cache-hit ratio; bundle size review |
| Postgres storage > forecast | >+30% | retention review; archive cold tier |
| Valkey memory > forecast | >+20% | eviction policy review; session TTL audit |
| KMS ops > forecast | >+100% | investigate key-rotation cadence |

## Reserved capacity vs. on-demand

- OKE node pool: 70 % reserved (1-year commit) + 30 % on-demand (burst).
- Postgres: reserved (production-tier).
- Valkey: reserved (production-tier).
- CDN: on-demand (egress-billed).

Verify-at-deploy: OCI pricing pages change; re-validate via
`oya-cost-budget-vendor-pricing-refresh` quarterly.

## References

- ADR-0117 packs.
- `microservices/application/capacity-model.md`.
- `microservices/observability/cost-budget.md` (precedent + format).
