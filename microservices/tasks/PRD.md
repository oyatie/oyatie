---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-tasks
microservice: tasks
status: Accepted
sales_segment: shared-substrate + suite-app
tenant_class_eligibility: [demo_trial, paid]
paid_billing_components_emitted: [per_seat, per_usage]
milestone_first_ship: M03-connect-dissolution
bominal_source: [ADR-0231-connect-tasks-board-and-views, ADR-0232-connect-tasks-dependency-graph, ADR-0233-connect-tasks-recurring-and-rsvp-equivalent]
related_adrs: [ADR-0056, ADR-0105, ADR-0106, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-0134, ADR-0140 (retired per ADR-0145), ADR-0329, ADR-0330, ADR-0331, ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345, ADR-TASKS-0001, ADR-TASKS-0002, ADR-TASKS-0003, ADR-TASKS-0004, ADR-TASKS-0005, ADR-TASKS-0006]
related_specs: [/specs/microservices/tasks.json, /specs/per-microservice-flat-layout.json, /specs/agentic-slo-gated-promotion.json]
date: 2026-05-17
owner_team: axis-tasks
doc_status: published
---

# PRD-tasks: Task Management µservice

## Purpose

The `tasks` µservice is oyatie's native **work-item / task management** substrate — the user-facing CRUD surface that humans use to organise actionable work. Per ADR-0132 (product-suite + bundle dissolution) and parallel-session ADR-0135 (Connect unbundle), `tasks` is a standalone tenant-facing µservice — no longer part of any Connect suite — owning: task CRUD with title/description/status/priority/assignee/due-date/labels/parent-subtask; list/project (collection-of-tasks with custom-field schema); board (kanban view with columns); view (list + board + gantt + calendar + timeline + table — multi-view per project); assignee + watcher; due-date + reminder; recurring-task (RRULE-aligned subset per RFC 5545); dependency graph (blocks / blocked-by / relates-to); checklist (subtasks-lite); attachment (cross-µservice to drive); comment + reaction; time-tracking (start/stop timer + manual entry; M02-onward); custom-field (text/number/date/dropdown/multi-select/person/url/checkbox); saved-filter + view-template; label/tag; sprint/iteration (Linear-class agile); milestone; status-workflow (per-project; configurable Todo→InProgress→Review→Done); priority (P0..P3 + Linear-style); automation (cross-µservice to workflow-engine — task-state-change triggers); notifications (cross-µservice to mail + messenger); bulk-edit; multi-assignee (with primary-assignee semantics); templates + template-marketplace; cross-task + cross-project search (Meilisearch); import (CSV + Jira + Asana + Trello + Linear + Todoist); export (CSV + JSON); API + webhooks; integrations (calendar bridge for due-date → event); epic-roadmap (Linear/Jira); portfolio-view (cross-project rollup); AI-task-suggest (T0 next-task suggest, T1 auto-categorise + priority-suggest, T2 auto-assign).

The µservice differentiates from `workflow-engine` (durable execution engine) by being the **user-facing CRUD primitive for work-items that humans manage manually**. Calendar binds time-blocks; tasks binds work-items.

The µservice carries dual-context (Personal / Professional) per parallel ADR-0238; task details never cross context boundaries except via explicit project membership or policy-bound projection.

Bominal inheritance: ADR-0231 board+views + ADR-0232 dependency graph + ADR-0233 recurring/RSVP-equivalent semantics inherited 1:1 per `feedback_bominal_inheritance_precedence.md`; oyatie additions captured below.

## Tenant Value

