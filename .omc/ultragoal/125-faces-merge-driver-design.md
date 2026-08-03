# Task #125 — Faces Merge-Conflict Scalability Defect: Design

Status: DESIGN (read-only analysis; no code changed). Author: architect lane.
Scope: eliminate the single-threaded-merge defect on born-accounting generated faces.

---

## 0. TL;DR / Decision

**Chosen layering (build in this order):**

1. **v1 — local faces merge driver** (`oya-faces-merge-driver`): a Rust binary wired via
   `.gitattributes <faces-glob> merge=oya-faces` that, during a LOCAL merge/rebase, takes
   *theirs* for the faces and re-materializes them deterministically from the post-merge
   working tree, **fail-closed**. This automates exactly the manual
   `git checkout --theirs <faces> && materialize && verify` dance agents do today. It is the
   v1 win because the agent/local-rebase flow IS the current pain (the 5 serial rebases of
   #780/#809/#810/#811/#812).

2. **v1 companion — hermetic git-config bootstrap** (`oya-repo-hooks-bootstrap`): a checked-in
   Rust binary + a buck2 build-graph wiring that sets `git config merge.oya-faces.driver`
   per-clone and per-CI-runner with **zero manual step**. This is the crux of universality:
   `.gitattributes` alone does NOT activate a merge driver. Without this, v1 is a README full
   of manual `git config` lines (exactly the trap the existing `oya-cargo-lock-merge-driver`
   README falls into — see `tools/oya-cargo-lock-merge-driver-app/README.md`).

3. **v2 — server-side merge-queue regenerate** (extend `oya/ci-tide`): the durable fix. Local
   merge drivers do NOT run on the GitHub squash-merge button. The merge queue must rebase the
   candidate onto the projected tip, **re-materialize the faces**, and re-evaluate freshness +
   registry-drift on that rebased tree before admitting. This is where the false-green window
   actually closes for real (see §1 and §6 for why this is mandatory, not optional).

**Plus a structural mitigation that shrinks the conflict class regardless of the above
(recommended, §6): shard `accounting-registry.generated.json` by top-level capability** so two
PRs touching disjoint capabilities never collide on the big face. This is the
highest-leverage, lowest-risk change and it composes with both the driver and the queue.

**Honest headline:** the merge driver is necessary-but-not-sufficient, and it is subtler than
the cargo-lock precedent because **the faces embed git-history facts (`last_touch_commit` SHA
per row), not just working-tree content.** That single fact (proven below) is the spine of this
whole design and the reason the naive "merge-driver fixes it" framing is incomplete.

---

## 1. Root cause (precise)

The faces are produced by a two-stage pipeline (entrypoint:
`infra/ci/materialize-cloud-ci-generated-faces.sh`):

1. `oya-cloud-ci-scm-facts-emitter-app` — **the single sanctioned `git` boundary**
   (ADR-0515 D3; see its `src/main.rs` header). It shells `git ls-files`,
   `git log --name-only --format=commit:%H` (path → last-touch SHA), and
   `git log --format=%H %ct` (SHA → author timestamp), writing `scm-facts.generated.json`.
2. `oya-cloud-ci-accounting-registry-app-bin` — a **PURE** function of
   `{scm-facts face + oya-ci.toml + declared tracked tree}` (its `main.rs` header is explicit:
   "NO ambient git — the producer never shells out"). It emits the other faces.

**The defect has two compounding layers:**

- **(a) The big face enumerates the whole tree.** `accounting-registry.generated.json` has one
  row per tracked path (~18,485 rows; the on-disk file is ~12.6 MB / ~391k lines —
  `wc -l` confirmed). ANY change to the tracked-path universe (add/remove/rename a file)
  rewrites large regions of this file, so any two PRs that touch *different* files still
  collide here. Same for `gate-baseline.generated.json` (~3.8 MB) and
  `scm-facts.generated.json` (~3.3 MB).

- **(b) The rows carry git-history facts, not just content.** Confirmed by inspection: every
  row has `"last_touch_commit": "<40-hex-sha>"` (e.g. row 0 →
  `2a0d41913d2f47834414aea3fad59bbdfdfeaa27`), plus TTL/aging derived from commit timestamps.
  So the face value depends on the *commit graph*, which is only finalized AFTER the merge
  commit exists. During an in-progress local merge/rebase the merge commit SHA does not exist
  yet and the last-touch SHA for any path touched on either side is unsettled.

Layer (a) is why merges conflict. Layer (b) is why a *local* merge driver can only ever be an
approximation — and why fail-closed determinism (§4) is non-negotiable.

**Why the repo is single-threaded for merges:** merge #1 advances `dev`, which rewrites the
faces; every other open PR now conflicts on those multi-MB JSON files and must rebase +
re-materialize + re-verify (a full ~30-min CI cycle each). The cost is O(open-PRs) serial
cycles. This is precisely the "anti-scalable pattern" the founder bar
(`pipeline-four-property-bar`) exists to kill.

---

## 2. What exists already (precedent + reuse)

This codebase already has the building blocks; the design is mostly *assembly + one missing
piece*, not green-field.

| Asset | Path | Role in this design |
|---|---|---|
| cargo-lock merge driver | `tools/oya-cargo-lock-merge-driver-app/` (`src/lib.rs::merge_lockfiles(base,ours,theirs)`, `src/main.rs` takes `%O %A %B`) | **Shape precedent** for the new driver binary (arg contract, exit-1-on-conflict, overwrite `%A`). |
| friction-ledger merge driver | `tools/oya-friction-ledger-merge-driver-app/` | Second precedent; structural JSONL merge. |
| `.gitattributes` driver wiring | `.gitattributes` (`Cargo.lock merge=cargo-lock`, `evidence/audit-chain.jsonl merge=union`) | The glob-line pattern to extend with the faces glob. |
| Producer library API | `cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/src/lib.rs` (`build_registry`, `build_decision_crosswalk`, `build_enforcement_inventory`, `build_gate_baseline`, `to_canonical_json`, `Policy::from_config`) | **The Rust libraries the driver calls directly** (no-shell, §3). |
| Producer stdout regen | `.../accounting-registry-app/src/main.rs` (`--stdout --face <name>`) | Single-face regen path the freshness gate already uses. |
| scm-facts emitter | `cloud/cloud-ci/gates/oya-cloud-ci-scm-facts-emitter-app/src/main.rs` | The one git boundary; the driver MUST reuse it, not re-implement git. |
| Existing regenerate+settle engine | `oya-cloud-ci-freshness-app/src/lib.rs` (`regenerate_faces_with_buck2`, `settle_regenerated_faces`, `assert_non_face_tree_clean`, `FaceSettleMode`) | **Reuse this** — the driver's regen step is morally `regenerate_faces` minus the buck2 indirection. |
| face-settle bin | `.../oya-cloud-ci-freshness-app/src/bin/oya-cloud-ci-face-settle.rs` | Existing CLI that does check/settle/settle-and-commit; the driver is the *non-interactive merge-time* sibling. |
| Freshness gate (the verifier) | `oya-cloud-ci-freshness-app/src/lib.rs` (`FindingCode::GeneratedFaceStale`, `check_repo_with_regenerated_faces`) | The gate the driver's output MUST satisfy (committed == regenerated). |
| registry-drift gate | `cargo test -p registry-drift` (CI step in `oya-ci-required.yml`) | Byte-parity gate the driver MUST satisfy. |
| Policy-as-data control plane | `registry/generated-artifact-control-plane.json` (`artifacts[].path`, `.generator.target`, `.parameters.face`, `merge_policy: never-manual-merge-regenerate-from-source-tree`) | **The universality surface** — the faces-glob + producer entrypoint are already declared here as DATA. The driver reads this, not hardcoded paths. |
| Merge queue substrate | `oya/ci-tide/crates/{oya-ci-tide-kernel,oya-ci-tide-github-adapter,oya-ci-tide-app}` | The v2 home. Today `is_mergeable` gates on "forge reports mergeable (no conflicts)" — i.e. it currently *refuses* faces-conflicting PRs rather than resolving them. |
| CI already has `merge_group` | `.github/workflows/oya-ci-required.yml` line 28 (`on: merge_group:`) | The merge-queue event already triggers full face re-materialization. v2 leverages this. |

**Key precedent caveat:** `tools/oya-cargo-lock-merge-driver-app/README.md` documents activation
as **manual** `git config merge.cargo-lock.driver "..."`. A repo-wide grep for who sets
`git config merge.*.driver` returns **nothing in tracked source** — the only hit is one
developer's local `~/.gitconfig` (the friction-ledger driver, pointing at a buck-out path in a
worktree). **The bootstrap problem is currently UNSOLVED for the existing drivers too.** Solving
it here (§3.2) retroactively fixes cargo-lock and friction-ledger activation as well.

