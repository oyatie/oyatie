---
doc_class: IP
ip_id: IP-006-async-event-surface
microservice: itsm
status: rewritten-wave-15-ip-substance
date: 2026-05-21
owner_team: axis-itsm + council-product
counterparts: [ServiceNow ITSM, Jira Service Management, Freshservice]
source_artifacts:
  - microservices/itsm/contracts/asyncapi-v1.yaml
  - microservices/itsm/src/adapter/mod.rs
  - microservices/itsm/src/domain/mod.rs
  - microservices/itsm/tests/integration.rs
---

# IP-006 ITSM Async Event Surface

## A. Problem
The stamped version claimed an async event surface but never named an ITSM event, topic, payload, or verifier. The concrete gap is narrower and more useful: ITSM needs a tenant-scoped event contract for incident open, SLA breach, change approval, CMDB relation update, and major incident bridge handoff so ServiceNow-style flows do not become untyped webhook glue.

ServiceNow ITSM, Jira Service Management, and Freshservice all expose workflow/event automations around incident, change, SLA, and service catalog actions. Oyatie's differentiator is that each event is emitted only after Cedar authorization and carries audit-chain correlation instead of being a best-effort automation trigger.

## B. Approach
Use `microservices/itsm/contracts/asyncapi-v1.yaml` as the public AsyncAPI 3.1.0 contract and bind it to the concrete adapter in `src/adapter/mod.rs`.

The first implementation should keep the topic family intentionally small:

| Event | Rust type or enum | Trigger |
|---|---|---|
| `itsm.incident.opened` | `IncidentOpenedEvent` + `AuditEventKind::IncidentOpened` | `OpenIncident::execute` succeeds |
| `itsm.sla.breached` | `SlaBreachedEvent` + `AuditEventKind::SlaBreached` | `RecomputeSla::execute` detects elapsed breach |
| `itsm.change.approved` | `ChangeApprovedEvent` + `AuditEventKind::ChangeApproved` | `ApproveChange::execute` succeeds |
| `itsm.cmdb.relation_updated` | future `CmdbRelationUpdatedEvent` | CMDB sync capability updates relation edge |
| `itsm.major_incident.bridge_opened` | future bridge event | P0/P1 incident opens MLS incident-room |

The event envelope must carry `tenant_id`, `principal_id` where available, `audit_event_class`, `event_time`, and `deal_set_id` where marketplace settlement is involved. It must keep raw tenant id inside signed evidence, not high-cardinality metrics.

## C. Deliverables
- Extend `microservices/itsm/contracts/asyncapi-v1.yaml` with one message per ITSM event rather than a single generic `ActionAccepted`.
- Extend `microservices/itsm/src/adapter/mod.rs` `asyncapi` module with serializers for CMDB and major-incident bridge events.
- Keep `IncidentOpenedEvent`, `SlaBreachedEvent`, and `ChangeApprovedEvent` aligned with `AuditEventKind` in `src/domain/mod.rs`.
- Add focused tests in `microservices/itsm/tests/integration.rs` for topic names and payload fields.
- Add audit event class names compatible with ADR-0263 and the dashboards under `microservices/itsm/dashboards/`.

## D. Implementation
1. Split `ActionAccepted` in `contracts/asyncapi-v1.yaml` into typed messages for incident, SLA, change, CMDB, and major incident bridge.
2. Add required envelope fields: `tenant_id`, `audit_event_class`, `event_time`, `transport_profile`, and operation-specific ids such as `ticket_id` or `change_id`.
3. In `src/adapter/mod.rs`, add serializer functions beside `ItsmAsyncApiHandler::incident_opened`, using the same `ServiceResult<PublishedMessage>` shape.
4. In `src/domain/mod.rs`, verify `AuditEventKind` has a one-to-one mapping with each emitted event.
5. Update `OpenIncident`, `RecomputeSla`, and `ApproveChange` call sites only if they need richer receipt metadata; keep usecase purity intact.
6. Add integration tests proving `itsm.incident.opened`, `itsm.sla.breached`, and `itsm.change.approved` serialize tenant id and audit kind.
7. Add a negative test for invalid identifiers using existing `TenantId::parse` behavior to prevent poisoned event envelopes.
8. Document consumer expectations in this IP, not in a generated event catalog.

## E. Acceptance
- `cargo test -p oya-itsm-service-management-service asyncapi_handler_serializes_incident_opened_event`
- New tests prove each event topic is deterministic and tenant-scoped.
- `contracts/asyncapi-v1.yaml` remains AsyncAPI 3.1.0 and contains no placeholder payload fields.
- No event can be emitted by bypassing `AuditPublisher::publish_audit` in `src/usecase/mod.rs`.

## F. Evidence
- `microservices/itsm/contracts/asyncapi-v1.yaml` currently defines `itsm.events.v1` and `ItsmActionAccepted`.
- `microservices/itsm/src/adapter/mod.rs` currently serializes `IncidentOpenedEvent`, `SlaBreachedEvent`, and `ChangeApprovedEvent`.
- `microservices/itsm/src/domain/mod.rs` currently defines `AuditEventKind::{IncidentOpened,SlaBreached,ProblemLinked,ChangeApproved,CmdbRelationUpdated,MajorIncidentBridgeOpened}`.
- `microservices/itsm/tests/integration.rs` already validates `itsm.incident.opened` serialization.
- ADR-0263 supplies audit event discipline; ADR-0244 supplies tenant scoping.

## G. Counterparts
| Counterpart | Gap closed by this IP |
|---|---|
| ServiceNow ITSM Flow Designer events | Typed incident/change/SLA events without Now Platform lock-in |
| Jira Service Management automation triggers | Tenant-scoped events with Cedar decision and audit-chain evidence |
| Freshservice workflow automations | Event envelope ties automation to DealSet, tenant, and audit evidence |

## H. Cold-start buildability notes
- Start with the existing `ItsmAsyncApiHandler::incident_opened` test before editing the contract.
- Add one event type at a time; do not expand all event families in a single unverified pass.
- Keep topic prefixes deterministic so replay consumers can subscribe by bounded context.
- Treat `deal_set_id` as required only for marketplace-bound actions.
- Record missing principal ids as a follow-up if current usecase receipts cannot provide them yet.
- Validate payload JSON for enum names before adding downstream consumers.
- Keep transport profile constant aligned with the current AsyncAPI payload.
- Do not add a broker-specific topic syntax until runtime broker selection is present.
- Update dashboard evidence only after event class names are stable.
- Preserve backward compatibility for existing `ItsmActionAccepted` consumers during transition.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/itsm/contracts/asyncapi-v1.yaml`, `microservices/itsm/contracts/itsm-v1.proto`, `microservices/itsm/contracts/local-asyncapi-v1.yaml`, `microservices/itsm/contracts/local-openapi-v1.yaml`, `microservices/itsm/contracts/local-operations-v1.proto`, `microservices/itsm/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`asyncapi`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.
