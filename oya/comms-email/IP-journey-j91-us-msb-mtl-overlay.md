---
doc_class: Implementation-Plan
ip_id: IP-journey-j91-us-msb-mtl-overlay
journey_ref: docs/user-journeys/j91-us-state-money-transmitter-licensing/
microservice: comms-email
status: draft
date: 2026-05-21
---

# IP-journey-j91: US MSB / MTL regulated email notices

## A. Problem
J91 needs comms-email to deliver licensed-state notices, regulator acknowledgements, bounce evidence, and suppression decisions without becoming the money-transmission system of record.

## B. Approach
Bind the journey to `microservices/comms-email/contracts/openapi.yaml`, `microservices/comms-email/contracts/asyncapi.yaml`, and `microservices/comms-email/policy/comms-email-send.cedar`. SES, Postal, Mailgun, and SMTP remain provider adapters; tenant and state scope are enforced before send.

## C. Deliverables
- State-scoped transactional-send examples in `microservices/comms-email/contracts/openapi.yaml`.
- Delivery/bounce event examples in `microservices/comms-email/contracts/asyncapi.yaml`.
- Policy references to `microservices/comms-email/policy/comms-email-send.cedar`.
- Runbook references to `microservices/comms-email/runbooks/bounce-storm-mitigation.md`.

## D. Implementation
1. Add `licensed_state_codes`, `notice_type`, and `regulator_ack_ref` examples.
2. Gate sends through tenant and state policy before provider selection.
3. Prefer SES or Mailgun only where the tenant pack allows SaaS delivery.
4. Route sovereign or restricted states through Postal when required.
5. Emit delivery, bounce, and complaint events with audit-chain references.
6. Rehearse bounce storm mitigation for regulator-notice campaigns.

## E. Acceptance
- J91 examples cite real contracts, send policy, and bounce runbook.
- Suppression list checks run before any send.
- Provider choice is policy-driven and tenant-scoped.
- No payment threshold logic is assigned to comms-email.

## F. Evidence
- Journey: `docs/user-journeys/j91-us-state-money-transmitter-licensing/README.md`.
- Policy: `microservices/comms-email/policy/comms-email-send.cedar`.
- Dashboard: `microservices/comms-email/dashboards/deliverability.json`.

## G. Counterparts
| Counterpart | Gap closed |
|---|---|
| AWS SES | Adds tenant/state policy and audit-chain delivery evidence. |
| SendGrid / Twilio | Adds Cedar-scoped regulated notice routing. |
| Mailgun / Postal | Supports SaaS and self-hosted paths with explicit pack selection. |
