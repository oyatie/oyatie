---
doc_class: CostBudget
title: Cost Budget + FinOps Posture
microservice: governance
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-finops + axis-foundry + ops-sre-reliability
deciders: ops-finops, axis-foundry, ops-sre-reliability, council-architecture
related_adrs: [ADR-0117, ADR-0131, ADR-0133]
related_artifacts:
  - microservices/governance/capacity-model.md
  - microservices/governance/multi-region.md
review_cadence: monthly + on every capacity-model revision
doc_status: published
---

# Cost Budget + FinOps Posture (governance µservice)

## Purpose

Track the governance µservice's monthly cloud cost across compute, storage, network, and external SaaS line items, per Layer-A + Layer-B component, per pack region; surface budget breach via the `oya-check-cost-budget` LEAN lane. Numbers cite OCI public pricing (2026-05-17) and the per-PR lane volumetric forecasts from `capacity-model.md`; verify-at-deploy markers called out where vendor pricing may have moved.

## Cost Categories

| Category | What | Reference |
|---|---|---|
| GitHub Actions minutes | ~50 lanes × per-PR runtime × per-month PR volume; ARC pool for self-hosted runners | `docs.github.com/en/billing/managing-billing-for-github-actions/about-billing-for-github-actions` + ARC self-hosted compute below |
| Compute (VM.Standard / OKE node) | Lane-runtime worker; policy-engine; evidence-emitter; aggregation-indexer; Postgres; ARC runner pool | `oracle.com/cloud/compute/pricing/` |
| Block storage (PV) | Postgres primary + replicas | `oracle.com/cloud/storage/block-volume/pricing/` |
| Object storage (S3-compatible) | Evidence blobs (7y compliance-mode object-lock) | `oracle.com/cloud/storage/object-storage/pricing/` |
| Network egress | Cross-pack auditor reads; baseline-pin HTTPS fetches (quarterly) | `oracle.com/cloud/networking/pricing/` |
| KMS | Per-pack KMS keyring for SSE + Ed25519 signing | `oracle.com/security/key-management/pricing/` |
| Audit-chain seal | upstream µservice; charged via inter-µservice transfer | upstream cost-budget |

## Per-PR cost decomposition (median PR, XS tier, pack-kr)

Per `capacity-model.md` §"Per-PR median path".

| Line item | Per-PR cost | Notes |
|---|---|---|
| ARC runner compute (50 lanes × avg 3s × 1 vCPU) | $0.0006 | 2.5 vCPU-min × $0.0144/vCPU-hour |
| Lane-runtime worker share | $0.0001 | amortized across PRs/min |
| Postgres write IO (50 finding rows max + 50 lane-run rows) | $0.0002 | $0.20 per 1M IOPS |
| S3 evidence writes (~50 KB median blob × 50 lanes) | $0.00005 | $0.0085/GB ingest + per-object cost |
| Audit-chain seal | $0.0001 | upstream µservice charges |
| KMS sign operations (50 Ed25519 seals) | $0.00015 | $0.03 per 10k operations |
| Network egress (intra-region) | ~$0 | intra-region free in OCI |
| **Total per-PR median** | **~$0.0012** | rounds to ~$0.0015 with overhead |

## Per-Component Monthly Cost (XS tier, single pack-kr region, M01 launch; 1k PRs/month)

| Component | Replicas × instance-type | Monthly compute | Monthly storage | Monthly total |
|---|---|---|---|---|
| ARC runner pool (per Helm chart `lane-runner-pool/values.yaml`) | 8 standby + autoscale to 50 × VM.Standard.E4 4-core | $580 baseline + $580 burst | – | $580–$1160 |
| `oya-governance-lane-runtime-worker` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `oya-governance-lane-runtime-rest` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `oya-governance-lane-runtime-app` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `oya-governance-policy-engine-worker` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `oya-governance-policy-engine-rest` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `oya-governance-evidence-emitter-worker` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `oya-governance-evidence-emitter-rest` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `oya-governance-aggregation-indexer-worker` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| Postgres (HA primary + 2 replicas; per `iac/helm/postgres/values.yaml`) | 3 × VM.Standard.E4 4-core | $435 | $50 PV × 3 | $585 |
| Postgres WAL-G backups → S3 | – | – | $20 (60 GB) | $20 |
| Evidence S3 bucket (hot tier; 7y object-lock) | – | – | $85 (10 TB hot) + $25 (3 TB cold) | $110 |
| KMS keyring + signing operations | – | $10 | – | $10 |
| Network egress (intra-region; quarterly baseline-pin fetch) | – | $5 | – | $5 |
| **XS tier total per pack region** | | **~$1820** | **~$180** | **~$2000 / month** |

