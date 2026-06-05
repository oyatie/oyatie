---
doc_class: Checklist
checklist_id: CHK-KICKOFF
status: pending approval
purpose: |
  The agent's first 5 actions before claiming any symbol. Encodes the agentic-navigation contract from `.omc/plans/MASTERPLAN.md §6`. Walked at the start of every agent session that intends to modify the repo.
lift_target: oyatie/docs/checklists/agent-kickoff.md
enforcing_fitness_lane: repo-hygiene-automation-check (audits current git/gh/Buck2 lane primitives)
owner_team: axis-foundry
related:
  - .omc/plans/MASTERPLAN.md
  - docs/AGENTS.md
  - /templates/checklists/agent-completion-checklist.md
---

# Agent Kickoff Checklist

> The first 5 actions every agent **MUST** complete before editing in a PR lane. Compact by design: a fresh agent should descend the tree in O(log n) clicks.

<!-- agent-instructions:start -->

## K1. Inspect current repo state

```
git status --short --branch
git log --oneline -5
```

Read the latest branch state and open PR list before trusting stale plan context.

## K2. Read the masterplan

Read `.omc/plans/MASTERPLAN.md` end-to-end. **MANDATORY** sections: §2 (compound principles 1-12), §6 (per-tier artifact contract), §7 (dual-audience contract).

## K3. Pick the milestone → phase → IP

1. From `MASTERPLAN.md §3 Milestone index`, pick a milestone with `status: open` or `in-progress`.
2. Open `milestones/<MNN>/INDEX.md`. From its `§Phases` table, pick an `open` phase.
3. Open `phases/<PNN-slug>/INDEX.md`. From its `§Implementation Plans` table, pick the next `open` IP.
4. Open the IP file. Read frontmatter + `§Agent prerequisites` + `§Lane-owned paths / symbols` + `§Acceptance test commands`.

If no `open` IP exists at the milestone-active layer, halt and emit `BLOCKED_ON_HUMAN_ORCHESTRATOR` per `/templates/checklists/escalation-checklist.md`.

## K4. Verify lane ownership is disjoint

```
git diff --name-only origin/dev...HEAD
gh pr list --base dev --state open --json number,headRefName,title
```

Every path in the IP `§Lane-owned paths / symbols` **MUST** be disjoint from active PR lanes. If paths collide, split the IP or pick a different lane (per phase INDEX `§Parallelism strategy`).

## K5. Create isolated branch/worktree + PR

```
git worktree add /tmp/<lane-worktree> -b <short-lived-branch> origin/dev
gh pr create --base dev --head <short-lived-branch>
```

The PR is the temporary publication/lane boundary while the native Rust SCM substrate is built.

<!-- agent-instructions:end -->

## Hard rules

- **Use plain `git`, `gh`, and Buck2 only.** Retired local VCS/governance wrappers are not SCM or CI authority.
- **No lane before the masterplan read.** Repo-hygiene checks audit the current pointer-thin docs and Buck2/Prow authority.
- **No silent retry.** If a lane collision appears, record the collision in PR notes/evidence and split or rebase before continuing.

## Stop conditions

- IP frontmatter `final_shape_compliance: false` — refuse to claim; route to `escalation-checklist.md`.
- IP `§Lane-owned paths / symbols` is empty or contains placeholders — refuse to edit; the IP is not in final shape.
- IP `agent_prerequisites` lists a missing file — refuse to claim; emit issue creation request to human orchestrator.

## Human path (junior developer)

Same five actions: inspect repo/PR state → read masterplan → descend milestone/phase/IP → check lane ownership → create a short-lived branch/worktree and PR. Use terminal-safe commands copied exactly from the checklist.
