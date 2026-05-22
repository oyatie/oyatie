---
doc_class: Checklist
checklist_id: CHK-KICKOFF
status: Accepted
date: 2026-05-12
purpose: |
  The agent's first 5 actions before claiming any symbol. Encodes the agentic-navigation contract from `.omc/plans/MASTERPLAN.md §6`. Walked at the start of every agent session that intends to modify the repo.
owner_team: axis-foundry
related:
  - .omc/plans/MASTERPLAN.md
  - docs/AGENTS.md
  - docs/checklists/agent-completion-checklist.md
adrs_cited:
  - ADR-0052  # inventory ledger (migration-class claim prerequisites)
  - ADR-0054  # scaffold-claim pattern (symbol verification)
doc_status: published
---

# Agent Kickoff Checklist


<!-- agent-instructions:start -->


```
```


## K2. Read the masterplan

Read `.omc/plans/MASTERPLAN.md` end-to-end. **MANDATORY** sections: §2 (compound principles 1-12), §6 (per-tier artifact contract), §7 (dual-audience contract).

## K3. Pick the milestone → phase → IP

1. From `MASTERPLAN.md §3 Milestone index`, pick a milestone with `status: open` or `in-progress`.
2. Open `milestones/<MNN>/INDEX.md`. From its `§Phases` table, pick an `open` phase.
3. Open `phases/<PNN-slug>/INDEX.md`. From its `§Implementation Plans` table, pick the next `open` IP.

If no `open` IP exists at the milestone-active layer, halt and emit `BLOCKED_ON_HUMAN_ORCHESTRATOR` per `docs/checklists/escalation-checklist.md`.

## K4. Verify no symbol collision

```
```



```
```


<!-- agent-instructions:end -->

## Hard rules

- **No `git`, no `gh`** outside the documented carve-out (see ADR-0053 §Cutover bootstrap window + §Glossary §Human orchestrator). If a workflow appears to require `git`/`gh`, emit `BLOCKED_ON_HUMAN_ORCHESTRATOR`.

## Stop conditions

- IP frontmatter `final_shape_compliance: false` — refuse to claim; route to `escalation-checklist.md`.
- IP `agent_prerequisites` lists a missing file — refuse to claim; emit issue creation request to human orchestrator.

## Human path (junior developer)