- **Tenant Outcome 1 — Native task management without third-party dependency.** Tenants do not need Asana / Trello / Linear / Jira / Monday.com / ClickUp / Notion / Todoist accounts; the µservice is a native first-party task substrate.
- **Tenant Outcome 2 — Multi-view per project.** List, board (kanban), gantt, calendar, timeline, table — all over the same project store; switching is a render-time concern.
- **Tenant Outcome 3 — Dependency graph with cycle prevention.** Cross-task blocks / blocked-by / relates-to with DAG enforcement at write-time per ADR-TASKS-0002.
- **Tenant Outcome 4 — Cross-project portfolio + epic rollup.** Linear/Jira-class agile + Asana-class portfolio at substrate granularity.
- **Tenant Outcome 5 — Bidirectional task ↔ workflow integration.** Task state changes trigger workflow-engine; workflow-engine can create + mutate tasks (per ADR-TASKS-0005). Removes the "tasks vs workflows" duality.
- **Tenant Outcome 6 — Migration from Asana / Trello / Linear / Jira / Todoist.** CSV + native importers; preserves issue keys + custom fields + history per `migration-from-connect.md` + the importers under `IP-009-importers.md`.
- **Internal Outcome 7 — Dual-context separation.** Personal vs Professional task entities isolated at the data-class + Cedar-policy boundary; cross-context inference is structurally impossible.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | tenant operator | to create a task with title, description, status, priority, assignee(s), due-date, labels, parent | I can capture work items | task-store | Must |
| FR-02 | tenant operator | to organise tasks into projects with custom-field schemas | I can model my workflow | project-list | Must |
| FR-03 | tenant operator | to render a project as list / board / gantt / calendar / timeline / table | I can switch perspectives | view-engine | Must |
| FR-04 | tenant operator | to declare blocks / blocked-by / relates-to between tasks with cycle prevention | I can model dependencies safely | dependency-graph | Must |
| FR-05 | tenant operator | to make a task recurring via RFC 5545 RRULE subset | I get repeated task materialisation | recurrence | Must |
| FR-06 | tenant operator | to set a status workflow per project (configurable Todo→InProgress→Review→Done) | I can model my own process | status-workflow | Must |
| FR-07 | tenant operator | to bulk-edit ≥100 tasks at once | I can manage at scale | bulk-edit | Must |
| FR-08 | tenant operator | to search across tasks + projects via Meilisearch | I can find any task | search-index | Must |
| FR-09 | tenant operator | to import from CSV + Jira + Asana + Trello + Linear + Todoist | I can migrate from incumbent | importers | Must |
| FR-10 | tenant operator | to export to CSV + JSON | I am not locked in (GDPR Art. 20) | exporters | Must |
| FR-11 | tenant operator | to receive a webhook on task-state change | downstream Workflow can react | (cross-cutting) | Must |
| FR-12 | tenant operator | to bind a task due-date to a calendar event | the time block is reserved | (cross-µservice bridge) | Should |
| FR-13 | tenant operator | to declare a sprint / iteration with start + end + capacity | Linear-class agile works | sprint | Must |
| FR-14 | tenant operator | to declare a milestone with target date | release planning works | milestone | Should |
| FR-15 | tenant operator | to roll up tasks across projects into a portfolio view | senior management visibility | portfolio | Should |
| FR-16 | tenant operator | to add custom fields (text/number/date/dropdown/multi-select/person/url/checkbox) per project | I can model arbitrary attributes | custom-field | Must |
| FR-17 | tenant operator | to time-track via start/stop timer + manual entry (M02-onward) | I can bill / forecast | time-tracking | Should |
| FR-18 | tenant compliance officer | to put a task under legal hold | task + history + dependencies preserved past retention | task-store | Must |
| FR-19 | tenant operator | to define a project template + share to template marketplace | I can codify repeatable work | template-marketplace | Should |
| FR-20 | tenant operator | to consume T0 next-task / T1 priority + auto-categorise / T2 auto-assign | AI assists task work | foundry-bridge | Should |

## Non-Functional Requirements

### Performance

| Metric | p50 | p95 | p99 | Notes |
|---|---|---|---|---|
| Task-list render (200 tasks) | ≤80ms | ≤200ms | ≤500ms | server-side render; client paint separate |
| Task create | ≤30ms | ≤80ms | ≤150ms | sync write; async fanout |
| Task update | ≤20ms | ≤50ms | ≤120ms | single-field updates dominate |
| Cross-project search | ≤150ms | ≤300ms | ≤700ms | Meilisearch; project-scoped index |
| Board render with drag-and-drop | ≤30ms | ≤50ms (perceived) | ≤100ms | client DnD; server commit async |
| Bulk-update 100 tasks | ≤180ms | ≤300ms | ≤700ms | atomic per-tenant transaction |
| Recurring-task materialisation | ≤400ms | ≤900ms | ≤1s | per RRULE expansion + insert |
| Webhook fire | ≤100ms | ≤200ms | ≤400ms | async dispatch + ack |
| Dependency-cycle-check on add | ≤10ms | ≤25ms | ≤50ms | bounded BFS on graph |
| CSV import (10k rows) | — | ≤45s | ≤90s | streaming parse |
| Time-tracking timer tick persistence | ≤20ms | ≤40ms | ≤100ms | append-only |

### Security

- All task payloads encrypted-at-rest under tenant-DEK (per Bominal ADR-0111 envelope encryption) in Professional context; Personal-context tenants may declare E2E.
- Dependency-graph cycle prevention enforced at the domain layer + database constraint; cycle-creating writes return `DependencyCycle::Refused` 422.
- All exports filter custom-field values per requestor role; sensitive fields refused at the Cedar layer.
- Bulk-edit is rate-limited per tenant; ≥10k-task operations require explicit second-confirmation.
- All third-party importers run in a hardened parser sandbox (subprocess + cgroup memory cap + 5min timeout) — defends against importer-payload-as-malware.

### Audit + Compliance

- Every `TaskCreated / TaskUpdated / TaskStateChanged / TaskAssigned / TaskCommented / TaskDependencyDeclared / TaskBulkEdited / TaskExported / TaskImported / TaskTimerTick` emits an audit-chain record (Merkle + Ed25519 per Bominal ADR-0028).
- Legal-hold preserves task + comments + history + dependency edges + time-tracking entries past retention expiry.
- Per-jurisdiction retention computed per ADR-0140 Cedar pack overlay.