---

## 3. v1 — the local faces merge driver

### 3.1 Mechanism + reach (Q1)

`.gitattributes` gains the policy-driven glob:

```
# Generated born-accounting faces are controller outputs, never a contributor merge surface.
# On local merge/rebase, regenerate from the merged tree instead of conflicting (fail-closed).
cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/*.generated.json merge=oya-faces
specs/reorg/move-manifest.generated.json                                   merge=oya-faces
```

(Cargo.lock keeps its existing `merge=cargo-lock` driver — that one IS a pure content merge and
its structural driver is correct; do not fold it in.)

Git invokes the driver as `oya-faces-merge-driver %O %A %B %P` (base, ours, theirs, pathname).
The driver does NOT try to 3-way-merge the JSON. It:

1. Takes **theirs** (`%B`) as the provisional content for `%A` (cheap, avoids a spurious
   conflict marker) — but this value is *discarded*; it exists only so git records "resolved".
2. After git finishes applying all per-file drivers and the merge/rebase lands in the working
   tree, a **post-merge settle** re-materializes ALL faces from the merged tree and overwrites
   them. (Per-file merge drivers run one-file-at-a-time and cannot see the whole merged tree, so
   the actual regeneration is driven post-merge — see §3.3 for the exact wiring.)

**WHAT THIS HELPS:** the agent/local rebase-and-regenerate loop. Instead of the manual
`rebase → checkout --theirs faces → materialize → diff-check`, the driver + post-merge hook do
it automatically and fail-closed. This removes the per-PR human step that makes the serial
grind painful.

