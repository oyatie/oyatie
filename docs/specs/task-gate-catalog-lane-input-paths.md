# Spec: gate-catalog lane input path-globs (task-gate-catalog-lane-input-paths)

## Objective

Add a pure, static `LANE_INPUT_GLOBS` table to
`crates/oya-governance-gate-catalog-domain` that maps each governance lane in
`AGGREGATED_VALIDATE_LANES` (`src/lib.rs`) to its declared input path-globs, plus
a pure fn `lanes_for_changed(changed: &[&str]) -> Vec<&'static str>` that returns
the subset of lanes a given changed-file set could affect — with a conservative
**unmapped-lane => always-selected** fallback so the table can never under-select
a lane that has not yet been declared.

This is the data table that gate-lane affected-scope selection reads. It is the
gate-lane sibling of the existing cargo-scope selector in
`crates/oya-dev-cli/src/commands/verify_affected.rs` (ADR-0360 O1): that file
narrows the *cargo* build/test scope from a changed-file set; this table narrows
the *governance-gate-lane* scope from the same kind of changed-file set.

**Scope guardrail:** lib + unit tests ONLY. NO consumer wiring. `gate run-all`
(`crates/oya-dev-cli/src/commands/gate/run_all.rs`) consumes the new fn in a
later window; that crate is out of scope for this task.

## Vertical

`foundation-cicd` lane. Crate touched (the ONLY crate this task may touch):
`crates/oya-governance-gate-catalog-domain`. Plus lane docs
(`tasks/gate-catalog-lane-input-paths-plan.md`) and this spec
(`docs/specs/task-gate-catalog-lane-input-paths.md`).

## Architecture doctrine

Tier 1 kernel-tier (ADR-0083): pure data + small validators; NO filesystem, NO
subprocess, NO network, NO panics outside `cfg(test)`. The crate already carries
the `#![cfg_attr(test, allow(clippy::unwrap_used, ...))]` Tier-3 test exemption;
the new code stays inside the same Tier-1 contract. NO new dependency is added —
the path-glob matcher is a self-contained pure fn, mirroring the established
house pattern `simple_glob_matches` in
`crates/oya-dev-cli/src/workspace_hygiene_gate.rs:436` (no `glob` crate in the
workspace; a kernel crate must stay dependency-free).

## Authority chain

- **ADR-0360** (`docs/decisions/ADR-0360-ci-pipeline-optimization-program.md`) O1
  affected-target selection — the canonical "presubmit selection can only ever
  NARROW work, never under-test trunk" doctrine. `lanes_for_changed` MUST obey
  the same never-under-select rule.
- **ADR-0083** Tier-1 kernel purity contract.
- Sibling reference (do NOT edit): `verify_affected.rs` — its
  `is_full_build_trigger` / `is_rust_relevant` conservative classification is the
  model the gate-lane table follows, one tier up.

## Path-glob semantics (the contract the table commits to)

Changed-file paths are repo-relative, forward-slashed, no leading `./` (same
normalization the cargo selector assumes). The matcher supports exactly three
glob shapes, kept deliberately small and pure:

1. **Exact**: `Cargo.lock` — matches the path verbatim.
2. **Directory prefix**: `microservices/**` or `crates/oya-foo/` — matches any
   path under that directory.
3. **Suffix / extension**: `**/*.openslo.yaml` or `*.md` — matches any path whose
   tail matches the suffix.

Anything the matcher cannot confidently classify is treated as NON-matching for
that glob (the lane is still selected via the unmapped/global fallback rules
below, so safety is preserved at the lane level, not the glob level).

## Selection rules for `lanes_for_changed`

For each lane in `AGGREGATED_VALIDATE_LANES`, in catalog order:

- **Lane has NO entry in `LANE_INPUT_GLOBS`** (unmapped) => ALWAYS selected
  (conservative fallback; declaration is incremental + reviewed, exactly like
  `lane_gate_inputs` defaulting to `Unenumerable` in `run_all.rs`).
- **Lane is declared with a `Global` marker** (reads the whole repo / cross-corpus
  gate, e.g. brand-residue, no-grouping, honest-claims) => ALWAYS selected.
- **Lane is declared with explicit globs** => selected only if at least one
  changed path matches at least one of the lane's globs.
- **`changed` is empty** => return the FULL lane list (no information to narrow
  on; never narrow to nothing).

Output preserves `AGGREGATED_VALIDATE_LANES` catalog order and contains no
duplicates (each lane appears at most once).

## Edge cases

- Empty `changed` slice => full lane list (never empty-narrow).
- A changed path matching a glob with mixed separators or a leading `./` =>
  matcher normalizes/trims a single leading `./` before matching; otherwise treats
  unknown shapes as non-matching for that glob.
- A lane mapped to an empty glob list => treated as `Global` (always selected),
  never "matches nothing" — an empty declared-globs list is a conservative,
  not a permissive, signal.
- Unmapped lane (new lane added to `AGGREGATED_VALIDATE_LANES` but not yet to
  `LANE_INPUT_GLOBS`) => always selected; a unit test enforces that every catalog
  lane is either mapped or falls through to always-selected (no silent drop).
- Duplicate-free output even if two globs in one lane both match.

## K8s / cloud-native + contract implications

None directly — this is a pure in-process data table with no runtime surface
(no HTTP/gRPC, no env, no health/readiness, no OTel emission of its own). It
exists to make the future `gate run-all` affected-scope path cheaper, which is a
CI-throughput concern (ADR-0360, ADR-0380 D-follow-on (b)). The crate stays
OpenSLO-exempt as a kernel data crate (no service promotion).

## Acceptance criteria

- `LANE_INPUT_GLOBS` is a pure `&[(lane, &[glob-or-Global])]`-shaped static, with
  every entry's lane name present in `AGGREGATED_VALIDATE_LANES`.
- `lanes_for_changed(&[]) == AGGREGATED_VALIDATE_LANES` (as a Vec, order-preserved).
- For a path that matches only one declared lane's globs, the result includes that
  lane PLUS every unmapped/global lane, and excludes the other explicitly-mapped
  lanes that do not match.
- Unmapped lanes are always present in the output regardless of `changed`.
- Output is duplicate-free and in catalog order.
- `cargo check -p oya-governance-gate-catalog-domain --all-targets` and
  `cargo nextest run -p oya-governance-gate-catalog-domain` both green with the
  warm shared target dir.
- No new dependency; no consumer (`oya-dev-cli`) edit.