### Availability + SLO

- Availability target: 99.95% monthly for task-read path; 99.9% for write path.
- RTO ≤ 15 min; RPO ≤ 60s (Postgres logical replication).
- Search index (Meilisearch) is **rebuildable**: a full search-index loss degrades to direct-Postgres-search fallback within 1 min, not a hard outage. Rebuild completes in ≤30 min for 10M tasks.

### Data residency

- Tenant data pinned to the tenant's region per ADR-0117 + ADR-0140; cross-region replication forbidden by default; SCC-gated when activated.

### DR posture (ADR-0343)

- Target: RTO <= 900 s and RPO <= 60 s for paid production task state, matching `manifest.json#dr.rto_p99_seconds` and `manifest.json#dr.rpo_p99_seconds`.
- Compliance floors considered: HIPAA-2024 requires RTO <= 3600 s and RPO <= 300 s with multi-region DR; SOC2-T2 requires RTO <= 14400 s and RPO <= 900 s; ISO27001-2022 requires RTO <= 14400 s and RPO <= 3600 s. The effective task target remains 900 s / 60 s because the service target is stricter than the applicable floors.
- Failover runbook reference: `runbooks/custom-field-schema-migration.md`, with quarterly evidence at `evidence/dr-drills/<pack>-<unix_ts>.json`. The manifest substrate is `postgres_wal_g`, `valkey`, and `object_storage_versioned` with active-passive cross-region continuous replication.
- Multi-region active-active posture: `false` in `manifest.json`; task writes, dependency-cycle enforcement, recurrence materialization, and legal-hold transitions remain single-writer until signed failover promotion.
- WHY: task boards, due dates, comments, dependency edges, and legal holds remain tenant-visible during a regional outage without letting pack boundaries leak work-item data across jurisdictions.

### Capacity model (ADR-0340)

- Manifest source: `manifest.json#capacity_model` declares the PRD capacity baseline.
- Per-tenant baseline: reserve 0.22 vCPU, 512 MiB RAM, 10 GB task/project/search storage, 4 Postgres connections, 2 Valkey connections, and 8 outbound HTTP slots per tenant.
- Scaling dimension: `per_user` for active task/project growth, with per-request triggers for writes, search, webhook fanout, recurrence, and importers.
- Cell placement class: Tier-3 product cell. Rationale: user-facing work management is product-critical, but task data stays behind tenant-scoped product cells rather than Tier-1 substrate control planes.
- Autoscaling boundaries: task-store-rest 5..100 replicas, project-list-rest 3..50, view-engine-rest 5..100, dependency-graph-rest 3..50, search-index-rest 5..100, importer-rest 3..30, and background workers 3..30 per queue family.
- WHY: the model serves steady daily project traffic plus bursty imports, webhooks, and board sessions while preserving dependency-cycle correctness and legal-hold writes under saturation.

### Sustainability + cost attribution (ADR-0344)

- Per-call emission claim: every state-changing audit row adds `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside the existing task event class, tenant, trace, and policy dimensions.
- Provider routing affected by carbon: yes for recurrence expansion, import/export, webhook fanout, search rebuild, and analytics backfill when SLO and residency allow; no for interactive task create/update, dependency-cycle checks, legal-hold, or policy-denied paths.
- Per-tenant cost transparency surface: task usage appears in the tenant admin billing/FinOps view by `per_seat` plus `per_usage` components, with drill-down by task-store, view-engine, search-index, importers, and webhook fanout.
- WHY: tenants can defend CSRD, SB-253, and SEC climate-disclosure reporting while seeing why high-volume task imports, webhooks, and search rebuilds moved cost or carbon.

### API versioning posture (ADR-0342)

- Public API version model: date carrier triplet using `Oyatie-Version: YYYY-MM-DD`, URL prefix `/v/<YYYY-MM-DD>/tasks/...`, and proto3 field `oyatie_version`.
- SDK semver model: generated SDKs use `major.minor.patch`; date-versioned wire contracts do not imply SDK major bumps unless source compatibility breaks.
- Support window: last N=3 public API dates are supported for at least 180 days.
- Per-tenant pinning supported: yes, for REST, webhook, AsyncAPI, and SDK clients during migrations from Asana, Trello, Linear, Jira, and Todoist.
- Internal-mesh exemption: yes. Direct gRPC inside the service mesh remains ADR-0145-compatible and does not require URL date prefixes.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase` rename for new crates). **Seven primary BCs.**

