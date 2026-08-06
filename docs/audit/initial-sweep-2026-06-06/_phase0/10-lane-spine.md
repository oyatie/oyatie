---
title: "PHASE-0 EXECUTION SPINE — FIREWALL-FIRST staged parallel lanes (D-SEQUENCE)"
lane: _phase0 / 10-lane-spine
charter: D-SEQUENCE (firewall-first) + D-CICD (oya-ci pattern-adoption) + D-DOCTRINE (maintainable-by-enforcement) + ADR-0365 (automated lifecycle) + WIP monorepo-consolidation-migration.md (execution framework)
date: 2026-06-06
mode: READ-ONLY assembly. No source file edited. This is the only artifact written.
status: PLAN (what to BUILD). Distinguishes WHAT EXISTS (live-read) from WHAT TO BUILD throughout.
scope_repos:
  - /Users/jasonlee/Developer/source  (the live source monorepo — STEP-0 + Phase-0 mutation target)
  - /Users/jasonlee/Developer/linux/docs/audit/initial-sweep-2026-06-06  (the decision/plan authority corpus)
verdict: |
  Phase-0 makes the FAÇADE real before any Phase-1 amendment. The dependency spine is
  producer → accounting-registry → 4 keystone gates → (unlocks Phase-1 amendments). Each gate
  is built UNDER advisory enforcement and proves ITSELF (RED/GREEN self-test reproducing live
  drift exhibits) before it may flip to blocking. door:one-way + founder sign-off gates: STEP-0
  base-commit, the canonical CI/CD ADR, FE-1 producer go-live, and every source mutation. The
  LOCKED rollback trigger is: buck2 build/test //... NOT green on the gate runner ⇒ HARD
  checkpoint, backlog, do not proceed.
---

# 10 — PHASE-0 EXECUTION SPINE (FIREWALL-FIRST)

> **Reading order:** §A STEP-0 (clean base) → §B dependency spine → §C per-lane breakdown + bootstrapping discipline → §D verification gates + door:one-way sign-off points → §E rollback/checkpoint + ralph-loop. Every load-bearing claim cites a real path+line, read live 2026-06-06. WHAT EXISTS = read from disk; WHAT TO BUILD = the lane work.

---

## §0 — GROUND TRUTH (live-read 2026-06-06; the firewall premise)

**The façade, confirmed live (not from a doc — read from the source tree):**

- **WHAT EXISTS — the producer KERNEL only, not a running producer.** `oya/ci-controller/crates/oya-ci-controller-kernel/src/lib.rs:471` declares `pub const GATE_CONTEXT: &str = "oya-ci-required";`. A repo-wide grep for `ci-controller`/`oya-ci-controller` across `.github/`, `Jenkinsfile`, `infra/ci/` returns **ONLY `.github/branch-protection.yaml`** — i.e. the controller is referenced as a *target context name*, never invoked by any executing CI. **No live producer posts `oya-ci-required` on a candidate SHA.**
- **WHAT EXISTS — both branch-protection files self-disclaim.** `infra/branch-protection/dev.json:2` (verbatim): *"…this file is not Phase-0 exit authority until a trusted cloud-ci/oya-ci producer is live and applied."* `.github/branch-protection.yaml:2-5` (verbatim): *"…live GitHub dev protection does not reflect this file exactly… not Phase-0 exit authority until a trusted cloud-ci/oya-ci required context is live."* The single required context is `[oya-ci-required]` (`dev.json:9-11`, `branch-protection.yaml:55-56`).
- **Net (audit verdict, `justify-account-robustness/10-robustness-enforcement.md:221-224`):** ~96 lanes claimed-as-enforcement / 91 registered-active / 109 aggregated / **0 proven-blocking-in-executing-CI**. The producer/required-context intersection is **empty** (FE-3). This is the mechanism of the drift D-DOCTRINE names.
- **WHAT EXISTS — live source state (the STEP-0 surface):** `git -C /Users/jasonlee/Developer/source` → branch `feat/oya-ci-tide`; remotes `origin=http://forgejo.local/oya-admin/oyatie.git` + `github-mirror=https://www.github.com/jason931225/oyatie`; **79 dirty** entries; the WIP plan `.omc/plans/monorepo-consolidation-migration.md` is **untracked (`??`)**.

