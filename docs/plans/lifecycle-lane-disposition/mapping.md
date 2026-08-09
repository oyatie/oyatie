---
doc_status: drafted
doc_class: HowTo
authority_tier: 3
---

# MAPPING — re-root-or-delete the seven dark lifecycle lanes

Branch: `impl/lom-dark-lifecycle-lanes` · Base: `origin/dev` @ `885794461`
Written before any implementation. Every later unit is checked against this file.

**Why this path.** `.omc/**` is a *restricted tracked root*:
`ci/facade/repo-root-hygiene/root-workspace-hygiene-policy.json` (`restricted_tracked_roots`,
`allowlist` rules `omc-ultragoal-*`) admits exactly four tracked `.omc/` paths and its own prose says
"do not expand the .omc tracked set". `git add -f` there reds `cloud-ci-repo-root-hygiene`. So this
document lives under `docs/`, and it pays the two costs that come with that, in its own commit:

- it declares `doc_status: drafted` in a leading `---` fence, so the `doc-status` lane sees a
  DECLARED artifact and `stage_not_declared` stays at 1921 instead of regressing to 1922;
- it moves the equality-pinned census in
  `governance/check/adr-citation-closure/adr-citation-closure-policy.json` by exactly +1 file, which
  is re-frozen with attribution in the same commit (§7 U0).
  It contains **no `ADR-<digits>` token anywhere**, deliberately, so `citation_lines` (8896),
  `adr_records` (458) and every finding count stay untouched and the census move is a single
  attributable number. Later units must keep that discipline or attribute what they move.

---

## 0. The one-paragraph summary

The anti-vacuity floor this goal asks for **already exists and is already RED-tested**. Lighting a
lane up is a DATA change: fix the config's glob and delete its `known_broken_lanes` entry in the same
commit — the deletion is what arms the floor. Do not build a gate, a crate, a floor, or a
`min_artifacts` mechanism. The real constraint is that the kernel's reader can physically reach
almost none of the proposed re-root surfaces, so **six of the seven lanes are DELETE and exactly one
is a genuine re-root**. End state: 9 configs → 3, `known_broken_lanes` empty, every surviving lane
observing a real corpus behind an armed floor.

---

## 1. The mechanism as it exists

| Thing | Where | What it already does |
|---|---|---|
| Live doctrine | `docs/decisions/*-general-live-apex.md` (lines 79, 482) | Restates the lifecycle-framework rule verbatim: one generic kernel, each lifecycle is a JSON config under `specs/lifecycle-configs/`. The original decision record is archived/Superseded — cite the rule via the live apex, never the archived path. |
| Kernel | `libs/oya-governance-lifecycle-kernel/src/lib.rs` | `evaluate()` (pure) + `discovery::{expand_glob, discover, frontmatter_scalar}` (I/O). |
| Gate kernel | `ci/facade/lifecycle-status/src/lib.rs` | `parse_policy`, `compare`. |
| Live run | `ci/facade/lifecycle-status/tests/lifecycle_status.rs` | ONE `#[test]`; `set_current_dir(repo_root)`; walks DISK; `EVALUATED_AT = 2026-01-01`. |
| The ledger | `ci/facade/lifecycle-status/lifecycle-status-policy.json` | `configs_dir`, `frozen_violation_baseline`, `known_broken_lanes`. |
| Wiring | `.github/workflows/oya-ci-required.yml` lines 579-589 | `buck2 test //ci/...` — recursive, by PATTERN not by name. Grepping the workflows for this gate's name returns nothing; it is nonetheless in the required context. |

The floor, in `compare()` (`ci/facade/lifecycle-status/src/lib.rs` lines 283-303):

```
Observed{artifacts:0}   && !known_broken -> LaneDiscoveredNothing     (blocking)
DiscoveryFailed(err)    && !known_broken -> LaneDiscoveryFailed       (blocking)
Observed{artifacts>0}   &&  known_broken -> KnownBrokenLaneNowLive    (blocking)
ledger/baseline row naming a deleted config -> KnownBroken/BaselineLaneWithoutConfig
observed > frozen row                       -> BaselineRegression
observed < frozen row                       -> BaselineStale          (bidirectional)
```

RED fixtures already committed in `ci/facade/lifecycle-status/src/tests.rs`:
`an_unlisted_lane_that_matches_zero_artifacts_is_born_blocking` (141),
`an_unlisted_lane_that_cannot_walk_its_corpus_is_born_blocking` (125),
`a_listed_broken_lane_is_tolerated_until_it_starts_working` (158),
`a_ledger_entry_outliving_its_config_fails_closed` (192).

**What is genuinely owed is the LIVE proof, not a new test.** See §8 item 4.

---

## 2. Kernel constraints every unit must obey

Read out of the source; undocumented anywhere else. These kill most "obvious" re-roots.

1. **`SourceSpec.kind` is never read.** `discover()` (kernel lines 482-524) iterates
   `config.sources` and calls `frontmatter_scalar` unconditionally. `kind` is descriptive prose.
   `crate-status`'s `cargo_metadata_table` reader does not exist and never did.
