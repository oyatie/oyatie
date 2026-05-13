# Post-consensus autopilot invocation prompt — oyatie SoT cutover

Prepared 2026-05-12 by ralplan orchestrator. Used once Critic returns `APPROVE` on `.omc/plans/ralplan-oyatie-sst-consolidation.md`. Skip if the user chooses a non-autopilot route (team / ralph / manual).

---

## Skill invocation

```
Skill(
  name: "oh-my-claudecode:autopilot",
  args: "--plan .omc/plans/ralplan-oyatie-sst-consolidation.md --start-phase 2 --grit-mode session-less"
)
```

The plan replaces autopilot's Phase 0+1 (Expansion + Planning); autopilot starts directly at Phase 2 (Execution). `--grit-mode session-less` is a hint flag — if autopilot doesn't recognize it, fall back to documenting the mode in the worker prompt.

---

## Worker prompt template (passed to each execution agent autopilot spawns)

```
You are an execution agent in the oyatie SoT + grit/icm cutover. Your task is bounded by the phase assignment passed below.

## Hard rules (non-negotiable)
1. Sanctioned primitives: grit, icm, oya-agent-read. No git, no gh, ever.
   - Read state: `grit symbols`, `grit status`, `grit watch`, `icm recall -t <topic>`, `oya-agent-read log/diff/pr-view/pr-comments`.
   - Lock symbols: `grit claim --agent <your-id> --intent "<one-line>" <file::Identifier>...`
   - Land: `grit done --agent <your-id>`. Worktree is at `.grit/worktrees/<your-id>/`.
2. Session-less mode: `grit session start` is broken upstream (filed-as-todo). Do NOT invoke session. Operate on the orchestrator's base branch directly.
3. New-crate scaffolding: if your phase creates a new crate (e.g., tools/oya-agent-read/), use the scaffold-claim pattern from `.omc/specs/pre-cutover-drafts-2026-05-12.md §Draft 2`. Try Cargo.toml::workspace_members first; if not indexed, fall back to icm-as-coordination-lock under topic `scaffold-locks-oyatie`.
4. Human-orchestrator carve-outs: if your phase touches a `git mv`, `git rm`, or `gh issue create` step, STOP and emit `BLOCKED_ON_HUMAN_ORCHESTRATOR` with the exact command needed. Do NOT invoke git/gh yourself.
5. icm store after every non-trivial event per oyatie/CLAUDE.md (errors-resolved | decisions-oyatie | preferences | context-oyatie).
6. Heartbeat your claim every 5 minutes for long phases: `grit heartbeat --agent <your-id> --ttl 600`.

## Phase-specific context
- Phase ID: <P1..P10>
- Symbols to claim: <enumerated from plan §Phase X §Symbols to claim>
- Acceptance evidence required: <enumerated from plan §Phase X §Outputs / acceptance evidence>
- Maps to spec criterion: <A1..A10>

## Verification before declaring done
- Run the spec acceptance test for your criterion. Paste output.
- icm store -t context-oyatie a 2-sentence summary of what your phase achieved.
- grit done --agent <your-id>

## Forbidden
- Editing files outside your claimed symbol scope.
- Bypassing the no-git/no-gh rule "just this once".
- Marking acceptance criteria complete without the lane / command output.
- Half-finished implementations.
- Adding scope beyond your phase ID's enumerated symbols.
```

---

## Phase routing strategy

Autopilot spawns workers in parallel where the plan's phase dependencies allow. The plan's Option A is strict-phased, so most phases gate on the prior one. Where the plan permits parallelism (e.g., P5 hook+skill audit can run alongside P3 PRD citation since they touch different files), autopilot should fan out.

Suggested fan-out groups (verify against final plan after Architect/Critic revisions):

| Group | Phases that can run together | Rationale |
|---|---|---|
| G1 (serial) | P1 inventory ADR | Single ADR write; small file; no parallelism gain |
| G2 (parallel) | P2 (helper CLI scaffold) + P3 (PRD bidirectional cite) | Different file trees; independent |
| G3 (parallel) | P4 (agent memory rewrite) + P5 (hook/skill audit) | Both touch agent-instruction surfaces but in different files |
| G4 (serial gate) | banned-primitives fitness lane goes live | Must precede deletion |
| G5 (serial) | P6 archive (human-orchestrator) | git mv requires human |
| G6 (parallel after G5) | P7 delete (human) + P8 demo recording | demo is read-only; can run during the two-green-CI window |
| G7 (parallel terminal) | P9 file upstream bug + P10 authoritative-tracked lane | Both terminal, both independent |

---

## Stop conditions for autopilot

Autopilot exits when:
- All 10 spec acceptance criteria (A1-A10) pass their lanes.
- The plan's verification matrix has 100% pass.
- `oya-foundry-fitness-portfolio-citation` and `oya-foundry-fitness-banned-primitives` lanes report green on the cutover branch.
- The inventory ADR is committed and referenced from ADR-INDEX.md.
- The /goal hook's condition criteria are all met.

Autopilot must NOT mark the cutover complete if:
- Any `BLOCKED_ON_HUMAN_ORCHESTRATOR` is unresolved.
- The grit session bug remains unfiled upstream.
- The parallel-claim demo runbook is missing.
- Any of the four OPEN ledger entries (LEDG-008/017/021/024) was force-closed.

---

## Handoff back to the user

When autopilot completes (or hits a hard block), produce a 200-word handoff at `.omc/state/post-autopilot-handoff.md` with:
- Which A1-A10 criteria passed and which (if any) need human follow-up.
- The cutover branch name (if a session was opened — note that under session-less mode the cutover lands directly on main via per-agent grit done).
- The list of any `BLOCKED_ON_HUMAN_ORCHESTRATOR` events with their exact required commands.
- The /goal hook status — is the goal condition met?
- Recommended next action (one sentence).
