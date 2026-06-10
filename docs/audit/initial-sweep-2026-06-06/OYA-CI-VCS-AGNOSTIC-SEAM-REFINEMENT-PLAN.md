# OYA-CI VCS-AGNOSTIC SEAM REFINEMENT — DESIGN PLAN

STATUS: pending-approval
DOOR: one-way (founder sign-off required before any mutation)
AUTHORED: 2026-06-08
SCOPE: fast-follow refactor that executes AFTER the in-flight hermetic-build executor lands + verifies (branch `cleanup/whole-tree-2026-06-07`, commits `609698931` + `b8e0d7c63`). This doc DOES NOT mutate source; it is the executable plan for the follow-on.

---

## 0. Governing constraint (founder)

> "cloud native, hyperscale pattern, hermeticity in mind — GIT IS TRANSITIONAL."

The bespoke SCM destination may NOT be git-based. The just-landed hermetic boundary already establishes the right *architecture* — a committed, content-addressed **facts snapshot** that the producer and every gate consume as a declared input, never calling the VCS — but it bakes the word **git** into the snapshot's identifiers, file name, CLI flag, crate name, env var, and schema id. Those identifiers leak a transitional implementation detail (git) into what is meant to be the **stable portable contract** (the snapshot) and the **swappable adapter** (the emitter).

This refinement does TWO things and nothing else:

1. **Rename** every `git`-flavored identifier in the oya-ci facts boundary to a VCS-agnostic name (`scm-facts`), so the contract and the adapter no longer name git.
2. **Refactor** the emitter's git-shelling internals behind a pluggable **`ScmFactsSource`** trait, so the git-CLI implementation is explicitly impl #1 (transitional) and a bespoke-SCM impl can plug in later with zero churn to the producer, the gates, or the snapshot shape.

NON-GOALS: this is a pure identifier + internal-seam refactor. It does NOT change the facts *content*, the producer logic, gate logic, or the hermetic execution model. Byte-parity of the six accounting faces is a hard invariant (§5).

---

## 1. Ground truth (verified, READ-ONLY on `/Users/jasonlee/Developer/source`)

Every claim below cites a real file + line from the current `cleanup/whole-tree-2026-06-07` tree.

### 1.1 The committed snapshot — the stable portable contract

File: `cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/git-facts.generated.json` (6,452,477 bytes).

Top-level shape (verified by `grep -nE '^  "[a-z_]+"'`):

| JSON field | line | meaning |
|---|---|---|
| `commit_author_ts_secs` | 2 | sha → author-timestamp (epoch secs) map, filtered to last-touch shas |
| `head_time_secs` | 528 | deterministic "now" = max last-touch ts (tree-content fn, not moving HEAD) |
| `last_touch_commit` | 529 | path → last-touch commit sha |
| `schema` | 47001 | `"oya-ci/git-facts/v1"` |
| `tracked_paths` | 47002 | sorted+deduped tracked-paths universe |

The snapshot is git-shaped only in NAME and in the *values* being commit shas/timestamps. Its fields name a generic concept: "tracked paths", "last touch", "author timestamps", "head time". These translate cleanly to any content-versioned SCM. This is exactly why the snapshot is the right contract boundary.

### 1.2 The emitter — the swappable VCS-source adapter (impl #1 = git CLI)

Crate dir (NOTE the `-app` suffix — the task brief abbreviated it): `cloud/cloud-ci/gates/oya-cloud-ci-git-facts-emitter-app/`
Files: `Cargo.toml`, `BUCK`, `src/main.rs` (no `lib.rs` — it is a `rust_binary` only).

`src/main.rs` shells `git` in exactly three helpers (the entire VCS coupling of the whole pipeline lives here):

- `git_commit_timestamps()` — `git log --format=%H %ct` (main.rs:134-153)
- `git_ls_files()` — `git ls-files` (main.rs:156-174)
- `git_last_touch()` — `git log --name-only --format=commit:%H` (main.rs:196-220)

plus `is_generated_class()` (main.rs:186-190, the convergence filter) and `discover_repo_root()` (main.rs:115-126). CLI flags: `--repo-root`, `--out` (main.rs:47-54). Schema const: `const SCHEMA: &str = "oya-ci/git-facts/v1";` (main.rs:30). It reuses the producer's `to_canonical_json` (main.rs:26,104) for byte-identical formatting.

