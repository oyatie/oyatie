---
ip_id: IP-024
microservice: comms-email
bounded_context: list-management
layer: rest
related_adrs: [ADR-0253, ADR-0258, ADR-0201]
---

# IP-024 — list management REST contract

## Goal

Expose tenant list, subscriber, segment, and preference management through the service
OpenAPI contract. The current `contracts/openapi.yaml` exposes sends, bounces,
suppressions, webhooks, and from-domain onboarding, but not list-management routes.

## Service anchors

- Contract target: `microservices/comms-email/contracts/openapi.yaml`.
- Capability: `microservices/comms-email/capabilities/T2-list-manage.json`.
- Policy: `microservices/comms-email/policy/action-authorization.cedar` for
  `comms-email-admins` tenant-admin access.
- Abuse and reputation gates: `microservices/comms-email/policy/abuse-defence.cedar`.
- Suppression counterpart: `microservices/comms-email/policy/comms-email-suppression-list.cedar`.

## Contract delta

Extend `contracts/openapi.yaml` with tenant-scoped routes for:

- Creating and deleting lists.
- Importing subscribers with consent source and purpose.
- Creating segments within a tenant list.
- Cursor-paginating subscribers.
- Updating per-purpose recipient preferences where a valid recipient token or tenant-admin
  authorization exists.

The API must make suppression status visible for eligibility decisions without allowing
tenant admins to remove protected suppression reasons.

## Counterpart refs

- IP-018 owns list use-case behavior and audit events.
- IP-019 owns unsubscribe and preference domain state.
- IP-010 owns suppression-list storage and removal rules.
- IP-020 feeds list-hygiene warnings after reputation drops.

## Acceptance

- OpenAPI route additions cite the `T2-list-manage.json` action vocabulary.
- Subscriber import requires tenant id, purpose, consent source, and acquisition timestamp.
- Protected suppression removal remains governed by `comms-email-suppression-list.cedar`.
- List routes are not represented as present until `contracts/openapi.yaml` changes.
