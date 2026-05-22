---
doc_class: Implementation-Plan
ip_id: IP-journey-j94-sox404-public-company-controls
journey_ref: docs/user-journeys/j94-sox-404-public-company-controls/
microservice: comms-email
status: draft
date: 2026-05-21
---

# IP-journey-j94: SOX 404 control notification evidence

## A. Problem
J94 needs evidence that financial-control notifications were sent from approved templates, with DKIM, provider choice, webhook receipt, and audit-chain timing preserved.

## B. Approach
Bind control notifications to `microservices/comms-email/contracts/openapi.yaml`, DKIM operations in `microservices/comms-email/runbooks/dkim-key-rotation.md`, and audit lag SLO `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`.

## C. Deliverables
- SOX control-notice examples.
- Template-rendering and DKIM evidence references.
- Webhook and audit dashboard references.
- Negative acceptance for unapproved templates.

## D. Implementation
1. Add `control_id`, `change_ticket_ref`, and `approved_template_ref` examples.
2. Require template version and DKIM signing before dispatch.
3. Emit delivery and webhook events to AsyncAPI.
4. Track audit lag in `microservices/comms-email/dashboards/webhook-and-audit.json`.
5. Rotate DKIM keys through the runbook without losing control evidence.
6. Reject sends with unapproved templates.

## E. Acceptance
- SOX examples cite OpenAPI, AsyncAPI, DKIM runbook, and audit SLO.
- Unapproved templates fail closed.
- Webhook delivery evidence is replayable.
- Control notification evidence does not claim financial-control ownership.

## F. Evidence
- Journey: `docs/user-journeys/j94-sox-404-public-company-controls/README.md`.
- Runbook: `microservices/comms-email/runbooks/dkim-key-rotation.md`.
- SLO: `microservices/comms-email/slos/audit-chain-emit-lag-p99.openslo.yaml`.

## G. Counterparts
| Counterpart | Gap closed |
|---|---|
| AWS SES | Adds SOX template and audit-lag evidence. |
| SendGrid / Twilio | Adds Cedar-gated approved-template control. |
| Mailgun / Postal | Provides alternate provider path with DKIM evidence. |
