# Monorepo Consolidation Migration — REVISED PLAN (ralplan deliberate, post-Architect+Critic)

> Planning only. NO execution. Forge = GitHub `jason931225/oyatie`, base branch `dev`, plain git + `gh`, remote `github-mirror`.
> Revision authority: addresses every Critic finding (CRITICAL authority-flip, fabricated generator citation, k8s/cloud-k8s surface, source inventory, squash-merge) and incorporates the Architect synthesis (pre-lane 0.4, build-to-BOTH gates, tools/ as standing invariant exception, no_std exclude-state inertness). Locked decisions preserved unchanged.

## Revision Ledger (what changed vs prior plan)

| # | Source | Finding | Resolution |
|---|--------|---------|------------|
| R1 | Critic CRITICAL / Architect antithesis | Repo is mid-migration from `github-lane-unlocker-required` → `oya-ci-required` + mandatory signing (`branch-protection.yaml@dev` 2026-06-04, ADR-0513). Plan pinned PRIMARY gate + every Done-Definition to a gate the repo's own ADR calls a non-authoritative shadow adapter; signing unprovisioned. | NEW gating pre-lane **0.4 AUTHORITY-SNAPSHOT + SIGNING PRE-PROVISION**; per-iteration **loop step 0** re-diffs live protection; new **gate G0** (HALT-for-USER on flip); signing **promoted from optional to a HARD precondition**; principle reframed to **BUILD-TO-BOTH** (pass live gate AND pre-characterize `oya-ci-required`). |
| R2 | Critic MAJOR | `gen_first_party_buck.py:129 = cargo metadata --no-deps` does NOT exist on dev (fabricated citation). | Re-derived: REAL generator = `scripts/ci/generate-first-party-buck.rs` (+ `scripts/tests/generate_first_party_buck_check.rs`), "Cargo metadata treated as input format only" (members-based). Lane 0.6 inertness proof is now **EMPIRICAL** (run the coverage-check with excluded tree on disk), not argued from a citation. |
| R3 | Critic MAJOR | k8s merge surface under-modeled: `cloud/cloud-k8s/` (sixth surface) un-mentioned; per-service `crates/` not correspondence-mapped; pilot k8s entangled. | Lane 0.5 produces a **crate-level merge-surface manifest**; `cloud/cloud-k8s` added to open questions + gated on USER confirmation. |
| R4 | Critic MAJOR | No physical source-tree inventory / allowlist; framekernel/node-os paths unnamed; db-engine source not found. | Lane 0.5 **SOURCE-INVENTORY** deliverable (per-lane source path + first-party allowlist + per-tree deny-globs). Paths resolved (below). **db-engine source NOT FOUND → hard USER blocker; lane DROPPED if absent.** |
| R5 | Critic MINOR | Merge-method mismatch: live repo allows **squash only** (`merge_commit=false, rebase_merge=false, squash=true`). | Loop step + Done-Definition reworded to **squash-merge** (still linear). |
| R6 | Architect synthesis | tools/ retirement entangled with 723 root members + one-version invariant; not a deferrable cleanup. | tools/ written as a **STANDING exception in the canonical-homes invariant** (gate-load-bearing `//tools/oya-doc-staleness-inventory-app` + `//tools/oya-adr-index-regenerator-app` kept on disk); any disk removal removes the root member in the SAME atomic step. |
| R7 | Architect synthesis | no_std exclusion mechanism is itself a graph mutation (root Cargo.toml has NO `exclude` key today). | Lane 0.6 proves inertness of the **EXCLUDED STATE INCLUDING the exclude-key edit**, not merely subtree absence. |
| R8 | Architect / observed | ADR regen via phantom `oya doc adr-index`. | Corrected to `tools/oya-adr-index-regenerator-app` + `.omc/automation/adr-index-pipeline.md`; index = `docs/ADR-INDEX.md`, ADRs in `docs/decisions/ADR-NNNN-*.md`. |
| R9 | Observed brand residue | office=`oyaoffice-*`, codex=`openai-codex-sdk`, claude=`claude-agent-sdk`, node-os=`talos-*`, oyago/oyapy codenames. | Each lane carries a **codename→`oya-*` rename pass** as its first in-lane mutation; canonical names ratified in Lane 0.5. |

---

## 0. Verified Ground Truth (re-checked against `github-mirror/dev` + live `gh api`)

