---
doc_class: Playbook
shape: anchor
length_cap: 120
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Search index/ranker rollouts with A/B + cohort.
planned_enforcement_ref:
  - governance-canary-required
  - governance-shadow-diff
  - governance-cohort-honor
related_adrs: [ADR-0030, ADR-0046, ADR-0047, ADR-0048, ADR-0053, ADR-0055]
adr_citations: [ADR-0053, ADR-0055]
doc_status: published
---

# Playbook: Search Rollout


## 1. Surface

Search-axis ([ADR-0030](../../decisions/ADR-0030-search-engine-architecture.md)) — index pipeline, ranker, query planner, vector store ([ADR-0046](../../decisions/ADR-0046-vector-store-strategy.md)), backend ([ADR-0047](../../decisions/ADR-0047-search-backend-strategy.md)), Korean morphology ([ADR-0048](../../decisions/ADR-0048-korean-morphology-and-multilingual-tokenization.md)).

## 2. Default rail per sub-surface

| Sub-surface | Rail |
|---|---|
| **Ranker (model swap)** | Dark-launch + canary + A/B cohort |
| **Ranker (feature additions)** | Canary + A/B cohort |
| **Index pipeline (schema change)** | Blue/green per index shard ([ADR-0040](../../decisions/ADR-0040-progressive-delivery-canary-blue-green-metric-gated-rollback.md)) |
| **Index pipeline (extraction)** | Canary with re-extract verification |
| **Query planner** | Canary + dark-launch |
| **Tokenizer (morphology)** | Blue/green per index |
| **Vector store (embedding model swap)** | Blue/green per shard with dual-embed window |

Per [ADR-0055](../../decisions/ADR-0055-four-layer-branch-pipeline.md), Search cadence: weekly staging → prod; `perf-reviewer` re-affirms at gate 5 using post-canary latency P95.

## 3. Ranker rollout (the most common case)

1. **Dark-launch.** 100% mirror of production query traffic; new ranker produces shadow rankings; `intelligence-shadow-diff-kernel` diffs top-10 overlap + per-position position-bias-corrected delta.
2. **Hold-out test set replay.** Versioned golden query set replays against the new ranker; quality metric (nDCG / MRR) tracked.
3. **Canary cohort A/B.** 1% → 5% → 25% on `canary-eligible` cohort; both control and treatment served; quality metric AND business metric (engagement, follow-up query rate) measured.
4. **Promote.** Full rollout, with `stable-regulated` lagged 28 d.

## 4. Per-tenant search quality SLOs

| Metric | Target | Window |
|---|---|---|
| Query p95 latency | < 200 ms | 30 d |
| Query availability | 99.95% | 30 d |
| Ranker quality (nDCG@10) | ≥ baseline − 0.5% | per release |
| Korean morphology accuracy ([ADR-0048](../../decisions/ADR-0048-korean-morphology-and-multilingual-tokenization.md)) | ≥ baseline | per release |

## 5. Index migration

Index schema change → blue/green per shard with dual-write window. Readers tolerate both shapes; writers dual-write until 100% backfill; readers cut to new shape; writers cut to new shape only; destructive teardown ≥ 7 days later. Per [`blue-green-spec.md`](blue-green-spec.md) §4.

## 6. Cohort honour

Search experiments respect the `stable-regulated` and `connect-no-ads` cohorts ([`stable-cohort-spec.md`](stable-cohort-spec.md)). Regulated tenants on the prior ranker until the new ranker has soaked ≥ 28 d on canary cohort.

## 7. Vector-store embedding swap

Embedding-model swap requires dual-embed: both old + new embedding indexed for every document during cutover. Query path tests both, scores combined or A/B-routed by cohort. Old embedding retained ≥ 30 d post-cutover.

## 8. Rollback

Per-shard rollback (default). Ranker rollback re-routes query traffic to prior ranker version atomically via mesh.

## 9. Hyperscaler equivalent

Google Search ranker A/B (the canonical reference); Bing Quality Lab; Yandex matrixnet rollouts. We adopt the dark-launch-into-A/B pattern with our cohort gating.

## 10. ADR citations

- [ADR-0055](../../decisions/ADR-0055-four-layer-branch-pipeline.md) — Search weekly cadence; `perf-reviewer` re-affirms at staging → prod gate 5 using post-canary latency P95; index-shard rebuild goes blue/green even on staging.
