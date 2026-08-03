# RALPLAN — Full Replan Before Fanout for Oyatie Ultragoal

Intended save path: `.omx/plans/ralplan-complete-ultragoal-through-team-ledger-full-replan.md`  
Supersedes: `.omx/plans/ralplan-complete-ultragoal-through-team-ledger-20260626T203000Z.md`  
Created: 2026-06-26  
Mode: full-replan-first planning; no source implementation or Team fanout.

**Source of truth used:** `AGENTS.md`, `docs/AGENTS.md`, `specs/root-hub-pointers.json`, `specs/master-plan-sequencing.json`, `specs/markdown-retirement-policy.json`, `docs/decisions/ADR-0363-retire-agentic-vcs-platform-to-intelligence-on-github-substrate.md`, `docs/decisions/ADR-0515*.md` (current CI authority), `docs/decisions/ADR-0513*.md` (historical/background only if needed), `.omx/specs/deep-interview-complete-ultragoal-through-team-ledger.md`, `.omx/context/complete-ultragoal-through-team-ledger-20260626T220541Z.md`, `.omx/ultragoal/{brief.md,goals.json,ledger.jsonl}`, prior plan above.

**Current state evidence:**  
- `git status --short --branch` → `dev...origin/dev [behind 202]`, dirty with tracked/untracked changes.  
- `git worktree list` → many stale/local worktrees present.  
- `gh pr list --base dev --state open` → `[]`.  
- `get_goal` → `null` (no active Codex goal).  
- Local `.omx/state/team/` has no live team directories.  
- `.omx/ultragoal/goals.json` → `G001` and `G015` are `in_progress`; `G002-G018` pending.  
- User deep-interview decisions: `full_replan_first`; no extra non-goals.

---

## 1) Requirements summary

Replan the entire Ultragoal backlog before any new execution fanout. The plan must:

- regenerate the backlog ordering and explain why the next execution unit is **G002 intake**, not immediate G015 relaunch;
- define a full dependency graph for **G001/G002-G018**;
- define disjoint Team lanes, ownership, avoid-paths, acceptance criteria, verification, and shutdown gates;
- preserve repo authority from `AGENTS.md`, `docs/AGENTS.md`, `specs/root-hub-pointers.json`, `specs/master-plan-sequencing.json`, `specs/markdown-retirement-policy.json`, and ADR-0363/0513/0515;
- respect the dirty leader checkout and stale worktrees by forbidding source edits here;
- keep `.omx/ultragoal` leader-owned; workers do not mutate goal state;
- include the full lifecycle gate chain: spec → plan → task breakdown → TDD/incremental implementation → API/interface design as needed → observability → CI/CD → security/performance → ai-slop-cleaner → independent code-reviewer + architect → ponytail-review/code-simplification → UltraQA → shipping/launch → Ultragoal checkpoint.

---

## 2) RALPLAN-DR summary

### Principles
1. **Integrity before throughput.** Parallelism only counts if lanes are disjoint and checkpointable.
2. **Intake before fanout.** A current collision map beats optimistic worker launch.
3. **Leader-owned durable state.** `.omx/ultragoal` and checkpointing stay with the leader; workers return evidence only.
4. **Hot surfaces are serial.** Generated files, root authority specs, and workflow/config surfaces cannot be shared casually.
5. **Evidence closes loops.** Every wave needs tests, review, CI, merge, and fresh goal reconciliation.

### Top 3 decision drivers
1. Current leader checkout is dirty and 202 commits behind `origin/dev`; stale worktrees are abundant.
2. The existing plan over-centered G015 too early; the safer first move is G002 trunk/intake plus collision control.
3. The backlog is large enough to justify parallelism, but only after the plan establishes disjoint ownership and shutdown gates.

### Viable options
#### Option A — Continue with G015 as the immediate execution wave
**Pros:** keeps momentum; aligns with existing in-progress wave label.  
**Cons:** still risks stale-team resurrection, ignores the dirty checkout/collision problem, and does not refresh intake first.  
**Verdict:** reject.

