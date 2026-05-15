# RALPLAN — oyatie Single Source of Truth + grit/icm Cutover

<!--
status: Accepted
date: 2026-05-12
related_adrs: ADR-0052, ADR-0053, ADR-0054, ADR-0055
-->

Phase-0 input: `docs/decisions/specs/deep-dive-oyatie-sst-consolidation.md`
Trace synthesis: `docs/decisions/specs/deep-dive-trace-oyatie-sst-consolidation.md`
Mode: `ralplan --consensus --direct --deliberate-auto` (deliberate engaged — multi-repo cutover, deletion path, agent-rule rewrite)
Iteration: **Iteration 2 — incorporates Architect (4 violations + 8 revisions) + Critic (4 additional findings) + foundry-salvage output + orchestrator existence findings**

---

## Directory contents

| File | Purpose |
|---|---|
| `INDEX.md` | This file — master cutover plan, phased execution sequence |
| `architect-review-iter-1.md` | Architect ITERATE verdict, iter-1 (8 revision requests) |
| `architect-review-iter-2.md` | Architect ITERATE verdict, iter-2 (ADR lift-source gap identified) |
| `critic-review-iter-2.md` | Critic APPROVE verdict, iter-2 final |
| `cross-cutting-amendments.md` | 12 directives + LTS pins + hyperscaler inheritance (post-Critic amendments) |
| `open-questions-resolutions.md` | Mechanical Q&A: ADR slots, helper language, scaffold-claim, demo symbols |
| `orchestrator-existence-findings.md` | Verified file/dir state before iter-2 planning |
| `pre-cutover-drafts.md` | Six implementation drafts feeding ADRs and runbooks |

---

## 1. RALPLAN-DR Summary

### Principles
- **Inventory precedes deletion.** No artifact leaves the active path without an ADR row classifying it (`KEEP|REPLACE-WITH-*|ARCHIVE|DELETE`). Per spec §Constraints item 3.
- **Sanctioned primitives are a closed set: `{grit, icm, oya-tooling-agent-read}`** — post-cutover. During the **cutover bootstrap window (P0.5–P2 inclusive)** the set is `{grit, icm}` plus an audited carve-out for new-crate scaffolding (`icm-coordination-lock`) and human-orchestrator `git mv`/`git rm`/`gh issue create`. Banned-primitives fitness lane activates at **P5 merge**. Per spec §Constraints item 1.
- **Authoritative = repo-tracked.** ICM may mirror cross-project memory, but project-canonical decisions land in `oyatie/docs/` as tracked files. Per spec §Constraints item 2.
- **Reshape data to eliminate special cases.** `G004-reconciliation-blocker.md`, `PAUSE.md`, omx checkpoint flow vanish because grit's data model has no place for them — not because we add a shim. Per spec §Constraints item 7.
- **Parallel agent work is the first-class default, with scaffold-claim fallback.** Every phase enumerates real `file::Identifier` claims for grit-indexed symbols, or — for new-crate phases (P2, P3, P10) and doc-only phases — coordinates via the `scaffold-locks-oyatie` icm topic per **ADR-0054 grit-scaffold-claim-pattern** (lift-source `pre-cutover-drafts.md §Draft 2`). Verified: `grit symbols | grep Cargo.toml` returns zero matches, so the `Cargo.toml::workspace_members` primary option in Draft 2 is not viable; the icm-coordination-lock fallback is the canonical path. Per spec §Constraints item 5.

### Decision Drivers
1. **Reversibility of the cutover** — archive-before-delete keeps `git revert` cheap; collapsing inventory and deletion into one PR makes rollback expensive.
2. **Agent-flow correctness during the cutover** — agents executing the plan are themselves bound by the new rule, so `oya-tooling-agent-read` must exist before agent-instruction rewrites force its use, AND the bootstrap-window meta-contradiction (P1–P2 cannot enforce a rule that names a primitive it is concurrently constructing) must be acknowledged in the ADR.
3. **bominal↔oyatie boundary integrity** — A1 bidirectional citation must land early so subsequent doc edits cannot accidentally re-orphan the boundary; the foundry corpus in `bominal/agents/ultragoal/` (9 KEEP-classified files, ~245KB of spec content) must be cross-cited into `oyatie/docs/products/foundry/PHASE-00-SPEC.md` (NEW) before any archive/delete-adjacent operation.

### Viable Options