- **Live merge gate (today):** `gh api repos/jason931225/oyatie/branches/dev/protection` → required contexts = `[github-lane-unlocker-required]` (app_id 15368), `required_linear_history=true`, `required_conversation_resolution=true`, `required_signatures.enabled=false`.
- **Documented TARGET gate:** `.github/branch-protection.yaml@dev` (2026-06-04, ADR-0513) → required context `oya-ci-required`, `require_signed_commits: true`, `require_signed_tags: true`; `github-lane-unlocker-required` = "shadow compatibility only." **THESE DISAGREE — the authority flip is in flight.**
- **Gate workflow** `.github/workflows/github-lane-unlocker-ci-cd.yml@dev`: 4-lane matrix building ONLY `buck2 build //:...-check` targets + `//tools/oya-doc-staleness-inventory-app:doc-staleness-inventory-unit-tests` and `:...-json`; never runs cargo deny/clippy/nextest.
- **Real first-party Buck generator:** `scripts/ci/generate-first-party-buck.rs` (members-based; "Cargo metadata as input format only"); checked by `scripts/tests/generate_first_party_buck_check.rs`. `reindeer.toml` buckifies third-party from `Cargo.lock`. `infra/ci/buck2-affected-gate.sh` present.
- **Root workspace:** `Cargo.toml` `[workspace]` line 1, **723 members**, `resolver="2"` line 727, **NO `exclude` key**.
- **Repo merge settings:** `allow_merge_commit=false`, `allow_rebase_merge=false`, `allow_squash_merge=true`, `delete_branch_on_merge=true`.
- **Signing:** local `commit.gpgsign / user.signingkey / gpg.format / tag.gpgsign` all EMPTY (exit 1). **No key provisioned.**
- **Physical sources** (resolved): framekernel=`linux/stack/kernel` (no_std, **nightly-2026-02-28**, build-std targets `aarch64-unknown-none-softfloat`, `x86_64-unknown-none`); node-os=`linux/stack/operating-system` (51 `talos-*` STD crates, **channel 1.96.0**; distinct from EXCLUDED `linux/stack/talos-reference`); k8s+containerd=`linux/stack/kubernetes/crates` (**139 crates = 44 first-party `ctrd_*` + 95 k8s**, entangled with `_upstream`, `_upstream_containerd`, `third-party`, `__pycache__`, `target`, `buck-out`, `prelude`, `toolchains`, `.omc`, `.omx`); claude=`~/Developer/claude` (`claude-agent-sdk`); codex=`~/Developer/codex/sdk/rust` (`openai-codex-sdk`); oyago=`~/Developer/oyago/crates`; oyapy=`~/Developer/oyapy/crates`; office=`~/Developer/office/crates` (13 `oyaoffice-*`). **db-engine: NOT FOUND.**
- **Existing merge targets on dev:** `cloud/cloud-intelligence/crates/oya-cloud-intelligence-codex-adapter` (MERGE), 4× `cloud/managed-k8s-*/crates` (MERGE), **`cloud/cloud-k8s/` populated** (un-modeled surface). Verified-absent CREATE homes: `cloud/cloud-kernel`, `cloud/cloud-node-os`, `cloud/cloud-container-runtime`, `oya/transpiler-go-to-rust`, `oya/transpiler-python-to-rust`, `oya/office`.
- **ADR state:** max committed ≈ ADR-0370 (`docs/decisions/`), in-flight branches reach adr-0505; regen via `tools/oya-adr-index-regenerator-app` → `docs/ADR-INDEX.md`.

---

## 1. Principles

