---
ip_id: IP-026
microservice: finops-portal
bounded_context: showback-chargeback
layer: adapter
related_adrs: [ADR-0263]
---

# IP-026 — showback-chargeback AsyncAPI emitter

## Goal

Fan-out showback-chargeback events to ops-dashboard + tenant-admin.

## Channels

- `oya.finops-portal.showback-chargeback-emitted.v1`

## Acceptance

- HMAC-signed envelope.
- Audit chain seal per ADR-0263.
