# Plan: gate-catalog lane input path-globs

**Lane:** foundation-cicd (kind=cicd, priority=high, effort=S)
**Crate (ONLY):** `oya-governance-gate-catalog-domain`
**Branch:** `feat/cd-gate-catalog-lane-input-paths` (base `origin/dev`)
**Spec:** `docs/specs/task-gate-catalog-lane-input-paths.md`

## Context

`AGGREGATED_VALIDATE_LANES` in `src/lib.rs` enumerates ~90 governance lanes that
`oya gate run-all` runs unconditionally. ADR-0360 O1 introduced affected-scope
narrowing at the *cargo* layer (`crates/oya-dev-cli/src/commands/verify_affected.rs`).
This task adds the *gate-lane* analogue as pure data inside the kernel catalog
crate: a table from lane -> declared input path-globs, and a pure selector that
narrows the lane set for a changed-file list, with a conservative
unmapped-lane => always-selected fallback. Lib + unit tests only. NO consumer
wiring (a later window wires `run_all.rs`).

## Guardrails

**Must have**
- Pure Tier-1 code (ADR-0083): no FS, no subprocess, no network, no non-test panics.
- Conservative selection (ADR-0360 O1): may only NARROW; never under-select.
- No new dependency; self-contained pure path-glob matcher.
- Output order = `AGGREGATED_VALIDATE_LANES` catalog order; duplicate-free.

**Must NOT have**
- No edits to `crates/oya-dev-cli/**` (no consumer wiring this window).
- No edits to root `Cargo.toml` / workspace members.
- No `glob`/`regex`/any new crate dep.
- No change to existing `AGGREGATED_VALIDATE_LANES` / `AGGREGATED_NON_GATE_COMMANDS`
  string contents or order (downstream substring lookups depend on them).

## Subtasks (ordered)

### 1. Add the path-glob matcher (pure)
Add `pub(crate)`/private `fn path_glob_matches(path: &str, glob: &str) -> bool`
supporting exact, directory-prefix (`dir/**` and `dir/`), and suffix/extension
(`**/*.ext`, `*.ext`) shapes; trim a single leading `./` from `path` before
matching; unknown shapes => `false`. Model on `simple_glob_matches`
(`oya-dev-cli/src/workspace_hygiene_gate.rs:436`).
- **Acceptance:** unit tests prove exact, `dir/**`, `dir/`, `**/*.yaml`, `*.md`
  positives and clear negatives (sibling dir, wrong extension); `./x` normalizes
  to `x`.

### 2. Define the `LaneInputs` shape + `LANE_INPUT_GLOBS` table
Introduce a small enum/struct distinguishing `Global` (whole-repo / cross-corpus
lane, always selected) from `Globs(&[&str])` (explicit declared inputs). Build
`pub const LANE_INPUT_GLOBS: &[(&str, LaneInputs)]` declaring a *starter* set of
clearly-scoped lanes (e.g. `slo-coverage` -> `microservices/**/slos/**`,
`*.openslo.yaml`; `architecture-boundaries`, `cloud-iac-*` -> their corpora;
`http-stack`/`dependency-seam` -> `crates/**`, `Cargo.toml`). Leave broad
content lanes (brand-residue, no-grouping, honest-claims, doc-axis) either
`Global` or unmapped. Every declared lane name MUST exist in
`AGGREGATED_VALIDATE_LANES`.
- **Acceptance:** unit test asserts each `LANE_INPUT_GLOBS` key is a member of
  `AGGREGATED_VALIDATE_LANES`; empty-globs entry treated as `Global`.

### 3. Implement `lanes_for_changed`
`pub fn lanes_for_changed(changed: &[&str]) -> Vec<&'static str>` iterating
`AGGREGATED_VALIDATE_LANES` in order: empty `changed` => clone full list; lane
unmapped => push; lane `Global` => push; lane `Globs(g)` => push iff any
`changed` path matches any `g`. Build a lookup from `LANE_INPUT_GLOBS` once;
guarantee duplicate-free output.
- **Acceptance:** `lanes_for_changed(&[])` equals `AGGREGATED_VALIDATE_LANES`
  collected to `Vec`; a single-file change matching one mapped lane yields that
  lane + all unmapped/global lanes and excludes non-matching mapped lanes;
  order preserved; no duplicates.

### 4. Unit tests for invariants + fallback safety
Cover: never-under-select (every lane reachable for *some* input), unmapped lane
always present, global lane always present, narrowing actually excludes a mapped
non-matching lane, catalog-order + dedup, and a regression test that any future
unmapped lane still appears (iterate `AGGREGATED_VALIDATE_LANES`, assert each is
in `lanes_for_changed` of a non-matching path unless explicitly mapped-and-narrowed).
- **Acceptance:** all new tests green; existing tests untouched and still green.

### 5. Verify (warm shared target dir)
```
export CARGO_TARGET_DIR=/Users/jasonlee/Developer/source/target
cargo check -p oya-governance-gate-catalog-domain --all-targets
cargo nextest run -p oya-governance-gate-catalog-domain
```
- **Acceptance:** both commands report their own green verdict (read the tool's
  own exit, not a masked pipeline). No clippy `-D warnings` regressions under the
  workspace lints.

### 6. Commit
`plan(gate-catalog-lane-input-paths)` for the plan+spec; implementation lands in
the PLAN->BUILD window per the workflow. PR targets `dev`.

## Success criteria

- `LANE_INPUT_GLOBS` + `lanes_for_changed` land in `oya-governance-gate-catalog-domain`
  as pure Tier-1 code with full unit coverage.
- Conservative fallback proven: unmapped/global lanes always selected; empty
  input returns the full catalog.
- No consumer wiring, no new dep, no cross-crate edits.
- Crate green on `cargo check --all-targets` + `cargo nextest run` (scoped `-p`).

## Open questions

- Exact starter membership of `LANE_INPUT_GLOBS` (which lanes are confidently
  scoped vs. left `Global`/unmapped) is a reviewed judgment call; the safe default
  is to under-map (more lanes always-selected) and grow the table in later windows,
  exactly as `lane_gate_inputs` grows incrementally in `run_all.rs`.