1. **BUILD-TO-BOTH-GATES (supersedes BUCK2-GRAPH-IS-THE-GATE):** every lane must (a) turn the LIVE blocking check green — today `github-lane-unlocker-required`, a Buck2-only whole-graph check (per-crate BUCK + `reindeer buckify` + `//:buck2-cargo-target-coverage-check`, `//:buck2-authority-policy-check`, `//:repo-hygiene-automation-check`, `//:third-party-durable-handedits-check`, `//:latest-toolchain-pin-updater-check`, `//:oya-ci-prowjob-registry-check`, `//:github-lane-unlocker-bridge-check`, `//:rust-llvm-coverage-runner-contract-check`, `//:rust-llvm-coverage-smoke-check`, `infra/ci/buck2-affected-gate.sh origin/dev HEAD`, `//tools/oya-doc-staleness-inventory-app:...`) AND (b) be ready for the DOCUMENTED-AUTHORITATIVE gate (`oya-ci-required` + signed commits/tags) by pre-provisioning signing and pre-characterizing `oya-ci-required`. cargo deny/clippy/nextest stay feedback-only for the live gate but are required by Done-Definition D-items.
2. **AUTHORITY-DRIFT-IS-A-FIRST-CLASS-RISK (NEW):** because a documented authority migration is in flight inside the execution window, every loop iteration begins by re-diffing live protection; a flip HALTS for USER (G0). Serial sequencing's long wall-clock is the exposure axis — this is acknowledged, not hidden.
3. **MIGRATE-CLEAN-FIRST / no_std-LAST:** gate-clean STD trees land first; the **only no_std tree is `linux/stack/kernel`** (framekernel). node-os (`talos-*`, STD 1.96.0) is a normal STD lane. The no_std framekernel lands LAST, toolchain-isolated, workspace-EXCLUDED, Buck2-driven, own pinned nightly-2026-02-28. Lane 0.6 proves BUILD-capability AND graph-INERTNESS of the EXCLUDED STATE (including the `exclude`-key edit) empirically.
4. **ONE-LANE-ONE-PR-WHOLE-GRAPH-GATED:** each landing zone = one squash-PR to dev. Because the gate is GLOBAL, the loop rebases each lane on current dev and re-runs the whole-graph buck2 -check targets locally BEFORE opening its PR. "Fully gated" = live blocking check green AND all conversations resolved AND linear history preserved.
5. **MERGE-NOT-DUPLICATE-OPERATIONALIZED:** each lane carries a CREATE-vs-MERGE attribute fixed in Lane 0.5 from a crate-level surface manifest. MERGE lanes (codex-adapter; k8s → 4 `managed-k8s-*` + resolved `cloud/cloud-k8s` relationship) run a merge-surface diff against live crates first. The k8s/containerd `crates/` dir is split 44 `ctrd_*` (→ container-runtime CREATE) vs 95 (→ k8s MERGE) in the manifest.
6. **ONLY-OUR-CODE-MOVES (per-tree parameterized):** only first-party crates + docs migrate; vendored `_upstream*`/`third-party`/reindeer caches, `legacy-port`, `legacy-kernel`, `stack/talos-reference`, `__pycache__`, `prelude`, `toolchains`, `target`, `buck-out`, `.omc/.omx/.claude/.c2work` stripped at boundary via **per-lane allowlist + per-tree deny-glob diff gate** (deny-globs differ per source tree, enumerated in 0.5).
7. **CANONICAL-HOMES-WITH-STANDING-tools/-EXCEPTION:** homes are ONLY `{oya,cloud}/<service>/crates/<crate>` + `libs/`. `tools/`, `services/`, `platform/`, `modules/` are RETIRED **EXCEPT** the gate-load-bearing `//tools/oya-doc-staleness-inventory-app` + `//tools/oya-adr-index-regenerator-app`, which remain a STANDING invariant exception (USER-ratified G2). Any tools/ crate removed from disk is removed from the 723 root members in the SAME atomic step.
8. **LIVE-COMPUTED-IDENTIFIERS:** ADR free block, codex-adapter merge surface, managed-k8s merge surface, and codename→`oya-*` names computed against LIVE dev HEAD + in-flight adr-*/chore branches at execution time (regenerate via `tools/oya-adr-index-regenerator-app`); docs lane recomputes + re-verifies-then-rebases immediately before merge. "~0519+" is a placeholder, never an assignment.
9. **FORGE-EXPLICIT:** every push/PR pins `git push github-mirror` / `gh pr create --repo jason931225/oyatie --base dev`; NEVER `origin` (Forgejo); NEVER assume the working branch is the base.
10. **GATE-BEFORE-START:** no lane executes until the kernel workflow is DONE, the independent kernel gate re-verify is green (check-tcb PASS, diff-oracle PASS, both build PASS, assert-talos re-confirming), pre-lanes 0.4/0.5/0.6/0.7 complete, and USER sign-off obtained.

## 2. Decision Drivers (top 3)

1. **AUTHORITY DRIFT IS THE DOMINANT CORRECTNESS RISK.** A documented, tooled, in-flight migration (`github-lane-unlocker-required` → `oya-ci-required` + signing, ADR-0513, 2026-06-04) sits inside the campaign window. Strict-serial wall-clock maximizes exposure. The plan must build to the live gate AND be flip-ready (signing pre-provisioned, `oya-ci-required` characterized) or risk every queued lane reddening mid-campaign.
2. **THE LIVE MERGE GATE IS A BUCK2-ONLY WHOLE-GRAPH CHECK.** Verified: required context `[github-lane-unlocker-required]` runs only `buck2 build //:...-check` + `//tools/...` targets; never cargo. The campaign bricks identically on all lanes if Buck2-graph completeness/authority is mis-targeted; reindeer buckify + per-crate BUCK + coverage-check green is the TOP Done-Definition item.
3. **SURFACE ENTANGLEMENT BLOCKS "MERGE-NOT-DUPLICATE" UNTIL MANIFESTED.** k8s has 5–6 live homes (4 `managed-k8s-*` + `cloud/cloud-k8s`), containerd is 44 `ctrd_*` crates intermingled in the k8s `crates/` dir, db-engine has no source, and brand residue (`oyaoffice-*`, `openai-codex-sdk`, `talos-*`) forces heavy renames. None of "merge not duplicate / only our code moves" is verifiable until Lane 0.5 emits crate-level surface + source manifests.

