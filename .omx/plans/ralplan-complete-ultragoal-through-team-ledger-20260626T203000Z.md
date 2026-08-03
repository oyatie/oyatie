# RALPLAN: Complete Ultragoal Through Team Ledger Closed Loop

Created: 2026-06-26T20:30:00Z
Autopilot phase: `ralplan`
Inputs:
- `.omx/context/complete-the-ultragoal-maximize-throughput-by-pa-20260626T201929Z.md`
- `.omx/specs/deep-interview-complete-ultragoal-through-team-ledger.md`
- `.omx/ultragoal/{brief.md,goals.json,ledger.jsonl}`
- `/specs/root-hub-pointers.json`, `docs/AGENTS.md`, `/specs/master-plan-sequencing.json`

## Requirements summary

Complete the current durable Ultragoal backlog under Autopilot's strict lifecycle. The immediate execution target is the active Wave A foundation story `G015-wave-a-m0-m4-team-foundation`, while preserving the aggregate `G001-complete-oyatie-through-small-merge` and later goals `G002..G018`.

The work must maximize throughput by planning disjoint lanes before fan-out. Team workers may own slices end-to-end only after this plan receives sequential Architect then Critic approval. The leader owns the Ultragoal ledger, open-PR collision map, worker ledger, PR queue, final reviews, UltraQA, and fresh Codex goal reconciliation.

## Current state snapshot

- `origin/dev` was fetched on 2026-06-26; local leader checkout is 199 commits behind and dirty. Do not edit source here.
- Open PR list was empty during preflight.
- Recent merged PRs include #908, #909, #910, plus #903/#904 from earlier waves, with `oya-ci-required` success verified in prior preflight.
- Prior team `python-mjs-to-rust-re-3d81f621` is missing/terminal and must not be resurrected.
- `.omx/ultragoal` summary: 18 goals; `G001` and `G015` in progress; 16 pending.
- Active Codex goal objective is the user Autopilot prompt, while `.omx/ultragoal` expects the stable aggregate objective. Final completion checkpoint is blocked until fresh reconciliation is valid or a sanctioned blocker is recorded.

## RALPLAN-DR summary

### Principles
1. **Merge safety over raw parallelism.** Throughput is useful only when slices remain disjoint and PRs merge cleanly.
2. **Delete/fence before porting.** Rust ports are justified only for live, valid, worth-preserving behavior; dead Python/MJS/shell/CLI authority is removed or fenced.
3. **Authority stays canonical.** Repository policy comes from root specs/docs; `.omx` is workflow state; protected PR + `oya-ci-required` is merge authority.
4. **One writer per hot surface.** Shared workflow/config/generated-policy/generated-artifact surfaces are serial lanes.
5. **Evidence closes each loop.** Every slice has behavior lock, targeted verification, review, CI status, PR outcome, and cleanup evidence.

### Decision drivers
1. Avoid merge conflicts across a large stale/dirty repo while still increasing throughput.
2. Advance active Wave A (`M0..M4`) because it unblocks later cloud-ci, generated-artifact, and Rust-purity waves.
3. Preserve Autopilot/Ultragoal gates: approved plan, durable execution evidence, code-review, UltraQA, and final reconciliation.

### Viable options considered

#### Option A — Launch a large Team immediately from the dirty leader checkout
- Pros: fastest apparent fan-out.
- Cons: violates Autopilot ralplan gate, dirty/behind checkout risks stale worktrees, no approved collision map, likely duplicate/stale worker resurrection.
- Decision: rejected.

#### Option B — Plan Wave A lanes, create fresh leader worktree, launch Team with ledger and disjoint ownership
- Pros: satisfies Autopilot, Team, Ultragoal, and repo worktree contracts; enables safe parallelism; supports PR/CI/merge closed loop.
- Cons: requires upfront planning and strict ledger maintenance.
- Decision: chosen.

#### Option C — Single-owner sequential Ultragoal/Ralph-style execution
- Pros: lowest coordination overhead and conflict risk.
- Cons: violates user preference for maximum parallelization, slower on disjoint M0-M4 slices.
- Decision: fallback only for serial/hot-file repair or if Team runtime blocks.

#### Option D — Continue only Python/MJS-to-Rust work
- Pros: aligns with recent merged #908/#909/#910 momentum.
- Cons: too narrow for the active Ultragoal; ignores M0 intake, generated-artifact, cloud-ci, and process lanes.
- Decision: use as one lane within Wave A, not the whole plan.

## Decision / ADR

