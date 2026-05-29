---
ip_id: IP-026
microservice: comms-email
bounded_context: unsubscribe-handling
layer: adapter
related_adrs: [ADR-0263, ADR-0252, ADR-0201]
---

# IP-026 — unsubscribe and suppression event emission

## Goal

Publish unsubscribe and suppression state changes through the service event contract so
cross-region consumers converge on "do not send" before another campaign or retry escapes.
This IP must use the real AsyncAPI file and existing suppression policy vocabulary.

## Service anchors

- AsyncAPI target: `microservices/comms-email/contracts/asyncapi.yaml`.
- Existing suppression channel:
  `comms-email.suppression-inserted` at `oya/comms-email/suppression-inserted/v1`.
- Suppression reason enum in `contracts/asyncapi.yaml`: `HardBounce`, `Complained`,
  `OperatorManual`, `RegulatoryOptOut`, `GdprErasure`.
- Policy: `microservices/comms-email/policy/comms-email-suppression-list.cedar`.
- Residency: `microservices/comms-email/policy/data-residency.cedar`.

## Adapter behavior

1. Consume normalized unsubscribe events from IP-019.
2. Publish suppression insertions with tenant id, recipient, reason, event id, and source
   region.
3. Enforce replay-window and HMAC checks consistently with
   `policy/abuse-defence.cedar` webhook anti-spoof controls.
4. Co-locate suppression state with tenant home jurisdiction per `policy/data-residency.cedar`.
5. Emit audit-chain evidence for every propagation attempt and reject.

## Counterpart refs

- IP-019 owns unsubscribe domain decisions.
- IP-010 owns suppression-list persistence and hot-path lookup.
- IP-021 emits hard-bounce and complaint suppressions through the same channel.
- IP-008/IP-012 provide webhook/audit-chain emission patterns.

## Acceptance

- Event names and payload fields match `contracts/asyncapi.yaml`.
- New unsubscribe-specific channel additions, if needed, are made in `asyncapi.yaml` before
  implementation is claimed.
- HMAC/replay handling cites existing `policy/abuse-defence.cedar` semantics.
- Suppression propagation never violates `policy/data-residency.cedar`.
