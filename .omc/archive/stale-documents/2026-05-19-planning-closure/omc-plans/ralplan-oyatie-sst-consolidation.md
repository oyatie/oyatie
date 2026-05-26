---
purpose: Auto-backfilled purpose for ralplan-oyatie-sst-consolidation.md
---

# RALPLAN — oyatie Single Source of Truth + grit/icm Cutover

Phase-0 input: `/Users/jasonlee/oyatie/.omc/scratch/deep-dive-oyatie-sst-consolidation.md`
Trace synthesis: `/Users/jasonlee/oyatie/.omc/scratch/deep-dive-trace-oyatie-sst-consolidation.md`
Mode: `ralplan --consensus --direct --deliberate-auto` (deliberate engaged — multi-repo cutover, deletion path, agent-rule rewrite)
Iteration: **Iteration 2 — incorporates Architect (4 violations + 8 revisions) + Critic (4 additional findings) + foundry-salvage output + orchestrator existence findings**

---

## 1. RALPLAN-DR Summary

### Principles
- **Inventory precedes deletion.** No artifact leaves the active path without an ADR row classifying it (`KEEP|REPLACE-WITH-*|ARCHIVE|DELETE`). Per spec §Constraints item 3.
- **Sanctioned primitives are a closed set: `{grit, icm, oya-tooling-agent-read}`** — post-cutover. During the **cutover bootstrap window (P0.5–P2 inclusive)** the set is `{grit, icm}` plus an audited carve-out for new-crate scaffolding (`icm-coordination-lock`) and human-orchestrator `git mv`/`git rm`/`gh issue create`. Banned-primitives fitness lane activates at **P5 merge**. Per spec §Constraints item 1.
- **Authoritative = repo-tracked.** ICM may mirror cross-project memory, but project-canonical decisions land in `oyatie/docs/` as tracked files. Per spec §Constraints item 2.
- **Reshape data to eliminate special cases.** `G004-reconciliation-blocker.md`, `PAUSE.md`, omx checkpoint flow vanish because grit's data model has no place for them — not because we add a shim. Per spec §Constraints item 7.
- **Parallel agent work is the first-class default, with scaffold-claim fallback.** Every phase enumerates real `file::Identifier` claims for grit-indexed symbols, or — for new-crate phases (P2, P3, P10) and doc-only phases — coordinates via the `scaffold-locks-oyatie` icm topic per **ADR-0054 grit-scaffold-claim-pattern** (lift-source `pre-cutover-drafts-2026-05-12.md §Draft 2`). Verified: `grit symbols | grep Cargo.toml` returns zero matches, so the `Cargo.toml::workspace_members` primary option in Draft 2 is not viable; the icm-coordination-lock fallback is the canonical path. Per spec §Constraints item 5.

### Decision Drivers
1. **Reversibility of the cutover** — archive-before-delete keeps `git revert` cheap; collapsing inventory and deletion into one PR makes rollback expensive.
2. **Agent-flow correctness during the cutover** — agents executing the plan are themselves bound by the new rule, so `oya-tooling-agent-read` must exist before agent-instruction rewrites force its use, AND the bootstrap-window meta-contradiction (P1–P2 cannot enforce a rule that names a primitive it is concurrently constructing) must be acknowledged in the ADR.
3. **bominal↔oyatie boundary integrity** — A1 bidirectional citation must land early so subsequent doc edits cannot accidentally re-orphan the boundary; the foundry corpus in `bominal/agents/ultragoal/` (9 KEEP-classified files, ~245KB of spec content) must be cross-cited into `oyatie/docs/products/foundry/PHASE-00-SPEC.md` (NEW) before any archive/delete-adjacent operation.

### Viable Options (sequencing strategies; the *what* is fixed by spec)

**Option A — Strict-phased, archive-first (RECOMMENDED).** Scaffold-claim ADR lands P0.5; inventory ADR lands P1; `oya-tooling-agent-read` ships P2 (under scaffold-claim) before any agent-instruction rewrite; bidirectional citation P3; foundry-corpus cross-cite P3.5; doc/hook rewrites P4–P5 on the new primitives; archive P6 with archive-orphan lane scaffolded; delete P7 only after archive PR has merged and three concrete gates green; demo and audits P8–P10. Multiple PRs gated on prior-phase merge.
- Pros:
  - Each phase has a clean rollback boundary; revert one PR, the rest still hold.
  - `oya-tooling-agent-read` exists by the time agents are told to use it, so the rewrite is enforceable on day one.
  - Inventory row is committed *before* anything moves, satisfying §Constraints item 3 literally.
  - Scaffold-claim ADR (P0.5) resolves the chicken-and-egg before P2/P3/P10.
- Cons:
  - Longest wall-clock to "done" — 12 sequential merge gates.
  - Cross-phase coordination overhead via icm locks for doc-only phases and scaffold windows.
  - Reviewer fatigue across many small PRs.

**Option B — Single-PR cutover.** One mega-PR carries the inventory ADR, helper CLI, citation, doc rewrites, archive moves, deletions, demo, and audit lanes together. Reviewable as one atomic unit.
- Pros:
  - One reviewer pass, one merge, atomic semantics.
  - No inter-phase icm-lock coordination needed.
  - Demo runs against the final shape directly.
- Cons:
  - Rollback is "revert everything" — catastrophic if any one subset is wrong.
  - Violates spec §Constraints item 3 in spirit: inventory and deletion ship in the same commit, so the "ledger precedes deletion" temporal ordering is fictive.
  - PR diff is too large for grit's parallel-claim model to demonstrate — agents would step on each other.