| BC | Crate family | Purpose | Key entities |
|---|---|---|---|
| `task-store` | `oya-tasks-task-store-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,worker,sdk,app}` | Task CRUD; tenant-DEK encryption; legal-hold; status workflow | `Task`, `TaskComment`, `TaskHistoryEntry`, `LegalHoldRef`, `RetentionPolicyRef` |
| `project-list` | `oya-tasks-project-list-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,app}` | Project / list collection; custom-field schema; project membership | `Project`, `CustomFieldSchema`, `ProjectMember` |
| `view-engine` | `oya-tasks-view-engine-{kernel,domain,usecase,api,adapter,adapter-valkey,rest,app}` | List/board/gantt/calendar/timeline/table; saved filters; presence | `View`, `SavedFilter`, `ViewState`, `BoardColumn` |
| `dependency-graph` | `oya-tasks-dependency-graph-{kernel,domain,usecase,api,adapter,rest,app}` | blocks/blocked-by/relates-to; DAG cycle prevention; critical-path | `DependencyEdge`, `CycleDecision`, `CriticalPath` |
| `recurrence` | `oya-tasks-recurrence-{kernel,domain,usecase,api,adapter,worker,app}` | RFC 5545 RRULE subset for recurring tasks; bounded materialisation | `TaskRecurrenceRule`, `MaterialisedTask`, `RecurrenceWindow` |
| `search-index` | `oya-tasks-search-index-{kernel,domain,usecase,api,adapter,adapter-meilisearch,worker,app}` | Cross-project search; per-tenant index; rebuildable | `SearchDocument`, `IndexJob`, `QueryPlan` |
| `importers` | `oya-tasks-importers-{kernel,domain,usecase,api,adapter,adapter-csv,adapter-jira,adapter-asana,adapter-trello,adapter-linear,adapter-todoist,worker,app}` | CSV + Jira + Asana + Trello + Linear + Todoist parse + map | `ImportJob`, `SourceFormat`, `FieldMapping`, `ImportReport` |

Naming justification (one of seven; same shape applies to others) — `task-store`:

```
NAME: oya-tasks-task-store-<layer>
JUSTIFICATION:
- microservice = tasks: this µservice; ADR-0056 v4.1 flat BNF + ADR-0131 per-microservice
  folder. No shared|vertical bisection.
- bc-tokens = task-store: primary BC for task persistence; siblings (project-list,
  view-engine, dependency-graph, recurrence, search-index, importers) justify explicit
  BC token per ADR-0056 v4.1 BC-optionality rule.
- layer = <layer>: one crate per layer per ADR-0105 13-value canonical enum.
  - kernel: port-trait + entity types (Task, TaskComment, TaskHistoryEntry,
    RetentionPolicyRef, LegalHoldRef, TaskContext{Personal|Professional}). Zero I/O.
    data_class annotations.
  - domain: pure task-invariant math (status-transition validity, priority ordering,
    custom-field type coercion, hold coverage).
  - usecase (per ADR-0106): orchestrators (create-task, update-task, change-state,
    apply-legal-hold, expire-retention, bulk-edit) reading via ports.
  - api: protocol-neutral typed contracts.
  - adapter: protocol-neutral implementations of kernel ports.
  - adapter-postgres: backend-qualified adapter (per ADR-0105 Amendment 3
    *-adapter-<backend> pattern); implements TaskRepository against Postgres with RLS.
  - rest: HTTP handler/route layer.
  - worker: long-lived background workers (retention sweep, hold cascade, timer-tick
    persister).
  - sdk: client library for tenants + workflow consumers.
  - app: composition root binary.
- exemptions claimed: none.
```

(Equivalent justifications recorded for the other six BCs at `microservices/tasks/specs/naming-justification.md`.)

Layer mapping table per BC (13-layer enum from ADR-0105; `usecase` per ADR-0106):

| BC | kernel | domain | usecase | api | adapter | adapter-postgres | adapter-valkey | adapter-meilisearch | adapter-csv | adapter-jira | adapter-asana | adapter-trello | adapter-linear | adapter-todoist | rest | worker | sdk | app |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `task-store` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | ✓ | ✓ | ✓ | ✓ |
| `project-list` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | ✓ | — | — | ✓ |
| `view-engine` | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ | — | — | — | — | — | — | — | ✓ | — | — | ✓ |
| `dependency-graph` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | ✓ | — | — | ✓ |
| `recurrence` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | ✓ | — | ✓ |
| `search-index` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | ✓ | — | — | — | — | — | — | — | ✓ | — | ✓ |
| `importers` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ |

Total crates introduced by this µservice: **57**.

Port traits declared in each kernel (zero business logic; zero I/O; `data_class` annotated per Bominal ADR-0028):

