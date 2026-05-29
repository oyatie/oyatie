---
doc_class: Implementation-Plan
ip_id: IP-journey-j96-ksa-uae-mena-onboarding
journey_ref: docs/user-journeys/j96-ksa-uae-mena-tenant-onboarding/
microservice: comms-email
status: draft
date: 2026-05-21
---

# IP-journey-j96: KSA/UAE MENA email onboarding

## A. Problem
J96 needs tenant onboarding notices for KSA/UAE where provider routing, templates, DKIM, and webhooks respect sovereign-pack decisions.

## B. Approach
Use `microservices/comms-email/policy/data-residency.cedar`, Postal failover via `microservices/comms-email/runbooks/postal-failover.md`, and onboarding timing through `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`.

## C. Deliverables
- MENA tenant onboarding examples in `microservices/comms-email/contracts/openapi.yaml`.
- Provider-route events in `microservices/comms-email/contracts/asyncapi.yaml`.
- Residency, DKIM, and Postal failover evidence.
- From-domain onboarding SLO references.

## D. Implementation
1. Add `mena_pack_ref`, `jurisdiction_code`, and `from_domain_ref` examples.
2. Validate residency before SaaS provider routing.
3. Use Postal for restricted sovereign-pack cases.
4. Rotate or verify DKIM through the existing DKIM runbook.
5. Emit onboarding and webhook delivery events.
6. Fail closed when from-domain onboarding proof is absent.

## E. Acceptance
- MENA examples cite contracts, residency policy, Postal failover, and SLO.
- Provider choice is pack-driven.
- DKIM and from-domain evidence are required before production sends.
- No tenant secret appears in contract examples.

## F. Evidence
- Journey: `docs/user-journeys/j96-ksa-uae-mena-tenant-onboarding/README.md`.
- Runbook: `microservices/comms-email/runbooks/postal-failover.md`.
- SLO: `microservices/comms-email/slos/from-domain-onboarding-time.openslo.yaml`.

## G. Counterparts
| Counterpart | Gap closed |
|---|---|
| AWS SES | Adds MENA pack routing and from-domain proof. |
| Postal | Provides self-hosted sovereign delivery path. |
| Mailgun / SendGrid | Remain allowed only when residency policy permits. |