Verify-at-deploy: OCI pricing changes; reconfirm against `oracle.com/cloud/pricing/` at deploy time. Buffer 15% for OCI rate increases + 20% for actual-vs-forecast.

## Per-Scale-Tier Forecast

| Scale tier | Monthly PR volume | Monthly cost per pack region | Notes |
|---|---|---|---|
| XS (M01 launch) | 1k PRs | ~$2000 | active: pack-kr |
| S | 10k PRs | ~$6.5k | active: pack-kr + pack-eu + pack-us |
| M | 50k PRs | ~$25k | 5 active packs |
| L | 100k PRs | ~$120k | all 11 packs; multi-region per pack |

## Per-Pack Multipliers

- **DR pair packs** (pack-eu, pack-us, pack-au, pack-in, pack-br, pack-ae, pack-ksa): 1.0× primary + 0.5× warm-standby (per `multi-region.md`).
- **HIPAA pack** (pack-us-healthcare): 1.5× base (extended 6y retention per HIPAA §164.316(b)(2) + dedicated HIPAA-eligible region; isolated Postgres + S3).
- **KR-FSS-regulated** tenants in pack-kr: 1.15× base (KR commercial code 5y retention).
- **Single-region packs** (pack-kr, pack-jp, pack-sg): 1.0× base (no DR multiplier).

## Budget + Alert Thresholds

| Metric | Threshold | Action |
|---|---|---|
| Monthly cost (per pack region) | within 90% of forecast | normal |
| 90% < cost < 110% | yellow alert | FinOps + ops-sre-reliability review |
| 110% < cost < 130% | orange alert | FinOps + leadership review; review autoscale + capacity-model |
| cost > 130% | red alert; budget breach incident | engage ops-finops + axis-foundry; consider PR-bomb mitigation (T-D-01) |
| Per-PR cost projection (95p) | ≤ 5× median per-PR cost | normal |
| Per-PR cost > 10× median (single PR) | yellow; investigate pathological PR (diff size, lane churn) | per-µservice review |
| Evidence S3 growth/month | within forecast | normal |
| Evidence S3 growth > 2× forecast in a quarter | yellow; cardinality discipline review (per-lane evidence trimming) | per-axis review |
| ARC pool utilization | < 80% | normal |
| ARC pool utilization > 90% sustained 1h | orange; scale-out trigger | autoscaling extends max-replicas |

## FinOps SLI

| SLI | Target | Burn-rate alert |
|---|---|---|
| Monthly cost / N_PRs (unit-economic) | within 5% of forecast | 6× burn over 6h |
| Storage growth / day (avg over last 7d) | within forecast | 14.4× burn over 1h (catches runaway-evidence-blob) |
| Per-runner $/min spot ratio | ≥ 60% spot for non-critical lane runners | informational |

## Cost-Optimisation Levers

| Lever | Estimated saving | Trade-off |
|---|---|---|
| Spot-instance ARC pool for non-critical lanes (WARN-severity) | 30–50% ARC compute | Spot eviction → re-queue (acceptable for WARN) |
| Per-µservice lane-subset selection (PRD Open Q4) | 30–40% per-PR runtime | Coverage loss if subset rule incorrect |
| Evidence-blob compression (zstd vs no-compression) | 40–60% S3 storage | CPU cost on emit |
| Postgres read replicas: 1 vs 2 (M01 launch tier) | $100/month per replica | Slightly higher p99 on replay-query |
| ARC max-replicas tuning (current cap 200) | varies | DoS risk if too tight |
| OCI committed-use discounts (1y / 3y) | 20–40% compute | Vendor lock-in window |
| Per-lane runtime budget tightening (60s → 30s p99) | 10–20% compute | Some lanes need 60s; per-lane configurable |
| Aggregation-index incremental regen (vs full) | 80–90% regen compute | Already in plan per IP-010 |

## Verification

- `buck2 build //:quality-lane-registry-authority-check # lane=cost-budget --microservice governance` — exit 0; current spend within 110%.
- Monthly FinOps review: actual vs forecast; lever decisions logged.
- Quarterly: capacity-model + cost-budget refresh; re-run formulas with current data.

## References

- `microservices/governance/capacity-model.md`.
- `microservices/governance/multi-region.md`.
- `microservices/governance/policy/data-residency.md` (per-pack retention multipliers).
- OCI pricing — `oracle.com/cloud/pricing/`.
- GitHub Actions billing — `docs.github.com/en/billing/managing-billing-for-github-actions/`.
- FinOps Foundation framework — `finops.org`.
- `microservices/observability/cost-budget.md` (shape reference).