#### Option B — Replan to G002 first, then reopen G015 as a re-scoped Team wrapper
**Pros:** safest; respects the dirty leader checkout, stale worktrees, and current PR vacuum; creates a current collision map before fanout.  
**Cons:** adds one upfront planning/intake step.  
**Verdict:** **chosen**.

#### Option C — Collapse to one serial owner for everything
**Pros:** lowest coordination risk.  
**Cons:** defeats the stated throughput goal and wastes the existing disjoint-wave structure.  
**Verdict:** fallback only for hot/shared surfaces.

---

## 3) ADR

### Decision
Adopt **Option B**: make **G002** the next execution unit, pause/re-scope **G015** as the Team wrapper for disjoint foundation slices after intake, and treat the rest of the backlog as staged waves with explicit serial hot-file control.

### Drivers
- Dirty, behind leader checkout.
- Many stale worktrees.
- No open PRs, so a clean restart is available.
- No active Codex goal to reconcile.
- User explicitly chose full replan before fanout.

### Alternatives considered
- Immediate G015 fanout.
- Single-owner sequential execution.
- Direct implementation from the prior plan.

### Why chosen
It is the smallest plan that actually reduces merge risk without abandoning the throughput objective.

### Consequences
- G002 becomes the first execution gate.
- G015 is not discarded; it is re-scoped after G002 into a fresh, validated Team wave.
- Serial/hot surfaces are explicitly protected from worker overlap.
- Later waves can still parallelize once intake and ownership are stable.

### Follow-ups
- After plan approval, leader refreshes intake and collision mapping.
- The G002 intake package must write a concrete collision map at `.omx/team-ledgers/g002-intake-collision-map.md` and a Team launch ledger at `.omx/team-ledgers/g015-wave-a-team-ledger.md` before any broad Team fanout.
- Then leader launches the next Team wave from fresh `origin/dev` worktrees only.
- Fresh `get_goal` snapshots are required before any checkpoint claim.

---

## 4) Milestone dependency graph  
**Inference:** `goals.json` does not declare explicit `dependsOn` edges, so the graph below is inferred from goal titles/objectives, the brief, and current repo state.

**Wrapper-state rule:** `G015` may remain the active Ultragoal wrapper in `goals.json`, but the next execution unit is `G002`; no Team fanout happens through `G015` until G002 intake/collision control is current.

```text
G001 (aggregate umbrella; not a separate execution wave)
└─ G002 (intake / trunk / PR / worktree / collision control)
   ├─ G003 (repo hygiene/runtime-state)
   ├─ G004 (universal cloud-ci substrate)
   ├─ G005 (generated-artifact conflict elimination)
   └─ G006 (Rust purity + Python/MJS retirement)

G015 (Wave A Team wrapper)
└─ re-scoped around G003-G006 after G002; not a standalone implementation lane

G007 (shared cloud infrastructure)
└─ can begin after G004/G005/G006 are stable enough to freeze contracts

G008 (platform substrate)
└─ depends on G007

G009 (core capability substrate)
└─ depends on G008

G010 (communication/collaboration)
└─ depends on G009

G011 (distribution/enterprise SaaS)
└─ depends on G010

G012 (cross-cutting production bar)
└─ runs continuously across G004-G014; must exist before launch

G013 (strangler migration)
└─ depends on G007-G011 and G012

G014 (launch readiness)
└─ depends on G012 + G013

G016 (Rust kernel/OS substrate)
G017 (AST/transpiler/program-analysis substrate)
└─ later-stage long-horizon lanes; parallel with each other only when they do not touch shared hot surfaces; schedule after the earlier product/foundation waves are stable

G018 (hyperscaler process / closed-loop review)
└─ loops across all waves; feeds the next planning cycle; not terminal
```

### Parallel vs sequence rules
- **Must sequence:** `G002` before any broad fanout; `G007 -> G008 -> G009 -> G010 -> G011`; `G013` after the earlier product substrate; `G014` after production bar + strangler readiness.
- **Can parallelize after G002:** `G003`, `G004`, `G005`, `G006` if lane ownership is disjoint.
- **Can parallelize later:** internal sub-slices inside `G007-G011`, and `G016` with `G017` only if they remain isolated from each other and from hot/shared files.
- **Continuous gate:** `G012` and `G018`.

