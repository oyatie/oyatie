# Route Policy Ranked Candidate Slate — Plan

## Objective

Add `RoutePolicy::rank_candidates(&[RouteCandidate], &RouteConstraints) -> Result<Vec<RouteScore>, RouteError>` to the `oya-intelligence-route-policy-kernel` crate.

Returns ALL eligible candidates ordered by the existing 4-step tiebreak chain rather than only the single winner, exposing the full auditable decision field (failover ladder).

## Requirements Analysis

### Core Behaviour
- Reuse the exact same eligibility filters as `select_weighted`: Active state check, failover_order membership, budget ceiling, residency, privacy boundary.
- Apply the same error-precedence chain: NoCandidates → UnsupportedProvider → NoActiveAccount → BudgetExceeded → ResidencyUnmet → PrivacyBoundaryUnmet → SilentSwitchPrevented.
- Return ALL eligible candidates (not just the winner) sorted by the 4-step tiebreak: cost_micros ASC → model_affinity DESC (true-first) → failover_rank ASC → account_id ASC (lexicographic).
- The silent-switch guard applies before returning — if triggered, `SilentSwitchPrevented` is returned (no slate emitted).
- `slate[0]` MUST equal the winner that `select_weighted` would have chosen for the same input.

### Edge Cases
1. Empty candidates → `NoCandidates`
2. Empty failover_order → `UnsupportedProvider`
3. All candidates inactive → `NoActiveAccount`
4. All active candidates over budget → `BudgetExceeded`
5. All within budget but residency mismatch → `ResidencyUnmet`
6. Residency ok but privacy mismatch → `PrivacyBoundaryUnmet`
7. Silent-switch triggered → `SilentSwitchPrevented`
8. Single eligible candidate → `Ok(vec![that_score])`
9. Multiple eligible — ordered by full tiebreak

### Tiebreak Table Tests
| Scenario | Expected winner |
|---|---|
| cost_micros differ | lowest cost |
| cost tie, affinity differs | affinity=true wins |
| cost+affinity tie, failover_rank differs | lower rank wins |
| full tie (cost+affinity+rank same) | lexicographically smallest account_id |

### Consistency Invariant
For any valid input, `rank_candidates(candidates, constraints)?[0].account_id` equals `select_weighted(candidates, constraints)?.chosen_account_id`.

### K8s / Cloud-Native Implications
None — pure deterministic in-process kernel; no I/O, no network, no storage. SLO authoring not triggered (no µservice promotion).

### Contracts
No new OpenAPI/proto/AsyncAPI changes required — this is an internal kernel function, not an HTTP or event contract surface.

## Subtasks

1. [ ] Write `tasks/route-policy-ranked-candidate-slate-plan.md` (this file)
2. [ ] Write `docs/specs/task-route-policy-ranked-candidate-slate.md`
3. [ ] Add red TDD tests in `lib.rs` `tests_rank_candidates` module — confirm compile fails (function missing)
4. [ ] Implement `RoutePolicy::rank_candidates` — minimum code to go green
5. [ ] Run `cargo nextest run -p oya-intelligence-route-policy-kernel` — confirm all pass
6. [ ] Self-review: correctness / architecture / security / performance / cloud-native-readiness
7. [ ] Simplify: guard clauses, dedupe, dead code, naming
8. [ ] Final green nextest run + commit discipline check