**Option A — Strict-phased, archive-first (RECOMMENDED).** Scaffold-claim ADR lands P0.5; inventory ADR lands P1; `oya-tooling-agent-read` ships P2 (under scaffold-claim) before any agent-instruction rewrite; bidirectional citation P3; foundry-corpus cross-cite P3.5; doc/hook rewrites P4–P5 on the new primitives; archive P6 with archive-orphan lane scaffolded; delete P7 only after archive PR has merged and three concrete gates green; demo and audits P8–P10. Multiple PRs gated on prior-phase merge.

**Option B — Single-PR cutover.** One mega-PR carries everything together. Rejected: violates spec §Constraints item 3 in spirit; diff too large for grit parallel-claim model.

**Option C — Archive-and-delete-in-parallel batches.** Inventory first; subsequent phases bundle archive+delete per logical batch. Rejected: archive→delete same-PR weakens recoverability; conflicts with §A3 verification.

**Recommended: Option A (strict-phased, archive-first).**

### Pre-mortem

| # | Failure scenario | Probability × Impact | Mitigation |
|---|---|---|---|
| 1 | `oya-tooling-agent-read` ships in P2 but lacks a primitive an agent actually needs mid-plan. | Medium × High | P2 acceptance gates on fixed read-set from spec §A4. Explicit escape hatch: missing-primitive → agent `icm store -t agent-read-missing-primitive` and HALT; human orchestrator extends helper or grants one-time bootstrap carve-out. |
| 2 | Archive directory is staged but deletion PR (P7) merges before all consumers re-wired. | Medium × Medium | Gate P7 on three concrete checks: (a) banned-primitives lane green, (b) archive-orphan lane confirms no living references, (c) inventory ledger `archived_at` non-null for every ARCHIVE row. |
| 3 | grit 0.3.0 `grit session start` bug widens to break `grit claim` or `grit done` mid-cutover. | Low × High | Plan uses session-less mode by default. P9 files upstream bug. Fallback: human-orchestrator sequential mode. |

### Expanded test plan

- **Unit.** `oya-tooling-agent-read` ships with unit tests per subcommand in `tools/oya-tooling-agent-read/tests/` (Rust `cargo test`). Each subcommand has at least one happy-path test and one fail-closed test.
- **Integration.** Banned-primitives fitness lane runs `rg` against an explicit file enumeration for banned tokens inside agent-instruction sections (HTML-comment-fence-scoped). ADR-shape lane confirms inventory ADR classification values from closed set. Portfolio-citation lane runs on both bominal and oyatie sides. Archive-orphan lane runs at P6→P7.
- **End-to-end.** Parallel-claim demo at `docs/runbooks/agentic-pipeline/grit-parallel-claim-demo.md` records two agents claiming `crates/oya-cloud-billing-app/src/lib.rs::CloudBillingEventIngestAppStatus` and `::CloudBillingMeterUnitRecord` in parallel. Demo executable in CI.
- **Observability.** Each `oya-tooling-agent-read` invocation emits an audit-chain event. Each cutover bootstrap-window human-orchestrator carve-out emits an `icm store -t cutover-orchestrator-actions ... -i critical` event BEFORE execution. Authoritative-tracked lane walks the repo and confirms every file referenced as authoritative in `docs/AGENTS.md` is tracked.

---

## 2. Phased Plan

**Total phases: 12** — P0.5, P1, P2, P3, P3.5, P4, P5, P6, P7, P8, P9, P10.

### P0.5 — Land ADR-0054 grit-scaffold-claim-pattern + human-orchestrator definition
- **Inputs/preconditions:** Architect verdict ITERATE consumed; Critic findings consumed; lift-source `pre-cutover-drafts.md §Draft 2` exists; `oyatie/docs/RACI-OWNERSHIP.md` exists (70 rows, verified).
- **Symbols to claim:** doc-only — coordinate via `icm-lock-p0.5` topic.
- **Agents and parallelism:** 1 agent drafts ADR-0054; 1 agent appends a "human orchestrator" row to `docs/RACI-OWNERSHIP.md` naming the cutover orchestrator role + the icm-event-before-execution requirement. Both edits can proceed in parallel via per-file icm sub-locks.
- **Outputs / acceptance evidence:** (a) `oyatie/docs/decisions/ADR-0054-grit-scaffold-claim-pattern.md` exists, lifted from `pre-cutover-drafts.md §Draft 2`. ADR body documents primary option (`Cargo.toml::workspace_members`) as **NOT viable** and adopts the icm-coordination-lock fallback as canonical. (b) `oyatie/docs/RACI-OWNERSHIP.md` has a new row for "human orchestrator (cutover-2026-05-12)". (c) `ADR-INDEX.md` appended.
- **Maps to spec criterion:** A2 (prerequisite for inventory cleanliness); unblocks A4/A1/A8.

