# HANDOFF — Oyatie Canon-Consolidation → Firewall-First Enforced Canon

> **Updated 2026-06-06.** Single resume document. Read this top-to-bottom on a fresh session, then open the **decision-record charter** (§2) and the **Phase-0 firewall plan** (§5) before touching anything. Two source commits have landed this session (STEP-0 base + ADR-0515); everything is on `feat/oya-ci-tide`, pushed to github-mirror, **no dev PR** (firewall-first).

---

## 0. One-paragraph orientation

This effort started as a canon-consolidation audit of `~/Developer/source` (GitHub `jason931225/oyatie`, the company monorepo) and became, by founder ruling, the founding act of an **enforced, automated, fully-accounted, drift-proof canon**. The trigger: we discovered a *second* execution-pending consensus body (the source `.omx/backlog`), and the founder ruled that **the drift between the two is the symptom of faulty process + enforcement** — so the fix is to make enforcement real first, then fix the canon through it (**firewall-first**). A read-only audit then *verified* that enforcement is a **façade** (the required merge context has no live producer; **0 gates block a merge today**). We are now executing the firewall: **STEP-0** (clean base) ✅ and **ADR-0515** (the one canonical CI/CD ADR = the producer spec) ✅ are done; the live `oya-ci-required` producer + the 4 keystone gates are next.

---

## 1. Repo + branch state (verified 2026-06-06)