## 3. Pre-Mortem (deliberate mode — ≥3 scenarios)

1. **AUTHORITY-FLIP MID-CAMPAIGN (NEW, top risk).** `oya-ci-required` goes live before the last lanes merge. Instantly: every lane's "buck2 coverage-check green" Done-Definition is no longer sufficient; signed commits become mandatory; already-rebased-but-unmerged lanes must be re-signed and re-verified against an uncharacterized gate. **Likelihood: MEDIUM–HIGH** (documented, tooled, dated one day before the plan; long serial wall-clock). **Mitigation:** pre-lane 0.4 + loop step 0 re-diff live protection every iteration; G0 HALTs for USER on flip; signing pre-provisioned as a HARD precondition; `oya-ci-required` characterized in 0.5 so the campaign pivots without re-discovery; each lane's Done-Definition asserts the live required context still matches what the lane built against.
2. **WRONG-GATE BRICK.** A lane ships green cargo deny/clippy/nextest but `//:buck2-cargo-target-coverage-check` rejects it (Buck graph drift, or a BUCK hand-edited without `reindeer buckify`); repeats on all lanes. **Likelihood: HIGH if unmitigated.** **Mitigation:** every lane runs the full whole-graph buck2 -check matrix + `infra/ci/buck2-affected-gate.sh origin/dev HEAD` locally against freshly-rebased dev BEFORE opening the PR; Lane 0.5 dry-runs the full unlocker matrix on a scratch branch to prove the local toolchain reproduces the server gate.
3. **tools/-RETIREMENT / db-engine PHANTOM BRICK.** Literal tools/ retirement removes `//tools/oya-doc-staleness-inventory-app` (or `//tools/oya-adr-index-regenerator-app`) and reds the gate on the next PR; OR a `cloud/cloud-data` CREATE lane is opened for db-engine whose source does not exist and stalls. **Likelihood: HIGH if locked map honored literally / MEDIUM for db-engine.** **Mitigation:** Lane 0.5 enumerates every `//tools/...`+`//services/...` target the workflow builds and scopes retirement to EXCLUDE them (standing invariant exception, USER ratifies G2); db-engine source confirmed in 0.5 — if absent, the lane is DROPPED (→ ~10 lanes) and noted in open questions; root-member removal is atomic with disk removal.
4. **no_std TREE PERTURBS STD LANES.** The excluded `stack/kernel` no_std tree lands on disk and `//:buck2-cargo-target-coverage-check` registers a Cargo↔Buck parity gap on an STD lane; OR the `exclude`-key edit itself perturbs the check. **Likelihood: MEDIUM.** **Mitigation:** Lane 0.6 proves inertness of the EXCLUDED STATE INCLUDING the `exclude`-key edit empirically (run the coverage-check with the tree present + excluded); framekernel lands LAST so any residual perturbation is contained to its own lane; USER sign-off on the 0.6 result before any STD lane opens.
5. **SERIAL WALL-CLOCK / GLOBAL-GATE ROLLBACK.** ~11 strictly-serial lanes each pay full Buck2 gate cost on `ubuntu-24.04-arm` runners; three+ front-loaded pre-lanes push first merge right; a merged lane can redden the next lane's global graph and a revert may not cleanly restore it once the next lane rebased. **Likelihood: MEDIUM (cost) / LOW–MEDIUM (rollback).** **Mitigation:** acknowledged as the cost of graph-coupling safety (Option B's parallelism amplifies graph/ADR-slot/codex-adapter races under a single shared unlocker app + `required_conversation_resolution=true`); clean rollback is real only for cargo-side (non-blocking) failures; each merge is followed by rebase-on-dev + re-run-authority-gate to keep the next lane honest; squash-merge keeps history linear and reverts atomic.

## 4. Options Considered

