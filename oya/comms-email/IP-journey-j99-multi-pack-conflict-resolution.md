---
doc_class: Implementation-Plan
ip_id: IP-journey-j99-multi-pack-conflict-resolution
journey_ref: docs/user-journeys/j99-cross-jurisdiction-multi-pack-conflict-resolution/
microservice: comms-email
status: draft
date: 2026-05-21
---

# IP-journey-j99: Multi-pack email routing conflict resolution

## A. Problem
J99 creates conflicts between residency, suppression, consent, provider, and audit requirements. comms-email must choose the stricter pack before rendering templates or selecting a provider.

## B. Approach
Use pack overlay authorization in `microservices/comms-email/policy/pack-overlay-authorization.cedar`, residency policy, and suppression-list policy. Provider routing is documented in contracts but enforced by policy before dispatch.

## C. Deliverables
- Multi-pack conflict examples in OpenAPI.
- Conflict-denied and conflict-resolved events in AsyncAPI.
- Policy references for pack overlay, residency, and suppression.
- Webhook replay and reputation runbook references.

## D. Implementation
1. Add `active_pack_refs`, `winning_pack_ref`, and `provider_route_ref` examples.
2. Deny sends when packs imply incompatible residency/provider constraints.
3. Apply the stricter suppression and consent rule before rendering.
4. Emit conflict resolution evidence.
5. Replay failed webhooks after pack conflict resolution.
6. Monitor reputation when a conflict forces provider migration.

## E. Acceptance
- Conflict examples cite real policy files.
- Missing `winning_pack_ref` fails closed.
- Provider migration keeps audit evidence and suppression state.
- No generic invariant rows remain.

## F. Evidence
- Journey: `docs/user-journeys/j99-cross-jurisdiction-multi-pack-conflict-resolution/README.md`.
- Policy: `microservices/comms-email/policy/pack-overlay-authorization.cedar`.
- Runbook: `microservices/comms-email/runbooks/webhook-replay.md`.

## G. Counterparts
| Counterpart | Gap closed |
|---|---|
| SendGrid / Twilio | Adds explicit multi-pack conflict handling before send. |
| AWS SES | Adds residency/provider conflict evidence. |
| Postal | Supplies stricter self-hosted route when SaaS providers are denied. |