### Decision
Run Wave A as a Team+Ultragoal execution wave after sequential Architect and Critic approval. Use a fresh worktree from `origin/dev` for leader/team launch, create a leader-owned `WORK_LEDGER.md` in the team state root, assign disjoint worker lanes with `/goal worker-N`, and require each lane to drive plan/test/build/review/fix/PR/CI/merge/cleanup for its slice.

### Drivers
- Current leader checkout is stale/dirty; source edits need isolated worktrees.
- No open PRs means a clean lane map can be established now.
- `G015` already encodes M0-M4 Team foundation execution.
- User explicitly requested maximum parallelization, lane planning before fan-out, and workers owning slices end-to-end.

### Alternatives considered
- Immediate Team launch without plan: rejected due gate and collision risks.
- Sequential-only execution: kept as fallback for serial shared surfaces.
- Narrow Python/MJS-only continuation: folded into M4 lane.

### Why chosen
Option B is the only approach that satisfies Autopilot, Team, Ultragoal, repo governance, and the user's throughput objective simultaneously.

### Consequences
- Team launch is blocked until this ralplan receives Architect then Critic approval.
- The leader must keep the ledger updated; workers should not wait for steering except true blockers.
- Shared/hot files are serial. If a lane discovers it needs a hot file, it must report conflict/scope expansion.
- Final Ultragoal completion checkpoint may remain blocked by Codex goal objective mismatch even if work completes; that is recorded, not papered over.

### Follow-ups
- After approval, create fresh leader/team worktree from `origin/dev`.
- Create Team task ledger and lane queues before launching workers.
- Add final gate artifacts: code-review report, UltraQA scenario matrix, PR/CI merge evidence, and fresh goal reconciliation.

## Task breakdown and dependency graph

### Phase 0 — Leader setup and collision map (serial)
Acceptance:
- Fresh worktree from `origin/dev` exists for team launch.
- `gh pr list --state open` and current branch/worktree inventory are captured.
- `WORK_LEDGER.md` template is ready with open PR/collision map and refresh rules.
Verification:
- `git -C <leader-worktree> status --short --branch`
- `gh pr list --state open --limit 50 --json number,title,headRefName,baseRefName,isDraft,mergeStateStatus,statusCheckRollup`
- `omx team status <team>` after launch.

### Phase 1 — Team launch and ledger creation (serial)
Acceptance:
- `omx team` starts successfully inside tmux with gpt-5.5/xhigh default launch args where runtime supports it.
- Team state root contains `WORK_LEDGER.md`.
- Worker assignments include `/goal worker-N`, owned paths, avoid paths, verification, PR/CI rules, and report-only conditions.
Verification:
- `omx team status <team> --json`
- `omx team api list-tasks --input '{"team_name":"<team>"}' --json`
- `tmux capture-pane` spot checks for leader/workers.

### Phase 2 — Wave A disjoint lanes (parallel where file ownership is disjoint)

#### Lane 1 / M0 Intake and merge queue
Owned scope:
- branch/worktree inventory, stale local worktree report, PR/issue/backlog snapshot, no source hot-file edits unless approved.
Avoid:
- source code edits that overlap worker implementation lanes.
Acceptance:
- Current PR/collision map and merge queue state are in ledger.
- Stale worktree cleanup recommendations are evidence-backed; destructive cleanup requires explicit leader approval if outside ignored state.
Verification:
- `git worktree list`, `gh pr list`, `gh issue list` as needed, ledger diff.

#### Lane 2 / M1 Root/runtime hygiene
Owned scope:
- local scratch/log/runtime boundary evidence; tracked source changes only if small, single-owner, and not generated.
Avoid:
- `.omx/**`/`.omc/**` tracked-policy confusion, generated JSON, hot specs.
Acceptance:
- Root scratch/runtime artifacts are deleted/fenced only when valid and safe; otherwise ledgered.
- No hand edit of generated files.
Verification:
- `git status --short`, targeted tests for any behavior-affecting cleanup.

#### Lane 3 / M2 Cloud-ci universal product boundary
Owned scope:
- `cloud/**`, `crates/**` cloud-ci APIs/policy data-pack seams, Rust gate packet boundaries.
Avoid:
- `.github/workflows/**`, `oya-ci.toml`, shared root specs unless sole-writer serial handoff.
Acceptance:
- At least one small PR-sized improvement moves repo-specific bridge behavior toward universal Rust API/data-pack boundary, or produces a no-code evidence ledger if implementation is not yet safe.
Verification:
- targeted Buck2/Rust tests; no Cargo-only authority; multispectrum evidence if PR.