2. **`expand_glob` (567-576) expresses exactly two shapes:** `<literal-dir>/**/<name-pattern>` and
   `<literal-dir>/<name-pattern>`. **No wildcard directory component.**
   `crates/*-domain/Cargo.toml` resolves `dir` to the literal string `crates/*-domain`, so that lane
   was **born broken** — the capability-first reorg is not its root cause, and the defect text in the
   ledger is only half true.
3. **`matches_glob` (640) matches the FILE NAME, with at most one `*`.** A recursive tail containing
   a `/` (e.g. `docs/**/sub/*.md`) is handed to the filename matcher and matches **zero** files
   forever. That is a vacuity bug wearing the costume of a correct pattern.
4. **Missing root → `Err` (red). Live root, empty match → `Ok(vec![])` (red once the ledger entry is
   gone).** This asymmetry is the entire subject of this goal.
5. **`frontmatter_scalar` (537-565)** requires the document to open a `---` fence, matches the field
   as a **prefix of a `trim_start()`ed line at any indentation**, and returns `None` for an empty
   value — indistinguishable from "absent" → `StageNotDeclared`.
6. **`discover()` returns `Err` on the FIRST unreadable / non-UTF-8 file**, turning the whole lane
   into `DiscoveryFailed`. Never re-root onto a tree holding binaries or symlinked directories.
7. **`SourceSpec.filter` IS honoured** (unlike `kind`): `discover()` 492-495 skips non-matching files
   silently. A filter matching nothing yields `artifacts: 0` and the floor fires. Correct — but know
   it is a second route to vacuity.
8. **Multiple `sources[]` entries are supported** by the loop; no shipped config uses more than one.
   Hand-listing N literal roots is legal and untried.
9. **The gate walks DISK, not git.** Untracked files count. A locally created directory can flip a
   lane live on your machine and not in CI.

---

## 3. THE RULINGS

Presumptive-binding. A unit may overturn one **only** by recording contradicting measured evidence
(command + output) in the same commit message.

### 3.1 DELETE — six lanes

| Lane (= config file stem) | Ruling | Evidence |
|---|---|---|
| `plan-status-lifecycle` | DELETE | Root `.omc/plans/milestones/**/*.md` is gitignored (`.gitignore`: `/.omc/*` with four re-inclusions, all under `.omc/ultragoal/`), so it is absent from every checkout. The nearest tracked surface, `specs/masterplan.json`, is ONE file; the kernel is one-artifact-per-FILE with no row reader, so a re-root observes 1 artifact — clearing the floor while observing nothing. Vacuity in a new shape. |
| `migration-status-lifecycle` | DELETE | Same gitignored root. Nearest tracked corpus is 27 `*/migration-playbooks/*.md` scattered across ~15 capability dirs: unreachable by one glob (§2.2), needing ~15 hand-listed sources, and none carries frontmatter — it would baseline 27 fabricated `stage_not_declared`. The ledger's own recorded resolution names `docs/migration-playbooks/`, which has **0 tracked files**. |
| `capability-status-lifecycle` | DELETE | `specs/**/*.capability.json` matches **0** files while `specs/` resolves — the vacuous green named in the goal. Candidate A `specs/capability-registry.json` = 1 file. Candidate B `*/capabilities/*.yaml` = **378 files across 66 distinct directories**, of which only **16** declare any `status:` and **1** is fenced. Both are vacuity or fabrication. |
| `dependency-status-lifecycle` | DELETE | `docs/dependencies/` has 0 tracked files. Both recorded alternatives (`registry/dependency-rationales.json`, `oya-deps.toml`) are single aggregate files → 1 artifact. |
| `crate-status-lifecycle` | DELETE — **re-affirmed, on different evidence; two claims below are RETRACTED** | `crates/*-domain/Cargo.toml` does not exist and was never satisfiable (§2.2); `git grep -c lifecycle_stage -- '*.toml'` matches **zero files**; the declared `cargo_metadata_table` reader does not exist. RETRACTED: "`registry/catalog/*.yaml` carries no lifecycle-stage field" (it carries `status:` on 56/750) and "a re-root needs new kernel code" (`c4925c55d` removed that constraint). The registry/catalog option — the only one that mattered — is answered on its merits in **§3.1.1**, not on the retracted premises. |
| `feature-flag-status-lifecycle` | DELETE **+ 3 reference repairs** | `docs/feature-flags/` has 0 tracked files. `flags/catalog/*.yaml` is a **crate** catalog (`oya-feature-flags-*`), not a per-flag artifact class; no per-flag corpus exists anywhere in the tree. Blast radius, all three repaired in the same commit: `ci/facade/contract-slice-conformance/contract-slice-policy.json:1670`, `ci/facade/contract-slice-conformance/slices/release-001-runtime-safety-policy.json:164`, `flags/release/runtime-safety-policy.json:94` (`status_lifecycle_ref`). |

#### 3.1.1 `crate-status-lifecycle` vs `registry/catalog/*.yaml` — the re-litigation, answered directly

