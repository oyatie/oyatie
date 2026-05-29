---
doc_class: FAQ
microservice: tasks
persona: engineer
date: 2026-05-20
doc_status: published
---

# Engineer FAQ — tasks

## When should I use `tasks` vs `workflow-engine`?

Rule of thumb: if a human is going to look at it, edit it, and check it off, it's a task. If it's a process running unattended (e.g., "every Monday at 9am, send a digest"), it's a workflow. The two integrate per ADR-TASKS-0005 — workflow-engine can create tasks (e.g., "when the deployment finishes, create a `QA-sign-off` task") and a task-state-change emits an event that workflow-engine can react to. Don't try to model long-running processes as tasks (the task-state machine is human-discretion, not deterministic); don't try to model human-CRUD as workflows (you'll re-invent the kanban board).

## Why does the dependency-graph reject some edges I "know" don't create a cycle?

Kahn's algorithm computes topological order on write. If your edge addition creates any cycle in the directed graph (where blocks-relations and blocked-by-relations are the same direction, and relates-to is bidirectional but cycle-permissive), the write is rejected. The common surprise: relates-to is treated as bidirectional for cycle-detection only on the read side; for writes, relates-to never causes a rejection (it's intentional — relates-to is non-causal). If you see a rejection mentioning relates-to, file a bug — that's an algorithm regression.

## What's the difference between `subtask` and `dependent-task`?

A subtask is part of its parent task (it inherits the parent's project + workspace + visibility-scope; completing all subtasks does not auto-complete the parent unless the parent has `auto-complete-on-subtasks` set). A dependent task is a separate task with a blocks/blocked-by relationship to another task in any project. Subtasks are hierarchy; dependencies are graph. Don't use subtasks for "wait until X is done before Y" — that's a dependency.

## Why is my task's due-date sometimes off by 1 day?

Time-zone. The task's due-date is stored as a `LocalDate` (year-month-day, no tz) per ADR-TASKS-0001 §"Date storage". Renderers in the UI apply the viewer's timezone — so a task with due-date `2026-06-15` renders as "Mon Jun 15" for a viewer in Seoul (UTC+9) and "Sun Jun 14, 23:00" for a viewer in Honolulu (UTC-10) if they have time-detail rendering on. The API returns the stored `LocalDate`; consumers must apply tz themselves. The mobile app has a known bug at the date-line transition — ticket `tasks-0184` tracks the fix.

## How do I bulk-update 500 tasks via API?

Use `oya tasks bulk-update` with `--filter` + `--patch`:

```sh
oya tasks bulk-update \
    --workspace my-workspace \
    --filter "status=Todo,project=Q3-Planning" \
    --patch '{"status":"In Progress","assignee":"jdoe"}'
```

The bulk surface is rate-limited to 5 000 tasks per request and 60 RPM per workspace; if you need more, page through. The bulk endpoint emits a single audit-chain `bulk_task_update_executed` event with the filter expression + patch + affected-count + Ed25519 signature.

## Recurring tasks are showing up too many times — what's wrong?

The recurring-task implementation follows RRULE per RFC 5545 with a subset (DAILY / WEEKLY / MONTHLY / YEARLY; no COUNT or UNTIL composition in the current admitted surface). If you see N+1 instances when you expect N, two common causes:

1. The recurring rule has `BYDAY=MO,TU,WE,TH,FR` and you're counting weekends — RRULE generates the working-week instances.
2. The workspace's `recurring_lookahead_window` is set to N+1 weeks; the µservice pre-materialises instances one window ahead so you can see them in the calendar view.

Inspect via `oya tasks recurrence inspect <task-id>` which lists the next 50 generated instances + their parent rule.

## My PR is failing the `oya-governance-tasks-cedar-fragment-shape` lane. What does it want?

Action verbs must follow the pattern `Tasks::<bounded_context>::<verb>`. Common violations: using `TasksAction` (without the `::` namespace) — old format from pre-ADR-0243. Using `Project::Create` instead of `Tasks::project::create` — Cedar fragments must use the `::` separator and the bounded_context middle segment. Run `oya gate validate tasks-cedar-fragment-shape --diff` to see per-fragment issues.

## How is the dual-context isolation enforced?

Two layers, defence-in-depth per ADR-0145:

1. Storage-layer: tasks are partitioned by `(workspace_id, context_kind ∈ {personal, professional})`. A query without `context_kind` in the where-clause is rejected by the DB-side audit; the storage layer never returns cross-context rows.
2. Cedar-layer: every `permit(...)` fragment includes `principal.active_context == resource.context_kind` as an invariant clause. The lane `oya-governance-dual-context-cedar-invariant` audits every Cedar fragment in this µservice for this clause; missing-clause is a BLOCKER.

Don't try to bypass the isolation for "convenience" cross-context features like "show me my personal + work tasks in one view". The product decision per `feedback_dual_context_strict.md` is that any cross-context surface requires an explicit, audit-logged consent flow.

## What's the difference between `assignee` and `watcher`?

Assignee: responsible for completing the task. Exactly one primary-assignee + 0+ co-assignees; tenant_class affects usage caps and billing components, not the assignee model.

Watcher: receives notifications on task changes without being responsible. 0+ watchers per task. A task creator is auto-added as a watcher if they're not the assignee.

The audit-chain emits separate events for assignee changes vs watcher changes — `task_assignee_changed` is a higher-severity event class than `task_watcher_added` because assignee changes affect work-distribution accountability.

## Why is the kanban board so slow with my 5 000-task project?

Probably hitting a demo_trial usage cap or a non-virtualized board path. Board view renders every active task as a card; at 5 000 tasks the JS framework's render budget is exhausted (60 fps → ~ 16 ms render budget per frame; 5 000 cards × 0.3 ms per card = 1 500 ms render). Two options:

1. Convert the workspace to paid tenant_class or raise the paid usage budget; the board view virtualises rendering so only visible cards are rendered.
2. Archive completed-and-older-than-90-days tasks; they remain in the list view's "archived" filter but don't show in board.

The board-view virtualisation work is tracked at `IP-021-board-view-virtualisation.md`; it is not feature-class gated.
