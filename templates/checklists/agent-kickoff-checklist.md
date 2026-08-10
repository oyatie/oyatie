---
doc_class: Checklist
checklist_id: CHK-KICKOFF
status: pending approval
purpose: |
  The agent's first 5 actions before claiming any symbol. Encodes the agentic-navigation contract from `.omc/plans/MASTERPLAN.md §6`. Walked at the start of every agent session that intends to modify the repo.
lift_target: oyatie/templates/checklists/agent-kickoff.md
enforcing_fitness_lane: oya-governance-banned-primitives (audits the first `grit claim` was preceded by the kickoff icm-read)
owner_team: axis-foundry
related:
  - .omc/plans/MASTERPLAN.md
  - docs/AGENTS.md
  - .omc/scratch/adr-draft-grit-icm-sanctioned-primitives.md
  - /templates/checklists/agent-completion-checklist.md
---

# Agent Kickoff Checklist

> The first 5 actions every agent **MUST** complete before any `grit claim`. Compact by design: a fresh agent should descend the tree in O(log n) clicks.

<!-- agent-instructions:start -->

## K1. Recall (icm)

```
icm recall-context "<change class> <axis> <slug>" --limit 5
```

Read every returned memory. If empty, that's a signal — your work has no prior context; proceed but emit a context-store at completion (per `agent-completion-checklist.md`).

## K2. Read the masterplan

Read `.omc/plans/MASTERPLAN.md` end-to-end. **MANDATORY** sections: §2 (compound principles 1-12), §6 (per-tier artifact contract), §7 (dual-audience contract).

## K3. Pick the milestone → phase → IP

1. From `MASTERPLAN.md §3 Milestone index`, pick a milestone with `status: open` or `in-progress`.
2. Open `milestones/<MNN>/INDEX.md`. From its `§Phases` table, pick an `open` phase.
3. Open `phases/<PNN-slug>/INDEX.md`. From its `§Implementation Plans` table, pick the next `open` IP.
4. Open the IP file. Read frontmatter + `§Agent prerequisites` + `§Symbols to grit-claim` + `§Acceptance test commands`.

If no `open` IP exists at the milestone-active layer, halt and emit `BLOCKED_ON_HUMAN_ORCHESTRATOR` per `/templates/checklists/escalation-checklist.md`.

## K4. Verify no symbol collision

```
oya-tooling-agent-read grit-status <symbol-1> <symbol-2> …
```

Every symbol in the IP `§Symbols to grit-claim` **MUST** show `unclaimed`. If any shows `claimed-by-<agent>`, pick a different IP (per phase INDEX `§Parallelism strategy`).

## K5. Grit claim + audit emit

```
grit claim <symbol-1> <symbol-2> … --ip IP-NNN-<slug>
```

The claim emits `EVT-GRIT-CLAIM` to the audit chain automatically. Confirm via `oya-tooling-agent-read audit-tail --last 1`.

<!-- agent-instructions:end -->

## Hard rules

- **No `git`, no `gh`** outside the documented carve-out (see `.omc/scratch/adr-draft-grit-icm-sanctioned-primitives.md §Cutover bootstrap window` + `§Glossary §Human orchestrator`). If a workflow appears to require `git`/`gh`, emit `BLOCKED_ON_HUMAN_ORCHESTRATOR`.
- **No claim before the masterplan read.** `oya-governance-banned-primitives` audits the sequence: kickoff icm-read → masterplan read → grit claim. Out-of-order sequences fail the lane.
- **No silent retry.** If `grit claim` fails, run `icm store -t errors-resolved -c "<failure mode>" -i high -k "grit,claim"` BEFORE retrying.

## Stop conditions

- IP frontmatter `final_shape_compliance: false` — refuse to claim; route to `escalation-checklist.md`.
- IP `§Symbols to grit-claim` is empty or contains non-`file::Identifier` placeholders — refuse to claim; the IP is not in final shape.
- IP `agent_prerequisites` lists a missing file — refuse to claim; emit issue creation request to human orchestrator.

## Human path (junior developer)

Same five actions: `icm recall-context` → read masterplan → descend milestone/phase/IP → check symbol availability → start working on the symbols. Use `rtk` prefixes for all terminal commands per `docs/CONSTITUTION.md §4 dual-audience clause`.