#### Lane 4 / M3 Generated-artifact conflict elimination
Owned scope:
- materializer/generator policy and drift-compare surfaces, not generated outputs themselves.
Avoid:
- `*.generated.json`, broad generated artifact churn.
Acceptance:
- Generated merge-surface reduction or validator/materializer improvement is implemented, or a precise conflict map is delivered.
Verification:
- materialization/drift gate command if available; targeted tests; prove no hand-edited generated output.

#### Lane 5 / M4 Rust purity and Python/MJS authority retirement
Owned scope:
- remaining live Python/MJS/shell bridge inventory and one small delete/fence/port slice that does not overlap recent #908/#909/#910/#907/#906 work.
Avoid:
- historical/vendor-only scripts, dead-code ports, generated outputs.
Acceptance:
- Delete/fence dead wrappers first; port only a live authoritative script if evidence shows it is valid and worth preserving.
Verification:
- inventory grep before/after; targeted tests/Buck2 gate; ponytail-review net deletion evidence.

#### Lane 6 / Verification, review, and launch-gate evidence
Owned scope:
- test matrix, CI status polling, code-review readiness, UltraQA scenario preparation, multispectrum evidence checklist.
Avoid:
- authoring implementation except test/evidence artifacts assigned by leader.
Acceptance:
- Provides independent verification lane and catches missing tests, security, performance, observability, and launch-gate gaps before PR merge.
Verification:
- actual commands from implementation lanes; review checklist; UltraQA scenario draft.

### Phase 3 — Integration and PR queue (leader serial)
Acceptance:
- No pending/in-progress Team tasks except CI/review follow-up explicitly ledgered.
- Each PR has owner, CI status, review result, merge outcome or blocker.
- Merge order avoids shared-file collisions.
Verification:
- `gh pr view <n> --json state,mergeStateStatus,statusCheckRollup,mergedAt,mergeCommit`
- `omx team status`, `omx team api list-tasks`, ledger.


### PR closeout checklist for every implementation lane
Workers and leader must satisfy the repo PR contract before a lane is terminal:
- PR targets `dev` from an isolated worktree branch and describes intent, scope, verification, multispectrum evidence, generated-file policy, and rollback/launch notes when runtime-impacting.
- PR body includes a `## Code Review` section and evidence aligned to the repo done-definition checklist.
- `oya-ci-required` is green; legacy/local `oya` or bridge checks are supporting evidence only, never merge authority.
- Review threads are resolved; required approval is present; generated JSON was not hand-edited.
- Merge is completed through the protected PR path, then the worker/leader records merge commit, CI status, and cleanup in `WORK_LEDGER.md` and Ultragoal evidence.

### Phase 4 — Autopilot final gates (serial)
Acceptance:
- `$code-review` two-lane review returns APPROVE + CLEAR.
- `$ponytail:ponytail-review` finds no required complexity cuts or cuts are applied.
- `$ai-slop-cleaner` cleanup plan/gates run for changed code where applicable.
- `$ultraqa` passes adversarial scenario matrix or is explicitly scoped/skipped only for non-runtime documentation.
- Fresh `get_goal` and `omx ultragoal status/checkpoint` reconcile or record the sanctioned blocker.
Verification:
- durable reports saved under `.omx/reviews/` or equivalent state/ledger paths.

## Available agent-types roster and staffing guidance

Known roles available in this environment:
- `explore` / `explorer`: fast repo-local lookup.
- `planner`: task sequencing and risk flags.
- `architect`: design, boundaries, long-horizon tradeoffs.
- `critic`: adversarial plan critique.
- `executor`: generic implementation/refactoring.
- `team-executor`: conservative supervised team execution.
- `test-engineer`: test strategy and flaky-test hardening.
- `verifier`: completion evidence and claim validation.
- `code-reviewer`: multi-axis review.
- `code-simplifier`: simplification pass.
- `debugger`: root-cause diagnosis.
- `dependency-expert`: package/SDK decisions.
- `researcher`: external official docs/reference.
- `git-master`: branch/history/merge hygiene.
- `writer`: documentation/handoff.

Recommended Team staffing after approval:
- 6 workers minimum for Wave A: 5 implementation/inventory lanes + 1 verification lane.
- Use `executor` or `team-executor` for lanes 1-5 depending runtime prompt availability.
- Use `test-engineer`/`verifier` as lane 6 if Team launch supports mixed roles; otherwise assign as executor with explicit verification-only objective.
- Model default requested by user: gpt-5.5 xhigh for leader/workers where Team launch args allow it; native role settings may be fixed by role catalog.


