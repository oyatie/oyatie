---
doc_class: Checklist
checklist_id: CHK-COMPLETE
status: pending approval
purpose: |
  The agent's last 5 actions before marking a PR lane ready. Closes the agentic-navigation contract. Walked at the end of every agent session that modified the repo.
lift_target: oyatie/docs/checklists/agent-completion.md
enforcing_fitness_lane: repo-hygiene-automation-check + oya-governance-audit-emission
owner_team: axis-foundry
related:
  - .omc/plans/MASTERPLAN.md
  - docs/AGENTS.md
  - /templates/checklists/agent-kickoff-checklist.md
  - /templates/checklists/done-definition-checklist.md
  - /templates/checklists/per-implementation-plan-checklist.md
---

# Agent Completion Checklist

> The last 5 actions every agent **MUST** complete before marking a PR lane ready. Mirrors `agent-kickoff-checklist.md` (symmetry by design).

<!-- agent-instructions:start -->

## C1. Verify acceptance criteria

Run every command from IP `§Acceptance test commands`. Capture stdout/stderr directly from Buck2 and remote-check commands. Paste outputs into the PR body `## Verification` section.

If any command fails, **do not** proceed. Re-walk the work, or halt and emit `BLOCKED_ON_HUMAN_ORCHESTRATOR` per `/templates/checklists/escalation-checklist.md`.

## C2. Evidence bundle

Emit the IP's `§Evidence-bundle payload` **verbatim** (do not paraphrase) under `/evidence/multispectrum/`:

```json
{
  "change_id": "IP-NNN-<slug>",
  "branch": "<short-lived-branch>",
  "acceptance_lanes_green": ["<buck2-target-or-remote-context>"],
  "next_ip": "<pointer>"
}
```

If the IP shipped a mechanical prevention for a prior failure, include the failure-mode row and prevention target in the evidence bundle.

## C3. Update inventory ledger (if migration-class)

Per `/templates/checklists/inventory-update-checklist.md` — append a row to the active inventory ledger naming: source path, target path, archive path (if any), tombstone (if delete), audit emission ID, agent or human orchestrator. *Lane:* `oya-governance-inventory-tracker`.

## C4. Audit-chain emission

Confirm:
- `EVT-IP-MERGED` emitted with IP ID + merge SHA + symbol list.
- PR remote checks and required evidence contexts are attached to the merge commit.
- (If capability publish) `oya.foundry.capability.invoked` topic registered.
- (If runbook author) `oya.ops.runbook.invoked` topic resolvable.

Paste the `EVT-*` IDs into PR `## Evidence`. *Lane:* `oya-governance-audit-emission`.

## C5. Mark PR lane ready

```
gh pr ready <pr-number>
gh pr checks <pr-number>
```

Confirms the short-lived PR lane has evidence attached and is ready for review/merge sequencing.

<!-- agent-instructions:end -->

## Hard rules

- **Use plain `git`, `gh`, and Buck2.** Retired local VCS/governance wrappers are not SCM or CI authority.
- **No PR-ready label before all lane-owned paths are committed and verified.** Half-done lanes corrupt the merge queue.
- **No silent loop exit.** If running inside Ralph / autopilot / ultrawork / team loop, re-walk the `done-definition-checklist.md` before cancelling (per `docs/AGENTS.md §Long-running loop rule`).

## Loop-cancellation

Cancel via `/oh-my-claudecode:cancel` **only** when (a) the change is complete and verified per `done-definition-checklist.md`, OR (b) the loop is structurally blocked (then emit `BLOCKED_ON_HUMAN_ORCHESTRATOR`).

## Stop conditions

- Any IP `§Acceptance test commands` row produced FAIL → halt, do not mark the PR ready.
- Audit-chain emission failed → halt, do not mark the PR ready; emit `BLOCKED_ON_HUMAN_ORCHESTRATOR`.
- Evidence bundle creation or secret-scan failed → fix evidence and re-run verification before marking ready.

## Human path (junior developer)

Same five actions, mapped to `rtk`-prefixed terminal commands: verify tests pass, append context note (could be a commit-message footer or PR comment), update inventory if applicable, confirm CI emitted audit-chain events, push the branch and request review. Reviewer adds `## Code Review` at merge.