### P1 — Inventory pass (no deletions yet)
- **Inputs/preconditions:** P0.5 merged.
- **Symbols to claim:** doc-only — coordinate via `icm-lock-p1` topic.
- **Agents and parallelism:** Up to 4 agents in parallel for inventory enumeration. Scope: `oyatie/**` excluding agents-config; `bominal/agents/`; `bominal/docs/`; and a fourth agent on agent-instruction scope inside the repo (`**/*.md`, `**/*.json`, `**/.claude/**`).
- **Outputs / acceptance evidence:** `oyatie/docs/decisions/ADR-0103-grit-cutover-inventory.md` (verified next free slot per `open-questions-resolutions.md §Q2`). Every file/dir/script in scope has one row with classification ∈ closed set AND a `archived_at` timestamp column (null until P6 stamps it). Referenced from `ADR-INDEX.md`. **Phantom-path correction**: `oyatie/.omx/ultragoal/` row marked `phantom — not present, no action`. `.codex/worktree_init.sh` **dropped from deletion list entirely** (verified non-existent). Helper-crate-target row reads `tools/oya-tooling-agent-read/ — REPLACE-WITH-HELPER`.
- **Maps to spec criterion:** A2.

### P2 — Ship `tools/oya-tooling-agent-read/` (under scaffold-claim pattern per ADR-0054)
- **Inputs/preconditions:** P0.5 merged; P1 merged.
- **Symbols to claim:** **Scaffold-claim per ADR-0054** (lift-source `pre-cutover-drafts.md §Draft 2`; primary `Cargo.toml::workspace_members` NOT viable, verified). icm-coordination-lock sequence; after window closes, normal `grit claim file::Identifier` against now-indexed symbols.
- **Agents and parallelism:** 1 lead agent owns scaffold-claim window; after window closes, 3 agents in parallel — one per command pair (log+diff, pr-view+pr-comments, audit+cli wiring). One agent gates on the others for integration tests.
- **Outputs / acceptance evidence:** `tools/oya-tooling-agent-read/` directory; `cargo test -p oya-tooling-agent-read` green; `oya-tooling-agent-read log 5` succeeds and emits audit event; mutation attempt fails closed. Scaffold-lock window closed in icm.
- **Maps to spec criterion:** A4. Citation: ADR-0054 and `pre-cutover-drafts.md §Draft 2`.

### P3 — Bidirectional PRD citation + portfolio-citation fitness lane
- **Inputs/preconditions:** P0.5 merged; P1 merged.
- **Symbols to claim:** Two-track — doc-only icm-locks for PRD edits; scaffold-claim per ADR-0054 for `oya-foundry-fitness-portfolio-citation-kernel`.
- **Agents and parallelism:** Track 1: 2 agents in parallel editing both PRD files. Track 2: 1 lead agent scaffold-claim window; 1 follow-up agent implements kernel logic (lifted from `pre-cutover-drafts.md §Draft 5`).
- **Outputs / acceptance evidence:** Both PRD files cross-cite; `cargo run -p oya-foundry-fitness-portfolio-citation-kernel` green.
- **Maps to spec criterion:** A1. Citation: ADR-0054.

### P3.5 — Cross-cite ultragoal foundry corpus into canonical foundry SPEC
- **Inputs/preconditions:** P1 merged; foundry-salvage output exists at `docs/products/foundry/PHASE-00-SPEC.md` (lifted in Stage 1 Wave 3); `oyatie/docs/products/foundry/PRD.md` exists (75.5KB, verified).
- **Symbols to claim:** doc-only — `icm-lock-p3.5` topic + scaffold-lock for new docs path.
- **Agents and parallelism:** 1 agent lands foundry PHASE-00-SPEC content; 1 agent in parallel edits `oyatie/docs/products/foundry/PRD.md` adding cross-cite block; 1 agent extends portfolio-citation lane or scaffolds sibling lane.
- **Outputs / acceptance evidence:** `oyatie/docs/products/foundry/PHASE-00-SPEC.md` exists; foundry corpus cross-cites in place; lane green.
- **Linus data-shape reshape:** "Foundry-canonical claims in bominal mega-plan without oyatie-side cite" → "every foundry-canonical claim has a tracked oyatie-side cite or landing."
- **Maps to spec criterion:** A1, A2, A8. **Deadline:** BEFORE P4.

