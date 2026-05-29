---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-calendar-foundation
impl_plan_id: IP-002-event-store-kernel
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-calendar
acceptance_lanes: [cargo-nextest, oya-governance-data-class, oya-governance-port-location]
---

# IP-002: Event-store kernel

## A. Problem
Calendar's core event model must prove that event content, attendee state, legal hold, retention, and dual-context boundaries exist before adapters or REST handlers can be trusted.

## B. Approach
Build the zero-I/O `oya-calendar-event-store-kernel` crate named in `manifest.json` and `catalog/oya-calendar-event-store-kernel.yaml`. Keep only entities, data-class annotations, and port traits here; persistence, time-zone lookup, and audit emission stay behind ports.

## C. Deliverables
| Artifact | Role |
|---|---|
| `microservices/calendar/catalog/oya-calendar-event-store-kernel.yaml` | Existing catalog anchor for the kernel crate. |
| `microservices/calendar/src/crates/oya-calendar-event-store-kernel/` | Planned crate path already named by this IP and manifest. |
| `CalendarEvent`, `Attendee`, `RetentionPolicyRef`, `LegalHoldRef`, `EventContext` | Core types from PRD bounded-context text. |
| `EventRepository`, `TimeZoneResolver`, `RetentionPolicyResolver`, `LegalHoldStore` | Ports listed in PRD and architecture. |

## D. Ordered implementation steps
1. Create the crate with no database, HTTP, queue, or clock dependencies.
2. Define event identity, tenant identity, context, attendee, recurrence reference, and audit correlation value objects.
3. Add explicit `#[data_class(...)]` annotations for personal and professional event fields.
4. Define repository and policy/hold/tzdb ports with async trait boundaries only if local crate convention already permits async ports.
5. Add compile tests that reject unannotated fields through the existing data-class gate.
6. Add property tests for tenant/context identity equality and redaction-safe debug output.
7. Register the crate in the workspace and catalog without importing other product microservice crates.

## E. Acceptance
- `cargo nextest run -p oya-calendar-event-store-kernel` passes.
- `cargo run -p oya-dev-cli -- gate validate port-location --microservice calendar` passes.
- `cargo run -p oya-dev-cli -- gate validate lean-a1 --microservice calendar` passes.
- `cargo run -p oya-dev-cli -- gate validate data-class --microservice calendar` passes.
- Policy references remain compatible with `policy/event-isolation.md` and `policy/tenant-scope.cedar`.

## F. Evidence
- PRD port-trait table: `microservices/calendar/PRD.md`.
- Catalog: `microservices/calendar/catalog/oya-calendar-event-store-kernel.yaml`.
- Contracts consuming lifecycle state: `contracts/openapi/calendar.yaml`, `contracts/asyncapi/calendar-events.yaml`, `contracts/proto/calendar.proto`.
- SLO pressure: `slos/agenda-render-latency.openslo.yaml` and `slos/notification-delivery-freshness.openslo.yaml`.

## G. Counterpart comparison
Google Calendar and Outlook expose rich event resources, but their tenant isolation and legal-hold semantics are platform-level. This kernel makes Oyatie's counterpart claim concrete by putting tenant id, context, retention, legal hold, and data class into the event type system before storage exists.

## H. Foundation delivery expansion
- Deliverable detail: define `CalendarEventId`, `TenantId`, `CalendarId`, `EventContext`, and `AuditCorrelationId` as separate value objects.
- Deliverable detail: model attendee state without importing invitation-flow behavior.
- Deliverable detail: model retention and legal-hold references as opaque identifiers, not policy-engine structs.
- Deliverable detail: expose repository, time-zone, retention, and hold ports without Postgres or Valkey types.
- Deliverable detail: include redaction-safe debug output for event titles, locations, notes, and attendee metadata.
- Deliverable detail: require data-class annotations for title, location, attendee, description, recurrence, and organizer fields.
- Deliverable detail: include a fake in-memory repository only inside tests.
- Deliverable detail: Slack shared-channel calendar expectations are counterpart pressure for attendee/context modeling, not a dependency.

## I. Acceptance expansion
- Acceptance detail: compile tests must fail when kernel imports database, HTTP, queue, or clock crates.
- Acceptance detail: property tests must prove tenant/context equality does not collapse personal and work calendars.
- Acceptance detail: redaction tests must prove `Debug` output omits sensitive title/location content.
- Acceptance detail: data-class gate must report the exact unannotated field name.
- Acceptance detail: trait signatures must preserve idempotency and audit-correlation parameters.
- Acceptance detail: port tests must support legal-hold refusal without requiring an adapter.
- Acceptance detail: workspace registration must include only the kernel crate for this slice.
- Acceptance detail: Slack, Google, and Outlook comparisons must remain type-system claims, not runtime integration claims.

## J. Evidence expansion
- Evidence detail: capture `cargo nextest run -p oya-calendar-event-store-kernel`.
- Evidence detail: capture the port-location gate proving zero adapter imports.
- Evidence detail: capture data-class gate output over the new kernel crate.
- Evidence detail: cite `catalog/oya-calendar-event-store-kernel.yaml` as the crate registry source.
- Evidence detail: cite `policy/event-isolation.md` for context separation behavior.
- Evidence detail: cite contract files only where they consume kernel lifecycle state.
- Evidence detail: cite Slack as collaboration-calendar interop pressure alongside Google and Outlook event-resource pressure.