| Port trait | Kernel crate | Implemented in | Data classes touched |
|---|---|---|---|
| `TaskRepository` | `oya-tasks-task-store-kernel` | `-adapter-postgres` | `PERSONAL_TASK_CONTENT` + `PROFESSIONAL_TASK_CONTENT` (per-context envelope encryption) |
| `TaskHistoryStore` | `oya-tasks-task-store-kernel` | `-adapter-postgres` | `AUDIT` |
| `RetentionPolicyResolver` | `oya-tasks-task-store-kernel` | `-adapter` (resolves to `tenancy` µservice via Workflow) | `AUDIT` |
| `LegalHoldStore` | `oya-tasks-task-store-kernel` | `-adapter-postgres` | `AUDIT` |
| `ProjectRepository` | `oya-tasks-project-list-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT` |
| `CustomFieldSchemaRepository` | `oya-tasks-project-list-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT` |
| `ViewStateStore` | `oya-tasks-view-engine-kernel` | `-adapter-valkey` | `BEHAVIORAL_TENANT_PRODUCT` (presence + view cursor) |
| `BoardProjector` | `oya-tasks-view-engine-kernel` | `-adapter` | `INTERNAL_ONLY` |
| `DependencyEdgeStore` | `oya-tasks-dependency-graph-kernel` | `-adapter-postgres` (subsumed under task-store -adapter-postgres) | `BEHAVIORAL_TENANT_PRODUCT` |
| `CycleDetector` | `oya-tasks-dependency-graph-kernel` | `-adapter` | `INTERNAL_ONLY` |
| `RecurrenceExpander` | `oya-tasks-recurrence-kernel` | `-adapter` (rrule-rs based; aligned with calendar ADR-CAL-0002) | `INTERNAL_ONLY` |
| `SearchIndexClient` | `oya-tasks-search-index-kernel` | `-adapter-meilisearch` | `PERSONAL_TASK_CONTENT` + `PROFESSIONAL_TASK_CONTENT` (indexed with per-tenant key + redaction) |
| `ImportSourceReader` (× 6) | `oya-tasks-importers-kernel` | `-adapter-csv`, `-adapter-jira`, `-adapter-asana`, `-adapter-trello`, `-adapter-linear`, `-adapter-todoist` | `PII_IDENTIFYING` (assignee/reporter emails) + content classes |

Data-class enforcement: every kernel struct field carries a `#[data_class(...)]` annotation; the `oya-check-data-class` LEAN lane refuses unannotated fields.

Cross-product rule: `tasks` MUST NOT import another product µservice crate at any layer. Cross-product flows go through Workflow (events) or Ontology (entity reads/writes). Consumed µservices: `tenancy` (tenant + identity resolution), `audit-chain` (seal emission), `mail` (notification + create-task-from-email), `messenger` (notification + create-task-from-message), `drive` (attachments), `calendar` (due-date → event bridge), `workflow-engine` (task-state triggers + workflow-creates-task), `foundry-runtime` (T0/T1/T2 capabilities), `ontology` (Task/Project/Person/Label entity bindings), `observability` (telemetry). LEAN-A2 CI lane enforces.

CI lanes that must green:

- `oya gate validate lean-a1 --microservice tasks`
- `oya gate validate lean-a2 --microservice tasks`
- `oya gate validate port-location --microservice tasks`
- `oya gate validate layer-correctness --microservice tasks`
- `oya gate validate per-microservice-layout --microservice tasks`
- `oya gate validate statelessness --microservice tasks`
- `oya gate validate shardability --microservice tasks`
- `oya gate validate hyperscaler-maturity --microservice tasks`
- `oya gate validate rrule-conformance --microservice tasks` (NEW; aligned with calendar)
- `oya gate validate dependency-graph-cycle-prevention --microservice tasks` (NEW)
- `oya gate validate task-state-machine-correctness --microservice tasks` (NEW)
- `oya gate validate auto-assign-fairness --microservice tasks` (NEW; EU AI Act Annex III §4 employment-context)

## Integration via Workflow + Ontology

### Workflow events produced

| Event | Topic | Trigger | Consumed by | Idempotency key |
|---|---|---|---|---|
| `TaskCreated` | `tasks.task.lifecycle.v1` | new task written | workflow-engine (triggers), audit-chain, search-index, messenger (notification), mail (notification) | `task_id` |
| `TaskUpdated` | `tasks.task.lifecycle.v1` | task mutation | search-index, audit-chain, workflow-engine | `task_id + version` |
| `TaskStateChanged` | `tasks.task.state.v1` | status transition | workflow-engine (automation), messenger, mail | `task_id + new_state + at` |
| `TaskAssigned` | `tasks.task.assignment.v1` | assignee added/removed | messenger, mail, audit-chain | `task_id + assignee_id + at` |
| `TaskCommented` | `tasks.task.comment.v1` | comment posted | messenger, mail, audit-chain | `comment_id` |
| `TaskDependencyDeclared` | `tasks.task.dependency.v1` | edge added | search-index, audit-chain | `(from_id, to_id, kind)` |
| `TaskBulkEdited` | `tasks.task.bulk.v1` | bulk-edit completed | audit-chain, observability | `bulk_op_id` |
| `TaskRecurrenceMaterialised` | `tasks.task.recurrence.v1` | recurring task instance written | observability | `recurrence_rule_id + window_hash` |
| `LegalHoldApplied` / `LegalHoldReleased` | `audit.tasks.legal_hold.v1` | hold transition | audit-chain, governance | `task_id + hold_id` |
| `TaskExported` / `TaskImported` | `tasks.task.io.v1` | export/import lifecycle | audit-chain, observability | `job_id` |
| `TaskTimerTickPersisted` | `tasks.task.time-tracking.v1` | timer tick written (1Hz worker) | observability | `task_id + tick_at` |

