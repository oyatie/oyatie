---
doc_class: ADRIndex
microservice: tasks
date: 2026-05-17
owner_team: axis-tasks + council-privacy
doc_status: published
---

# tasks µservice — service-scoped ADRs

This directory holds ADRs that govern the `tasks` µservice exclusively,
per the per-microservice flat layout in ADR-0131. Cross-cutting ADRs
that govern multiple µservices remain at `docs/decisions/` at the repo
root.

Each ADR closes one Open Question (or derived gap) surfaced in
`microservices/tasks/PRD.md`, in
`microservices/tasks/PHASE-01-TASKS-FOUNDATION.md`, or in a capability /
runbook / threat-model / DPIA artifact under `microservices/tasks/`.

## Index

| ID | Title | Status | Date | Closes |
|---|---|---|---|---|
| [ADR-TASKS-0001](./ADR-TASKS-0001-task-data-model-and-custom-fields.md) | Task data model + custom fields — hybrid typed-schema-per-project + flexible JSON columns; strict type coercion (refuse silent coerce-from-string); 6 importers with strict assignee resolution | Accepted | 2026-05-17 | PRD §"Functional Requirements" FR-01 + FR-16 + FR-09 + Hyrum #2 + Hyrum #7 |
| [ADR-TASKS-0002](./ADR-TASKS-0002-dependency-graph-and-cycle-prevention.md) | Dependency graph + cycle prevention at write time — DAG enforcement at adapter + domain layer; circular-dependency policy refuses cycle-creating writes (PRD AC-02 correctness 100%) | Accepted | 2026-05-17 | PRD AC-02 + Hyrum #3 |
| [ADR-TASKS-0003](./ADR-TASKS-0003-recurring-task-engine.md) | Recurring task engine — RFC 5545 RRULE subset aligned with calendar ADR-CAL-0002 rrule-rs 0.13.x; bounded materialisation (5y horizon) | Accepted | 2026-05-17 | PRD §"Functional Requirements" FR-05 + Hyrum #6 |
| [ADR-TASKS-0004](./ADR-TASKS-0004-view-engine-and-board-realtime.md) | View engine + board realtime — Loro CRDT for collaborative description editing only; deterministic re-rank for board DnD moves; aligned with workflow-studio CRDT scope | Accepted | 2026-05-17 | PRD §"Bounded Contexts" `view-engine` + PRD §"Performance" board DnD |
| [ADR-TASKS-0005](./ADR-TASKS-0005-automation-engine-cross-microservice.md) | Automation engine cross-µservice — bidirectional workflow-engine bridge (workflow creates tasks; tasks trigger workflows); no in-µservice durable-execution engine duplication | Accepted | 2026-05-17 | PRD §"Functional Requirements" automation surface + PRD §"Open Questions" |
| [ADR-TASKS-0006](./ADR-TASKS-0006-ai-auto-assign-and-eu-ai-act-bounds.md) | AI auto-assign + EU AI Act Annex III §4 bounds — T2 auto-assign in employment-context REFUSED at Cedar layer pending conformity-assessment ADR per pack | Accepted | 2026-05-17 | PRD §"Functional Requirements" FR-20 + PRD AC-12 + capabilities/T2-auto.yaml |

## Authoring conventions

- ADR ID format: `ADR-TASKS-XXXX` (4-digit, scope-prefixed) per ADR-0131 service-scoped-ADR convention.
- Each ADR carries: Status, Date (ISO yyyy-mm-dd), Context, Decision, Alternatives Considered (≥3 per decision; each with Pros/Cons/Rejected reason), Consequences (≥3 downstream impacts), References.
- Service-scoped ADRs may reference cross-cutting ADRs (`ADR-####` at repo root) and sibling µservice ADRs. Cross-µservice citations encouraged where decisions are genuinely paired (e.g., ADR-TASKS-0003 ↔ ADR-CAL-0002 for rrule-rs alignment).
- Lifecycle per ADR-0131 §"ADR Lifecycle": `Proposed → Accepted → (Superseded by ADR-TASKS-NNNN | Deprecated)`. Never delete; supersede.

## Open questions not yet closed

| PRD Open Question | Status | Notes |
|---|---|---|
| #1 (Gantt timeline editor in tasks vs workflow-studio bridge) | Open | subsequent-to-M04-completion ADR; will pair with workflow-studio's view-bridge ADR |
| #3 (T2 auto-assign in non-EU jurisdictions — when to relax Cedar refusal) | Open | depends on per-pack conformity-assessment ADRs (ADR-TASKS-XXXX per pack) |
| #4 (JMAP-Tasks adapter at M05) | Open | depends on draft-ietf-jmap-tasks IETF stabilisation |
| #5 (Apple Reminders compat via CalDAV-Tasks VTODO at M05) | Open | depends on calendar's CalDAV stack maturity + iOS/macOS demand |

These remain in `microservices/tasks/PRD.md` §"Open Questions"; future ADRs land here with sequential IDs.

## References

- ADR-0131 (per-microservice flat layout + service-scoped ADR convention).
- agent-skills documentation-and-adrs SKILL.md — ADR template authority.
- agent-skills deprecation-and-migration SKILL.md — Strangler Pattern.
- `microservices/calendar/decisions/README.md` — sibling µservice ADR index pattern.
- `microservices/messenger/decisions/README.md` — sibling µservice ADR index pattern.