**Firewall logic (D-SEQUENCE `decision-record-oyatie-canon.md:196`):** *"you cannot fix the canon on fake enforcement; make enforcement REAL first, then fix the canon THROUGH it."* Phase-0 stands up the real producer + the 4 keystone gates so Phase-1 amendments are gate-verified and cannot re-drift.

---

## §A — STEP-0: COMMIT-WIP-TO-CLEAN-BASE (precondition to ALL mutation)

**Authority:** D-SEQUENCE `:197` ("Step-0: commit the WIP to a clean base"). **door:one-way + founder sign-off** (this is the base every later commit descends from; once Phase-0 commits land on it, the choice of what entered the base is irreversible in practice).

**WHAT EXISTS (the 79 dirty, triaged by `git status --porcelain`):**

| Class | Examples (live) | Disposition |
|---|---|---|
| **Decision/spec source (canon)** | `docs/adr-archive/ADR-0363-retire-agentic-vcs-platform-to-intelligence-on-github-substrate.md (M), `specs/masterplan.json` (MM), `specs/master-plan-sequencing.json` (MM), `registry/quality/lanes.yaml` (MM) | **STAGE as base canon** — this is the corpus Phase-0 gates will police. Verify each diff is intended (separate verifier pass) before committing. |
| **WIP execution authority** | `.omc/plans/monorepo-consolidation-migration.md` (??), `.omc/plans/open-questions.md` (??) | **COMMIT to the base** — D-MERGE made this the EXECUTION AUTHORITY (`decision-record:47`). Founder folds the UNIFIED-EXECUTION-PLAN into it on commit; until then it is untracked and at risk. This is the single highest-value untracked file. |
| **Phase-0 net-new specs/tests already on disk** | `specs/cloud-*.json` (8 ??), `scripts/tests/cloud_*_check.py` (7 ??), `docs/ideas/buck2-native-ci-gate.md` (??), `docs/ideas/agent-execution-controller.md` (??), `infra/external-secrets/` (??) | **STAGE selectively** — these are Phase-0/CI seed artifacts; commit the ones that are decision-grounded, leave runner junk. |
| **Tooling/harness churn** | `.claude/`, `.codex/`, `.gemini/`, `.omc/state/*`, `.omc/sessions/*`, `.claire/`, `evidence/multispectrum/*` | **GITIGNORE or leave** — agent/session state, not canon. Do NOT commit session jsonl / checkpoints to VCS (the `git ls-files` accounting discipline, `enforcement-primitives:79`). |
| **Renames/deletes** | `specs/language-discipline-registry.json` (AD), `specs/repo-hygiene-automation.json` (AD), `.omc/state/*.jsonl` (D) | **Resolve intent** — AD = added-then-deleted; confirm whether the spec is being relocated or dropped before committing. |

**STEP-0 procedure (the ordered actions):**
1. **Stay on `feat/oya-ci-tide`** (the live branch; do NOT create a new base branch — the WIP loop rebases lanes on `github-mirror/dev`, and `feat/oya-ci-tide` is the in-flight authority-migration branch). Phase-0 work branches OFF this as `phase0/<gate>` feature branches (one-lane-one-squash-PR, §C).
2. **Triage the 79** per the table (separate verifier pass — founder rule "verify at each step"; no blind `git add -A`).
3. **Provision signing FIRST** (G3, already DONE per `decision-record:47` "signing now provisioned — SSH-signed verified"; re-confirm `commit.gpgsign`/`user.signingkey`/`gpg.format`/`tag.gpgsign` are non-empty — they were EMPTY at WIP authoring time, WIP §11 G3).
4. **Commit the canon + WIP authority** as a SIGNED commit, linear history, squash-shaped. Body carries the 5-H2 PR template (`prelane-0.7/00-GOVERNANCE-BOOTSTRAP.md §1`) + DOC-CATALOG/CHANGELOG rows where the gate reads them (`docs/DOC-CATALOG.md`, not `docs/CATALOG.md` — known path bug, prelane-0.7 §0).
5. **Push `github-mirror` ONLY** — never `origin`/Forgejo (FORGE-EXPLICIT principle, WIP §1.8). `gh pr create --repo jason931225/oyatie --base dev`.
6. **door:one-way founder sign-off** on the base commit before any gate lane opens.

**Why STEP-0 is door:one-way:** it fixes what is "in the canon" at the moment the firewall is built. The gates born in Phase-0 will police *this base*; anything left dirty/untracked after STEP-0 is invisible to total-accounting and re-introduces drift.

---

## §B — THE DEPENDENCY SPINE (producer → registry → gates → unlocks Phase-1)

**Locked order (D-SEQUENCE `:201`):** `producer → accounting-registry → gates → amendments → reorg`.

```
STEP-0 base commit (signed, founder-signed-off)
        │  door:one-way
        ▼
[FE-CICD-ADR]  the ONE canonical CI/CD ADR (D-CICD producer spec)        ◄── door:one-way + founder sign-off
        │       WHAT TO BUILD: consolidate ADR-0349/0359/0361/0408/0511/0513/0514 → ONE ADR (dest 0513)
        │       via the ADR-0365 lifecycle (research+consensus → generative ADR → oya gen propagate)
        ▼
[FE-1 PRODUCER]  stand up + PROVE the oya-ci-required producer            ◄── door:one-way + founder sign-off (GO-LIVE)
        │       WHAT EXISTS: oya-ci-controller-kernel (lib.rs:471) — kernel logic only
        │       WHAT TO BUILD: wire the controller into an executing presubmit so it POSTS
        │                      oya-ci-required on a candidate SHA; capture a real check_run;
        │                      apply the ruleset; snapshot live GitHub required_status_checks
        ▼
[REGISTRY]  accounting-registry.generated.json   ◄── the ONE good data structure (Gate-2 owns it)
        │       WHAT TO BUILD: a buck2-native Rust producer packet (rust_binary), NOT an oya CLI cmd (#20)
        │       generated from git ls-files × OWNERS × ADR-front-matter × masterplan reachability
        ▼
[4 KEYSTONE GATES]  (predicates over the registry + their own face-source; G-INTEGRITY track)
        ├─ GATE-2 cloud-ci-total-accounting   (producer of record — rows complete?)
        ├─ GATE-1 cloud-ci-cross-artifact-agreement   (decision faces agree?)
        ├─ GATE-3 cloud-ci-staleness-reaper   (TTL over budget?)
        └─ GATE-4 cloud-ci-automation-ratchet (claims honest + monotonic? polices Gates 1-3)
        │       each: RED/GREEN fixtures it actually BLOCKS; committed==regenerated; required-context-not-advisory
        ▼
[FIREWALL LIVE]  oya-ci-required green ⇔ all 4 gates green on the candidate SHA
        │  door:one-way (firewall declared real only after each gate's self-test reproduces its live exhibits as RED)
        ▼
UNLOCKS PHASE-1  (A-CI / A-STRUCT / A-FOUNDRY / A-INTEGRITY / A-TASTE / A-IDENTITY amendments,
                  each now GATE-VERIFIED and unable to re-drift)
```

**Spine invariants (all live-grounded):**
- **Gate-2 is the producer of record; Gates 1/3/4 are views over its registry** (`enforcement-primitives:198`) — Linus "one good data structure kills the special cases." No four parallel scanners.
- **All four are G-INTEGRITY track: specs+filesystem, NO buck2-build-graph dependency** (`enforcement-primitives:199`), so the firewall ships Phase-0 BEFORE the migration build work — "the false-green firewall must not wait."
- **Producer is always a buck2 Rust gate crate, never a new `oya` CLI command** (register #20; `enforcement-primitives:29`). This directly retires the present defect where ADR-0365's own gate is `verified_by: oya gen propagate --check` (`ADR-0365:26`) — Gate-4's self-test must flag that as `oya-cli-authority`.

---

## §C — PER-LANE BREAKDOWN (sequential / parallel) + BOOTSTRAPPING DISCIPLINE

**Staging model:** the spine is sequential at the boundaries (STEP-0 → ADR → producer → registry are a hard chain), then the **4 gate lanes run in a bounded parallel fan-out** once the registry exists, because each is a predicate over the shared registry (no inter-gate code dependency except Gate-4 polices 1-3, which is a *test-time* dependency, not a build dependency).

### The bootstrapping paradox + its discipline (the heart of Phase-0)

Phase-0 artifacts are built UNDER advisory enforcement (the firewall they will become does not exist yet). The discipline that prevents a fake-green Phase-0:

1. **Honest status start.** Each gate's status in `specs/phase0-automation-matrix.json` starts at the on-disk `seed-contract-not-green` (the matrix already exists with that discipline, `enforcement-primitives:31,200`). It is classified `automated_advisory_until_p0_0` — never overstated.
2. **Born-already-blocking self-test.** A gate may flip to `automated_blocking_now` ONLY after its own RED/GREEN self-test reproduces the **live drift exhibits** as RED on the *current* corpus (`enforcement-primitives:70,200`). A gate that cannot demonstrably block real drift is itself a claim-ceiling #21 violation and must not be marked enforced.
3. **The producer is proven by a known-bad PR.** FE-1 go-live is proven not by assertion but by a deliberate RED PR: a candidate SHA with a known-bad input (e.g. an `axes_count:6 vs 7` drift, or a tracked file with no accounting row) that the producer MUST post `oya-ci-required = failure` against, captured as a real check_run. Then a known-good PR it passes. (D-SEQUENCE robustness bar: every gate proven by RED + GREEN + proof it runs in CI and BLOCKS.)
4. **Gate-4 polices the others.** The automation-ratchet's self-test must flag Gates 1-3 if any claims "enforced" without its self-test reproducing its live exhibits (`enforcement-primitives:200,214`). The ratchet polices itself.

### LANE TABLE (Phase-0; each is one signed squash-PR onto the STEP-0 base)

| Lane | Seq/Par | WHAT EXISTS (live) | WHAT TO BUILD | RED self-test exhibit (must block) | door:one-way? |
|---|---|---|---|---|---|
| **P0.STEP-0** base commit | SEQ #1 | 79 dirty + untracked WIP plan | triage + signed commit of canon+WIP authority | n/a (base) | **YES — founder sign-off** |
| **P0.ADR** canonical CI/CD ADR | SEQ #2 | ADR-0513 (M) + 0349/0359/0361/0408/0511/0514 cluster | consolidate → ONE ADR (dest 0513) via ADR-0365 lifecycle; supersede/relate w/ reciprocal edges | n/a (authored doc; Gate-1 later proves its supersession graph acyclic) | **YES — founder sign-off (the canonical CI/CD ADR)** |
| **P0.PRODUCER** FE-1 go-live | SEQ #3 | `oya-ci-controller-kernel/src/lib.rs:471` GATE_CONTEXT | wire controller into executing presubmit; post `oya-ci-required` on candidate SHA; apply ruleset; snapshot live required_status_checks | a known-bad candidate SHA posts `oya-ci-required=failure` (captured check_run) | **YES — founder sign-off (producer go-live) + credentials (GitHub apply)** |
| **P0.REGISTRY** accounting registry | SEQ #4 | `git ls-files` + OWNERS + ADR front-matter (sources exist) | buck2 Rust producer emits `accounting-registry.generated.json` (path→owner+justify+reach+ttl) | n/a (registry; Gate-2 proves it) | no (mutation of source ⇒ standard per-PR sign-off) |
| **P0.GATE-2** total-accounting | PAR (after registry) | `oya-governance-orphan-detection-kernel` in `libs/` (reuse); 780 `oya-foundry-*` residue; 57 unwired `oya-governance-*` | predicate: unaccounted·unowned·unjustified·unreachable·no-ttl·registry-drift; 7 fixtures | flags the live 780 `oya-foundry-*` as unjustified (vs ADR-0363's false "eradicated") + 57 `oya-governance-*` as unreachable | per-PR sign-off |
| **P0.GATE-1** cross-artifact-agreement | PAR | catalog.json:12 `axes_count:6`; dup-0377; 0511↔0513 half-edge | predicate: orphan·unpropagated·status-disagreement·face-drift·dual-collision·half-edge; 7 fixtures | reproduces RED: `axes_count:6 vs 7`, dup-0377, 0511↔0513 half-supersession on current corpus | per-PR sign-off |
| **P0.GATE-3** staleness-reaper | PAR | Task-#14 >48h class; ai-slop `_partial`/`_verify` scratch docs | predicate over registry + generated `ttl-policy` per resource-class + `git log` last-touch; report-then-git-mv-to-`_archive/` NEVER rm; 7 fixtures | flags an over-budget-AND-unreachable stale doc as archive candidate (age alone ≠ stale; protected classes never reaped) | per-PR sign-off |
| **P0.GATE-4** automation-ratchet | PAR (polices 1-3) | `specs/phase0-automation-matrix.json` + 4 live RED/GREEN fixtures on disk (reuse) | predicate: enforceable-as-human-judgment·advisory-claiming-enforced·oya-cli-authority·incomplete-exception·no-retirement·ratchet-regression; reuse 4 + 2 net-new | flags 57 `oya-governance-*`, `diataxis-doc-class`, `prd-axis-coverage`, ADR-0365's `oya gen` `verified_by` as advisory/oya-cli-authority | per-PR sign-off |

**Per-lane shape (every Phase-0 PR obeys the WIP loop, `monorepo-consolidation-migration.md` STEP 0-14):** step-0 re-diff live protection (G0 HALT on authority flip) → rebase on `github-mirror/dev` → build → Cargo+Buck2 DUAL build + `buck2 //...-check` matrix locally → multispectrum evidence + 5-H2 PR body + DOC-CATALOG/CHANGELOG rows → push `github-mirror` + signed commits → drive `github-lane-unlocker-required` green → **squash-merge** → rebase + re-run authority gate.

> **Note — TWO live gates during Phase-0 (BUILD-TO-BOTH, WIP R1/§1):** the live required context is still `github-lane-unlocker-required` (Buck2-whole-graph, signatures off); the *target* is `oya-ci-required`+signing (ADR-0513, in flight). Phase-0 IS the work that makes `oya-ci-required` real, so Phase-0 lanes build to the live unlocker AND are the producer of the target. The **authority-flip is G0-HALT** (WIP top risk): if `oya-ci-required` flips live mid-Phase-0, HALT for founder.

---

## §D — VERIFICATION GATES + DOOR:ONE-WAY / FOUNDER-SIGNOFF / CREDENTIAL POINTS

**The robustness bar (founder, D-SEQUENCE):** no gate is "live" without a **RED fixture (known-bad it MUST fail) + GREEN fixture (known-good it passes) + proof it runs in CI and BLOCKS.** Verification is layered (founder verify-at-each-step): a separate verifier lane vs the real source, no phantom findings, no self-approval.

**door:one-way + FOUNDER SIGN-OFF points (the irreversible gates — D-SEQUENCE `:201`):**

| # | Gate | Type | Why irreversible / what is verified |
|---|---|---|---|
| 1 | **STEP-0 base commit** | door:one-way + sign-off | fixes what is "in the canon" the firewall will police; everything left dirty after this is invisible to total-accounting |
| 2 | **The canonical CI/CD ADR** | door:one-way + sign-off | D-CICD producer spec; consolidates 7 ADRs into one; supersession is irreversible (build-first-cutover-later — Jenkins/Argo stay operative until cutover, `decision-record:161`) |
| 3 | **FE-1 producer go-live** | door:one-way + sign-off **+ CREDENTIALS** | turning on real enforcement; requires GitHub credentials to apply the ruleset + snapshot live `required_status_checks` (G1 push creds, WIP §11) |
| 4 | **Each source mutation** | per-PR sign-off | "every source mutation" is door:one-way (`decision-record:201`); ADR-0365 `decision-door` gate: `door: one-way` ADRs CANNOT auto-merge — founder sign-off required (`ADR-0365:36-37,78-80`) |
| 5 | **Firewall declared real** | door:one-way + sign-off | only after all 4 gates' self-tests reproduce their live exhibits as RED and the producer blocks a known-bad PR; this is the Phase-0 EXIT gate that unlocks Phase-1 |

**CREDENTIAL gates (WIP §11, founder-held):** G1 GitHub push credentials for `github-mirror` (origin=Forgejo, never push); G3 signing key (DONE — re-confirm non-empty). FE-1 additionally needs the GitHub admin credential to APPLY the branch-protection ruleset and snapshot the live state.

**ADR-0365 live regimen each Phase-0 ADR/spec change obeys (WHAT EXISTS, `ADR-0365`):**
- `adr-provenance` gate (`:32-33`): a planning_impact ADR must cite best-practice-research evidence + a consensus record; no ADR bypasses the pipeline.
- `decision-door` gate (`:36-37`): `door: two-way` auto-merge on green; `door: one-way` require founder sign-off.
- `oya gen propagate <ADR>` (`:25-26`) regenerates masterplan + affected_surfaces, idempotent — **but** the producer must move OFF the `oya` CLI per register #20 (Gate-4 polices this; the propagation *discipline* stays, the CLI *authority* goes).
- one-doc-per-PR · squash-only · signed commits + linear history · github-mirror NEVER origin/Forgejo.

---

## §E — ROLLBACK / CHECKPOINT + RALPH-LOOP STRUCTURE

### The LOCKED rollback trigger (buck2-builds-first-party stall = HARD checkpoint)

**Source (WHAT EXISTS — `docs/ideas/buck2-native-ci-gate.md`, untracked in source):** the gate's MVP **P0 (gating risk)**, verbatim: *"prove `buck2 build //... && buck2 test //...` green on Linux in-cluster. Fix any Linux fixup gaps (clang/triple). **NOTHING else proceeds until this is green.**"* The #1 risk: the native fixups (psm/blake3/ring/aws-lc/openssl) hardcode `/usr/bin/clang` + `*-apple-darwin` triples; only darwin is verified green, **Linux-correctness is unproven**.

**The trigger, locked:** if `buck2 build //... && buck2 test //...` does **NOT** go green on the first-party graph on the gate runner (Linux in-cluster), Phase-0 **HALTS at a HARD checkpoint** and the work **backlogs** — the buck2-native gate body cannot ship, so the producer (FE-1) cannot run its gates on the real build graph, so the firewall cannot become real on the build axis. This is distinct from (and harder than) the G-INTEGRITY gates, which have NO buck2-build-graph dependency and can ship regardless (§B) — but the FULL firewall (build-correctness as a required context) is gated on this.

**Why HARD (not iterate):** unlike a cargo-side red (clean per-lane revert, WIP §9), a buck2 first-party-build stall is an infra-correctness blocker, not a code defect — iterating in the ralph loop would burn the serial wall-clock against an unproven toolchain. Backlog it, surface to founder, do not proceed.

**Rollback mechanics (WHAT EXISTS — WIP §9 `:179`):** squash-merge keeps reverts atomic + history linear; a clean per-lane revert is real ONLY for cargo-side (non-blocking) failures — a merged lane can redden the next lane's GLOBAL Buck2 graph, so after ANY revert re-run the whole-graph `-check` matrix on rebased dev before resuming. Auto-archive (Gate-2/3) is `git-mv` to `_archive/`, **never `rm`**, gated by a second verifier (founder rule: never delete on an unverified verdict).

### Checkpoint set (HARD HALT for founder):
- **CP-AUTH-FLIP (G0):** `oya-ci-required` flips live mid-Phase-0 — every lane's Done-Definition + signing assumption shifts; HALT, founder decides pivot (WIP top risk, §3.1).
- **CP-BUCK2-LINUX:** the locked rollback trigger above — buck2 first-party build/test not green on Linux runner.
- **CP-PRODUCER-RED:** FE-1 producer cannot post a real check_run / ruleset apply fails on credentials — firewall cannot go live; backlog until credentials/producer fixed.
- **CP-GATE-SELFTEST-FAIL:** a gate's self-test does NOT reproduce its live exhibit as RED — the gate is fake-green; it MUST NOT flip to blocking (claim-ceiling #21).

### RALPH-LOOP STRUCTURE (Phase-0 specialization of the WIP serial loop):

```
PRECONDITION (once):
  STEP-0 base committed + founder-signed-off
  && signing provisioned (G3 re-confirmed non-empty)
  && live protection baseline snapshot recorded (0.4-style; for G0 drift-detect)
  && founder sign-off on the canonical CI/CD ADR (door:one-way)

PHASE0_QUEUE = [ ADR(consolidate→0513), PRODUCER(FE-1 go-live),
                REGISTRY(accounting-registry.generated.json),
                GATE-2(total-accounting), GATE-1(cross-artifact),
                GATE-3(staleness-reaper), GATE-4(automation-ratchet) ]
  # ADR→PRODUCER→REGISTRY are SEQUENTIAL (hard chain); the 4 GATES fan out after REGISTRY

while PHASE0_QUEUE not empty:
  lane = PHASE0_QUEUE.pop_front()        # strict serial driver; one graph mutation at a time
  STEP 0: re-diff live protection vs baseline -> if flipped to oya-ci-required: HALT (CP-AUTH-FLIP/G0)
  STEP 1: rebase phase0/<lane> on github-mirror/dev
  STEP 2..7: build the lane artifact (Rust producer / gate crate / ADR);
             Cargo+Buck2 DUAL build; if buck2 first-party build/test RED on Linux: HALT (CP-BUCK2-LINUX)
  STEP 8: SELF-TEST — run the gate's RED/GREEN fixtures; the RED fixture MUST reproduce the
          live drift exhibit as a block; if not: HALT (CP-GATE-SELFTEST-FAIL); do NOT mark enforced
  STEP 9: prove-by-known-bad-PR — for PRODUCER lane, post oya-ci-required=failure on a known-bad SHA
          (captured check_run) + pass a known-good SHA
  STEP 10: multispectrum evidence + 5-H2 PR body + DOC-CATALOG/CHANGELOG rows
  STEP 11: push github-mirror; gh pr create --base dev; signed commits + linear history
  STEP 12: drive github-lane-unlocker-required GREEN + resolve conversations
  STEP 13: door:one-way founder sign-off where required (PRODUCER go-live; else per-PR sign-off)
  STEP 14: SQUASH-merge -> rebase-on-dev + re-run authority gate (keep next lane honest)
  STEP 15: flip the lane's status in phase0-automation-matrix.json
           seed-contract-not-green -> automated_blocking_now (ONLY after STEP-8 self-test passed)

TERMINATION: all 4 gates automated_blocking_now (self-tests green) && producer proven by known-bad PR
             && oya-ci-required is a REQUIRED context posting on candidate SHAs
             => FIREWALL REAL => door:one-way founder sign-off => UNLOCKS PHASE-1 => /cancel
```

**The boulder never stops:** within a lane, a cargo-side red iterates (do not open PR until green, WIP STEP-8). The HARD checkpoints (CP-*) are the only legitimate HALTs — they stop the loop and backlog to founder, they do not iterate.

---

## §F — COVERAGE / WHAT I DID NOT COVER (no silent caps)

- **Read live + cited:** D-SEQUENCE/D-CICD/D-DOCTRINE (`decision-record-oyatie-canon.md:168-201`), the full robustness register (`10-robustness-enforcement.md`, all 10 FE findings), the 4-gate design (`10-enforcement-primitives.md:26-219`), the WIP plan (`monorepo-consolidation-migration.md` §0-11, full loop), ADR-0365 (`:13-112`), the live source state (branch/remotes/79-dirty/untracked WIP), both branch-protection façade files (live), the producer kernel (`oya-ci-controller-kernel/src/lib.rs:471` + grep proving no CI wiring), the buck2-native gate idea (rollback trigger), prelane-0.7 governance bootstrap (5-H2 + real-vs-aspirational enforcer table).
- **WHAT I DID NOT do (DESIGN, not build):** I did not author the Rust producer crates, the `accounting-registry.generated.json` schema, the `ttl-policy` data, or the fixture JSON files — these are the Phase-0 BUILD work, not this spine. I did not re-derive the 346-ADR / ~90-spec counts (reused `20-verify-register-coverage.md` 172/131/16 and `20-verify-foundry-hygiene.md` 4,714/780 prior-verified figures). I did not query the live GitHub API for the *actually-applied* ruleset (read-only; FE-1's whole point is that the checked-in config is a disclaimed target). I did not enumerate every one of the 79 dirty entries individually — I triaged by class (the full list is in `git status --porcelain`); the STEP-0 verifier pass must walk each one. I did not read every Phase-1 A-lane spec (out of Phase-0 scope; the spine only needs the *unlocks-Phase-1* boundary).
- **Honest figure carried forward (D-SEQUENCE `:201`):** foundry residue = `microservices/foundry/` 597-file shell + ~4110 mentions + Cedar `oyatie.foundry.*` (NOT 201 un-renamed crates — that raw count was worktree-inflated). Gate-2's self-test target is the 780 `oya-foundry-*` files / 4,714 residue (census-of-record, `20-verify-foundry-hygiene.md`).

*End PHASE-0 LANE SPINE. Authority: D-SEQUENCE (order) · D-CICD (oya-ci pattern) · D-DOCTRINE (enforcement) · ADR-0365 (lifecycle) · WIP monorepo-consolidation-migration.md (framework) · 10-enforcement-primitives.md (gate designs). READ-ONLY assembly; no source mutated.*
