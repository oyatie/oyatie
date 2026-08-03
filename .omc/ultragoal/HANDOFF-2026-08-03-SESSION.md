# SESSION HANDOFF — 2026-08-03

State at write time. `origin/dev` = `d11567a1a`. **Nothing was pushed. No PR opened. No cluster touched.**
The canonical checkout remains on `preserve/hermes-w1-dirty-20260630` with its 1,384 tracked modifications untouched.

---

## 0. IN FLIGHT — check this first

> **OUTCOME: KILLED MID-FLIGHT.** All four agents started 04:45 and ran to 04:52; the journal holds **four `started` records and zero results**. Two produced commits anyway, both now tagged. **NONE OF IT IS VERIFIED** — no verify stage ran, no gate output exists, no agent returned evidence. Treat every commit below as UNPROVEN, not as ready-to-land.
>
> | lane | survived | state |
> |---|---|---|
> | `w0c` | **`6c93682ce`**, tag `rescued/w0c-restack-20260803` | W0-C restacked onto dev, clean tree, 1 commit. **Unverified: nobody confirmed the two generated projections were resolved by re-running the producer rather than hand-edited.** That is the first thing to check. |
> | `reorg` | **`4f94836b9`**, tag `rescued/reorg-park-20260803` | park commit only (commit 1 of 2). The `first_move_plan` / `committed_move_plan` **deletion fix was never written**. |
> | `census` | nothing | branch created, zero commits. |
> | `supply` | nothing | branch created, 1 dirty path, zero commits. |
>
> Canonical checkout verified intact after the kill: HEAD `c52bdb09e`, exactly **1384** ` M`.

**Workflow `wqjogv5ww`** — run `wf_d6bcf588-fc5`. Launched 2026-08-03 with a usage limit imminent; it was cut off mid-flight.

- **Read the journal BEFORE assuming anything ran:** `.../subagents/workflows/wf_d6bcf588-fc5/journal.jsonl` under `~/.claude/projects/-Users-jasonlee-Developer-oyatie/6ae0b4c0-a1c7-4641-af10-a945adc5ec8b/`. It records each agent's actual return value.
- **Script:** `~/.claude/projects/.../workflows/scripts/resume-parallel-2026-08-03-wf_d6bcf588-fc5.js`. Resume with `Workflow({scriptPath: <that>, resumeFromRunId: "wf_d6bcf588-fc5"})` — unchanged agent calls return cached, only new/edited ones re-run.
- **Four disjoint build lanes**, each in its own scratchpad worktree off `origin/dev`, each then adversarially verified by `codex exec` (cross-model, GPT family — deliberate perspective diversity, not another Claude agreeing):

| lane | worktree | branch | what it does |
|---|---|---|---|
| `w0c` | `wt-w0c` | `restack/w0c-graph-v2` | cherry-pick #1524's `3f5c4b8a0` onto dev, drop the redundant W0-B, resolve the two generated projections **by producer** |
| `supply` | `wt-supply` | `fix/supply-chain-multi-lockfile` | derive the lockfile set instead of the single `"Cargo.lock"` string; four-run RED proof |
| `census` | `wt-census` | `fix/census-gate-epoch-selfcheck` | the 2 approved hunks + mutation proof |
| `reorg` | `wt-reorg` | `fix/reorg-plan-selection-and-park` | park `intelligence-remainder` by suffix; **delete** `first_move_plan` / `committed_move_plan` |

- **Precedent for distrust:** the prior workflow `wrza5sv1w` (`wf_6468123a-da5`, same supply-chain scope) produced a **0-byte output and no journal directory** — it died emitting nothing. Its RED proof was never produced. Treat "a workflow was dispatched" as no evidence at all until the journal is read.
- Every lane was given the standing constraints verbatim, including the 1384-` M` canonical-checkout assertion, materialize-before-gate, no push / no PR / no cluster, and never hand-edit producer output.

---

## 1. STANDING CONSTRAINTS (unchanged, still binding)

- Never mutate the canonical checkout's working tree or `preserve/hermes-w1-dirty-20260630`.
- No push, no PR, no merge without explicit approval. No cluster mutation ever without separate authority — the lab cluster is SHARED with the console project.
- Never hand-edit `*.generated.json` / generated faces; run the generator.
- Buck2 is the evidence authority; cargo is local feedback only.
- **Materialize generated faces on the candidate tree BEFORE running any gate locally.** A gate run without it can return a false green — measured: `ci-baseline-ratchet-gate` "11 passed" → RED with 3 regressions on the identical commit after materializing. See [[local-gate-verification-order]].
- The canonical working tree is STALE — read `origin/dev` via `git show`, never the worktree.

---

## 2. READY TO LAND (verified, unpushed, awaiting founder go)

