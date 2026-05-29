---
doc_class: CostBudget
title: Cost Budget + FinOps Posture
microservice: foundry-eval
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-finops + axis-foundry + ops-sre-reliability
deciders: ops-finops, axis-foundry, ops-sre-reliability, council-architecture
related_adrs: [ADR-0024, ADR-0026, ADR-0117, ADR-0131]
related_artifacts:
  - microservices/intelligence-eval/capacity-model.md
  - microservices/intelligence-eval/multi-region.md
review_cadence: monthly + on every capacity-model revision
doc_status: published
---

# Cost Budget + FinOps Posture (foundry-eval µservice)

## Purpose

Track foundry-eval's monthly cloud cost across infrastructure (compute + GPU + storage + network + KMS + provider model API tokens), per Layer-A + Layer-B component, per pack region. Surface budget breach via `oya-check-cost-budget` LEAN lane. Numbers cite OCI public pricing (2026-05-17) + provider model API pricing snapshots; verify-at-deploy markers called out.

## Cost Categories

| Category | What | Pricing reference |
|---|---|---|
| Compute (VM.Standard / OKE node) | eval-runner-worker, parity-analyzer-worker, replay-engine-worker, rest pods | `oracle.com/cloud/compute/pricing/` |
| GPU compute (Standard.GPU + spot fleet) | eval-case dispatch pool | `oracle.com/cloud/compute/gpu/pricing/` |
| Object storage (S3-compatible) | Baseline outputs + replay traces + eval-run results cold-tier | `oracle.com/cloud/storage/object-storage/pricing/` |
| Block storage (PV) | Postgres + ClickHouse local cache | `oracle.com/cloud/storage/block-volume/pricing/` |
| Network egress | Provider API calls (eval-time); cross-region replication; auditor JIT reads | `oracle.com/cloud/networking/pricing/` |
| KMS | Per-pack KMS keyring; per-subject DEK ops | `oracle.com/security/key-management/pricing/` |
| Load balancer | Per-pack Istio gateway + public ingress | `oracle.com/cloud/networking/load-balancing/pricing/` |
| **Provider model API tokens (eval-time)** | Per-case provider invocation cost | provider pricing (OpenAI / Anthropic / Google / xAI / etc.) |

## Per-Component Monthly Cost (XS tier, single pack-kr region, M01 launch)

| Component | Replicas × instance-type | Monthly compute | Monthly storage | Monthly total |
|---|---|---|---|---|
| eval-runner-worker | 4 × VM.Standard.E4 4-core | $290 | – | $290 |
| eval-runner-rest | 3 × VM.Standard.E4 2-core | $108 | – | $108 |
| parity-analyzer-worker | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| replay-engine-worker | 2 × VM.Standard.E4 4-core | $145 | – | $145 |
| eval-set-registry (Postgres-backed REST) | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| baseline-output-store-rest | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| Postgres (HA primary+replica + 2 standby) | 3 × VM.Standard.E4 4-core | $435 | $150 PV | $585 |
| ClickHouse (3-replica × 3-shard) | 9 × VM.Standard.E4 8-core | $1300 | $400 PV | $1700 |
| ZooKeeper (ClickHouse coordination) | 3 × VM.Standard.E4 2-core | $108 | $30 PV | $138 |
| GPU pool (eval-case dispatcher) | 8 × Standard.GPU.A10 | $2200 | – | $2200 |
| SeaweedFS (baseline + replay store) | 4 × VM.Standard.E4 2-core | $144 | $200 PV cache | $344 |
| Object storage hot-tier (baselines + recent runs) | – | – | $250 (10 TB) | $250 |
| Object storage cold-tier (replay 24mo retention) | – | – | $350 (175 TB archive) | $350 |
| KMS keyring + DEK operations | – | $15 | – | $15 |
| Load balancer (per-pack gateway + public LB) | – | $20 | – | $20 |
| **Provider model API tokens (eval-time, M01 launch budget)** | per-capability quota | $800 (monthly across all capabilities) | – | $800 |
| **XS tier subtotal (infrastructure)** | | **~$5180** | **~$1380** | **~$6560 / month** |
| **XS tier total (infrastructure + tokens)** | | | | **~$7360 / month** |

Verify-at-deploy: OCI pricing changes; reconfirm at deploy. Buffer 15% for OCI rate increases + 20% for actual-vs-forecast. Provider model API token budget reviewed monthly per capability.

## Per-Scale-Tier Forecast

