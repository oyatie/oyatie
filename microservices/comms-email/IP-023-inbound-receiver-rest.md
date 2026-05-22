---
ip_id: IP-023
microservice: comms-email
bounded_context: inbound-receiving
layer: rest
related_adrs: [ADR-0253, ADR-0258, ADR-0201]
---

# IP-023 — inbound receiver REST contract

## Goal

Add the tenant/admin REST contract for inbound received-message listing, quarantine
inspection, and quarantine release. This IP is contract work against the existing
`contracts/openapi.yaml`; the current OpenAPI file does not yet expose inbound routes.

## Service anchors

- Current REST contract file: `microservices/comms-email/contracts/openapi.yaml`.
- Capability: `microservices/comms-email/capabilities/T3-inbound-receive.json`.
- Policy: `microservices/comms-email/policy/action-authorization.cedar` for tenant-admin
  and inbound-receiver principals.
- Operator procedure:
  `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`.

## Contract delta

Extend `contracts/openapi.yaml` with tenant-scoped paths for:

- Registering inbound webhook/MX routing metadata for a tenant.
- Cursor-paginating received inbound messages.
- Reading quarantine metadata without exposing unsafe raw content by default.
- Releasing a quarantined message after the runbook checks pass.

All route additions must include `tenant_id`, cursor pagination where listing is involved,
and explicit Cedar authorization mapping. Do not claim these routes as implemented until
the OpenAPI diff exists.

## Counterpart refs

- IP-016 supplies message authentication verdicts.
- IP-017 owns quarantine state and audit emission.
- IP-020 consumes aggregate quarantine and authentication-failure metrics.

## Acceptance

- `contracts/openapi.yaml` gains inbound paths in the same changeset that implements this IP.
- Quarantine release maps to the command and rollback in the runbook.
- Route authorization is backed by existing `policy/action-authorization.cedar` principals.
- No direct raw-message download is exposed without an explicit safe-content review state.
