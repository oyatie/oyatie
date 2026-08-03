# Implementation Plan: Reorg completion + Open-PR drain + Pipeline productization

One phased program. Sequenced so open-PR merges are **never reverted or merge-conflicted by the reorg**, with authoring fully parallel and only the merge slot serial.

## Overview

Three interlocking workstreams, one merge queue:
1. **Open-PR drain** — 12 stale (all CI-RED) PRs: rebase / rework / retarget / merge.
2. **Reorg completion** — Batches 1–5 (crates ~90% landed; doc drains + ci + intelligence + the ADR-0615 app/ moves remain).
3. **De-brand tail** — `oya-check-*`, workflow strings, and the STOP-THE-WORLD `oya-ci-required → ci-required` + new `ci-advisory`.

Cross-cutting: every change clears the **6-property pipeline bar** (canonical · automated · portable · universal · comprehensive · productized), and a dedicated **Workstream P** closes the gaps.

**Execution mode: parallel fan-out by default.** Every read/QA/audit sweep and every independent authoring task runs as a concurrent fan-out (workflow of agents), not serially. Only true dependency edges and the single move-slot serialize. If two tasks don't share a file or a dependency edge, they run at the same time.

## Product thesis — the pipeline IS the product (portable, repo-agnostic)

The pain we hit building oyatie is pain everyone building a serious repo hits. So the **entire development pipeline** — spec → plan → implement → review → test → merge → release → observe, plus every gate, scaffold, and reconciler — is a **product** any repo can adopt to get a cloud-native, scalable, well-engineered and -architected, automatically- and easily-maintainable project. This is **not** "CI/CD + governance"; it is the **whole lifecycle**.

**Anti-patterns to eradicate — they destroy portability:**
- **ADR-quoting in engine code/messages.** A gate that emits "per ADR-0555" is meaningless in a stranger's repo. → The **engine is a neutral kernel**; the ADR/authority citation is **policy-as-data** the engine echoes only if the adopting repo supplies one.
- **oyatie-nuance baked into gate logic.** Hardcoded oyatie paths, capability names, ADR ids, or repo-specific branches in engine code. → **policy-as-data**: the engine takes governed roots/params as input. Reference pattern already in-repo: the `canonical-json` gate is explicitly "a NEUTRAL engine… nothing oyatie-specific… all repo-specifics are DATA in the policy file."
- **Context loss across repos.** Any gate/step whose value evaporates outside oyatie. → the engine must produce a sensible result on a bare repo with an empty/default policy, and a great result once a policy is supplied.

**The test for every gate/step/scaffold:** *would it work, and still make sense, dropped into a stranger's repo with only its policy file swapped?* If not, the oyatie-specifics belong in DATA, not CODE. This test is added to the Definition of Done for every Workstream-P task and every gate we touch in Phases 2–4.

## The Ordering Doctrine (the core constraint the user named)

> **Land content BEFORE the move that relabels it. Retarget content whose home already moved. Never merge at a vacated location.**

Concretely, per open PR, the disposition depends on whether its capability's crate move has already landed:

| Capability move state | Open-PR rule | Why |
|---|---|---|
| **Already moved** (19/21 caps) | Rebase + **retarget paths to the landed location**, then merge. | The PR was authored against the pre-move tree; its files at the vacated path are misplaced. The reorg won't revert it, but merging as-authored re-creates a dead directory. |
| **Move still pending** (ci, intelligence) | Merge the PR at the current location **first**, then the move-plan's codemod relabels it. | The deterministic codemod relabels all edges; landing first means the move carries the new code automatically. |
| **Batch-5 app/ move** (comms, storage, console, …) | Re-author the PR's content at the ADR-0615 destination before merge. | e.g. #1293 targets vacated `oya/messenger`; comms crates already landed at `comms/` → the content belongs at `comms/`. |

**Serialization:** only **one `specs/reorg/*-move-plan.json` may be committed-and-in-flight at a time** (the move slot). Open-PR merges use the normal Tide queue. The two interleave at the queue; authoring of everything is parallel.

## Dependency graph (open-PR ↔ reorg ↔ de-brand)