**Option C — Archive-and-delete-in-parallel batches.** Inventory pass P1 standalone; subsequent phases bundle archive+delete per logical batch (orchestration glue as one batch, agent-instruction rewrite as another, etc.); each batch is its own PR but combines archive+delete rather than splitting them.
- Pros:
  - Half the PR count of Option A; faster than A, less risky than B.
  - Each batch is internally coherent (one concern per PR).
  - Still preserves inventory-first.
- Cons:
  - Archive→delete same-PR weakens recoverability: if the deletion was wrong, the archive line in the same diff often isn't enough to recover the original semantics (paths, intent metadata).
  - Conflicts with §A3 verification ("`grit symbols` shows no active orchestration-glue paths; archive directory contains the moved set") which implies temporally distinct states.
  - Demo and audit lanes still need their own phases at the end.

**Recommended: Option A (strict-phased, archive-first).** Cost in wall-clock is the price for clean rollback boundaries and literal compliance with §Constraints item 3. The cutover is once-only and the wall-clock difference is days, not weeks; reversibility is worth far more than that.

### Pre-mortem (deliberate mode)

| # | Failure scenario | Probability × Impact | Mitigation |
|---|---|---|---|
| 1 | `oya-tooling-agent-read` ships in P2 but lacks a primitive an agent actually needs mid-plan (e.g., `git blame` for a regression hunt). Agents either silently fall back to `git` or stall. | Medium × High | P2 acceptance gates on a fixed read-set from spec §A4 (`log`, `diff`, `pr-view`, `pr-comments`). Explicit escape hatch with **30-minute resume window**: missing-primitive → agent `icm store -t agent-read-missing-primitive -c "<verb> needed for <task>" -i high` and HALT; human orchestrator within 30 minutes either extends `oya-tooling-agent-read` (lands as inventory-ledger follow-up row) or grants a one-time bootstrap carve-out logged via `icm store -t cutover-orchestrator-actions -c '<action>' -i critical`. Banned-primitives lane (§A5) catches any silent fallback once it activates at P5 merge. |
| 2 | Archive directory `archive/pre-grit-cutover-2026-05-12/` is staged but the deletion PR (P7) is merged before all consumers of those paths have been re-wired, causing a partial-rewrite breakage in CI. | Medium × Medium | Gate P7 on **three concrete checks** (per Architect revision #3): (a) banned-primitives lane green on main HEAD post-P6 merge, (b) NEW `oya-foundry-fitness-archive-orphan` lane scaffolded at P6 confirms no living code/docs/configs reference archived paths, (c) inventory ledger's per-row `archived_at` timestamp is non-null for every ARCHIVE-class row. All three must be green at the merge commit. |
| 3 | The grit 0.3.0 `grit session start` bug widens to break `grit claim` or `grit done` mid-cutover, stalling parallel agents. | Low × High | Plan uses session-less mode by default (per spec §Technical Context: agents claim on `main`, auto-worktree, `grit done` lands). P9 files the upstream bug. Fallback: human-orchestrator sequential mode for any phase that cannot progress, documented in `docs/runbooks/agentic-pipeline/grit-parallel-claim-demo.md`. |

### Expanded test plan (deliberate mode)

- **Unit.** `oya-tooling-agent-read` ships with unit tests per subcommand in `tools/oya-tooling-agent-read/tests/` (Rust `cargo test`, per Q1 resolution: Rust is the workspace idiom, audit-chain kernel reuse is direct, distribution is static). Each subcommand has at least one happy-path test (read succeeds, audit event emits) and one fail-closed test (mutation attempt rejected). Verification path: `cargo test -p oya-tooling-agent-read` green in CI.
- **Integration.** Banned-primitives fitness lane (`oya-foundry-fitness-banned-primitives`) runs `rg` against an explicit file enumeration (see P5 deliverable below) for the tokens `rtk git`, `rtk gh`, `\bgit\b`, `\bgh\b` inside agent-instruction sections (HTML-comment-fence-scoped per `pre-cutover-drafts §Draft 6`). Zero hits required. Lane scope is also extended at P5 implementation time to grep for stray `archive/pre-grit-cutover-2026-05-12/` path tokens from active paths (per Q4 resolution). ADR-shape lane (existing `oya-foundry-fitness-cohesion`) confirms the inventory ADR has a classification value from the closed set on every row. Portfolio-citation lane (`oya-foundry-fitness-portfolio-citation`) runs on both bominal and oyatie sides AND covers the foundry-corpus cross-cite from P3.5. Archive-orphan lane (`oya-foundry-fitness-archive-orphan`) runs at P6→P7. Verification path: `cargo run -p oya-foundry-fitness-banned-primitives` (and siblings) green in CI.
- **End-to-end.** Parallel-claim demo at `docs/runbooks/agentic-pipeline/grit-parallel-claim-demo.md` (NEW subdir, per orchestrator existence findings) records two agents claiming non-overlapping symbols **within the same file** — `crates/oya-cloud-billing-app/src/lib.rs::CloudBillingEventIngestAppStatus` and `crates/oya-cloud-billing-app/src/lib.rs::CloudBillingMeterUnitRecord` (both grit-verified-indexed per Q5; pre-cutover-drafts §Draft 3 is the runbook seed). Demonstrates per-symbol locking within a single file, satisfying A7's non-overlapping-symbols criterion. Demo is executable (`bash docs/runbooks/agentic-pipeline/grit-parallel-claim-demo.sh` or equivalent) and produces a deterministic transcript. Verification path: run the demo script in CI's e2e job; transcript byte-equivalence (modulo timestamps) required.
- **Observability.** Each `oya-tooling-agent-read` invocation emits an audit-chain event (per spec §A4); audit-chain query confirms invocation count matches the demo's known-good count. `grit watch` event excerpts included in the demo runbook. Each cutover-bootstrap-window human-orchestrator carve-out emits an `icm store -t cutover-orchestrator-actions ... -i critical` event BEFORE execution (per Architect revision #5). Authoritative-tracked lane (`oya-foundry-fitness-authoritative-tracked`) walks the repo and confirms every file referenced as authoritative in `docs/AGENTS.md` is tracked (not `.gitignored`). Verification path: audit-chain query + lane output captured in P10 acceptance evidence.

---

## 2. Phased Plan

**Total phases: 12** — P0.5, P1, P2, P3, P3.5, P4, P5, P6, P7, P8, P9, P10.

### P0.5 — Land ADR-0054 grit-scaffold-claim-pattern + human-orchestrator definition
- **Inputs/preconditions:** Architect verdict ITERATE consumed; Critic findings consumed; lift-source `.omc/scratch/adr-draft-grit-icm-sanctioned-primitives.md` exists; `.omc/scratch/pre-cutover-drafts-2026-05-12.md §Draft 2` exists; `oyatie/docs/RACI-OWNERSHIP.md` exists (70 rows, verified).
- **Symbols to claim:** doc-only — coordinate via `icm-lock-p0.5` topic. The scaffold-claim ADR is the canonical resolution of the new-crate chicken-and-egg; landing it BEFORE P2 means P2 can cite a sanctioned pattern rather than improvise.
- **Agents and parallelism:** 1 agent drafts ADR-0054; 1 agent appends a "human orchestrator" row to `docs/RACI-OWNERSHIP.md` naming the cutover orchestrator role + the icm-event-before-execution requirement. Both edits can proceed in parallel via per-file icm sub-locks.
- **Outputs / acceptance evidence:** (a) `oyatie/docs/decisions/ADR-0054-grit-scaffold-claim-pattern.md` exists, lifted from `pre-cutover-drafts-2026-05-12.md §Draft 2`. ADR body documents primary option (`Cargo.toml::workspace_members`) as **NOT viable** (`grit symbols | grep Cargo.toml` returns zero — verified) and adopts the icm-coordination-lock fallback as the canonical scaffold-claim path: `icm store -t scaffold-locks-oyatie -c "agent=<id> path=<new-path> window=open started_at=<ts>" -i critical -k "scaffold-lock,<crate-name>,open"`, scaffold, `grit init` re-index, then normal `grit claim`. (b) `oyatie/docs/RACI-OWNERSHIP.md` has a new row for "human orchestrator (cutover-2026-05-12)" with `icm store -t cutover-orchestrator-actions -c '<action>' -i critical` listed as the pre-execution requirement for each carve-out. (c) `ADR-INDEX.md` appended.
- **Maps to spec criterion:** A2 (prerequisite for inventory cleanliness); unblocks A4/A1/A8 by giving P2/P3/P10 a sanctioned new-crate path.

### P1 — Inventory pass (no deletions yet)
- **Inputs/preconditions:** P0.5 merged (so the inventory ADR can cite ADR-0054 for any new-crate row's scaffold-claim path). Spec + trace files exist; current grit installation v0.3.0; `.grit/` populated.
- **Symbols to claim:** doc-only — coordinate via `icm-lock-p1` topic. Parallel agents produce per-directory classification rows in separate icm topics; a single author merges into the ADR.
- **Agents and parallelism:** Up to 4 agents in parallel for inventory enumeration. Scope per agent (revised per orchestrator existence findings): `oyatie/**` excluding agents-config; `bominal/agents/`; `bominal/docs/`; and a fourth agent on **agent-instruction scope inside the repo** — specifically `**/*.md`, `**/*.json`, and `**/.claude/**` paths (since `oyatie/agents/settings/` and `bominal/agents/settings/` are verified NOT present). One agent serializes the merge into the ADR.
- **Outputs / acceptance evidence:** `/Users/jasonlee/oyatie/docs/decisions/ADR-0103-grit-cutover-inventory.md` (verified next free slot, per `open-questions-resolutions-2026-05-12.md §Q2`). Every file/dir/script in scope has one row with classification ∈ `{KEEP, KEEP+ANNOTATE, REPLACE-WITH-GRIT, REPLACE-WITH-ICM, REPLACE-WITH-HELPER, ARCHIVE, DELETE}` AND a `archived_at` timestamp column (null until P6 stamps it for ARCHIVE rows). Referenced from `ADR-INDEX.md`. **Phantom-path correction**: the row for `oyatie/.omx/ultragoal/` is marked `phantom — not present, no action` (was line 473 of inventory-draft saying "DELETE if discovered"; updated per Critic finding). `.codex/worktree_init.sh` is **dropped from the deletion list entirely** (verified non-existent; orchestrator existence findings). Helper-crate-target row reads `tools/oya-tooling-agent-read/ — REPLACE-WITH-HELPER` (name reconciled per Critic #2).
- **Maps to spec criterion:** A2.

### P2 — Ship `tools/oya-tooling-agent-read/` (under scaffold-claim pattern per ADR-0054)
- **Inputs/preconditions:** P0.5 merged (ADR-0054 in place); P1 merged (inventory row exists for `tools/oya-tooling-agent-read/` as `REPLACE-WITH-HELPER`).
- **Symbols to claim:** **Scaffold-claim per ADR-0054** (lift-source `pre-cutover-drafts-2026-05-12.md §Draft 2`; primary `Cargo.toml::workspace_members` NOT viable, verified). Sequence: (1) lead agent `icm store -t scaffold-locks-oyatie -c "agent=<id> path=tools/oya-tooling-agent-read window=open started_at=<ts>" -i critical -k "scaffold-lock,oya-tooling-agent-read,open"`; (2) other agents `icm recall -t scaffold-locks-oyatie -k "open"` and back off; (3) lead agent scaffolds `tools/oya-tooling-agent-read/{Cargo.toml, src/main.rs, src/lib.rs, src/cli.rs, src/commands/{log,diff,pr_view,pr_comments}.rs, src/audit.rs, tests/*}` (Rust per Q1); (4) appends `tools/oya-tooling-agent-read` to root `Cargo.toml [workspace] members`; (5) `grit init` re-index; (6) `icm store ... window=closed`; (7) subsequent agents use normal `grit claim file::Identifier` against the new crate's now-indexed symbols (e.g., `tools/oya-tooling-agent-read/src/commands/log.rs::run`, `::diff::run`, `::pr_view::run`, `::pr_comments::run`, `src/audit.rs::emit_event`).
- **Agents and parallelism:** 1 lead agent owns scaffold-claim window; after window closes, 3 agents in parallel — one per command pair (log+diff, pr-view+pr-comments, audit+cli wiring) — using normal grit claims on now-indexed symbols. One agent gates on the others for the integration tests.
- **Outputs / acceptance evidence:** `tools/oya-tooling-agent-read/` directory; `cargo test -p oya-tooling-agent-read` green; an invocation `oya-tooling-agent-read log 5` succeeds and emits an audit event; an attempted `oya-tooling-agent-read commit` (or any non-listed subcommand) fails closed with non-zero exit. Scaffold-lock window closed in icm with `started_at`/`finished_at` timestamps.
- **Maps to spec criterion:** A4. Citation: ADR-0054 (P0.5) and `pre-cutover-drafts-2026-05-12.md §Draft 2`.

### P3 — Bidirectional PRD citation + portfolio-citation fitness lane (under scaffold-claim pattern per ADR-0054)
- **Inputs/preconditions:** P0.5 merged (ADR-0054 cite-able); P1 merged (inventory classifies the citation as `KEEP+ANNOTATE` on both sides AND classifies the new fitness-lane crate as `REPLACE-WITH-GRIT`).
- **Symbols to claim:** **Two-track**. Track 1 (doc edits): `icm-lock-p3-docs` — two PRD files in different repos, can proceed concurrently. Track 2 (new fitness-lane crate `oya-governance-portfolio-citation-kernel`): **scaffold-claim per ADR-0054** (same sequence as P2 — icm-coordination-lock under `scaffold-locks-oyatie`, scaffold `crates/oya-governance-portfolio-citation-kernel/{Cargo.toml, src/lib.rs}` and lane runner at `tools/oya-foundry-fitness-portfolio-citation/`, append to workspace, `grit init`, close window). After window closes, normal grit claims against `crates/oya-governance-portfolio-citation-kernel/src/lib.rs::verify` etc.
- **Agents and parallelism:** Track 1: 2 agents in parallel — one edits `oyatie/docs/PRD.md` adding citation to `bominal/docs/consolidated/PRD.md`; the other edits `bominal/docs/consolidated/PRD.md` adding citation to `oyatie/docs/PRD.md` as canonical implementation home. Track 2: 1 lead agent owns scaffold-claim window for the lane crate; 1 follow-up agent implements kernel logic on now-indexed symbols (lifted from `pre-cutover-drafts-2026-05-12.md §Draft 5`).
- **Outputs / acceptance evidence:** Both PRD files cross-cite; `cargo run -p oya-governance-portfolio-citation-kernel` (or lane runner) green; lane wired into the foundry fitness manifest. Scaffold-lock window closed in icm.
- **Maps to spec criterion:** A1. Citation: ADR-0054 (P0.5).

### P3.5 — Cross-cite ultragoal foundry corpus into canonical foundry SPEC
- **Inputs/preconditions:** P1 merged (foundry corpus rows in inventory classified as KEEP; phantom-path row for `oyatie/.omx/ultragoal/` correctly marked); foundry-salvage output exists at `.omc/scratch/foundry-salvage-from-ultragoal-2026-05-12.md` (22KB, verified). `oyatie/docs/products/foundry/PRD.md` exists (75.5KB, verified). `oyatie/docs/products/foundry/PHASE-00-SPEC.md` does NOT exist (verified — new file target).
- **Symbols to claim:** doc-only — `icm-lock-p3.5` topic. New file landing is doc-only (no new crate); scaffold-claim ADR-0054 fallback applies for the new docs path: `icm store -t scaffold-locks-oyatie -c "agent=<id> path=docs/products/foundry/PHASE-00-SPEC.md window=open" -i critical` BEFORE creation; close window after.
- **Agents and parallelism:** 1 agent owns scaffold-lock and lands `oyatie/docs/products/foundry/PHASE-00-SPEC.md` containing Phase 00 contract surface (`ProviderAccount`, `AuthSession`, `UsageWindow`, `SecretReference`, `ProviderFamily` allowlist, state machine `Draft→Verified→Active→Degraded→Disabled→Revoked`, provider-gateway parity, secret handling, P00-01..P00-08 acceptance gates — all per foundry-salvage §A–E). 1 agent in parallel edits `oyatie/docs/products/foundry/PRD.md` to add cross-cite block referencing the three bominal source files: `bominal/agents/ultragoal/2026-05-12-foundry-ultragoal-mega-plan.md`, `bominal/agents/ultragoal/foundry-agentic-substrate-master.md`, `bominal/agents/ultragoal/oyatie-product-delivery-implementation-plan.md` (cite-only; original sources stay KEEP-classified). 1 agent extends `oya-foundry-fitness-portfolio-citation` lane (from P3) OR scaffolds a sibling `oya-foundry-fitness-foundry-corpus-citation` lane to verify the cross-cite. Inventory ledger updated with the phantom-path correction (`oyatie/.omx/ultragoal/ — phantom, not present, no action`).
- **Outputs / acceptance evidence:** (i) `oyatie/docs/products/foundry/PHASE-00-SPEC.md` exists with Phase 00 contract surface inlined from foundry-salvage §A–E. (ii) `oyatie/docs/products/foundry/PRD.md` cross-cites the three bominal source files. (iii) `oya-governance-portfolio-citation-kernel` (or sibling lane) asserts the foundry-corpus cross-cite exists. (iv) Inventory ledger row for `oyatie/.omx/ultragoal/` reads `phantom — not present, no action`. Scaffold-lock window closed in icm.
- **Linus data-shape reshape:** "Foundry-canonical claims in bominal mega-plan without oyatie-side cite" → "every foundry-canonical claim has a tracked oyatie-side cite or landing."
- **Maps to spec criterion:** A1 (cross-cite extends to foundry corpus), A2 (inventory phantom-path correction), A8 (authoritative ≡ tracked — Phase 00 contract surface now lives on oyatie side as tracked file).
- **Deadline:** BEFORE P4 (per Critic refinement: "BEFORE P1 inventory PR" is too aggressive; the 9 foundry files are KEEP not deletion targets, so foundry-side cross-cite can run on P1+P3 outputs but MUST land before agent-facing-memory rewrite in P4 references the foundry SPEC path).

### P4 — Rewrite agent-facing memory
- **Inputs/preconditions:** P2 merged (rewrite must point agents at `oya-tooling-agent-read`, which must exist). P1 merged (inventory classifies the affected files). P3.5 merged (foundry SPEC landing path is known so agent docs can reference it correctly).
- **Symbols to claim:** doc-only — `icm-lock-p4`. Files: `oyatie/CLAUDE.md`, `oyatie/AGENTS.md`, `oyatie/docs/AGENTS.md`. Each is a distinct file; up to 3 agents in parallel via icm-lock-p4-<filename> sub-topics.
- **Agents and parallelism:** 3 agents in parallel (one per file). Each removes agent-instruction references to `rtk git`, `rtk gh`, bare `git`, bare `gh`. Each adds or links to the "Sanctioned Primitives" section naming `{grit, icm, oya-tooling-agent-read}` and citing ADR-0053 (post-cutover steady-state) and ADR-0054 (scaffold-claim pattern for new-crate phases).
- **Outputs / acceptance evidence:** `oya-foundry-fitness-banned-primitives` lane scaffolded (not yet enforcing — activates at P5 merge per ADR-0053 bootstrap-window clause). Human-facing terminal usage of `rtk git`/`rtk gh` remains documented in user-facing sections (per spec §Non-Goals: human usage unaffected). HTML-comment-fence convention `<!-- agent-instructions:start -->` / `<!-- agent-instructions:end -->` introduced (lift from `pre-cutover-drafts §Draft 6`).
- **Maps to spec criterion:** A5.

### P5 — Hook + skill audit (banned-primitives lane activates here)
- **Inputs/preconditions:** P4 merged.
- **Symbols to claim:** doc/config-only — `icm-lock-p5`. **Critic finding #3 resolution**: explicit file enumeration. Since `oyatie/agents/settings/` and `bominal/agents/settings/` are verified NOT present (orchestrator existence findings), A6's enforcement scope is **option (a) — all `**/*.md`, `**/*.json`, and `**/.claude/**` paths inside the repo that contain agent-instruction sections** (identified by the `<!-- agent-instructions:start -->`/`<!-- agent-instructions:end -->` fences introduced in P4). The existing `grit-claim-state-on-stop` Stop hook (lives at user-machine `~/.claude/projects/...` path per orchestrator existence findings — NOT inside the repo) stays per spec §A6 but is out-of-repo and audited via inventory ledger reference row only. The P5 deliverable is therefore: produce an enumeration document `oyatie/docs/AGENT-INSTRUCTION-SOURCES.md` listing every file inside the repo that contains agent-instruction sections, audited row-by-row in the inventory ledger.
- **Agents and parallelism:** N agents in parallel, one per touched file, coordinated via per-file icm-lock sub-topics. The audit row in the inventory ledger is updated by a single serializing agent at the end.
- **Outputs / acceptance evidence:** (a) `oyatie/docs/AGENT-INSTRUCTION-SOURCES.md` enumerates every agent-instruction-section-bearing file in the repo. (b) Each touched file appears in the inventory ledger with a passing audit row (classification + rewrite-verified flag). (c) `oya-foundry-fitness-banned-primitives` lane **activates** (per ADR-0053 bootstrap-window clause): token-grep against agent-instruction sections returns zero hits for banned tokens. (d) Lane scope extended at implementation time to grep for stray `archive/pre-grit-cutover-2026-05-12/` references from active paths (per Q4 resolution).
- **Linus data-shape reshape (Architect rev #7):** "scattered git/gh references in agent skills across N files" → "single grit/icm/oya-tooling-agent-read invocation pattern, lane-enforced from this phase forward."
- **Maps to spec criterion:** A5 (lane activation), A6 (hook+skill audit).

### P6 — Archive (NOT delete) orchestration glue + scaffold archive-orphan lane
- **Inputs/preconditions:** P1, P2, P3.5, P4, P5 merged. Banned-primitives lane green on main HEAD.
- **Symbols to claim:** **Two-track**. Track 1 (`git mv`): `icm-lock-p6-archive` — operation is a `git mv` of the listed paths to `bominal/agents/ultragoal/archive/pre-grit-cutover-2026-05-12/`. **Human-orchestrator carve-out** (per ADR-0053 §Consequences §Neutral, per RACI-OWNERSHIP.md row landed in P0.5): the move is a `git mv` invocation by a *human orchestrator* — agents do not invoke `git`. Each carve-out invocation requires `icm store -t cutover-orchestrator-actions -c '<action>' -i critical` BEFORE execution. Track 2 (new lane `oya-foundry-fitness-archive-orphan`): **scaffold-claim per ADR-0054** for `crates/oya-foundry-fitness-archive-orphan-kernel/`; after window closes, normal grit claim against `crates/oya-foundry-fitness-archive-orphan-kernel/src/lib.rs::check`.
- **Agents and parallelism:** Track 1: 1 agent prepares the move manifest and emits the pre-execution icm event; the human orchestrator runs the moves; 1 agent verifies post-move state via `oya-tooling-agent-read` and `grit symbols`; 1 agent stamps `archived_at` timestamp on every ARCHIVE-class row in the inventory ledger. Track 2: 1 lead agent owns scaffold-claim window for the lane crate; 1 follow-up agent implements kernel (greps active paths for references to `archive/pre-grit-cutover-2026-05-12/` tokens and confirms zero hits except deprecation notice).
- **Outputs / acceptance evidence:** (a) `bominal/agents/ultragoal/archive/pre-grit-cutover-2026-05-12/{ledger.jsonl, goals.json, codex-goal-*.json, G004-reconciliation-blocker.md, PAUSE.md, goals.before-stale-*.json, final-readiness-*.json, implementation-docs-*.json}` exists (file list per foundry-salvage §I "Items NOT to migrate"). (b) Deprecation notice for `omx ultragoal checkpoint/complete-goals` lands in `bominal/agents/ultragoal/DEPRECATED.md` citing `grit done --agent <id>` + `icm store -t context-oyatie` as the successor (body per `pre-cutover-drafts-2026-05-12.md §Draft 4`). (c) `oya-foundry-fitness-archive-orphan-kernel` scaffolded and green. (d) Inventory ledger: every ARCHIVE-class row has non-null `archived_at` timestamp. (e) `icm recall -t cutover-orchestrator-actions` shows the pre-execution event for each `git mv` invocation.
- **Maps to spec criterion:** A3 (archive half). Citation: ADR-0053 §Consequences §Neutral (cutover bootstrap carve-out), ADR-0054 (lane scaffold-claim), RACI-OWNERSHIP.md (human orchestrator row).

### P7 — Delete archived glue from active path; three concrete gates
- **Inputs/preconditions:** P6 merged AND **three concrete gates all green** (per Architect revision #3, replacing prior "two consecutive CI green runs" unfalsifiable gate):
  1. `oya-foundry-fitness-banned-primitives` lane green on main HEAD post-P6 merge.
  2. `oya-foundry-fitness-archive-orphan` lane (scaffolded in P6) green: confirms no living code/docs/configs reference archived paths from active paths (deprecation notice at `bominal/agents/ultragoal/DEPRECATED.md` is the only sanctioned reference).
  3. Inventory ledger's per-row `archived_at` timestamp is **non-null for every ARCHIVE-class row** (auditable via inventory-ledger query: `rows where classification=ARCHIVE AND archived_at IS NULL` returns zero).
- **Symbols to claim:** doc-only — `icm-lock-p7`. Operation is `git rm` of the original active paths (the archived copies survive at the new location). Same human-orchestrator carve-out as P6 — `icm store -t cutover-orchestrator-actions -c '<action>' -i critical` BEFORE each `git rm` invocation.
- **Agents and parallelism:** 1 agent prepares delete-list and emits pre-execution icm events; human orchestrator runs the deletions; 1 agent verifies via `grit symbols` (no active orchestration-glue paths) and re-runs both fitness lanes.
- **Outputs / acceptance evidence:** `grit symbols` shows no active orchestration-glue paths in `bominal/agents/ultragoal/` (only `archive/pre-grit-cutover-2026-05-12/` retains them). Banned-primitives lane green. Archive-orphan lane green. `icm recall -t cutover-orchestrator-actions` shows pre-execution events.
- **Maps to spec criterion:** A3 (delete half).

### P8 — Parallel-claim demo runbook (symbols pinned to Draft 3)
- **Inputs/preconditions:** P2, P7 merged (so the demo runs against the final-shape repo).
- **Symbols to claim:** **Pinned per Architect revision #6, Q5 resolution, `pre-cutover-drafts §Draft 3`**: `crates/oya-cloud-billing-app/src/lib.rs::CloudBillingEventIngestAppStatus` and `crates/oya-cloud-billing-app/src/lib.rs::CloudBillingMeterUnitRecord`. Both are verified grit-indexed (`grit symbols | grep CloudBilling...` returned both). Both are in the same file, different identifiers — demonstrates per-symbol locking within a single file, satisfying A7's non-overlapping-symbols criterion.
- **Agents and parallelism:** 2 agents in parallel for the demo itself (`agent-A` claims `CloudBillingEventIngestAppStatus`, `agent-B` claims `CloudBillingMeterUnitRecord`); 1 agent records the transcript into `docs/runbooks/agentic-pipeline/grit-parallel-claim-demo.md` (NEW subdir per orchestrator existence findings). Session-less mode (agents claim on `main`, auto-worktree, `grit done` lands).
- **Outputs / acceptance evidence:** `docs/runbooks/agentic-pipeline/grit-parallel-claim-demo.md` with timestamps, `grit watch` event excerpts, and reproducible script (seeded from `pre-cutover-drafts-2026-05-12.md §Draft 3`). Demo executable in CI. Includes negative case: a third agent attempting to claim a symbol already locked by `agent-A` queues or errors as expected.
- **Maps to spec criterion:** A7. Citation: `pre-cutover-drafts-2026-05-12.md §Draft 3`.

### P9 — File upstream `grit session start` bug
- **Inputs/preconditions:** None — can run in parallel with any later phase. Source draft at `pre-cutover-drafts-2026-05-12.md §Draft 1`.
- **Symbols to claim:** doc-only — `icm-lock-p9`. The bug file is filed at `rtk-ai/grit` upstream (an external repo), so the agent's deliverable is the local artifact: `docs/runbooks/grit-session-bug-upstream.md` capturing the reproducer (`git checkout -b failed: 'grit/<name>' is not a commit`), the root-cause hypothesis (missing default for source-ref in session-start codepath), and the expected fix shape. **Human-orchestrator carve-out** (per ADR-0053 §Consequences §Neutral): filing the upstream GitHub issue requires `gh issue create` or the web UI — agents cannot do this; the human orchestrator files it, with `icm store -t cutover-orchestrator-actions -c 'gh issue create rtk-ai/grit session-start-bug' -i critical` BEFORE execution. The agent produces the reproducer document only.
- **Agents and parallelism:** 1 agent writes the local artifact (body per Draft 1); human orchestrator files the upstream issue and updates the artifact with the issue URL.
- **Outputs / acceptance evidence:** `docs/runbooks/grit-session-bug-upstream.md` with upstream issue URL. Session-mode demo follow-up tracked in ADR-0053 Follow-up list. `icm recall -t cutover-orchestrator-actions` shows pre-execution event.
- **Maps to spec criterion:** A7 (deferred-session-mode tracking).

### P10 — Authoritative-tracked repo-walk audit (under scaffold-claim pattern per ADR-0054)
- **Inputs/preconditions:** P1–P7 merged (so the repo is in final shape).
- **Symbols to claim:** **Scaffold-claim per ADR-0054** for new fitness-lane crate `crates/oya-governance-authoritative-tracked-kernel/`: lead agent `icm store -t scaffold-locks-oyatie -c "agent=<id> path=crates/oya-governance-authoritative-tracked-kernel window=open" -i critical`; scaffold; append to workspace; `grit init`; close window. After window closes, normal grit claims against `crates/oya-governance-authoritative-tracked-kernel/src/lib.rs::check` etc. Plus `icm-lock-p10-docs` for any companion doc updates.
- **Agents and parallelism:** 1 lead agent owns scaffold-claim window; 1 agent walks `docs/AGENTS.md` enumerating every file referenced as authoritative; 1 agent audits `.gitignore` to confirm none of those files are ignored; 1 agent implements the lane kernel on now-indexed symbols. Lane wired into the foundry fitness manifest.
- **Outputs / acceptance evidence:** `oya-governance-authoritative-tracked-kernel` lane green; any `.gitignored` paths that held authoritative state are either committed or demoted (with the demotion logged in the inventory ledger). Scaffold-lock window closed in icm.
- **Linus data-shape reshape (Architect rev #7):** "authoritative state spread across tracked-and-ignored paths" → "authoritative ≡ tracked, lane-enforced."
- **Maps to spec criterion:** A8. Citation: ADR-0054 (P0.5).

### Final integration: spec-to-plan handoff verification
This document at `/Users/jasonlee/oyatie/.omc/plans/ralplan-oyatie-sst-consolidation.md` references `.omc/scratch/deep-dive-oyatie-sst-consolidation.md` as Phase-0 input (line 3) and routes execution to autopilot/team via parallel-grit-claim batches. **A9 verified by Critic via direct file-path check at line 3 of this plan during iter-1 review; status = SATISFIED** (per Critic finding #4).

The cutover PR body's "good-taste audit" section (A10) lives in the final integration PR template — not a discrete phase. It enumerates (a) special cases eliminated (no `G004-reconciliation-blocker.md` because grit's data model has no objective-state to mismatch; no `PAUSE.md` because grit has no PAUSE verb — release-or-TTL is the model), (b) deep hierarchies flattened (`bominal/agents/ultragoal/` glue collapses to grit primitives + icm topics), (c) ceremony deleted (omx ultragoal checkpoint/complete-goals retired), (d) **P5 data-shape reshape** ("scattered git/gh references in agent skills" → "single grit/icm/oya-tooling-agent-read invocation pattern"), (e) **P10 data-shape reshape** ("authoritative state spread across tracked-and-ignored paths" → "authoritative ≡ tracked"). Empty section = fail.

---

## 3. ADR Block — single source of truth

Per Architect revision #8 and Critic finding #1 (resolving ADR Follow-up divergence between plan and pre-draft): **the inline ADR text previously in this section is DELETED**. The canonical ADR lift-sources are:

- **ADR-0052 (inventory)** ← `.omc/plans/deep-dive-inventory-draft.md` (inventory enumeration; lands as `oyatie/docs/decisions/ADR-0103-grit-cutover-inventory.md` at P1).
- **ADR-0053 (sanctioned primitives + cutover direction)** ← `.omc/scratch/adr-draft-grit-icm-sanctioned-primitives.md` (lands as `oyatie/docs/decisions/ADR-0053-grit-icm-as-sanctioned-primitives.md` at P5 alongside banned-primitives lane activation; iter-2 edits to lift-source: add bootstrap-window clause to §Decision, add human-orchestrator definition to §Decision Drivers/§Glossary, add cutover-is-one-time-not-retroactive clause to §Consequences §Neutral, unify §Follow-ups with this plan's prior §3 follow-ups).
- **ADR-0054 (scaffold-claim pattern)** ← `.omc/scratch/pre-cutover-drafts-2026-05-12.md §Draft 2` (lands as `oyatie/docs/decisions/ADR-0054-grit-scaffold-claim-pattern.md` at P0.5; iter-2 edit: document `Cargo.toml::workspace_members` as NOT viable per verified `grit symbols` output, adopt icm-coordination-lock fallback as canonical).

Per Critic finding #1: ADR-0053 §Follow-ups must be unified with the prior plan §3 follow-ups (5 vs 5 different items) at the iter-2 edit to the lift-source. The unified list lives in `adr-draft-grit-icm-sanctioned-primitives.md`, not here, to enforce single source of truth.

ADR slot numbers verified against `oyatie/docs/decisions/` per `open-questions-resolutions-2026-05-12.md §Q2`: highest existing is ADR-0051; 0052/0053/0054 are free.

---

## 4. Verification Matrix

| Spec criterion | Phase(s) | Verification command / lane |
|---|---|---|
| A1 — Bidirectional PRD citation | P3 + P3.5 | `cargo run -p oya-governance-portfolio-citation-kernel` green on both sides; foundry-corpus cross-cite asserted |
| A2 — Inventory ledger committed | P1 (+ P3.5 phantom-path correction) | ADR-shape lane green; every row's classification ∈ closed set; `archived_at` column present; phantom path corrected; referenced from `ADR-INDEX.md` |
| A3 — Orchestration glue archived + deleted | P6 (archive) + P7 (delete) | `grit symbols` shows no active orchestration-glue paths; archive dir contains the moved set; three concrete gates green at P7 (banned-primitives + archive-orphan + `archived_at` non-null) |
| A4 — `oya-tooling-agent-read` helper shipped | P2 | `cargo test -p oya-tooling-agent-read` green; audit-chain query shows invocations; mutation attempts fail closed; scaffold-claim per ADR-0054 |
| A5 — Agent-facing memory rewritten + lane active | P4 (scaffold) + P5 (activate) | `oya-foundry-fitness-banned-primitives` lane green from P5 forward (zero hits in agent-instruction sections; HTML-comment-fence-scoped) |
| A6 — Hook + skill audit | P5 | `oyatie/docs/AGENT-INSTRUCTION-SOURCES.md` enumerates every agent-instruction-bearing file; each has a passing audit row in inventory ledger; banned-primitives lane still green |
| A7 — Parallel-claim demo (session-less, single-file) | P8 (+ P9 follow-up for session mode) | `docs/runbooks/agentic-pipeline/grit-parallel-claim-demo.md` reproducible; pinned to `CloudBillingEventIngestAppStatus` + `CloudBillingMeterUnitRecord`; recorded transcript matches |
| A8 — Authoritative artifacts repo-tracked | P10 | `oya-governance-authoritative-tracked-kernel` lane green; scaffold-claim per ADR-0054 |
| A9 — Spec-to-plan handoff path | this document (line 3) | **SATISFIED** — Critic verified via direct file-path check at iter-1 review |
| A10 — Linus-style audit | Final integration PR template | PR body "good-taste audit" section populated with five reshape items (special cases + hierarchies + ceremony + P5 reshape + P10 reshape); empty = fail |

---

## 5. Status footer

Status: **pending approval**

Iteration: 2 — incorporates Architect (4 violations V1–V4 + 8 revisions) + Critic (4 additional findings + P3.5 refinement) + foundry-salvage output + orchestrator existence findings + open-questions mechanical resolutions (Q1/Q2/Q3/Q4/Q5/Q6).

Phase count: 12 (P0.5, P1, P2, P3, P3.5, P4, P5, P6, P7, P8, P9, P10).

Next reviewers: Architect (iter-2 structural soundness check, all 12 revisions accounted for) then Critic (iter-2 gap closure, ADR source-of-truth unified). On both green, hand off to `/autopilot` or `/team` for execution under the grit `claim → work → done` lifecycle.

**A9 attestation (per Critic finding #4):** A9 verified by Critic via direct file-path check at line 3 of this plan; status = SATISFIED.

**Two flagged open-questions noted but NOT blocking iter-2 approval** (per Critic):
- Q3 (carve-out scope): proceed under "humans orchestrating cutover may invoke git/gh"; ADR-0053 landing requires explicit user confirmation.
- Q6 (retention policy): adopt 90 days (resolutions-doc recommendation) in ADR-0053 §Follow-ups; user may amend.