The first DELETE ruling for this lane argued only about `crates/*-domain/Cargo.toml`. That is not
the option that mattered. The ledger's own recorded resolution named **`registry/catalog/*.yaml`**,
which is 750 separate one-record YAML files — neither "a single file holding many records" nor
"JSON/TOML that cannot carry a `---` fence", the two shapes the deletion rested on. It is the
identical surface the `api-stability-tier` lane was successfully RE-ROOTED onto in §3.2, on this
same branch, after `c4925c55d` made fence-less documents readable whole. So the re-root deserved
an answer on its merits. Here it is, and the ruling still lands on DELETE — for three reasons that
did not appear in the original rationale.

**Measured at this tree** (`registry/catalog/`, 750 `*.yaml`, one per crate):

| Fact | Command | Result |
|---|---|---|
| Corpus size | `ls registry/catalog/*.yaml \| wc -l` | **750** |
| `status:` present | `grep -l '^status:' registry/catalog/*.yaml \| wc -l` | **56** |
| `status:` values | `grep -h '^status:' registry/catalog/*.yaml \| sort \| uniq -c` | `2 active`, `34 designed-ahead-row-no-crate`, `20 retired-compatibility-row-no-crate` |
| Config vocabulary | `specs/lifecycle-configs/crate-status-lifecycle.json` (deleted) | `{scaffolded, live, quiescent, archived}` |
| Intersection of the two vocabularies | — | **∅** |

So a re-root reports **750 of 750 artifacts violating**: 694 `stage_not_declared` (field absent —
`evaluate()` step 1) plus 56 `unknown_stage` (present, not in the vocabulary — step 2). It cannot
be narrowed to the 56: `SourceFilter` (kernel `lib.rs:155`) offers only `kind_field_value` (an
EXACT value match) and `filename_contains_any`, and neither can express "the field is present".

**R1 — `status` is a different AXIS, not merely a different vocabulary.** The `api-stability-tier`
fix worked because `[preview, stable, GA]` is *the same property* the config named, spelled
differently, with three independent canonical confirmations. Here the corpus vocabulary is
row-provenance — every one of the five canonical values asserts *"no crate exists for this row"*
(`ci/facade/artifact-inventory-registry/src/main.rs:1082`, `NON_LIVE_STATUS_MARKERS`). Adopting it
does not fix `crate-status`; it renames the lane to `catalog-row-non-liveness`.

**R2 — absence is CONTRACTUALLY CORRECT for the 694, so the debt is not retirable by any edit this
lane can make.** The field's owning gate states the contract verbatim: *"A LIVE record needs no
marker (the gate checks live OR marked)"* (`catalog_non_live_marker`, same file, ~`:1147`). The
lifecycle kernel has no "absence is legal" mode — absent is `StageNotDeclared`. The 694 rows would
therefore be simultaneously **correct** under `cloud-ci-catalog-liveness` and **violating** under
`cloud-ci-lifecycle-status`. Retiring the 694 means either declaring 694 live crates non-existent
(false), or introducing a sixth value into a closed vocabulary that belongs to another gate — an
edit outside this lane. This is the same failure mode as the retracted `unknown_stage: 750`
api-stability baseline whose only remedy the claim-ceiling gate rejected (§3.2), one hop further
out: **not forbidden, but not retirable by anything this lane owns.** `status: active` on 2 rows
is the live proof — it is outside `NON_LIVE_STATUS_MARKERS`, so the owning gate silently ignores it.

**R3 — the property is already enforced twice, born-blocking, with zero authoring.** Row↔crate
correspondence is closed in *both* directions today: `ci/facade/crate-catalog-coverage` (crate→row)
and `cloud-ci-catalog-liveness` (row→crate), both computed mechanically from the workspace member
set. A frozen count of 694 undeclared rows adds no property those two do not already prove, at the
cost of 694 hand-authored declarations of a fact derivable from disk. Contrast `doc-status`, which
IS kept with 1921 `stage_not_declared`: that count is the **only** measurement of doc lifecycle
anywhere in the repo, which is exactly why it is a ratchet and this one would be a third copy.

**R4 — coverage, for completeness.** `ci/facade/crate-catalog-coverage/crate-catalog-coverage-policy.json`
declares `min_expected_crates: 800` with **197** live crates carrying no row at all. The catalog is
not the crate universe, so this surface structurally cannot govern the ~895 crates the lane claims.

**THE RE-OPENING CONDITION, which is the part worth keeping.** This lane becomes correct the moment
a per-crate maturity declaration exists **whose absence is not already meaningful to another gate** —
concretely, a `lifecycle_stage:` key in the catalog record schema that
`libs/oya-crate-registrar-app/src/lib.rs` `catalog_yaml::compute` **emits**, so new crates are born
declaring it. Until the producer emits it, any lane rooted on a catalog field reds on the next
`register_crate`; that is not a hypothetical, it is the defect this branch fixed for `api_stability`
in the same change that records this section.

#### 3.1.2 The other five, re-litigated against their alternate surfaces

§3.1.1 exists because the first `crate-status` rationale argued the wrong option. The same class of
error — dismissing the surface that mattered in one line, or never naming it — is checkable in the
other five, so each was re-measured at this tree against its best available alternate. **All five
DELETE rulings stand. None was overturned.** Two candidates that the §3.1 table never named are
surfaced and answered here rather than left for a future re-litigation to find.

