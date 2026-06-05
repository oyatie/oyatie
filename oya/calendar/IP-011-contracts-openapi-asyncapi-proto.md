---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-calendar-foundation
impl_plan_id: IP-011-contracts-openapi-asyncapi-proto
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-calendar + council-architecture
acceptance_lanes: [openapi-validate, asyncapi-validate, buf-lint, oya-governance-contract-sync]
---

# IP-011: Calendar contracts

## A. Problem
Calendar implementation slices need one contract source for REST, event, and proto consumers; otherwise event-store, availability, room, invite, ICS, and workflow handoff drift.

## B. Approach
Finalize the existing OpenAPI, AsyncAPI, and proto files as the foundation contract set. Each endpoint/event/message must carry tenant, context, data-class, audit, idempotency, and policy-result fields where the PRD requires them.

## C. Deliverables
| Artifact | Role |
|---|---|
| `contracts/openapi/calendar.yaml` | REST API for event, free/busy, room, invitation, and import/export flows. |
| `contracts/asyncapi/calendar-events.yaml` | Event lifecycle, RSVP, room, recurrence, and legal-hold topics. |
| `contracts/proto/calendar.proto` | Typed internal/event schema anchor. |
| `manifest.json` | Contract index consumed by governance checks. |

## D. Ordered implementation steps
1. Validate OpenAPI syntax and schema references.
2. Validate AsyncAPI topic names, idempotency keys, and message payload fields.
3. Lint proto packages and field numbering.
4. Cross-check every PRD must-have feature has a contract command, query, event, or documented non-API path.
5. Verify policy-sensitive responses never include raw event metadata in free/busy projection.
6. Add examples for event create, free/busy, room booking, invitation reply, and `.ics` import job.
7. Update manifest contract entries only if paths changed.

## E. Acceptance
- OpenAPI validator passes for `contracts/openapi/calendar.yaml`.
- AsyncAPI validator passes for `contracts/asyncapi/calendar-events.yaml`.
- `buf lint microservices/calendar/contracts/proto` passes or the repo-equivalent proto lint passes.
- `buck2 build //:quality-lane-registry-authority-check # lane=contract-sync --microservice calendar` passes.
- Contract fields align with `policy/tenant-scope.cedar` and `policy/event-isolation.md`.

## F. Evidence
- PRD functional requirements and workflow tables: `microservices/calendar/PRD.md`.
- Manifest contract index: `microservices/calendar/manifest.json`.
- Existing contracts: `contracts/openapi/calendar.yaml`, `contracts/asyncapi/calendar-events.yaml`, `contracts/proto/calendar.proto`.
- Feature parity matrix: `feature-parity-matrix-2026-05-20.md`.

## G. Counterpart comparison
Google Calendar and Microsoft Graph expose public API contracts; Cal.com exposes booking contracts; Calendly exposes scheduling APIs. Oyatie needs equivalent contract clarity plus policy-result, audit-chain, and dual-context fields absent from those counterpart APIs.

## H. Foundation delivery expansion
- Deliverable detail: OpenAPI examples cover event create/update/cancel, free/busy, room booking, RSVP, and import jobs.
- Deliverable detail: AsyncAPI examples cover lifecycle, RSVP, room conflict, recurrence refusal, and legal-hold events.
- Deliverable detail: proto messages preserve tenant, context, audit, idempotency, and data-class fields.
- Deliverable detail: schema names align with manifest contract entries and catalog crate names.
- Deliverable detail: error responses distinguish policy denial, validation failure, conflict, and degraded remote state.
- Deliverable detail: free/busy responses never include raw private event metadata.
- Deliverable detail: examples include KR and US healthcare pack-sensitive responses.
- Deliverable detail: Slack workflow/app integrations are pressure for clear event and REST contracts.

## I. Acceptance expansion
- Acceptance detail: OpenAPI validation must resolve every `$ref`.
- Acceptance detail: AsyncAPI validation must resolve every channel and message reference.
- Acceptance detail: proto lint must reject package or field-number drift.
- Acceptance detail: contract-sync gate must map every PRD must-have to a command, query, event, or non-API note.
- Acceptance detail: negative examples must include Cedar denial and idempotency conflict responses.
- Acceptance detail: manifest contract entries must point at the same files validated in this IP.
- Acceptance detail: generated SDK smoke tests must compile when contract tooling exists.
- Acceptance detail: Slack, GitHub, and Microsoft-style integrations must remain external consumers of explicit contracts.

## J. Evidence expansion
- Evidence detail: capture OpenAPI validator output for `calendar.yaml`.
- Evidence detail: capture AsyncAPI validator output for `calendar-events.yaml`.
- Evidence detail: capture proto lint output for `calendar.proto`.
- Evidence detail: capture contract-sync gate output for calendar.
- Evidence detail: cite `manifest.json` for registered contract paths.
- Evidence detail: cite `feature-parity-matrix-2026-05-20.md` for counterpart contract claims.
- Evidence detail: cite Slack as collaboration integration pressure requiring stable public contracts.
