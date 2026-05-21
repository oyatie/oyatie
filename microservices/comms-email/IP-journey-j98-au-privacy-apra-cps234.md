---
doc_class: Implementation-Plan
ip_id: IP-journey-j98-au-privacy-apra-cps234
journey_ref: docs/user-journeys/j98-au-privacy-apra-cps-234-tenant/
microservice: comms-email
status: draft
date: 2026-05-21
---

# IP-journey-j98: AU Privacy / APRA CPS 234 email controls

## A. Problem
J98 needs comms-email to support APRA incident and customer notices with provider resilience, DKIM continuity, suppression safety, and webhook evidence.

## B. Approach
Use provider-failover runbooks `microservices/comms-email/runbooks/ses-failover.md` and `microservices/comms-email/runbooks/postal-failover.md`, delivery SLOs, and action authorization policy.

## C. Deliverables
- APRA notice examples in OpenAPI.
- Provider failover events in AsyncAPI.
- DKIM, delivery, and webhook SLO references.
- Incident notification runbook references.

## D. Implementation
1. Add `apra_cps234_profile`, `incident_notice_ref`, and `failover_policy_ref` examples.
2. Gate send actions with `microservices/comms-email/policy/action-authorization.cedar`.
3. Verify DKIM and DMARC before provider failover.
4. Emit provider failover and webhook evidence events.
5. Monitor send success and webhook success SLOs.
6. Rehearse SES-to-Postal failover without losing audit evidence.

## E. Acceptance
- APRA examples cite action policy, failover runbooks, and SLOs.
- Provider failover preserves DKIM and audit evidence.
- Suppression-list checks still run during incident mode.
- comms-email does not own APRA business-risk scoring.

## F. Evidence
- Journey: `docs/user-journeys/j98-au-privacy-apra-cps-234-tenant/README.md`.
- Runbook: `microservices/comms-email/runbooks/ses-failover.md`.
- SLO: `microservices/comms-email/slos/webhook-success-rate.openslo.yaml`.

## G. Counterparts
| Counterpart | Gap closed |
|---|---|
| AWS SES | Adds auditable provider failover around SES sends. |
| Postal | Provides self-hosted recovery route for APRA incident mode. |
| SendGrid / Mailgun | Remain comparable providers behind the same policy gate. |
