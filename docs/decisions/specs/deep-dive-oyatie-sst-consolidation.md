---
doc_status: deprecated-wrong-location
canonical_at: docs/specs/deep-dive-oyatie-sst-consolidation.md
deprecation_note: |
  This copy is in the wrong location. docs/decisions/ is for ADRs only, not
  specs. The canonical spec lives at docs/specs/deep-dive-oyatie-sst-consolidation.md.
  This copy diverged from canonical (~127 lines added / ~122 removed / ~51 modified)
  per PRs-12-18 multispectrum review (pr-15-A2-r1.json, pr-15-A3-r1.json).
  Filed as F-SST-TRIPLICATE-DEDUP-CONTENT-MERGE in registries/cross-cutting/
  fixuptasks.jsonl to merge unique content back into canonical and delete
  this copy. Until that closes: this file is read-only; new content goes to
  canonical.
---
# Deep-Dive Spec: oyatie Single Source of Truth + grit/icm agentic pipeline

<!--
status: Accepted
date: 2026-05-12
related_adrs: ADR-0052, ADR-0053, ADR-0054, ADR-0055
-->

Captured 2026-05-12. Output of `/deep-dive` Phase 4 crystallization. Feeds `/ralplan --consensus --direct` then `/autopilot` or `/team` under the grit `claim → work → done` lifecycle.

## Goal

Establish two cleanly-separated layers as oyatie's single source of truth, with every authoritative artifact tracked in the repo and every agent operation routed through `grit` (coordination) and `icm` (memory) without direct `git` or `gh` invocation.

**Layer 1 — Product-content SoT.** `oyatie/docs/` (CONSTITUTION → PRD → DESIGN → SPEC → ROADMAP → ADRs) is the canonical product authority. `bominal/docs/consolidated/PRD.md` is acknowledged as the portfolio-parent document via a bidirectional citation. The 2026-05-09 reframing (Workspace as Axis 2, Builder-OS → Foundry, in-house model substrate) is propagated to all stale ULTRAGOAL artifacts. The four OPEN ledger entries (LEDG-008, LEDG-017, LEDG-021, LEDG-024) remain open on their existing resolution batches — the direction shift is orthogonal to them.

**Layer 2 — Agentic-pipeline contract.** `grit` is the sole sanctioned primitive for agent state transitions (`claim --agent --intent`, `worktree`, `done`, `session`, `assign`, `heartbeat`, `gc`). `icm` is the sole sanctioned primitive for cross-session knowledge (`recall`, `recall-context`, `store`, `update`, `health`). For read-side operations grit does not natively cover (commit history, branch diffs, PR comments), a thin sanctioned read-only helper CLI (`agent-read`) wraps `git`/`gh` for agents — these helpers are the *only* non-grit/icm path agents may invoke. Direct agent-side `git` and `gh` are banned. The current orchestration glue (`bominal/agents/ultragoal/{ledger.jsonl, goals.json, codex-goal-*.json, G004-*, PAUSE.md}`, `omx ultragoal checkpoint/complete-goals`, `.codex/worktree_init.sh`, RTK `git`/`gh` references in agent-facing memory) is archived and deleted.

## Constraints

