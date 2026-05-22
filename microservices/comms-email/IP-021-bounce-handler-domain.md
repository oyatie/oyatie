---
ip_id: IP-021
microservice: comms-email
bounded_context: bounce-handling
layer: domain
related_adrs: [ADR-0201, ADR-0263]
---

# IP-021 — bounce and complaint handler domain

## Goal

Classify provider bounce and complaint events into comms-email domain outcomes, insert
hard-bounce or complaint suppressions, and engage the tenant bounce-storm path. This IP
specializes the broader IP-008 webhook pipeline.

## Service anchors

- Capability: `microservices/comms-email/capabilities/T1-bounce-handle.json` defines
  `bounce_classify`, `bounce_emit_suppression`, and audit class `oya.comms-email.bounce`.
- AsyncAPI: `microservices/comms-email/contracts/asyncapi.yaml` defines
  `comms-email.bounce-classified` and `comms-email.suppression-inserted` channels.
- Policy: `microservices/comms-email/policy/comms-email-suppression-list.cedar` permits
  insertion by the `comms-email-bounce-handler` role.
- Runbook: `microservices/comms-email/runbooks/bounce-storm-mitigation.md` defines the
  >5% bounce-rate incident path.

## Domain behavior

1. Consume normalized delivery events from IP-008.
2. Classify bounce as hard or soft using provider code, SMTP code, and provider reason.
3. Emit `BounceClassified` on `oya/comms-email/bounce-classified/v1`.
4. Insert suppression for hard bounce or complaint with reason from the AsyncAPI enum.
5. Track tenant bounce rate and trigger the bounce-storm breaker above 5%.

## Counterpart refs

- IP-008 verifies provider webhook signatures and normalizes event shape.
- IP-010 stores and serves suppression-list state.
- IP-020 consumes bounce and complaint rates for reputation scoring.
- IP-018 uses suppression state to keep lists sendable only where allowed.

## Acceptance

- Hard bounce inserts suppression through the existing policy role, not by bypassing policy.
- Soft bounce does not insert suppression; it is returned to retry/provider handling.
- AsyncAPI message names match `contracts/asyncapi.yaml`.
- Bounce-storm threshold matches `policy/abuse-defence.cedar` and the runbook.