---

## 5) Lane plan before fanout

### Lane 0 — Intake / backlog / collision control
**Owner:** leader only  
**Owned surfaces:** `.omx/ultragoal/{brief.md,goals.json,ledger.jsonl}`, `git worktree list`, PR inventory, stale lane inventory.  
**Output paths:** `.omx/team-ledgers/g002-intake-collision-map.md` and `.omx/team-ledgers/g015-wave-a-team-ledger.md` before Team launch.  
**Avoid:** all source edits; all generated files.  
**Depends on:** none.  
**Acceptance:** current collision map, stale worktree list, PR list, and next-wave ordering are recorded.  
**Verification:** `git status --short --branch`, `git worktree list`, `gh pr list --base dev --state open`.

### Lane 1 — Repo hygiene / runtime boundaries
**Owner:** executor + debugger, with verifier signoff  
**Owned surfaces:** scratch/runtime drift, invalid hook output, GraphQL residue, root dirt that is not canonical.  
**Avoid:** root authority specs, generated outputs, CI workflow files.  
**Depends on:** G002.  
**Acceptance:** dirt is categorized as authoritative / ignored runtime / stale scratch; valid fences or deletions are backed by evidence.  
**Verification:** targeted grep/smoke checks; zero hand-edited generated files.

### Lane 2 — Universal cloud-ci product boundary
**Owner:** executor + architect + test-engineer  
**Owned surfaces:** `cloud/**`, `crates/**` cloud-ci API/data-pack seams, Rust gate/result packet boundaries.  
**Avoid:** `.github/workflows/**`, `oya-ci.toml`, root hub specs, generated artifacts.  
**Depends on:** G002 and stable intake map.  
**Acceptance:** one small universal/productized boundary improvement or a precise evidence ledger if the code path is not safe to change yet.  
**Verification:** targeted Buck2/Rust tests; no Cargo-only authority.

### Lane 3 — Generated-artifact conflict controls
**Owner:** executor + verifier  
**Owned surfaces:** generators/materializers, drift checks, validator code.  
**Avoid:** `*.generated.json`, generated face outputs, hand-edits to derived files.  
**Depends on:** G002.  
**Acceptance:** conflict surface shrinks, or a precise no-code evidence ledger is produced.  
**Verification:** materialization/drift check; prove no generated output was hand-edited.

### Lane 4 — Rust purity / Python-MJS retirement inventory
**Owner:** executor + dependency-expert  
**Owned surfaces:** live Python/MJS/shell authority inventory, delete/fence slices, Rust parity only where the old authority is still valid and worth preserving.  
**Avoid:** historical/vendor-only scripts, dead code, recently merged surfaces unless revalidated.  
**Depends on:** G002 and G003 hygiene findings.  
**Acceptance:** at least one live authority is retired/fenced, or the lane produces a precise “keep it” inventory with justification.  
**Verification:** inventory before/after, targeted tests/Buck2, ponytail-style deletion evidence.

### Lane 5 — Verification / review / launch gate prep
**Owner:** verifier + code-reviewer + architect  
**Owned surfaces:** test matrix, review evidence, UltraQA scenarios, launch checklist, and the `G012` cross-cutting production-bar evidence ledger.  
**Avoid:** implementation except test/evidence artifacts explicitly needed for gating.  
**Depends on:** all implementation lanes as they produce output.  
**Acceptance:** review-ready evidence exists before merge attempts; `G012` has one accountable evidence owner instead of being implicitly shared.  
**Verification:** independent code-reviewer + architect review, UltraQA, CI status, fresh goal snapshot.

### Continuous process owner — G018 closed-loop review
**Owner:** leader, with Lane 5 review/evidence support  
**Owned surfaces:** cross-wave retrospectives, next-cycle planning inputs, Ultragoal/Team shutdown lessons, and reusable process guardrails.  
**Avoid:** worker-side mutation of `.omx/ultragoal/**` or revival of stale Team state.  
**Depends on:** every wave checkpoint and shutdown gate.  
**Acceptance:** each wave records what changed, what was verified, what remained blocked, and what should feed the next full-replan/Ultragoal checkpoint.  
**Verification:** fresh `get_goal`, ledger entry, review evidence, and explicit next-cycle decision.