**WHAT THIS DOES NOT HELP (state plainly):** the GitHub server-side squash/merge button does
NOT run local merge drivers or local hooks. `merge=oya-faces` is inert on github.com. The PR
"this branch has conflicts" banner will still appear for two PRs that both touch the big face
until v2 lands. v1 makes the *local agent flow* one-command; v2 makes the *server merge* truly
parallel. See §6 for the structural mitigation that reduces how often the banner even appears.

### 3.2 No-shell doctrine — which producer libraries the driver calls (Q3)

The driver is a Rust binary calling producer **libraries** directly. It does NOT shell to
`buck2` or to `materialize-cloud-ci-generated-faces.sh`.

Direct library calls (pure, no shell):
- `oya_cloud_ci_accounting_registry_app::Policy::from_config` /`::from_bundled`
- `::build_registry`, `::build_decision_crosswalk`, `::build_enforcement_inventory`,
  `::build_gate_baseline`, `::to_canonical_json`
- `oya_ci_config_kernel::OyaCiConfig` (load `oya-ci.toml`)
- the same library entrypoints `main.rs` uses, refactored behind one
  `materialize_all_faces(repo_root, scm_facts) -> Vec<(filename, bytes)>` helper so the driver
  and the existing producer `main.rs` share ONE code path (no second materializer to drift).

**Irreducible shell glue → name it for the ledger:** the `scm-facts` emitter shells `git`
(`git ls-files`, `git log`). Per ADR-0515 D3 this is the *single sanctioned git boundary* and
must stay shell-out. The driver therefore invokes the **scm-facts-emitter binary** (one process
spawn) to obtain `scm-facts.generated.json` for the merged tree, then runs the pure producer
libraries in-process. The one process spawn (emitter) is the irreducible glue:

> **Irreducible-glue ledger entry:** `oya-faces-merge-driver` spawns
> `oya-cloud-ci-scm-facts-emitter-app` (1 subprocess) because git history facts are only
> obtainable through the sanctioned ADR-0515-D3 git boundary. No other shell. This matches the
> existing `regenerate_faces_with_buck2` pattern, minus the buck2 indirection (the driver
> resolves the emitter binary path from a build-time-injected env/config, not a `buck2 build`).

### 3.3 The git-config bootstrap (Q2) — the crux of universality

**Problem:** `.gitattributes merge=oya-faces` names a driver; it does NOT define it. The driver
is only active if `merge.oya-faces.driver` exists in *that clone's* git config. This is per-clone
local state, not carried by the repo, and absent on fresh CI runners. Same gap that leaves the
existing cargo-lock/friction-ledger drivers un-activated.