| Scale tier | N_capabilities | N_tenants | Monthly cost per pack region | Notes |
|---|---|---|---|---|
| XS (M01 launch; ~20 capabilities, 20 tenants) | 20 | 20 | ~$7400 | pack-kr active only |
| S (~100 capabilities, 100 tenants) | 100 | 100 | ~$25k | 3 packs active |
| M (~500 capabilities, 1000 tenants) | 500 | 1000 | ~$140k | 5 packs typical |
| L (~2000 capabilities, 10000 tenants) | 2000 | 10000 | ~$1.3M | all 11 packs |

## Per-Pack Multipliers

- **DR pair packs**: 1.0× primary + 0.5× warm-standby (per `multi-region.md`).
- **HIPAA pack**: 1.4× base (extended retention 6y per HIPAA §164.316(b)(2); dedicated HIPAA-eligible region; synthetic-PHI fixture validation overhead).
- **EU AI Act high-risk pack**: 1.2× base (extended §17 logging; per-eval-run §15 evidence schema; quarterly external auditor read budget).
- **Single-region packs**: 1.0× base.

## Provider Token Budget Allocation

Per ADR-0024 §"Resolved 4" (per-capability eval token budget split):
- **Capability-owner cost-center**: pays for per-capability baseline + adversarial + linguistic cohorts (rate-carded).
- **foundry shared budget**: pays for cross-capability replay infrastructure + harness + DEK shred system.
- **Per-capability monthly token budget**: $40 default; capability owner may request increase via OpenBao + cost-center approval.
- **Foundry shared monthly token budget**: $200 baseline for replay sampling + cross-capability A/B.

## Budget + Alert Thresholds

| Metric | Threshold | Action |
|---|---|---|
| Monthly cost (per pack region) | within 90% of forecast | normal |
| 90% < cost < 110% | yellow | FinOps + ops-sre-reliability review |
| 110% < cost < 130% | orange | FinOps + leadership; review autoscale + capacity-model + token budget |
| cost > 130% | red; budget breach incident | engage ops-finops + axis-foundry; consider per-capability token-quota tightening |
| GPU spot-vs-on-demand ratio | ≥ 70% spot for nightly | informational |
| Per-capability token spend | within 110% of allocation | normal; exceed → capability-owner notification |

## FinOps SLI

| SLI | Target | Burn-rate alert |
|---|---|---|
| Monthly cost / N_capabilities (unit-economic) | within 5% of forecast | 6× burn over 6h |
| GPU pool utilization | ≥ 60% (efficient use) | informational |
| Storage growth / day (avg over 7d) | within forecast | 14.4× burn over 1h (catches replay-trace runaway) |
| Provider token spend / day | within forecast | 6× burn over 6h |

## Cost-Optimisation Levers

| Lever | Estimated saving | Trade-off |
|---|---|---|
| Spot-instance fleet for nightly eval (vs on-demand for publish-gate) | 30-50% GPU compute | Spot eviction recovery via re-queue |
| ClickHouse compaction aggressiveness | 10-15% ClickHouse storage | More compactor CPU |
| Per-capability adversarial cohort sub-sample | 20-30% provider token spend | Slightly weaker adversarial coverage |
| Replay-sample throttle (10/day vs 100/day per capability) | 30-50% replay storage + provider tokens | Slower drift detection |
| Archive replay cold-tier earlier (90d → 30d hot threshold) | 5-10% storage | Slower historical query |
| OCI committed-use discounts (1y/3y) | 20-40% compute | Vendor lock-in window |
| Cosign verification cache (hot-path) | minimal direct cost | TTL bounds |
| Provider-tier-down for nightly (use cheaper provider tier for non-publish-gate) | 30-50% token spend | Less representative drift signal |

## Verification

- `cargo run -p oya-dev-cli -- gate validate cost-budget --microservice foundry-eval` — exit 0; spend within 110%.
- Monthly FinOps review: actual vs forecast; lever decisions logged.
- Quarterly: capacity-model + cost-budget refresh.

## References

- ADR-0024 §"Resolved 4" (per-capability token budget split).
- `microservices/intelligence-eval/capacity-model.md`.
- `microservices/intelligence-eval/multi-region.md`.
- `microservices/intelligence-eval/policy/data-residency.md`.
- OCI pricing — `oracle.com/cloud/pricing/`.
- FinOps Foundation framework — `finops.org`.
- Anthropic / OpenAI / Google / xAI pricing pages (snapshot at deploy time).