- **TWO REPOS (do not confuse):** **MUTATE** `~/Developer/source` (the `oyatie` company monorepo — all source changes land here); **READ** `~/Developer/linux/docs/audit/initial-sweep-2026-06-06/` (the *linux* pilot repo, which also hosts this effort's decision-record/plans/registers). Bare paths in §6 are relative to that linux audit dir; `source/...` paths are in the monorepo. A session resumed from the linux repo must `git -C ~/Developer/source` for all mutations.
- **Mutation target:** `~/Developer/source` — branch **`feat/oya-ci-tide`**, HEAD **`869e48ca4`** (verify with `git -C ~/Developer/source log --oneline -1`).
- **Remotes:** `origin` = `http://forgejo.local/...` (**NEVER push here**); remote named **`github-mirror`** = `https://www.github.com/jason931225/oyatie` (**push here only** — note the `www.`). **`git fetch github-mirror` first** each session; local `dev` is stale (behind `github-mirror/dev`).
- **`feat/oya-ci-tide` is ~982 commits ahead of `github-mirror/dev`** (as-of 2026-06-06; the count grows by one per commit — re-verify with `git -C ~/Developer/source rev-list --count github-mirror/dev..HEAD`; do NOT reason off local `dev`). A `--base dev` PR would be the whole migration — **do NOT open it** until the firewall is real (firewall-first). The branch is the working base; Phase-0 builds on it.
- **Signing works (G3):** `commit.gpgsign=true`, `gpg.format=ssh`, key `~/.ssh/id_ed25519.pub`, user `Jason Lee <56489493+jason931225@users.noreply.github.com>`. Commit with `git commit -S`; verify via `git cat-file commit HEAD | grep gpgsig`.
- **`gh` is authed** (jason931225, https) — push + `gh` work. **But applying GitHub branch-protection rulesets needs the founder's GitHub admin** (the FE-1 producer go-live dependency).
- **Audit/work corpus:** `~/Developer/linux/docs/audit/initial-sweep-2026-06-06/` (decision-record, plans, registers — the linux pilot repo).

---

## 2. The governing charter (the SSOT of rulings)

**File:** `linux/docs/audit/initial-sweep-2026-06-06/synthesis/decision-record-oyatie-canon.md` — ~40 founder D-rulings, continuously appended. The ones that govern the current work:

- **D-CICD** — oya-ci/oya-cd = bespoke-Rust reimplementation that **adopts the patterns** of Prow + Tekton + Argo-WF + Argo-CD + Argo-Rollouts ("do what Go does, cloud-native, in Rust"). New canonical ADR = **ADR-0515** (collapses 0349/0359/0361/0511/0513/0514; **0408 stays separate, `depends_on`**).
- **D-PURESPLIT** — exactly two service trees `oya/` (products) + `cloud/` (platform); eradicate services//platforms//microservices//flat-crates/. Amend ADR-0131/0512.
- **D-SCOPE-UNIFY** — the source `.omx/backlog` platform-readiness program is **in scope**; the two consensus bodies merge into ONE canon; cloud services are dogfood products; hyperscaler bar.
- **D-D1-TOPOLOGY** — **co-located consistency domain, federated execution**: ontology = single transactional write-path + effective-dating kernel type + consistency token; workflow mutates domain only via ontology typed-actions in-txn; intelligence federated/stateless; Kafka off the critical path. Acceptance = a payroll-close read-your-writes conformance test. Author a net-new **D1 meta-ADR** (+ EntityMutated proto A2a + workflow→AI gRPC A2b).
- **D-AEC-DECLINE** — decline the agent-execution-controller; record the decline in `source/docs/ideas/agent-execution-controller.md`; ADR-0116/0363 stand.
- **D-DOCTRINE** (governing) — drift = process/enforcement failure; **maintainable BY ENFORCEMENT + automate everything**; **every file/doc/folder accounted-for + justified**; **robust-not-false** (no thin/flaky/advisory-shell enforcement; every gate RED/GREEN-proven); hyperscaler + **Linus-taste**; check **all arch invariants** (BNF-13-enum, clean/hexagonal, parallel lanes, minimal blast radius, data_class).
- **D-DOCORG** — reorg docs onto the **existing** scheme: Diátaxis quadrants (folder topology) + 7-product-axis (`docs/DESIGN.md`) + DOC-CATALOG tier; unify into one machine-readable per-doc record.
- **D-SEQUENCE** — **firewall-first** order (§5).

---

## 3. The verified problem (why firewall-first)

Read-only audit (`justify-account-robustness/00-JUSTIFY-ACCOUNT-ROBUSTNESS.md`), apex finding independently re-verified:

- **Enforcement is a FAÇADE — 0 gates block a merge.** The sole required context `oya-ci-required` has **no live producer**; both `infra/branch-protection/dev.json` and `.github/branch-protection.yaml` self-disclaim it as a P0.0 target; live GitHub `dev` actually requires `[cargo-* + oya-pr-review(HTTP 501)]`. This *is* the mechanism of the drift.
- **False-green exhibits (verified):** ADR-0363:35 says *"The Foundry name was eradicated"* (and still cites the pre-migration `microservices/foundry/` 597-file shell) — but on THIS branch `microservices/` is gone; tracked foundry residue now lives at `source/docs/foundry`, `source/contracts/openapi/foundry`, `source/docs/products/foundry`, `source/docs/runbooks/foundry` + Cedar `oyatie.foundry.*` principals (ADR-0247). Honest count = **~4,746 tracked files mention `foundry`** (`git -C ~/Developer/source grep -il foundry | wc -l`; re-verify — earlier "~4,110/201-crates/3,771" were stale/worktree-inflated). The ADR-0363 text itself is stale and is part of the A-FOUNDRY fix. Duplicate **ADR-0377** (two files). Phantom **ADR-0150** citations (real 0150 = cursor-pagination). ADR-INDEX/decisions.json generated-index drift. ~96 claimed gates / **0 proven-blocking** (per the audit; re-verify against live GitHub).
- **Conclusion:** you cannot fix the canon on fake enforcement — make enforcement real first.

---

## 4. What's DONE this session (source mutations, both signed + pushed)

1. **STEP-0 — clean base** = commit **`e77f16eb2`** (signed). 41 files: 15 canon decisions (the 2026-06-02 pure-split + oya-ci ADR amendments + CLAUDE.md/AGENTS.md/masterplan/sequencing/root-hub-pointers/Cargo.lock) + 2 WIP-authority plans + 24 Phase-0 seeds (incl. 8 Python deletion-tagged for §Q). Pushed to github-mirror. **No dev PR.**
2. **ADR-0515 — the CI/CD canon** = commit **`869e48ca4`** (signed). New ADR-0515 (Accepted) + reciprocal `superseded_by: [ADR-0515]` + body §Status on the 6→1 collapsed ADRs (0124/0349/0359/0361/0511/0513/0514) + 0408 kept separate (`depends_on`, added to `related`) + 0359 drift fixed + 0511↔0513 whiplash closed. Independent grep lane confirmed all edges bidirectional, no status drift. Pushed.

**Deferred by design (do NOT silently "finish" these — they're sequenced):**
- `registry/quality/lanes.yaml` **buck2-native lane migration** — excluded from STEP-0 (its `buck2 build //libs/oya-check-*` targets don't build yet = false-green). Snapshot preserved at branch **`wip/pre-step0-snapshot-4faff03ca`** (the dangling accidental "verify signing" commit). Re-apply in Phase-0 with the targets built + RED/GREEN.
- **All gitignore / `git rm --cached`** of agent-state (`.claude/.codex/.gemini/.omc/.claire`) — deferred to the Phase-0 **total-accounting gate** (these trees mix tracked *content*, e.g. `.omc/state/*.md` wave findings + design docs, with ephemeral churn; hand-classification = re-drift). ~40 dirty agent-state entries remain in the worktree by design.
- Stray **`source/docs/audit/initial-sweep-2026-06-06/docs-sweep/20-products-localization.md`** — one misplaced file (an audit agent wrote to `source/` instead of `linux/`); untracked, excluded; verify-it's-a-dup then remove.

---

## 5. What's NEXT — the firewall-first sequence (D-SEQUENCE)

**Plan:** `linux/docs/audit/initial-sweep-2026-06-06/PHASE-0-FIREWALL-PLAN.md` (+ the 4 `_phase0/10-*.md` lanes; `_phase0/10-ci-adr-spec.md` is the ADR-0515 authoring spec). **Spine: producer → accounting-registry → 4 gates → (unlocks Phase-1).** Each step is door:one-way + founder sign-off.

- **Phase-0 (false-green firewall):**
  1. ✅ **ADR-0515** authored + ratified (done).
  2. ⏭ **`oya-ci-required` producer (the next step).** The controller is **scaffolded** at `source/oya/ci-controller/crates/` — 4 crates exist (hexagonal `-kernel`/`-forgejo-adapter`/`-k8s-adapter`/`-app`; `-kernel` has `GATE_CONTEXT="oya-ci-required"` at `lib.rs:471` and a `ForgejoStatusPoster` trait at `lib.rs:620`) but it's **Forgejo-only and the GitHub poster is missing** (verify build/test state before relying on it). Forge-of-record is **GitHub**. Work: build a **`GitHubCommitStatusPoster`** behind the existing `ForgejoStatusPoster`-style seam + fix tide context (default `oya-ci-gate`, `oya-ci-tide-kernel/src/lib.rs:76`; override env `OYA_TIDE_REQUIRED_STATUS_CONTEXT` → `oya-ci-required`) + deploy + **flip the live GitHub `dev` ruleset to `["oya-ci-required"]`** + **prove it blocks** a known-bad PR. RED/GREEN fixtures: `source/specs/fixtures/phase0-ci-enforcement-baseline/` (GREEN `tc-0.0-good-...` + 10 RED `tc-0.0.1*/0.0.2/0.0.3`); contract `source/specs/phase0-ci-enforcement-baseline.json`; output schema `source/specs/phase0-ci-enforcement-result-schema.json`.
     - **Autonomy boundary:** a session may build the adapter + wiring and get the **local RED/GREEN fixtures green autonomously**. It MUST **HALT before any live GitHub ruleset change** — the flip + "prove it blocks on a real PR" is a **founder-paired step (needs founder GitHub-admin credential)**. If `oya-ci-required` is found already required live mid-build → halt (`CP-AUTH-FLIP/G0`).
  3. **Accounting-registry + 4 keystone gates** (`accounting-registry.generated.json` + cross-artifact-agreement / total-accounting / staleness-reaper / automation-ratchet), buck2-native, each RED/GREEN reproducing its live exhibit. Gate-4 exists as a seed (`specs/phase0-automation-matrix.json`); 1/2/3 are [BUILD]. This is where the deferred gitignore/total-accounting + lanes.yaml buck2-native get done **systematically**.
- **EXIT (Phase-0 done → Phase-1 unlocks):** ADR-0515 Accepted ✅ · producer posts live + blocks a known-bad PR (proven) · registry generates+validates (`committed==regenerated`) · 4 gates wired + RED/GREEN-proven · no CP-* checkpoint open.
- **Phase-1 (amendments, each GATE-VERIFIED):** A-CI (finish the 6→1: resolve `byp_adr_0349`, delete Jenkins gate path at cutover, relocate the product-spec into `source/`) · A-FOUNDRY (fix ADR-0363's false "eradicated" + complete foundry eradication) · A-INTEGRITY (dup-0377 renumber, phantom-0150, status-enum, regenerate indexes) · A-STRUCT (pure-split eradication + amend 0131/0512 homes) · A-TASTE · A-IDENTITY.
- **Phase-2:** doc-reorg (44→6 Diátaxis homes) + OWNERS/reachability closure + net-new ADRs (D1 meta-ADR + effective-dating kernel + the sequenced platform-readiness program).

**Locked rollback trigger:** `CP-BUCK2-LINUX` — `buck2 build //… && buck2 test //…` must be green on the **Linux** gate runner (only darwin proven today); if not → **hard halt, backlog, do not iterate**. Other hard halts: `CP-AUTH-FLIP/G0` (oya-ci-required flips live mid-Phase-0), `CP-PRODUCER-RED`, `CP-GATE-SELFTEST-FAIL`.

---

## 6. Key artifacts (paths)

- Charter / rulings SSOT: `synthesis/decision-record-oyatie-canon.md`
- Firewall plan: `PHASE-0-FIREWALL-PLAN.md` + `_phase0/10-{ci-adr-spec,producer-plan,gates-registry,lane-spine}.md`
- ADR-0515 (committed to source): `source/docs/adr-archive/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md` (draft copy also at `_phase0/ADR-0515-DRAFT-*.md`)
- Audit: `justify-account-robustness/00-JUSTIFY-ACCOUNT-ROBUSTNESS.md` (+ `10-*.md` lanes)
- Reconciliation: `backlog-reconciliation/00-BACKLOG-RECONCILIATION.md`
- The two consensus bodies: linux decision-record (above) + `source/.omx/backlog/platform-readiness-backlog.md` (pillars A-Q + register 1-21)
- WIP execution authority (committed): `source/.omc/plans/monorepo-consolidation-migration.md`
- Prior registers: `synthesis/01-ADR-DISPOSITION-TABLE.md`, `02-DECISION-ATOM-LEDGER`, `03-PROPOSED-RESOLUTION-LEDGER`, `04-DOMAIN-TAXONOMY`; `docs-sweep/00-REST-OF-DOCS-REGISTER` (CC-1..13); `monorepo-conformance/00-CONFORMANCE-REGISTER`
- Memory: `~/.claude/projects/-Users-jasonlee-Developer-linux/memory/oyatie-consolidation-execution-state.md` (+ MEMORY.md index)

---

## 7. Hard-won lessons (don't repeat)

- **Verify your own tooling + the audit's numbers.** The audit's "201 foundry crates / 3,771 files" was worktree-inflated; the honest figure is far smaller. Always re-verify a load-bearing count against the live tree before acting on it.
- **The index can carry index-only WIP.** STEP-0's `git reset` would have silently dropped `lanes.yaml`'s buck2-native change (it was staged-only, worktree==HEAD) and the two `AD` seeds (staged-added, worktree-deleted). Always `git show :path`/`checkout-index` to recover before reset; preserve dangling commits with a branch ref.
- **`--base dev` ≠ STEP-0.** `feat/oya-ci-tide` is 981 ahead of dev; a dev PR is the mega-migration. Push the branch; defer the dev-merge until gate-verified.
- **Don't hand-classify the agent-state mess.** `.omc/.claude` mix tracked content with churn — that's the total-accounting gate's job, not an ad-hoc gitignore.
- **No false enforcement in the canon.** `lanes.yaml` buck2-native references non-building targets = false-green; it waits for Phase-0. ADR-0363's "eradicated" claim is the cautionary example.
- **Separate authoring vs verification lanes.** ADR-0515's reciprocal edges were written by an executor agent, then verified by an independent grep lane before commit.
- **`assert-talos-boot.sh` needs an arch arg.** `git commit -S` with staged files can sweep them into the commit (the prior "verify signing" incident → reset --soft → left 5 files staged). Check the index before committing.

---

## 8. Resume checklist

1. Read this HANDOFF → the charter (§2) → the firewall plan (§5).
2. Confirm branch/HEAD: `source` on `feat/oya-ci-tide` @ `869e48ca4`; signing works; push github-mirror only.
3. If proceeding with the **producer**: draft the `GitHubCommitStatusPoster` + wiring, RED/GREEN-test locally; the **live ruleset flip + "prove it blocks" needs the founder's GitHub admin** — that step is collaborative.
4. Every source mutation is **door:one-way → founder sign-off**; commit signed; push github-mirror; **no dev PR** until Phase-0 EXIT.
5. If anything contradicts a ruling, surface it before amending (consensus-first).
6. Keep the memory file + this HANDOFF current as state changes.
