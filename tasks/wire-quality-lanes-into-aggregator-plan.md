# Plan: wire-quality-lanes-into-aggregator

## Goal
Append the four built-but-unaggregated quality lanes (`statelessness`, `shardability`,
`perf-budget`, `benchmark`) to `AGGREGATED_VALIDATE_LANES` in
`crates/oya-governance-gate-catalog-domain/src/lib.rs`.

The check crates and `gate validate` dispatch arms already exist in `oya-dev-cli`;
the lanes are simply absent from the aggregate array.

## Context
- `AGGREGATED_VALIDATE_LANES` is a `&[&str]` constant in `src/lib.rs`.
- `LANE_INPUT_GLOBS` maps lane names to affected path sets for affected-only verify.
- Existing tests assert count (`>= 30`), uniqueness, and full-catalog render coverage.
- All four lanes appear as dispatch arms in `crates/oya-dev-cli/tests/gate_cli.rs`.

## Acceptance criteria
1. All four lane strings appear in `AGGREGATED_VALIDATE_LANES`.
2. Uniqueness invariant holds (no duplicate entries).
3. `LANE_INPUT_GLOBS` has entries for all four new lanes (path-scoped, not Global).
4. `cargo nextest run -p oya-governance-gate-catalog-domain` passes with zero failures.
5. No other crate is modified.

## Subtasks (ordered)
1. [x] Write plan (this file).
2. [ ] Write spec `docs/specs/task-wire-quality-lanes-into-aggregator.md`.
3. [ ] Add four lane entries to `AGGREGATED_VALIDATE_LANES` (append under `doc-axis`).
4. [ ] Add four `LANE_INPUT_GLOBS` entries for path-scoped affected selection.
5. [ ] Run `cargo check -p oya-governance-gate-catalog-domain --all-targets` (green).
6. [ ] Run `cargo nextest run -p oya-governance-gate-catalog-domain` (green).
7. [ ] Self-review / simplify pass.
8. [ ] Commit + push + PR.