### 1.3 The producer — already 100% VCS-agnostic

`cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/src/main.rs` NEVER calls git. It:

- declares `struct GitFacts { head_time_secs, tracked_paths, last_touch, commit_author_ts_secs }` (main.rs:59-68) — NOTE: the struct field is `last_touch` but the JSON field is `last_touch_commit` (main.rs:92 maps `value["last_touch_commit"]` → `last_touch`). The rename must keep this deliberate field-name asymmetry intact, or change neither side's content.
- `fn load_git_facts(path) -> Result<GitFacts, CliError>` (main.rs:73) — pure parse of the snapshot.
- CLI flag `--git-facts <path>` (main.rs:147-150), default path main.rs:178-179.
- local bindings `git_facts_path` (main.rs:130), `git_facts` (main.rs:181), passed into `build_staleness_input`, `collect_repo_inputs`, `collect_enforcement_inputs`, `tracked_paths_matching` (main.rs:192-203, 757-906).
- `lib.rs` has ZERO git-facts identifiers (verified: grep empty) — all the naming lives in `main.rs`. Good: the library API needs no rename.

### 1.4 The gate corpus — consume the snapshot only

Every gate test wires `--git-facts <committed snapshot>` into the producer it spawns and never calls git. Verified occurrences (each `let git_facts = ...; .arg("--git-facts").arg(&git_facts)`):

- `oya-cloud-ci-total-accounting-app/tests/total_accounting.rs:205,216,231`
- `oya-cloud-ci-cross-artifact-agreement-app/tests/cross_artifact_agreement.rs:181,188,205`
- `oya-cloud-ci-staleness-reaper-app/tests/staleness_reaper.rs` — local helpers `git_facts_path()`:235, `git_facts_value()`:264, plus `--git-facts` at 200,217 and reads `["head_time_secs"]`:244, `commit_author_ts_secs`:251
- `oya-cloud-ci-automation-ratchet-app/tests/automation_ratchet.rs:195,202,219`
- `oya-cloud-ci-manifest-hygiene-app/tests/manifest_hygiene.rs:34,41,58`
- `oya-cloud-ci-cargo-prefix-app/tests/cargo_prefix.rs:36,43,60`
- `oya-cloud-ci-bnf-layer-suffix-app/tests/bnf_layer_suffix.rs:35,42,59`
- `oya-cloud-ci-firewall-app/tests/firewall.rs:52 (`faces_dir(root).join("git-facts.generated.json")`),62,79`

All gate tests resolve the producer via `OYA_CI_PRODUCER_BIN` env var at runtime (verified firewall.rs:48,53 — "resolved at RUNTIME: under buck2 from `OYA_CI_PRODUCER_BIN`"). This env var is producer-naming, NOT git-naming — it is **out of scope** for this rename.

### 1.5 registry-drift — the tamper-evidence + byte-validation

`oya-cloud-ci-firewall-app/tests/gate_registration.rs:28-31`:
```
const NON_GATE_CRATES: [&str; 2] = [
    "oya-cloud-ci-accounting-registry-app",
    "oya-cloud-ci-git-facts-emitter-app",
];
```
`registry-drift/tests/registry_drift.rs`: const `GIT_FACTS_FACE: &str = "git-facts.generated.json"` (:54), fn `regenerate_git_facts()` (:98), temp name `oya-ci-git-facts-regen-{}.json` (:100), execs `oya-cloud-ci-git-facts-emitter-app` (:118), gate `committed_git_facts_equal_regenerated()` (:184), env opt-in `OYA_CI_GIT_FACTS_REGEN` (:197).

### 1.6 The workflow — the out-of-graph git boundary step

`.github/workflows/oya-ci-required.yml`: step `git-facts-regen (out-of-graph git boundary) + byte-validate` (:189), env `OYA_CI_GIT_FACTS_REGEN: "1"` (:191), builds + runs `//cloud/cloud-ci/gates/oya-cloud-ci-git-facts-emitter-app:...` (:194-196), diffs `git-facts.generated.json` vs `/tmp/git-facts-regen.json` (:197), and the prose comments at :144-146,158,184-188. The producer-emit step (:56) and matrix gate legs (:88-94) do NOT name git-facts — only the dedicated regen step does.

