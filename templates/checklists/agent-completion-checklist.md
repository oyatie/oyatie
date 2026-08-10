---
doc_class: Checklist
checklist_id: CHK-COMPLETE
status: pending approval
purpose: |
  The agent's last 5 actions before `grit done`. Closes the agentic-navigation contract. Walked at the end of every agent session that modified the repo.
lift_target: oyatie/templates/checklists/agent-completion.md
enforcing_fitness_lane: oya-governance-banned-primitives + oya-governance-audit-emission
owner_team: axis-foundry
related:
  - .omc/plans/MASTERPLAN.md
  - docs/AGENTS.md
  - /templates/checklists/agent-kickoff-checklist.md
  - /templates/checklists/done-definition-checklist.md
  - /templates/checklists/per-implementation-plan-checklist.md
---

# Agent Completion Checklist

> The last 5 actions every agent **MUST** complete before `grit done`. Mirrors `agent-kickoff-checklist.md` (symmetry by design).

<!-- agent-instructions:start -->

## C1. Verify acceptance criteria

Run every command from IP `§Acceptance test commands`. Capture stdout via `oya-tooling-agent-read run-evidence <cmd>`. Paste outputs into the PR body `## Verification` section.

If any command fails, **do not** proceed. Re-walk the work, or halt and emit `BLOCKED_ON_HUMAN_ORCHESTRATOR` per `/templates/checklists/escalation-checklist.md`.

## C2. ICM store (durable memory)

Emit the IP's `§Icm-store-payload` **verbatim** (do not paraphrase):

```
icm store \
  -t context-<project> \
  -c "IP-NNN-<slug> merged at <changebundle-id>; grit symbols released: <list>; acceptance lanes green: <list>; next IP: <pointer>" \
  -i high \
  -k "M0N,P0N,IP-NNN,<axis>"
```

If the IP shipped a mechanical prevention for a prior failure, **also** emit:

```
icm store \
  -t errors-resolved \
  -c "<failure mode description> prevented by <mechanical prevention name>" \
  -i high \
  -k "MFL-NNNN,<prevention-name>"
```

## C3. Update inventory ledger (if migration-class)

Per `/templates/checklists/inventory-update-checklist.md` — append a row to the active inventory ledger naming: source path, target path, archive path (if any), tombstone (if delete), audit emission ID, agent or human orchestrator. *Lane:* `oya-governance-inventory-tracker`.

## C4. Audit-chain emission

Confirm:
- `EVT-IP-MERGED` emitted with IP ID + merge SHA + symbol list.
- `EVT-GRIT-DONE` emitted on `grit done`.
- (If capability publish) `oya.foundry.capability.invoked` topic registered.
- (If runbook author) `oya.ops.runbook.invoked` topic resolvable.

Paste the `EVT-*` IDs into PR `## Evidence`. *Lane:* `oya-governance-audit-emission`.

## C5. Grit done

```
grit done --ip IP-NNN-<slug> --symbols <symbol-1> <symbol-2> …
```

Releases all claimed symbols. Emits `EVT-GRIT-DONE` automatically. Confirm via `oya-tooling-agent-read audit-tail --last 1`.

<!-- agent-instructions:end -->

## Hard rules

- **No `git push` from agent.** PR creation is via `oya-tooling-agent-read pr-create` (read-only adapter wraps `gh pr create` behind a controlled verb, audit-emitting). If unavailable, halt and emit `BLOCKED_ON_HUMAN_ORCHESTRATOR`.
- **No `grit done` before all symbols' work is committed.** Half-done releases corrupt the merge queue.
- **No silent loop exit.** If running inside Ralph / autopilot / ultrawork / team loop, re-walk the `done-definition-checklist.md` before cancelling (per `docs/AGENTS.md §Long-running loop rule`).

## Loop-cancellation

Cancel via `/oh-my-claudecode:cancel` **only** when (a) the change is complete and verified per `done-definition-checklist.md`, OR (b) the loop is structurally blocked (then emit `BLOCKED_ON_HUMAN_ORCHESTRATOR`).

## Stop conditions

- Any IP `§Acceptance test commands` row produced FAIL → halt, do not `grit done`.
- Audit-chain emission failed → halt, do not `grit done`; emit `BLOCKED_ON_HUMAN_ORCHESTRATOR`.
- `icm store` failed → retry once with explicit `icm store -t errors-resolved` row recording the prior failure; if still failing, halt.

## Human path (junior developer)

Same five actions, mapped to `rtk`-prefixed terminal commands: verify tests pass, append context note (could be a commit-message footer or PR comment), update inventory if applicable, confirm CI emitted audit-chain events, push the branch and request review. Reviewer adds `## Code Review` at merge.
