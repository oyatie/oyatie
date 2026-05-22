---
doc_class: Checklist
checklist_id: CHK-COMPLETE
status: Accepted
date: 2026-05-12
purpose: |
enforcing_fitness_lane: oya-governance-banned-primitives + oya-governance-audit-emission
owner_team: axis-foundry
related:
  - .omc/plans/MASTERPLAN.md
  - docs/AGENTS.md
  - docs/checklists/agent-kickoff-checklist.md
  - docs/checklists/done-definition-checklist.md
  - docs/checklists/per-implementation-plan-checklist.md
adrs_cited:
  - ADR-0052  # inventory ledger (migration-class completion)
  - ADR-0054  # scaffold-claim (symbol release)
doc_status: published
---

# Agent Completion Checklist


<!-- agent-instructions:start -->

## C1. Verify acceptance criteria

Run every command from IP `§Acceptance test commands`. Capture stdout via `oya-tooling-agent-read run-evidence <cmd>` (ADR-0053 sanctioned primitive). Paste outputs into the PR body `## Verification` section.

If any command fails, **do not** proceed. Re-walk the work, or halt and emit `BLOCKED_ON_HUMAN_ORCHESTRATOR` per `docs/checklists/escalation-checklist.md`.



```
  -t context-<project> \
  -i high \
  -k "M0N,P0N,IP-NNN,<axis>"
```

If the IP shipped a mechanical prevention for a prior failure, **also** emit:

```
  -t errors-resolved \
  -c "<failure mode description> prevented by <mechanical prevention name>" \
  -i high \
  -k "MFL-NNNN,<prevention-name>"
```

## C3. Update inventory ledger (if migration-class)

Per `docs/checklists/inventory-update-checklist.md` (ADR-0052) — append a row to the active inventory ledger naming: source path, target path, archive path (if any), tombstone (if delete), audit emission ID, agent or human orchestrator. *Lane:* `oya-governance-inventory-tracker`.

## C4. Audit-chain emission

Confirm:
- `EVT-IP-MERGED` emitted with IP ID + merge SHA + symbol list.
- (If capability publish) `oya.foundry.capability.invoked` topic registered.
- (If runbook author) `oya.ops.runbook.invoked` topic resolvable.

Paste the `EVT-*` IDs into PR `## Evidence`. *Lane:* `oya-governance-audit-emission`.


```
```


<!-- agent-instructions:end -->

## Hard rules

- **No `git push` from agent.** PR creation is via `oya-tooling-agent-read pr-create` (read-only adapter wraps `gh pr create` behind a controlled verb, audit-emitting per ADR-0053). If unavailable, halt and emit `BLOCKED_ON_HUMAN_ORCHESTRATOR`.
- **No silent loop exit.** If running inside Ralph / autopilot / ultrawork / team loop, re-walk the `done-definition-checklist.md` before cancelling (per `docs/AGENTS.md §Long-running loop rule`).

## Loop-cancellation

Cancel via `/oh-my-claudecode:cancel` **only** when (a) the change is complete and verified per `done-definition-checklist.md`, OR (b) the loop is structurally blocked (then emit `BLOCKED_ON_HUMAN_ORCHESTRATOR`).

## Stop conditions


## Human path (junior developer)

