---
doc_class: MigrationPlaybook
microservice: tasks
vendor: Atlassian Jira Cloud
date: 2026-05-20
doc_status: published
---

# Migration playbook — Jira Cloud → oyatie tasks

Audience: an oyatie tenant moving an existing Atlassian Jira Cloud instance to oyatie. The playbook covers Scrum / Kanban / Bug-tracker boards; Service-Management (ITSM) is out of scope (use `microservices/itsm/migration-playbooks/from-jira-service-management.md` instead).

Note: counterpart vendors may use pricing tiers; Oyatie does not. Oyatie uses binary tenant_class (`demo_trial`, `paid`) and paid billing components.

## Mapping at-a-glance

| Jira Cloud object | oyatie tasks object | Field-level deltas |
|---|---|---|
| Issue | Task | Jira's `summary` → oyatie `title`; Jira `description` (ADF) → oyatie `description` (Markdown via ADF→MD converter); Jira `issuetype` → oyatie tag (configurable) |
| Sub-task | Subtask (1-level) | Jira's nested subtasks deeper than 1 level → flattened with a `parent-chain` tag annotating original depth |
| Epic | Epic | Jira's `Epic Link` field → oyatie's `epic_id` |
| Story Point custom-field | Story Points custom-field (Number) | Direct map; preserves value |
| Sprint | Sprint | Direct map when tenant_class and usage caps admit the import; preserves start/end dates and goal text |
| Project | Project | 1:1 |
| Components | Labels (per project) | Jira components are project-scoped lists; oyatie labels are workspace-scoped — namespace conflicts handled by prefixing with project key |
| Fix Version / Version | Milestone | Direct map; preserves due-date |
| Priority | Priority (P0-P3) | Highest/Highest→P0; High→P1; Medium→P2; Low/Lowest→P3 |
| Status | Status | Custom workflows preserved; default Jira Software workflow maps to oyatie's default |
| Assignee | Assignee | Mapped by email; unmapped users default to `__unassigned__` |
| Reporter | Watcher (first watcher) | Jira's Reporter is single-valued; oyatie's Reporter equivalent is "first watcher" |
| Watchers | Watchers | Direct map |
| Comments | Comments | ADF→MD conversion; preserves order; preserves @-mentions (remapped to oyatie user IDs) |
| Attachments | Drive attachments | Pulled to oyatie's drive µservice; URL references preserved |
| Issue links (blocks/is blocked by/relates to) | Dependencies (blocks/blocked-by/relates-to) | 1:1; cycle detection runs post-import and surfaces issues |
| Issue links (duplicates/is duplicated by) | Tag `dup-of:<task-id>` | Not modelled as graph; preserved as a tag |
| Sub-task time tracking | Time-tracking entries | Preserves logged time per user when the paid billing component and usage policy admit time-tracking import |
| Worklog | Time-tracking entries | Direct map |
| Saved filter (JQL) | Saved filter (oyatie query syntax) | JQL→oyatie-query converter handles ~ 85% of JQL; flagged for human review otherwise |

## Step 1 — Inventory + export (≤ 30 min for 5 000 issues)

Jira Cloud's bulk export API has rate limits; the importer paginates at 100 issues/page with a 1 s inter-page delay (Jira's quota is 10 req/s; we run at ~ 100 req/s sustained which is fine within their bucket).

```sh
oya tasks import inventory \
    --vendor jira-cloud \
    --instance-url https://your-org.atlassian.net \
    --email <admin-email> \
    --api-token <jira-api-token> \
    --project <jira-project-key> \
    --out inventory/jira-<key>.yaml
```

Inventory file is ~ 50 MB per 5 000 issues (text + metadata; attachments are not in this file). Review the inventory for:

- Custom issue types not in the standard set ("Spike", "Tech Debt", "Initiative" — common Atlassian-marketplace add-ons). The importer maps these to tags by default; you can edit the YAML to remap.
- Custom workflows. Jira workflow states map to oyatie statuses, but if your team has a 12-step workflow, oyatie defaults to 4-step — you'll need to define a custom status schema in the target project before import.
- ScriptRunner / Jira Misc Workflow Extensions automations. These do NOT migrate. The importer flags them in `inventory.warnings.yaml`; you re-author equivalent workflows in `workflow-engine` post-import.

## Step 2 — Pre-create target project + workflow + custom-fields (≤ 30 min)

Before import, the target oyatie project must exist with:

- Statuses matching Jira's workflow (if non-default). Define via `oya tasks workspace status-schema set --project <id> --statuses Todo,InProgress,InReview,Blocked,Done`.
- Custom fields matching Jira's custom fields (the importer creates them at import time if missing, but pre-creating gives you control over the field order + descriptions).
- Members matching Jira's project members (via the user-mapping step below).

## Step 3 — User mapping (≤ 15 min for ≤ 100 users)

```sh
oya tasks import map-users \
    --inventory inventory/jira-<key>.yaml \
    --strategy by-email \
    --out mapping/jira-<key>-users.yaml
```

Common edits:

- Jira's bot accounts (jenkins-bot, github-bot, scriptrunner-system-user) → map to oyatie's system principal `__system__` so their commit/assignee history is preserved without taking a user seat.
- Disabled Jira accounts → map to `__disabled__` to retain audit trail without granting access.