### Workflow events consumed

| Event | Producer | Handler | Action |
|---|---|---|---|
| `TenantOnboarded` | `tenancy` | task-store usecase | provision tenant-DEK; create default project + status workflow + saved views |
| `TenantOffboarded` | `tenancy` | task-store usecase | mark tasks for retention sweep / legal-hold scan |
| `WorkflowTrigger` | `workflow-engine` | task-store usecase | task-bound automation (e.g., "create task X when workflow Y completes step N") |
| `CalendarEventCreated` | `calendar` | task-store usecase | optional bind: due-date created → calendar event reciprocates |
| `MailReceived` (`create-task-from-email` rule active) | `mail` | task-store usecase | new task from inbound email; assignee defaults to inbox owner |
| `MessengerMessagePinned` (`create-task-from-message` rule active) | `messenger` | task-store usecase | new task from pinned chat message |
| `FoundryClassificationCompleted` | `foundry-runtime` | task-store usecase | T1 auto-categorise / priority suggestion materialised |

### Ontology writes

| Object Type | Link Type | Written by BC | Audit |
|---|---|---|---|
| `Task{task_id, tenant, context, status, priority, due_at, created_at, ...}` | `tasks→Tenant`, `tasks→User(assignee)`, `tasks→Project` | `task-store` | Ed25519 |
| `Project{project_id, tenant, custom_field_schema_id, status_workflow_id}` | `project_of→Tenant` | `project-list` | Ed25519 |
| `DependencyEdge{from_task, to_task, kind}` | `depends_on→Task` | `dependency-graph` | Ed25519 |
| `Sprint{sprint_id, project_id, starts_at, ends_at, capacity}` | `sprint_of→Project` | `project-list` | Ed25519 |
| `Milestone{milestone_id, project_id, target_at}` | `milestone_of→Project` | `project-list` | Ed25519 |
| `LegalHold{hold_id, task_id, opened_by, opened_at}` | `holds→Task` | `task-store` | Ed25519 |

### Ontology reads

| Object | Read by | Query shape |
|---|---|---|
| `User` (tenant directory) | `task-store`, `project-list` | by `(tenant_id, user_id)` |
| `Tenant` | every BC | by `tenant_id` |
| `RetentionPolicy` | `task-store` | by `(tenant_id, pack)` |
| `Label` | `task-store`, `search-index` | by `(tenant_id, label_id)` |
| `Workflow` (cross-µservice) | `task-store-usecase` | by `(tenant_id, workflow_id)` for workflow-creates-task path |

## Competitive Benchmark

| Competitor | Product | Parity dimensions | Primary source |
|---|---|---|---|
| Asana | enterprise task management | projects + custom fields + portfolio + goals | `developers.asana.com` |
| Trello | board-first | kanban + power-ups + automation (Butler) | `developer.atlassian.com/cloud/trello` |
| Linear | engineering-focused | issue tracker + cycles + roadmap + Linear Method | `developers.linear.app` |
| Atlassian Jira | enterprise issue tracker | issue + epic + sprint + JQL + workflow | `developer.atlassian.com/cloud/jira` |
| Monday.com | flexible boards | item + board + automation | `developer.monday.com` |
| ClickUp | "everything app" | task + doc + chat + goal | `clickup.com/api` |
| Notion Database | wiki + database | unified database; rich custom fields | `developers.notion.com` |
| Airtable Tasks | spreadsheet-database | tabular + form + automation | `airtable.com/developers/web/api` |
| Todoist | personal + small-team | natural-language input + recurring | `developer.todoist.com` |
| Microsoft Planner + To Do | M365 task pair | Planner (board) + To Do (personal) | `learn.microsoft.com/graph/api/resources/planner-overview` |
| Apple Reminders | iCloud reminders | CalDAV-Tasks (RFC 4791 + iCalendar VTODO); native iOS/macOS | `developer.apple.com/documentation/eventkit` |
| Google Tasks | G Suite tasks | basic personal task; limited project | `developers.google.com/tasks` |
| Basecamp | small-team simple | to-do list + message + schedule | `github.com/basecamp/bc3-api` |
| Wrike | enterprise project | task + project + Gantt + custom workflow | `developers.wrike.com` |
| Smartsheet | spreadsheet-project | sheet + Gantt + form | `smartsheet-platform.github.io/api-docs/` |
| Height | AI-first task | tasks + AI auto-categorise | (proprietary; no public API) |
| Shortcut (Clubhouse) | engineering | story + epic + iteration | `developer.shortcut.com` |
| Plane | OSS Jira-like | issue + cycle + module | `plane.so/docs/api` |
| TaskJuggler | gantt scheduling OSS | resource + duration + critical path | `taskjuggler.org` |