**Solution — `oya-repo-hooks-bootstrap` (checked-in Rust binary) + automatic invocation:**

1. **One bootstrap binary** (`tools/oya-repo-hooks-bootstrap-app`) that, given a repo root:
   - resolves the built driver binary path,
   - runs `git config --local merge.oya-faces.name "..."` and
     `git config --local merge.oya-faces.driver "<abs-path> %O %A %B %P"`,
   - installs a `post-merge` + `post-rewrite` hook (the post-merge settle, §3.1) by setting
     `git config core.hooksPath .githooks` and shipping `.githooks/` as checked-in Rust-invoking
     shims (a 2-line shim that calls the bootstrap binary's `settle` subcommand — irreducible
     glue, ledgered),
   - is **idempotent** (re-running is a no-op),
   - reads the driver/glob set from `registry/generated-artifact-control-plane.json` so it is
     repo-neutral.

2. **Automatic invocation — three lanes, zero manual step:**
   - **Local dev:** a checked-in `.githooks/post-checkout` (set via `core.hooksPath`, which IS
     repo-relative once bootstrapped once) re-runs the idempotent bootstrap. Cold-start
     chicken-and-egg (`core.hooksPath` itself isn't set on first clone) is solved by making the
     **buck2 build of any target depend on a `:repo-hooks-bootstrapped` genrule** that runs the
     bootstrap as a side-effect of the developer's first `buck2 build` — which the buck2-first
     doctrine guarantees every contributor runs. (Memory: `buck2-primary-build`.)
   - **CI runners:** add one step to `oya-ci-required.yml` *before* any checkout-consuming gate:
     `Run repo hooks bootstrap` → builds + runs `oya-repo-hooks-bootstrap-app .`. Cheap,
     hermetic, idempotent. This also makes CI's own local merges (if any) safe.
   - **Merge queue (v2):** the tide worker runs the same bootstrap in its workspace.

   **Why not `git config --global`?** Rejected: pollutes the developer's machine, not hermetic,
   not per-repo. `--local` written by an idempotent checked-in binary is the hermetic answer.

   **Why not rely on `.gitattributes` alone?** It cannot define a driver — git design.

   **Why a genrule dependency rather than a documented manual step?** The founder bar is
   AUTOMATED, not flag-only (memory `enforcement-layering`). A README `git config` line (the
   cargo-lock status quo) is the incomplete state we are explicitly fixing.

### 3.4 Fail-closed + determinism contract (Q4)

The driver/settle MUST satisfy ALL of:

1. **Regenerate from the merged working tree only** (after both sides applied). Reuse
   `assert_non_face_tree_clean`-style invariants: the non-face tree must be the post-merge
   content before faces are computed.
2. **Determinism self-check:** materialize twice; if byte output differs between runs → exit
   non-zero, leave the conflict, emit a diagnostic. (`to_canonical_json` already gives a
   sorted/stable encoding; this is the belt-and-suspenders check.)
3. **On ANY error** (emitter spawn fails, producer error, IO error, config missing, scm-facts
   malformed) → **exit non-zero and leave the git conflict in place.** NEVER write a partial or
   guessed faces file. A half-written 12 MB face that happens to satisfy nothing is a
   false-green vector; the fail-closed default is "let the human/queue see the conflict."
4. **No clock/rand/network.** The only non-determinism source is git history, which is captured
   deterministically by scm-facts (`head_time_secs` = max last-touch ts, not wall-clock — see
   the emitter). The driver adds none.
5. **Output must satisfy the verifiers byte-for-byte:** after settle, `freshness`
   (`GeneratedFaceStale`) and `registry-drift` (byte-parity) must be green on the result. This
   is the acceptance oracle (§7 test plan).

**Critical fail-closed nuance from §1(b):** during a *local* merge the last-touch SHA of any
path the merge itself touched is computed against an UNCOMMITTED tree, so it can legitimately
differ from the post-merge-commit value CI will compute. The driver MUST therefore treat the
local result as *provisional* and the **authoritative regeneration is the post-merge/post-commit
settle** (the `post-rewrite`/`post-merge` hook runs after the commit exists). If the post-commit
settle produces a different face than the in-merge settle, that is EXPECTED, not a bug — the
post-commit value wins and is what gets committed. This is the single most important correctness
subtlety and the reason v1 cannot be "just a %O %A %B driver" like cargo-lock.

### 3.5 Universality (Q5)

- Faces-glob set, producer target, and per-face parameters come from
  `registry/generated-artifact-control-plane.json` (already DATA, already a declared public
  product contract). The driver hardcodes nothing.
- The driver binary is a neutral engine: "given a control-plane manifest + a producer + an
  scm-facts boundary, regenerate the declared generated artifacts on merge." Any repo adopting
  oya-ci ships its own control-plane manifest + its own producer target and gets the same
  behavior.
- `.gitattributes` lines are generated FROM the control-plane manifest (a tiny emitter), so the
  glob never drifts from the policy. (Flag for §8: this means `.gitattributes` becomes a
  generated face itself — see the gate-conflict note.)

---

## 4. v2 — server-side merge-queue regenerate (the real fix)

Local drivers don't run on github.com. The durable fix lives in `oya/ci-tide`:

- Extend `oya-ci-tide-kernel::is_mergeable` and the worker so that, for a PR at the front of the
  queue, tide: (1) rebases the candidate onto the projected tip in its own workspace, (2) runs
  `oya-repo-hooks-bootstrap` + the faces settle (post-commit, so SHAs are final), (3) re-runs
  freshness + registry-drift on that rebased+settled tree, (4) admits only if green.
- This is exactly what the existing `merge_group` CI event already does for materialization
  (`oya-ci-required.yml` re-materializes faces in the merge_group lane). v2 makes tide *commit
  the settled faces into the rebased candidate* rather than just verifying — so two PRs touching
  the big face merge sequentially **without human rebasing**, which is the actual scalability
  win for the server path.
- Today `is_mergeable` requires "forge reports mergeable (no conflicts)" — which means it
  currently *blocks* faces-conflicting PRs. v2 replaces that hard block (for faces-only
  conflicts) with rebase-and-regenerate. Non-face conflicts still block (correct).
- ADR-0515 owns oya-ci/tide; ADR-0111 owns projected merge state. v2 is an amendment to those,
  not a new substrate. Recommend a short ADR ("merge-queue regenerates born-accounting faces on
  rebase").

This is explicitly OUT OF SCOPE for v1 delivery but IN SCOPE for the design's honesty: **v1
alone does not make server merges parallel.**

---

## 5. Deeper alternative — should the 18k-row registry be committed at all? (Q6)

Three options, each must still satisfy ADR-0555-style "unaccounted artifacts unmergeable by
design" (the structural-accounting invariant: an artifact that isn't accounted-for cannot
merge). Note: no `ADR-0555.md` exists in `docs/decisions/` (the invariant is currently carried
by `registry/generated-artifact-control-plane.json` + the total-accounting/firewall gates +
ADR-0539 freshness). Flag for §8.

| Option | How it works | Pros | Cons | Invariant preserved? |
|---|---|---|---|---|
| **(a) Merge-driver on committed faces** (v1+v2) | Keep faces committed; auto-regenerate on merge | Smallest change to the accounting model; the face stays a reviewable artifact; gates unchanged | Driver is subtle (§3.4); v1 doesn't fix server merges; 12 MB files still churn git history hugely | YES (faces stay committed + gated) |
| **(b) Don't commit the big registry; commit a digest/baseline; CI computes the full face** | Commit only a content digest (e.g. a stable hash of the canonical registry) + the accepted-violation baseline; CI regenerates the full 12 MB face ephemerally and compares its digest to the committed one | **Conflict class largely vanishes** — a digest is a few bytes; two disjoint PRs rarely collide on it; git history stops carrying 12 MB churn; freshness becomes "digest matches" | The full face is no longer a reviewable in-repo artifact (must be a CI artifact); the digest must itself be deterministic across runners (achievable — output is already canonical); a stale digest still conflicts (but trivially) | YES *if* the digest is the accounted-for artifact and CI fails-closed when recompute ≠ digest |
| **(c) Shard the registry by capability** | Split `accounting-registry.generated.json` into N per-capability faces (`accounting-registry.<capability>.generated.json`) keyed by top-level dir (compute/iam/k8s/...) | Two PRs touching disjoint capabilities touch disjoint face files → **no conflict at all** for the common case; each shard is small + reviewable; composes with (a) and (b) | N files instead of 1; a cross-capability change (rename moving a path between capabilities) touches two shards; sharding key must be policy-as-data | YES (each shard committed + gated; union still accounts for every path) |

**Recommendation: (c) + (a), with (b) as the strategic end-state.**

- **Do (c) first** — it is the highest-leverage, lowest-risk change. It directly attacks layer
  (a) of the root cause (whole-tree enumeration in one file). After sharding, the wave-1 PRs
  that touched disjoint capabilities would NOT have conflicted at all. Sharding key = top-level
  capability dir, declared in the control-plane manifest. The 18,485 rows already carry `path`,
  so the shard assignment is a pure partition of existing data.
- **Keep (a) v1** for the residual case (PRs that DO touch the same shard, plus the
  history-fact churn) and to automate the agent flow now.
- **Treat (b) as the W-horizon end-state.** It is the cleanest ("the big generated blob is a CI
  artifact, only its digest is committed"), and it aligns with the founder doctrine that
  generated outputs should be ignored-by-default / regenerated-from-source (the control-plane
  manifest already says `merge_policy: never-manual-merge-regenerate-from-source-tree`). But it
  is a deeper change to the freshness/registry-drift gates (they currently byte-compare the
  committed file) and to reviewer expectations, so it should be a separate ADR + its own slice.

**Why not (b) immediately:** the freshness gate's `check_repo_with_regenerated_faces` and the
registry-drift test both assert *byte-equality of the committed file*. Switching to digest-only
is a gate-contract change touching ADR-0539 and the registry-drift gate; doing it under time
pressure risks a false-green window. Sequence it.

---

## 6. Born-accounting + new crates/files (Q7)

**New crates (each born-accounted via `register_crate` / #105):**

1. `tools/oya-faces-merge-driver-app` (lib + bin)
   - `src/lib.rs`: `materialize_all_faces(repo_root, scm_facts_path) -> Result<Vec<(String,String)>>`
     (the ONE shared materializer; the existing producer `main.rs` should be refactored to call
     it too — flag as follow-up, not required for v1), `run_merge_driver(base,ours,theirs,path)`,
     `run_post_merge_settle(repo_root)`, `DeterminismCheck`.
   - `src/main.rs`: arg parse `%O %A %B %P`; subcommands `driver` (per-file) and `settle`
     (post-merge). Exit non-zero on any failure (§3.4).
   - `BUCK`, `Cargo.toml`, `README.md` (activation via bootstrap, NOT manual git config).
   - `tests/` — see §7.

2. `tools/oya-repo-hooks-bootstrap-app` (lib + bin)
   - `src/lib.rs`: `bootstrap(repo_root, control_plane) -> Result<BootstrapReport>` (idempotent
     `git config --local` writes + `core.hooksPath` + `.githooks` install), reads driver/glob set
     from `registry/generated-artifact-control-plane.json`.
   - `src/main.rs`, `BUCK`, `Cargo.toml`, `README.md`.

3. (optional, small) `tools/oya-gitattributes-emitter-app` — emits the `merge=oya-faces` lines
   from the control-plane manifest so `.gitattributes` cannot drift from policy. Could instead
   live as a subcommand of the bootstrap binary to avoid crate proliferation (memory:
   ADR-0132 no-grouping favors single-concern — but a subcommand of bootstrap is defensible as
   "hook/driver activation" is one concern). **Recommend: subcommand of bootstrap**, not a new
   crate, to respect ADR-0132.

**Modified (no new crate):**
- `.gitattributes` — add the `merge=oya-faces` glob lines (generated, see above).
- `registry/generated-artifact-control-plane.json` — add `merge_driver` + `shard_key` policy
  fields (DATA only).
- `.github/workflows/oya-ci-required.yml` — one bootstrap step before gate lanes.
- `oya/ci-tide/...` (v2 only) — rebase-and-regenerate in the worker.
- `docs/decisions/` — new ADR for merge-queue face regeneration (v2) and (if doing §6c) an ADR
  for registry sharding.

**Born-accounting checklist per new crate (register_crate / #105):** BUCK target + reindeer
wiring; OWNERS entry; the crate's own row lands in the accounting registry; freshness +
target-parity + manifest-hygiene + cargo-prefix gates green for the new crate; SLO coverage if
it promotes past dev. (Memory: `register-crate-scaffold`, `buck2-build-green-not-ci-green` —
regen lock + faces + run freshness/affected-set gates before claiming done.)

---

## 7. Test plan (Q7)

Acceptance oracle: **simulate a faces conflict → driver/settle resolves → freshness +
registry-drift green.** Build on the existing precedent
`oya-cloud-ci-freshness-app/tests/face_settle.rs` (it already constructs a throwaway git repo
with faces, commits, mutates, and asserts settle behavior).

1. **Conflict-resolution integration test** (`tools/oya-faces-merge-driver-app/tests/merge_resolve.rs`):
   - init a temp git repo with a minimal faces fixture + scm-facts emitter stub (or the real
     emitter against the temp repo's own history);
   - create branch A (adds file `a.txt`) and branch B (adds file `b.txt`) — both regenerate the
     big face → both touch overlapping regions;
   - run the bootstrap, then `git merge` / `git rebase` B onto A;
   - ASSERT: no conflict markers remain; the merged faces == a from-scratch materialize of the
     merged tree (byte-equal); exit 0.
2. **Fail-closed test:** inject a producer error (malformed `oya-ci.toml` / missing scm-facts);
   ASSERT driver exits non-zero AND leaves the conflict (no partial face written).
3. **Determinism test:** materialize twice on the same tree; ASSERT byte-equal; then corrupt the
   determinism (e.g. feed a non-canonical input) and ASSERT the self-check trips and exits
   non-zero.
4. **Verifier-parity test (the real acceptance):** after driver settle, run the freshness gate
   library (`check_repo_with_regenerated_faces`) and the registry-drift byte-compare against the
   settled tree; ASSERT both green. This proves the driver's output satisfies the existing gates
   — the only definition of "done" that matters.
5. **Bootstrap idempotency test** (`tools/oya-repo-hooks-bootstrap-app/tests/idempotent.rs`):
   run bootstrap twice on a temp repo; ASSERT `git config merge.oya-faces.driver` set, hooksPath
   set, second run is a no-op, and the config matches the control-plane manifest.
6. **Universality test:** point the bootstrap + driver at a *synthetic* control-plane manifest
   with a different glob/producer; ASSERT the driver regenerates the synthetic artifact (proves
   neutral-engine + policy-as-data, no hardcoded oyatie paths).
7. **(v2) Merge-queue test:** in `oya-ci-tide` kernel tests, assert a faces-only conflicting PR
   becomes mergeable after rebase-and-regenerate, while a non-face conflict still blocks.
8. **Born-accounting tests:** the new crates appear in the accounting registry; gate_registration
   meta-test still green.

All tests are buck2 targets (memory: `buck2-primary-build`); run under
`cargo test --locked -p ... -- --test-threads=1` for the git-touching ones (match the existing
freshness test convention).

---

## 8. Seven-property self-audit (`pipeline-four-property-bar`)

| Property | v1 driver + bootstrap | Notes |
|---|---|---|
| **Universal** (neutral engine + policy-as-data, any repo) | PASS | Glob/producer/shard-key all read from `generated-artifact-control-plane.json`; §3.5 + test #6 prove it. |
| **Productized** (engine + packs + control-plane + public contract) | PASS | Control-plane manifest already has `public_product_contract`; driver/bootstrap are the engine; the manifest is the pack. |
| **Hermetic** (pure Rust, no shell/net/clock/rand, deterministic, buck2) | PARTIAL → ACCEPT | The ONE sanctioned subprocess is the scm-facts emitter (ADR-0515 D3 git boundary) — ledgered as irreducible glue (§3.2). Everything else is in-process pure libraries. `.githooks` shim is 2 lines calling the Rust binary — ledgered. |
| **Automated** (ships its own auto-fix, not flag-only) | PASS | The driver IS the auto-fix (regenerate instead of conflict). Bootstrap auto-activates with zero manual step (§3.3). This is the whole point — no README `git config` line. |
| **Cloud-native / API-driven** (CRD/operator/typed-API, not CLI) | PARTIAL | A git merge driver is inherently a local-git-invoked binary, not an API — but it is *transitional local glue* per the all-CLI-retirement doctrine; the **cloud-native answer is v2 (merge queue / tide operator)**, which IS API/controller-driven. v1 is the bridge; v2 is the destination. Flag honestly. |
| **Modern / right-tool** | PASS | Reuses git's native merge-driver mechanism (the right tool for local merge auto-resolution); reuses the existing producer libraries; no reinvention. |
| **Latest-info** | PASS | Design grounded in the current tree (faces are SHA-bearing, ci-tide gates on no-conflict, merge_group already wired) — all verified this session, not assumed. |

**Net:** v1 honestly sits at "transitional local glue that is universal+productized+automated+
fail-closed"; the cloud-native/API-pure box is checked by v2. The design is explicit that v1 is
the bridge and v2 is the doctrinally-pure destination.

---

## 9. Where the design fights an existing gate / doctrine (flagged)

1. **`.gitattributes` would become a generated artifact.** If §3.5's gitattributes-emitter is
   adopted, `.gitattributes` is now controller-generated — but it is NOT under the faces glob and
   is itself a merge surface. **Conflict with the no-manual-merge doctrine if two PRs both change
   the glob set.** Mitigation: keep the glob set tiny + stable, regenerate it only when the
   control-plane manifest's artifact list changes (rare), and add it to the control-plane
   manifest's own artifact list so it is accounted-for. Decide explicitly before building.

2. **ADR-0555 does not exist as a file.** The prompt cites ADR-0555 (structural accounting) but
   `docs/decisions/` has no `ADR-0555*.md` (greps for 0551/0554/0555 also returned nothing; only
   0539 freshness exists). The "unaccounted artifacts unmergeable" invariant is *currently*
   carried by `generated-artifact-control-plane.json` + total-accounting/firewall gates +
   ADR-0539. **Action: confirm the correct ADR number before citing it in code/ADR front-matter,
   or author the missing ADR.** (Memory `remaining-work-ssot-hierarchy`: snapshots drift —
   verify the live ADR set.)

3. **Freshness gate hardcodes the 7 face paths** (`GENERATED_FACE_PATHS` in
   `oya-cloud-ci-freshness-app/src/lib.rs`) and the producer hardcodes `PRODUCER_FACES`. If §6c
   (sharding) is adopted, the face set becomes dynamic (N capability shards), so these hardcoded
   arrays must move to policy-as-data (read from the control-plane manifest). **The sharding
   option fights the current hardcoded face list** — sequence the gate refactor with the shard.

4. **`assert_non_face_tree_clean` requires the non-face tree be committed before settling.** The
   merge driver runs *mid-merge* when the tree is by definition not "clean." The driver's settle
   must run *post-merge-commit* (the `post-rewrite`/`post-merge` hook), not mid-merge, to satisfy
   this invariant — this is consistent with §3.4's "authoritative regen is post-commit" but it
   means the per-file `%A` write during the merge is purely cosmetic (avoid-the-conflict-marker)
   and the real work is in the hook. **Do not try to make the per-file driver authoritative; it
   fights `assert_non_face_tree_clean`.**

5. **Bootstrap via buck2 genrule side-effect** (§3.3) couples a git-config side-effect into the
   build graph. This is mildly unclean (builds should be pure). Acceptable because it is
   idempotent and the alternative (manual step) violates the AUTOMATED bar — but flag it for
   review; an alternative is a dedicated `buck2 run //tools/oya-repo-hooks-bootstrap-app` that
   the dev-onboarding doc + CI both call explicitly (less magic, one documented command).

---

## 10. What this design explicitly does NOT solve

- **GitHub server-side merge button / squash merge.** Local merge drivers + local hooks do not
  run there. Two PRs touching the same face shard will still show "conflicts" on github.com
  until **v2 (merge-queue rebase-and-regenerate in `oya/ci-tide`)** ships. v1 makes the *local
  agent flow* one-command and fail-closed; it does not make the *server path* parallel.
- **The git-history churn of multi-MB committed faces.** Even auto-resolved, committing 12 MB
  faces on every PR bloats history. Only §6(b) (digest-only, CI-computed) actually fixes that;
  §6(c) (sharding) reduces per-PR churn but doesn't eliminate it.
- **Cross-capability moves under sharding** still touch two shard files (rare, acceptable).

**Recommended delivery order:** (1) §6c shard the registry [biggest leverage, attacks the root
cause directly] → (2) v1 driver + bootstrap [automates the residual local flow, fail-closed] →
(3) v2 merge-queue regenerate [closes the server path] → (4) §6b digest-only [history-churn
end-state, separate ADR].