## Hot-file reservation appendix

These reservations are binding for Team launch and must be copied into `WORK_LEDGER.md` before workers claim tasks.

### Serial/hot surfaces: leader-owned unless explicitly rebound
- `.github/workflows/**`, `oya-ci.toml`, CI admission/context configuration, and any required-status/check-name changes.
- `/specs/root-hub-pointers.json`, `docs/AGENTS.md`, `/specs/masterplan.json`, `/specs/master-plan-sequencing.json`, `/specs/markdown-retirement-policy.json`, `/specs/agent-operating-contract.json`, and root authority/contract surfaces.
- `*.generated.json`, generated-face outputs, generated registry projections, and generated materialization products. Workers may edit only generators/materializers/tests, never generated outputs by hand.
- Shared generator/materializer policy surfaces that can affect multiple lanes, including broad drift-compare or admission-gate policy files.
- Root build/toolchain policy surfaces that affect all lanes (`BUCK`, `BUCK.v2`, `MODULE.bazel`-like roots if present, Rust toolchain policy, lock/update policy) unless assigned as a single-writer task.
- `.omx/ultragoal/**`, `.omx/state/team/**`, Codex goal state, and the Team `WORK_LEDGER.md`; workers read/report only. The leader owns mutations.

### Lane path reservations before fan-out
- Lane 1 (M0) is read/report-first: branch/worktree/PR/backlog metadata only; no source edits without leader rebind.
- Lane 2 (M1) owns root scratch/runtime hygiene only after leader confirms whether each file is tracked authority, ignored runtime state, or stale scratch. It must not touch canonical specs or generated outputs.
- Lane 3 (M2) owns cloud-ci product/API boundary slices under `cloud/**` and the smallest related Rust crate paths assigned in the ledger. It must avoid generator/materializer files claimed by Lane 4 and any CI workflow/status-context files unless the leader serializes that handoff.
- Lane 4 (M3) owns generated-artifact conflict controls in generator/materializer/test surfaces only. It must avoid cloud-ci runtime API implementation files claimed by Lane 3 unless the leader rebinds ownership.
- Lane 5 (M4) owns Python/MJS/shell authority retirement inventory and one small delete/fence/port slice. It must avoid recently merged #907-#910 surfaces unless fresh `origin/dev` evidence shows remaining live work.
- Lane 6 owns verification/review evidence and may add tests or evidence files only when the ledger grants that path.

### Rebind rule
No worker may claim, edit, or create a new shared/hot path not listed in its task. If a slice requires a reserved path, the worker reports `shared-file request` with the exact file, reason, and downstream effect; the leader updates `WORK_LEDGER.md`, reassigns ownership, and serializes affected tasks before work continues.

## Team + Ultragoal launch hints

After Architect and Critic approval:

```sh
# create fresh launch worktree from origin/dev
mkdir -p /Users/jasonlee/oyatie-worktrees
git worktree add /Users/jasonlee/oyatie-worktrees/ultragoal-wave-a-$(date -u +%Y%m%dT%H%M%SZ) origin/dev

# launch coordinated team from fresh worktree in tmux
export OMX_TEAM_WORKER_LAUNCH_ARGS="--model gpt-5.5 --reasoning-effort xhigh"
omx team 6:executor "Wave A G015: execute M0-M4 disjoint lanes from WORK_LEDGER under Ultragoal; each worker owns its slice through PR/CI/merge cleanup; leader owns ledger/checkpoints."
```

Create/update `WORK_LEDGER.md` in the new `.omx/state/team/<team>/` root immediately after launch. Each task body must include:

```text
/goal worker-N: <one sentence outcome>
Read WORK_LEDGER.md first, then claim this task. Own <lane> end to end.
Owned paths: <paths>. Avoid: <open PRs/shared/hot files>.
Ponytail: delete/fence dead code; port only live valid behavior to Rust.
Verify: <targeted commands>. Commit, push, PR, watch/fix CI.
Report only blocker/shared-file request/scope expansion/verification-failed-after-3/PR terminal/CLEAR REQUEST/COMPACT REQUEST.
```

## Verification and quality gates by skill

