---
ip_id: IP-019
microservice: comms-email
bounded_context: unsubscribe-handling
layer: domain
related_adrs: [ADR-0201, ADR-0244, ADR-0252, ADR-0272]
---

# IP-019 — unsubscribe and preference domain

## Goal

Implement one-click unsubscribe and per-purpose preference changes as tenant-scoped domain
state that feeds the suppression list and cross-region propagation path. The domain must
distinguish recipient self-service from operator suppression removal.

## Service anchors

- Policy: `microservices/comms-email/policy/action-authorization.cedar` permits
  `Action::"unsubscribe"` and `Action::"view_preferences"` only for recipient principals
  with a valid unsubscribe token.
- Suppression policy:
  `microservices/comms-email/policy/comms-email-suppression-list.cedar` controls insertion
  and removal authority.
- Residency rule: `microservices/comms-email/policy/data-residency.cedar` requires
  suppression list placement in the tenant home jurisdiction.
- SLO: `microservices/comms-email/slos/suppression-lookup-latency-p99.openslo.yaml`
  sets the hot-path suppression lookup target.

## Domain behavior

1. Validate signed unsubscribe token, tenant id, recipient, purpose, and expiry.
2. Apply one-click global unsubscribe for compliant mail headers.
3. Apply preference-center updates per purpose without removing regulatory opt-outs.
4. Insert suppression entries with reason `RegulatoryOptOut` when the request is global.
5. Emit unsubscribe evidence for audit and hand cross-region propagation to IP-026.

## Counterpart refs

- IP-010 owns suppression storage and lookup behavior.
- IP-018 provides list and purpose membership context.
- IP-026 publishes unsubscribe propagation events using the AsyncAPI surface.
- IP-024 must expose preference/list management REST surfaces before clients can manage
  preferences directly.

## Acceptance

- Recipient self-service never grants suppression removal authority.
- Regulatory opt-out entries follow `policy/comms-email-suppression-list.cedar`.
- Suppression region matches `policy/data-residency.cedar`.
- IP-026 receives a normalized unsubscribe domain event for propagation.
