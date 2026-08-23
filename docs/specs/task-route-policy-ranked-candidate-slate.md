# Spec: Route Policy Ranked Candidate Slate

**Crate**: `intelligence-route-policy-kernel`
**Slice**: `route-policy-ranked-candidate-slate`
**Priority**: high | **Effort**: M | **Lane**: intelligence

## Objective

Expose the full auditable failover ladder from the route-policy kernel by adding `RoutePolicy::rank_candidates`, which returns ALL eligible `RouteScore` entries ordered by the existing 4-step tiebreak chain. The existing `select_weighted` returns only the winning `RouteExplanation`; `rank_candidates` returns every eligible candidate ranked so callers can render full decision provenance.

## Signature

```rust
pub fn rank_candidates(
    candidates: &[RouteCandidate<'_>],
    constraints: &RouteConstraints,
) -> Result<Vec<RouteScore>, RouteError>
```

### Return Contract
- `Ok(slate)` where `slate` is non-empty and `slate[0]` matches the `select_weighted` winner.
- `Err(e)` using the same error-precedence as `select_weighted`.

## Mod Layout (flat-clean-arch)

All code stays in `src/lib.rs` — no new modules or files. The function is added as an `impl RoutePolicy` method alongside `select` / `select_weighted` / `explain_route`.

## Eligibility Filters (reused verbatim from `select_weighted`)

1. `AccountState::Active` in a family present in `constraints.failover_order`
2. `cost_micros <= constraints.budget_micros_ceiling`
3. `residency_region == constraints.required_residency_region`
4. `privacy_boundary == constraints.required_privacy_boundary`

## Error Precedence (identical to `select_weighted`)

`NoCandidates` → `UnsupportedProvider` → `NoActiveAccount` → `BudgetExceeded` → `ResidencyUnmet` → `PrivacyBoundaryUnmet` → `SilentSwitchPrevented`

The silent-switch guard is applied after sorting. If triggered, `Err(SilentSwitchPrevented)` is returned and no slate is emitted.

## 4-Step Tiebreak Chain

| Step | Key | Direction |
|---|---|---|
| 1 | `cost_micros` | ASC |
| 2 | `model_affinity` | DESC (true beats false) |
| 3 | `failover_rank` | ASC |
| 4 | `account_id.0` | ASC (lexicographic) |

## Consistency Invariant

For any valid input `(candidates, constraints)`:

```
rank_candidates(candidates, constraints)?[0].account_id
  == select_weighted(candidates, constraints)?.chosen_account_id
```

## Testing Strategy

All tests are hermetic unit tests in `#[cfg(test)] mod tests_rank_candidates` inside `src/lib.rs`.

### Coverage Matrix

| Test | Asserts |
|---|---|
| empty candidates | `Err(NoCandidates)` |
| empty failover_order | `Err(UnsupportedProvider)` |
| all inactive | `Err(NoActiveAccount)` |
| all over budget | `Err(BudgetExceeded)` |
| residency mismatch | `Err(ResidencyUnmet)` |
| privacy mismatch | `Err(PrivacyBoundaryUnmet)` |
| silent-switch | `Err(SilentSwitchPrevented)` |
| single eligible | `Ok(vec![score])`, `slate[0].account_id` correct |
| multi-eligible, cost tiebreak | slate ordered by cost ASC |
| cost tie, affinity tiebreak | affinity=true first |
| cost+affinity tie, rank tiebreak | lower failover_rank first |
| full tie, id lexicographic | `"a-account"` before `"z-account"` |
| consistency invariant | `slate[0].account_id == select_weighted winner` |
| full slate length | `len == eligible count` |

## Observability / SLO

No I/O and no µservice promotion triggered. No new SLO file required. Existing crate's OTel span attribution (caller layer) remains unchanged.

## Crate Boundary

- ONLY `intelligence-route-policy-kernel/src/lib.rs` is modified.
- No new workspace member, no root `Cargo.toml` changes, no cross-crate edits.
- No new dependencies beyond the already-declared `intelligence-account-domain`.
