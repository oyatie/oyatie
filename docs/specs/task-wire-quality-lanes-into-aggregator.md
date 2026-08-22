# Spec: wire-quality-lanes-into-aggregator

## Objective
Append the four built-but-unaggregated quality lanes (`statelessness`, `shardability`,
`perf-budget`, `benchmark`) to `AGGREGATED_VALIDATE_LANES` in
`crates/governance-gate-catalog-domain/src/lib.rs` so that `oya gate run-all`
dispatches to the check crates that already exist.

## Scope
Single crate: `governance-gate-catalog-domain` (one-file flat crate).
No workspace additions. No external crate changes.

## Lane descriptions
| Lane | Validator crate | Governing ADR/spec |
|---|---|---|
| `statelessness` | `check-statelessness` (M02b/P22 exit-gate lane 1) | ADR-0231 §"Plane 8 — Statelessness" |
| `shardability` | `check-shardability` (M02b/P22 exit-gate lane 2) | ADR-0231 §"Plane 8 — Shardability" |
| `perf-budget` | `check-perf-budget` (M02b/P22 exit-gate lane 3) | ADR-0062 §"performance budgets" |
| `benchmark` | `check-benchmark` (M02b/P22 exit-gate lane 4) | ADR-0062 §"competitive benchmark" |

## Mod layout (flat-clean-arch, ADR-0509)
`src/lib.rs` — single file; all logic inline; no new modules.

## Contract surface
- `AGGREGATED_VALIDATE_LANES: &[&str]` — static slice extended with 4 new entries.
- `LANE_INPUT_GLOBS: &[(&str, LaneInputs)]` — 4 new path-scoped entries added.
- No new public API surface.

## Testing strategy
All tests reside in `src/lib.rs` under `#[cfg(test)]`.

New tests (red → green):
1. `aggregated_validate_lanes_contains_all_four_quality_lanes` — asserts all four
   new lane names are present in `AGGREGATED_VALIDATE_LANES`.
2. `lane_input_globs_contains_entries_for_quality_lanes` — asserts all four lanes
   have entries in `LANE_INPUT_GLOBS` (key validity already covered by existing
   `lane_input_globs_every_key_is_a_member_of_aggregated_validate_lanes`).

Existing tests preserved without modification:
- `aggregated_validate_lanes_is_non_empty` (count >= 30 — satisfied).
- `aggregated_validate_lanes_entries_unique`.
- `all_canonical_commands_concatenates_both_lists`.
- `rendered_form_contains_each_validate_lane_canonical_invocation`.
- All path-glob and `lanes_for_changed` tests.

## Observability / SLO
Domain crate is Tier 1 (kernel): no OTel, no SLO. Observability contract unchanged.

## Crate boundary
Touches ONLY `crates/governance-gate-catalog-domain/src/lib.rs`.