### 1.7 Build wiring

- Root `Cargo.toml:242`: workspace member `"cloud/cloud-ci/gates/oya-cloud-ci-git-facts-emitter-app"`.
- `oya-cloud-ci-git-facts-emitter-app/BUCK:8`: `rust_binary(name = "oya-cloud-ci-git-facts-emitter-app", ...)`.
- All gate `BUCK` files carry the comment "the producer reads the committed git-facts face" (automation-ratchet:36, manifest-hygiene:34, cargo-prefix:39, total-accounting:35, firewall:35, staleness-reaper:35, cross-artifact:36).
- `registry-drift/BUCK:15,18,19,23` prose + `OYA_CI_GIT_FACTS_REGEN`.

### 1.8 The faces themselves reference the snapshot by PATH (the byte-parity hot spot)

`accounting-registry.generated.json:19752` and `gate-baseline.generated.json:6312,23364` contain the literal path string `cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/git-facts.generated.json` AND the emitter crate's paths (`.../oya-cloud-ci-git-facts-emitter-app/BUCK|Cargo.toml|src/main.rs` at accounting:20419-20465, baseline:6341-6343). These are **tracked-path content**: renaming the file and the crate dir CHANGES these path strings inside the faces. This is the source of the byte-parity / 2-commit-settle interaction (§5).

### 1.9 No `/docs` ADR references

`grep -rl git[-_]facts /Users/jasonlee/Developer/source/docs` returns EMPTY. The `OYA-CI-HERMETIC-EXECUTION-DESIGN` name is referenced in code comments but no such file exists under source `/docs` (verified `find -iname '*HERMETIC*'` empty). So there is NO ADR/doc body to rename in source; the only "doc" surface is the inline comments enumerated above. (This linux-side plan and any future ADR should adopt the new vocabulary.)

---

## 2. Rename decision: `git-facts` → `scm-facts`

**Chosen:** `scm-facts` / `scm_facts` / `SCM_FACTS` / `ScmFacts`.

**Justification (vs `repo-facts`):**
- `scm` (Source Control Management) is the precise generic supertype of git, and of a bespoke content-versioned store. It names the *role* (the thing that provides versioning facts) without naming the *impl*.
- `repo-facts` reads as "facts about a repository," but the bespoke hyperscale destination may not be a "repo" at all (it may be a content-addressed object store / fact service). `scm` keeps the focus on the *capability* (source-control facts: tracked paths, last-touch, author time) rather than the *container* (a repo).
- `scm` is the term already in the founder vocabulary ("the bespoke SCM destination"), so it aligns the code with the canon language and the `ScmFactsSource` adapter name (§4) reads naturally.

