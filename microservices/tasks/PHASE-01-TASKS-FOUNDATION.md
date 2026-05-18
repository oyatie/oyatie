---
doc_class: PhasePlan
template_id: TPL-PHASE-PLAN
microservice: tasks
phase_id: PHASE-01
phase_title: Tasks Foundation — task-store + project-list + view-engine + dependency-graph + recurrence + search-index + importers
status: Accepted
date: 2026-05-17
owner_team: axis-tasks
related_adrs: [ADR-0056, ADR-0105, ADR-0106, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-TASKS-0001, ADR-TASKS-0002, ADR-TASKS-0003, ADR-TASKS-0004, ADR-TASKS-0005, ADR-TASKS-0006]
doc_status: published
---

# PHASE-01 — Tasks Foundation

## Intent

Stand up the seven bounded contexts (task-store, project-list,
view-engine, dependency-graph, recurrence, search-index, importers)
with full Layer-A + Layer-B substrate, Bominal ADR-0231/0232/0233
inheritance, dependency-graph cycle prevention at write time, dual-
context isolation, audit-chain emission, EU AI Act Cedar refusal for
auto-assign in employment-context, and SLO-gated promotion. Phase exit
= AC-01 through AC-13 in `PRD.md` green.

## Phase scope

In-scope:
- 57 crates per the layer mapping table in `PRD.md`.
- Postgres task-store + project-list + dependency-edge schema + per-
  tenant RLS + tenant-DEK envelope encryption.
