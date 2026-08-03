# Deep-Interview Spec: Complete Ultragoal Through Full Replan Before Fanout

## Metadata
- Created: 2026-06-26T22:26:09.726224+00:00
- Profile: standard
- Rounds: 2
- Final ambiguity: 9.4%
- Threshold: 20%
- Context type: brownfield
- Context snapshot: `.omx/context/complete-ultragoal-through-team-ledger-20260626T220541Z.md`
- Transcript: `.omx/interviews/complete-ultragoal-through-team-ledger-20260626T222609Z.md`
- Previous related spec superseded/updated: `.omx/specs/deep-interview-complete-ultragoal-through-team-ledger.md`

## Intent
Finish Oyatie through the existing durable Ultragoal backlog while maximizing safe throughput through planned parallel lanes, Team/worker ownership, and closed-loop lifecycle verification.

## Desired outcome
Before any new implementation fanout, regenerate and critique the full Ultragoal execution plan. The replan must produce a lane/sequence/verification design that can later drive Ultragoal + Team execution without stale-team resurrection, dirty-leader edits, generated-artifact hand edits, or merge-conflict-prone worker overlap.

## In scope
- Re-read current `.omx/ultragoal/brief.md`, `.omx/ultragoal/goals.json`, and `.omx/ultragoal/ledger.jsonl` as durable state.
- Reconcile repo authority from `specs/root-hub-pointers.json`, `docs/AGENTS.md`, `specs/master-plan-sequencing.json`, `specs/markdown-retirement-policy.json`, ADR-0363/0513/0515, and generated-artifact policy surfaces.
- Produce a full replan before execution fanout, including:
  - updated milestone/story sequencing for G001/G002-G018;
  - lane dependency graph and disjoint ownership map;
  - Team + Ultragoal bridge contract;
  - Team ledger structure and worker assignment template;
  - verification ladder per lane, with Buck2/Rust authority where applicable;
  - final quality gate sequence: tests/build/lint/static/security/perf as applicable, ai-slop-cleaner, code-review with independent `code-reviewer` + `architect`, UltraQA, shipping/launch checklist, and Ultragoal checkpointing with fresh `get_goal` snapshot;
  - collision policy for dirty checkout, stale worktrees, shared/hot files, generated files, and open PRs.
- Write planning artifacts under `.omx/plans/` and keep source code unchanged until the plan is approved/selected for execution.

## Out of scope / non-goals
- No source implementation during deep-interview.
- No tmux Team launch until the regenerated plan defines lanes, collision map, verification path, and shutdown/checkpoint gates.
- No mutation/deletion of existing dirty files, stale worktrees, branches, generated files, or runtime state as an interview side effect.
- No hand edits to `*.generated.json` or generated faces.
- No new dependencies by default; dependency additions require explicit later justification.
- No legacy `oya`/CLI merge-authority claims; cloud-ci `oya-ci-required` remains the required context.
- No stale worker/team resurrection; fresh team scheduling must start from current state evidence.

## Decision boundaries
OMX may decide without further user confirmation:
- How to structure the replan document and Team ledger fields.
- Which repo-local evidence to inspect for current state, stale team/worktree risk, and collision maps.
- Which native subagent roles to use for sequential planning review (`planner`, then `architect`, then `critic`) and later independent final review (`code-reviewer`, `architect`) when execution reaches that gate.
- Conservative lane sizing, role allocation, and ordering that preserve the selected full-replan-before-fanout stride.

OMX must not decide without explicit user or plan approval:
- Starting source implementation or Team fanout before replan approval.
- Destructive cleanup of dirty files/worktrees/branches or generated artifacts.
- Weakening verification gates or treating Cargo-only/legacy CLI output as final authority.
- Adding dependencies or changing public/product API direction beyond the approved plan.

## Constraints and brownfield evidence
- Leader checkout is dirty and behind `origin/dev` (`0	202`), so implementation must use clean isolated worktrees from fresh `origin/dev`.
- No open PRs against `dev` were reported during preflight, but many stale local worktrees exist; the replan must include a current collision/worktree intake gate.
- Existing Ultragoal state has active/in-progress ids `['G001-complete-oyatie-through-small-merge', 'G015-wave-a-m0-m4-team-foundation']` and pending goals count `16`.
- `get_goal` returned no active Codex goal; Ultragoal create/checkpoint may create or reconcile only through Codex goal tools, not shell mutation.
- Generated-artifact policy: never hand-edit generated JSON; use materializer/controller/gate flows.
- Plain git + PR to `dev` + `oya-ci-required` is merge authority; legacy CLI bridge output is local evidence only.

## Testable acceptance criteria for the next planning stage
- A regenerated plan is written under `.omx/plans/` and cites this spec plus current Ultragoal files.
- The plan includes RALPLAN-DR principles, drivers, viable options, ADR section, risks/mitigations, and concrete verification steps.
- The plan contains a full milestone dependency graph for G001/G002-G018 and identifies which goals can run in parallel vs must be sequential.
- The plan contains Team staffing guidance with installed role names only, suggested worker count, disjoint path/lane ownership, and a ledger/assignment template.
- The plan includes explicit no-go boundaries for generated files, dirty leader checkout, stale worktrees, shared hot files, and PR collision handling.
- The plan defines the entire lifecycle gate: spec -> plan -> task breakdown -> TDD/incremental implementation -> observability/security/perf/CI gates -> ai-slop-cleaner -> code-review -> UltraQA -> shipping/launch -> Ultragoal checkpoint.
- Architect review and Critic review are run sequentially on the plan; unresolved BLOCK/REVISE findings are incorporated before any fanout recommendation.

## Assumptions exposed and resolutions
- Assumption: “finish the project” means execute the existing Ultragoal backlog, not invent a separate new project definition. Resolution: use existing `.omx/ultragoal` as durable state.
- Assumption: stale team mode should not block questions or be resurrected. Resolution: stale `team` mode marker was cleared only after confirming no local team dirs or worker panes.
- Assumption: full replan is preferred over immediate G015 resume. Resolution: user selected `full_replan_first` in Round 1.
- Assumption: no extra non-goals beyond repo and Ultragoal constraints. Resolution: user answered `none` in Round 2.

## Pressure-pass findings
The pressure pass revisited the “Full replan first” answer and asked which potential parallel work must remain out of scope. The answer added no extra exclusions, so repo/Ultragoal constraints remain the binding non-goal set and the main material decision is the full-replan-before-fanout stride.

## Docs / terminology ledger
- Inspected: `AGENTS.md`, `specs/root-hub-pointers.json`, `docs/AGENTS.md`, `specs/master-plan-sequencing.json`, `specs/markdown-retirement-policy.json`, ADR-0116, ADR-0363, existing `.omx/context/` and `.omx/specs/`, `.omx/ultragoal/*`.
- Canonical terms: Ultragoal, Team, Team ledger, generated faces, `oya-ci-required`, isolated worktree branch, protected PR against `dev`, Buck2/Rust evidence, bridge/local evidence.
- Conflict resolved: “finish project” is too broad as a direct implementation command; it is now scoped as “full replan first, then Ultragoal/Team execution under the regenerated plan.”

## Execution bridge
Recommended next workflow: `$plan --consensus --direct .omx/specs/deep-interview-complete-ultragoal-through-team-ledger.md` or equivalent sequential `planner -> architect -> critic` replan.

Do not implement directly from this spec. After the replan is accepted, use `$ultragoal` as durable leader-owned state and `$team` for coordinated parallel execution only where the plan proves disjoint lane safety.