Every ruling below turns on one of two kernel limits, and both are now pinned by tests so a widening
of either forces this section to be re-opened rather than silently expiring (§2.2, §2.3; tests
`a_wildcard_directory_component_is_a_literal_path_component_not_an_expansion` and
`a_recursive_tail_containing_a_slash_matches_zero_files_forever`).

| Lane | Best alternate surface | Measured | Ruling |
|---|---|---|---|
| `capability-status` | `*/capabilities/*.yaml` | 378 files, **66 directories, 18 distinct top-level roots** | DELETE — unreachable, and wrong axis |
| `dependency-status` | `third-party/**/fixups.toml` (**not named in §3.1**) | 66 files vs **1274** third-party crate targets | DELETE — 5% coverage, no lifecycle key |
| `migration-status` | `*/migration-playbooks/*.md` | 27 files, 14 directories, 0 fenced | DELETE — unreachable, fabricated debt |
| `plan-status` | `docs/plans/**/*.md` (**not named in §3.1**) | 13 files, one legal glob, 3 declare `status:` | DELETE — subset of a lane already kept |
| `feature-flag-status` | `flags/catalog/*.yaml` | 12 files, `canonical-crate-record-schema.json` | DELETE — a crate catalog, not a flag corpus |

**`capability-status`.** The §3.1 cell dismissed this in one line, and the corpus is the largest of
the five, so it got the closest look. Three independent grounds, any one sufficient:

- *Not expressible.* `expand_glob` treats the head of a `/**/` split, and the `rsplit_once('/')`
  directory of a shallow glob, as a **literal path**. `*/capabilities/*.yaml` resolves `dir` to the
  literal string `*/capabilities`, which does not exist → `Err("missing source root")` → the lane is
  `DiscoveryFailed`, not a 378-file corpus. Reaching the files needs **66 hand-listed shallow
  sources** (18 roots × their sub-paths), which is legal (§2.8) and is the worst possible shape here:
  `shallow_glob` errors on a missing root, so **any one of the 66 directories moving reds the entire
  lane**, and a directory that is added is silently invisible. That is 66 tripwires wired into a
  repo whose capability-first reorg is moving directories now.
- *Wrong axis.* Config vocabulary is `{proposed, granted, revoked, expired}` with
  `deadline_field: expires_at` — an authz **grant** lifecycle. Measured in the 378: `expires_at`
  present on **0**, `granted`/`revoked` on **0**, `status:` on **16** with values
  `{Accepted × 10, Active × 3, incubating × 3}` — vocabulary intersection **∅**.
- *The near-miss, ruled out on its merits.* `maturity:` is present on **108/378** with
  `{stable × 66, preview × 18, incubating × 11, proposed × 8, scaffolded × 4, experimental × 1}`.
  This is the one field that would survive the api-stability re-vocabulary precedent — and it must
  not be used, because it is a **maturity** axis, which is the axis `api-stability-tier` already
  governs (§3.2). Adopting it renames the lane and makes it a second copy. The remaining 270 would
  be fabricated debt on a field whose absence is not wrong.

**`dependency-status`.** §3.1 named only two single-file alternates (1 artifact each = vacuity in the
shape the goal is about). It never named the real per-dependency corpus: `third-party/fixups/*/fixups.toml`,
which **is** reachable by one legal glob (`third-party/**/fixups.toml`, literal head, filename tail)
and **is** one record per dependency — so it had to be answered, not assumed away. It fails on
coverage and content: **66** fixups against **1274** third-party crate targets, i.e. a fixup exists
only for a dependency that needs a *build* fixup, so the corpus is not "the dependencies" but "the
awkward five percent". Every key across all 66 is reindeer build vocabulary
(`run`, `env`, `cfgs`, `srcs`, `preferred_linkage`, `name`, `headers`, …) — **no lifecycle key on any
file**, so a re-root baselines 66 fabricated `stage_not_declared` on a schema that is not ours to
extend.

**`migration-status`.** Confirmed: 27 `*/migration-playbooks/*.md` across **14** directories, every
one opening with a `# Migration playbook — …` heading and **none** carrying a fence. Two failures,
and the second is the more instructive: the natural-looking single glob `<root>/**/migration-playbooks/*.md`
is **not** a narrowing — the recursive tail is handed to `matches_glob`, which matches a **filename**,
and no filename begins `migration-playbooks/`. It matches **zero files forever** while looking
exactly like a correct pattern (T2). This is the trap that makes the lane look re-rootable on paper.