```
ADR-0615 (MERGED) ──unblocks──▶ Batch-5 app/ moves (comms, storage, console, observability, compliance, iam-consent, marketplace, data)
                                        │
#1293 (messenger)  ──retarget to comms/──┤ (must land as comms/ content, or ride the comms move)
#1307 (application)──retarget to app/────┤ (application→app/ per registry app_products)
                                        │
#1287 (iam PDP) ───land before──▶ iam consent Batch-5 slice
#1251 (ci firewall)─land before──▶ Batch-3 ci services move
                                        │
Batch-3 ci move ───────precedes──▶ De-brand tail (oya-ci-required→ci-required needs ci settled)
Batch-4 intelligence ──precedes──▶ De-brand tail
All reorg moves ───────precede───▶ oya-ci-required→ci-required + ci-advisory (STOP-THE-WORLD, LAST)

Workstream P (pipeline productization) ── runs continuously alongside; gates land as each friction is hit
```

## Cross-cutting bar — the 6 pipeline properties (audit axes for Workstream P)

| Property | Definition here | Known gap → task |
|---|---|---|
| **Canonical** | single-source gates; de-branded names; ONE required context | `oya-`/`cloud-ci-` gate names; two-context split pending → P1, P4 |
| **Automated** | gates auto-FIX (not flag-only); auto-rebase queue; born-accounting productized | flag-only gates; manual rebase/shepherding → P2 |
| **Portable** | pipeline runs on ANY repo (hermetic, policy-as-data, no baked-in oyatie paths) | audit gates for hardcoded oyatie specifics → P3 |
| **Universal** | every gate applies repo-wide (R0-pack); evaluates the CANDIDATE tree, not only merge-base | PR/push baseline asymmetry class → P3 |
| **Comprehensive** | full test ladder (contract+integration+E2E+RED/GREEN+load+failure-injection); every friction→gate | coverage holes; friction-ledger burn-down → P2 |
| **Productized** | pipeline-as-product / paved road; frictions→ledger→gates; scaffolds (register_crate, contract-slice) | remaining bespoke/manual steps → P2, P5 |

---

## Phase 1 — Open-PR triage + /ultraqa  *(fully parallel; read-only)*

Fan out one QA pass per open PR (12 independent → parallel agents). Classify each into a disposition; no merges yet.

### Task 1.1 — /ultraqa classify all 12 open PRs
**Description:** For each open PR, inspect the diff + CI failure + reorg-entanglement; assign a disposition: **R** (rebase-only, sound), **W** (rework — slop/Python/CLI), **T** (reorg-retarget), **C** (close/superseded).
**Acceptance criteria:**
- [ ] Every PR has a disposition + one-line reason + the exact post-reorg target path.
- [ ] Python/shell/CLI slop flagged per PR (no-Python/no-shell/no-CLI bar).
- [ ] Reorg-entanglement recorded (which capability move it collides with).
**Verification:** the disposition table reviewed by an independent agent (reviewer≠author).
**Parallel:** yes (12-way fan-out). **Deps:** none. **Scope:** M.

**Provisional dispositions (to confirm in 1.1):**
| PR | Cap | Prov. disposition | Target |
|---|---|---|---|
| #1300 billing | billing (landed) | R | `billing/…` |
| #1305 tools arch-graph | tools | R | `tools/` |
| #1285 bot-autofix | tools | R/W | `tools/` |
| #1209 docs hygiene | docs | R | `docs/` |
| #1287 iam PDP bridge | iam (landed) | R, land before consent slice | `iam/…` |
| #1294 flags RELEASE-001 | flags | W (founder-hold history) | `flags/…` |
| #1290 compliance slice | compliance | W (Python scripts → contract-slice gate) | `compliance/…` |
| #1296 residency slice | data/compliance | W (Python) | per-spec |
| #1297 talos slice | os/k8s | W (Makefile+scripts) | per-spec |
| #1251 ci firewall de-commit | ci (move pending) | R, land before Batch-3 | `ci/…` |
| #1293 messenger control-loop | comms (moved) | T → comms/ | `comms/…` |
| #1307 application shell | app (moving) | T → app/ | `app/application` |

### Checkpoint 1
- [ ] All 12 dispositions confirmed + independently reviewed.
- [ ] Slop list + retarget list finalized. **Human review before draining.**

---

## Phase 2 — Open-PR drain  *(parallel authoring; serial merge)*

Order within the phase by reorg-entanglement (land-before-move first). Each PR: rebase onto current `dev` → retarget paths → fix CI → 5-section admission body → independent review → merge on green `oya-ci-required`.

### Task 2.1 — Drain the sound, non-entangled PRs (R)
`#1300, #1305, #1285, #1209` + `#1287` (land before iam consent slice), `#1294` if salvageable.
**Acceptance:** each rebased, CI-green, independently reviewed, merged; no Python/shell/CLI introduced.
**Parallel:** author all in parallel; merge serially through the queue. **Deps:** Phase 1. **Scope:** S each.