### Serial/hot surfaces reserved to leader
- `.omx/ultragoal/**`
- `.omx/state/team/**`
- `*.generated.json`
- `.github/workflows/**`
- `oya-ci.toml`
- `specs/root-hub-pointers.json`
- `docs/AGENTS.md`
- `specs/master-plan-sequencing.json`
- `specs/markdown-retirement-policy.json`
- any branch-protection / required-status configuration
- root build/toolchain policy surfaces

**Rule:** if a lane needs one of these, it stops and requests rebinding; it does not improvise overlap.

---

## 6) Team + Ultragoal bridge

- **Leader owns** `.omx/ultragoal/{brief.md,goals.json,ledger.jsonl}`.
- **Workers do not mutate goal state.**
- **Team returns evidence only:** task ids, owned-path updates, tests, PR refs, blockers, and cleanup notes.
- **Checkpoint rule:** before any checkpoint or completion claim, leader captures a fresh `get_goal` snapshot and reconciles it with `.omx/ultragoal/goals.json` and `ledger.jsonl`.
- **Current snapshot:** `get_goal` returned `null`, so there is no active Codex goal to reconcile in this turn.

### Team ledger template
```md
## Team Ledger
- team_id:
- wave:
- leader:
- goal_ids:
- collision_map:
- workers:
  - worker_id:
    role:
    reasoning_level:
    owned_paths:
    avoid_paths:
    verification:
    exit_criteria:
- shared_hot_files:
- PRs:
- blockers:
- shutdown_gate:
```

### Worker assignment template
```md
- worker_id: W1
  role: executor
  owned_paths: [...]
  avoid_paths: [...]
  verification: [...]
  blocker_policy: stop + escalate on shared-file request
```

### Shutdown gate
Team may shut down only when:
1. every assigned task is closed,
2. every open PR has `oya-ci-required` green or is explicitly blocked,
3. review threads are resolved,
4. no worker owns a shared/hot file,
5. generated-file policy is clean,
6. leader has a fresh `get_goal` snapshot,
7. leader records the durable checkpoint or a blocker.

---

## 7) Lifecycle gates by named skill

Use this order across the backlog:

1. **spec** — define the slice and success criteria.
2. **plan** — break into implementable tasks.
3. **task breakdown** — assign disjoint ownership.
4. **TDD / incremental implementation** — small slices only.
5. **API/interface design** as needed — only for actual boundary changes.
6. **observability** — add logs/metrics/traces for runtime-facing work.
7. **CI/CD** — wire or update pipeline evidence.
8. **security / performance** — only where the slice affects trust or load.
9. **ai-slop-cleaner** — remove speculative clutter after the slice is working.
10. **independent code-reviewer + architect** — reviewer must not be the authoring context.
11. **ponytail-review / code-simplification** — delete or simplify after correctness is proven.
12. **UltraQA** — adversarial checks on runtime-facing work.
13. **shipping / launch** — only when CI/review/QA are green.
14. **Ultragoal checkpoint** — fresh `get_goal` + durable ledger evidence.

---

## 8) Risks and mitigations

- **Dirty, behind leader checkout**  
  *Mitigation:* no source edits here; use fresh isolated worktrees from fresh `origin/dev`.

- **Stale team/worktree resurrection**  
  *Mitigation:* treat `.omx/state/team` as read-only evidence; do not revive missing terminal lanes.

- **Hot-file overlap**  
  *Mitigation:* serial leader ownership; explicit rebinding required.

- **Generated artifact drift**  
  *Mitigation:* no hand edits to derived files; only generator/materializer/controller paths.

- **Goal-state mismatch**  
  *Mitigation:* leader refreshes with `get_goal` before any checkpoint and records a blocker if the context conflicts.

- **Over-parallelization of early waves**  
  *Mitigation:* G002 first; G015 is re-scoped only after intake.

---

## 9) Verification steps