### P4 — Rewrite agent-facing memory
- **Inputs/preconditions:** P2 merged; P1 merged; P3.5 merged.
- **Symbols to claim:** doc-only — `icm-lock-p4`. Files: `oyatie/CLAUDE.md`, `oyatie/AGENTS.md`, `oyatie/docs/AGENTS.md`. Up to 3 agents in parallel via per-file icm sub-locks.
- **Outputs / acceptance evidence:** `oya-foundry-fitness-banned-primitives` lane scaffolded (not yet enforcing). HTML-comment-fence convention `<!-- agent-instructions:start -->` / `<!-- agent-instructions:end -->` introduced.
- **Maps to spec criterion:** A5.

### P5 — Hook + skill audit (banned-primitives lane activates here)
- **Inputs/preconditions:** P4 merged.
- **Symbols to claim:** doc/config-only — `icm-lock-p5`. Scope: all `**/*.md`, `**/*.json`, `**/.claude/**` paths inside the repo containing agent-instruction sections.
- **Outputs / acceptance evidence:** (a) `oyatie/docs/AGENT-INSTRUCTION-SOURCES.md` enumerates every agent-instruction-bearing file. (b) `oya-foundry-fitness-banned-primitives` lane **activates** — zero hits for banned tokens in agent-instruction sections. (c) Lane scope extended to grep for stray `archive/pre-grit-cutover-2026-05-12/` references.
- **Linus data-shape reshape:** "scattered git/gh references in agent skills" → "single grit/icm/oya-tooling-agent-read invocation pattern, lane-enforced."
- **Maps to spec criterion:** A5 (activation), A6.

### P6 — Archive (NOT delete) orchestration glue + scaffold archive-orphan lane
- **Inputs/preconditions:** P1, P2, P3.5, P4, P5 merged. Banned-primitives lane green.
- **Symbols to claim:** Two-track: `git mv` (human-orchestrator carve-out) + scaffold-claim for `oya-foundry-fitness-archive-orphan-kernel`.
- **Outputs / acceptance evidence:** Archive directory exists; deprecation notice at `bominal/agents/ultragoal/DEPRECATED.md`; archive-orphan lane green; every ARCHIVE-class row has non-null `archived_at` timestamp; icm carve-out events recorded.
- **Maps to spec criterion:** A3 (archive half).

### P7 — Delete archived glue from active path; three concrete gates
- **Inputs/preconditions:** P6 merged AND three concrete gates all green: (1) banned-primitives lane green; (2) archive-orphan lane green; (3) `archived_at IS NULL` returns zero rows.
- **Symbols to claim:** doc-only — `icm-lock-p7`. `git rm` is human-orchestrator carve-out.
- **Outputs / acceptance evidence:** `grit symbols` shows no active orchestration-glue paths; both fitness lanes green; icm carve-out events recorded.
- **Maps to spec criterion:** A3 (delete half).

### P8 — Parallel-claim demo runbook (symbols pinned)
- **Inputs/preconditions:** P2, P7 merged.
- **Symbols to claim:** `crates/oya-cloud-billing-app/src/lib.rs::CloudBillingEventIngestAppStatus` (agent-A) + `crates/oya-cloud-billing-app/src/lib.rs::CloudBillingMeterUnitRecord` (agent-B).
- **Outputs / acceptance evidence:** `docs/runbooks/agentic-pipeline/grit-parallel-claim-demo.md` with timestamps, `grit watch` excerpts, reproducible script, negative case.
- **Maps to spec criterion:** A7. Citation: `pre-cutover-drafts.md §Draft 3`.

### P9 — File upstream `grit session start` bug
- **Inputs/preconditions:** None — can run in parallel with any later phase.
- **Outputs / acceptance evidence:** `docs/runbooks/grit-session-bug-upstream.md` with upstream issue URL. Human-orchestrator carve-out event recorded in icm.
- **Maps to spec criterion:** A7 (deferred session-mode tracking).

### P10 — Authoritative-tracked repo-walk audit
- **Inputs/preconditions:** P1–P7 merged.
- **Symbols to claim:** Scaffold-claim per ADR-0054 for `crates/oya-foundry-fitness-authoritative-tracked-kernel/`.
- **Outputs / acceptance evidence:** `oya-foundry-fitness-authoritative-tracked-kernel` lane green; no `.gitignored` paths hold authoritative state.
- **Linus data-shape reshape:** "authoritative state spread across tracked-and-ignored paths" → "authoritative ≡ tracked, lane-enforced."
- **Maps to spec criterion:** A8. Citation: ADR-0054.

