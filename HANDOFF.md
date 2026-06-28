# Oyatie — Fresh-Session HANDOFF & Backlog

**Generated:** 2026-06-08 (session wind-down) · **Last updated:** 2026-06-09 (CI/root-surface refresh) · **Owner:** founder · **Author:** Claude (orchestrator)
**Purpose:** authoritative state + backlog so a brand-new session can resume with zero context loss.

**SELF-CONTAINED.** The source repo on GitHub (`jason931225/oyatie`) is the ONLY artifact that survives into a fresh session. Everything needed to resume is in THIS file plus the committed tree that survives with it: the canon ADRs under `docs/decisions/` (ADR-0516…0535), the 3 SSOT stores under `registry/stores/`, and the cloud-ci conformance ratchet under `cloud/cloud-ci/` + `oya-ci.toml`. The local cross-session memory (`~/.claude/...`) and the `linux` repo do **not** survive — do not depend on them. Detailed session-local working plans (migration/productization/CLI-governance/store-schema) are NOT in `dev`; their essence is inlined below, and full copies are preserved on the `consolidate/kernel-snapshot-2026-06-08` branch under `docs/audit/initial-sweep-2026-06-06/` (transient — that branch is deleted once Task #20 consolidation lands).

---

## 0. TL;DR — where everything is right now

- **The canon is ratified and on `dev`.** Current `dev` = `f93a44f9bcdc236c95c4ef9c13aa8aefc7b9095a` after PR #633, carrying the 20-ADR **Agentic Delivery Fabric** cluster (ADR-0516…0535), the 3 SSOT stores, the 0280/0392/0510 status flips, the cloud-ci conformance ratchet, and productized generated-output hygiene.
- **All sibling + kernel work is preserved on source's GitHub** as `consolidate/*` snapshot branches (signed). Nothing is lost. The *gated tree-merge* into the monorepo is still TODO (Task #20).
- **The authoritative CI verdict on `dev` is green as of 2026-06-09** — post-merge `oya-ci-required` push run `27232700111` completed `success` on `f93a44f9bcdc236c95c4ef9c13aa8aefc7b9095a`.
- **Source is the destination for everything** ("source is where everything belongs"). The kernel (`linux/stack`) and the 5 siblings consolidate INTO source.

---

## 1. HARD GUARDRAILS (persist across all sessions — do not violate)

1. **Confirm remotes before push.** In the current `/Users/jasonlee/Developer/oyatie` checkout, `origin` = `https://github.com/jason931225/oyatie.git`. In older `~/Developer/source` clones, `origin` may still point at a retired local mirror; do not push any retired/local mirror remote. The only active source-of-record remote is GitHub `jason931225/oyatie`.
2. **ALL commits SSH-signed.** Identity: `Jason Lee <56489493+jason931225@users.noreply.github.com>`, key `~/.ssh/id_ed25519.pub`, `gpg.format=ssh`. Source has this in **local** config; sibling repos have **no** identity, so sign explicitly: `git -c gpg.format=ssh -c user.signingkey=~/.ssh/id_ed25519.pub -c user.name="Jason Lee" -c user.email="56489493+jason931225@users.noreply.github.com" commit -S`. Verify: `git cat-file commit <sha> | grep "BEGIN SSH SIGNATURE"`.
3. **SOURCE-FORCED.** `cd /Users/jasonlee/Developer/source` + `pwd` self-check for all source work. The session CWD `/Users/jasonlee/Developer/linux` is the kernel-port **contamination trap**.
4. **Verify each step in a separate lane. No rubber-stamping, no phantom findings.** (This session a glowing "complete+pushed" agent report was checked against ground truth before being trusted — and a polling race nearly produced a phantom "missing commit" finding. Always confirm against `git`/`gh`, not narration.)
5. **No blind `git add -A` in source canon work; `git rm` only, never `filter-branch`.** (Exception used this session: sibling *preservation* snapshots used `git add -A` with a build-junk safety rail — justified because the goal was lossless capture and `.gitignore`/`.git/info/exclude` covered `target/`/`buck-out/`.)
6. **Recovery anchor:** `e38624dc4` (on `cleanup/whole-tree-2026-06-07`). A-delete (R2) batches diff against this.
7. **PAUSE-AND-PAIR (do not do unilaterally):** live GitHub branch-protection ruleset toggles; canon ratification (door:one-way founder sign-off); prod/access.
8. **Cloud-ci conformance ratchet = CI-product invariant:** keep it canonical/configurable/reusable. Gate predicate `.rs` sources + `oya-ci.toml` + `oya-ci-config` crate + workflow `.yml` must stay byte-unchanged unless deliberately changing the product. Generated JSON faces are CI/controller outputs, not contributor merge surfaces: do not commit add/modify changes to `*.generated.json`; `buck2 run //cloud/cloud-ci/gates/oya-cloud-ci-freshness-app:oya-cloud-ci-materialize-generated-faces-bin` materializes them before gates consume them, and `generated-output-diff-policy` rejects generated add/modify PR surfaces while allowing generated-output deletion retirements.

---

## 2. Repo / remote topology

| Repo (local) | Role | Remote | Branch state |
|---|---|---|---|
| `~/Developer/oyatie` | **the oyatie monorepo (destination)** | `origin` = jason931225/oyatie | `dev` = `f93a44f9bcdc236c95c4ef9c13aa8aefc7b9095a` (canon + generated-output hygiene) |
| `~/Developer/linux` | kernel port (kuberos-kernel + `stack/`) | **none locally** (now snapshotted to source's github) | `main` = `26173992` (+ snapshot commit) |
| `~/Developer/{office,oyago,oyapy,codex,claude}` | the 5 siblings to consolidate | `oyatie` remote added → source's github | snapshotted (see §3) |

---

## 3. PRESERVED THIS SESSION — `consolidate/*` branches on source's GitHub (all signed)

| Branch on jason931225/oyatie | From | Content | Notes |
|---|---|---|---|
| `dev` (`f93a44f9b`) | source | the ratified canon + productized generated-output hygiene | `oya-ci-required` run `27232700111` green |
| `consolidate/codex-snapshot-2026-06-08` | codex | SDK (`sdk/rust`), 36 files | unborn→first commit |
| `consolidate/office-snapshot-2026-06-08` | office | `oyaoffice-*` crates, 167 files | maps → `oya/office/` |
| `consolidate/oyago-snapshot-2026-06-08` | oyago | Go analyzer + `oyago-*` + fixtures, 249 files (~22 MB) | destination TBD |
| `consolidate/oyapy-snapshot-2026-06-08` | oyapy | Python analyzer + `oyapy-*`, 117 files | destination TBD |
| `consolidate/claude-snapshot-2026-06-08` | claude | Rust agent tooling, full 78-commit history + deltas | destination TBD |
| `consolidate/kernel-snapshot-2026-06-08` | linux | kernel + `stack/` working tree, 624 files | destination TBD (no `stack/` in source yet) |

These are **isolated branches** — they do NOT touch `dev`/`main` and do NOT run the firewall. They exist purely so no work is lost. The real consolidation is Task #20.

---

## 4. Sibling/kernel CONSOLIDATION MAP — **FOUNDER-AUTHORITATIVE (2026-06-08)**

| Component (preserved branch) | What it is | Destination in source | Exists? |
|---|---|---|---|
| **kernel** (`linux/stack/kernel`) | kuberos framekernel | **`cloud/cloud-kernel/`** | NEW |
| **OS** (`linux/stack/operating-system`) | Talos-style OS | **`cloud/cloud-os/`** | NEW |
| **office** (`consolidate/office-*`) | productivity suite ("oyatie-office") | **`oya/office/`** — `oya-office-*` crates already landed (L1 pilot); sibling `oyaoffice-*` → rename `oyaoffice-`→`oya-office-`, reconcile deltas | EXISTS |
| **claude** (`consolidate/claude-*`) | **SDK adapter for intelligence**, Rust (Anthropic) | **`oya/intelligence/`** | EXISTS |
| **codex** (`consolidate/codex-*`) | **SDK adapter for intelligence**, Rust (OpenAI) | **`oya/intelligence/`** | EXISTS |
| **oyago** (`consolidate/oyago-*`) | **Go → Rust transpiler (WIP)** | transpiler tooling area (exact path TBD) | — |
| **oyapy** (`consolidate/oyapy-*`) | **Python → Rust transpiler (WIP)** | transpiler tooling area (exact path TBD) | — |
| **kubernetes** | k8s control plane | **`cloud/cloud-k8s/`** | EXISTS |

Note: `consolidate/kernel-snapshot-2026-06-08` holds BOTH kernel + OS (`linux/stack/{kernel,operating-system}`); the split into `cloud/cloud-kernel` + `cloud/cloud-os` happens during consolidation. claude+codex are the two intelligence SDK adapters → both land under `oya/intelligence/` (which exists, with `crates/`, `contracts/`, `capabilities/`).

**Consolidation pattern (from the office pilot, Tasks #62–67):** copy → rename crates to conform (BNF 13-suffix enum, `oya-` cargo prefix, manifest-hygiene) → add to root workspace → `cargo`/`buck2` build+test green → floor-gate green → freeze → signed atomic commits → PR to `dev` (firewall must pass).

---

## 5. The ratified vision (context for the backlog)

**Agentic Delivery Fabric** (deep-interview-converged + founder-ratified 2026-06-08): an owned, cloud-native, ∞-scale, **productized** unified **SCM + CI + CD** over **one owned AST substrate**, built so anyone can automatically create + maintain hyperscaler-grade projects, with **AI agents as the primary producers**. One content-addressed work-area hash = SCM change-id = buck2/RBE key = CD artifact. Canon = ADR-0516…0535.

**Owned stack (kernel→fabric):** Fabric → bespoke distributed-SQL (Spanner/CockroachDB-class, multi-Raft leader-per-range) + bespoke ∞-scale object-store (CAS) → k8s+containerd → Talos-style OS → kuberos-kernel. Transitional-impl-behind-stable-interface → owned-bespoke, cutover-gated (ADR-0510).

**SCM anchor (panel-ratified 2026-06-08):** **Google (Piper/CitC/Rosie/TAP) + Meta (Sapling/Mononoke/EdenFS/CommitCloud) = the DESTINATION architecture anchor**; **Microsoft (Scalar/partial-clone) = transitional git-phase virtual-materialization bridge only** (book it; retire at cutover); **Amazon = interface-version discipline only** (reject polyrepo). Corrections: "Mononoke is Rust" was REFUTED → Meta is a *data-model study*, build the server bespoke-Rust; **no-single-leader resolves to leader-per-range (multi-Raft), NOT fully-leaderless** (cite Spanner/CRDB DB-literature, not "Google's SCM"); ~0%-wasted-work is OUR AST claim-time-locking innovation, not proven by CitC/EdenFS.

**Staged roadmap:** W0 done (hermetic buck2 + firewall live + 3 stores) · **W1 = convergence (in progress)** · W2 owned AST parser + gates + auto-remediation · W3 productization + `oya new` + gate/plugin SDK + RBE · W4 bespoke SCM (cutover-gated) · W5 bespoke DB + object-store · W6 bespoke CD.

---

## 6. OUTSTANDING BACKLOG

### 6.1 Immediate (verification + wrap-up)
- [x] **CI trigger gap closed (verified 2026-06-09):** `.github/workflows/oya-ci-required.yml` now runs on `workflow_dispatch`, `push` to `dev`, `pull_request` to `dev`, and `merge_group`; branch protection requires the single `oya-ci-required` context; post-merge push run `27232700111` was green.
- [x] **Legacy workflow retired:** `.github/workflows/backbone-microservices-ci.yml` is absent from active workflows; the remaining active workflows are `oya-ci-required.yml` (required) and `docs-graph-drift.yml` (feedback-only, non-required).
- [x] **Masterplan-reachability DEVIATION (wired 2026-06-09):** ADR-0516..ADR-0535 are now wired into `masterplan.json` `planning_authority.bound_adrs` + P-TOOLCHAIN `authoring_adrs` and `master-plan-sequencing.json` `fabric_cluster_adrs`; the stale `unpropagated_decision` signoff entries for those 20 ADRs were retired so future reachability loss fails the ratchet instead of reusing the bootstrap door. Remaining pre-existing unpropagated baseline keys are outside this slice.
- **Remote hygiene DONE (2026-06-08):** github-mirror pruned to **12 protected heads** (dev, main, production, staging, cleanup/whole-tree-2026-06-07, phase0/producer + the 6 `consolidate/*-snapshot-2026-06-08`); 206 stale branches deleted. The `consolidate/*` snapshots are intentionally NOT merged into dev — that is the **future gated Task #20** consolidation (merging raw non-conformant trees now would RED the firewall with ~225 BNF/manifest/prefix violations; the office-pilot rename→conform→workspace→gate-green pattern is required first).

### 6.2 W1 convergence remainder (Task #74)
- [ ] **R2 — A-delete sprawl:** per-batch destructive deletion vs recovery anchor `e38624dc4`, AFTER dev CI verified green + checkpoint. Gated.
- [ ] **Descriptive renames (Task #25 scale):** `firewall` → `oya-ci-conformance-ratchet` / "conformance ratchet" (founder: "firewall is not descriptive"); `council`/`council-architecture` → `founder` canon-wide (multispectrum-review retired); residual `foundry` references (ADR-0280 Foundry-DAG node, ADR-0017 brand, retired-CLI). Note: `specs/masterplan.json` test fixture still has `council-architecture` owner.
- [ ] **Lock the interfaces** with ∞-scale baked in: WorkAreaTree, scm-facts (ADR-0526 `ScmFactsSource`, GitCli impl #1), object-store-kernel, DB/metadata trait, gate Finding contract (ADR-0528), content-address.
- [ ] **Zero-shell refactors** (`scripts/*.sh` → declarative buck2 targets; irreducible-glue ledger).
- [ ] **Migration L2→L11** conformance readiness.
- [ ] **Root-surface cleanup:** root scratch is born-blocked by the root-workspace-hygiene gate; `bin/oya` and `.envrc` are retired with the CLI/PATH shim surface instead of kept alive; `.config/nextest.toml`, `plan/`, and `.gitattributes` remain tracked until separate migration evidence says otherwise; `.claire`, root `test/`, and root `tests/` remain absent; `.omc/` and `.omx/` durables are local-only and must stay gitignored; live planning authority points at tracked `/specs`/`/registry`/`/evidence`/`/templates` surfaces instead of `.omc`/`.omx`.

### 6.3 Consolidation (Task #20 + #7)
- [ ] **Task #20 — Monorepo-conformance: AUDIT → RALPLAN → CONSOLIDATION** of the 5 siblings + kernel (see §4 map). Decide destinations for oyago/oyapy/claude/kernel. Respect the firewall.
- [ ] **Task #7 — execute source migration** std-first lane PRs into jason931225/oyatie.
- [ ] **Task #52 (AP6)** — migration #6 deltas → WIP commit → #7 PRs.

### 6.4 CLI-governance + vocab (Tasks #25, #26)
- [ ] **Task #26** — migrate the 72 `oya-check-*` libs + 11 `oya-dev-cli` gate subcommands into the firewall ratchet; retire `oya-gate`/`oya-cli`/`oya-dev-cli`. Also retire the legacy `backbone-microservices-ci.yml` (permanently-red, non-required). SUBSTANCE: build one `oya-cloud-ci-<discipline>-app` crate per check (mirror the live gates), baseline existing violations + block-NEW, fold into the producer + `oya-ci-required` matrix; reuse the existing check LOGIC (do not rewrite). Floor already done: bnf-layer-suffix (79), manifest-hygiene (233), cargo-prefix. (Full ralplan preserved on the `consolidate/kernel-snapshot` branch: `docs/audit/.../CLI-GOVERNANCE-TO-FIREWALL-MIGRATION-PLAN.md`.)
- [ ] **Task #25** — finish forbidden-vocab eradication (foundry/forgejo/jenkins/oya-vcs residue, incl. README's old "Foundry" mention if still present).

### 6.5 Roadmap forward (W2–W6) + product
- [ ] **W2 (high prio, agentic-dev-primary):** owned AST parser ("tree-sitter-our-way" + markdown, rowan-style, content-addressed node identity) behind `WorkAreaTree`; AST practice/anti-pattern gates + behavior-preserving auto-fix; AST doc-tracking (`stale_reference`/`unreachable_doc`/`broken_link`/`derived_doc_drift`); work-area affected-set; auto-remediation bot fleet.
- [ ] **AUTO/ADVISE/GATE safety governor** (ADR-0519/0529): every finding-code declares a tier; meta-gate rejects untagged; AUTO requires the 5 safety proofs.
- [ ] **W3:** de-oyatie config + gate/plugin SDK + marketplace; `oya new` scaffolder; generated forge adapters; buck2 RBE/NativeLink.
- [ ] **W4–W6:** bespoke SCM (per §5 anchor) → bespoke DB + object-store → bespoke CD + full fabric assembly.
- [ ] **Task #18** — vertical-coverage map (incl. net-new defense + power-grid) → ADRs + masterplan.
- [ ] **Tasks #13/#14** — ADR amendments per dispositions; stale-file audit (>48h, gated).

### 6.6 Kernel (was `linux/stack` locally — now preserved on github as `consolidate/kernel-snapshot-2026-06-08`; destination `cloud/cloud-kernel` + `cloud/cloud-os`)
- [x] Hermeticity Stage A + B done (reproducible musl-static talos-init/svc carriers; no Docker; external-blob debt closed).
- [x] S4c cross-CPU TLB shootdown enabled both arches (F-0020 resolved).
- [ ] WAVE1 conformance test; SMP frontier (S4b work-stealing-deque + reschedule-IPI); P4·SMP.
- [x] Decide kernel's destination in source (§4): kernel → `cloud/cloud-kernel/`, OS → `cloud/cloud-os/`.

---

## 7. OPEN QUESTIONS FOR FOUNDER (surface early in the new session)

1. **Sibling/kernel destinations** — RESOLVED by founder (§4): kernel→`cloud/cloud-kernel`, OS→`cloud/cloud-os`, office→`oya/office`, claude+codex→`oya/intelligence` (SDK adapters), k8s→`cloud/cloud-k8s`. Only the **oyago/oyapy transpiler exact path** remains to pin (they're WIP).
2. **W4 commitment** — is the bespoke SCM a committed destination, or is "git-native indefinitely" (core-git keeps absorbing scale features) an acceptable terminal state? The no-single-leader metadata store is the highest-blast-radius greenfield piece with no precedent in any of the 4 hyperscalers.
3. **ADR-0510 concrete numeric cutover triggers** — still effectively Proposed; without them "cutover-gated, no deadline" can't be falsified.
4. **Masterplan reachability** — RESOLVED 2026-06-09: ADR-0516..ADR-0535 are wired into `masterplan.json` + `master-plan-sequencing.json`.
5. **Rosie analogy bound** — Rosie = bots executing human-defined refactors; *autonomous-AI out-producing humans* is unproven and ours to demonstrate. Accept that gap?

---

## 8. Key paths
- Canon ADRs: `docs/decisions/ADR-0516..0535-*.md`
- SSOT stores: `registry/stores/{design-store,instructions-store,registry-store,canon-id-crosswalk}.json`
- Firewall: `cloud/cloud-ci/gates/` · config `oya-ci.toml` + `libs/oya-ci-config/` · producer `oya-cloud-ci-accounting-registry-app` · workflow `.github/workflows/oya-ci-required.yml`
- **Survives in source `dev` (depend on these):** canon ADRs `docs/decisions/ADR-0516..0535-*.md` (+ amended 0280/0392/0510); SSOT stores `registry/stores/{design-store,instructions-store,registry-store,canon-id-crosswalk}.json`; firewall `cloud/cloud-ci/gates/` + config `oya-ci.toml` + `libs/oya-ci-config/`; producer `oya-cloud-ci-accounting-registry-app`; required-check workflow `.github/workflows/oya-ci-required.yml`; root-hub `specs/root-hub-pointers.json`; entry points `AGENTS.md` / `CLAUDE.md`.
- **Session-local — do NOT depend on (does not survive):** the detailed working plans (`CONSOLIDATION-EXECUTION-MAP`, `CLI-GOVERNANCE-TO-FIREWALL-MIGRATION-PLAN`, `OYA-CI-PRODUCTIZATION-PLAN`, `OYA-CI-PRODUCT-ARCHITECTURE-PLAN`, `MIGRATION-PLAN-RESYNC`, `STORE-SCHEMA`, the deep-interview spec, the canon proposal) and the cross-session memory. Their **essence is inlined in this file (§1–§7)**; full copies are on the transient `consolidate/kernel-snapshot-2026-06-08` branch under `docs/audit/initial-sweep-2026-06-06/` until Task #20 lands. If you need them long-term, migrate them into `dev` before deleting that branch.