### Planning-stage verification
- Confirm file authority chain: `AGENTS.md`, `docs/AGENTS.md`, `specs/root-hub-pointers.json`, `specs/master-plan-sequencing.json`, `specs/markdown-retirement-policy.json`, ADR-0363, ADR-0515 as current CI authority, and ADR-0513 as historical/background only.
- Confirm current state: dirty/behind checkout, stale worktrees, no open PRs, no active Codex goal.
- Confirm backlog state in `.omx/ultragoal/goals.json` and `ledger.jsonl`.
- Confirm no unresolved user preference remains.

### Execution-stage verification
- `git status --short --branch`
- `git worktree list`
- `gh pr list --base dev --state open`
- targeted Buck2/Rust tests for changed slices
- generated-face materialization/drift checks where relevant
- independent code-reviewer + architect review
- UltraQA
- fresh `get_goal`
- `omx ultragoal checkpoint ...` only after durable evidence exists

---

## 10) Available-agent-types roster and staffing guidance

### Roster
`explore`, `planner`, `architect`, `critic`, `executor`, `team-executor`, `test-engineer`, `verifier`, `code-reviewer`, `code-simplifier`, `debugger`, `dependency-expert`, `researcher`, `writer`, `git-master`

### `$ultragoal` staffing guidance
Use `$ultragoal` when durable backlog ownership and checkpointed progress matter most.  
Recommended stack:
- 1 leader: `planner` or `verifier` with high reasoning
- 1 gate reviewer: `architect`
- 1 adversarial reviewer: `critic`
- 2–4 execution lanes: `executor` / `team-executor`
- 1 evidence lane: `verifier` or `test-engineer`
- optional `code-reviewer` for post-slice review

**Why:** Ultragoal is the durable state owner; it keeps the backlog, ledger, and checkpoint logic coherent across waves.

### `$team` staffing guidance
Use `$team` when the plan has disjoint lanes that can run in parallel.  
Recommended Wave A staffing:
- 1 leader
- 5 workers minimum for G003/G004/G005/G006 + verification
- one worker per hot surface only if the surface is explicitly serialized
- suggested reasoning levels: leader high; workers medium-high; review lanes high

**Why:** Team is the throughput layer; Ultragoal is the durable ledger layer. Team proves work; Ultragoal records the durable completion.

### `$ralph` fallback note
Use `$ralph` only if a single-owner sequential lane is intentionally chosen for a hot/shared surface or a deeply coupled slice. It is **not** the default follow-up here.

### Goal-Mode Follow-up Suggestions
- **Default:** `$ultragoal`
- **Research project with deliverable/evaluator:** `$autoresearch-goal`
- **Optimization/performance project:** `$performance-goal`

### Launch hints
- Use `$ultragoal` with this plan path when you want durable goal-mode follow-up.
- Use `$team` with this plan path only after the leader has a current collision map and disjoint ownership list.
- Keep `omx team` / `$team` launch arguments aligned with the lane ledger and the approved path set.
- Do **not** launch Team in this planning phase.

### Team verification path
Team proves:
1. lane-local tests/builds,
2. no shared-file collisions,
3. PRs target `dev`,
4. `oya-ci-required` green,
5. review threads resolved,
6. cleanup completed.

Ultragoal checkpoints:
1. fresh `get_goal`,
2. reconcile against `.omx/ultragoal/goals.json`,
3. record durable evidence in `.omx/ultragoal/ledger.jsonl`,
4. only then checkpoint completion or blocker.

---

## 11) Changelog vs prior plan

Applied changes relative to `.omx/plans/ralplan-complete-ultragoal-through-team-ledger-20260626T203000Z.md`:
- shifted the next execution unit from G015-first to **G002-first**;
- added explicit stale-worktree / dirty-checkout handling;
- made hot-file serialization explicit;
- clarified that G015 is a re-scoped Team wrapper, not the immediate fanout target;
- added a full G001-G018 dependency graph;
- strengthened leader-owned `.omx/ultragoal` / fresh `get_goal` checkpointing;
- added the staffing roster, launch guidance, and team shutdown gate.

**Stop rule:** do not fan out implementation until this plan is approved and the leader has re-established intake from the current repo state.