1. **Sanctioned primitives are `grit` + `icm` + `agent-read` only.** Any new agent-callable tool must justify itself against "why doesn't `grit` or `icm` already do this." Agent-side `git`/`gh` is banned.
2. **Every authoritative artifact is repo-tracked.** Working-state directories (`.omx/`, `.omc/state/`, equivalents) that hold session ephemera stay .gitignored; anything that any agent or human treats as authoritative for "what is true" must be committed. ICM external storage is acceptable for cross-project memory but MUST NOT be the only home of a project-canonical decision; project-canonical decisions land in `oyatie/docs/` as tracked files.
3. **Inventory before deletion.** No file or script is deleted until it appears in the inventory ledger at `oyatie/docs/decisions/registry/placeholder-debt/adr-follow-ups.yaml#grit-cutover-inventory (superseded by ADR-0116)` (or equivalent) with classification: `KEEP` / `REPLACE-WITH-GRIT` / `REPLACE-WITH-ICM` / `REPLACE-WITH-HELPER` / `ARCHIVE` / `DELETE`. The inventory pass covers `/Users/jasonlee/oyatie/**` (excluding `target/`, `node_modules/`, `.git/`) and the bominal surfaces the cutover touches (`/Users/jasonlee/bominal/agents/`, `/Users/jasonlee/bominal/docs/`).
4. **Clean Architecture dependency direction is preserved.** `kernel ← domain ← app ← {api, worker, adapter} ← runtime` per the flat-crates ADR-0015 target. Any new crate introduced by the cutover names itself `oyatie-<context>-<role>[-<capability>]` and respects the dependency direction.
5. **Parallel agent work is first-class.** The cutover and the subsequent `/ralplan`-driven implementation are expected to run multiple agents in parallel. Every agent claims its symbols with `grit claim --agent X --intent "…"` before any edit; `--with-deps` is used wherever the work crosses callee boundaries. `grit session` carves the multi-agent feature-branch space.
6. **No git/gh shortcuts in CI either.** CI flows that previously invoked `gh pr create` or `git rebase --autosquash` for agent-authored work route through the same sanctioned helpers; the merge-gate hook validates that the PR's authoring sequence references a `grit done --agent` event rather than a manual rebase.
7. **Linus-style discipline.** Delete bureaucracy that hides bad data structures. Eliminate special cases by reshaping the data, not by adding shims. Flat structure > deep hierarchy when the deep one is ceremony. "Good taste" means the simplest representation that handles all cases without branching. No half-finished implementations.
8. **bominal-to-oyatie boundary is explicit.** `oyatie/docs/PRD.md` cites `bominal/docs/consolidated/PRD.md` as portfolio parent; `bominal` references `oyatie` as the canonical implementation home for the flat-catalog product. Cross-cite enforcement lands as a new fitness lane: `governance-portfolio-citation`.
9. **All four OPEN ledger entries stay open.** Direction shift does not force-close LEDG-008, LEDG-017, LEDG-021, LEDG-024. They continue on their existing resolution-batch ownership.

## Non-Goals

- Re-scoping the flat-catalog EaaS product frame. The product definition survives the direction shift unchanged.
- Re-decomposing the foundry kernels. The 7 suspect fitness/policy kernels (`claim-ceiling`, `authority-cohesion`, `bypass`, `pr-traceability`, `pre-push`, `quality-lane`, `cohesion-fitness`) are not coordination kernels; they govern product-quality and survive.
- Reversing ADR-0025 (Builder-OS → Foundry consolidation) or any other 2026-05-09 reframing decision.
- Rewriting `~/.claude/CLAUDE.md` (user-machine config). The agentic-pipeline rules land in `oyatie/CLAUDE.md` and `oyatie/AGENTS.md` only, unless the user explicitly broadens the rule.
- Wrapping `gh` for *humans* on the terminal — RTK's `rtk gh`/`rtk git` savings are still fine for human terminal usage. The ban applies to agent execution paths.
- Re-implementing grit or icm primitives inside oyatie. We integrate the upstream tools; we do not fork them.

## Acceptance Criteria

The spec is complete when all of the following hold; each criterion has a typed verification path.