The schema id becomes `"oya-ci/scm-facts/v1"`. The `v1` is RETAINED (this is not a breaking *shape* change — the shape is byte-identical; only the id string changes). See §5 for why even the schema-string change is content-neutral to the six faces (the schema id lives *inside* the scm-facts snapshot, which is itself a generated-class file the producer nulls out of last-touch — so it never enters the faces' content).

---

## 3. Executable rename + refactor map

Every identifier kind, `from → to`. Apply ONLY after the hermetic-build executor lands + verifies. Order matters — see §6.

### 3.1 Crate directory + crate/binary name

| from | to | kind |
|---|---|---|
| `cloud/cloud-ci/gates/oya-cloud-ci-git-facts-emitter-app/` (dir) | `cloud/cloud-ci/gates/oya-cloud-ci-scm-facts-emitter-app/` | crate-dir |
| `name = "oya-cloud-ci-git-facts-emitter-app"` (Cargo.toml:2,17) | `name = "oya-cloud-ci-scm-facts-emitter-app"` | symbol (crate+bin name) |
| `rust_binary(name = "oya-cloud-ci-git-facts-emitter-app"` (BUCK:8) | `...scm-facts-emitter-app` | symbol (buck target) |

### 3.2 The committed snapshot file

| from | to | kind |
|---|---|---|
| `git-facts.generated.json` (the file) | `scm-facts.generated.json` | file |
| default `--out` path literal `.../git-facts.generated.json` (emitter main.rs:18,66) | `.../scm-facts.generated.json` | file (path literal) |

### 3.3 JSON field names + schema id (INSIDE the snapshot)

| from | to | kind |
|---|---|---|
| `"schema": "oya-ci/git-facts/v1"` (emitter main.rs:30; JSON:47001) | `"oya-ci/scm-facts/v1"` | json-field (value) |
| `last_touch_commit` (JSON key; producer main.rs:92; emitter main.rs:101) | `last_touch_commit` — **KEEP** | json-field |
| `commit_author_ts_secs` | `commit_author_ts_secs` — **KEEP** | json-field |
| `head_time_secs` | `head_time_secs` — **KEEP** | json-field |
| `tracked_paths` | `tracked_paths` — **KEEP** | json-field |

DECISION — KEEP the four data field names. They are already VCS-neutral nouns (a bespoke SCM still has "tracked paths," a "last touch" revision per path, an "author timestamp," and a "head time"). Renaming them would be churn with no agnostic-ness gain AND would force a producer parse change. ONLY the `schema` id string is renamed (it is the one field that literally says "git-facts"). This minimizes blast radius and keeps the data contract stable.

NOTE on `last_touch_commit`: the JSON key literally contains "commit," a git-ish word. Consider it for a *future* v2 (e.g. `last_touch_revision`) but NOT now — changing a data key forces a producer `value["..."]` change and is a breaking shape change requiring `v2`. Logged as an open question (§7), deliberately deferred to preserve byte-parity and a non-breaking `v1`.

### 3.4 Producer (`oya-cloud-ci-accounting-registry-app/src/main.rs`)

| from | to | kind |
|---|---|---|
| `--git-facts` CLI flag (main.rs:147) | `--scm-facts` | cli-flag |
| `struct GitFacts` (main.rs:59) | `struct ScmFacts` | symbol |
| `fn load_git_facts` (main.rs:73) | `fn load_scm_facts` | symbol |
| local `git_facts_path` (main.rs:130,149,178,181) | `scm_facts_path` | symbol |
| local `git_facts` (main.rs:181,192,196,203) | `scm_facts` | symbol |
| param `git_facts: &GitFacts` (main.rs:314,757,842,903) | `scm_facts: &ScmFacts` | symbol |
| doc comment usage `[--git-facts <path>]` (main.rs:13,17-18) + "git-facts face" prose (main.rs:57,70,175,898 etc.) | `--scm-facts`, "scm-facts face" | symbol (doc) |
| `value["last_touch_commit"]` (main.rs:92) | UNCHANGED (key kept per §3.3) | json-field |

BACK-COMPAT for `--git-facts`: NOT required. The flag has exactly one set of callers (the gate corpus tests in §1.4 + workflow), all renamed in the same atomic change. No external consumer exists (`publish = false`). Do a hard rename, not an alias.

### 3.5 Emitter internals (`oya-cloud-ci-scm-facts-emitter-app/src/main.rs`)

| from | to | kind |
|---|---|---|
| `const SCHEMA = "oya-ci/git-facts/v1"` (main.rs:30) | `"oya-ci/scm-facts/v1"` | symbol (value) |
| "git-facts emitter" / "git-facts face" doc prose (main.rs:1,6,29,73,131,180-194) | "scm-facts emitter" / "scm-facts face" | symbol (doc) |
| `git_commit_timestamps` / `git_ls_files` / `git_last_touch` fns | refactor into the `GitScmFactsSource` impl (§4) — NOT a flat rename | symbol |

### 3.6 registry-drift + gate_registration

| from | to | kind |
|---|---|---|
| `NON_GATE_CRATES[1] = "oya-cloud-ci-git-facts-emitter-app"` (gate_registration.rs:30) | `"oya-cloud-ci-scm-facts-emitter-app"` | symbol |
| `const GIT_FACTS_FACE = "git-facts.generated.json"` (registry_drift.rs:54) | `const SCM_FACTS_FACE = "scm-facts.generated.json"` | symbol + file |
| `fn regenerate_git_facts` (registry_drift.rs:98) | `fn regenerate_scm_facts` | symbol |
| temp name `oya-ci-git-facts-regen-{}.json` (registry_drift.rs:100) | `oya-ci-scm-facts-regen-{}.json` | symbol |
| exec `"oya-cloud-ci-git-facts-emitter-app"` (registry_drift.rs:118) | `"oya-cloud-ci-scm-facts-emitter-app"` | symbol |
| `fn committed_git_facts_equal_regenerated` (registry_drift.rs:184) | `fn committed_scm_facts_equal_regenerated` | symbol |
| env `OYA_CI_GIT_FACTS_REGEN` (registry_drift.rs:197; BUCK:23) | `OYA_CI_SCM_FACTS_REGEN` | env-var |

### 3.7 Gate corpus tests (the `--git-facts` wiring; §1.4)

For EACH of the 8 gate test files: rename local `git_facts` / `git_facts_path` / `git_facts_value` → `scm_facts*`, the `.arg("--git-facts")` → `.arg("--scm-facts")`, the path literal `git-facts.generated.json` → `scm-facts.generated.json`, and the "git-facts face" doc comment → "scm-facts face".

| file | identifiers | kind |
|---|---|---|
| total_accounting.rs:203,205,216,231 | `--git-facts`, `git_facts`, path | cli-flag/symbol/file |
| cross_artifact_agreement.rs:179,181,188,205 | same | cli-flag/symbol/file |
| staleness_reaper.rs:188,190,200,217,234-268 | `git_facts_path`,`git_facts_value`,`--git-facts`, path | cli-flag/symbol/file |
| automation_ratchet.rs:193,195,202,219 | same | cli-flag/symbol/file |
| manifest_hygiene.rs:32,34,41,58 | same | cli-flag/symbol/file |
| cargo_prefix.rs:34,36,43,60 | same | cli-flag/symbol/file |
| bnf_layer_suffix.rs:33,35,42,59 | same | cli-flag/symbol/file |
| firewall.rs:50,52,62,79 | `--git-facts`, `git_facts`, path | cli-flag/symbol/file |

### 3.8 Workflow + root manifest + BUCK comments

| from | to | kind |
|---|---|---|
| root `Cargo.toml:242` member path | `cloud/cloud-ci/gates/oya-cloud-ci-scm-facts-emitter-app` | crate-dir |
| workflow step name `git-facts-regen (out-of-graph git boundary)` (:189) | `scm-facts-regen (out-of-graph SCM boundary)` | symbol |
| workflow env `OYA_CI_GIT_FACTS_REGEN` (:191) | `OYA_CI_SCM_FACTS_REGEN` | env-var |
| workflow buck target refs (:194-195) | `...oya-cloud-ci-scm-facts-emitter-app...` | symbol |
| workflow diff path + temp `/tmp/git-facts-regen.json` (:196-198) | `scm-facts.generated.json`, `/tmp/scm-facts-regen.json` | file |
| workflow prose `git-facts face` (:144-146,158,184-188) | `scm-facts face` | symbol (doc) |
| emitter `Cargo.toml` + `BUCK` prose `git-facts.generated.json` / `git-facts-regen` (Cargo.toml:12,23; BUCK:3,5,13) | `scm-facts*` | symbol (doc) |
| all 7 gate `BUCK` comments "committed git-facts face" | "committed scm-facts face" | symbol (doc) |
| registry-drift `BUCK:15,18,19,23` prose + env | `scm-facts*`, `OYA_CI_SCM_FACTS_REGEN` | symbol/env-var |

### 3.9 Out of scope (do NOT rename)

- `OYA_CI_PRODUCER_BIN` — producer naming, not git naming (firewall.rs:53 etc.).
- The four data JSON keys (§3.3 KEEP).
- The producer's `lib.rs` — zero git-facts identifiers (verified).
- `discover_repo_root` / `specs/root-hub-pointers.json` marker — root discovery, not VCS facts.

---

## 4. The pluggable VCS-source ADAPTER — `ScmFactsSource`

### 4.1 Boundary

Today the emitter's three `git_*` helpers ARE the only VCS coupling in the entire pipeline (§1.2). The refactor extracts that coupling behind a trait so:

- **Producer + all gates stay 100% VCS-agnostic** — unchanged; they only read the snapshot (§1.3, §1.4). The adapter lives ENTIRELY inside the emitter crate.
- **The git CLI becomes impl #1, explicitly transitional.** A bespoke-SCM impl plugs in later by adding one struct that implements the trait — zero change to the snapshot shape, the producer, or the gates.

### 4.2 Trait shape

Define in the emitter crate (promote `src/main.rs` to `src/main.rs` + `src/lib.rs` so the trait is unit-testable, or a `src/source.rs` module). The trait emits exactly the three primitives the snapshot needs — the same three the git helpers compute today:

```rust
/// A source of source-control facts for the oya-ci hermetic boundary.
///
/// The ONE seam where the pipeline touches a VCS. Implementations run OUTSIDE the
/// buck2 action graph (CI pre-step + local regen hook). The producer and every gate
/// consume only the emitted snapshot, so they never see this trait — swapping the
/// impl is invisible to the rest of oya-ci.
///
/// GIT IS TRANSITIONAL: `GitCliScmFactsSource` is impl #1. A bespoke-SCM impl plugs
/// in later by implementing this trait; the snapshot shape and all consumers are unchanged.
pub trait ScmFactsSource {
    /// The tracked-paths universe (sorted + deduped). git impl: `git ls-files`.
    fn tracked_paths(&self) -> Result<Vec<String>, ScmFactsError>;

    /// path -> last-touch revision id, EXCLUDING generated-class paths (the convergence
    /// filter is the caller's invariant, not the source's — see note). git impl:
    /// `git log --name-only --format=commit:%H`, newest-first first-seen wins.
    fn last_touch(&self) -> Result<BTreeMap<String, String>, ScmFactsError>;

    /// revision id -> author timestamp (epoch secs). git impl: `git log --format=%H %ct`.
    fn revision_author_timestamps(&self) -> Result<BTreeMap<String, u64>, ScmFactsError>;
}
```

The emitter's `run()` keeps the derivation logic (the CONVERGENCE math at main.rs:73-103: filter `commit_author_ts_secs` to last-touch shas, `head_time_secs = max`, canonical-json serialize) UNCHANGED — that math is VCS-agnostic and must stay verbatim for byte-parity. Only the three raw reads move behind the trait.

Decision on the generated-class filter: KEEP it in the git impl's `last_touch()` (mirroring today's `is_generated_class` skip at main.rs:211-212) rather than hoisting it into shared derivation, because a future bespoke SCM might expose last-touch differently and the "skip generated-class so the snapshot converges" rule is a property of how we *read* per-path revisions. Document it as a contract the trait's `last_touch` MUST honor. (Alternatively hoist the filter into `run()` to guarantee every impl converges — logged as open question §7; either is byte-neutral today because there is only the git impl.)

