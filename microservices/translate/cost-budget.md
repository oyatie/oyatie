---
doc_class: CostBudget
title: Cost budget + unit economics
microservice: translate
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-finance + axis-translate + gtm-pricing
related_adrs: [ADR-0026, ADR-0117, ADR-0131, ADR-TRANSLATE-0001]
related_artifacts:
  - microservices/translate/capacity-model.md
  - microservices/translate/multi-region.md
review_cadence: quarterly + on every vendor pricing change
doc_status: published
---

# Cost Budget — translate µservice

## Unit economics targets

| Unit | Target gross margin | Notes |
|---|---|---|
| Translation request (≤ 500 chars, in-house) | ≥ 80 % | In-house path dominant; cost ≈ vLLM token + amortized infra |
| Translation request (external vendor) | ≥ 35 % | Per-vendor cost-per-1K-tokens passes through; oyatie markup + TM-leverage discount |
| TM-leveraged request (≥ 75 % match) | ≥ 95 % | TM hit avoids engine call; only metadata cost |
| Batch translate (100 segments) | ≥ 65 % | Concurrent fan-out amortized |
| Document translate (10-page DOCX) | ≥ 50 % | Sandbox + LibreOffice CPU dominates cost |
| Real-time caption stream (per active session-hour) | ≥ 60 % | foundry-runtime infra dominates |
| Bulk-translate (10 k segment XLIFF) | ≥ 70 % | Worker fan-out amortized |

## Vendor cost-per-1K-tokens reference (M01 baseline; current at 2026-05-17)

| Vendor | Model class | Cost / 1K tokens (input) | Cost / 1K tokens (output) | Notes |
|---|---|---|---|---|
| Anthropic | Claude (frontier; long-context) | per Anthropic pricing | per Anthropic pricing | Used for content classes requiring LLM-grade quality |
| OpenAI | GPT-4-class | per OpenAI pricing | per OpenAI pricing | Used as alternate |
| Google | Cloud Translation API (NMT) | per Google pricing | n/a (char-based) | Used for high-volume short segments |
| DeepL | Pro API | per DeepL pricing | n/a (char-based) | Used for EU pairs (premium quality) |
| Microsoft Translator | Standard | per Microsoft pricing | n/a (char-based) | Tracked; not yet enabled in M01 |
| Amazon Translate | Standard | per AWS pricing | n/a (char-based) | Tracked; not yet enabled in M01 |
| In-house (foundry-runtime) | vLLM/TGI on co-located GPU | ~ $0.0005 / 1K tokens amortized | ~ $0.0005 / 1K tokens amortized | Drives margin; preferred when parity bar met (ADR-0026) |

(Exact $-per-1K figures live in `microservices/translate/iac/helm/translate-router/cost-values.yaml`; updated on every vendor pricing change; not embedded here per ADR-0133 "no hardcoded pricing in policy docs".)

## Per-pack cost driver

| Pack | Engine mix (M01 default) | Cost driver |
|---|---|---|
| pack-kr | in-house 60 % / Anthropic 25 % / Google 10 % / DeepL 5 % | mostly in-house |
| pack-eu | in-house 50 % / DeepL 30 % / Anthropic 15 % / Google 5 % | DeepL premium on EU pairs |
| pack-us | balanced; tenant-driven | tenant-defined |
| pack-us-healthcare | in-house 70 % / Anthropic (BAA + ZDR) 30 % | BAA-restricted |
| pack-jp | in-house 60 % / Anthropic 20 % / DeepL 10 % / Google 10 % | balanced |
| pack-sg / pack-au | balanced | tenant-driven |
| pack-in | in-house 70 % / Anthropic 20 % / Google 10 % | DPDPA cross-border restrictions |
| pack-br | in-house 70 % / Anthropic 20 % / Google 10 % | LGPD restrictions |
| pack-ae / pack-ksa | in-house 80 % / Anthropic 20 % | strict residency |
| pack-cn-stub | in-house 100 % | PIPL cross-border forbidden |

## Infra cost per pack (M01 baseline; OCI list pricing reference)

Per `capacity-model.md` resource estimates × OCI per-vCPU + per-GiB pricing:

| Component | Monthly cost (OCI ap-seoul-1; reference) | Notes |
|---|---|---|
| Compute (router + workers + adapters; 28 cores × 24×7) | ~ $1.5k–2k | E5 shape |
| Memory (70 Gi requests; included in compute) | (folded in) | |
| Postgres (HA primary + replica; 8 cores × 32 Gi) | ~ $1k | |
| Meilisearch (2 nodes × 4 cores × 16 Gi) | ~ $600 | |
| Redis (HA sentinel; 2 cores × 8 Gi) | ~ $200 | |
| S3 (OCI Object Storage; 500 GB base + tenant growth) | ~ $15 base + per-GB tenant growth | |
| Network egress (per-pack inter-AZ + external vendor calls) | ~ $200–500 | tenant-rate-dependent |
| **Baseline per-pack** | **~ $3.5k–$4.5k / month** | scales with tenant base |

Cross-pack: 11 packs × baseline = ~ $40k–$50k / month at idle; scales linearly with tenant rate above baseline.

## Tenant billing model

Per `gtm-pricing` strategy (tracked separately; not in this doc's scope):
- Per-MAU + per-translation tiered pricing.
- TM-leverage discount (≥ 75 % match charged at 5 % of engine call cost).
- Bulk-translate per-segment with volume discount.
- Document-translate per-page surcharge.
- Real-time caption stream per session-hour.

## Cost monitoring

Mimir metrics:
- `oya_translate_cost_per_call_usd{vendor,pack,tenant}` — per-call cost.
- `oya_translate_cost_per_tenant_daily_usd` — daily rolling.
- `oya_translate_tm_leverage_savings_usd_total` — TM-leverage savings (vs no-TM baseline).
- `oya_translate_in_house_share_pct{pack}` — % of calls served by in-house.

Dashboards:
- `dashboards/translation-pipeline.json` — per-engine cost panel.
- `dashboards/quality-and-tm-leverage.json` — leverage savings panel.

Alerts:
- Per-tenant per-day cost > 3× weekly average → alert tenant operator.
- Per-pack engine cost > $1k / day → alert ops-finance.
- In-house share < 30 % when in-house parity met → alert axis-translate (router routing health).

## Cost-optimization levers

1. **Prefer in-house** when parity bar met per ADR-TRANSLATE-0001 + ADR-0026.
2. **Increase TM leverage** — per-tenant TM accumulation; goal ≥ 30 % at month 3, ≥ 60 % at month 12.
3. **Per-tenant content-class hint** — route short segments to NMT (cheap), long contextual to LLM (expensive); per ADR-TRANSLATE-0001.
4. **Batch fan-out** — batch up to 100 segments / call; amortize per-vendor overhead.
5. **Cache identical segments** — Redis cache on (segment_hash, target_lang, content_class); TTL 24 h.
6. **gVisor sandbox pool warm** — avoid 200 ms cold-start per doc.

## Verification

- Quarterly ops-finance review.
- Per-tenant cost projection vs actual reviewed monthly.
- Tenant per-MAU model recalibrated quarterly with gtm-pricing.

## References

- ADR-0026 — in-house AI substrate roadmap (drives in-house cost).
- ADR-0117 — pack residency model.
- ADR-TRANSLATE-0001 — engine routing (drives engine mix).
- `microservices/translate/capacity-model.md`.
- OCI pricing — `cloud.oracle.com/iaas/pricing`.
- Per-vendor pricing pages (live).
- AWS Well-Architected Cost Optimization Pillar.
