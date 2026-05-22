---
doc_class: Implementation-Plan
ip_id: IP-journey-j100-pack-rollout-first-action
journey_ref: docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action/
microservice: comms-email
status: draft
date: 2026-05-21
---

# IP-journey-j100: Pack rollout first email action

## A. Problem
J100 needs the first email action after tenant onboarding to prove from-domain readiness, DKIM alignment, suppression checks, provider route, webhook evidence, and audit-chain emission.

## B. Approach
Use `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`, DKIM and DMARC SLOs, and the send/openapi contract to gate the first production send.

## C. Deliverables
- First-send examples in `microservices/comms-email/contracts/openapi.yaml`.
- First delivery/webhook examples in `microservices/comms-email/contracts/asyncapi.yaml`.
- From-domain onboarding and DKIM runbook references.
- Send pipeline dashboard reference.

## D. Implementation
1. Add `tenant_onboarding_ref`, `from_domain_ref`, `dkim_key_ref`, and `first_action_id` examples.
2. Verify from-domain onboarding before the first send.
3. Run suppression-list and tenant action policy before provider routing.
4. Emit send, delivery, and webhook evidence.
5. Monitor `microservices/comms-email/dashboards/send-pipeline.json`.
6. Roll back by disabling the route and preserving evidence, not deleting events.

## E. Acceptance
- First action cannot complete without DKIM/from-domain evidence.
- Send pipeline and webhook evidence are cited.
- Suppression checks are mandatory.
- Provider route is tenant and pack scoped.

## F. Evidence
- Journey: `docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action/README.md`.
- Runbook: `microservices/comms-email/runbooks/per-tenant-from-domain-onboard.md`.
- Dashboard: `microservices/comms-email/dashboards/send-pipeline.json`.

## G. Counterparts
| Counterpart | Gap closed |
|---|---|
| AWS SES | Adds tenant first-action readiness around DKIM/from-domain setup. |
| SendGrid / Twilio | Adds suppression and webhook evidence before first send completion. |
| Mailgun / Postal | Supports alternate first-send routes under the same contract. |
