# User execution-approval question skeleton

Used when Critic iter-N returns APPROVE on `.omc/plans/ralplan-oyatie-sst-consolidation.md`. Fires the structured AskUserQuestion that captures execution approval before any cutover phase runs.

---

## Question shape

```
AskUserQuestion(
  questions: [{
    question: "Ralplan consensus complete (Planner→Architect→Critic, iter-N). Plan at `.omc/plans/ralplan-oyatie-sst-consolidation.md` has APPROVE verdict. 12 phases (P0.5/P1/P2/P3/P3.5/P4/P5/P6/P7/P8/P9/P10) under grit session-less mode + icm + oya-tooling-agent-read sanctioned primitives. Two human-orchestrator carve-outs flagged at P6/P7/P9 (git mv / git rm / gh issue create). Execute now?",
    header: "Execute cutover",
    multiSelect: false,
    options: [
      {
        label: "Approve — execute via /autopilot (Recommended)",
        description: "Hand off to oh-my-claudecode:autopilot starting at Phase 2 (Execution). Spawns workers per phase fan-out group (G1..G7). Each worker grit-claims its phase symbols, works in .grit/worktrees/<agent>/, lands via grit done. Two BLOCKED_ON_HUMAN_ORCHESTRATOR pauses at P6/P7 await your git mv / git rm; one at P9 for gh issue create. Maximum auto-parallelism."
      },
      {
        label: "Approve — execute via /team",
        description: "Hand off to oh-my-claudecode:team. N coordinated parallel agents on shared task list, each owning a phase. Better for users who want explicit task-list visibility per agent. Same grit/icm primitives; same human-orchestrator pauses."
      },
      {
        label: "Approve — execute via /ralph",
        description: "Hand off to oh-my-claudecode:ralph. Sequential agent with verification loop per phase. Slowest of the three; highest verification depth. Use when phase-level rollback would be expensive."
      },
      {
        label: "Approve — but pause after Phase 1 (inventory ADR)",
        description: "Execute P0.5 and P1 only (scaffold-claim ADR + inventory ADR). Stop. You review the inventory ADR + cutover ADR draft before authorizing P2 onward. Safest. Highest user-touch."
      },
      {
        label: "Request changes — re-iterate before execution",
        description: "Send the plan back to Planner with your specific change requests. Loops back into ralplan iter-(N+1). Use if the Critic-approved plan still has something you want different."
      }
    ]
  }]
)
```

## Notes for the orchestrator

- The "Approve — pause after Phase 1" option is the most conservative; recommend it if user has not yet seen the inventory ADR draft.
- All four "Approve" options share the same grit-mode preconditions: session-less mode (grit session start bug), icm-coordination-lock fallback for new-crate scaffolding, no agent-side git/gh, audit-emitting helper for all read-side beyond grit.
- "Request changes" reroutes to iter-(N+1) Planner — only viable if iter ≤ 4 (ralplan rule: max 5 iterations).
- After user picks an approve option, the orchestrator:
  1. icm store the chosen execution route + iter-N consensus snapshot.
  2. Invoke the chosen Skill with `.omc/plans/ralplan-oyatie-sst-consolidation.md` as input.
  3. Update TaskList to move Phase 5 Execution Bridge → completed; create per-phase tasks owned by execution agents.
  4. Update /goal with the post-execution success criteria (already drafted at `.omc/specs/post-consensus-autopilot-prompt.md`).

## Open-questions to attach to the same Question call (optional)

Two open-questions are flagged user-pending in `critic-review-iter-1.md`:
- Q3 carve-out scope (P6/P7/P9 human-orchestrator git/gh)
- Q6 retention policy (60 vs 90 days for archive)

If user picks any Approve option, the orchestrator may attach these as inline confirmations:
- "Confirm the cutover proceeds under 'human orchestrator may invoke git mv / git rm / gh issue create at P6/P7/P9; agents never do' interpretation? (Y/N)"
- "Confirm archive retention = 90 days before final DELETE? (Y/N or supply alternative)"

Or defer them to the autopilot's first BLOCKED_ON_HUMAN_ORCHESTRATOR pause.
