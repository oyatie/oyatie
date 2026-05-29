---
doc_class: CompetitiveBenchmark
title: Competitor Parity Matrix
microservice: tasks
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-tasks + council-architecture
deciders: axis-tasks, council-architecture, gtm-customer-success
related_adrs: [ADR-0123, ADR-0135, ADR-0131, ADR-0132, ADR-0133, ADR-TASKS-0001, ADR-TASKS-0002, ADR-TASKS-0005, ADR-TASKS-0006]
related_artifacts:
  - microservices/tasks/PRD.md (§Competitive Benchmark)
  - /specs/hyperscaler-gates.json (HG-TASKS gate)
review_cadence: bi-annually + on every new competitor entrant
doc_status: published
---

# Competitor Parity Matrix (tasks µservice)

## Purpose

Quantitative + qualitative parity comparison against industry-leading
task / project management products. Drives `oya-governance-
hyperscaler-maturity-claims` gate per HG-TASKS (ADR-0123) and
constrains what gtm-customer-success can claim in tenant sales
conversations. Re-validated bi-annually because the task-management
landscape moves quickly (Linear's AI assistant, ClickUp 3.0, Notion
Calendar acquisition, Height AI-first, Plane OSS rise).

## Competitor Set

| Competitor | Product / surface | Primary differentiator | Source |
|---|---|---|---|
| Asana | enterprise task management | projects + custom fields + portfolio + goals | `developers.asana.com` |
| Trello | board-first | kanban + power-ups + Butler automation | `developer.atlassian.com/cloud/trello` |
| Linear | engineering-focused issue tracker | issue + cycles + roadmap + Linear Method + AI | `developers.linear.app` |
| Atlassian Jira | enterprise issue tracker | issue + epic + sprint + JQL + workflow | `developer.atlassian.com/cloud/jira` |
| Monday.com | flexible boards | item + board + automation | `developer.monday.com` |
| ClickUp | "everything app" | task + doc + chat + goal | `clickup.com/api` |
| Notion Database | wiki + database | unified database; rich custom fields; AI Q&A | `developers.notion.com` |
| Airtable Tasks | spreadsheet-database | tabular + form + automation | `airtable.com/developers/web/api` |
| Todoist | personal + small-team | natural-language input + recurring | `developer.todoist.com` |
| Microsoft Planner + To Do | M365 pair | Planner (board) + To Do (personal) + Copilot | `learn.microsoft.com/graph/api/resources/planner-overview` |
| Apple Reminders | iCloud reminders | CalDAV-Tasks VTODO; native iOS/macOS | `developer.apple.com/documentation/eventkit` |
| Google Tasks | G Suite tasks | basic personal task | `developers.google.com/tasks` |
| Basecamp | small-team simple | to-do list + message + schedule | `github.com/basecamp/bc3-api` |
| Wrike | enterprise project | task + Gantt + custom workflow | `developers.wrike.com` |
| Smartsheet | spreadsheet-project | sheet + Gantt + form | `smartsheet-platform.github.io/api-docs/` |
| Height | AI-first task | tasks + AI auto-categorise | (proprietary) |
| Shortcut | engineering | story + epic + iteration | `developer.shortcut.com` |
| Plane | OSS Jira-like | issue + cycle + module | `plane.so/docs/api` |
| TaskJuggler | gantt scheduling OSS | resource + duration + critical path | `taskjuggler.org` |

## Feature Parity Matrix

### Core task management

| Capability | oyatie | Asana | Linear | Jira | Trello | Monday | ClickUp | Notion | Todoist |
|---|---|---|---|---|---|---|---|---|---|
| Tasks (CRUD with title/desc/status/priority/assignee/due/labels/parent) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Custom fields (text/number/date/dropdown/multi-select/person/url/checkbox) | ✅ strict-coerce | ✅ | ✅ | ✅ | partial | ✅ | ✅ | ✅ | partial |
| **Strict custom-field type coercion (refuse "1"→number)** | ✅ (differentiator) | partial | partial | partial | ❌ | partial | partial | partial | ❌ |
| Subtasks / parent-child | ✅ | ✅ | ✅ | ✅ | partial (checklist) | ✅ | ✅ | ✅ | ✅ |
| Project / list collection | ✅ | ✅ | ✅ | ✅ | ✅ (boards) | ✅ | ✅ | ✅ | ✅ |
| Comments + reactions + threads | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Attachments (cross-µservice to drive) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Time-tracking (start/stop + manual; M02-onward) | ✅ M02-onward | ❌ (3rd party) | partial | ✅ | partial | ✅ | ✅ | ❌ | partial |
| Legal hold on tasks | ✅ | partial | ❌ | partial (Atlassian Vault) | ❌ | ❌ | ❌ | ❌ | ❌ |
| Bulk edit ≥100 tasks | ✅ AC-05 SLO | ✅ | ✅ | ✅ | partial | ✅ | ✅ | partial | partial |