1. **A1 — Bidirectional PRD citation.** `oyatie/docs/PRD.md` cites `bominal/docs/consolidated/PRD.md` as portfolio parent; bominal cites oyatie as canonical implementation home. *Test*: new fitness lane `governance-portfolio-citation` passes on both sides.
2. **A2 — Inventory ledger committed.** `oyatie/docs/decisions/registry/placeholder-debt/adr-follow-ups.yaml#grit-cutover-inventory (superseded by ADR-0116)` (or equivalent) exists, classifies every file/dir/script under the inventory scope, and is referenced from `ADR-INDEX.md`. *Test*: ADR shape lane + every entry has a classification value from the closed set.
3. **A3 — Orchestration glue archived.** `bominal/agents/ultragoal/{ledger.jsonl, goals.json, codex-goal-*.json, G004-reconciliation-blocker.md, PAUSE.md}` moved to `bominal/agents/ultragoal/archive/pre-grit-cutover-2026-05-12/` then removed from the active path. `omx ultragoal checkpoint/complete-goals` flow is retired with a deprecation notice citing `grit done` + `icm store -t context-oyatie` as the successor. *Test*: `grit symbols` shows no active orchestration-glue paths; archive directory contains the moved set.
4. **A4 — `agent-read` helper shipped.** A thin read-only CLI exposing `agent-read log <N>`, `agent-read diff <ref1> <ref2>`, `agent-read pr-view <num>`, `agent-read pr-comments <num>`. Read-only by construction; emits an audit-chain event per invocation. *Test*: invocation count appears in audit-chain query; mutation attempts (anything not in the read set) fail closed.
5. **A5 — Agent-facing memory rewritten.** `oyatie/CLAUDE.md` and `oyatie/AGENTS.md` (and the `docs/AGENTS.md` they redirect to) remove every agent-instruction reference to `rtk git`, `rtk gh`, `git`, `gh`. The "sanctioned primitives" section names `grit`, `icm`, `agent-read`. *Test*: a grep lane (`governance-banned-primitives`) for the banned tokens in agent-instruction sections returns zero hits.
6. **A6 — Hook + skill audit.** `agents/settings/claude.settings.json` and any project-level skill that previously called `git`/`gh` is rewritten to route through grit + icm + `agent-read`. The existing `grit-claim-state-on-stop` Stop hook stays. *Test*: each touched file has a passing audit row in the inventory ledger.
7. **A7 — Parallel-claim demo (session-less mode).** A reproducible demo proves N>1 agents can `grit claim` non-overlapping symbols, work in their auto-created `.grit/worktrees/<agent>/` directories in parallel, and land via `grit done --agent <agent>` without conflict. *Test*: a recorded session in `docs/runbooks/grit-parallel-claim-demo.md` shows the sequence with timestamps and `grit watch` event excerpts. Session-mode demo (`grit session start` + `grit session pr`) is scheduled-for-distinct-tracked-work until upstream session bug fix; tracked as a successor-IP runbook.
8. **A8 — All authoritative artifacts repo-tracked.** A repo-walk audit confirms that every file referenced as authoritative in `docs/AGENTS.md` is tracked. `.gitignored` paths that house authoritative state are either committed or demoted to non-authoritative. *Test*: `governance-authoritative-tracked` lane.
9. **A9 — Spec-to-plan handoff path.** `/ralplan --consensus --direct .omc/scratch/deep-dive-oyatie-sst-consolidation.md` produces a plan at `.omc/plans/` that names this spec as Phase 0 input and routes execution to autopilot/team with parallel-grit-claim batches. *Test*: plan file exists and references this spec by path.
10. **A10 — Linus-style audit.** The cutover PR body section "good-taste audit" enumerates: (a) the special cases eliminated by reshaping data (e.g., `G004-reconciliation-blocker.md` no longer needs to exist), (b) the deep hierarchies flattened, (c) the ceremony deleted. Empty section is a fail. *Test*: PR template gate.

## Assumptions Exposed

1. **`grit` upstream is stable for the primitives we depend on** (`claim`, `worktree`, `done`, `session`, `assign`, `heartbeat`, `gc`, `watch`, `symbols`, `status`). Version 0.3.0 confirmed installed. If a primitive misbehaves, escalation is an upstream issue, not an oyatie patch.
2. **`icm` upstream is stable** for `recall`, `recall-context`, `store -t -c -i -k`, `update`, `health`, `topics`. The user is already actively using icm (we stored 5 records this session).
3. **`agent-read` is implementable as a thin wrapper.** No exotic functionality — it shells out to read-only `git`/`gh` invocations and emits audit events.
4. **Bominal-to-oyatie boundary is parent-child, not peer.** Bominal owns portfolio; oyatie owns product. Confirmed by the PRD-SoT decision.
5. **The 2026-05-09 reframing is intended to stick.** Workspace as Axis 2, Builder-OS folded into Foundry, in-house model substrate added. The cutover does not revisit these.
6. **The seven `foundry-*-kernel` fitness/policy crates are correctly scoped.** Their continued existence is assumed; this spec does not propose merging or splitting them.
7. **CI pipeline can be migrated incrementally** — agent flows route through grit before human flows are forced; human terminal usage of `rtk git`/`rtk gh` is unaffected.
8. **ICM external storage is acceptable for cross-project memory** but project-canonical authority lives in `oyatie/docs/` as tracked files (per Constraint 2).
9. **Parallel-claim demo will use ≥2 agents on non-overlapping symbols** — this is the canonical demonstration that the new pipeline preserves the parallelism the user explicitly requested.
10. **The PR-merge flow under `grit done` produces something semantically equivalent to a GitHub PR.** If `grit done` only does local merge today, the helper layer extends to `agent-write pr-finalize <session>` as a one-call atom that creates the PR record without exposing `gh` to agents.

## Technical Context

### Repo topology (after cutover)