- **A\* — Strict serial single-driver ralph loop, STD-first / no_std-last, with gating pre-lanes 0.4/0.5/0.6/0.7 [CHOSEN].** Matches LOCKED one-PR-per-zone + STD-first/no_std-last exactly; pre-lanes eliminate the four catastrophic-discovery risks (authority-flip, wrong-gate, tools/+db-engine, no_std blast radius) before any data lane; serial loop sequences the global Buck2 authority graph so only one lane mutates it at a time; single driver keeps the 723-member one-version root workspace coherent; targets the live gate explicitly AND is flip-ready. Cons: lowest throughput; rollback is clean only cargo-side; Lane 0.7/docs are asymmetric; serial wall-clock maximizes authority-drift exposure (mitigated by step-0 re-diff + G0).
- **B — Phased parallel fan-out (batch independent STD lanes in worktrees, converge on root Cargo.toml, no_std last).** Higher throughput, shrinks the drift window. Rejected: parallel lanes all mutate the 723-member root `Cargo.toml`/`Cargo.lock` → one-version drift + conflicts; concurrent PRs race on the shared whole-graph `//:...-check` + the single shared `github-lane-unlocker-required` app with `required_conversation_resolution=true`; live-computed ADR free block + codex-adapter/managed-k8s surfaces become race-prone (two lanes claim a slot / both touch the codex adapter). Parallelism amplifies the unavoidable global coupling; serial at least sequences it.
- **C — Big-bang dual-PR (all STD in one PR, all no_std in a second).** Rejected: directly violates LOCKED one-PR-per-landing-zone; a mega-PR is unreviewable, cannot resolve all conversations, fails D1..D18; any single `//:...-check` failure blocks the whole migration; catastrophic revert blast radius; multispectrum evidence + ADR-shape unattributable.

## 5. Chosen Option

**A\*** — preserved exactly where it was correct (serial single-driver, STD-first/no_std-last, gating pre-lanes, rebase-on-dev + re-run-authority-gate after each merge, MERGE-not-duplicate, only-our-code-moves, live-computed identifiers). Architect/Critic findings injected as: (R1) authority-flip handled by pre-lane 0.4 + loop step 0 + gate G0 + signing-as-hard-precondition + BUILD-TO-BOTH principle; (R2) generator citation corrected, 0.6 inertness proof made empirical; (R3) k8s/cloud-k8s + containerd split manifested in 0.5; (R4) per-lane source inventory in 0.5, db-engine gated/droppable; (R5) squash-merge; (R6) tools/ as standing canonical-homes exception; (R7) 0.6 proves excluded-state-including-exclude-edit inertness; (R8) ADR regen tool corrected; (R9) codename rename pass per lane. Options B and C remain rejected.

---

## 6. Lane Sequence (gating pre-lanes → STD lanes → no_std last)

**GATE-BEFORE-START** (kernel workflow DONE + independent kernel gate re-verify green) then:

- **0.4 AUTHORITY-SNAPSHOT + SIGNING PRE-PROVISION** (NEW gating pre-lane, runs FIRST)
- **0.5 GOVERNANCE-SURFACE TRUTHING + SOURCE/MERGE-SURFACE MANIFESTS**
- **0.6 no_std BUILD-CAPABILITY + WHOLE-GRAPH-INERTNESS SPIKE** (excluded-state incl. exclude-key edit)
- **0.7 GOVERNANCE-FILE BOOTSTRAP**
- **L1 office** (`oya/office`, CREATE) — STD, rename `oyaoffice-*`→`oya-office-*`
- **L2 oyago** (`oya/transpiler-go-to-rust`, CREATE) — STD, rename `oyago-*`→`oya-transpiler-go-to-rust-*`
- **L3 oyapy** (`oya/transpiler-python-to-rust`, CREATE) — STD, rename `oyapy-*`→`oya-transpiler-python-to-rust-*`
- **L4 claude SDK** (`cloud/cloud-intelligence/crates/oya-cloud-intelligence-anthropic-claude-adapter`, CREATE) — STD, rename `claude-agent-sdk`
- **L5 codex SDK** (MERGE into existing `cloud/cloud-intelligence/crates/oya-cloud-intelligence-codex-adapter`) — STD, merge-surface diff first
- **L6 k8s (our crates)** (MERGE into 4× `cloud/managed-k8s-*` + resolved `cloud/cloud-k8s` relationship) — STD, 95-crate subset of the 139
- **L7 containerd** (`cloud/cloud-container-runtime`, CREATE) — STD, 44 `ctrd_*` subset of the 139
- **L8 cloud-data / db-engine** (`cloud/cloud-data`, CREATE) — **CONDITIONAL: only if db-engine source confirmed in 0.5; else DROPPED**
- **L9 node OS** (`cloud/cloud-node-os`, CREATE) — STD (talos-* 1.96.0), rename `talos-*`→`oya-cloud-node-os-*`
- **L10 docs** (`docs/{context,research}` + 13 ADRs renumbered into the LIVE-computed free block via `tools/oya-adr-index-regenerator-app`) — recompute + re-verify-then-rebase immediately before merge
- **L11 framekernel (no_std, LAST)** (`cloud/cloud-kernel`, CREATE) — workspace-EXCLUDED, Buck2-driven, nightly-2026-02-28, build-std custom targets

