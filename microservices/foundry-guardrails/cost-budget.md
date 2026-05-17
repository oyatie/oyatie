---
doc_class: CostBudget
title: Cost Budget + FinOps Posture
microservice: foundry-guardrails
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-finops + axis-foundry-guardrails + ops-sre-reliability
deciders: ops-finops, axis-foundry-guardrails, ops-sre-reliability, council-architecture
related_adrs: [ADR-0117, ADR-0130, ADR-0131]
related_artifacts:
  - microservices/foundry-guardrails/capacity-model.md
  - microservices/foundry-guardrails/multi-region.md
review_cadence: monthly + on every capacity-model revision
doc_status: published
---

# Cost Budget + FinOps Posture (foundry-guardrails µservice)

## Purpose

Track the foundry-guardrails monthly cloud cost across infrastructure (compute + storage + network + KMS + LLM-judge passthrough to foundry-providers), per pack region; surface budget breach via the `oya-check-cost-budget` LEAN lane. Numbers cite OCI public pricing (2026-05-17); verify-at-deploy.

## Cost Categories

| Category | What | OCI pricing reference |
|---|---|---|
| Compute (OKE nodes) | classifier-serving + cedar engine + rest + worker + app pods | `oracle.com/cloud/compute/pricing/` |
| Object storage | Cosign-signed classifier artifacts | `oracle.com/cloud/storage/object-storage/pricing/` |
| Block storage (PV) | Postgres rule-store | `oracle.com/cloud/storage/block-volume/pricing/` |
| Network egress | LLM-judge fallback to foundry-providers (in-cluster, ~$0); audit-chain emit (in-cluster, ~$0); auditor JIT reads | minimal |
| KMS | Per-pack KMS for Cosign signing + Postgres TDE | `oracle.com/security/key-management/pricing/` |
| Load balancer | Per-pack Istio gateway | `oracle.com/cloud/networking/load-balancing/pricing/` |
| **LLM-judge passthrough** | Provider tokens via foundry-providers (largest variable cost) | foundry-providers usage |

## Per-Component Monthly Cost (XS tier, single pack-kr, M01 launch)

Per `capacity-model.md` §"Worked example: XS tier".

| Component | Replicas × instance-type | Monthly compute | Monthly storage | Monthly total |
|---|---|---|---|---|
| Classifier-serving PII/PHI | 4 × VM.Standard.E4 4-core | $290 | $0.50 (artifacts) | $290 |
| Classifier-serving Jailbreak | 4 × VM.Standard.E4 4-core | $290 | $1 | $291 |
| Classifier-serving Content-safety | 4 × VM.Standard.E4 4-core | $290 | $1 | $291 |
| Classifier-serving AI-slop | 4 × VM.Standard.E4 2-core | $144 | $0.10 | $144 |
| Cedar engine (standalone batch) | 2 × VM.Standard.E4 4-core | $145 | – | $145 |
| `*-rest` (per-BC; consolidated) | 4 × VM.Standard.E4 4-core | $290 | – | $290 |
| `*-worker` (per-BC; consolidated) | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `*-app` (composition root) | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| Postgres primary (rule-store) | 1 × r6g.large equiv | $145 | $25 PV | $170 |
| Postgres read replicas | 2 × r6g.large equiv | $290 | $50 PV | $340 |
| KMS keyring (per pack) | – | $5 | – | $5 |
| Load balancer (per-pack gateway) | – | $20 | – | $20 |
| **LLM-judge passthrough** | per-tenant variable | ~$200 (20 tenants × ~$10/mo) | – | $200 |
| **XS tier total per pack region** | | **~$2243** | **~$80** | **~$2530 / month** |

LLM-judge passthrough is the largest variable cost; budgets per `policy/guardrail-enforcement.md` §"LLM-judge budget" cap tenant-driven cost.

Verify-at-deploy: OCI pricing changes; reconfirm against `oracle.com/cloud/pricing/`. Buffer 15% for rate increases + 20% for actual-vs-forecast.

## Per-Scale-Tier Forecast

| Scale tier | N_tenants | Monthly cost per pack region (incl. LLM-judge) | Notes |
|---|---|---|---|
| XS (M01 launch; 20 tenants) | 20 | ~$2530 | active: pack-kr |
| S (~100 tenants) | 100 | ~$8500 | active: pack-kr + pack-eu + pack-us (3 packs) |
| M (~1000 tenants) | 1000 | ~$45k | typically 5 active packs |
| L (~10000 tenants) | 10000 | ~$400k | all 11 packs + multi-region per pack |

## Per-Pack Multipliers

- **DR-pair packs**: 1.0× primary + 0.6× warm-standby.
- **HIPAA pack**: 1.4× base (extended retention 6y; dedicated HIPAA-eligible region; isolated).
- **KR-FSS-regulated tenants** in pack-kr: 1.2× base.
- **Single-region packs**: 1.0× base.

## Budget + Alert Thresholds

| Metric | Threshold | Action |
|---|---|---|
| Monthly cost (per pack region) | ≤ 90% forecast | normal |
| 90-110% forecast | yellow alert | FinOps + ops-sre-reliability review |
| 110-130% forecast | orange alert | leadership review |
| > 130% forecast | red; budget breach incident | engage ops-finops + axis-foundry-guardrails |
| Per-tenant LLM-judge cost projection (highest spender) | within 5× median | normal |
| Per-tenant LLM-judge > 10× median | yellow; tenant engaged on prompt-discipline | tenant dashboard surfaces |

## FinOps SLI

| SLI | Target | Burn-rate alert |
|---|---|---|
| Monthly cost / N_tenants (unit-economic) | within 5% of forecast | 6× burn over 6h |
| LLM-judge invocations / tenant / hour | within budget | 14.4× burn over 1h (runaway) |
| Classifier cold-start frequency | within forecast | informational |
| Spot-vs-on-demand ratio | ≥ 70% spot for non-critical (e.g., AI-slop classifier) | informational |

## Cost-Optimisation Levers

| Lever | Estimated saving | Trade-off |
|---|---|---|
| Quantize classifier models (int8 already; explore int4 for non-critical) | 30-50% classifier compute | Slightly degraded recall; A/B in shadow-mode |
| Distill larger classifiers into smaller per-pack BERT variants | 50% classifier compute | Per-pack training cost |
| Reduce LLM-judge fallback (tighten ensemble disagreement threshold) | 20-40% LLM-judge cost | More ambiguous prompts blocked (fail-closed) |
| OCI committed-use discounts (1y / 3y) | 20-40% compute | Vendor lock-in window |
| Spot-instance fleet for stateless classifier-serving | 30-50% compute | Spot eviction recovery via HA |
| Per-tenant cardinality budget enforcement on rule overlays | 5-15% Postgres | Tenant disruption if too aggressive |
| Cache cedar decisions for read-heavy capabilities | 5-10% cedar compute | Cache invalidation complexity |

## Verification

- `cargo run -p oya-dev-cli -- gate validate cost-budget --microservice foundry-guardrails` — exit 0; current spend within 110%.
- Monthly FinOps review: actual vs forecast; lever decisions logged.
- Quarterly capacity-model + cost-budget refresh.

## References

- `microservices/foundry-guardrails/capacity-model.md`.
- `microservices/foundry-guardrails/multi-region.md`.
- `microservices/foundry-providers/cost-budget.md` (LLM-judge passthrough origin).
- OCI pricing — `oracle.com/cloud/pricing/`.
- FinOps Foundation framework — `finops.org`.