### 4.3 Single impl + selection

```rust
/// Impl #1 (TRANSITIONAL): shells `git` at the build-graph edge. The only place in the
/// whole pipeline allowed to exec git (ADR-0515 D3 narrow exception).
pub struct GitCliScmFactsSource { repo_root: PathBuf }
```

Selection in `run()`: for now a single hardcoded `GitCliScmFactsSource::new(repo_root)`. Do NOT build a registry/factory yet (YAGNI — there is one impl). The seam is the *trait*, not a plugin system. When impl #2 arrives, selection becomes a match on a `--scm-kind git|bespoke` flag (defaulting to `git`) or an env/config value — but that flag is added WITH impl #2, not speculatively now. This keeps the refactor minimal and the byte output identical.

### 4.4 What the refactor must NOT change

- The exact git invocations (verbatim args) — byte-parity (§5).
- The `to_canonical_json` formatting reuse (emitter main.rs:26,104).
- The derivation/convergence math (main.rs:73-103).
- The snapshot's four data field names (§3.3).

---

## 5. BYTE-PARITY invariant + the 2-commit settle

### 5.1 Content vs identifiers

The rename touches IDENTIFIERS only (file name, crate name, flag, symbols, schema-id string, env var). It must NOT touch the facts CONTENT. The six accounting faces and their baselines must stay **byte-identical**, preserving the verified counts:

