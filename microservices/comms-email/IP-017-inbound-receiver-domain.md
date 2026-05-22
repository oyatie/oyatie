---
ip_id: IP-017
microservice: comms-email
bounded_context: inbound-receiving
layer: domain
related_adrs: [ADR-0201, ADR-0263]
---

# IP-017 — inbound receiver domain and quarantine

## Goal

Turn the IP-016 verification verdict into tenant-scoped inbound-message state, quarantine
decisions, audit evidence, and operator recovery flow. This is the service-local domain
slice that keeps untrusted inbound mail from becoming tenant inbox data by default.

## Service anchors

- Capability: `microservices/comms-email/capabilities/T3-inbound-receive.json` names the
  inbound receive and quarantine-release actions and the two audit classes this domain owns.
- Policy: `microservices/comms-email/policy/action-authorization.cedar` allows the
  SPIFFE-attested `oyatie::CommsEmail::Principal::"inbound-receiver"` service principal.
- Abuse gate: `microservices/comms-email/policy/abuse-defence.cedar` denies inbound receive
  when SPF, DKIM, and DMARC all fail.
- Operator flow: `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`
  defines the release command, false-positive evidence path, and rollback command.

## Domain behavior

1. Consume the IP-016 verdict and raw message metadata.
2. Persist only the tenant-visible envelope and quarantine pointer until policy allows
   release; do not expose raw content to tenant APIs while quarantined.
3. Emit `oya.comms-email.inbound-received` for accepted messages and
   `oya.comms-email.inbound-quarantined` for phishing or authentication-failed mail.
4. Require a Cedar-allowed quarantine release before tenant delivery.
5. Preserve evidence needed by the runbook: SPF/DKIM/DMARC status, classifier reason, and
   tenant id.

## Counterpart refs

- IP-016 produces the verification verdict and must remain side-effect free.
- IP-023 owns the REST exposure for message pagination and quarantine release.
- IP-020 reads quarantine rates and authentication-failure trends as reputation inputs.

## Acceptance

- Quarantined messages cannot be delivered until a release command follows the runbook.
- Audit classes match `T3-inbound-receive.json`.
- Domain authorization uses existing `policy/action-authorization.cedar` principals only.
- OpenAPI route claims remain deferred to IP-023 and `contracts/openapi.yaml`.
