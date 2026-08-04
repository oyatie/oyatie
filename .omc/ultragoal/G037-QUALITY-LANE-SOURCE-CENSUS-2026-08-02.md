# G037 quality-lane source census — 2026-08-02

State: **PLANNING_ONLY — NOT ACTIVATED, NOT RETIRED, NOT BOUND, NOT REVIEWED**

Authority: coordinator worktree tree (same registry content as `origin/dev` for this face) plus `ci/facade/baseline-ratchet/tests/gate_registration.rs` half (c). No registry mutation. No second registry. No lane bind/retire.

## Cardinality reconciliation (93 vs 96)

| Count | Evidence |
|---|---:|
| Live rows | **96** unique `id`s |
| `status: active` | **91** |
| `status: planned` | **5** |
| G037 goal text “93” | stale snapshot, not a live invariant |

History of `registry/quality/lanes.yaml` (git log on that path):

| Commit | Date | Total | Active | Planned | Delta that matters |
|---|---|---:|---:|---:|---|
| `d6f1e0db11c6` | 2026-05-26 | **93** | 88 | 5 | ADR-0363 PR-3 retire `oya vcs` / flip to plain git |
| `46ab8e8a6550` | 2026-05-26 | 95 | 90 | 5 | **+2** `oya-governance-adr-planning-completeness`, `oya-governance-masterplan-drift` |
| `f2efc12ca3c6` | 2026-05-27 | **96** | 91 | 5 | **+1** `oya-governance-adr-supersession-consistency` |
| `6c436e395d2a` (tip for this face through 2026-08-02) | 2026-08-02 | 96 | 91 | 5 | no cardinality change; rewired 80 dark commands + half-(c) hatch |

G037 objective originally said “Audit all 93 … rows” (2026-05-26 post-ADR-0363 snapshot). Durable goal text was hygiene-corrected to **96** in `.omx/ultragoal/goals.json` only. Live SSOT remains **96**. Do not assert a fixed historical count in any new test; assert properties of the live registry (or shrink-only known-bad targets).

## Already productized contract (do not re-build)

`ci/facade/baseline-ratchet/tests/gate_registration.rs` half (c) already rides the required fan-in:

- Workflow job `gate-baseline-ratchet` runs `//ci/facade/baseline-ratchet:ci-baseline-ratchet-gate-registration` (`.github/workflows/oya-ci-required.yml`).
- Active-lane parser + target extractor + resolvability tests:
  - `the_lane_resolvability_probe_is_falsifiable_on_known_controls`
  - `every_active_quality_lane_resolves_to_a_real_target`
- Shrink-only hatch `KNOWN_UNRESOLVABLE_LANE_TARGETS` (2 target keys, covering 5 active lanes):
  1. `cargo-package:oya-vcs-merge-queue-fix-loop-app` — lane `oya-governance-merge-queue-ref-hygiene` (ADR-0363 removed package).
  2. `repo-file:tools/governance/adr-0221-governance-gates.sh` — four ADR-0221 hook-efficacy lanes (shell harness deleted by ADR-0523; no Rust replacement yet).
- Registry header itself states `check_command` is local/transitional bridge feedback, **not** protected-branch authority. Merge authority remains `oya-ci-required` + constituent cloud-ci/Rust gate packets.

## Live active-lane resolvability census

Oracle = same rules as half (c) (`lane_targets` + dispatcher SOURCE match arms + declared cargo packages + BUCK `name =` + repo file exists).

| Class | Count | Notes |
|---|---:|---|
| Active lanes fully resolvable | **86** | includes 6 bare-toolchain cargo verbs (no repo-local target by design) |
| Active lanes only via known hatch | **5** | 4 shell + 1 dead merge-queue package |
| Active lanes unknown-unresolvable | **0** | half (c) is green on unknown regressions |
| Missing `owner_team` / `source` / `check_command` on active | **0** | |
| Planned rows (no `check_command`) | **5** | `lean-a10-regression`, `quality-statelessness`, `quality-shardability`, `quality-perf-budget`, `quality-benchmark` |

### Active command classes

| Class | Count |
|---|---:|
| `buck2 run //marketplace/facade/dev-cli:oya -- gate validate <lane>` | 78 |
| bare `cargo …` toolchain verbs | 7 (`fmt/check/clippy/nextest/deny/machete` + one package form counted under hatch) |
| shell harness `bash tools/governance/adr-0221-…` | 4 |
| other `buck2` direct | 2 |

### Target extraction totals (active only)

| Target kind | Occurrences |
|---|---:|
| `buck-target` | 80 |
| `gate-lane` | 78 |
| `repo-file` | 4 |
| `cargo-package` | 1 |

Dispatcher SOURCE (`marketplace/facade/dev-cli/src/commands/gate/mod.rs`) currently exposes **120** `(Some("validate"), Some("…"))` arms. Every active `gate validate <lane>` name resolves to an arm (0 missing). That is **bridge reachability**, not protected-context execution of the lane obligation.

## Protected-context vs bridge-only (the real G037 defect)

| Claim | Verdict |
|---|---|
| Lane row exists in YAML | true for 96 |
| Active lane names something that exists (half c) | true except 5 known-hatched |
| `check_command` is executed by `oya-ci-required` | **false** for lane commands; workflow does not mention `lanes.yaml`, `quality-lane`, or `gate validate` |
| Lane registry participates in required CI | **meta only**: half (c) resolvability rides `ci-baseline-ratchet-gate-registration` |
| Bridge `dev-cli` is merge authority | **false** (registry header + CLI retirement doctrine) |