> ~11 lane PRs (L8 droppable → as few as ~10). One squash-PR per landing zone. Pilot scaffold (HANDOFF/PROGRESS/STRUCTURE/AGENTS.md/lane charters) retired as part of L10/cleanup, never migrated.

---

## 7. Per-Lane Detail

### Pre-lane 0.4 — AUTHORITY-SNAPSHOT + SIGNING PRE-PROVISION (NEW)
- **Source:** live `gh api .../branches/dev/protection` + `.github/branch-protection.yaml@dev` + check-runs on dev HEAD.
- **Home:** none (governance precondition).
- **Key steps:** (1) capture a baseline snapshot of the live required contexts + the documented target; DIFF them; (2) if the live required context has ALREADY flipped to `oya-ci-required`, **HALT for USER (G0)** before any further work; (3) since the target declares `require_signed_commits: true`, **PRE-PROVISION the signing key now** (USER credential gate) and verify `commit.gpgsign`/`user.signingkey`/`gpg.format`/`tag.gpgsign`; (4) record the baseline so loop step 0 can detect drift.
- **Gates:** G0 (no flip / USER acknowledges), signing provisioned.
- **Blockers:** USER signing key + GitHub push credentials.

### Pre-lane 0.5 — TRUTHING + SOURCE/MERGE-SURFACE MANIFESTS
- **Source:** `github-mirror/dev` workflow + root `Cargo.toml` + all pilot/sibling trees.
- **Key steps:** (1) enumerate every `//tools/...`/`//services/...` target the unlocker workflow builds; scope retirement to EXCLUDE them (standing exception, G2 USER ratify); (2) **SOURCE-INVENTORY**: for each landing zone record (source path, first-party allowlist, per-tree deny-globs) — resolved paths in §0; (3) **CONFIRM db-engine source exists** — if not, mark L8 DROPPED; (4) **k8s/containerd crate-level split manifest**: assign each of the 139 `linux/stack/kubernetes/crates/*` to k8s-MERGE (95) vs container-runtime-CREATE (44 `ctrd_*`) vs vendored-exclude; (5) resolve `cloud/cloud-k8s` relationship (sixth merge target / docs-only / out-of-scope); (6) codex-adapter + managed-k8s merge-surface diffs; (7) ratify codename→`oya-*` canonical names per lane; (8) **characterize `oya-ci-required`** (producer crate `oya/ci-controller/...`, what it posts/checks); (9) dry-run the full unlocker matrix on a scratch branch to prove the local toolchain reproduces the server gate; (10) determine whether the gate reads root vs `docs/` DOC-CATALOG/CHANGELOG.
- **Gates:** G2 (tools/ exception ratified), surface/source manifests complete, scratch-branch matrix green.
- **Blockers:** db-engine confirmation, `cloud/cloud-k8s` decision, codename ratification (USER).

### Pre-lane 0.6 — no_std BUILD-CAPABILITY + WHOLE-GRAPH-INERTNESS SPIKE
- **Source:** `linux/stack/kernel` (nightly-2026-02-28, build-std `aarch64-unknown-none-softfloat` + `x86_64-unknown-none`).
- **Key steps:** (1) prove the kernel BUILDS isolated under its own pinned nightly + custom targets; (2) **EMPIRICALLY** prove the EXCLUDED STATE is inert against `//:buck2-cargo-target-coverage-check` — add the `[workspace] exclude` entry to root `Cargo.toml` (today there is none) AND place the tree on disk, then run the coverage-check + `scripts/ci/generate-first-party-buck.rs` check; the inertness claim is the EMPIRICAL result, not the (corrected) generator citation; (3) confirm `reindeer`/first-party generators ignore the excluded subtree; (4) USER sign-off on the result.
- **Gates:** build PASS, excluded-state-incl-exclude-edit coverage-check inert, USER sign-off.
- **Blockers:** USER sign-off; nightly-2026-02-28 toolchain availability.

### Pre-lane 0.7 — GOVERNANCE-FILE BOOTSTRAP
- **Key steps:** stage the per-PR governance scaffolding the gate reads (multispectrum evidence dir `/evidence/multispectrum/`, PR-body 5-H2 template, Done-Definition D1..D18 checklist, `## Code Review` hook expectation, DOC-CATALOG/CHANGELOG row location per 0.5) so each data lane is reproducible.
- **Gates:** governance scaffold present and gate-readable.

