---
doc_class: Implementation-Plan
ip_id: IP-journey-j95-iso27001-soc2-annual-audit
journey_ref: docs/user-journeys/j95-iso-27001-soc-2-annual-audit/
microservice: comms-email
status: draft
date: 2026-05-21
---

# IP-journey-j95: ISO 27001 / SOC 2 email evidence packet

## A. Problem
J95 requires an annual audit packet for email delivery controls: provider routing, DKIM rotation, suppression-list governance, webhook replay, deliverability SLOs, and audit-chain lag.

## B. Approach
Assemble the packet from service-local artifacts only: `microservices/comms-email/dashboards/deliverability.json`, `microservices/comms-email/dashboards/webhook-and-audit.json`, `microservices/comms-email/slos/dkim-signing-rate.openslo.yaml`, and `microservices/comms-email/policy/auditor-scope.cedar`.

## C. Deliverables
- Auditor read examples in OpenAPI.
- Evidence-export events in AsyncAPI.
- Links to DKIM, deliverability, suppression, webhook, and audit SLOs.
- Auditor-scope authorization text.

## D. Implementation
1. Add `audit_window`, `control_family`, and `evidence_bundle_ref` examples.
2. Include provider routing, DKIM, suppression, webhook, and audit-lag evidence.
3. Gate reads through auditor-scope Cedar.
4. Exclude message bodies and recipient secrets from audit bundles.
5. Replay webhook evidence from `microservices/comms-email/runbooks/webhook-replay.md`.
6. Document gaps against `microservices/comms-email/feature-parity-matrix-2026-05-20.md`.

## E. Acceptance
- Audit packet examples cite dashboards, SLOs, policy, and runbooks.
- Auditor reads are read-only and tenant-scoped.
- Evidence can be rebuilt from delivery/webhook events.
- No provider dashboard screenshots are required as source of truth.

## F. Evidence
- Journey: `docs/user-journeys/j95-iso-27001-soc-2-annual-audit/README.md`.
- Policy: `microservices/comms-email/policy/auditor-scope.cedar`.
- Matrix: `microservices/comms-email/feature-parity-matrix-2026-05-20.md`.

## G. Counterparts
| Counterpart | Gap closed |
|---|---|
| SendGrid / Twilio | Adds tenant-scoped annual evidence bundles. |
| AWS SES | Adds audit packet shape beyond service metrics. |
| Postal | Supplies self-hosted evidence for restricted tenants. |