| item | branch / location | evidence |
|---|---|---|
| **census gate fix** | 2 hunks, not yet applied | 4-iteration ralplan consensus **APPROVE**. Insert `validate_dormant_p3_epoch_policy_for_event(&validated)` after line 1570 / before 1572 in `ci/facade/scm-facts-snapshot/tests/snapshot_integration.rs` — **only** at the P2-active host `:1553`; the other three hosts are P3-active where stage-3 Ok *entails* stage-4 Ok. Plus delete dead `pub fn validate_dormant_p3_epoch_policy` (`src/lib.rs:1309-1318`, zero callers). **No fallback host exists.** The plan doc at `.omc/plans/ci-gate-matrix-contention-durable-fix.md` should be DELETED, not revised — its §6 fallback and AC#2 route an executor to an inert test. |
| **init-app 66 tests** | `worktree-wf_b064d62b-601-1` @ `ed575887e` | 65 in `main.rs` + 1 integration, all pass, no `#[ignore]`, real per-test names. Filter RED-proved independently. **Sequencing risk:** touching `os/core/init-app/BUCK` forces FULL tier → runs `//...` → runs the flaky `platform_config.rs` sibling. Pair with a flake fix. |
| **C2 / G1 / G3** | `land/c2`, `land/g1`, `land/g3`, rebased on `d11567a1a` | All three independently RED-proved by the adversarial filter. G1: 11 passed, mutation-proved. G3: manifest 8966 bytes, 78 crate_idents, correctly untracked. C2: 443 total / 36 superseded reproduced. |
| **E-shard** | `08f3bbdc2` | **DO NOT LAND.** Verified but unjustified — the contention it targeted was already gone; it weakens a fleet-wide review surface. |

---

## 2b. OPEN PR — #1524, the only one, and it is now UNBLOCKED

**`#1524 draft(reorg): preserve W0-C graph and W0-D reset work`** — branch `draft/reorg-w0c-w0d-preservation-20260802`, head `b1c4664d0`, OPEN / draft / `CONFLICTING` `DIRTY`, untouched since 2026-08-02T10:07Z. **This is the ONLY open PR in the repo.** An earlier revision of this handoff omitted it entirely.

