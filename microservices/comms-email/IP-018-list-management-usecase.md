---
ip_id: IP-018
microservice: comms-email
bounded_context: list-management
layer: usecase
related_adrs: [ADR-0201, ADR-0244, ADR-0263]
---

# IP-018 — tenant list management use case

## Goal

Implement tenant-scoped subscriber list lifecycle for imports, segments, deletes, exports,
and consent-carrying membership. This IP binds list operations to comms-email policy and
audit evidence instead of describing a generic contacts feature.

## Service anchors

- Capability: `microservices/comms-email/capabilities/T2-list-manage.json` defines actions
  `list_create`, `list_import`, `list_segment_create`, and `list_delete`.
- Policy: `microservices/comms-email/policy/action-authorization.cedar` allows tenant admins
  in `comms-email-admins` when `principal.tenant_id == resource.tenant_id`.
- Abuse guard: `microservices/comms-email/policy/abuse-defence.cedar` enforces per-tenant
  marketing rate limits and the reputation-drop marketing circuit breaker.
- Counterpart suppression policy:
  `microservices/comms-email/policy/comms-email-suppression-list.cedar`.

## Use-case behavior

1. Create a tenant-owned list record with explicit purpose: transactional, marketing,
   regulatory notice, or tenant-admin announcement.
2. Import subscribers with source, timestamp, consent purpose, and acquisition channel.
3. Segment only within the tenant boundary; cross-tenant joins are forbidden.
4. Before any marketing send, intersect the segment with suppression-list state from IP-010.
5. Emit audit events named in `T2-list-manage.json` for import and segment creation.

## Counterpart refs

- IP-024 owns REST contract changes in `microservices/comms-email/contracts/openapi.yaml`.
- IP-019 consumes list membership for per-purpose unsubscribe and preferences.
- IP-021 and IP-010 feed hard-bounce and complaint suppression back into list eligibility.
- IP-020 may mark a list source as risky after reputation drop.

## Acceptance

- Every subscriber mutation carries tenant id and purpose.
- Marketing eligibility fails closed when `policy/abuse-defence.cedar` reputation or rate
  gates deny the send.
- Audit event names stay aligned with `T2-list-manage.json`.
- No list route is claimed as present until IP-024 adds it to `contracts/openapi.yaml`.
