---
doc_class: SdkPlan
title: SDK + Client-Bindings Plan
microservice: tasks
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-tasks + gtm-customer-success
deciders: axis-tasks, council-architecture
related_adrs: [ADR-0131, ADR-0132, ADR-0133, ADR-TASKS-0001, ADR-TASKS-0005]
related_artifacts:
  - microservices/tasks/contracts/openapi/tasks.yaml
  - microservices/tasks/contracts/proto/tasks.proto
  - microservices/tasks/contracts/asyncapi/tasks-events.yaml
  - microservices/tasks/PRD.md
review_cadence: per-SDK-language-launch
doc_status: published
---

# SDK + Client-Bindings Plan (tasks µservice)

## Purpose

Tenants integrate with tasks via three primary surfaces: REST facade, gRPC service, and per-language SDKs. This document specifies the SDK strategy.

## Surface choice (first decision for tenants)

| Surface | Use when | Authority |
|---|---|---|
| REST facade (tasks.yaml) | Tenant writes a custom task app or backend pipeline | OpenAPI 3.2.0 |
| gRPC (tasks.proto) | Tenant runs a backend service; wants strongly-typed contracts | proto3 |
| AsyncAPI subscription | Tenant subscribes to task lifecycle events | AsyncAPI 3.1.0 |
| Per-language SDK | Tenant wants ergonomic auth + tenant binding + retry | this plan |
| Webhook subscription | Tenant wants to react to task state changes without polling | per `contracts/asyncapi/tasks-events.yaml` |
| CSV / JSON import + export | Tenant migrates from incumbent or backs up | per importers BC |
| Importer adapters (CSV / Jira / Asana / Trello / Linear / Todoist) | Tenant migrates from named incumbent | per ADR-TASKS-0001 |
| JMAP-Tasks (draft; M05+) | Modern JSON-over-HTTP; portable | draft-ietf-jmap-tasks (M05) |
| CalDAV-Tasks via VTODO (M05+) | Apple Reminders / native client integration | RFC 4791 + RFC 5545 VTODO (defers to calendar's CalDAV stack) |

## Launch order

| Language | Priority | Generation strategy | Authority |
|---|---|---|---|
| **Rust** | M03 (oyatie's own language) | First-party authored `oya-tasks-<bc>-sdk` crates per BC | axis-tasks |
| **TypeScript** | M03 (Node + Browser) | OpenAPI-generated baseline; ergonomic wrappers + react-hooks helper; published to npm | axis-tasks + gtm |
| **Python** | M03+1 (data-pipeline + scripting tenants) | OpenAPI-generated; published to PyPI; pairs with pandas DataFrame adapter | axis-tasks + gtm |
| **Go** | M04 (backend services + ops tools) | gRPC-generated baseline + ergonomic wrappers | axis-tasks + gtm |
| **JVM (Kotlin / Java)** | M04 (enterprise tenants) | gRPC-generated baseline; Maven Central | axis-tasks + gtm |
| **Swift** | M05 (iOS / macOS integrators) | thin wrapper over REST + JMAP-Tasks (when stable) + CalDAV-Tasks VTODO (defers to calendar's CalDAV stack) | axis-tasks |
| **C# / .NET** | M05 (Microsoft-ecosystem tenants) | OpenAPI-generated; NuGet | axis-tasks + gtm |

## Generation strategy

### Rust SDKs (first-party)

Per-BC under `microservices/tasks/src/crates/oya-tasks-<bc>-sdk/`:

- `oya-tasks-task-store-sdk`: read tasks; write tasks; legal-hold; tenant-DEK envelope helper; bulk-edit helper
- `oya-tasks-project-list-sdk`: project + custom-field schema CRUD
- `oya-tasks-view-engine-sdk`: list / board / gantt / calendar / timeline / table; saved-filter helper; DnD-commit helper
- `oya-tasks-dependency-graph-sdk`: edge add/remove; critical-path query; cycle pre-check helper
- `oya-tasks-recurrence-sdk`: client-side RRULE expansion via `rrule-rs` (mirrors server engine; consumers can pre-compute)
- `oya-tasks-search-index-sdk`: cross-project search query; saved-query helper
- `oya-tasks-importers-sdk`: CSV / Jira / Asana / Trello / Linear / Todoist importer client; per-importer streaming progress

Common shape (mirrors calendar SDK):
- `Client::new(opts)` with OIDC token provider closure.
- `Client` bound to tenant + context at construction; `X-Tenant-Id` + `X-Task-Context` headers automatic.
- Built-in exponential backoff for 5xx + 429.
- gRPC streaming where applicable (task-lifecycle subscription).
- Re-exports types from corresponding `-kernel` crate.
- `#![deny(unsafe_code)]`.

### Generated SDKs

Pipeline (lives in `microservices/tasks/sdk-generation/`, future IP):

1. Source of truth: `contracts/openapi/tasks.yaml` + `contracts/proto/tasks.proto` + `contracts/asyncapi/tasks-events.yaml`.
2. OpenAPI → language: `openapi-generator-cli` 7.x with language profile.
3. Proto → language: `protoc` + language plugin.
4. AsyncAPI → language: `asyncapi-generator` 2.x for typed event subscription clients.
5. Ergonomic wrapper hand-authored on top: auth helpers, tenant-context binding, retry policy + circuit-breaker matching Rust SDK behavior.
6. Per-language CI lane: build + lint + integration-test against staging tasks cluster.

### Third-party importer adapters

Per-importer, in `oya-tasks-importers-adapter-<source>`:

- **CSV**: streaming line-by-line; column mapping declared by user
- **Jira**: REST API (Atlassian Cloud or Server); JQL query; epic + sprint + issue + comment + attachment + custom-field migration
- **Asana**: REST API; project + task + section + custom-field + comment
- **Trello**: REST API; board + list + card + checklist + custom-field
- **Linear**: GraphQL API; team + cycle + issue + project + custom-field
- **Todoist**: REST v2 API; project + section + task + label + recurring

Per ADR-TASKS-0001, every importer:
- Runs in subprocess sandbox (cgroup + 5min timeout per `threat-model.md` T-T-03 + T-D-04)
- Refuses ambiguous assignee mapping (`ImportAssigneeAmbiguous::Refused` per Hyrum #7)
- Refuses cycle-creating edge import (per ADR-TASKS-0002)
- Emits per-row report (succeeded / skipped / failed with redacted reason)

## Public surface (across SDKs)

All SDKs expose:

| Capability | Method | Returns |
|---|---|---|
| List projects | `listProjects()` | `Project[]` |
| Create project | `createProject(req)` | `Project` |
| List tasks (paginated; cursor) | `listTasks(project, filter, cursor)` | `TaskPage` |
| Read task | `getTask(task_id)` | `Task` |
| Create task | `createTask(req)` | `Task` |
| Update task | `updateTask(req)` | `Task` |
| Bulk update tasks | `bulkUpdateTasks(req)` | `BulkUpdateReceipt` |
| Change state | `changeTaskState(task_id, new_state, reason)` | `Task` |
| Add comment | `addComment(task_id, req)` | `Comment` |
| Add dependency edge | `addDependency(from, to, kind)` | `DependencyEdge` |
| Cross-project search | `searchTasks(query, project_scope, cursor)` | `SearchPage` |
| Render view (list/board/gantt/...) | `renderView(project, view_kind, params)` | `ViewState` |
| Apply legal hold | `applyLegalHold(task_id, hold_id)` | `LegalHold` |
| Subscribe to events | `streamTaskLifecycle()` | streaming events |
| Import (CSV/Jira/Asana/Trello/Linear/Todoist) | `importTasks(source, blob_or_credentials)` | `ImportJob` |
| Export (CSV/JSON) | `exportTasks(project_scope)` | `ExportJob` |

Helper utilities:
- Client-side RRULE expansion helper — Rust + TS + Python.
- Client-side cycle pre-check helper — Rust + TS + Python (lets consumers validate before writing).
- Custom-field type coercion helper — Rust + TS + Python (mirrors server strict coercion per ADR-TASKS-0001; refuses coerce-from-string for number-typed fields).

## Tenant SDK onboarding

| Step | Owner |
|---|---|
| Issue OIDC + per-tenant API key + per-tenant DEK reference via OpenBao | ops-security |
| Provide tenant onboarding doc + SDK quick-start (per language) | gtm-customer-success |
| Provide sample workflow: how to subscribe to `TaskCreated` in tenant pipeline | axis-tasks |
| Provide importer tutorial (Jira / Asana / Trello / Linear / Todoist / CSV) | gtm + axis-tasks |
| Provide workflow-bridge tutorial (workflow-engine → tasks) | gtm + axis-tasks |
| Quarterly SDK update notifications (breaking changes 6mo advance) | axis-tasks |

## Sunset policy

| SDK | Sunset trigger | Window |
|---|---|---|
| Any SDK with < 1% tenant usage for ≥ 12mo | underused | 6mo advance + migration help |
| Generator lib upstream-deprecated | dep-deprecated | 12mo + auto-migrate where possible |
| Breaking API change in tasks µservice | per-release | major version bump in SDK; backwards-adapter for 1 prior major |
| Importer adapter for source that is sunset/EOL | (e.g., Asana retired by upstream) | 12mo + migration help to next adapter |

Per `agent-skills:deprecation-and-migration`: every sunset emits an ADR-shaped notice + deprecation-warning in SDK + tenant comms.

## Versioning

- tasks µservice: semver.
- SDK per language: matches tasks major.minor; SDK patch independent.
- Compat matrix per language; CI lane verifies SDK against current + 1 prior major.

## Open-source decision

Defer per-SDK OSS decision until API stable in production ≥ 6mo. Default: closed-source until tenant-driven request. Stripe + Twilio precedent.

## Verification

- Per-SDK CI lane: build + lint + integration-test exit 0.
- Per-SDK compat test: SDK version N+1 against tasks versions N-1, N, N+1.
- Annual SDK telemetry review per language; underused sunsetted.

## References

- `microservices/tasks/contracts/openapi/tasks.yaml`
- `microservices/tasks/contracts/proto/tasks.proto`
- `microservices/tasks/contracts/asyncapi/tasks-events.yaml`
- ADR-0105 (13-layer enum; `sdk` is canonical)
- ADR-TASKS-0001 (data model + importers)
- ADR-TASKS-0005 (workflow-engine bridge)
- OpenAPI Generator — `openapi-generator.tech`
- gRPC — `grpc.io`
- Jira REST API — `developer.atlassian.com/cloud/jira/platform/rest/v3`
- Asana REST API — `developers.asana.com`
- Trello REST API — `developer.atlassian.com/cloud/trello/rest`
- Linear GraphQL API — `developers.linear.app`
- Todoist v2 REST API — `developer.todoist.com/rest/v2`
- Stripe SDK precedent — `stripe.com/docs/libraries`
- `microservices/calendar/sdk-plan.md` — sibling reference template.