### Task 2.2 — Rework the slop PRs as owned-Rust contract slices (W)
`#1290 (compliance), #1296 (residency), #1297 (talos)` — the contract-slice-conformance gate is merged, so redo each conversion cleanly (append a slice to `contract-slice-policy.json`, delete the Python/Makefile, born-account).
**Acceptance:** zero Python/shell; slice green in the contract-slice gate; spec array-members pulled complete; independently reviewed.
**Parallel:** yes (3 independent). **Deps:** Phase 1. **Scope:** M each.

### Task 2.3 — Land the ci-internal PR before Batch-3 (R)
`#1251` (firewall frozen-ref de-commit) — merge before the ci services move so Batch-3 relabels the settled code.
**Acceptance:** merged; ci gate fleet still green; sequenced before Task 3.3. **Deps:** Phase 1. **Scope:** S.

### Task 2.4 — Retarget the reorg-entangled PRs (T)
`#1293 → comms/`, `#1307 → app/application`. Re-author the content at the post-move location (the vacated `oya/messenger`/`oya/application` are dead).
**Acceptance:** content lands at the ADR-0615/registry destination; no new dir at a vacated path; independently reviewed.
**Deps:** Phase 1; coordinate with Batch-5 comms move (Task 3.5). **Scope:** M each.

### Checkpoint 2
- [ ] All non-close PRs merged or explicitly re-scoped; open-PR count driven to founder-hold-only.
- [ ] No Python/shell/CLI added anywhere; no dead directories created.

---

## Phase 3 — Reorg completion (Batches 1–5)  *(parallel authoring; ONE move-slot serial)*

From `reorg-plan.md`. Author move-plans in parallel branches; merge one committed move-plan at a time.

