# MIGRATION-PLAN RE-SYNC — source migration (pilot + 5 siblings → jason931225/oyatie)

> **Task #6 (ANALYSIS form). PREP ONLY — no trigger pulled.** This doc re-syncs the source migration plan to current post-consolidation consensus. It does NOT mutate `source`, does NOT touch branch-protection, does NOT open a PR or merge. Task #7 (the actual std-first PRs) is the outward, founder-gated execution step and is marked as such below.
>
> **Authoring location:** linux audit dir (read-only on `source`, per the CWD-contamination guard). The founder folds the deltas below into the live `source/.omc/plans/monorepo-consolidation-migration.md` + `UNIFIED-EXECUTION-PLAN.md` at execution time.
> **Date:** 2026-06-07.

---

## 0. Verified ground truth (this session, real output)

| Fact | Evidence |
|---|---|
| CWD self-check passed | `pwd=/Users/jasonlee/Developer/source`, branch `cleanup/whole-tree-2026-06-07`, HEAD `7adae31fb` |
| Migration plan exists | `source/.omc/plans/monorepo-consolidation-migration.md` (197 lines, "REVISED PLAN — ralplan deliberate, post-Architect+Critic") |
| Unified plan exists | `linux/docs/audit/initial-sweep-2026-06-06/UNIFIED-EXECUTION-PLAN.md` (merges WHAT+HOW + D-CONFORM conformance gates) |
| Decision canon exists | `linux/docs/audit/initial-sweep-2026-06-06/synthesis/decision-record-oyatie-canon.md` (~35 ruled D-decisions) |
| **Migration NEVER ran (Task #20 confirmed)** | cleanup branch and `github-mirror/dev` are **DISJOINT — no merge-base** (`git merge-base HEAD github-mirror/dev` → empty). The cleanup tree is a local consolidation; the sibling PRs to `dev` were never opened. |
| **Sibling zones ABSENT from cleanup tree** | `oya/office`, `oya/transpiler-go-to-rust`, `oya/transpiler-python-to-rust`, `cloud/cloud-kernel`, `cloud/cloud-node-os`, `cloud/cloud-container-runtime` all ABSENT in the cleanup tree (verified by dir probe). So consolidation did NOT fold the siblings — they remain external, un-migrated. |
| Sibling SOURCES still on disk | `~/Developer/office/crates` (13), `~/Developer/oyago/crates` (3), `~/Developer/oyapy/crates` (3), `~/Developer/codex/sdk/rust` (1), `~/Developer/claude` (1) |
| Pilot stack sources on disk | `linux/stack/kernel` (19 Cargo.toml; no_std), `linux/stack/operating-system` (45; node-os STD), `linux/stack/kubernetes/crates` (139 total; **44 `ctrd_*`** → 95 k8s) |
| Live gate STILL `github-lane-unlocker-required` (NOT flipped) | `gh api .../branches/dev/protection` → `contexts:[github-lane-unlocker-required]`, `linear:true`, `sigs:false`, `conv:true` |
| Merge method: squash-only | `gh api repos/jason931225/oyatie` → `merge:false, rebase:false, squash:true` |
| Root workspace (cleanup tree) | `Cargo.toml [workspace]` line 1, **726 members** (plan said 723 vs dev — drift), `resolver="2"`, **NO `exclude` key** |
| **Foundry residue is MASSIVE** | **3,459 tracked files** mention `foundry` in the cleanup tree (excl. buck-out/target/third-party/.git); `oya-foundry-supervisor` package names + `context="foundry"` live in `oya/intelligence/`; root `Cargo.toml` has `[workspace.metadata.oya.microservices.foundry]` |

---

## 1. What is now STALE vs current consensus

The WIP plan was authored 2026-06-04/06 against **`dev`** as the merge base. Since then the cleanup branch destroyed/renamed much, and the decision-record canon (2026-06-06/07) ratified the open questions and added new doctrine. The following clauses of the WIP plan are STALE:

### 1a. STALE — base-branch / disjoint-tree assumption
- **WIP says:** lanes rebase on and squash-merge into `github-mirror/dev`; "723-member root workspace" is the live base.
- **Reality:** the active consolidated tree is `cleanup/whole-tree-2026-06-07` (726 members), **disjoint from `dev`** (no merge-base). The migration cannot literally "rebase a lane branch on `github-mirror/dev`" and land cleanly, because the consolidation work + all the A-lane amendments live on the cleanup branch, not on dev.
- **RE-SYNC:** the **migration base is the consolidated cleanup branch** (whatever it is named at execution time — the producer-rooted consolidated source tree), NOT raw `dev`. The first execution-time decision (founder) is the **branch-reconciliation question**: does the consolidated cleanup branch *become* `dev` (force-update / fast-forward-by-policy) before any sibling PR, or do sibling PRs target the cleanup branch which is later promoted? Either way, "rebase on dev" in WIP §7/§8 steps must read "rebase on the consolidated base." **This is a prerequisite the WIP plan does not model.**

### 1b. STALE — db-engine L8 left "conditional"; canon DROPPED it
- **WIP says:** L8 cloud-data/db-engine is CONDITIONAL ("only if db-engine source confirmed in 0.5; else DROPPED").
- **Canon ruled (D-CONFORM, G4-D2):** **L8 is DROPPED** — db-engine has NO source (only a 33KB design spec); the owned distributed-DB engine is a FUTURE BUILD campaign (D4/D-LAYER), not part of this migration. Verified: `cloud/cloud-data` has 0 first-party crates for this.
- **RE-SYNC:** migration is **~10 lanes** (L8 removed, not "conditional"). `cloud-data` stays a docs/design home; no migration lane creates it.

### 1c. STALE — cloud-k8s "open question / sixth surface"; canon ruled it OUT
- **WIP says:** `cloud/cloud-k8s` is an un-modeled 6th merge surface, "gated on USER confirmation."
- **Canon ruled (D-CONFORM, G4-D1):** **`cloud/cloud-k8s` = docs-only (0 crates), OUT as a merge target** — stays design-SSOT / cross-link. Verified: `cloud/cloud-k8s` has 0 crate dirs in the cleanup tree.
- **RE-SYNC:** L6 k8s MERGE targets **`managed-k8s-control-plane-host`** (the 95 apimachinery crates → `oya-cloud-k8s-*` beneath the 17 product crates, ADR-0015/0016), NOT cloud-k8s. cloud-k8s drops out of the merge-surface entirely.

### 1d. STALE — k8s/containerd split "to be manifested in 0.5"; now verified
- **Canon ruled (D-CONFORM):** k8s/containerd split is RATIFIED: **95 k8s** apimachinery crates → `managed-k8s-control-plane-host` (`oya-cloud-k8s-*`); **44 `ctrd_*`** → container-runtime CREATE (L7). Verified this session: 44 `ctrd_*` dirs + 139 total = 95 k8s ✓.
- **RE-SYNC:** the L6/L7 split is no longer a 0.5 deliverable to discover — it is a fixed input.

### 1e. STALE — pre-lane status; canon recorded actual completion
- **Canon (D-CONFORM "Pre-lanes status"):** **0.4 ✓** (G0 no-flip, G3 signing DONE — SSH-signed verified, github-mirror) · **0.5 ✓** (manifests + G2/G4 ratified) · **0.6 DEFERRED to consolidation-time** (the exclude-state-inertness-vs-Buck2 proof needs the real merged root workspace + the **12-entry exclude key** + founder toolchain/G4 sign-off — cannot run under simulate-merged; kernel-builds-isolated half is known-good) · **0.7 ✓** (governance bootstrap bundle).
- **RE-SYNC:** treat 0.4/0.5/0.7 as DONE inputs; **0.6 is the one open pre-lane** and it must run against the REAL merged root workspace at consolidation-time. The WIP's "prove inertness of ONE excluded tree" is wrong — see 1f.

### 1f. STALE — no_std exclude is "1 tree"; canon expanded to 12
- **WIP says:** "the only no_std tree is `linux/stack/kernel` (framekernel)"; L11 excludes 1 tree; 0.6 proves 1-entry inertness.
- **Canon ruled (D-CONFORM #4, founder-directed):** **EXCLUDE the ENTIRE `kernel/` subtree = 12 no_std workspaces** (framekernel + 9 `user-*-src` ELF test-target binaries + `fsbase-worker-src` + `tests-host`) via the `[workspace] exclude` key; also EXCLUDE vendored `third-party/rust/`. COLLAPSE the 2 STD service trees (`kubernetes/` 95k8s+44ctrd, `operating-system/` node-os) into the one-version root.
- **RE-SYNC:** pre-lane 0.6 + L11 must cover the **full 12-entry kernel exclude**, not 1. The `[workspace] exclude` key edit is itself part of the inertness proof.

### 1g. STALE — multispectrum evidence in the per-lane Done-Definition; canon RETIRED it
- **WIP says (Principle §9, STEP 9, D-items):** every lane emits `/evidence/multispectrum/<id>-<ts>.json` v2.4.0 (CC-1..7, F1..F13); merge-gate refuses without `## Code Review`.
- **Canon ruled (D-MULTISPECTRUM-RETIRED, 2026-06-07):** the 21-facet multispectrum doctrine is **RETIRED/SUPERSEDED**; the per-changeset critique half is **DROPPED** (not re-homed); the accounting half → Phase-0 firewall + D-CICD-AUTHORITY. The multispectrum evidence requirement, the GATE-4 reviewer row, and `bespoke-cloud-toolchain-services.json:174` must be DE-REQUIRED.
- **RE-SYNC:** **DELETE the multispectrum-evidence step from every lane's Done-Definition.** Per-lane evidence is whatever the Phase-0 firewall + the one canonical CI require — NOT multispectrum JSON, NOT a `## Code Review` hook that has no on-disk enforcer (canon: `guard-pr-merge-review.mjs` not on disk; 5-H2 traceability checks only 3 booleans).

### 1h. STALE — the "authority-flip to oya-ci-required" gate model; canon collapsed it
- **WIP says:** BUILD-TO-BOTH-GATES — pass live `github-lane-unlocker-required` AND pre-characterize `oya-ci-required`; G0 HALTs on the flip; the flip is the dominant risk.
- **Canon ruled (D-CICD-AUTHORITY, 2026-06-07):** ONE canonical CI, ONE blocking context **`oya-ci-required`**, produced by **GitHub Actions now** (live authority/runner) → owned oya-ci runner after cutover (a runner *migration*, not two parallel runners). **DROPS the parallel-shadow run + verdict-agreement apparatus.** `github-lane-unlocker-required` is the legacy bridge being retired.
- **Reality check (this session):** the LIVE required context is STILL `github-lane-unlocker-required` (not yet `oya-ci-required`) and `sigs:false`. So the *physical* live gate is unchanged, but the *target/doctrine* is now D-CICD-AUTHORITY's single `oya-ci-required`, NOT the WIP's "BUILD-TO-BOTH + flip-HALT."
- **RE-SYNC:** keep the operational truth (today the lane must turn the live blocking check green — currently `github-lane-unlocker-required`) BUT reframe the doctrine target to **the single canonical `oya-ci-required` (GitHub-Actions-produced)**, drop the "shadow / verdict-agreement / BUILD-TO-BOTH" framing, and treat the live→`oya-ci-required` context swap as **Task #23's Phase-0 firewall go-live** (a founder-paired GitHub-admin branch-protection step), NOT an in-campaign flip to defend against. **Sequencing: the Phase-0 firewall must land `oya-ci-required` as the blocking context BEFORE (or as the first act of) the migration**, so all sibling PRs build to the canonical gate, not the dying bridge.

### 1i. STALE — Forgejo "bridge" framing in path/remote prose
- **WIP says:** `origin`=Forgejo (`http://forgejo.local/...`), push `github-mirror`. (The remote facts are CORRECT and verified this session.)
- **Canon ruled (D-FORGE-CLARIFY + D-CLOUD-NATIVE):** **Forgejo is DROPPED entirely** (not a bridge); the brand-stem `forge` is FORBIDDEN VOCAB; GitHub is the ONLY interim forge; bespoke `cloud-scm` is the destination. `infra/forge` consolidates into `infra/gitops`.
- **RE-SYNC:** keep the operational fact (push `github-mirror`, never `origin`) but add `forgejo`/`forge` to the brand-residue FORBID list the migration lanes scan, and ensure no migrated sibling re-introduces a forge/forgejo adapter.

### 1j. NEW residue the WIP brand-scan must catch — `foundry` (3,459 files) + others
- **WIP brand FORBID list:** `foundry-*`/`oyatie-*`/`oyago`/`oyapy`/`oyaoffice`/`kuberos`.
- **NEW reality:** `foundry` residue is now **3,459 tracked files** in the consolidated tree (live package names like `oya-foundry-supervisor`, `context="foundry"`, root `Cargo.toml` microservices metadata). This is governed by **Task #25** (eradicate foundry/forgejo/jenkins/oya-vcs) + **D-FOUNDRY-CLARIFY** (foundry FORBIDDEN; 4-way route: platform→oya-intelligence-current-home / fitness→oya-governance / agentic-VCS→retired; HARD carve-out external "Palantir Foundry").
- **RE-SYNC:** the migration brand-residue gate FORBID enum must be `foundry · forgejo · forge · jenkins · oya-vcs · oyatie- · oyago · oyapy · oyaoffice · talos- · kuberos` (Palantir-Foundry carve-out). **CRITICAL sequencing:** the foundry/forgejo/jenkins eradication (Task #25, the A-lanes) must complete on the consolidated base BEFORE the sibling M-lanes land, so siblings land onto a foundry-clean canon (per UNIFIED-EXECUTION-PLAN §2 sequencing rationale).

### 1k. STALE — per-lane acceptance was build+brand-only; canon added 9 conformance gates
- **Canon ruled (D-CONFORM):** the WIP per-lane acceptance is build+brand-centric; lanes could merge green-on-Buck2 while violating ~12 architecture/governance policies. ADD ~9 conformance gates (BNF layer-suffix enum, hexagonal import-matrix, slot2 registration, manifest hygiene, dependency-rationale no-orphan, vendor A/B/C registry, per-service colocation+buildability, rebrand-arrow/retired-terms scan, `data_class`). KEEP the serial loop / pre-lanes / sequence / authority machinery unchanged.
- **RE-SYNC:** fold the 9 conformance gates into every lane's STEP 7 (per UNIFIED-EXECUTION-PLAN §6). The hexagonal import-matrix is the biggest reshape — the oyago/oyapy/claude/codex monoliths SPLIT into kernel/domain/usecase/adapter/app layers (they are not currently hexagonal).

---

## 2. RE-SYNCED migration plan (current consensus)

### 2.1 Order: std-first / no_std-last (UNCHANGED — canon affirms)
Migrate gate-clean STD trees first; the **12-entry `kernel/` no_std subtree lands LAST** (workspace-EXCLUDED, own pinned nightly-2026-02-28, build-std custom targets `aarch64-unknown-none-softfloat` + `x86_64-unknown-none`). node-os (`talos-*`, STD 1.96.0) is a NORMAL STD lane, not no_std.

### 2.2 The siblings (5) + pilot stack
| # | Lane | Source on disk | Home (canonical) | Mode | Crates | Codename rename |
|---|---|---|---|---|---|---|
| L1 | office | `~/Developer/office/crates` | `oya/office` (CREATE) | CREATE | 13 | `oyaoffice-*` → `oya-office-*` |
| L2 | oyago | `~/Developer/oyago/crates` | `oya/transpiler-go-to-rust` (CREATE) | CREATE | 3 | `oyago-*` → `oya-transpiler-go-to-rust-*` |
| L3 | oyapy | `~/Developer/oyapy/crates` | `oya/transpiler-python-to-rust` (CREATE) | CREATE | 3 | `oyapy-*` → `oya-transpiler-python-to-rust-*` |
| L4 | claude SDK | `~/Developer/claude` | `cloud/cloud-intelligence/crates/oya-cloud-intelligence-anthropic-claude-adapter` (CREATE) | CREATE | 1 | `claude-agent-sdk` → canonical; MIT→Apache-2.0 relicense (D-CONFORM #2) |
| L5 | codex SDK | `~/Developer/codex/sdk/rust` | `cloud/cloud-intelligence/crates/oya-cloud-intelligence-codex-adapter` (NEW sibling crate, D-CONFORM L5) | CREATE (canon refined "MERGE"→NEW sibling, ~0 overlap with the 942-LOC proxy adapter) | 1 | vendor `openai-codex-sdk` |
| L6 | k8s (our crates) | `linux/stack/kubernetes/crates` (95 of 139) | `managed-k8s-control-plane-host` → `oya-cloud-k8s-*` (ADR-0015/0016) | MERGE | 95 | k8s apimachinery |
| L7 | containerd | `linux/stack/kubernetes/crates` (44 `ctrd_*`) | `cloud/cloud-container-runtime` (CREATE) | CREATE | 44 | `ctrd_*` → `oya-cloud-container-runtime-*` (drop snake_case) |
| ~~L8~~ | ~~cloud-data/db-engine~~ | **DROPPED** (D-CONFORM G4-D2 — no source) | — | — | — | — |
| L9 | node OS | `linux/stack/operating-system` (45) | `cloud/cloud-node-os` (CREATE) | CREATE | 45 | `talos-*` → `oya-cloud-node-os-*` |
| L10 | docs | linux pilot docs + 13 pilot ADRs | `docs/{context,research}` + ADRs into the LIVE-computed free block | CREATE/MERGE | — | renumber 13 ADRs additive (no corpus renumber, D13-amend) |
| L11 | framekernel (no_std, LAST) | `linux/stack/kernel` (12-entry subtree) | `cloud/cloud-kernel` (CREATE) | CREATE | 19 (12 workspaces) | workspace-EXCLUDED; Buck2-driven; nightly-2026-02-28 |

**~10 lanes total** (L8 removed). One squash-PR per landing zone. The pilot scaffold (HANDOFF/PROGRESS/STRUCTURE/AGENTS.md/lane charters) is RETIRED at L10/cleanup, never migrated.

### 2.3 The 5 "siblings" precisely
The "5 siblings" = **office, oyago (transpiler-go), oyapy (transpiler-py), claude SDK, codex SDK** (the `~/Developer/*` repos). The **pilot** = the `linux/stack/*` trees (k8s+containerd L6/L7, node-os L9, framekernel L11) + pilot docs (L10). This matches MEMORY's "pilot + 5 siblings → the source monorepo."

### 2.4 PR sequence into jason931225/oyatie (Task #7 — the gated execution)
Per UNIFIED-EXECUTION-PLAN §2, **A-lanes (amendment, source-internal) run BEFORE M-lanes (migration)** in ONE serial ralph loop, so siblings land onto a foundry-clean / integrity-clean / namespaced canon:

```
PRE-LANES (gating, once):  0.4 ✓ · 0.5 ✓ · 0.6 (RUN against real merged root, 12-entry exclude) · 0.7 ✓
                           + Phase-0 firewall go-live: oya-ci-required is the blocking context (Task #23, founder-paired admin step)

A-LANES (Task #25/#26/#13 — clean existing canon, amend-in-place + additive, NO renumber):
  A1 foundry per-file rename (4-way: platform→oya-intelligence / fitness→oya-governance / agentic-VCS→retired; Palantir carve-out)
  A2 integrity sweep (KCMVP/KISA restore · tautology fixes · ALL dangling supersedes/amends on the STABLE id-space)
  A3 vocab namespacing (tier→autonomy_tier/eu_ai_act_risk_tier/dr_tier/storage_tier/tenant_class)
  A4 Proposed-ledger resolution (~122 RATIFY / DROP 0325,0316,0349-claim / AMEND-0352 + CI-cluster ADR-debt drop)
  A5 NEW/reshaped ADRs (additive into live free block: oya-ci reshape, safety-gate, KR pack, infra-seq, meta-ADRs, data-engine-endpoint)
  A6 CC-1..CC-13 doc fixes (Forgejo→GitHub, Kafka→Pulsar, masterplan-authority, Cedar→PARC, etc.)
  + D-MULTISPECTRUM-RETIRED execution (de-require GATE-4 reviewer row; remove dependency-seam multispectrum subcheck atomically)

M-LANES (Task #7 — migrate external code onto the cleaned canon):
  L1 office → L2 oyago → L3 oyapy → L4 claude-SDK → L5 codex-SDK → L6 k8s(MERGE→managed-k8s-control-plane-host) →
  L7 containerd → L9 node-os → L10 docs(+13 pilot ADRs) → L11 framekernel(no_std, LAST)
```

Each lane is ONE squash-PR. The serial loop sequences the global Buck2 graph (one mutation at a time). Per-lane shape (re-synced from WIP §7 STEP 0–13, with the stale steps fixed): step-0 re-diff live protection → rebase on the **consolidated base** (not raw dev) → allowlist-copy first-party only + per-tree deny-glob strip → codename→`oya-*` rename + brand-residue scan (FORBID `foundry·forgejo·forge·jenkins·oya-vcs·oyatie-·oyago·oyapy·oyaoffice·talos-·kuberos`) → MERGE-surface diff for L6 → add to root `Cargo.toml` (one-version, no nested `[workspace]`) → `reindeer buckify` + per-crate BUCK → Cargo+Buck2 dual build → conformance gates (§2.5) → push `github-mirror` → `gh pr create --repo jason931225/oyatie --base <consolidated-base>` → drive the blocking context green + resolve conversations → **squash-merge** → rebase + re-run gate.

### 2.5 Conformance gates each PR must pass (per lane, STEP 7 — D-CONFORM §6)
1. **Live blocking CI green** — today `github-lane-unlocker-required` (Buck2 whole-graph: per-crate BUCK + `reindeer buckify` + `//:buck2-cargo-target-coverage-check` + the `//:...-check` matrix + `infra/ci/buck2-affected-gate.sh` + the 22-target `//tools/oya-*` standing exception); **target = the single canonical `oya-ci-required`** once the Phase-0 firewall is the blocking context.
2. **Cargo+Buck2 dual build** (no_std L11 excluded from `--workspace --all-features` by design).
3. **`cargo deny check` + `clippy --workspace --all-features --all-targets -D warnings` + `nextest --workspace --all-features`.**
4. **BNF layer-suffix ENUM** — every crate ends in the closed-enum suffix (`-kernel/-domain/-usecase/-adapter-<tech>/-app/-check-<discipline>`); reject `-core/-runtime/-port/-api-contracts` + snake_case `ctrd_*`.
5. **Hexagonal layer-import-matrix** — kernels→kernels/ports only; adapters→their kernel + one tech; api→kernel-only; app→no-app; ports-in-kernel. **(Biggest reshape: split the oyago/oyapy/claude/codex monoliths.)**
6. **Microservice slot2 registration** — each migrated service in the flat catalog/registry (ADR-0131/0115).
7. **Manifest hygiene** — `resolver="2"`, `version.workspace=true`, `publish=false`, `license="Apache-2.0"`, `[lints] workspace=true`, `[lib] doctest=false`, workspace-pinned rust-version.
8. **Dependency-rationale no-orphan** — every `[workspace.dependencies]` entry justified; new external dep needs deny.toml clearance + own-vs-reuse rationale.
9. **Vendor A/B/C registry** — vendored deps classified+registered (fix office's misplaced `deny.toml`).
10. **Per-service colocation + buildability-bar** — PRD/contracts/decisions/catalog/slos/threat-model present; builds standalone.
11. **rebrand-arrow / retired-terms scan** — catches retired vocab (M0-M3, tier-system, "Foundry" live, rebrand arrows) beyond the brand-residue word scan.
12. **`data_class`** on every new kernel-struct field.
13. **Linearity + squash-merge** — squash-only enforced; signed commits (signing provisioned in 0.4 — SSH-signed verified; live `sigs:false` today, but build signed-ready).
- **REMOVED vs WIP:** multispectrum evidence JSON (D-MULTISPECTRUM-RETIRED); `## Code Review` hard hook (no on-disk enforcer); 5-H2-as-gate (only 3 booleans checked) — keep these as CONVENTION, not blocking gates.

---

## 3. Prerequisites / sequencing (vs the other doors / big-2)

This migration is **NOT first**. Strict ordering:

1. **GATE-BEFORE-START** (WIP §1.10): the kernel workflow must be DONE and the independent kernel gate re-verify GREEN (check-tcb PASS, diff-oracle PASS, both arch builds PASS, assert-talos re-confirming). Per MEMORY this is largely DONE+verified (P4·SMP is the live frontier on an isolated worktree, but the conformance floor is reached).
2. **Pre-lane 0.6** must RUN against the REAL merged root workspace with the **12-entry `kernel/` exclude key** + founder G4 sign-off (the only open pre-lane; cannot run under simulate-merged).
3. **Phase-0 firewall go-live (Task #23)** — `oya-ci-required` becomes the single canonical blocking context (founder-paired GitHub-admin branch-protection step). The migration should build to the canonical gate, not the dying `github-lane-unlocker-required` bridge. This is the D-CICD-AUTHORITY "runner authority" precondition.
4. **A-lanes (Task #25 foundry/forgejo/jenkins/oya-vcs eradication + #26 CLI-governance + #13 amendments)** must land on the consolidated base FIRST — siblings land onto a foundry-clean / integrity-clean / namespaced canon (3,459 foundry-residue files cannot leak into new sibling crates).
5. **Branch-reconciliation decision (NEW prerequisite, §1a)** — resolve how the disjoint consolidated cleanup branch relates to `dev` BEFORE any sibling PR. The WIP plan assumes a contiguous `dev` base that does not exist.
6. **Big-2 / DEFERRED campaigns are AFTER migration** (UNIFIED §4): full ADR-0000+ re-foundation, the AI-engine RE-HOME (`oya/intelligence`→`cloud/cloud-intelligence`, 96k LOC), governance build-out, AI-substrate maturity. Do NOT entangle these with the migration (they would invalidate citations mid-flight, D13-amend).

**Net:** Task #7 (the PRs) cannot fire until 0.6 + Phase-0-go-live + A-lanes + branch-reconciliation clear. The migration is the LAST step of the consolidation campaign, not a parallel one.

---

## 4. Task #7 — the outward, founder-gated execution step

**Task #7 = the actual std-first sibling PRs into `jason931225/oyatie`.** This is the OUTWARD execution step (it pushes to GitHub + merges). It is **founder-gated** (door:one-way + founder sign-off at every source mutation, per D-RECON dependency-spine). It does NOT fire in this prep. Required founder GO + credentials before Task #7 opens its first PR:

- **G1** GitHub push credentials for `github-mirror` (never push `origin`/Forgejo). [verified remote facts]
- **G2** tools/ 22-target standing-exception — RATIFIED (canon D-CONFORM). [done]
- **G3** signing key — DONE (SSH-signed verified; live `sigs:false` but signed-ready). [done]
- **G4** db-engine DROP (done), cloud-k8s OUT (done), codename names ratified (done), **no_std 0.6 inertness sign-off against the real merged root (OPEN)**.
- **G0** authority-flip HALT → reframed: the migration builds to the canonical `oya-ci-required` after Phase-0 go-live; the live context swap is a founder-paired admin step (Task #23), not an in-campaign hazard.
- **Branch-reconciliation GO** (§1a) — founder decides cleanup-branch↔dev relationship.

---

## 5. Founder action (single next action to fire this prep's downstream)

The migration (Task #7) is blocked on prerequisites, so the single founder action is NOT "open PRs." It is:

> **Fold these RE-SYNC deltas into the live `source/.omc/plans/monorepo-consolidation-migration.md` + `UNIFIED-EXECUTION-PLAN.md` (on a WIP commit on the cleanup branch), then give GO to RUN pre-lane 0.6 against the real merged root workspace with the 12-entry `kernel/` exclude key.** 0.6 sign-off (G4) + the Phase-0 firewall go-live + A-lane completion + the branch-reconciliation decision are the gates that, once cleared, unblock Task #7's first std-first PR (L1 office).

(All mutations to `source` remain founder-go + commit-WIP-first, per the consolidation execution state.)