| face | count |
|---|---|
| bnf-layer-suffix | 79 |
| manifest-hygiene | 233 |
| cargo-prefix | 1 |
| brand | 4494 |
| cross-artifact | 168 |
| automation | 153 |
| staleness | 64 |

The `schema` string change (`oya-ci/git-facts/v1` → `oya-ci/scm-facts/v1`) is content-neutral to these faces because that string lives *inside* the scm-facts snapshot, which is itself a `.generated.json` (generated-class) file. The producer NEVER reads the snapshot's `schema` field into any face (it reads only `head_time_secs`, `tracked_paths`, `last_touch_commit`, `commit_author_ts_secs` — main.rs:83-110), and the snapshot file is nulled out of `last_touch` by `is_generated_class` (emitter main.rs:186-189). So the schema-id change has ZERO effect on face content.

### 5.2 Why the rename DOES shift tracked file paths (the churn)

The rename changes two TRACKED path strings (and the emitter crate's three tracked files):

- `.../git-facts.generated.json` → `.../scm-facts.generated.json`
- `.../oya-cloud-ci-git-facts-emitter-app/{BUCK,Cargo.toml,src/main.rs}` → `.../oya-cloud-ci-scm-facts-emitter-app/{...}`

These path strings appear as DATA inside:
- `git-facts.generated.json` itself — `tracked_paths` + `last_touch_commit` keys (the snapshot lists the whole tree, including itself and the emitter crate).
- `accounting-registry.generated.json:19752,20419-20465` and `gate-baseline.generated.json:6312,6341-6343,23364` (verified §1.8).

So the faces ARE expected to change in exactly these path strings — that is NOT a parity violation, it is the rename correctly propagating. The seven face COUNTS above (which are about brand/bnf/manifest/etc. *findings*, not raw path lists) stay identical because renaming a file does not change how many findings each gate produces — only WHICH path is named. The reviewer must confirm: counts unchanged; only the renamed path strings differ.

### 5.3 The 2-commit settle (last_touch_commit churn)

git's `git ls-files` / `git log` treat a rename as delete-old + add-new, so the renamed paths get a NEW `last_touch_commit` = the rename commit. Per the emitter's own CONVERGENCE doc (main.rs:73-85), the snapshot is a pure function of the COMMITTED TREE STATE — so:

1. **Commit A** (the rename): mutate all identifiers + move files. After commit A, the *committed* tree has the new paths but the snapshot still needs regeneration. Run the emitter (`scm-facts-regen`) — it now sees the new paths, assigns them last-touch = commit A, recomputes `head_time_secs`. The faces regenerate with the new path strings. This is the standard "produce → settle" step the pipeline already does for any tree change.
2. **Commit B** (the settle): commit the regenerated `scm-facts.generated.json` + the regenerated faces. After commit B the snapshot is a fixpoint again (the renamed `.generated.json` files are generated-class, so they are nulled from `last_touch` and do NOT re-churn — main.rs:180-185 guarantees convergence in exactly this 2-commit window).

This is the SAME 2-commit settle the emitter's docstring already calls out ("git-facts converges in the standard 2-commit settle," main.rs:185). The rename is just one more tree-state change that rides that existing mechanism. `registry-drift`'s `committed_scm_facts_equal_regenerated` must be GREEN after commit B (regenerate == committed).

### 5.4 Verification gate (reviewer lane, separate context)

Before claiming done, a separate verifier must confirm, with evidence:
1. `cargo build` + `cargo test` green across the renamed workspace.
2. `cargo clippy` clean.
3. The 7 face counts above are byte-unchanged (diff shows ONLY renamed path strings + schema-id, nothing else).
4. `registry-drift` `committed_scm_facts_equal_regenerated` GREEN (post commit B).
5. `gate_registration` GREEN (NON_GATE_CRATES updated; emitter still recognized as non-gate).
6. Zero residual `git-facts` / `git_facts` / `GIT_FACTS` tokens in source (excluding `/target`, `/buck-out`, `/.git`): `grep -rE 'git[-_]facts|GIT_FACTS' --include='*.rs' --include='*.toml' --include='BUCK' --include='*.yml' --include='*.json' | grep -vE '/target/|/buck-out/'` returns EMPTY.
7. buck2 build of the renamed emitter target green (workflow step references resolve).

---

## 6. Execution steps (ordered; one atomic logical change)

1. GATE: confirm the in-flight hermetic-build executor has LANDED + VERIFIED on `cleanup/whole-tree-2026-06-07` (commits `609698931`+`b8e0d7c63` present and registry-drift green). Do not start until then. Get founder go (door:one-way).
2. Branch off the post-landing tip in an ISOLATED worktree (never mutate the warm tree the executor owns).
3. `git mv` the crate dir `oya-cloud-ci-git-facts-emitter-app` → `oya-cloud-ci-scm-facts-emitter-app` and `git mv` the snapshot `git-facts.generated.json` → `scm-facts.generated.json` (preserves history; lets git track the rename).
4. Refactor the emitter: extract `ScmFactsSource` trait + `GitCliScmFactsSource` impl (§4); rename `SCHEMA` value; update doc prose. Keep derivation math + git invocations verbatim.
5. Apply the producer rename (§3.4): flag `--git-facts`→`--scm-facts`, `GitFacts`→`ScmFacts`, `load_git_facts`→`load_scm_facts`, locals/params, doc. Keep the four JSON data keys.
6. Apply registry-drift + gate_registration renames (§3.6); update `NON_GATE_CRATES` entry, `SCM_FACTS_FACE`, fns, env var.
7. Apply all 8 gate-test renames (§3.7): `--scm-facts`, locals, path literals, doc comments.
8. Update root `Cargo.toml` member, all BUCK targets/comments, and the workflow step name/env/target/diff-path/prose (§3.8).
9. Build/test/clippy green (cargo); fix any missed identifier.
10. Run the renamed emitter (`scm-facts-regen`) → regenerate the snapshot + faces. COMMIT A (rename + code) then regenerate then COMMIT B (settle: regenerated snapshot + faces). Per §5.3.
11. Verify byte-parity of the 7 counts + registry-drift green + zero residual tokens + buck2 build (§5.4) in a SEPARATE verifier lane.
12. NEVER push. Signed atomic commits only, on founder go.

---

## 7. Open questions (for founder / reviewer)

1. **`last_touch_commit` JSON key** still says "commit" (git-ish). Defer to a future v2 (`last_touch_revision`) to keep this change non-breaking + byte-parity, or rename now and bump to `oya-ci/scm-facts/v2`? Recommendation: DEFER (this doc assumes defer).
2. **Generated-class convergence filter**: keep inside the git impl's `last_touch()` (per-impl contract) or hoist into the emitter's shared `run()` derivation (guaranteeing every future impl converges regardless of how it reads last-touch)? Byte-neutral today; matters when impl #2 lands. Recommendation: hoist into `run()` for a stronger cross-impl invariant — but flagged for founder call.
3. **Schema id `v1` retention**: confirm that changing the schema-id STRING while keeping `v1` is acceptable (shape unchanged, only the namespace word changes). Alternative: treat any id-string change as a `v2`. Recommendation: keep `v1` (shape is identical; consumers ignore the field).
4. **`--scm-kind` selection flag**: confirm we do NOT add it speculatively now (YAGNI) and only introduce it with impl #2. Recommendation: defer.
5. **ADR**: should this refinement get its own ADR (the canon is ADR-SSOT)? The hermetic boundary's design name `OYA-CI-HERMETIC-EXECUTION-DESIGN` is referenced in code but has no source `/docs` file — confirm whether to author that design doc + an amendment ADR adopting the `scm-facts` vocabulary as part of this work.
6. **Worktree isolation timing**: the source tree is currently owned by a mid-commit executor. Confirm the executor has fully released the tree before step 2.

---

## 8. Summary

The just-landed hermetic boundary already put the *right architecture* in place — a content-addressed facts snapshot as the stable portable contract, an out-of-graph emitter as the only VCS toucher, and a producer + gate corpus that are already 100% VCS-agnostic. This refinement removes the one remaining leak of the transitional impl (the word "git") from the contract + adapter identifiers (rename to `scm-facts`), and formalizes the emitter's git-shelling internals behind a `ScmFactsSource` trait whose git-CLI impl is explicitly impl #1. The data field names + derivation math + execution model are untouched, so the six accounting faces stay byte-identical (counts: bnf=79, manifest=233, cargo-prefix=1, brand=4494, cross-artifact=168, automation=153, staleness=64), with the rename's only legitimate face delta being the renamed path strings, settled in the existing 2-commit window. When the bespoke (non-git) SCM destination arrives, it plugs in as impl #2 with zero churn to the producer, the gates, or the snapshot shape — honoring "git is transitional."