- **Task 3.1 — Batch 1 (zero-decision):** audit residue sweep + set `status=landed`; compute close-out. *Scope: S. Parallel authoring.*
- **Task 3.2 — Batch 2 (doc drains):** network, gateway, k8s, flags, workflow, billing (+ delete 3 placeholder stubs); secrets/tenancy/cell residue; iam residue slices. *Scope: M. Parallel author, serial merge.*
- **Task 3.3 — Batch 3 (ci services):** controller → tide → webhook-gateway (serial, self-hosting; prove byte-identical Cargo.lock). *After Task 2.3. Scope: L.*
- **Task 3.4 — Batch 4 (intelligence):** outlier re-homes first, then `cloud-intelligence` tail, then `oya/intelligence` sub-batches; per-svc SLO namespacing. *Scope: XL → sub-tasked. Serial tail.*
- **Task 3.5 — Batch 5 (ADR-0615 app/ moves):** comms (unblocks #1293), storage (drive/recordings stay facade; imaging→app/healthcare park), console ops-console verticals, observability (diagnostics park), compliance (governance decompose), iam consent slice, marketplace (dev-cli→ci + settlement→billing), data facades confirm. *Scope: L. Serial merge; several unblock Phase-2 retargets.*

### Checkpoint 3
- [ ] `capability-registry.json` `absorbs_current_dirs` structurally match the tree (no phantom absorbs; ADR-0615 relocations executed).
- [ ] Only 2 top-level meta dirs pending de-brand identifiers remain; membership lint green; no orphans.

---

## Phase 4 — De-brand tail  *(mostly mechanical; ends with the one STOP-THE-WORLD step)*

- **Task 4.1 — `oya-check-*` → `check-*` under `governance/`** (mechanical codemod; rides the governance dep-lint crate moves). *Scope: M.*
- **Task 4.2 — Workflow display-string de-brand** (`cloud-ci-firewall` job, `cloud-ci generated faces` steps, `oya-ci-required.yml` filename → `ci.yml`). Non-required strings, low-risk. *Scope: S.*
- **Task 4.3 — `oya-ci-required` → `ci-required` + new `ci-advisory`** — ADR-0515 amendment; the D-7 shadow/warn→enforce ladder maps onto advisory→required. **Cutover:** add `ci-required` (+`ci-advisory`) as also-required in parallel → bake period both green → flip branch-protection primary → retire `oya-ci-required`. **STOP-THE-WORLD; sequenced LAST; never mechanical.** *Scope: L; founder-gated.*

### Checkpoint 4
- [ ] Zero `oya-`/`cloud-ci-` brand on any surviving gate/crate/context; single canonical required context `ci-required`; `ci-advisory` homes the born-advisory gates.

---

## The agentic delivery fabric — per-task lifecycle (the METHOD for all execution)

Every task/PR flows through this systematic, hyperscaler-pattern pipeline; each stage is handled by the right agent (research / planning / implementation) with the right `{model, effort, skills, MCP, context}` bundle (P7), and each is a **parallel fan-out point** where independent. This is both the productized fabric (ADR-0516..0535) AND the method for this program's own execution — especially the Phase-2b owned-Rust reworks.

1. **Research** (research agents) → evidence brief.
2. **Design doc** — draft DD/RFC as a node in the corpus graph (Workstream D).
3. **Plan + spec** (planner) → plan + spec; ambiguity≤0.2 (NEEDS_CLARIFICATION) gate.
4. **RED/GREEN tests FIRST** (TDD; test-engineer) → failing tests before code.
5. **Code** (implementation/executor; owned-Rust, buck2).
6. **Review + fix** (code-reviewer; reviewer≠author).
7. **Harden + security audit** (security-auditor; the P6 CVE/RustSec/SAST/CNAPP battery).
8. **Edge cases + perf/algorithms + capacity** — predictable bottlenecks (Big-O, cache, preload, modern system-design + data quirks); **distributed load generation** + **auto-scaling validation** (KEDA/HPA); **stress + capacity testing to find the ABSOLUTE breaking point** — deliberately impose limits to surface hidden ceilings: cloud-provider API quotas, IP-address allocation pools, DB connection thresholds, fd/port exhaustion.
9. **Simplify + refactor** (code-simplifier).
10. **Full test suite** (test-engineer) — write the REST of the tests beyond the initial RED/GREEN: **regression tests** locking every review / harden / edge-case fix so it can't recur, plus **integration + E2E + contract + load + failure-injection per tier**, **IaC unit tests** (helm/kustomize/CRD/Terraform-equiv), and **policy-as-code enforcement** (owned PDP/Cedar over IaC + configs) — the full testing-standards ladder. No behavior ships without regression + integration coverage; "unit green" never satisfies acceptance.
11. **Slop cleanup** — /ai-slop-cleaner + /ponytail:ponytail-audit + /ponytail-review.
12. **Comprehensive review + /doubt-driven-development + /verification-before-completion** — root-cause, doubt-until-it-survives, evidence-based completion.
13. **CI/CD + automation** — /ci-cd-and-automation; green `oya-ci-required` (→ `ci-required`).
14. **Ship + launch** — /shipping-and-launch; progressive delivery (ADR-0040 canary/blue-green/metric-gated rollback).
15. **Self-improve, closed loop after meta-assessment** — /self-improve on a measured metric (deferred until the program settles; first target = the P7 routing policy).

Ordering rules: RED/GREEN precedes code (4→5); the full ladder (10) follows the stabilized/refactored code and **locks every fix as a regression test**; review≠author; verification (12) gates completion; stages 6–10 pipeline per task; research (1) + the reworks fan out across tasks.

## Workstream P — Productize the ENTIRE development pipeline as a portable product  *(continuous; heavy parallel fan-out)*

Scope is the **whole lifecycle** (spec→plan→implement→review→test→merge→release→observe), not just CI/CD + governance. Every task here follows one **method** — never symptom-patching:

**Method (per friction / anti-pattern):**
1. **Root-cause, not symptom.** A red gate / stale PR / manual step is a symptom — find the class behind it.
2. **/idea-refine + /doubt-driven-development** to verify the diagnosis is the *actual* root cause: question until ambiguity is low; assume the first diagnosis is wrong until it survives doubt. (Ouroboros discipline.)
3. **Canonical/universal capability** as the output — a **neutral-engine + policy-as-data** gate/scaffold/reconciler that fixes the CLASS repo-agnostically, passes the **stranger's-repo test**, and drives its friction-ledger entry to zero.
4. Ships with the **full test ladder** + a **regression gate** so the class cannot recur.
5. **Automation where possible, enforcement everywhere.** The capability **auto-fixes by default** and **BLOCKS as the backstop** — no flag-only gates. Advisory/shadow is only the D-7 on-ramp (reports to `ci-advisory`), never the resting state; every gate graduates to enforcing on the R0 path.

**P0 — Neutralization audit (centerpiece; parallel fan-out over the whole gate/step fleet).** For every gate/step/scaffold: does the ENGINE quote an ADR, hardcode an oyatie path / capability / id, or lose meaning outside oyatie? Fan out one auditor per gate → each returns `{neutral | violations[]}` → violations pipeline into a neutralization fix (specific → policy-as-data; leave a neutral kernel). Reference: the `canonical-json` NEUTRAL-engine pattern already in-repo.

- **P1 Canonical:** de-brand gate identifiers (feeds Phase 4); single-source duplicated gate logic; one required context.
- **P2 Automated + Productized:** flag-only → auto-fix; stand up the **auto-rebase queue** — the root-cause class-fix for "all 12 PRs went stale" (manual shepherding is a process failure, not a per-PR chore); friction-ledger burn-down; extend the contract-slice + register_crate paved roads; **productize the earlier lifecycle too** (spec→plan→implement→review scaffolds), not only gates.
- **P3 Portable + Universal:** land the P0 outputs; ensure gates evaluate the CANDIDATE tree, not only merge-base (close the PR/push baseline-asymmetry false-green class); **prove portability on a scratch repo**.
- **P4 Canonical (contexts):** the `ci-required` + `ci-advisory` topology (Task 4.3).
- **P5 Comprehensive + test-health:** full-ladder coverage audit (contract+integration+E2E+RED/GREEN+load+failure-injection) per tier; every merged friction carries a regression gate; **flaky-test detection + quarantine + hardening** (deterministic ret/seed analysis; no flaky test on the R0 path — flakiness is a defect, not noise).
- **P8 Operational-friction prevention:** every recurring operational friction (manual step, hand-rebase/re-dispatch, drift, staleness) is a **process failure** logged to the friction-ledger and closed by a universal capability (root-cause method) — never absorbed as toil. The auto-rebase queue (P2) is the exemplar; "manual twice → write the automation."
- **P6 Security & supply-chain (depth + breadth, ENFORCING):** owned-Rust vulnerability + supply-chain coverage benchmarked against the **best-in-class set** — **CVE, RustSec, Trivy, Snyk, Veracode, Checkmarx, Black Duck, AST (SAST/DAST/IAST umbrella), Prisma Cloud, Wiz, Aqua Security, Cycode** — adopting their methodology and reimplementing owned-Rust (references + transient adapters per the transient-stack bar; owned destination behind a port).
  - **Categories (breadth):** SAST (code), DAST, SCA / OSS + license (Black Duck/Snyk), container & image (Trivy/Aqua), K8s & runtime (Aqua/Prisma), CNAPP / CSPM cloud posture (Wiz/Prisma), IaC scanning, secrets/credential leakage, ASPM / software-supply-chain (Cycode), SBOM (CycloneDX/SPDX).
  - **Depth:** full transitive dependency graph, advisory severity (CVSS), reachability/exploitability, fix-version guidance, provenance/attestation (SLSA), license posture.
  - **Enforcing** on every PR **and** on a schedule (zero-day drift catches deps clean at merge). **Fail-closed on new criticals**; informational advisory rides `ci-advisory` until D-7 graduation. Benchmark battery: mechanical proof of coverage per category (which tool it matches, what it catches).

- **P7 Agentic routing — the {model, effort, skills, MCP, context} bundle (agentic fabric):** the orchestrator provisions each team-member / subagent with the right bundle by task complexity + scope:
  - **Model — CROSS-MODEL:** Claude (Opus/Sonnet/Haiku/Fable), **Codex/GPT**, or **Gemini** (per the subscription-OAuth pooling thesis) — e.g. cheap tiers for mechanical/rebase, sonnet/medium for standard authoring, opus (or fable for reviews) for architecture/design/judge, **codex-xhigh for cross-model adversarial verification**.
  - **Effort:** low→max matched to difficulty.
  - **Skills / MCP / Context:** provision only the skills + tools the task needs, and right-sized context (enough to act, never a dump).
  - **Orchestrator context stays CLEAR:** the main/orchestrator context is reserved for orchestration + coordination — heavy detail/context is pushed DOWN to subagents; the orchestrator holds coordination state + distilled conclusions, never raw material (parse subagent results compactly; only conclusions come back up). Right context down, only conclusions up.
  - Routing is **policy-as-DATA** (a complexity→bundle table), not hardcoded; measured on **cost vs success-rate**. This is the natural **first `/oh-my-claudecode:self-improve` target** once the program settles (optimize the routing policy against a cost/quality metric — self-improve deferred per founder 2026-07-10).

### Checkpoint P
- [ ] Model/effort routing is policy-as-data + measured (cost vs success-rate); no blanket top-tier default.
- [ ] Every gate/step passes the **stranger's-repo test** — neutral engine + policy-as-data; **zero ADR-quotes / oyatie-nuance in engine code** on the R0 path.
- [ ] Pipeline runs green on a **scratch repo** with only its policy swapped (portability proof).
- [ ] Auto-rebase queue live; friction-ledger at/under target; zero flag-only gates on the R0 path.
- [ ] Full-ladder coverage matrix has no red cells on the R0 path.
- [ ] **Security green:** CVE / RustSec / Trivy / Snyk-equivalent depth+breadth enforcing on PR **and** schedule; fail-closed on new criticals; SBOM + license posture clean.
- [ ] **Enforcement everywhere:** no flag-only gate remains on R0; every gate auto-fixes or blocks (advisory only as the D-7 on-ramp to `ci-advisory`).
- [ ] Each closed friction traces to a **root-cause record** (idea-refine / doubt-driven) + a **universal capability**, not a symptom patch.

---

## Workstream D — Documentation alignment: PRD ↔ RFC/Design ↔ ADR as a relational graph  *(research → owned substrate)*

**Root cause:** loose cloud folders of markdown let PRDs (business requirements), RFCs/Design Docs (engineering specs), and ADRs (architectural realities) drift into cross-document contradictions + staleness. The fix is a **structured, relational knowledge graph / docs-as-code** framework where every requirement→spec→decision edge is live and contradictions are mechanically detectable.

**Fits the existing direction:** this IS the founder-approved **corpus-governance substrate** (`governance/corpus/` live AST graph replacing markdown-ADRs + JSON-SSOT). The `cross-artifact-agreement` gate already enforces ADR↔spec↔masterplan↔roadmap agreement — extend it to PRD↔RFC↔ADR.

- **D0 — Explore (DONE, workflow wlxyve2vr):** the three categories collapse into ONE evaluator extension, NOT a new subsystem. **Docs-as-Code = spine** (schema-validated typed nodes + forced PRD→RFC→ADR→code traceability + a NEEDS_CLARIFICATION ambiguity gate = the ambiguity≤0.2 doctrine made structural). **Relational Wiki = substrate** (governance/corpus/ typed node/edge graph; reverse edges DERIVED from one declared end — Backstage's trick, so two ends physically cannot disagree; the graph is a PROJECTION of the code-AST reality — Foundry ontology, freshness automatic; rendered ADR/PRD/RFC markdown + JSON SSOT become BUILD ARTIFACTS compiled OUT of the graph → contradiction structurally impossible, and md/json count drops per masterplan-v2). **AI Context Engine = ADVISORY-ONLY** (retrieve→NLI→judge semantic-contradiction files an ISSUE, never a merge verdict — LLM verdicts are inadmissible as merge authority per the evidence-admissibility bar).
- **D1 — Extend the EXISTING `cross-artifact-agreement` evaluator (do NOT rebuild):** it already IS this engine (pure fixture-driven evaluator over a corpus Value → keyed {code,key} findings + net-new ratchet + carve-outs-as-data). Add 2 node kinds (PRDRequirement, DesignDoc/RFC — DD-NNNN is oyatie's RFC tier) + typed edges (realized_by / ratified_by / implemented_by / conflicts_with; implemented_by SOURCED from existing ADR frontmatter affected_surfaces.crates) + content-hash drift anchors (reuse accounting-registry hashes + the generated_face_drift compare) + a rulepack-as-DATA. Reuse staleness-reaper (TTL/reachability/report-not-reap) as-is.
- **D2 — New ENFORCING gates (deterministic ONLY):** unrealized_requirement, unratified_design, unimplemented_decision, conflicting_accepted_pair, unresolved_clarification, anchor_fingerprint_drift, untyped_anchor — plus EXTEND status_disagreement / unpropagated_decision / supersession_half_edge / dual_id_collision to cover PRD + DesignDoc, and orphan_node (report-not-reap). Predicates MUST read the CANDIDATE tree (only the ratchet is merge-base-anchored). Ship shape = neutral engine (node/edge store + reconciler + invariant evaluator) with ontology-pack / anchor-extractors / rulepack / MIF conformance-levels as DATA; **stranger's-repo fixture (REQUIREMENT→TEST→MODULE, a non-ADR repo) is a REQUIRED gate** so oyatie-nuance leakage into the engine fails CI. Author as an IMPLEMENTATION of the corpus-pivot + cross-artifact-agreement ADR cluster.

### Checkpoint D
- [ ] Exploration synthesized → owned-substrate recommendation (adopt methodology, reimplement owned-Rust).
- [ ] PRD/RFC/ADR modeled as a relational graph; contradiction + staleness detectors enforce; portable (stranger's-repo test).

## Parallelization map

- **Fully parallel:** Phase 1 (12-way QA), all authoring in Phase 2/3, Workstream P gates.
- **Serial:** the single move-plan slot (Phase 3); the ci services chain (3.3); the intelligence tail (3.4); the STOP-THE-WORLD context flip (4.3).
- **Coordination edges:** #1251→3.3; #1287→iam consent slice; #1293/#1307→comms/app Batch-5; all reorg→4.3.

- **D3 — `/documentation-and-adrs` authoring skill (HYBRID: seed thin skill + amend owned-Rust; workflow wxl1we5pd):** the enforcement half is **~70% already built** — `governance/corpus/{core,doc-parser,work-area-rust-parser,extract}` (content-addressed anchors via `core::Function::signature_hash`, ADR/PRD/RFC markdown→node parsing per ADR-0517/0541, Rust-AST→node facts, a projection binary) + the pure `cross-artifact-agreement` evaluator (carve-outs-as-DATA = the neutral-engine shape, already satisfied; 8 decision-face codes map 1:1 to the target's status/propagation/supersession family).
  - **Deliverable (A) SEED:** a thin `.omc/skills/documentation-and-adrs/SKILL.md` — 9 steps: ingest+classify → author-as-graph-mutation (edges declared one-end, reverse derived) → anchor-bind to code-AST → **deterministic ambiguity gate** (NEEDS_CLARIFICATION, ≤0.2, structural predicate not LLM) → compile docs as build artifacts → **deterministic invariants (sole merge authority)** → LLM/NLI advisory pass (files an issue, never a verdict; optional `--advisor codex`) → separate review lane → **stranger-repo conformance fixture**.
  - **Deliverable (B) AMEND** 7 owned-Rust surfaces: corpus core/doc-parser/extract (+ render mode + reverse-edge derivation), the evaluator (VIOLATION_CODES +7: unrealized_requirement, unratified_design, unimplemented_decision, conflicting_accepted_pair, anchor_fingerprint_drift, untyped_anchor, unresolved_clarification), fixtures, `specs/ontology-packs/` (DATA: ADR/PRD/RFC ontology+rulepack+MIF-levels + a stranger-repo pack), ADRs Proposed (anchored to the ADR-0517/0541/0580 corpus cluster).
  - **Do NOT amend** deep-research (compiled into the CLI binary — un-editable) or ralplan (shared OMC plugin; its Critic-LLM-verdict-gates conflicts with the deterministic-only merge bar) — **ingest one-way**: deep-research REPORT → requirement/clarification nodes; ralplan ADR-block + RALPLAN-DR → decision/design nodes (harvest the schema 1:1). Namespace oya-scoped to avoid the addy `agent-skills:documentation-and-adrs` collision.
  - **7 open founder Qs (rule when we build D):** node-home (core vs doc-parser), gate single-concern-vs-sibling (ADR-0132), PRD/DD id-registry authority, the exact deterministic ambiguity rubric, de-commit accounting for the rendered faces, skill namespace, advisory substrate (owned-NLI vs Codex-xhigh).

## Benchmark & reference battery (adopt-methodology → reimplement owned-Rust, behind a port)

The owned pipeline + infra is measured against best-in-class across every dimension. Transient tools are **references + adapters** (transient-stack bar); the destination is owned-Rust, neutral-engine + policy-as-data, portable. Each category ships a **mechanical coverage proof** (which target it matches, what it catches) and is **enforcing** where applicable.

| Dimension | Benchmark / reference targets | Owned home (capability) |
|---|---|---|
| Security & supply-chain | CVE · RustSec · Trivy · Snyk · Veracode · Checkmarx · Black Duck · AST · Prisma Cloud · Wiz · Aqua Security · Cycode | P6 → ci/ + a security capability |
| Architecture verification / conformance | **Axivion Architecture Verification · ArchUnit** (Rust-native equivalent for our concerns) **· FINOS CALM** (Common Architecture Language Model) | ci/ dep-lint + membership-lint + the ADR-0280 DAG; extend to full arch-conformance |
| Performance / load | **Grafana K6** | P5 test ladder → load-testing capability |
| Continuous Delivery / progressive rollout | **Harness CD** (+ ADR-0040 canary / blue-green / metric-gated rollback) | ci/ + iac/ delivery |
| Chaos / resilience (CONTINUOUS) | **Chaos Mesh · LitmusChaos** — continuous chaos engineering + automated fault injection + failover validation + graceful-degradation verification + **multi-region DB failover within target RTO/RPO** | resilience capability + observability SLOs; runs in the test ladder (stage 10) AND continuously post-ship |
| Code quality | **SonarQube Enterprise / SonarCloud · Code Climate Quality** | ci/ code-quality gates + the review lane |
| Developer portal / service catalog / IDP maturity | **Spotify Backstage · Qovery · Cortex · OpsLevel** | console/ + the catalog + Workstream D corpus graph |
| Serverless & event-driven + autoscaling | **KEDA** · event-driven processing paradigm | compute/ + messaging/ (event-driven autoscaling) |
| IaC testing + policy-as-code | Terratest · OPA/Conftest · Kyverno · Checkov · tfsec | iac/ + the owned PDP/Cedar policy engine |
| Capacity / breaking-point | distributed load (K6) · stress + soak to failure; find hidden ceilings — cloud API quotas, IP-pool exhaustion, DB connection limits, fd/port exhaustion | perf/capacity (lifecycle stage 8) + observability SLOs |
| Observability & telemetry | synthetics + **RUM** · high-cardinality telemetry · alert fingerprinting · **synthetic trace verification** · high-volume telemetry injection · cross-cloud telemetry fidelity | observability/ + SLOs |
| Identity & access security | **IAM fuzzing** · multi-provider IAM assertions · **matrixed privilege-escalation scans** · federated token-expiration testing · secrets-management audits · network-segmentation auditing | iam/ (RBAC+ABAC+PBAC, Cedar PDP) + secrets/ + P6 |
| Multi-cloud / cross-cloud | multi-cloud security + isolation testing · cross-cloud interoperability · orchestration abstraction + **drift validation** · multi-provider IaC mocking · **cross-platform K8s compliance** | k8s/ (owned-K8s conformance bar) + iac/ + the transient adapters (adapter-aws/-oci/-capi) that vanish at owned-stack cutover |
| Network | latency + jitter simulation · MTU/fragmentation testing · **egress cost anomaly** · **unified network overlay** eval (eBPF/Cilium-class → owned overlay, or better) | network/ (service mesh + overlay) + billing/finops (egress attribution) |
| Active-active / multi-region / DR | **active-active split-workload** validation · disaster-recovery + failover drills · RTO/RPO conformance · region evacuation | cell/ + data/ + network/ (ADR-0158 multi-region active-active); ties to the dr conversion (#1302) |

Notes: (1) **ArchUnit/Axivion/FINOS CALM** validate the exact clean-arch conformance the reorg produces (core purity, port seams, layering DAG) — fold into the membership + dep-lint so architecture drift is caught mechanically. (2) **Backstage/Cortex/OpsLevel** are the catalog/IDP reference for Workstream D (Backstage's declared-entity → reconciled-relation-graph model already surfaced as the top analog). (3) **Harness CD + KEDA + Chaos Mesh** extend the pipeline past merge into deploy/scale/resilience — the "entire dev lifecycle" scope.

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Open PR merged at a vacated path → dead dir | Med | Ordering doctrine: retarget-before-merge for moved caps |
| Reorg move reverts a just-merged PR | Med | Land-before-move for pending-move caps; codemod relabels |
| `oya-ci-required` flip wedges the merge queue | **High** | Parallel-add + bake + flip; never one-shot; sequenced last |
| Re-staling: PRs go RED again while queue drains | Med | Auto-rebase queue (P2) is the class-fix, not per-PR toil |
| Slop reintroduced in reworks | Med | contract-slice + automation-language gates block new Python/shell |

## Founder decisions (resolved 2026-07-10)
- ✅ **`ci-required` + `ci-advisory` split — LOCKED.** Final de-brand step; ADR-0515 amendment; D-7 shadow/warn→enforce ladder maps onto advisory→required; STOP-THE-WORLD parallel-add → bake → flip.
- ✅ **#1294** — Phase-1 /ultraqa classifies it (rework / close / keep, with evidence); founder rules with full context.
- ✅ **Batch-2 doc-drains run PARALLEL** with the Phase-2 open-PR drain (only the merge queue + single move-slot serialize).