## Step 4 — JQL → oyatie query conversion

```sh
oya tasks import convert-filters \
    --inventory inventory/jira-<key>.yaml \
    --out mapping/jira-<key>-filters.yaml
```

The converter handles JQL constructs:

- `project = <KEY>` → `project = <oyatie-id>`.
- `assignee = currentUser()` → `assignee = ${me}`.
- `status = "In Progress"` → `status = "InProgress"` (oyatie uses no-spaces by default; configurable).
- `due > now() AND due < now("7d")` → `due BETWEEN today AND today+7d`.
- `text ~ "foobar"` → `text contains "foobar"` (Meilisearch-backed).
- `priority IN (Highest, High)` → `priority IN (P0, P1)`.

Constructs that do NOT auto-convert (flagged for human review):

- `worklogAuthor = X` (oyatie's time-tracking model differs; query manually).
- `parent = <KEY>` (oyatie's subtask hierarchy is different; rewrite as `parent.title contains <title>` or use the parent's oyatie ID).
- Plugin-specific JQL (Tempo, Structure, etc.).

## Step 5 — Dry-run

```sh
oya tasks import dry-run \
    --inventory inventory/jira-<key>.yaml \
    --user-mapping mapping/jira-<key>-users.yaml \
    --filter-mapping mapping/jira-<key>-filters.yaml \
    --target-workspace my-workspace \
    --target-project <oyatie-project-id>
```

Review the dry-run report carefully. The most-common surprise is comment-ordering — Jira's API returns comments in chronological order but Jira ADF has `creationDate` fields that can drift from server-time; the importer uses ADF `creationDate` and may reorder comments slightly. The dry-run report flags any comment whose order changes by > 5 positions.

## Step 6 — Live import

```sh
oya tasks import run \
    --inventory inventory/jira-<key>.yaml \
    --user-mapping mapping/jira-<key>-users.yaml \
    --filter-mapping mapping/jira-<key>-filters.yaml \
    --target-workspace my-workspace \
    --target-project <oyatie-project-id> \
    --throughput-tasks-per-min 200
```

For 5 000 issues at 200 tasks/min, expect ~ 25 min. The Jira API has a 1 000 req/min hard cap; we cap throughput at ~ 600 req/min sustained to leave headroom for your own users continuing to use Jira during the import window.

## Step 7 — Attachments + worklogs

```sh
oya tasks import attachments \
    --inventory inventory/jira-<key>.yaml \
    --jira-instance-url https://your-org.atlassian.net \
    --jira-token <jira-api-token> \
    --target-project <oyatie-project-id>

oya tasks import worklogs \
    --inventory inventory/jira-<key>.yaml \
    --target-project <oyatie-project-id>
```

Both subcommands are idempotent — they re-run safely if interrupted.

## Step 8 — Verify

```sh
oya tasks import verify \
    --inventory inventory/jira-<key>.yaml \
    --target-project <oyatie-project-id>
```

Verify checks: issue count, comment count per issue, attachment count per issue, status of every issue, assignee of every issue, due-date of every issue, custom-field values of every issue, sprint membership of every issue, epic-link of every issue.

A 5 000-issue project takes ~ 8 min to verify. The output is either `PASS` or per-issue delta list.

## Step 9 — Sprint cutover

Critical: if your team is mid-sprint when you import, freeze Jira sprint changes during the import window. Once import completes:

1. Verify the Sprint object matches in oyatie via `oya tasks sprint diff --jira-sprint <id> --oyatie-sprint <id>`.
2. Cut over the sprint board UI to oyatie (change the team's URL bookmark).
3. Run sprint as normal in oyatie; Jira sprint becomes read-only.
4. After sprint close (≤ 2 weeks), archive the Jira sprint and Jira project (don't delete; retain for 30 days minimum).

## Step 10 — Sunset evidence

```sh
oya tasks import sunset-evidence \
    --vendor jira-cloud \
    --jira-project <jira-project-key> \
    --oyatie-project <oyatie-project-id> \
    --out evidence/migrations/jira-<key>-to-oyatie-<id>.json
```

The evidence is referenced by the `oya-governance-migration-evidence` lane and forms the auditor-visible record.

## Common pitfalls

| Pitfall | Symptom | Fix |
|---|---|---|
| Custom workflow with > 12 statuses not pre-defined | Import fails on first issue with non-default status | Pre-create the status schema in Step 2 |
| Marketplace plugin custom field types (e.g., Sprint, Epic Link) | Field mapping yields `__unknown__` | Manual mapping per field in `mapping/jira-<key>-fields.yaml` |
| Heavy JQL in saved filters with plugin-specific syntax | Filter doesn't translate | Re-author manually in oyatie's query syntax |
| Comment ADF with embedded media | Comment imports as text-only with placeholder | Re-upload media via attachments subcommand; comment auto-updates |
| Jira user without email (bot account, deactivated user) | User mapping defaults to `__unassigned__` | Edit user-mapping file before live-run |
| Linked issues across multiple Jira projects | Cross-project links broken after import | Run `oya tasks import resolve-cross-links --batch <all-imported-projects>` after all project imports complete |
