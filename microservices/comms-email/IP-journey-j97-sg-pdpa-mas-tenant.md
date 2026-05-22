---
doc_class: Implementation-Plan
ip_id: IP-journey-j97-sg-pdpa-mas-tenant
journey_ref: docs/user-journeys/j97-sg-pdpa-mas-singapore-tenant/
microservice: comms-email
status: draft
date: 2026-05-21
---

# IP-journey-j97: Singapore PDPA / MAS email tenant

## A. Problem
J97 needs Singapore tenant email delivery with consent-aware templates, MAS incident-notice readiness, bounce handling, and audit evidence.

## B. Approach
Use list-management and suppression capabilities from `microservices/comms-email/capabilities/T2-list-manage.json`, inbound/bounce evidence from `microservices/comms-email/capabilities/T1-bounce-handle.json`, and policy from `microservices/comms-email/policy/comms-email-suppression-list.cedar`.

## C. Deliverables
- Singapore notice examples in OpenAPI.
- Delivery, bounce, and complaint examples in AsyncAPI.
- Suppression-list and reputation dashboard references.
- Bounce storm and reputation-drop runbook references.

## D. Implementation
1. Add `sg_pdpa_pack_ref`, `consent_basis_ref`, and `mas_notice_type` examples.
2. Require suppression and unsubscribe checks before send.
3. Emit bounce/complaint events with evidence hashes.
4. Monitor `microservices/comms-email/dashboards/reputation-monitoring.json`.
5. Exercise bounce-storm and reputation-drop runbooks.
6. Deny sends without consent basis or tenant policy.

## E. Acceptance
- Singapore examples cite list, bounce, suppression, and reputation artifacts.
- Complaint/bounce events are auditable.
- MAS incident notices use approved templates.
- Provider routing remains tenant-scoped.

## F. Evidence
- Journey: `docs/user-journeys/j97-sg-pdpa-mas-singapore-tenant/README.md`.
- Policy: `microservices/comms-email/policy/comms-email-suppression-list.cedar`.
- Dashboard: `microservices/comms-email/dashboards/reputation-monitoring.json`.

## G. Counterparts
| Counterpart | Gap closed |
|---|---|
| SendGrid / Twilio | Adds consent and suppression gates before delivery. |
| AWS SES | Adds bounce/complaint evidence tied to tenant policy. |
| Mailgun / Postal | Supplies provider alternatives with the same evidence contract. |
