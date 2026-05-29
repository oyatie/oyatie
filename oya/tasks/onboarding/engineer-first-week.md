---
doc_class: Onboarding
microservice: tasks
persona: engineer
date: 2026-05-20
doc_status: published
---

# Engineer onboarding — first 5 working days

Audience: a new engineer on the `tasks` µservice team (axis-tasks). By Day-5 they will have shipped a small bug-fix PR, understand the dependency-graph cycle-prevention path, and shadowed a customer-facing incident.

## Day 1 — Domain tour

Read in order:

1. `PRD.md` end-to-end (≤ 30 min). Focus on the 7 Tenant Outcomes — they encode why we exist.
2. `ARCHITECTURE.md` §principals + §cedar-gates + §events (≤ 25 min).
3. The four key ADRs:
   - `decisions/ADR-TASKS-0001-board-and-views-architecture.md` (the view-rendering split).
   - `decisions/ADR-TASKS-0002-dependency-graph-cycle-prevention.md` (the algorithm).
   - `decisions/ADR-TASKS-0005-bidirectional-workflow-engine-integration.md` (the boundary with workflow-engine).
   - `decisions/ADR-TASKS-0006-dual-context-personal-professional-isolation.md` (the isolation invariant).

End-of-day: you can answer "what is the difference between `tasks` and `workflow-engine`?" without notes. Hint: tasks is the human-CRUD-primitive for work-items; workflow-engine is durable-execution for processes. They cross at task-state-change → workflow-engine trigger.

## Day 2 — Stand up a local workspace

Pre-req: local cell + `tasks` µservice running.

```sh
make cell-up
make tasks-up
```

Create a workspace:

```sh
oya tasks workspace create --name my-onboarding --owner $(whoami)
```

Add a project:

```sh
oya tasks project create --workspace my-onboarding --name "Tutorial Project"
```

Create your first 10 tasks via the CLI:

```sh
for i in {1..10}; do
  oya tasks task create \
    --workspace my-onboarding \
    --project "Tutorial Project" \
    --title "Tutorial task $i" \
    --assignee $(whoami) \
    --due-in-days 7
done
```

Open the web UI at `https://tasks.local.oyatie.test`. You should see 10 tasks in the List view. Switch to Board view; you should see them all in the Todo column (default status).

## Day 3 — Dependency-graph

The dependency-graph is the most algorithmically interesting surface. Add a dependency:

```sh
oya tasks dep add \
    --source <task-2-id> \
    --target <task-1-id> \
    --type blocks
```

This says "task-2 blocks task-1" (i.e., task-1 cannot start until task-2 completes).

Now try to add a cycle:

```sh
oya tasks dep add \
    --source <task-1-id> \
    --target <task-2-id> \
    --type blocks
```

Expected: `ERROR: would create cycle: task-1 -[blocks]-> task-2 -[blocks]-> task-1`. The cycle-prevention algorithm is Kahn's topological-sort-on-write — read `decisions/ADR-TASKS-0002-dependency-graph-cycle-prevention.md` §"Algorithm" before Day 4.

Add a longer chain (5 tasks, all blocking the next). Now try to mark task-1 (the deepest blocked one) as `In Progress`:

Expected: `WARN: task-1 has 4 incomplete blockers (task-2, task-3, task-4, task-5). Block-warning enabled — proceed with --force?`. The warning is configurable per-workspace (`workspace.config.block_warning_mode ∈ {warn, hard-block, off}`).

## Day 4 — Shipping a bug-fix

The current bug list is at `oya tasks dev bugs --severity p2 --status open`. Pick a small one — ideally a list-render-edge-case (the list-view code is well-tested and easy to extend; the gantt-render code has subtle date-math you should not touch in your first week).

Sample bug: "List view sort-by-due-date doesn't put 'no due date' tasks last consistently".

Fix in `crates/oya-tasks-app/src/views/list.rs`. The relevant function is `sort_tasks_by_due_date`. Look at the existing tests in `crates/oya-tasks-app/tests/views/list_sort.rs`; add a test for the no-due-date case; then fix.

PR via `oya submit --title "fix(tasks): no-due-date sorting in list view"`. The PR enters the Foundry pipeline. Expect:

- Lane `oya-governance-tasks-cedar-fragment-shape`: green (you didn't touch Cedar).
- Lane `oya-governance-tasks-event-schema`: green (you didn't touch events).
- Lane `oya-tasks-tests`: green (your new test passes).
- Reviewer-agent: pulls the diff and posts an APPROVE/REQUEST_CHANGES.

If reviewer-agent requests changes, address them in a follow-up commit — do not rebase, do not amend (per `feedback_self_merge_via_contract_path.md` we always add commits, never rewrite history during review).

## Day 5 — Shadow an incident

Schedule with axis-tasks on-call for a 90-min incident-shadow window (Wed afternoons typically). You will:

1. Watch a triage. The on-call gets a P2 page; they read the alert, query Grafana, check the audit-chain.
2. See the customer-comms surface — `tasks` incidents get a customer-side post on `status.oyatie.io` within 5 min of P2-or-above acknowledgment.
3. Watch the post-mortem template (which lives at `runbooks/post-mortem-template.md`) get drafted.

End-of-week: you have shipped one PR, observed one incident, and can sketch the request flow of a "create task" operation from web UI through ingress → API gateway → tasks-app → tasks-domain → ontology object insert → audit-chain emission.