### STD lanes L1–L9 (uniform shape; per-lane source/home/rename from 0.5)
Each lane: (step 0) **re-diff live protection vs 0.4 baseline — if flipped, HALT G0**; (1) rebase a fresh branch on current dev; (2) allowlist-copy first-party crates + docs only, apply per-tree deny-globs; (3) codename→`oya-*` rename, enforce `package.name==basename` + `oya-*` prefix + brand-residue scan (FORBID `foundry-*`/`oyatie-*`/`oyago`/`oyapy`/`oyaoffice`/`kuberos`); (4) for MERGE lanes (L5 codex, L6 k8s) run the merge-surface diff against live crates FIRST; (5) add crates to the 723-member root `Cargo.toml` (one root workspace, one-version, no nested `[workspace]`); (6) `reindeer buckify` + author per-crate BUCK → Cargo+Buck2 DUAL build; (7) `cargo deny check` + `clippy --workspace --all-features --all-targets -D warnings` + `nextest --workspace --all-features` (feedback) + `data_class` on every new kernel-struct field; (8) run the whole-graph buck2 -check matrix + `infra/ci/buck2-affected-gate.sh origin/dev HEAD` locally; (9) emit multispectrum evidence, PR body (5 H2 + `## Code Review`), DOC-CATALOG + CHANGELOG rows; (10) push `github-mirror`, `gh pr create --repo jason931225/oyatie --base dev`, signed commits + linear history; (11) drive `github-lane-unlocker-required` green + resolve all conversations; (12) **squash-merge** (`allow_squash_merge` only); (13) rebase-on-dev + re-run-authority-gate before the next lane.

### L10 docs
As above, plus: regenerate the ADR free block via `tools/oya-adr-index-regenerator-app` (`.omc/automation/adr-index-pipeline.md`) against LIVE dev HEAD + in-flight adr-* branches, renumber the 13 docs ADRs into the computed free block, regenerate `docs/ADR-INDEX.md`, re-verify-then-rebase IMMEDIATELY before merge; retire the pilot scaffold here.

### L11 framekernel (no_std, LAST)
As STD shape but: workspace-EXCLUDED (the `exclude` entry validated in 0.6), Buck2-driven CI lane with its own pinned nightly-2026-02-28 + build-std custom targets; cannot pass `--workspace --all-features` (excluded by design); merge-blocking artifact is its Buck2 graph + the whole-graph -check targets remaining green with the excluded tree present.

---

## 8. RALPH Loop Structure

```
PRECONDITION (once):
  kernel workflow DONE && independent kernel gate re-verify GREEN
    (check-tcb PASS, diff-oracle PASS, both build PASS, assert-talos re-confirming)
  && pre-lanes 0.4, 0.5, 0.6, 0.7 COMPLETE && USER sign-off (G0..G4)

LANE_QUEUE = [L1 office, L2 oyago, L3 oyapy, L4 claude, L5 codex(MERGE),
              L6 k8s(MERGE), L7 containerd, L8 cloud-data(IF db-engine exists),
              L9 node-os, L10 docs, L11 framekernel(no_std,LAST)]

while LANE_QUEUE not empty:
  lane = LANE_QUEUE.pop_front()           # STRICT SERIAL — one driver, one graph mutation
  STEP 0: re-diff live `gh api .../protection` vs 0.4 baseline
          if required context flipped to oya-ci-required -> HALT (G0), wait USER
  STEP 1: git fetch github-mirror; rebase lane branch on github-mirror/dev
  STEP 2: allowlist-copy first-party only + per-tree deny-glob strip
  STEP 3: codename->oya-* rename; basename==package.name; brand-residue scan
  STEP 4: if MERGE -> merge-surface diff vs live crate(s) first
  STEP 5: add to 723-member root Cargo.toml (one-version, no nested [workspace])
  STEP 6: reindeer buckify + per-crate BUCK -> Cargo+Buck2 dual build
  STEP 7: cargo deny/clippy/nextest (feedback) + data_class fields
  STEP 8: run whole-graph buck2 //:...-check matrix + affected-gate.sh LOCALLY
          if red -> iterate (do NOT open PR)   # boulder never stops
  STEP 9: multispectrum evidence + PR body (5 H2 + ## Code Review) + DOC-CATALOG/CHANGELOG rows
  STEP 10: push github-mirror; gh pr create --repo jason931225/oyatie --base dev; signed commits
  STEP 11: drive github-lane-unlocker-required GREEN + resolve all conversations
  STEP 12: assert live required context STILL matches what lane built against (else G0)
  STEP 13: SQUASH-merge (allow_squash_merge only) -> linear history
  STEP 14: rebase-on-dev + re-run authority gate to keep next lane honest

TERMINATION: queue empty && every lane squash-merged && Done-Definition D1..D18 satisfied
             && ADR free block stable && /oh-my-claudecode:cancel
```

## 9. Verification & Test Plan (deliberate — expanded)

