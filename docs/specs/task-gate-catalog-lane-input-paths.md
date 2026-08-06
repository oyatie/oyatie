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

- **ADR-0360** (`docs/decisions/ADR-0700-ci-admission-live-apex.md`) O1
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

## Mod layout (flat-clean-arch, ADR-0509)

This crate uses a single flat file (`src/lib.rs`) with no sub-modules; all new
symbols are added inline. The flat-clean-arch doctrine (ADR-0509) for a
kernel-domain crate with no service boundary means there is no
`domain/usecase/adapter` split — everything lives in the one canonical lib root.

New symbols added in this task (all in `src/lib.rs`):

| Symbol | Visibility | Description |
|---|---|---|
| `fn path_glob_matches(path: &str, glob: &str) -> bool` | `pub(crate)` | Pure three-shape glob matcher; exact / dir-prefix / suffix. |
| `enum LaneInputs` | `pub` | `Global` (always selected) vs `Globs(&'static [&'static str])` (explicit). |
| `const LANE_INPUT_GLOBS: &[(&str, LaneInputs)]` | `pub` | Starter table: lane name -> input shape. |
| `fn lanes_for_changed(changed: &[&str]) -> Vec<&'static str>` | `pub` | Returns the affected lane subset for a changed-file list. |

No new files, no sub-modules, no new `mod` declarations.

## Testing strategy

All tests are `#[cfg(test)]` unit tests inside `src/lib.rs` (the house pattern
for this crate). No integration tests, no snapshot tests, no property-based
tests (not warranted for a deterministic, bounded data table).

Required test coverage:

1. **`path_glob_matches` shape coverage** — exact, `dir/**`, `dir/` prefix,
   `**/*.ext`, `*.ext`; positive and negative cases; `./`-prefix normalisation.
2. **`LANE_INPUT_GLOBS` key validity** — every key in the table is a member of
   `AGGREGATED_VALIDATE_LANES`; no key is listed twice.
3. **`lanes_for_changed` base cases** — empty `changed` returns the full
   catalog; a path matching one mapped lane returns that lane plus all
   unmapped/global lanes; a path matching no mapped lane returns all
   unmapped/global lanes (no mapped lane stripped).
4. **Conservative invariants** — every unmapped lane appears in
   `lanes_for_changed` for any input (regression: iterate
   `AGGREGATED_VALIDATE_LANES`, assert each absent-from-table lane is always
   present in the result); output is duplicate-free; output order matches
   catalog order.
5. **Exclusion proof** — a path that clearly does NOT match an explicitly-mapped
   lane's globs causes that lane to be absent from the result (proving narrowing
   actually works).

Verification command (warm shared target dir):
```
export CARGO_TARGET_DIR=/Users/jasonlee/Developer/source/target
cargo check -p oya-governance-gate-catalog-domain --all-targets
cargo nextest run -p oya-governance-gate-catalog-domain
```

## Observability / SLO touchpoints

None. This crate is a Tier-1 kernel data crate with no service promotion (no
HTTP/gRPC surface, no OTel emission, no health/readiness endpoint). It is
OpenSLO-exempt per ADR-0130: SLO authoring is mandatory only for µservices
promoted past dev, not for kernel library crates. No `*.openslo.yaml` is added
or modified by this task.

## Crate boundary (explicit)

**In scope (this task may touch):**
- `crates/oya-governance-gate-catalog-domain/src/lib.rs` — new symbols added.
- `tasks/gate-catalog-lane-input-paths-plan.md` — plan artifact (read-only reference).
- `docs/specs/task-gate-catalog-lane-input-paths.md` — this file.

**Explicitly out of scope (must NOT be touched):**
- `crates/oya-dev-cli/**` — consumer wiring deferred to a later window.
- Root `Cargo.toml` / workspace `[members]` — no new crate, no new dep.
- Any other crate or file in the workspace.
- Existing `AGGREGATED_VALIDATE_LANES` / `AGGREGATED_NON_GATE_COMMANDS` string
  contents or order — downstream substring lookups depend on them.

## K8s / cloud-native + contract implications

None directly — this is a pure in-process data table with no runtime surface
(no HTTP/gRPC, no env, no health/readiness, no OTel emission of its own). It
exists to make the future `gate run-all` affected-scope path cheaper, which is a
CI-throughput concern (ADR-0360, ADR-0380 D-follow-on (b)). The crate stays
OpenSLO-exempt as a kernel data crate (no service promotion).

## OpenAPI / AsyncAPI / proto3

Not applicable. This crate exposes no HTTP, gRPC, or event-bus surface. There
is no schema file to write or update.

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
