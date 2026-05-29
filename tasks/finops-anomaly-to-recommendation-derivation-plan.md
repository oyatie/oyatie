# Plan: finops-anomaly-to-recommendation-derivation

## Objective

Add a pure, deterministic `recommend_from_anomalies(anomalies, id_seed)` function to
`oya-cloud-finops-domain` that maps `CostAnomaly` values to `FinopsRecommendation` values.

## Scope

- Crate: `oya-cloud-finops-domain` (only)
- No new workspace members
- No new dependencies
- No I/O

## Tasks

1. Understand existing domain types: `CostAnomaly`, `CostAnomalyKind`, `FinopsRecommendation`,
   `RecommendationKind`, `RecommendationId`.
2. Implement `pub fn recommend_from_anomalies(anomalies: &[CostAnomaly], id_seed: u64) ->
   Vec<FinopsRecommendation>` in `src/lib.rs`.
3. Mapping rules:
   - `SpendSpike` → `InvestigateSpendSpike`; if `resource_id` is `Some`, also emit
     `DownsizeResource` for the same resource.
   - `BudgetSoftLimit` | `BudgetHardLimit` → `PurchaseCommitment`
   - `MarginBelowTarget` → `ReviewRateCard`
4. IDs minted as `frec_s{seed}p{slot}` for determinism; `slot` increments per emitted
   recommendation.
5. Carry `axis`, `resource_id`, `evidence_anomaly` through.
6. Add ≥ 8 hermetic unit tests covering each mapping, resource-present branch, empty input,
   determinism.

## Verification

- `cargo check -p oya-cloud-finops-domain --all-targets` → clean
- `cargo nextest run -p oya-cloud-finops-domain` → 16/16 passed
