---
doc_class: CostBudget
title: Cost Budget + FinOps Posture
microservice: cloud-iac
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-finops + axis-cloud-iac + ops-sre-reliability
deciders: ops-finops, axis-cloud-iac, ops-sre-reliability, architecture-governance
related_adrs: [ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/cloud-iac/capacity-model.md
  - microservices/cloud-iac/multi-region.md
review_cadence: monthly + on every capacity-model revision
doc_status: published
---

# Cost Budget + FinOps Posture (cloud-iac µservice)

## Purpose

Track cloud-iac's monthly cloud cost across infrastructure (compute + storage + network), per Layer-A + Layer-B component, per pack region; surface budget breach via the `oya-check-cost-budget` LEAN lane. Numbers cite OCI public pricing (2026-05-17); verify-at-deploy markers where vendor pricing may have moved.

## Cost Categories

| Category | What | OCI pricing reference |
|---|---|---|
| Compute (VM.Standard / OKE node) | ArgoCD / Flux / OpenTofu / Helm-controller / Kustomize-controller / Postgres / oyatie-Layer-B workers | `oracle.com/cloud/compute/pricing/` |
| Object storage (S3-compatible) | OpenTofu state buckets per pack; iac-state-index backups | `oracle.com/cloud/storage/object-storage/pricing/` |
| Block storage (PV) | Postgres iac-state-index data volumes; Sigstore Rekor cache | `oracle.com/cloud/storage/block-volume/pricing/` |
| Network egress | Pulls from upstream chart registries; per-pack cluster API traffic | `oracle.com/cloud/networking/pricing/` |
| KMS | Per-pack KMS keyring for state-encryption + chart-signing | `oracle.com/security/key-management/pricing/` |
| Load balancer | Per-pack ArgoCD UI + iac-registry-rest gateway | `oracle.com/cloud/networking/load-balancing/pricing/` |
| Substrate-on-substrate | cloud-iac's own SLO ingest cost (small; inherits from observability) | recursive |

## Per-Component Monthly Cost (XS tier, single pack-kr region, M01 launch)

Per `capacity-model.md` §"Worked example: oyatie XS tier (M01 launch; ~36 microservices pack-kr-only)".

| Component | Replicas × instance-type | Monthly compute | Monthly storage | Monthly total |
|---|---|---|---|---|
| ArgoCD application-controller | 3 × VM.Standard.E4 2-core | $108 | $30 PV (etcd) | $138 |
| ArgoCD repo-server | 3 × VM.Standard.E4 2-core | $108 | $20 PV (git cache) | $128 |
| ArgoCD server (UI + API) | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| Flux source-controller | 2 × VM.Standard.E4 1-core | $36 | – | $36 |
| Flux kustomize-controller | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| Flux helm-controller | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| OpenTofu runner (Helm-deployed) | 3 × VM.Standard.E4 4-core | $216 | $50 PV (plan cache) | $266 |
| Postgres iac-state-index (HA primary + replica) | 2 × VM.Standard.E4 4-core | $145 | $200 PV + $50 WAL archive | $395 |
| `oya-cloud-iac-iac-renderer-worker` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `oya-cloud-iac-iac-validator-worker` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `oya-cloud-iac-iac-applier-worker` | 2 × VM.Standard.E4 4-core | $145 | – | $145 |
| `oya-cloud-iac-iac-rollback-worker` | 2 × VM.Standard.E4 1-core | $36 | – | $36 |
| `oya-cloud-iac-iac-registry-worker` | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `oya-cloud-iac-iac-renderer-rest` / `-iac-registry-rest` (combined deployment) | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `oya-cloud-iac-iac-applier-app` (composition root) | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| OpenTofu state-buckets (per pack) | – | – | $30 hot (~1 TB) + $5 archive | $35 |
| iac-state-index backup buckets | – | – | $15 hot + $5 archive | $20 |
| Sigstore Rekor cache (read-only mirror of public log; minimal storage) | 1 × VM.Standard.E4 1-core | $18 | $10 PV | $28 |
| KMS keyring (per pack; for state-encryption + chart-signing) | – | $5 | – | $5 |
| Load balancer (per-pack Istio gateway + public LB) | – | $20 | – | $20 |
| **XS tier total per pack region** | | **~$1413** | **~$415** | **~$1828 / month** |

Verify-at-deploy: OCI pricing changes; reconfirm at deploy time. Buffer 15% for OCI rate increases + 20% for actual-vs-forecast.

## Per-Scale-Tier Forecast

| Scale tier | N_microservices | Concurrent applies | Monthly cost per pack region | Notes |
|---|---|---|---|---|
| XS (M01 launch; ~36 µservices) | 36 | ~20 | ~$1.8k | active: pack-kr |
| S (~100 µservices) | 100 | ~50 | ~$4.5k | active: pack-kr + pack-eu + pack-us (3 packs; ~$13.5k total) |
| M (~500 µservices) | 500 | ~200 | ~$18k | 5 active packs |
| L (~2000 µservices) | 2000 | ~800 | ~$70k | all 11 packs + multi-region per pack |

## Per-Pack Multipliers

- **DR pair packs** (pack-eu, pack-us, pack-au, pack-in, pack-br, pack-ae, pack-ksa): 1.0× primary + 0.6× warm-standby per `multi-region.md`.
- **HIPAA pack** (pack-us-healthcare): 1.4× base (extended retention 6y + dedicated isolated HIPAA-eligible region).
- **KR-FSS-regulated** tenants in pack-kr: 1.2× base (retention 5y per KR commercial code).
- **Single-region packs** (pack-kr, pack-jp, pack-sg): 1.0× base.

## Budget + Alert Thresholds

| Metric | Threshold | Action |
|---|---|---|
| Monthly cost (per pack region) | within 90% of forecast | normal |
| 90% < cost < 110% | yellow alert | FinOps + ops-sre-reliability review |
| 110% < cost < 130% | orange alert | FinOps + leadership review; review autoscale + capacity-model |
| cost > 130% | red alert; budget breach incident | engage ops-finops + axis-cloud-iac; consider rate-limit tightening |
| Per-µservice apply rate (highest spender) | within 5× median µservice | normal |
| Per-µservice apply rate > 10× median | yellow; engage owner on cadence discipline | µservice-facing dashboard surfaces self-overage |

## FinOps SLI

| SLI | Target | Burn-rate alert |
|---|---|---|
| Monthly cost / N_microservices (unit-economic) | within 5% of forecast | 6× burn over 6h |
| OpenTofu state storage growth / day (avg over 7d) | within forecast | 14.4× burn over 1h (catches runaway-state) |
| Spot-vs-on-demand ratio | ≥ 60% spot for stateless workers (renderer / validator) | informational |
| Failed-apply retry rate | ≤ 5% (failed applies cost compute) | informational |

## Cost-Optimisation Levers

| Lever | Estimated saving | Trade-off |
|---|---|---|
| Increase render cache hit-rate (digest-based dedup) | 10–20% renderer compute | More registry storage |
| Reduce drift-detection cadence from ≤1h to ≤4h in low-churn packs | 30–50% validator compute | Slower drift detection |
| Spot-instance fleet for stateless renderer / validator pods | 30–50% renderer + validator compute | Spot eviction recovery via HA |
| OCI committed-use discounts (1y / 3y) | 20–40% compute | Vendor lock-in window |
| Postgres connection-pooling via PgBouncer | 5–10% Postgres compute | Adds PgBouncer ops surface |
| Archive OpenTofu state to infrequent-access tier after 30d | 5–10% storage | Slower historical restore |
| Per-µservice apply rate-limit enforcement | 5–15% compute | µservice owner friction if too aggressive |

## Verification

- cloud-ci/oya-ci governance gate `cost-budget` for --microservice cloud-iac is green in the branch-protected `oya-ci-required` context — exit 0; current spend within 110%.
- Monthly FinOps review: actual vs forecast; lever decisions logged.
- Quarterly: capacity-model + cost-budget refresh; re-run formulas with current data.

## References

- `microservices/cloud-iac/capacity-model.md`.
- `microservices/cloud-iac/multi-region.md`.
- `microservices/cloud-iac/policy/data-residency.md` (per-pack retention multipliers).
- OCI pricing — `oracle.com/cloud/pricing/`.
- ArgoCD reference architectures — `argo-cd.readthedocs.io/en/stable/operator-manual/`.
- OpenTofu capacity guides — `opentofu.org/docs/`.
- FinOps Foundation framework — `finops.org`.