```
oyatie/
  Cargo.toml                       # flat-crates workspace, 140+ crates, unchanged
  crates/                          # kernel ← domain ← app ← {api, worker, adapter} ← runtime
  docs/                            # canonical product authority (CONSTITUTION, PRD, DESIGN, SPEC, ADRs)
  contracts/                       # per-cross-microservice contract files (OpenAPI/Proto/AsyncAPI)
  registry/      # catalog + capability records
  scripts/                         # build/lint/release helpers (humans + sanctioned CI)
  tools/agent-read/            # NEW: sanctioned read-only helper CLI
  .grit/                           # grit local state (already exists)
  .omc/                            # OMC plans + state (session-scoped; .gitignored for state subdirs)
  .omx/                            # working state ONLY; nothing authoritative
  CLAUDE.md, AGENTS.md             # Redirect-class files pointing to docs/AGENTS.md
  README.md
bominal/
  docs/consolidated/PRD.md         # portfolio parent PRD; cites oyatie as canonical impl home
  agents/ultragoal/                # planning corpus (active artifacts only post-cutover)
  agents/ultragoal/archive/pre-grit-cutover-2026-05-12/   # archived glue (ledger.jsonl etc.)
```

### Agent flow (canonical sequence — session-less mode)

```
1. grit claim --agent <id> --intent "<one-line>" <file::symbol>...
   # grit auto-creates a per-agent worktree at .grit/worktrees/<id>/
   # symbols must be real indexed code symbols (file::Identifier);
   # markdown/text files are NOT in the symbol index — claim them via file-path locks (future grit feature) or coordinate via icm
2. <agent works inside .grit/worktrees/<id>/>
3. agent reads via: grit symbols | grit status | grit watch | icm recall-context | agent-read log/diff/pr-view
4. agent writes via: file edits inside its worktree (grit symbol-locks prevent cross-agent overwrite)
5. icm store -t {decisions-oyatie|context-oyatie|errors-resolved|preferences} per CLAUDE.md mandate
6. grit done --agent <id>        # rebase + merge worktree back to base + release locks in one atomic step
7. (optional, when grit session bug is fixed upstream) grit session pr   # creates PR for the session
```

**Known constraint — grit 0.3.0 session bug.** `grit session start` fails on this repo state with `git checkout -b failed: 'grit/<name>' is not a commit`. Root cause appears to be a missing default for the source-ref argument in grit's session-start codepath. Operating under **session-less mode** until upstream fix: agents claim symbols directly on `main` (or whatever the orchestrator's current base branch is), grit auto-worktrees per agent, `grit done` lands the worktree back to base. The `grit session pr` PR-creation primitive is **scheduled-for-distinct-tracked-work** until the upstream session bug ships a fix; in the interim, PR creation routes through `agent-read`'s sibling write-helper or remains a human-orchestrator-only step. File the bug upstream at `rtk-ai/grit` as part of A6 deliverables.

No agent step calls `git` or `gh` directly.

### Inventory classification scheme

Each row in the inventory ledger uses one of:

- `KEEP` — survives unchanged
- `KEEP+ANNOTATE` — survives; needs cross-cite or metadata added
- `REPLACE-WITH-GRIT` — function absorbed by a grit primitive; original deleted
- `REPLACE-WITH-ICM` — function absorbed by icm topic; original deleted
- `REPLACE-WITH-HELPER` — read-side function moved into `agent-read`
- `ARCHIVE` — moved to `archive/pre-grit-cutover-2026-05-12/` and removed from active path; recoverable
- `DELETE` — removed; not recoverable except via git history

## Ontology

| Entity | Stable definition | Where it lives |
|---|---|---|
| **Oyatie** | One cohesive ecosystem-as-a-service across all microservices (SaaS, Workspace, Vertical, Foundry, Cloud, Search, Ads + Analytics). Single product. | `oyatie/docs/PRD.md` (canonical) ← `bominal/docs/consolidated/PRD.md` (portfolio parent) |
| **Foundry** | Axis 4: AI agent runtime + engineering platform + control plane. Unified per ADR-0025 (2026-05-09). Multi-provider adapter (Claude/OpenAI/Gemini, plus future in-house). | `oyatie/docs/DESIGN.md §3` |
| **grit (rtk-ai/grit)** | Upstream coordination CLI. Agents claim symbols, work in worktrees, release via `done`. Symbol-locking guarantees merge-conflict-free parallel agent work. | Installed at `~/.cargo/bin/grit v0.3.0`. Local state in `.grit/`. |
| **icm (rtk-ai/icm)** | Upstream persistent-memory CLI. Topic-partitioned, importance-tagged, recall via keyword search. Survives session compaction. | External storage; project-canonical decisions duplicated into `oyatie/docs/` per Constraint 2. |
| **agent-read** | NEW thin sanctioned helper. Read-only wrapper over `git`/`gh` for operations grit doesn't cover. Audit-emitting. | `tools/agent-read/` (to be created). |
| **Sanctioned primitive** | A tool agents may invoke. The set is exactly `{grit, icm, agent-read}`. Adding to the set requires an ADR. | `oyatie/docs/AGENTS.md` |
| **Orchestration glue** | Pre-cutover artifacts (`ledger.jsonl`, `goals.json`, `codex-goal-*.json`, `omx ultragoal checkpoint`, `.codex/worktree_init.sh`, agent-facing RTK references) that duplicate grit/icm primitives. Deletion target. | `bominal/agents/ultragoal/archive/pre-grit-cutover-2026-05-12/` after cutover |
| **Inventory ledger** | The ADR + tracked table that classifies every file/dir/script in scope before any deletion happens. | `oyatie/docs/decisions/registry/placeholder-debt/adr-follow-ups.yaml#grit-cutover-inventory (superseded by ADR-0116)` |
| **Parallel-claim demo** | A reproducible recording proving N>1 agents can work simultaneously on non-overlapping symbols and land via `grit done` without conflict. | `oyatie/docs/runbooks/grit-parallel-claim-demo.md` |

