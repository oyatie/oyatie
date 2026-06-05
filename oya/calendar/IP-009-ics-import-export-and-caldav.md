---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-calendar-foundation
impl_plan_id: IP-009-ics-import-export-and-caldav
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-calendar
acceptance_lanes: [cargo-nextest, rfc-4791-conformance, ics-import-export-test]
---

# IP-009: ICS import/export and CalDAV

## A. Problem
Calendar migration and interoperability claims fail unless `.ics` import/export and CalDAV read/write behavior are bounded, tested, and attached to the chosen backend decisions.

## B. Approach
Implement the manifest/catalog-named ics import/export adapters for iCalendar, Radicale CalDAV, and SabreDAV CalDAV. Bound parser input before persistence, preserve tenant/context labels, and use CalDAV as an adapter surface over event-store ports.

## C. Deliverables
| Artifact | Role |
|---|---|
| `catalog/oya-calendar-ics-import-export-adapter-icalendar.yaml` | iCalendar adapter catalog anchor. |
| `catalog/oya-calendar-ics-import-export-adapter-caldav-radicale.yaml` | Radicale adapter catalog anchor. |
| `catalog/oya-calendar-ics-import-export-adapter-caldav-sabredav.yaml` | SabreDAV adapter catalog anchor. |
| `src/crates/oya-calendar-ics-import-export-adapter-{icalendar,caldav-radicale,caldav-sabredav}/` | Planned paths named by manifest/catalog. |
| `migration-playbooks/from-google-calendar.md` | Migration behavior anchor. |

## D. Ordered implementation steps
1. Implement bounded `.ics` parser and emitter with max event, size, and recurrence limits.
2. Map imported events to event-store usecases with tenant and context labels.
3. Implement Radicale read/write adapter for default packs.
4. Implement SabreDAV adapter behavior for the US healthcare overlay.
5. Add CalDAV PROPFIND, REPORT, PUT, DELETE, sync-token, and auth tests.
6. Add malformed `.ics`, recurrence-bomb, and large-file import tests.
7. Wire import/export throughput and CalDAV availability SLO evidence.

## E. Acceptance
- `cargo nextest run -p oya-calendar-ics-import-export-adapter-icalendar` passes.
- `cargo nextest run -p oya-calendar-ics-import-export-adapter-caldav-radicale` passes.
- `cargo nextest run -p oya-calendar-ics-import-export-adapter-caldav-sabredav` passes.
- `buck2 build //:quality-lane-registry-authority-check # lane=rfc-4791-conformance --microservice calendar` passes.
- SLOs resolve for `caldav-availability` and `ics-import-throughput`.

## F. Evidence
- PRD FR-07, FR-08, FR-09: `microservices/calendar/PRD.md`.
- Decisions: `decisions/ADR-CAL-0001-caldav-server-backend-selection.md`, `decisions/ADR-CAL-001-icalendar-rfc5545-rfc7986-freebusy-acl.md`.
- Runbooks: `runbooks/caldav-sync-loop.md`, `runbooks/ics-import-failure.md`.
- Migration: `migration-playbooks/from-google-calendar.md`.

## G. Counterpart comparison
Google, Outlook, Apple, Fastmail, and Proton define the import/export and CalDAV bar; Cal.com and Calendly are weaker here. Oyatie must match calendar-suite interop while adding parser bounds, healthcare backend separation, and auditable tenant/context preservation.

## H. Foundation delivery expansion
- Deliverable detail: iCalendar adapter enforces maximum file size, event count, recurrence horizon, and property length.
- Deliverable detail: Radicale adapter covers default CalDAV behavior with tenant/context labels.
- Deliverable detail: SabreDAV adapter covers the US healthcare overlay where the ADR requires it.
- Deliverable detail: import maps every event through event-store usecases, not direct adapter writes.
- Deliverable detail: export redacts fields according to caller policy before `.ics` emission.
- Deliverable detail: sync-token handling records tenant, context, and adapter backend.
- Deliverable detail: malformed file paths emit safe diagnostics and runbook hooks.
- Deliverable detail: Slack calendar imports and app-directory integrations are pressure for robust interop.

## I. Acceptance expansion
- Acceptance detail: malformed `.ics` fixtures must fail before persistence.
- Acceptance detail: recurrence-bomb import tests must reuse recurrence-engine safety limits.
- Acceptance detail: CalDAV tests must cover PROPFIND, REPORT, PUT, DELETE, and sync-token behavior.
- Acceptance detail: export tests must prove redaction for restricted event metadata.
- Acceptance detail: healthcare overlay tests must select SabreDAV without changing default Radicale behavior.
- Acceptance detail: throughput SLO checks must include import and export paths.
- Acceptance detail: migration tests must align with `from-google-calendar.md`.
- Acceptance detail: Slack/Google/Outlook comparisons must be limited to import/export and collaboration interop pressure.

## J. Evidence expansion
- Evidence detail: capture nextest output for all three import/export adapter crates.
- Evidence detail: capture RFC 4791/CalDAV conformance gate output.
- Evidence detail: capture `.ics` parser refusal fixture output.
- Evidence detail: cite `ADR-CAL-0001` and `ADR-CAL-001` for backend and protocol decisions.
- Evidence detail: cite `runbooks/caldav-sync-loop.md` and `runbooks/ics-import-failure.md`.
- Evidence detail: cite `migration-playbooks/from-google-calendar.md` for migration behavior.
- Evidence detail: cite Slack as app-directory calendar import pressure, not the CalDAV authority.