### View engine

| Capability | oyatie | Asana | Linear | Jira | Trello | Monday | ClickUp | Notion | Todoist |
|---|---|---|---|---|---|---|---|---|---|
| List view | ✅ | ✅ | ✅ | ✅ | partial | ✅ | ✅ | ✅ | ✅ |
| Board (kanban) view | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | partial |
| Gantt view | ✅ M03 | ✅ (Timeline) | ✅ | ✅ | partial | ✅ | ✅ | partial | ❌ |
| Calendar view | ✅ M03 (bridge to calendar µservice) | ✅ | ✅ | ✅ | partial | ✅ | ✅ | ✅ | partial |
| Timeline view | ✅ M03 | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ❌ |
| Table view | ✅ | ✅ | ✅ | ✅ | partial | ✅ | ✅ | ✅ | ❌ |
| Saved filters + view templates | ✅ | ✅ | ✅ | ✅ (JQL) | ✅ | ✅ | ✅ | ✅ | partial |
| Real-time DnD board collaboration | ✅ M03 (Loro CRDT collab-edit description; deterministic DnD) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| Multi-view per project (switching) | ✅ | ✅ | ✅ | partial | partial | ✅ | ✅ | ✅ | partial |

### Dependency + Agile

| Capability | oyatie | Asana | Linear | Jira | Trello | Monday | ClickUp | Notion | Todoist |
|---|---|---|---|---|---|---|---|---|---|
| Dependency edges (blocks/blocked-by/relates-to) | ✅ | ✅ | ✅ | ✅ | partial | ✅ | ✅ | partial | ❌ |
| **Cycle prevention at write time (correctness SLO 100%)** | ✅ (differentiator) | ❌ (lazy detect) | ❌ (lazy) | ❌ (lazy) | ❌ | ❌ | ❌ | ❌ | ❌ |
| Critical-path query | ✅ | partial | ❌ | partial | ❌ | ❌ | partial | ❌ | ❌ |
| Sprint / iteration / cycle | ✅ M03 | partial | ✅ | ✅ | ❌ | partial | ✅ | partial | ❌ |
| Milestone + roadmap | ✅ | ✅ (Goals) | ✅ | ✅ | ❌ | partial | ✅ | partial | ❌ |
| Portfolio rollup (cross-project) | ✅ | ✅ (Portfolios) | ✅ | ✅ (Advanced Roadmaps) | ❌ | ✅ | ✅ | ❌ | ❌ |
| Custom status workflow per project | ✅ | partial | ✅ | ✅ | partial | ✅ | ✅ | ✅ | partial |
| Epic-roadmap | ✅ | ✅ | ✅ | ✅ | ❌ | partial | partial | ❌ | ❌ |
| Recurring tasks | ✅ RFC 5545 RRULE subset | partial | ❌ | partial | partial | ✅ | ✅ | ✅ | ✅ |

### Search + Automation

| Capability | oyatie | Asana | Linear | Jira | Trello | Monday | ClickUp | Notion | Todoist |
|---|---|---|---|---|---|---|---|---|---|
| Cross-project full-text search | ✅ Meilisearch | ✅ | ✅ | ✅ (JQL) | partial | ✅ | ✅ | ✅ | partial |
| Saved searches | ✅ | ✅ | ✅ | ✅ (JQL filters) | partial | ✅ | ✅ | ✅ | partial |
| **Per-tenant Meilisearch index with degraded-mode fallback to Postgres** | ✅ M03 (differentiator) | ❌ | ❌ | partial | ❌ | ❌ | ❌ | ❌ | ❌ |
| Automation rules (in-µservice) | partial (cross-µservice to workflow-engine) | ✅ (Rules) | ✅ | ✅ (Automation) | ✅ (Butler) | ✅ | ✅ | partial | partial |
| **Bidirectional workflow-engine bridge** | ✅ M03 (differentiator) | ❌ (one-way only via Zapier/etc.) | ❌ | partial | ❌ | ❌ | ❌ | ❌ | ❌ |
| Webhooks | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