- Valkey view-cache + presence + bulk-edit dedup.
- Meilisearch per-tenant cross-project search index; rebuildable.
- IANA tzdata + chrono-tz integration (shared with calendar via
  Workflow handoff; tasks does NOT ship its own tzdb worker — relies
  on `calendar`'s tzdb refresh worker per ADR-TASKS-0003).
- RFC 5545 RRULE subset (recurring task materialisation).
- Six third-party importers: CSV, Jira, Asana, Trello, Linear, Todoist.
- Workflow events produced + consumed per `PRD.md`.
- Ontology writes + reads per `PRD.md`.
- HG-TASKS hyperscaler-maturity claim registered per ADR-0123 +
  ADR-0133.

Out-of-scope (scheduled-for-distinct-tracked-work):
- Time-tracking (M02-onward per PRD Open Question 2).
- AI auto-assign in employment-context outside pack-eu (M03-onward1 per ADR-
  TASKS-0006 conformity-assessment ADR).
- JMAP-Tasks adapter — M05 (per PRD Open Question 4).
- Apple Reminders CalDAV-Tasks (VTODO) adapter — M05 (per PRD Open
  Question 5).
- Apple Reminders sync via calendar µservice — M05.

## Phase outputs

| Output | Path | Owner |
|---|---|---|
| 57 crates | `crates/oya-tasks-*` (scaffolded into workspace) | axis-tasks |
| Postgres schema migrations | `microservices/tasks/iac/helm/postgres/migrations/` | axis-tasks |
| Helm charts | `microservices/tasks/iac/helm/{postgres,redis,meilisearch}` | ops-sre-reliability |
| Kustomize overlays | `microservices/tasks/iac/kustomize/{base,overlays/pack-{kr,eu,us,us-healthcare}}` | ops-sre-reliability |
| OpenAPI / AsyncAPI / Proto contracts | `microservices/tasks/contracts/` | axis-tasks |
| Cedar policies | `microservices/tasks/policy/*.cedar` | ops-security |
| Runbooks | `microservices/tasks/runbooks/*.md` | ops-sre-reliability |
| Dashboards | `microservices/tasks/dashboards/*.json` | axis-observability |
| HG-TASKS claim entry | `registry/hyperscaler-maturity-claims.json` | axis-tasks |

## Phase milestones (ChangeSets, per ADR-0110)

| CS | Title | DAG-position | Slice |
|---|---|---|---|
| CS-01 | task-store kernel + domain + usecase + api (10 crates) | Layer-B base | A |
| CS-02 | task-store -adapter-postgres + RLS schema + tenant-DEK envelope | depends CS-01 | A |
| CS-03 | task-store rest + worker + sdk + app | depends CS-02 | A |
| CS-04 | project-list kernel..app (8 crates) | depends CS-01 | A |
| CS-05 | dependency-graph kernel..app (7 crates) — PRD AC-02 cycle refusal at write time | depends CS-01 + CS-04 | A |
| CS-06 | recurrence kernel..worker..app (7 crates) — rrule-rs align with calendar ADR-CAL-0002 | depends CS-01 | B |
| CS-07 | view-engine kernel..adapter-redis..rest..app (8 crates) | depends CS-01 + CS-04 | B |
| CS-08 | search-index kernel..adapter-meilisearch..worker..app (8 crates) | depends CS-01 + CS-04 | C |
| CS-09 | importers kernel..adapter-{csv,jira,asana,trello,linear,todoist}..rest..worker..app (14 crates) | depends CS-01 + CS-04 + CS-05 + CS-06 | C |
| CS-10 | Cedar policy + DPIA + threat-model + EU AI Act Annex III §4 sign-off | depends CS-01..CS-09 | D |
| CS-11 | OpenAPI + AsyncAPI + Proto contracts + capabilities | depends CS-01..CS-09 | D |
| CS-12 | Helm + Kustomize + dashboards + runbooks | depends CS-01..CS-09 | D |
| CS-13 | Workflow-engine bridge (bidirectional; per ADR-TASKS-0005) | depends CS-01 + workflow-engine GA | D |
| CS-14 | HG-TASKS maturity-claim entry + SLO manifests + canary cohort weighting | depends all | D |
| CS-15 | Cross-µservice bridges (mail / messenger / calendar / drive) — create-task-from-{email,message,event}; calendar due-date bridge; drive attachments | depends CS-01..CS-14 + sibling µservices GA | E |

## Phase gate

Phase-exit gate (per ADR-0139): all 13 AC-IDs green; SLO eligibility
verdict `eligible` for `tasks` µservice over `dev → staging` window;
reviewer-agent APPROVE on each ChangeSet; per-changeset evidence
committed at `microservices/tasks/evidence/multispectrum/*.json`.

## Risks + mitigations

| Risk | Mitigation |
|---|---|
| Dependency-graph cycle false-positive on legitimate complex graphs (project with 1000+ tasks + dense edges) | Bounded BFS per ADR-TASKS-0002 §"performance" with explicit 50ms p99 budget; integration tests with 1k-task / 5k-edge fixtures |
| Custom-field strict coercion breaks legacy imports | Document Hyrum surface #2 explicitly; pre-import validator surfaces type-mismatches as report rows |
| RFC 5545 RRULE alignment with calendar drifts | Shared rrule-rs version pin in workspace Cargo.toml; CI lane `rrule-conformance` runs against same libical corpus as calendar |
| Meilisearch upstream cools | ADR-TASKS-0001 §"search-backend alternatives" lists Tantivy + Quickwit as fallbacks; backend-qualified adapter pattern admits swap |
| EU AI Act Annex III §4 auto-assign conformity assessment delayed | T2 auto-assign REFUSED at Cedar layer for pack-eu + non-pack-eu employment contexts until conformity ADR ships; T1 priority + auto-categorise remain available (lower risk class) |
| Workflow-engine bidirectional bridge cycles (workflow creates task; task triggers workflow; loop) | Workflow-engine durable-execution semantics + idempotency-key per ADR-TASKS-0005; CI integration test with cycle-detection harness |
| Importer payload as malware (Jira/Asana/Trello XML/JSON injection) | Per PRD §"Security", subprocess sandbox + cgroup memory cap + 5min timeout + size limit per file |
| Search-index data leak (tenant A's task indexed under tenant B's prefix due to bug) | LEAN check `oya-check-search-index-tenant-prefix` + property test on adapter; full-rebuild idempotency property |