**`plan-status`.** `.omc/plans/milestones` is absent from this checkout and gitignored, so the lane
is `DiscoveryFailed` in CI regardless. The candidate §3.1 never named is `docs/plans/**/*.md` — and
unlike every other alternate here it **is** one legal glob over a real corpus, so it is the strongest
of the five and gets the direct answer §3.1.1 gave `registry/catalog`. It still fails, on two
grounds. First, content: 13 tracked `.md`, of which **8** declare `doc_status: published` (a
different lane's field), **3** declare `status:` with `{Accepted, Superseded, approved}`, **1**
carries no fence at all — against a 17-stage vocabulary whose intersection with those three values is
**∅** (`approved-folded` is a stage; `approved` is not, and stage matching is equality, not prefix).
A re-root reports 13/13 violating on two codes. Second, and decisive independent of any vocabulary
edit: `docs/plans/**` is a **strict subset** of `doc-status`'s `docs/**/*.md`, so every one of those
13 files is already counted in that lane's frozen 1921 — R3 of §3.1.1, a third copy of a property
already measured, at 13 artifacts.

**`feature-flag-status`.** Confirmed there is **no per-flag artifact class anywhere in the tree**
(negative established by enumerating `flags/` and every tracked path matching `feature-flag`; the
matches are archived decision records, two architecture docs, one canonical spec, one SLO file, and
crate rows — no per-flag record). `flags/catalog/*.yaml` is 12 files whose first line names
`specs/catalog/canonical-crate-record-schema.json`, the same schema `registry/catalog/*.yaml` uses:
re-rooting a flag lane there governs twelve **crates** named `oya-feature-flags-*`, which is a third
copy of `crate-status` wearing a flag's name.

**RE-OPENING CONDITIONS.** `capability-status` re-opens if `expand_glob` gains a wildcard directory
component *and* a grant-axis field is emitted into capability records by their producer.
`plan-status` re-opens if plan lifecycle is separated from `doc_status` onto a field of its own.
The other three re-open only if a per-artifact corpus is created that does not exist today.

### 3.2 RE-ROOT — one lane

**`api-stability-tier-lifecycle` → `registry/catalog/*.yaml`.** The only surface in the repo where
per-file lifecycle data already exists.

Measured at base: `registry/catalog/` holds **750** `*.yaml` (one per crate); **750/750** carry
`api_stability:`; value distribution **`preview` × 750**; **0/750** are fenced.

Config edit — exactly this, nothing more:

```json
"sources": [{ "kind": "yaml_document",
              "glob": "registry/catalog/*.yaml",
              "stage_field": "api_stability",
              "supersession_field": "replaced_by_api",
              "deadline_field": "removal_deadline",
              "milestone_field": null }]
```

`registry/catalog/*.yaml` is a SHALLOW glob over a literal directory — legal under §2.2.
Non-`.yaml` siblings (`BUCK`, `OWNERS`) are filtered out by `matches_glob`.

**Kernel edit required** (§2.5 blocks it otherwise): make `frontmatter_scalar` read a **fence-less
document whole**. A document whose FIRST line is `---` keeps today's exact fenced semantics.

Measured cost of that kernel edit — **provably inert for both live lanes**:

| lane | today | with fence-less read |
|---|---|---|
| `doc-status` (2701 `docs/**/*.md` on disk) | `stage_not_declared` 1921, `unknown_stage` 9 | **1921 / 9 — unchanged** |
| `adr-status` (10 `docs/decisions/ADR-*.md`) | 10 × `Accepted`, 0 violations | **unchanged** |
| `api-stability-tier` | discovery never ran | **750 artifacts, `unknown_stage` 750** |

Today's 1921/9 reproduce the frozen baseline exactly, which validates the reader model; and **0** of
the 1921 undeclared docs carry a `doc_status:` line outside a fence, which is why the change cannot
move that number. Re-derive both columns yourself before landing — do not carry these over.

**RETRACTED — this ruling was overturned by U7 on contradicting measured evidence (§0 line 107).
What SHIPPED is `stages: [preview, stable, GA]` with NO violation row.** The retraction is recorded
in place, in the §3.1.1 convention, because the amendment was previously applied only *partially*:
R2 of §3.1.1 already calls this baseline "retracted" while this section still read as binding, so a
re-litigation arriving here — the path `specs/reachability-registry.json` anchors this file for —
would have read a live ruling that the tree had already reversed.

**The overturning evidence, measured at this tree.** `750/750 registry/catalog/*.yaml` carry
`api_stability: preview`; a 100% failure rate on a SINGLE code is a config-mismatch signature, not a
debt signature. `[preview, stable, GA]` is the repo's canonical tier vocabulary, confirmed three
independent ways: `docs/machine-readable/contracts.json` `_metadata.stability_tiers`;
`enum ApiStability` in `intelligence/core/catalog-domain/src/lib.rs`; and `validate_claim_ceiling_gate`
in `marketplace/facade/dev-cli/src/governance_gates.rs`, which runs
`FoundationClaimCeiling::preview_foundation()` over this exact directory. So this was never `preview`
being *added* to a vocabulary — it was the config carrying the WRONG vocabulary, sharing one token
with the right one. Both original reasons are kept below with their dispositions, because deleting
them would erase the reasoning a re-litigation needs.

1. ~~It converts 750 undeclared tiers into fabricated compliance.~~ **VOID ON ITS PREMISE.** The 750
   were never undeclared: every one declares `api_stability: preview` explicitly. The
   `gate-self-conformance` prohibition it cites is against *stamping a default stage onto artifacts
   that declare none* — a different act from aligning a config to values the corpus already carries.
2. **`unknown_stage: 750` IS the anti-shrink floor** — **CORRECT ANALYSIS, OVERTURNED AT A COST, and
   the cost is real.** The floor could not be kept: its ONLY remedy — declaring higher tiers per
   crate — is precisely what `validate_claim_ceiling_gate` rejects over this directory, so no
   permitted action could ever have shrunk it, and a ratchet no permitted action can retire is not a
   ratchet. The stated consequence stands exactly as written: with no violation row this lane's only
   floor is `artifacts > 0` (`ci/facade/lifecycle-status/src/lib.rs` `compare()`, the
   `LaneObservation::Observed { artifacts: 0, .. }` arm), so a PARTIAL corpus collapse is silent
   **on this lane**. What catches it instead is `ci/facade/crate-catalog-coverage`, recorded in the
   lane's own `_comment` in `ci/facade/lifecycle-status/lifecycle-status-policy.json`.

~~Baseline row to add: `"api-stability-tier-lifecycle": { "unknown_stage": 750 }`.~~ **RETRACTED with
the ruling above: no violation row is added and the lane ships at zero violations.**

Also add one line of doc comment on `SourceSpec.kind` recording that no reader dispatches on it.
That is an honesty fix, not a mechanism — **do not implement kind dispatch.**

### 3.3 End state

```
specs/lifecycle-configs/   adr-status · doc-status · api-stability-tier      (3, all live)
known_broken_lanes         {}  (keep a "_comment" key; parse_policy requires the object to exist)
frozen_violation_baseline  doc-status {1921, 9} · api-stability-tier — NO violation row (§3.2 RETRACTED)
```

---

## 4. Naming, module and ownership conventions

- Lane key = the config **file stem** (`api-stability-tier-lifecycle`), used identically in
  `known_broken_lanes` and `frozen_violation_baseline`. **Never** the config's inner `"name"` field
  (`api-stability-tier`) — the gate derives lanes from `file_stem()`.
- `configs_dir` stays `specs/lifecycle-configs/`. The flat-specs doctrine carves this typed family
  out explicitly; do not hoist or nest it.
- No new crate, no new gate directory, no new policy file. All work lands in files that already
  exist, plus zero new tracked files after this one.
- Commit subject: `lom(<scope>): <ruling> — <one-line why>`.
- Cite decision records as the **bare id form** `ADR-<number>`, never as a
  `docs/decisions/ADR-<number>-...` path: that directory holds only the ten live apexes, so a path
  citation of any archived id is a dangling-path finding against an equality-pinned ceiling.
  If a line does not need an id at all, leave it out — see the header note on `citation_lines`.

---

## 5. Units, and why this goal is NOT parallel-shaped

Every unit edits `ci/facade/lifecycle-status/lifecycle-status-policy.json`. It is **the hotfile**.
Running the units in parallel buys three merge conflicts and zero speed, so they are SERIAL. Say so
rather than pretending otherwise.

| # | Unit | Touches | Lands after |
|---|---|---|---|
| U0 | this mapping + its census re-freeze | `docs/plans/lifecycle-lane-disposition/mapping.md`, `governance/check/adr-citation-closure/adr-citation-closure-policy.json` (`files_scanned` 16524 → 16525) | — |
| U1 | DELETE the six | 6 × `specs/lifecycle-configs/*.json` (deleted), the ledger (6 entries), 3 feature-flag reference sites | U0 |
| U2 | RE-ROOT api-stability | `libs/oya-governance-lifecycle-kernel/src/lib.rs`, `specs/lifecycle-configs/api-stability-tier-lifecycle.json`, the ledger (1 entry out, 1 baseline row in) | U1 |
| U3 | LAND | `governance/check/adr-citation-closure/adr-citation-closure-policy.json` (`files_scanned` 16525 → 16519), whole-graph sweep, **the single PR** | U2 |

U2's kernel edit and its two RED proofs may be drafted while U1 is in flight; only the ledger edit
must wait.

**U0 re-freezes its own +1 rather than deferring it to U3.** The census is pinned by EQUALITY, so an
unattributed +1 reds `check-adr-citation-closure-gate` for every downstream unit and pollutes every
base-vs-head failing-set diff with an inherited red that each unit would then have to re-explain.
This is friction that buys quality, so it moves earlier. All OTHER integrator-only bookkeeping is
batched into U3. **No unit opens a pull request. U3 opens exactly one, and only with the gates
already green.**

Git discipline, non-negotiable: commit named paths directly to `impl/lom-dark-lifecycle-lanes`; no
per-unit branch, no merge commit; never `git add -A`; never `git stash`, `git reset` or `git clean`
— other lanes share this repository and a destructive git command destroys their work.

---

## 6. Invariants — must hold after EVERY unit

Checkable against one diff, in isolation.

- **I1 Pairing.** Deleting `specs/lifecycle-configs/<stem>.json` also deletes
  `known_broken_lanes.<stem>` **and** `frozen_violation_baseline.<stem>` in the same commit.
  Deleting a `known_broken_lanes` entry *without* deleting its config means that config's
  `sources[].glob` changed in the same commit.
- **I2 No orphans.** Every lane named anywhere in the ledger has a config on disk; every config on
  disk is either observing ≥1 artifact or listed in `known_broken_lanes`.
- **I3 Ledger honesty.** Every surviving `known_broken_lanes` entry still carries a non-empty
  `resolution` (`parse_policy` lines 225-231 hard-errors otherwise) **and** that resolution is
  executable as written against the tree in this commit.
- **I4 Armed floors.** No lane observes >0 artifacts while listed known-broken; none observes 0
  artifacts while unlisted.
- **I5 Shrink floor.** A lane lit up in this unit either carries ≥1 baselined violation row, or the
  commit message states explicitly that its only floor is `artifacts > 0` and why that was accepted.
- **I6 Census.** No new tracked file under a scanned prefix without same-commit attribution.
- **I7 Canonical JSON.** `specs/**` is governed by `cloud-ci-canonical-json`
  (`governed_roots: ["specs"]`): 2-space indent, LF, trailing newline, literal UTF-8, and
  `sort_keys=false` — so key order is **preserved, never sorted**.
- **I8 Gate green.** `//ci/facade/lifecycle-status:ci-lifecycle-status-gate` is green at the unit's
  HEAD, with the base-vs-head failing-set diff shown.

---

## 7. TRAPS — where the obvious translation is subtly wrong

**T1 — `kind` is decorative.** Writing `"kind": "cargo_metadata_table"` or `"toml_table"` changes
nothing; the file is still read as front matter. This is exactly how `crate-status` looked
configured-and-working while reading nothing for its whole life.

**T2 — a recursive glob's tail is a FILENAME, not a path.** `a/**/b/*.md` compiles, walks `a/`, and
matches zero files forever. Only `<dir>/**/<name>` and `<dir>/<name>` are real.

**T3 — the field match is a prefix at ANY indentation.** With `stage_field: "status"`, the document

```
---
review:
  status: draft
---
```

reads `draft`. A nested key silently wins if it appears before the top-level one. Inspect the SHAPE
of the corpus you re-root onto, not one sample file.

**T4 — an empty value is indistinguishable from an absent one.** `doc_status:` with nothing after it
returns `None` and lands in `stage_not_declared`. Never read that count as "the field is missing".

**T5 — `BaselineStale` is bidirectional.** Fixing violations without shrinking the frozen row in the
SAME commit is a red gate; so is a partial repair. Intended: it denies the ratchet headroom.

**T6 — a zero-count baseline row is a parse ERROR**, not a no-op (`parse_policy` 199-204). An absent
pair already means "must be zero".

**T7 — the gate walks DISK.** Do not create `.omc/plans/milestones/` locally: it flips
`plan-status`/`migration-status` from `DiscoveryFailed` to `Observed` → `KnownBrokenLaneNowLive` →
RED on your machine and green in CI, and you will spend an hour on a phantom. Same class: never
leave scratch `.md` under `docs/` or scratch `.yaml` under `registry/catalog/` — both sit inside a
live lane's glob. Scratch belongs in the session scratchpad, outside the repo.

**T8 — `EVALUATED_AT` is pinned to 2026-01-01 on purpose.** Any surviving `deadline_field` is
evaluated against that date, so `OverdueTransition` is frozen in time. Do NOT "fix" it to the wall
clock: that turns a required lane into a time bomb that reds the branch with no change to the tree.

**T9 — `artifacts_observed` is NOT asserted.** The ledger `_comment` says "2667 docs observed"; the
disk corpus is now **2701** and the lane is still green, because `compare()` asserts only
`artifacts > 0` plus the per-kind counts. Do not "repair" the 2667 — it is stale prose. Do not cite
it as a measurement either.

**T10 — deleting a config moves an EQUALITY-pinned census.**
`governance/check/adr-citation-closure/adr-citation-closure-policy.json` pins
`measured.files_scanned = 16524` (also `citation_lines 8896`, `adr_records 458`). `.json` is a
`scan_extension` and `specs/` is not an exempt prefix, so six deletions → **16518**; with this
document's +1 the net at land is **16519**. Re-freeze with the attribution the file's own
`_corpus_remeasure_2026_08_09` block models: prove the tracked-delta and the scanned-delta AGREE,
then show `citation_lines`, `adr_records` and every finding count UNCHANGED. A narrowed walk and a
genuine delete produce the same number and only one is legitimate. The census is `git ls-files`-based
and skips any path with a **dot-directory** component or a `buck-out`/`target`/`node_modules`/
`vendor` component; it is reproducible in a few lines and was reproduced at 16524 exactly while
writing this.

**T11 — the six doomed configs carry no citations.** Verified: the only `ADR-` occurrences under
`specs/lifecycle-configs/` are `adr-status`'s glob string `docs/decisions/ADR-*.md` and one `_doc`
prose line, and the scanner requires `ADR-` followed by ASCII digits, so a `*` wildcard is not an
id. `citation_lines` must therefore be UNCHANGED by U1 — if it moves, something else moved and you
must find out what before re-freezing.

**T12 — edit policies as TEXT keyed by name.** `lifecycle-status-policy.json` stores its em dashes as
`—` escapes (6 of them). Round-tripping through a JSON dumper re-escapes or un-escapes the whole
file and buries the real diff. Same for the adr-citation-closure policy.

**T13 — one buck2 client per project root.** A neighbouring buck2 in this worktree cancels yours and
reports `The evaluation of this key was cancelled: Rejected`, which reads as a build failure and has
already been misdiagnosed as one twice. Check `ps` before blaming your change. A fresh worktree's
first build is always cold — buck2 shares no cache across worktrees.

**T14 — a lane is not idle because its files are.** A unit writes files for seconds then runs buck2
for minutes. Check process liveness and what the process is building; mtime is the cheap first look,
never the verdict.

**T15 — the goal's own premise is partly unsupported; do not write it into a commit message.** The
"natural experiment" (records retire at 95% because they declare an end-of-life) is **not evidenced
by this gate**: `adr-status`'s glob is SHALLOW (`docs/decisions/ADR-*.md`), so it observes exactly
**10** records, all `Accepted`, zero violations; the 448 retired ones sit in `docs/adr-archive/`
outside it. The gate is also only days old and cannot have caused any retirement rate. The doctrine
may well be right; this repo's records-vs-docs comparison is not the evidence for it. Argue each
ruling from the measured surface instead.

**T16 — `.omc/` is a restricted tracked root**, not a free scratch home for committed artifacts. Four
paths are allowlisted and the policy says do not expand the set. See the header note.

---

## 8. DEFINITION OF DONE for one unit

A reviewer holding only the diff and the commit message applies all nine.

1. **Pairing holds.** Every deleted config path has its `known_broken_lanes` entry AND any
   `frozen_violation_baseline` row deleted in the SAME diff; every deleted ledger entry accompanies
   either a deleted config or a changed `sources[].glob` in the same diff. (I1)
2. **Every number in the diff is reproducible from a command printed in the commit message.** No
   number is carried over from prose, from this document, or from a scout report.
3. **Gate evidence, BOTH sides.** The commit message carries the literal command and the verdict
   lines at the untouched base and at HEAD:
   ```
   buck2 test //ci/facade/lifecycle-status:ci-lifecycle-status-gate \
              //ci/facade/lifecycle-status:ci-lifecycle-status-unittest
   ```
   plus the **diff of the failing-target SETS**, never a count of green. Identical sets = zero
   regressions, which is the correct verdict even when both sides fail. A one-target disagreement is
   chased, not averaged.
4. **A light-up unit shows BOTH floors firing, LIVE.** Temporarily point the re-rooted glob at a
   missing directory, run the gate, capture `lifecycle_status_lane_discovery_failed`; then at a live
   directory with a pattern matching nothing, capture `lifecycle_status_lane_discovered_nothing`.
   Paste both RED outputs into the commit message and confirm the final diff carries the REAL glob.
   A floor nobody has seen fire is the false green it exists to prevent.
5. **The gates governing the PATHS were run, not only the lane's own.**
   `specs/**` edits → `//ci/facade/canonical-json:ci-canonical-json-gate`;
   kernel edits → `//libs/oya-governance-lifecycle-kernel:oya-governance-lifecycle-kernel-unittest`;
   the feature-flag deletion → `//ci/facade/contract-slice-conformance:ci-contract-slice-conformance-gate`
   and `:ci-contract-slice-conformance-fragments-gate`;
   any census move → `//governance/check/adr-citation-closure:check-adr-citation-closure-gate`.
6. **No new tracked file under a scanned prefix** unless the same commit attributes the census move.
7. **Canonical form preserved** in every touched `specs/**/*.json`: 2-space indent, LF, trailing
   newline, literal UTF-8, key order unchanged. (I7)
8. **Decision records cited as bare ids**, never as a `docs/decisions/` path for an archived id.
9. **Git discipline** per §5: named paths, direct commit, no `-A`, no `stash`/`reset`/`clean`, no PR
   from anyone but U3.

### What "done" is NOT

- Not a new gate, crate, floor, or `min_artifacts` key. The mechanism exists; using it is the work.
- Not stamping a default stage onto undeclared artifacts — that converts a measured gap into
  fabricated compliance and is explicitly forbidden for this gate.
- Not a green count. A count cannot distinguish "fixed one, broke one" from "changed nothing".

---

## 9. What this document does NOT cover

- **No buck2 was run to author it.** Every number here is a static read of the base tree
  (`git ls-files` plus a re-implementation of the kernel's reader that reproduces the frozen
  `doc-status` baseline 1921/9 and the frozen census 16524 exactly). U1 owes the first real base run,
  including confirmation that `ci-lifecycle-status-gate` is green at the untouched base.
- The three feature-flag reference sites were located but their own pinned counts were not read; U1
  must check whether repairing them moves anything inside the contract-slice policies.
- Whether `registry/catalog/`'s 750 rows cover all ~895 crates is unattributed (~145-row gap). It
  does not change the ruling — the lane governs the rows that exist — but a later unit that widens
  the catalog will move `unknown_stage: 750` and must shrink or grow that row deliberately.
- No claim is made about other `ci/facade/` gates carrying the same moved-scan-root blindness.
  `ci/facade/scan-root-liveness` exists for that class and its `coverage_bearing_keys`
  (`roots`, `scan_roots`, `crate_root_globs`, `manifest_paths`, `store_manifest_paths`) do **not**
  include lifecycle `sources[].glob`. Registering lifecycle globs there is a defensible follow-up and
  is deliberately OUT of scope: it is added mechanism for a class this gate's own floor already
  covers once the ledger is empty.
