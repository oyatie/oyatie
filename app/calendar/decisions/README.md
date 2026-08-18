---
doc_class: ADRIndex
microservice: calendar
date: 2026-05-17
owner_team: axis-calendar + council-privacy
doc_status: published
---

# calendar µservice — service-scoped ADRs

This directory holds ADRs that govern the `calendar` µservice exclusively, per the per-microservice flat layout in ADR-0131. Cross-cutting ADRs that govern multiple µservices remain at `docs/decisions/` at the repo root.

Each ADR closes one Open Question (or derived gap) surfaced in `microservices/calendar/PRD.md`, in `microservices/calendar/PHASE-01-CALENDAR-FOUNDATION.md`, or in a capability / runbook / threat-model / DPIA artifact under `microservices/calendar/`.

## Index

| ID | Title | Status | Date | Closes |
|---|---|---|---|---|
| [ADR-CAL-0001](./ADR-CAL-0001-caldav-server-backend-selection.md) | CalDAV server backend selection — Radicale 3.x LTS primary; SabreDAV 4.x adapter alternative; Cyrus IMAP+CalDAV rejected | Accepted | 2026-05-17 | PRD Open Question (CalDAV backend pick — derived from `iac/helm/` chart decision) |
| [ADR-CAL-0002](./ADR-CAL-0002-recurrence-engine-rfc-conformance.md) | RFC 5545 RRULE conformance — full conformance against libical corpus + `rrule-rs` engine; BYSETPOS + EXDATE edge-cases covered | Accepted | 2026-05-17 | PRD Open Question 4 (BYSETPOS × EXDATE behaviour) + AC-03 (RRULE conformance corpus 100%) |
| [ADR-CAL-0003](./ADR-CAL-0003-jmap-vs-caldav-frontend-priority.md) | JMAP Calendars vs CalDAV — CalDAV (RFC 4791) ships first at M03; JMAP Calendars (draft-ietf-jmap-calendars) ships M04 once draft stabilises | Accepted | 2026-05-17 | PRD §"Functional Requirements" FR-09 prioritisation; parallels ADR-MAIL-0003 SDK launch order |
| [ADR-CAL-0004](./ADR-CAL-0004-tzdb-refresh-and-pinning-policy.md) | IANA tzdb refresh + per-tenant pin policy — automated tz-announce poller; cluster-default pinned to N; per-tenant override allowed for regulated sectors | Accepted | 2026-05-17 | PRD Open Question 2 (chrono-tz vs ICU4X tz handling) + derived gap from `runbooks/timezone-db-refresh.md` |

## Authoring conventions

- ADR ID format: `ADR-CAL-XXXX` (4-digit, scope-prefixed) per ADR-0131 service-scoped-ADR convention.
- Each ADR carries: Status, Date (ISO yyyy-mm-dd), Context, Decision, Alternatives Considered (≥3 per decision; each with Pros/Cons/Rejected reason), Consequences (≥3 downstream impacts), References.
- Service-scoped ADRs may reference cross-cutting ADRs (`ADR-####` at repo root) and sibling µservice ADRs. Cross-µservice citations encouraged where decisions are genuinely paired (e.g., ADR-CAL-0003 ↔ ADR-MAIL-0003).
- Lifecycle per ADR-0131 §"ADR Lifecycle": `Proposed → Accepted → (Superseded by ADR-CAL-NNNN | Deprecated)`. Never delete; supersede.

## Open questions not yet closed

| PRD Open Question | Status | Notes |
|---|---|---|
| #1 (native conferencing vs Workflow-trigger to external Zoom/Meet) | Open | subsequent-to-M03-completion ADR; will pair with messenger huddles ADR-MSGR-0001 |
| #3 (federation with Google/Outlook coexistence mode vs migration-only) | Open | subsequent-to-M03-completion; parallels ADR-MSGR-0004 federation posture |

These remain in `microservices/calendar/PRD.md` §"Open Questions"; future ADRs land here with sequential IDs.

## References

- ADR-0131 (per-microservice flat layout + service-scoped ADR convention).
- agent-skills documentation-and-adrs SKILL.md — ADR template authority.
- `microservices/mail/decisions/README.md` — sibling µservice ADR index pattern.
- `microservices/messenger/decisions/README.md` — sibling µservice ADR index pattern.