- **Per-lane build/lint/test gate:** Cargo+Buck2 dual build; `cargo deny check`; `clippy --workspace --all-features --all-targets -D warnings`; `nextest --workspace --all-features`; `data_class` annotation present on every new kernel-struct field. (no_std L11 excluded from `--workspace --all-features` by design.)
- **Per-lane Buck2 whole-graph gate (PRIMARY, live):** all `//:...-check` targets + `//tools/oya-doc-staleness-inventory-app:...` + `infra/ci/buck2-affected-gate.sh origin/dev HEAD` green locally on freshly-rebased dev BEFORE PR; `reindeer buckify` regenerated (no hand-edited BUCK).
- **Integration smoke:** after each MERGE lane (L5 codex, L6 k8s), build the merged service end-to-end to confirm no duplicate-crate / one-version break; after L11, confirm the excluded no_std tree leaves the whole-repo coverage-check green (Lane 0.6 result re-confirmed in situ).
- **Source-governance gates (per PR):** basename==package.name + `oya-*` prefix; brand-residue scan (FORBID `foundry-*`/`oyatie-*`/`oyago`/`oyapy`/`oyaoffice`/`kuberos`); one-root-workspace/no-nested-`[workspace]`; multispectrum evidence `/evidence/multispectrum/<change_id>-<unix_ts>.json` (v2.4.0, CC-1..7, F1..F13); PR body 5 H2 (Issue/Summary/Verification/Traceability/Evidence) + automated `## Code Review` (merge-gate hook refuses without it); Done-Definition D1..D18; signed commits + linear history; DOC-CATALOG.md + CHANGELOG.md rows; ADR-shape lane.
- **Governance/authority gate (NEW):** loop step 0 + step 12 assert the live required context still matches the gate the lane built against; G0 HALT on flip; signing pre-provisioned (0.4) so a mid-campaign flip does not block queued lanes; `oya-ci-required` characterized (0.5) for pivot-without-rediscovery.
- **Rollback:** squash-merge keeps reverts atomic + history linear; clean per-lane revert is real ONLY for cargo-side (non-blocking) failures — a merged lane can redden the next lane's GLOBAL Buck2 graph, so after any revert re-run the whole-graph -check matrix on the rebased dev before resuming; for no_std (L11) rollback = drop the `exclude` entry + the excluded tree atomically.

## 10. ADR (final consensus record)

- **Decision:** Strict-serial single-driver ralph loop (A\*), STD-first / no_std-last, with gating pre-lanes 0.4 (authority+signing) / 0.5 (truthing+manifests) / 0.6 (no_std inertness) / 0.7 (governance bootstrap), ~10–11 squash-PR lanes to `dev` on `jason931225/oyatie`, BUILD-TO-BOTH gates (live `github-lane-unlocker-required` + flip-ready `oya-ci-required`+signing).
- **Drivers:** authority-drift is the dominant correctness risk; the live merge gate is a Buck2-only whole-graph check; surface entanglement (k8s 5–6 homes, containerd intermingled, db-engine absent, brand residue) blocks merge-not-duplicate until manifested.
- **Alternatives considered:** B (parallel fan-out) — amplifies global-graph + root-workspace + ADR-slot races under a single shared unlocker app; C (big-bang dual-PR) — violates locked one-PR-per-zone, unreviewable, catastrophic blast radius.
- **Why chosen:** A\* matches every locked decision, sequences the global Buck2 authority graph to one mutation at a time, keeps the 723-member one-version workspace coherent, and is the only option that bounds authority-drift exposure via per-iteration re-diff + signing pre-provision.
- **Consequences:** lowest throughput / longest wall-clock (the drift-exposure axis, mitigated by step-0 re-diff + G0); rollback clean only cargo-side; tools/ becomes a standing canonical-homes exception; signing is now a hard precondition; L8 (db-engine) is conditional/droppable.
- **Follow-ups:** USER provisions signing key + push creds (0.4); USER confirms db-engine source + `cloud/cloud-k8s` relationship + codename canonical names (0.5); USER signs off no_std inertness (0.6); if `oya-ci-required` goes live mid-campaign, pivot the gate per the 0.5 characterization.

## 11. User Gates / Credentials

- **G0 (NEW, HALT):** authority flip detected (`oya-ci-required` live) — STOP, USER decides pivot.
- **G1:** GitHub push credentials for `github-mirror` (`origin` is Forgejo, never push there).
- **G2:** ratify the tools/ standing canonical-homes exception (deviation from locked retirement).
- **G3 (PROMOTED to HARD precondition):** provision the signing key (`commit.gpgsign`/`user.signingkey`/`gpg.format`/`tag.gpgsign` are EMPTY today).
- **G4:** confirm db-engine source location (drop L8 if absent); confirm `cloud/cloud-k8s` relationship; ratify codename→`oya-*` canonical names; sign off no_std inertness (0.6); confirm `agent-skills` out of scope.
