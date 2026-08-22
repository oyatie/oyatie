# Spec: finops-anomaly-to-recommendation-derivation

**Crate**: `cloud-finops-domain`
**Lane**: cloud
**Priority**: med
**Effort**: M
**Kind**: dev

## Summary

Add a public, pure, deterministic derivation function that maps detected `CostAnomaly` values to
`FinopsRecommendation` values in `cloud-finops-domain`.

## Function Signature

```rust
pub fn recommend_from_anomalies(
    anomalies: &[CostAnomaly],
    id_seed: u64,
) -> Vec<FinopsRecommendation>
```

## Mapping Rules

| `CostAnomalyKind`                           | `RecommendationKind`                         | Condition                    |
|---------------------------------------------|----------------------------------------------|------------------------------|
| `SpendSpike`                                | `InvestigateSpendSpike`                      | always                       |
| `SpendSpike` with `resource_id = Some(_)`   | `DownsizeResource` (additional)              | only when resource_id present |
| `BudgetSoftLimit`                           | `PurchaseCommitment`                         | always                       |
| `BudgetHardLimit`                           | `PurchaseCommitment`                         | always                       |
| `MarginBelowTarget`                         | `ReviewRateCard`                             | always                       |

## ID Minting

Recommendation IDs are minted as `frec_s{id_seed}p{slot}` where `slot` is an incrementing counter
across emitted recommendations. This ensures:
- Same `(anomalies, id_seed)` always produces identical IDs (determinism).
- Different seeds produce different IDs for the same logical recommendation.

## Fields Carried Through

- `axis` — copied from the source anomaly
- `resource_id` — copied from the source anomaly
- `evidence_anomaly` — set to the source `CostAnomalyKind`

## Acceptance Criteria

1. Deterministic output order and IDs for a given input (same call twice → identical `Vec`).
2. `SpendSpike` without `resource_id` → exactly 1 `InvestigateSpendSpike` recommendation.
3. `SpendSpike` with `resource_id` → 2 recommendations: `InvestigateSpendSpike` +
   `DownsizeResource`, both carrying the resource_id.
4. `BudgetSoftLimit` → exactly 1 `PurchaseCommitment`.
5. `BudgetHardLimit` → exactly 1 `PurchaseCommitment`.
6. `MarginBelowTarget` → exactly 1 `ReviewRateCard`.
7. Empty input → empty `Vec`.
8. ≥ 8 hermetic unit tests; pure, no I/O, no new deps.

## Constraints

- No new workspace member.
- Root `Cargo.toml` untouched.
- No new dependencies.
- All changes inside `cloud-finops-domain`.