Key parity gaps to close (ordered):

1. **Workflow-engine-native bidirectional task ↔ workflow** — none of the competitors couple a durable execution engine to the task primitive at code granularity. **Differentiator** (per ADR-TASKS-0005).
2. **Dual-context (Personal / Professional) isolation enforced structurally** — no competitor enforces context-separation in code; tenant-policy only. **Differentiator.**
3. **Dependency-graph cycle prevention as a load-bearing invariant (correctness SLO 100%)** — competitors detect lazily; oyatie refuses at write-time per ADR-TASKS-0002. **Differentiator.**
4. **Per-tenant Meilisearch index with rebuildable degraded mode** — Linear / Jira's search rebuild is opaque; oyatie's degrades cleanly + rebuilds publicly.
5. **CSV + Jira + Asana + Trello + Linear + Todoist importers at substrate granularity** — covers every realistic incumbent.
6. **Audit-chain (Ed25519 + Merkle) on every task lifecycle** — beyond competitor offerings.
7. **EU AI Act Annex III §4 high-risk auto-assign in employment-context refused at Cedar layer** — competitors leave this to tenant policy.

## Performance Targets (canonical bench surface)

| Metric | Target | Verification |
|---|---|---|
| Task-list render (200 tasks) p95 | ≤ 200ms | `cargo bench -p oya-tasks-task-store-adapter-postgres -- list_render` |
| Task create p95 | ≤ 80ms | `cargo bench -p oya-tasks-task-store-usecase -- create` |
| Task update p99 | ≤ 50ms | `cargo bench -p oya-tasks-task-store-usecase -- update` |
| Cross-project search p95 | ≤ 300ms | `cargo bench -p oya-tasks-search-index-adapter-meilisearch -- query` |
| Bulk-update 100 tasks p95 | ≤ 300ms | `cargo bench -p oya-tasks-task-store-usecase -- bulk` |
| Recurring-task-materialise p99 | ≤ 1s | `cargo bench -p oya-tasks-recurrence-domain -- rrule_expand` |
| Webhook-fire p95 | ≤ 200ms | `cargo bench -p oya-tasks-task-store-worker -- webhook_fanout` |
| Dependency-cycle-detection p99 | ≤ 50ms | `cargo bench -p oya-tasks-dependency-graph-domain -- cycle_check` |

Error budget: monthly 99.95% availability → ~22 min/month.

## Horizontal Scalability

State strategy (per Bominal ADR-0019): `mixed`. Postgres (task-store + project-list + dependency-edge; per-tenant RLS); Valkey (view-cache + presence); Meilisearch (per-tenant cross-project search); stateless workers for retention-sweep + recurrence-expansion + search-index-rebuild + importers + webhook-fanout.

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Active projects | 500k | 5M | Postgres connection pool > 70% |
| Active tasks | 10M | 100M | task-store rest p99 > 200ms |
| Tasks/s write | 2k | 20k | task-store rest p99 > 200ms |
| Cross-project search/s | 500 | 5k | search-index rest p99 > 300ms |
| Recurrence materialisation/s | 200 | 2k | recurrence worker queue depth > 60s of cadence |
| Webhook fire/s | 5k | 50k | webhook worker queue > 1000 |
| Active board sessions (DnD presence) | 50k | 500k | view-engine rest CPU > 70% |
| Active CSV/JSON imports concurrent | 10 | 100 | importer worker queue > 5min |

Scale-out policy:
- Kubernetes HPA: rest pods scale on CPU > 70%; min 3, max 100.
- Postgres: per-tenant logical shard; cross-cell replication-factor 3 with Patroni.
- Valkey: cluster mode; per-tenant key prefix; eviction policy `allkeys-lru` for view-cache.
- Meilisearch: per-tenant index; cluster-mode (Meilisearch 0.10 LTS); rebuildable.
- Pre-warmed pool: 5 standby pods; cold-start ≤ 700ms.

Cross-region: M03 launches in KR (ap-seoul-1); M04 expands to EU + US per ADR-0117 jurisdiction pack.