- **Its own stated resume condition is now satisfied.** The body says: admit #1522, then rebase/re-run #1523, *then* restack W0-C and W0-D as serial protected PRs. **#1522 merged 2026-08-02T10:24Z; #1523 merged 2026-08-03T00:57Z.** It was written while both were pending. Nobody came back.
- **It is the sole surviving copy of that work.** `ADR-0635`, `ci/facade/reset-eligibility-policy/**`, `specs/reset-eligibility.schema.json`, `specs/substrate-dependency-dag.schema.json`, `tests/fixtures/graph-v2-cases.json` — **all ABSENT from `origin/dev`** (verified by `git ls-tree -r origin/dev`). ~6,700 lines. It survives only because the remote branch still exists; do not delete it.
- **The `CONFLICTING` status overstates the cost.** `git merge-tree origin/dev <head>` conflicts in **exactly two files — `docs/ADR-INDEX.md` and `docs/machine-readable/decisions.json`** — both **generated projections**. Every byte of real content merges clean. Per [[adr-index-is-generated-use-the-generator]] the resolution is **re-run the producer**, never hand-merge the hunks. `ADR-0635` is still free (dev's highest is `ADR-0634`), so no renumbering.
- **Restack shape:** drop commit `b04328f84` (W0-B — already on dev via #1522, and the sole cause of the dirty state), then `3f5c4b8a0` (W0-C) and `b1c4664d0` (W0-D) as two serial PRs.
- **W0-D's `affected-set-policy.json` edit is the correct shape** — three `synthetic_dependencies` entries mapping the new policy/schema/evidence files to the new gate target, not an `inert_selection_classes` grant. It merges clean against #1527. Still owes the [[affected-set-inert-declaration-bar]] check on the candidate tree.
- **Instance ten of the structure-vs-state defect, in its own PR body:** "the cumulative diff contains no `*.generated.json` changes" is literally true and materially false — it changes two producer-owned projections that simply lack that suffix. The check read the **filename pattern** when the deciding fact was **who owns the artifact**.

---

## 3. RESCUED — irreplaceable, do not lose

**17 uncommitted move plans** extracted to `.omc/ultragoal/rescued-move-plans/` with a provenance manifest (`README.md`). These were staged/intent-to-add in agent worktree indexes and committed nowhere; a `--force` sweep would have destroyed them permanently. The codemod does **not** author plans — a human decides each disposition.

Most consequential: **`ci-tide-move-plan--2962ca57ed.json` is A1**, the paved-road proof, specifying
`oya/ci-tide/crates/oya-ci-tide-{kernel,github-adapter,app}` → `ci/{ports/tide-kernel, adapters/tide-github-adapter, facade/tide-app}`.
Note it targets `ci/ports/`, **not** the `ci/core/` the spine assumed.

Also: 3 commits tagged `rescued/wf_a6e35b25-9c4-5`, `-6`, `rescued/wf_8bfcdc98-b6f-5` (detached HEADs, 4/3/2 ahead).

---

## 4. REORG — the unblock is known

- **Ten move plans committed; nine LANDED; one ACTIVE-UNSTARTED**: `intelligence-remainder-move-plan.json`, 156 of 156 probe paths still present, zero moved. Commit `d11567a1a` (#1528) was **plan-only — it took the repo-wide mutex without moving a byte**.
- **Park it** by renaming outside the discovery glob (`.PARKED.json`); precedent `e6230244f`/#1521 parked `kernel-move-plan.BLOCKED.json`, suffix exclusion proven mechanically.
- **A1 is blocked TODAY, not prospectively** — worktree `wf_18a9e839-868-4` also holds `ci-tide-move-plan.json` staged, so two ACTIVE plans → `MultipleMovePlans` fires from step 1.
- **Live latent defect (deletion-shaped fix):** `ci/facade/crate-registration/src/lib.rs:702` (`first_move_plan`) and `inventory-registry-drift/tests/registry_drift.rs:207` take the FIRST SORTED plan, not the ACTIVE one, and explicit `--plan` takes precedence over the codemod's correct selection. A1 would have derived its manifest from the inert `ci-move-plan.json`. Fix = drop both functions and stop passing `--plan`.
- A1 also needs 3 catalog rows authored + 3 stale `uncatalogued` baseline lines removed in the same PR.

---

## 5. MEASURED FACTS — do not re-derive, do not contradict without new evidence

- **`os/` is 3 of 41 crates real.** 25 have zero reverse dependencies. mTLS gRPC machine API **unimplemented** (`apid-domain`: zero third-party deps, no tonic/prost/rustls, zero consumers). **Zero image rules in the entire 912-package buck2 graph** — this single absence blocks the wire oracle and makes "ships in production" unprovable everywhere.
- **`k8s/` delivers zero working behavior against real Kubernetes.** Not an owned Kubernetes — a cluster-lifecycle manager and a *client* of CAPI, so CNCF conformance is the wrong oracle. All 3 binaries fake-backed; zero reverse deps repo-wide.
- **Neither port has an oracle.** Zero `.go` files in tree; the 59 "differential vectors" are hand-committed constants whose cited generator does not exist. `k8s/`'s acceptance suite omits `-adapter-capi` — it asserts the fake behaves as written.
- **Asterinas has no aarch64 backend on `main`** (`ostd/src/arch` = x86, riscv, loongarch). Maintainer branch `arm-v4` active 2026-07-30, 31 ahead, blocked on open PRs **#3511** and **#3480**. OSDK does **not** port architectures. Asterinas also has **zero required status checks** and merges over red, and its harnesses pass on zero tests in 4 of 5 tiers.
- **Standing lanes are THREE plus one task**, not four — Lane 3 has 0 of 24 capabilities admitted. See [[standing-lanes-three-not-four]].
- **Supply-chain gate is blind to 66 packages** in two nested workspaces (`lockfile_path` is a single string; 3 workspaces exist). **Zero are actually affected** — 34 name-matches all clear on version range. Coverage defect, **not** an incident. Do not report it as one.
- **12 crates / 133 never-compiled tests** repo-wide from the same generator blindness that hid init-app's 65.
- **agent-skills' hook injector is inert** — emits `{priority, message}`, neither field Claude Code reads; its 10.4 KB payload never reaches context.

---

## 6. BLOCKED ON THE FOUNDER

1. **D1 backup** — designed, unwired by design, needs 7 named approvals; cluster is shared.
2. **G028 live 22Gi** — needs content digests, named bootstrap principal + scoped RBAC, audit sink, rollback owner, Secret restoration authority, and disposition of 3 `HELM_LIVE_NOT_RENDERED` kube-mode objects.
3. **~40 `nethelpers` vectors** — delete them (default), or re-admit a Go toolchain for a digest-pinned oracle.
4. **Lane 3 seed** — approve `tenancy` as the single seed (drain `cloud/tenancy`, 211 files, 0 crate manifests) or reject.

---

## 7. THE ONE LESSON WORTH CARRYING

**Nine times** across this session — by the Planner, the Architect, the Critic, and me — a property was asserted from a **symbol name, a range, a helper's title, or a call-graph shape** when the deciding fact was **state**: which epoch the fixture wrote, whether a type was `pub`, whether a blob existed in the index, whether a version satisfied a range. Every instance was invisible from inside the work that produced it, and each was caught only by someone opening the actual file.

The consensus loop converged a four-PR program with a new gate, a new face, an ADR, and a born-blocking check into **two hunks with a net-negative line count** — entirely by that mechanism. Keep authoring and review in separate passes; never self-approve.
