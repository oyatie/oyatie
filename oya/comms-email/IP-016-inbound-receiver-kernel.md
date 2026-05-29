---
ip_id: IP-016
microservice: comms-email
bounded_context: inbound-receiving
layer: kernel
related_adrs: [ADR-0201, ADR-0263]
---

# IP-016 — inbound receiver verification kernel

## Goal

Build the deterministic MX-message verification kernel for inbound mail. This IP is not a
generic "inbound parse" shell: it owns the local SPF, DKIM, DMARC, and ARC verdict bundle
that the comms-email domain layer consumes before quarantine or delivery decisions.

## Service anchors

- Capability: `microservices/comms-email/capabilities/T3-inbound-receive.json` declares
  `inbound_receive` and `inbound_quarantine_release` actions plus audit classes
  `oya.comms-email.inbound-received` and `oya.comms-email.inbound-quarantined`.
- Policy: `microservices/comms-email/policy/abuse-defence.cedar` forbids
  `Action::"inbound_receive"` when SPF, DKIM, and DMARC all fail.
- Runbook counterpart: `microservices/comms-email/runbooks/inbound-receiver-quarantine-release.md`
  is the operator path after this kernel marks a message suspicious.

## Kernel contract

Input is a raw RFC 5322 message plus envelope metadata: tenant id, receiving region,
SMTP peer IP, `MAIL FROM`, `RCPT TO`, and received timestamp. Output is an immutable
`InboundVerificationVerdict` carrying:

- SPF result and authenticated domain.
- DKIM selector/domain result set, including multiple signatures.
- DMARC disposition: none, quarantine, reject.
- ARC chain result.
- Canonical reason code for deny or quarantine.

The kernel does not persist, emit audit-chain events, call Intelligence, or release
quarantined mail. Those are IP-017 responsibilities.

## Counterpart refs

- IP-017 consumes the verdict and decides persistence, quarantine, and audit emission.
- IP-023 exposes quarantine release and received-message retrieval once OpenAPI is
  extended in `microservices/comms-email/contracts/openapi.yaml`.
- IP-020 uses aggregate inbound-authentication failure rates as one reputation signal.

## Acceptance

- Property tests cover DMARC reject/quarantine/pass paths and the all-auth-failed Cedar
  condition represented in `policy/abuse-defence.cedar`.
- `T3-inbound-receive.json` action names remain unchanged or the IP is updated in the
  same changeset.
- The kernel emits no side effects; all audit emission remains in IP-017/IP-026 surfaces.
- No route is claimed until it exists in `contracts/openapi.yaml`.