Sharding: tasks partitioned by `(tenant_id, project_id_hash)`; comments partitioned by `task_id_hash`; dependency edges partitioned by `(tenant_id, project_id_hash)`.

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | Task-list render of 200 tasks completes within p95 ≤ 200ms | `cargo bench` |
| AC-02 | Dependency-cycle creating write is REFUSED with `DependencyCycle::Refused` 422 (PRD AC-02 = 100% correctness, no error budget) | `cargo nextest -p oya-tasks-dependency-graph-domain -- cycle_refusal` |
| AC-03 | RFC 5545 RRULE subset conformance test corpus 100% pass | `cargo nextest -p oya-tasks-recurrence-domain -- rrule_corpus` |
| AC-04 | Status-workflow refusal on invalid transition emits `InvalidTransition::Refused` 422 | `cargo nextest -p oya-tasks-task-store-domain -- state_machine` |
| AC-05 | Bulk-update of 100 tasks completes within p95 ≤ 300ms with all-or-nothing semantics | `cargo nextest -p oya-tasks-task-store-usecase -- bulk_atomicity` |
| AC-06 | Legal-hold preserves task + history + dependency edges + time-tracking entries past retention expiry | `cargo nextest -p oya-tasks-task-store-domain -- legal_hold` |
| AC-07 | Personal-context tasks NEVER appear in Professional-context queries | `cargo nextest -p oya-tasks-task-store-domain -- context_isolation` |
| AC-08 | Tenant-DEK envelope encryption applied to Professional task content; verified at rest | `tests/e2e/encryption-at-rest.rs` |
| AC-09 | Search-index rebuild from cold Meilisearch completes ≤30 min for 10M tasks | `cargo bench -p oya-tasks-search-index-worker -- full_rebuild` |
| AC-10 | CSV / Jira / Asana / Trello / Linear / Todoist importer rejects malformed input rather than auto-repair | `cargo nextest -p oya-tasks-importers-domain -- malformed_refusal` |
| AC-11 | Audit-chain seal emitted for every task lifecycle + assignment + comment + dependency + bulk-edit + timer tick | `cargo nextest -p oya-tasks-task-store-app -- audit_chain_emission` |
| AC-12 | Auto-assign in EU AI Act Annex III §4 employment-context REFUSED at Cedar layer pending conformity assessment per ADR-TASKS-0006 | `cargo nextest -p oya-tasks-task-store-domain -- eu_ai_act_auto_assign_refusal` |
| AC-13 | `oya gate validate per-microservice-layout --microservice tasks` exit 0 | ADR-0131 lane |

## Open Questions

| # | Question | Owner | Target |
|---|---|---|---|
| 1 | Should we ship a native Gantt timeline editor or rely on view-engine → workflow-studio bridge? Currently both; revisit at M04 | council-product | subsequent-to-M04-completion |
| 2 | Time-tracking M03 vs M02-onward — current scope is M02-onward; user opt-in via tenant_class and paid billing components | axis-tasks | resolved at M02-onward |
| 3 | AI-task-suggest T2 auto-assign in non-EU jurisdictions — relax Cedar refusal? | council-product + council-privacy | ADR successor-IP |
| 4 | JMAP-Tasks (draft) protocol portability — defer to M05 | axis-tasks | M05 |
| 5 | Apple Reminders compatibility via VTODO over CalDAV — defer to M05; align with calendar's CalDAV chart | axis-tasks | M05 |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0105 | 13-layer enum | layer authority |
| ADR-0106 | application→usecase | layer rename |
| ADR-0117 | Cloud-native infrastructure | data residency |
| ADR-0135 | Connect unbundle (parallel session) | dual-context inheritance |
| ADR-0139 | Agentic SLO-gated promotion | gate authority |
| ADR-0131 | Per-microservice flat layout | layout authority |
| ADR-0132 | Product-suite + bundle dissolution | µservice independence |
| ADR-0133 | Industry-best-practice conformance | hyperscaler-grade bar |
| ADR-0134 | Connect dissolution Strangler migration | migration policy |
| ADR-0140 | Cedar policy enforcement | policy substrate |
| ADR-0329 | Tier system retired and replaced by tenant_class | tier-retirement authority |
| ADR-0330 | demo_trial vs paid tenant_class with composable paid billing components | tenant_class authority |
| ADR-0331 | Per-µservice tenant_class adoption | per-service adoption authority |
| ADR-TASKS-0001 | Task data model + custom fields | model authority |
| ADR-TASKS-0002 | Dependency graph + cycle prevention | invariant authority |
| ADR-TASKS-0003 | Recurring task engine (rrule-rs alignment with calendar ADR-CAL-0002) | recurrence authority |
| ADR-TASKS-0004 | View engine + board realtime | CRDT/realtime authority |
| ADR-TASKS-0005 | Automation engine cross-µservice (workflow-engine bridge) | automation authority |
| ADR-TASKS-0006 | AI auto-assign + EU AI Act Annex III §4 bounds | AI authority |
| Bominal ADR-0231 | Connect tasks board + views | inherited 1:1 |
| Bominal ADR-0232 | Connect tasks dependency graph | inherited 1:1 |
| Bominal ADR-0233 | Connect tasks recurring + RSVP-equivalent | inherited 1:1 |

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `tasks` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `tasks` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 2 module pin(s) across 1 context(s).
- Scaling input: `per_user` with cell placement `Tier-3` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
