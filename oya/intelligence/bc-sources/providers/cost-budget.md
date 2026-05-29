---
doc_class: CostBudget
title: Cost Budget + FinOps Posture
microservice: foundry-providers
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-finops + axis-foundry + ops-sre-reliability
deciders: ops-finops, axis-foundry, ops-sre-reliability, council-architecture
related_adrs: [ADR-0025, ADR-0026, ADR-0117, ADR-0131]
related_artifacts:
  - microservices/intelligence-providers/capacity-model.md
  - microservices/intelligence-providers/multi-region.md
  - microservices/intelligence-providers/dashboards/provider-cost-per-tenant.json
review_cadence: monthly + on every capacity-model or vendor-pricing change
doc_status: published
---

# Cost Budget + FinOps Posture (foundry-providers µservice)

## Purpose

Track the foundry-providers µservice's monthly cloud cost and (separately) the upstream vendor cost flowing through provider calls. Numbers cite OCI public pricing (2026-05-17) plus per-vendor public token pricing (Anthropic / OpenAI / Google) and ADR-0026 in-house serving baseline; verify-at-deploy markers called out where vendor pricing may have moved.

## Cost Categories

| Category | What | Pricing source |
|---|---|---|
| Compute (provider-router pods) | router-rest, router-worker, per-vendor adapter pods | `oracle.com/cloud/compute/pricing/` |
| Postgres | provider-config persistence (per-pack HA primary+replica) | OCI block storage + compute |
| Valkey | rate-limit / token-bucket state (per-pack sentinel HA) | OCI block storage + compute |
| Network egress (to vendor) | All provider calls — typically the dominant traffic | OCI networking + vendor ingress (charged by vendor) |
| KMS | Per-pack Ed25519 signing keys | `oracle.com/security/key-management/pricing/` |
| **Upstream vendor cost** | Per-token costs charged by Anthropic / OpenAI / Google | per-vendor public pricing pages |
| **In-house serving cost** | GPU compute (vLLM/TGI fleet); amortised per ADR-0026 | `oracle.com/cloud/gpu-compute/pricing/` |

## Per-Component Monthly Cost (substrate; XS tier, single pack-kr region, M01 launch)

Per `capacity-model.md` reference XS-tier scenario (20 tenants, ~10⁶ provider calls / day across all tenants).

| Component | Replicas × instance-type | Monthly compute | Monthly storage | Monthly total |
|---|---|---|---|---|
| `oya-foundry-providers-router-rest` | 4 × VM.Standard.E4 2-core | $144 | – | $144 |
| `oya-foundry-providers-router-worker` (health monitor + cost roll-up) | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| `oya-foundry-providers-router-app` (composition root) | 2 × VM.Standard.E4 2-core | $72 | – | $72 |
| Adapter pods (anthropic / openai / gemini / in-house transports) | 2 × VM.Standard.E4 2-core each, × 4 transports = 8 pods | $288 | – | $288 |
| Postgres (provider-config; HA primary+replica) | 2 × VM.Standard.E4 2-core | $72 | $50 PV | $122 |
| Valkey (rate-limit; sentinel HA) | 3 × VM.Standard.E4 1-core | $54 | $20 PV | $74 |
| KMS keyring | – | $5 | – | $5 |
| Load balancer (per-pack Istio gateway) | – | $20 | – | $20 |
| **Substrate total per pack region (XS)** | | **~$727** | **~$70** | **~$800 / month** |

**Verify-at-deploy:** OCI pricing changes; reconfirm against `oracle.com/cloud/pricing/` at deploy time. Buffer 15 %.

## Upstream Vendor Cost (XS tier; reference scenario)

Reference scenario per `capacity-model.md`:
- 20 active tenants, average workload: 50 K provider calls per tenant per day → 1 M total calls/day.
- Average request: 2 K prompt tokens + 500 completion tokens (best-fit-estimate).
- Distribution: 60 % to Anthropic API, 25 % to OpenAI API, 10 % to Gemini API, 5 % to in-house (M01 minimal in-house traffic).

| Vendor | Cost per 1K prompt tokens (USD) | Cost per 1K completion tokens (USD) | Calls/day | Daily upstream cost | Monthly cost |
|---|---|---|---|---|---|
| Anthropic Claude 3.5 Sonnet (verify) | $3.00 | $15.00 | 600 K | $0.012 × 600K = $7200 + completion 0.5 K * 600K * $15/1K = $4500 | ~$9000 + ~$4500 = ~$13.5K |
| OpenAI gpt-4o (verify) | $2.50 | $10.00 | 250 K | $1250 + $1250 | ~$2.5K |
| Gemini 1.5 Pro (verify) | $1.25 | $5.00 | 100 K | $250 + $250 | ~$500 |
| In-house (amortised GPU + ops cost) | — | — | 50 K | (in serving cost below) | – |
| **Subtotal upstream vendor** | | | | | **~$16.5K / month** |

Verify-at-deploy: every vendor pricing page must be reconfirmed; vendor prices change quarterly. Conservative buffer: 25 %.

## In-house Serving Cost (XS tier)

| Component | Replicas × instance-type | Monthly cost | Notes |
|---|---|---|---|
| vLLM/TGI inference pods (8 × A100 80GB) | per ADR-0026 phase-1 sizing | ~$8K | small fleet for M01 in-house traffic |
| Model artefact storage (cold tier; 30B-parameter quantised) | – | ~$50 | per-pack |
| GPU-node power + cooling allocation | – | (included in compute) | OCI bare-metal GPU |
| **In-house total per pack (XS)** | | **~$8K / month** | |

