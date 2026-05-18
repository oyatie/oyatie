---
doc_class: DeprecationNotice
template_id: TPL-DEPRECATION-NOTICE
microservice: tasks
deprecated_artifact: oya-connect-tasks-* crate family
status: Deprecated
deprecation_date: 2026-05-17
removal_target: advisory — HG-TASKS accepts at p99 SLOs sustained 30d
related_adrs: [ADR-0126, ADR-0131, ADR-0132, ADR-0133, ADR-0134, ADR-TASKS-0001, ADR-TASKS-0002, ADR-TASKS-0003, ADR-TASKS-0004, ADR-TASKS-0005, ADR-TASKS-0006]
related_specs: [/specs/products/connect/tasks.json]
owner_team: axis-tasks
date: 2026-05-17
doc_status: published
---

# Deprecation Notice: `oya-connect-tasks-*` crate family

> Formal deprecation notice in the format prescribed by the agent-skills
> `deprecation-and-migration` skill SKILL.md §"Step 2: Announce and Document".

## Status

**Deprecated as of 2026-05-17.**

## Replacement

`oya-tasks-*` crate family under `microservices/tasks/src/crates/` per
ADR-0131. See **`microservices/tasks/migration-from-connect.md`** for the
full import-path map (60+ crate mappings), Hyrum's-Law-bound surface
callouts (7 surfaces — state-transition ordering, custom-field type
coercion, dependency cycle edge-cases, webhook payload field ordering,
notification timing observable, recurring horizon, importer assignee
resolution), configuration delta table, runbook continuity table (1
preserved + 6 net-new), and step-by-step migration guide.

## Removal date