## Ontology Convergence

The trace surfaced one ontological gap: Lane 3 enumerated five direction-shift dimensions (sequencing, taxonomy, axis count, regional, compliance) but the user's actual shift was a sixth — **agentic-pipeline mechanism**. The spec adopts "agentic-pipeline mechanism" as a first-class dimension and names it explicitly.

Lane 1 and Lane 2 had one factual disagreement (whether `oyatie/docs/PRD.md` exists). Lane 1's direct citation wins; Lane 2's structural-boundary thesis stands but the example is voided. The ontology resolves: oyatie/docs/PRD.md is canonical, bominal/docs/consolidated/PRD.md is portfolio parent.

No remaining entity-stability issues. The 7-axis EaaS frame is stable across all corpora and survives the direction shift.

## Trace Findings

**Leader hypothesis**: SoT-ownership / orchestration (Hypothesis 1, confidence High). The "no single source of truth" pain is structurally an ownership/orchestration boundary problem layered on a real-but-tracked contradiction backlog. The major direction shift absorbs the agentic-orchestration layer into upstream tools; product content survives untouched.

**Per-lane critical unknowns resolved**:
- Lane 1 (was the 2026-05-09 reframing Council-ratified): scheduled-for-distinct-tracked-work — spec assumes ratified per Constraint 5; ADR check is part of inventory pass.
- Lane 2 (is oyatie sovereign or downstream of bominal): RESOLVED — oyatie sovereign, bominal portfolio parent, bidirectional cite required.
- Lane 3 (what is the direction shift): RESOLVED — agentic-pipeline mechanism, sanctioned primitives `{grit, icm, agent-read}`, agent-side `git`/`gh` banned.

**Evidence that shaped the interview**:
- Foundry kernel inspection showed the suspect `foundry-*-kernel` crates are fitness/policy kernels, not coordination — they survive. The deletion target is the orchestration glue layer, not the foundry crates.
- `grit status` showed expired stale claims in bominal — exactly the failure mode `grit gc` handles natively, validating the "don't reinvent" thesis.
- The published `bominal/docs/consolidated/PRD.md` and the existing `oyatie/docs/PRD.md` use identical flat-catalog language but did not cross-cite. Bidirectional citation closes that gap without merging the two.

**Trace path**: `docs/decisions/specs/deep-dive-trace-oyatie-sst-consolidation.md`

**Related ADRs**: ADR-0052 (inventory ledger), ADR-0053 (sanctioned primitives), ADR-0054 (scaffold-claim pattern), ADR-0055 (cross-cutting amendments).

## Interview Transcript

User signals captured across the session, in order:

1. (Initial goal): "/ralplan plan and implement oyatie as described. Use grit for full workflow from claim, work, done. /ultrawork or /batch when sensible. Begin with /deep-dive — major shift in direction after consolidation. I want a single source of truth."
2. "Major shift in direction of the project so discuss with me in detail. I want you to come up with agentic development implementation plan."
3. "Because of grit, we no longer have to over engineer the agentic pipeline of foundry (merge conflict no longer possible as long as agents follow grit protocol)."
4. "We can implement what works from rtk-ai/grit and rtk-ai/icm to our pipeline."
5. "that means agents should not use git at all. or gh. everything is through grit pipeline."
6. "Make sure all the files, directories, and scripts are accounted for."
7. (AskUserQuestion answers): PRD = oyatie/docs/PRD.md canonical; agent reads = icm + sanctioned helpers; ledger = all four open; inventory scope = "repo files directories scripts. source of truth and all files must live inside repo as tracked artifact."
8. "We can also work in parallel in clean architecture. structured and organized. How linus torvald would approach."

Final ambiguity ≤ 20%. Spec gated on `/ralplan --consensus --direct` next.