## XS Tier Total (substrate + upstream + in-house)

| Component | Monthly cost |
|---|---|
| Substrate per pack-kr | ~$800 |
| Upstream vendor cost (passed through to tenants per usage) | ~$16.5K |
| In-house serving | ~$8K |
| **Total XS per pack** | **~$25K / month** |

## Per-Scale-Tier Forecast

| Scale tier | N_tenants | Provider calls/day | Monthly cost per pack | Notes |
|---|---|---|---|---|
| XS (M01 launch; 20 tenants) | 20 | 10⁶ | ~$25K | pack-kr only |
| S (~100 tenants) | 100 | 5×10⁶ | ~$100K | 3 active packs |
| M (~1000 tenants) | 1000 | 5×10⁷ | ~$800K | 5 active packs |
| L (~10000 tenants) | 10000 | 5×10⁸ | ~$6M | all 11 packs |

## Per-Pack Multipliers

- **DR pair packs**: 1.0× primary + 0.6× warm-standby substrate (vendor cost is per-call so no DR multiplier).
- **HIPAA pack** (pack-us-healthcare): 1.4× base substrate (HIPAA-eligible isolated region); vendor cost identical (BAA does not change rate card).
- **In-house preferred** packs (pack-in, pack-br, pack-ae, pack-ksa): 0.7× upstream vendor cost (most traffic shifts to in-house) but 1.3× in-house serving cost (larger fleet).
- **Single-region packs** (pack-kr, pack-jp, pack-sg): 1.0× base.

## Budget + Alert Thresholds

### Substrate

| Metric | Threshold | Action |
|---|---|---|
| Monthly substrate cost (per pack region) | within 90 % of forecast | normal |
| 90 % < cost < 110 % | yellow alert | FinOps review |
| 110 % < cost < 130 % | orange alert | FinOps + leadership review |
| cost > 130 % | red alert; budget breach incident | ops-finops + axis-foundry |

### Per-tenant upstream

| Metric | Threshold | Action |
|---|---|---|
| Per-tenant per-day upstream cost | within configured ceiling | normal |
| 90 % < cost < 100 % | yellow; tenant-facing dashboard surfaces approaching limit | tenant self-throttle |
| cost ≥ 100 % | router stops invocations for that tenant; tenant-facing 429 + escalation | `runbooks/rate-limit-cascade-recovery.md` |
| Per-tenant cost > 10× median | yellow; engage tenant on workload discipline | tenant outreach |

### In-house

| Metric | Threshold | Action |
|---|---|---|
| GPU utilisation (rolling 1h) | 50–80 % | normal |
| > 80 % | yellow; consider fleet expansion | capacity-model refresh |
| < 30 % sustained 24h | yellow; consider fleet contraction | rightsizing |

## FinOps SLI

| SLI | Target | Burn-rate alert |
|---|---|---|
| Monthly substrate cost / N_tenants (unit-economic) | within 5 % of forecast | 6× burn over 6h |
| Per-call cost-per-1K-tokens drift vs vendor rate card | ≤ 2 % | 14.4× burn over 1h |
| Spot/on-demand ratio for stateless components | ≥ 70 % | informational |
| In-house cost-per-1K-tokens vs incumbent | ≤ 0.7× (per ADR-0026 rollout criteria) | per-release |

## Cost-Optimisation Levers

| Lever | Estimated saving | Trade-off |
|---|---|---|
| Shift more traffic to in-house once parity met | 30–60 % upstream | requires GPU capex + parity validation |
| Negotiate enterprise volume discount with vendors (Anthropic / OpenAI) | 15–30 % vendor cost | annual commit lock-in |
| Prompt-caching (Anthropic prompt-caching API) | 50–90 % per cacheable prompt | requires workload pattern fit |
| Batch API (where supported) | 50 % vendor cost | latency trade-off |
| OCI committed-use discounts (1y / 3y) | 20–40 % substrate compute | vendor lock window |
| Smaller-model routing (capability-fit allowing) | 50–80 % vendor cost per affected call | quality trade-off; needs eval-set |
| Spot fleet for non-critical router replicas | 30–50 % substrate compute | spot-eviction recovery via HA |
| Per-tenant prompt-size discipline (max prompt length) | 5–20 % vendor cost | tenant disruption if too aggressive |

## Verification

- `cargo run -p oya-dev-cli -- gate validate cost-budget --microservice foundry-providers` exits 0.
- Monthly FinOps review: actual vs forecast; per-vendor unit-economic drift documented.
- Quarterly: capacity-model + cost-budget refresh with current vendor rate cards.

## References

- `microservices/intelligence-providers/capacity-model.md`.
- `microservices/intelligence-providers/multi-region.md`.
- `microservices/intelligence-providers/policy/data-residency.md`.
- ADR-0026 — in-house AI model substrate roadmap.
- OCI pricing — `oracle.com/cloud/pricing/`.
- Anthropic pricing — `anthropic.com/pricing`.
- OpenAI pricing — `openai.com/pricing`.
- Google AI pricing — `cloud.google.com/vertex-ai/pricing`.
- FinOps Foundation framework — `finops.org`.
