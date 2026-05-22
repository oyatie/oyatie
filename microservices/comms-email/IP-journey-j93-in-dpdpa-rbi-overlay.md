---
doc_class: Implementation-Plan
ip_id: IP-journey-j93-in-dpdpa-rbi-overlay
journey_ref: docs/user-journeys/j93-in-dpdpa-rbi-financial-overlay/
microservice: comms-email
status: draft
date: 2026-05-21
---

# IP-journey-j93: India DPDPA / RBI email notices

## A. Problem
J93 needs financial-tenant notices whose templates, provider path, webhook evidence, and suppression state are DPDPA/RBI-aware. comms-email must not own account decisions or financial scoring.

## B. Approach
Use `microservices/comms-email/contracts/openapi.yaml` for send requests, `microservices/comms-email/contracts/asyncapi.yaml` for delivery/bounce events, and `microservices/comms-email/policy/pack-overlay-authorization.cedar` for pack overlay gating.

## C. Deliverables
- RBI overlay notification examples.
- Provider-routing notes tied to SES, Mailgun, Postal, or SMTP fallback.
- Suppression-list and webhook evidence references.
- Reputation monitor checks for high-volume financial notices.

## D. Implementation
1. Add `rbi_overlay_ref`, `financial_notice_type`, and `template_version` examples.
2. Require pack overlay authorization before provider selection.
3. Apply suppression-list policy before send.
4. Emit async delivery and complaint evidence.
5. Watch `microservices/comms-email/dashboards/reputation-monitoring.json`.
6. Use `microservices/comms-email/runbooks/reputation-drop-circuit-breaker-engaged.md` on reputation drops.

## E. Acceptance
- RBI examples cite real contracts and pack overlay policy.
- Reputation monitoring is part of the readiness gate.
- Bounce/complaint events are auditable.
- Financial account decisions remain out of scope.

## F. Evidence
- Journey: `docs/user-journeys/j93-in-dpdpa-rbi-financial-overlay/README.md`.
- Policy: `microservices/comms-email/policy/pack-overlay-authorization.cedar`.
- Dashboard: `microservices/comms-email/dashboards/reputation-monitoring.json`.

## G. Counterparts
| Counterpart | Gap closed |
|---|---|
| AWS SES | Adds financial-pack authorization and audit evidence. |
| Mailgun | Adds tenant-scoped reputation safeguards. |
| SendGrid / Twilio | Adds policy-gated event evidence before provider dispatch. |