- **Spec-driven development:** this plan and deep-interview handoff define requirements before implementation.
- **Best-practice research/source discipline:** external research is required only when a lane depends on current external APIs/tools; otherwise root specs and installed skill docs are authoritative.
- **TDD:** behavior changes require a failing or regression test before fix/port; pure deletion/fencing needs targeted proof that removed surface is dead or non-authoritative.
- **Incremental implementation:** each worker slice must stay PR-sized and independently verifiable.
- **API/interface design:** new cloud-ci/Rust APIs need contract-first schemas and boundary validation.
- **Observability:** production/runtime features need on-call questions, structured logs/metrics/traces or explicit N/A.
- **CI/CD:** PRs must rely on `oya-ci-required`; local checks are evidence only.
- **Performance:** optimize only measured hot paths; no speculative performance complexity.
- **Security:** threat model any boundary accepting untrusted input or external data; no secrets in logs/prompts.
- **Code simplification / ponytail:** reduce code and dependency surface; no new abstraction unless earned.
- **Doubt-driven development:** non-trivial decisions require adversarial review; cross-model is skipped unless explicitly authorized in non-interactive/autonomous context.
- **Shipping:** each merged wave needs rollback/readiness notes where runtime-impacting.
- **UltraQA:** final QA must include adversarial matrix for state, stale prompts, dirty worktrees, misleading success output, and Team/Ultragoal resume behavior.

## Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Codex goal objective mismatch | Final Ultragoal complete checkpoint fails | Continue execution evidence; do not mark complete; record sanctioned blocker if mismatch remains at terminal gate |
| Shared hot-file conflict | PR churn, merge conflicts | One writer per hot surface; ledger conflict map; serial handoffs |
| Dirty/stale checkout | Wrong base, accidental source edits | Fresh worktree from `origin/dev`; leader checkout remains orchestration-only |
| Over-porting Python/MJS | Wasted complexity and dead Rust code | Ponytail delete/fence first; require live authority evidence before port |
| Generated-file hand edits | Gate failure | Workers avoid `*.generated.json`; use materializer/drift gates |
| Team backlog queues messages | Worker confusion/duplicates | Leader monitor loop uses status, task API, mailbox, and tmux capture before triggers |
| Large backlog overwhelms Team | Half-finished PRs | Wave A only until terminal; no additional work beyond approved lane queues |


## Codex-goal reconciliation matrix

Before any Ultragoal checkpoint or Autopilot completion claim, the leader must call `get_goal` and pass the fresh snapshot path/JSON into `omx ultragoal status` or `omx ultragoal checkpoint`.

| Fresh `get_goal` result | Allowed action | Forbidden action |
|---|---|---|
| Matching active or completed aggregate objective for `.omx/ultragoal/goals.json` | If all goals/gates are actually complete, checkpoint the relevant goal/story with evidence and final quality-gate JSON. | Do not omit evidence or skip code-review/UltraQA. |
| Active objective differs from `.omx/ultragoal` stable aggregate objective (current known state) | Continue execution evidence; do not mark complete. At terminal gate, record a non-terminal reconciliation blocker if no sanctioned context exists. | Do not call `update_goal`; do not force a complete checkpoint. |
| Completed task-scoped objective for the same aggregate Ultragoal plan | Checkpoint `G001-complete-oyatie-through-small-merge` complete only if OMX reconciliation accepts the completed task-scoped scope and final gates are clean. | Do not mutate Codex goal state from hooks/shell. |
| Different completed legacy objective and complete checkpoint fails | Record `blocked` checkpoint evidence naming the conflicting completed legacy objective, then stop. | Do not repeat `--status complete` in the same thread. |
| `get_goal` unavailable/null/schema error (for example DB/table/context issue) | Record an auditable safe-recovery blocker with the unavailable/error JSON/path and continue only from a working Codex goal context. | Do not fabricate a snapshot or silently self-attest completion. |

## Execution stop rules

Stop and report blocker when:
- Team runtime/tmux is unavailable and native subagents cannot satisfy the requested durable Team protocol.
- Required credentials/branch protection prevent PR/CI/merge follow-through.
- Same review/QA failure repeats 3 cycles without a new fix path.
- Final goal reconciliation remains false after all work and the hook-prescribed blocker must be recorded.

## Ralplan consensus checklist

- [ ] Architect review completed after this plan was written.
- [ ] Critic review completed after Architect result.
- [ ] Accepted Architect/Critic improvements applied to this plan.
- [ ] Autopilot state records `ralplan_architect_review` then `ralplan_critic_review`, both approve, before transition to Ultragoal.


## Ralplan revision changelog

- Revision 1 after Architect WATCH/revise: added hot-file reservation appendix, lane path reservations, rebind rule, explicit PR closeout checklist, and Codex-goal reconciliation matrix.