### Final integration
- A9 verified by Critic via direct file-path check at line 3 of this plan; status = **SATISFIED**.
- A10 PR body "good-taste audit" enumerates five reshape items; empty = fail.

---

## 3. ADR Block — single source of truth

Per Architect revision #8 and Critic finding #1: the canonical ADR lift-sources are:

- **ADR-0052 (inventory)** ← inventory enumeration; lands as `oyatie/docs/decisions/ADR-0103-grit-cutover-inventory.md` at P1.
- **ADR-0053 (sanctioned primitives + cutover direction)** ← lands as `oyatie/docs/decisions/ADR-0053-grit-icm-as-sanctioned-primitives.md` at P5 alongside banned-primitives lane activation. Required edits: add bootstrap-window clause to §Decision, add human-orchestrator definition to §Decision Drivers/§Glossary, add cutover-is-one-time-not-retroactive clause to §Consequences §Neutral, unify §Follow-ups.
- **ADR-0054 (scaffold-claim pattern)** ← `pre-cutover-drafts.md §Draft 2`; lands as `oyatie/docs/decisions/ADR-0054-grit-scaffold-claim-pattern.md` at P0.5. Document `Cargo.toml::workspace_members` as NOT viable; adopt icm-coordination-lock fallback as canonical.

ADR slot numbers verified: highest existing is ADR-0051; 0052/0053/0054 are free.

---

## 4. Verification Matrix

| Spec criterion | Phase(s) | Verification command / lane |
|---|---|---|
| A1 — Bidirectional PRD citation | P3 + P3.5 | `cargo run -p oya-foundry-fitness-portfolio-citation-kernel` green on both sides; foundry-corpus cross-cite asserted |
| A2 — Inventory ledger committed | P1 (+ P3.5 phantom-path correction) | ADR-shape lane green; every row's classification ∈ closed set; `archived_at` column present; phantom path corrected; referenced from `ADR-INDEX.md` |
| A3 — Orchestration glue archived + deleted | P6 (archive) + P7 (delete) | `grit symbols` shows no active orchestration-glue paths; archive dir contains the moved set; three concrete gates green at P7 |
| A4 — `oya-tooling-agent-read` helper shipped | P2 | `cargo test -p oya-tooling-agent-read` green; audit-chain query shows invocations; mutation attempts fail closed; scaffold-claim per ADR-0054 |
| A5 — Agent-facing memory rewritten + lane active | P4 (scaffold) + P5 (activate) | `oya-foundry-fitness-banned-primitives` lane green from P5 forward |
| A6 — Hook + skill audit | P5 | `oyatie/docs/AGENT-INSTRUCTION-SOURCES.md` enumerates every agent-instruction-bearing file; each has a passing audit row |
| A7 — Parallel-claim demo (session-less, single-file) | P8 (+ P9 follow-up for session mode) | `docs/runbooks/agentic-pipeline/grit-parallel-claim-demo.md` reproducible; pinned to billing-app symbols |
| A8 — Authoritative artifacts repo-tracked | P10 | `oya-foundry-fitness-authoritative-tracked-kernel` lane green; scaffold-claim per ADR-0054 |
| A9 — Spec-to-plan handoff path | this document (line 3) | **SATISFIED** — Critic verified via direct file-path check at iter-1 review |
| A10 — Linus-style audit | Final integration PR template | PR body "good-taste audit" section populated with five reshape items; empty = fail |

---

## 5. Status footer

Status: **Accepted — pending execution approval**

Iteration: 2 — incorporates Architect (4 violations V1–V4 + 8 revisions) + Critic (4 additional findings + P3.5 refinement) + foundry-salvage output + orchestrator existence findings + open-questions mechanical resolutions (Q1/Q2/Q3/Q4/Q5/Q6). Cross-cutting amendments in `cross-cutting-amendments.md`.

Phase count: 12 (P0.5, P1, P2, P3, P3.5, P4, P5, P6, P7, P8, P9, P10).

**A9 attestation:** A9 verified by Critic via direct file-path check at line 3 of this plan; status = SATISFIED.

**Two flagged open-questions noted but NOT blocking approval:**
- Q3 (carve-out scope): proceed under "humans orchestrating cutover may invoke git/gh"; ADR-0053 landing requires explicit user confirmation.
- Q6 (retention policy): adopt 90 days; user may amend at ADR landing.