### Importers + Exporters

| Capability | oyatie | Asana | Linear | Jira | Trello | Monday | ClickUp |
|---|---|---|---|---|---|---|---|
| CSV import | ✅ | ✅ | ✅ | ✅ | partial | ✅ | ✅ |
| Jira import | ✅ M03 (full epic + sprint + issue + comment + attachment + custom-field) | ✅ | ✅ | n/a | partial | ✅ | ✅ |
| Asana import | ✅ M03 | n/a | ✅ | partial | ❌ | partial | ✅ |
| Trello import | ✅ M03 | ✅ | partial | partial | n/a | partial | ✅ |
| Linear import | ✅ M03 (full team + cycle + issue + project + custom-field) | partial | n/a | ❌ | ❌ | ❌ | partial |
| Todoist import | ✅ M03 | ❌ | ❌ | ❌ | ❌ | partial | partial |
| **Strict assignee resolution (refuse ambiguous)** | ✅ (differentiator; Hyrum #7) | ❌ (silent fuzzy) | ❌ (silent) | ❌ (silent) | ❌ | ❌ | ❌ |
| CSV export | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| JSON export | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

### Privacy + isolation

| Capability | oyatie | Asana | Linear | Jira | Trello | Monday | ClickUp |
|---|---|---|---|---|---|---|---|
| Dual-context (Personal/Professional) structural isolation | ✅ (differentiator) | ❌ (acct switching only) | ❌ | ❌ | ❌ | ❌ | ❌ |
| E2E encryption at rest (Tenant-DEK) | ✅ professional context | partial | partial | partial | ❌ | ❌ | ❌ |
| Audit-chain for every task lifecycle | ✅ Ed25519 + Merkle | partial (Audit Log) | partial | partial (UAL) | partial | partial | ❌ |
| Per-jurisdiction retention (11 packs) | ✅ M03 | partial | partial | partial | ❌ | ❌ | ❌ |
| Per-tenant Postgres RLS | ✅ | partial | partial | partial | ❌ | partial | ❌ |
| Cross-tenant data leak prevention (Meilisearch tenant prefix) | ✅ (differentiator) | partial | partial | partial | partial | partial | partial |

### AI + assist (autonomy tiers)

| Capability | oyatie | Asana | Linear | Jira | Notion | ClickUp | Height |
|---|---|---|---|---|---|---|---|
| T0 next-task suggest | ✅ M03 | partial (Smart Status) | ✅ | partial (Atlassian Intelligence) | ✅ (AI Q&A) | ✅ (Brain) | ✅ |
| T0 title / agenda suggest | ✅ M03 | partial | ✅ | partial | ✅ | ✅ | ✅ |
| T1 priority + auto-categorise | ✅ M03 | partial | ✅ | partial | ✅ | ✅ | ✅ |
| T2 auto-assign in employment-context | ✅ M03 (REFUSED at Cedar layer pending ADR-TASKS-0006 conformity ADR; available outside employment-context) | partial | partial | partial | ❌ | partial | ✅ |
| **EU AI Act Annex III §4 conformity (employment-context auto-assign)** | ✅ (refused at Cedar layer until ADR-TASKS-0006 conformity ADR ships) | unclear | unclear | unclear | unclear | unclear | unclear |
| **EEOC bias audit on T2 auto-assign in employment** | ✅ M03 (slos/auto-assign-fairness-correctness) | unclear | unclear | unclear | unclear | unclear | unclear |
| **EU AI Act Art. 14 reversibility window 30s** | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| **EU AI Act Art. 50 user labelling** | ✅ | partial | partial | partial | partial | partial | partial |

## Key differentiators (ordered)

1. **Bidirectional workflow-engine bridge with durable-execution semantics** — no competitor couples a workflow engine to the task primitive at code granularity. **Differentiator** (per ADR-TASKS-0005).
2. **Dependency-graph cycle prevention at write time (correctness SLO = 100%)** — every competitor detects lazily; oyatie refuses at write-time per ADR-TASKS-0002. **Differentiator.**
3. **Dual-context (Personal / Professional) structural isolation enforced in code** — competitors are policy-only or account-switching. **Differentiator.**
4. **EU AI Act Annex III §4 conformity-bounded T2 auto-assign with Cedar refusal until conformity** — competitors leave this to tenant policy or don't implement. **Differentiator.**
5. **Per-tenant Meilisearch index with rebuildable degraded mode** — Linear / Jira's rebuild is opaque; oyatie's degrades + rebuilds publicly.
6. **6 third-party importers (CSV + Jira + Asana + Trello + Linear + Todoist) with strict assignee resolution** — covers every realistic incumbent.
7. **Strict custom-field type coercion** — refuses silent type-mismatch; differentiator over Asana/Linear/Jira silent coerce.
8. **Audit-chain (Ed25519 + Merkle) on every task lifecycle** — beyond competitor offerings.
9. **Recurring tasks aligned with RFC 5545 RRULE subset (shared with calendar)** — bounded materialisation (5y horizon); not unbounded.

## Gap closing plan (M03 → M05)

| Gap | Current state | Plan | Target |
|---|---|---|---|
| In-µservice automation rules | cross-µservice to workflow-engine only | offer simple in-µservice rules + workflow-engine for complex; bridge | M03-onward1 |
| JMAP-Tasks (draft) | M05 once IETF stabilises | adapter-jmap crate; SDK Swift integration | M05 |
| Apple Reminders compat (CalDAV-Tasks VTODO) | M05 | defer to calendar's CalDAV stack; thin VTODO mapper | M05 |
| Time-tracking | M02-onward | extend task-store-worker; Cedar admission for opt-in surveillance posture | M02-onward |
| Microsoft Planner / To Do compat shim | M04-onward1 | OpenAPI shim adapter | M04-onward1 |
| Wrike / Smartsheet importer | M05 | extend importers BC; CSV-based mapping | M05 |
| Plane (OSS) importer | M05 | extend importers BC | M05 |
| AI auto-assign conformity-assessment ADR (ADR-TASKS-0006) | DEFERRED | dedicated conformity-ADR + notified-body audit | M04-onward |
| Native dependency-graph visualiser | M04 | view-engine extension | M04 |
| Workload balancing across assignees | M04 | view-engine + foundry-runtime T1 capability | M04 |

## Verification

- HG-TASKS gate validates this matrix is consistent with the PRD `§Competitive Benchmark` row.
- gtm-customer-success references this matrix in sales materials; any claim of parity / superiority that diverges from this matrix is a process violation.
- Bi-annual review re-validates each row against current competitor release notes; new competitor entrants get added.

## References

- ADR-0123 — Hyperscaler maturity claim gate.
- ADR-0135; ADR-0131; ADR-0132; ADR-0133.
- ADR-TASKS-0001 — Data model + custom fields strict coercion + importers.
- ADR-TASKS-0002 — Dependency graph + cycle prevention.
- ADR-TASKS-0005 — Workflow-engine bridge.
- ADR-TASKS-0006 — AI auto-assign + EU AI Act Annex III §4 bounds.
- `microservices/tasks/PRD.md` §Competitive Benchmark.
- Asana API — `developers.asana.com`.
- Linear GraphQL — `developers.linear.app`.
- Atlassian Jira REST — `developer.atlassian.com/cloud/jira`.
- Trello REST — `developer.atlassian.com/cloud/trello`.
- Monday API — `developer.monday.com`.
- ClickUp API — `clickup.com/api`.
- Notion API — `developers.notion.com`.
- Todoist API — `developer.todoist.com`.
- Microsoft Graph (Planner) — `learn.microsoft.com/graph/api/resources/planner-overview`.
- Apple EventKit — `developer.apple.com/documentation/eventkit`.
- Plane API — `plane.so/docs/api`.
- TaskJuggler — `taskjuggler.org`.
- `microservices/calendar/competitor-parity-matrix.md` — sibling reference template.