**Advisory — no hard deadline.** Concrete removal target is HG-TASKS
accepts at p99 SLOs sustained 30d (per ADR-0126 retirement trigger #3).
Following the 5-month Strangler window in ADR-0134 (Phase 2 adapter
soak + Phase 3 canary), the indicative advisory removal date is
**2026-11-17**, gated on the SLO trigger.

## Reason

The legacy `oya-connect-tasks-*` family was authored before the
following ADRs crystallised; each ADR makes the legacy shape non-
conforming:

1. **ADR-0132 — no-suite forward-policy.** `connect-*` encodes bundle
   membership at the architecture layer; bundle membership is a
   brand-layer concept and must not appear in crate names.
2. **ADR-0130 — agentic SLO-gated promotion.** Tasks needs independent
   SLO targets per surface (task-list-render latency, task-create
   latency, task-update latency, cross-project search latency, bulk-
   update latency, recurring-materialise latency, webhook-fire
   latency, dependency-cycle-detection correctness 100%, auto-assign-
   fairness correctness); a `connect-*` umbrella SLO cannot serve
   them.
3. **ADR-0131 — per-µservice flat layout.** Tasks's IaC, runbooks,
   threat-model, DPIA, compliance, capacity-model, cost-budget all
   need to live under one folder (`microservices/tasks/`).
4. **ADR-0133 — 11-pack-overlay program.** pack-kr (KR PIPA + 근로기준법
   retention), pack-eu (GDPR + EU AI Act Annex III §4 employment-context
   refusal), pack-us-healthcare (HIPAA clinical-task assignment), pack-
   jp, pack-sg, pack-au, pack-in, pack-br, pack-ae, pack-ksa — each
   lives at per-µservice overlay granularity.
5. **ADR-TASKS-0001 → ADR-TASKS-0006** — tasks-specific decisions (data
   model, dependency graph, rrule alignment, view engine + CRDT scope,
   workflow bridge, AI auto-assign EU AI Act bounds) need to live at
   per-µservice ADR granularity, not at the Connect suite level.

## Migration Guide pointer

→ **`microservices/tasks/migration-from-connect.md`**

Includes: 1:1 import-path map (60+ mappings); net-new-boundary
features (Meilisearch search, 5 third-party importers, recurrence
worker, view-engine Redis, dependency cycle prevention, workflow
bridge, EU AI Act Cedar refusal, create-task-from-{email,message,
calendar-event}); concrete `use` and `Cargo.toml` rewrites;
configuration delta table; dual-context isolation invariant
preservation; Hyrum's-Law surface callouts (7 surfaces); runbook
continuity table (1 preserved + 6 net-new); 5-step migration recipe;
6-phase Strangler timeline; verification checklist.

## Affected packages enumerated

Per `find crates -maxdepth 1 -type d -name 'oya-connect-tasks-*'`
(2026-05-17 workspace state):

| Currently extant in `crates/` | Mapped replacement |
|---|---|
| `oya-connect-tasks-domain` | split per BC → `oya-tasks-{task-store,project-list,view-engine,dependency-graph,recurrence,search-index,importers}-domain` |

Plus all `oya-connect-tasks-{kernel,usecase,api,adapter*,rest,worker,
sdk,app}-*` crates scaffolded during Phase 2 adapter authoring.

## Breaking changes flagged per `feedback_no_silent_regression`

| Change | Phase | Breaking? | Sunset notice |
|---|---|---|---|
| New `oya-tasks-*` crates ship in parallel | 1 | No (additive) | — |
| New `oya-tasks-search-index-adapter-meilisearch` | 1 | No (net-new) | — |
| New `oya-tasks-importers-adapter-{jira,asana,trello,linear,todoist}` | 1 | No (net-new) | — |
| Cycle prevention at write time (ADR-TASKS-0002) | 1 | **Behaviourally divergent** for consumers with latent cycles | adapter does NOT mask; documented in migration guide Hyrum #3 |
| Strict custom-field coercion (ADR-TASKS-0001) | 1 | **Behaviourally divergent** for consumers writing `"1"` to number-typed field | adapter does NOT mask; documented Hyrum #2 |
| Recurring horizon bounded at 5y (ADR-TASKS-0003) | 1 | **Behaviourally divergent** for unbounded legacy recurring tasks | adapter does NOT mask; documented Hyrum #6 |
| Importer strict assignee resolution (ADR-TASKS-0001) | 1 | **Behaviourally divergent** — refuses ambiguous matches instead of silent fuzzy-misassign | adapter does NOT mask; documented Hyrum #7 |
| Cedar refusal of auto-assign in EU AI Act Annex III §4 employment context (ADR-TASKS-0006) | 1 | **New constraint** — auto-assign refused at Cedar layer until conformity assessment | adapter does NOT mask; documented in capabilities/T2-auto.yaml |
| `oya-connect-tasks-migration-adapter` shim authored | 2 | No (preserves legacy symbol surface for non-strengthened paths) | — |
| Feature-flagged canary 10→50→100% | 3 | No (additive, gated) | — |
| Zero-usage verification | 4 | No (observability only) | — |
| **`oya-connect-tasks-*` crates removed from workspace** | **5** | **YES — breaking** | **6-mo advisory sunset from 2026-05-17** |
| `microservices/connect/` umbrella folder removed | 6 | No | — |

Per `feedback_no_silent_regression.md`, the Phase 5 breaking change carries:

- **This deprecation notice** (renders the change loud + immediate +
  CI-detectable).
- **ADR-0134** (carries the migration policy decision).
- **ADR-TASKS-0002 + ADR-TASKS-0001 + ADR-TASKS-0003 + ADR-TASKS-0006**
  (each specifically documents the named behavioural strengthenings as
  deliberate, owner-authored design choices — NOT silent regressions).
- **Version bump.** The `Cargo.toml` of every consumer crate is bumped
  per semver when its legacy imports are removed (treating the
  `oya-connect-tasks-*` re-export as the public contract).
- **Sunset schedule.** 6-month advisory window from this notice;
  concrete date 2026-11-17 contingent on the HG-TASKS SLO trigger.
- **Owning-axis migration ChangeSets.** axis-tasks ships migration
  ChangeSets for every known internal consumer per the Churn Rule
  before Phase 5.

## Verification (per skill SKILL.md §"Verification")

- [ ] Replacement is production-proven and covers all critical use cases —
  HG-TASKS gate at p99 SLO sustained 30d.
- [ ] Migration guide exists with concrete steps and examples —
  `migration-from-connect.md`.
- [ ] All active consumers have been migrated — verified by Phase 4
  commands (see ADR-0134 §Phase 4).
- [ ] Old code, tests, documentation, configuration removed — Phase 5
  commands.
- [ ] No references to the deprecated system remain — `rg
  "oya_connect_tasks" --type rust` produces zero hits outside
  historical surfaces.
- [ ] Deprecation notices removed — this notice deletes itself in Phase 5.

## References

- ADR-0126, ADR-0131, ADR-0132, ADR-0133, ADR-0134.
- ADR-TASKS-0001 (data model + custom fields strict coercion).
- ADR-TASKS-0002 (dependency graph cycle prevention at write time).
- ADR-TASKS-0003 (recurring tasks via rrule-rs; align calendar ADR-CAL-0002).
- ADR-TASKS-0004 (view engine + board realtime; Loro scope).
- ADR-TASKS-0005 (automation engine cross-µservice; workflow-engine bridge).
- ADR-TASKS-0006 (AI auto-assign + EU AI Act Annex III §4 bounds).
- `microservices/tasks/migration-from-connect.md` — full migration guide.
- `microservices/tasks/PRD.md` — target-state product definition.
- `microservices/tasks/runbooks/*.md` — 7 runbooks.
- `feedback_no_silent_regression.md`.
- agent-skills deprecation-and-migration SKILL.md.
- RFC 5545 — iCalendar VTODO (aligned subset).
- draft-ietf-jmap-tasks — JMAP Tasks (deferred to M05).
- `microservices/calendar/deprecation-notice.md` — sibling reference template.