Therefore G037 is **not** “make half (c) exist” — it already does. G037 is disposition of each retained obligation:

1. **BIND** — replace bridge `check_command` with a Rust/Buck2 target that is fan-in-reachable from `oya-ci-required` (or already covered by an existing required gate packet), with RED/GREEN acceptance; **or**
2. **RETIRE / stale-mark** — governance act for lanes whose obligation is gone (ADR retirement, superseded check), shrinking the known-unresolvable hatch only when the lane leaves `active` for a real reason; **or**
3. **KEEP BRIDGE-LOCAL** — explicitly non-binding catalog row, never claimed as protected.

Do not invent a second registry. Do not baseline darkness. Do not flip `active → planned` solely to clear half (c) (the test forbids that class of green-by-relabel).

## Owner / stage distribution (context only)

- Owners: `council-architecture` 52, `axis-foundry` 22, `ops-sre-reliability` 10, others ≤4 each.
- Stages: `per-pr` 59, `foundation` 28, `nightly` 6, `per-release` 3.

## Known-hatched lanes (owner disposition required)

| Lane id | Hatch key | Blocking reason already recorded in half (c) |
|---|---|---|
| `oya-governance-merge-queue-ref-hygiene` | `cargo-package:oya-vcs-merge-queue-fix-loop-app` | ADR-0363 removed package |
| `oya-governance-vacuous-green` | `repo-file:tools/governance/adr-0221-governance-gates.sh` | ADR-0523 deleted shell; no Rust port |
| `oya-governance-adr-orphan-citation` | same | same |
| `oya-governance-version-pin-source-citation` | same | same |
| `oya-governance-buildability-line-count` | same | same |

## Preliminary G037-C bridge ↔ required-graph join

This join is a **candidate classifier, not protected-reachability proof**. It normalizes each of the 78 active `gate validate` arm names and lane ids, compares them with live `ci/facade/*` and `governance/check/*` BUCK package names, and then checks actual dev-cli BUCK dependency edges. It overlays the eight `governance/check` packages selected by affected-set policy and the recursive `ci/facade` required-workflow coverage.

### Name-only first pass

| Join class | Lane rows | Meaning |
|---|---:|---|
| Exact package-name join | **44** | implementation package with same normalized arm/lane name exists |
| Fuzzy-only package join | **4** | must not be auto-bound; needs semantic/source review |
| No package-name join | **30** | dispatch implementation may live inside monolithic dev-cli; no standalone package discovered by name |

### Actual dev-cli BUCK dependency edge

`marketplace/facade/dev-cli/BUCK` has **57 unique first-party package deps** relevant to this surface: all **56/56** `governance/check` packages plus `ci/facade/generated-artifact-freshness`. The bridge wrappers therefore reuse check-kernel libraries rather than cloning all their algorithms.

| Stronger class | Lane rows | Meaning |
|---|---:|---|
| Shared governance/check core **and** policy-selected protected candidate | **7** | same core library is depended on by dev-cli, and its package is selected by affected-set policy |
| Shared governance/check core, but package remains bridge-only by current G036 census | **36** | algorithm reuse proven; protected execution still absent/unproven |
| Exact name join but no dev-cli dependency edge | **1** | `cloud-ci-slo-coverage` → `ci/facade/slo-coverage`; name is not sufficient |
| No exact shared-core/package join | **34** | includes fuzzy and no-name cases; requires source/call review |

The seven **shared-core + protected-route candidates** are lane rows for:

- `codeowners-mirror`
- `data-class`
- `doc-catalog`
- `active-artifact-contract` (two lane rows)
- `pr-traceability`
- `slsa-l3-evidence-grounded`

`cloud-ci-slo-coverage` was removed from this stronger set: it has a name match to `ci/facade/slo-coverage`, but dev-cli's BUCK does not depend on that package. Treating the name alone as equivalence would be a false proof.

For sampled shared-core rows, dispatcher wrappers call functions backed by the check crate (for example, `validate_codeowners_mirror_gate` imports and calls `check_codeowners_mirror::validate_codeowners_mirror`), while the bridge handles filesystem/input adaptation. The remaining obligation is to prove that the protected target executes the same core over acceptance-equivalent inputs—not merely that both link the crate.

The 36 shared-core-but-unprotected rows align with G036's bridge-only governance/check class and should be consumed by the same multi-root reachability work rather than a second G037 registry.

## Smallest safe next slices (ordered)

1. **G037-A (docs/goal hygiene only):** correct G037 objective count 93→96 in ultragoal goals text; no registry edit. **Completed in durable goal state only; no repo mutation.**
2. **G037-B (owner dispositions for hatch only):** five known-unresolvable active lanes → BIND (owned-Rust port + required target) or RETIRE with ADR/source citation; hatch shrinks only when target starts resolving or lane leaves active for governance reason.
3. **G037-C (class, not row-by-row dark rewrite):** continue the preliminary 44/4/30 join above into a call-target/core-logic proof. Only the proven bridge-only remainder needs BIND or RETIRE. No mass `check_command` rewrite without that join.
4. **G037-D (planned five):** leave planned until an owner writes a real target; empty `check_command` on planned is fine; never activate without a resolvable target (half c already born-blocks that).

## Independent-review status

Explore audit subagent for this census terminated on transport/decrypt failure. **No APPROVE inferred.** Local oracle evidence above is coordinator-computed and must still face independent review before any mutation PR.

## Non-goals this slice

- No push of G028.
- No live cluster mutation.
- No lane activation/retirement commits.
- No new multispectrum evidence files.
- No hand-edit of `*.generated.json`.
