---
doc_class: PhasePlan
template_id: TPL-PHASE-PLAN
microservice: calendar
phase_id: PHASE-01
phase_title: Calendar Foundation — event-store + recurrence + availability + room-booking + invitation + ics/CalDAV
status: Accepted
date: 2026-05-17
owner_team: axis-calendar
related_adrs: [ADR-0056, ADR-0105, ADR-0106, ADR-0117, ADR-0126, ADR-0130, ADR-0131, ADR-0132, ADR-0133]
doc_status: published
---

# PHASE-01 — Calendar Foundation

## Intent

Stand up the six bounded contexts (event-store, recurrence-engine, availability-resolver, room-booking, invitation-flow, ics-import-export) with full Layer-A + Layer-B substrate, Bominal ADR-0208 + ADR-0215 inheritance, RFC 5545 + RFC 4791 conformance, dual-context isolation, audit-chain emission, and SLO-gated promotion. Phase exit = AC-01 through AC-12 in `PRD.md` green.

## Phase scope

In-scope:
- 44 crates per the layer mapping table.
- Postgres event-store schema + per-tenant RLS + tenant-DEK envelope encryption.
- Redis availability cache + cross-tenant cache invalidation.
- IANA tzdata + chrono-tz integration.
- RFC 5545 .ics parser + emitter (via `oya-calendar-ics-import-export-adapter-icalendar` wrapping a hardened parser; `icalendar-rs` adopted with `vetted-fork` posture per ADR-0133).
- RFC 4791 CalDAV adapter (in-house implementation; no third-party crate adoption at GA).
- Workflow events produced + consumed per `PRD.md`.
- Ontology writes + reads per `PRD.md`.
- HG-CALENDAR hyperscaler-maturity claim registered per ADR-0123 + ADR-0133.

Out-of-scope (deferred):
- Native conferencing (oyatie-Connect-Conference) — ADR follow-up.
- Federation with external Google / Outlook as source-of-truth — migration-only at GA.
- ML-based smart scheduling — post-GA.

## Phase outputs

| Output | Path | Owner |
|---|---|---|
| 44 crates | `crates/oya-calendar-*` | axis-calendar |
| Postgres schema migrations | `microservices/calendar/iac/helm/postgres/migrations/` | axis-calendar |
| Helm charts | `microservices/calendar/iac/helm/{postgres,redis,timezone-data}` | ops-sre-reliability |
| Kustomize overlays | `microservices/calendar/iac/kustomize/{base,overlays/pack-kr}` | ops-sre-reliability |
| OpenAPI / AsyncAPI / Proto contracts | `microservices/calendar/contracts/` | axis-calendar |
| Cedar policies | `microservices/calendar/policy/*.cedar` | ops-security |
| Runbooks | `microservices/calendar/runbooks/*.md` | ops-sre-reliability |
| Dashboards | `microservices/calendar/dashboards/*.json` | axis-observability |
| HG-CALENDAR claim entry | `registry/hyperscaler-maturity-claims.json` | axis-calendar |

## Phase milestones (ChangeSets, per ADR-0110)

| CS | Title | DAG-position | Slice |
|---|---|---|---|
| CS-01 | event-store kernel + domain + usecase + api | Layer-B base | A |
| CS-02 | event-store -adapter-postgres + RLS schema | depends CS-01 | A |
| CS-03 | event-store rest + worker + sdk + app | depends CS-02 | A |
| CS-04 | recurrence-engine kernel..app (6 crates) | depends CS-01 | B |
| CS-05 | availability-resolver kernel..adapter-redis + rest + worker + app (9 crates) | depends CS-01 + CS-04 | B |
| CS-06 | room-booking kernel..rest + app (7 crates) | depends CS-01 | B |
| CS-07 | invitation-flow kernel..worker + app (7 crates) | depends CS-01 + mail µservice | C |
| CS-08 | ics-import-export kernel..adapter-icalendar + adapter-caldav + rest + app (9 crates) | depends CS-01 + CS-04 | C |
| CS-09 | Cedar policy + DPIA + threat-model sign-off | depends CS-01..CS-08 | D |
| CS-10 | OpenAPI + AsyncAPI + Proto contracts + capabilities | depends CS-01..CS-08 | D |
| CS-11 | Helm + Kustomize + dashboards + runbooks | depends CS-01..CS-08 | D |
| CS-12 | HG-CALENDAR maturity-claim entry + SLO manifests + canary cohort weighting | depends all | D |

## Phase gate

Phase-exit gate (per ADR-0130): all 12 AC-IDs green; SLO eligibility verdict `eligible` for `calendar` µservice over `dev → staging` window; reviewer-agent APPROVE on each ChangeSet; per-changeset evidence committed at `microservices/calendar/evidence/multispectrum/*.json`.

## Risks + mitigations

| Risk | Mitigation |
|---|---|
| RFC 5545 RRULE corner cases (BYSETPOS + BYDAY + EXDATE interaction) | Adopt libical RFC 5545 corpus as conformance suite; 100% pass before GA |
| CalDAV client diversity (Apple / Thunderbird / Evolution) | E2E test against three real clients in staging |
| Cross-tenant availability privacy regression | LEAN check `oya-check-cross-tenant-availability-projection` (NEW) refuses build if projection includes raw fields |
| Time-zone DB staleness (IANA tzdata) | Hourly refresh job; verification gate before deploy |
| Recurrence storm DoS | Recurrence horizon ≤ 5y enforced at API; worker rate-limit |
| Mail µservice unavailable for invitation fanout | Async retry + dead-letter; surface "delivery-failed" recipient card |
