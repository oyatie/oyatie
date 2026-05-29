# IP-011 — Per-tenant from-domain onboarding

> ADR anchor: ADR-0201, ADR-0064.
> Owner: `oya-substrate-comms`.
> Estimate: 4 days.

## Goal

Onboard a new tenant's from-domain end-to-end: DNS records
(SPF, DKIM, DMARC), provider identity bindings, OpenBao key
material, suppression-list scope, audit-chain provenance.

## Why this IP

Every new tenant — and every new from-domain for an existing
tenant — needs the four deliverability primitives (DKIM, SPF,
DMARC, MX/Postmaster) plus provider identity bindings. Without
this IP, onboarding is a manual ticket-driven process that
takes days.

## Pre-conditions

- IP-005 DKIM key rotation pipeline lands.
- ADR-0064 pack overlay structure exists.
- ADR-0202 OpenTofu DNS module exists.

## Tasks

### 1. Onboarding API

- REST endpoint `POST /v1/tenants/{tenant_id}/from-domains`:
  - Body: `{from_domain, locale_pack, support_inbox, provider_preference}`.
  - Returns: `{onboarding_id, dns_records_to_publish, status}`.

### 2. DKIM generation

- Trigger IP-005 to generate a DKIM key pair with the canonical
  selector for the current period (e.g. `oya202605`).
- Private key into OpenBao at canonical path.
- Public key returned in `dns_records_to_publish`.

### 3. DNS record emission

- Generate canonical record strings:
  - SPF: `v=spf1 include:{provider-include}.{region} -all`
  - DKIM TXT (at `<selector>._domainkey.{from_domain}`):
    canonical Ed25519 + RSA-2048 records.
  - DMARC TXT (at `_dmarc.{from_domain}`):
    `v=DMARC1; p=quarantine; rua=mailto:dmarc-reports@oya.io`.
- Emit a `dns.publish.requested` audit chain event the
  OpenTofu DNS module consumes.

### 4. Provider identity binding

- For SES: call `aws-sdk-sesv2 CreateEmailIdentity` +
  `PutEmailIdentityDkimAttributes`.
- For Postal: provision the per-domain identity via Postal
  API.
- For Mailgun: provision the per-domain via Mailgun API.
- For SMTP: no provider-side binding; the adapter handles
  signing locally.

### 5. Status machine

- States: `requested` → `dns-pending` → `dns-published` →
  `provider-binding` → `provider-bound` → `warm-up` →
  `active`.
- `warm-up`: 14 days of low-rate sends (≤ 50/min) to build
  IP / domain reputation. During warm-up, DMARC default is
  `p=quarantine`; post-warm-up the operator can flip to
  `p=reject`.

### 6. Audit chain

- Every state transition emits an ADR-0145 audit chain entry
  with state-machine context.

### 7. Tests

- Unit tests for the state machine.
- Integration test: onboard a test domain end-to-end through
  to `active` state in CI (using a test DNS provider + SES
  sandbox).

## Failure modes

- DNS publication failure: state stays at `dns-pending`;
  retry every 5 min for up to 24h; then alert.
- Provider identity binding failure: state stays at
  `provider-binding`; runbook
  `per-tenant-from-domain-onboard.md` covers manual
  recovery.

## Acceptance criteria

- New tenant from-domain reaches `active` in ≤ 24h end-to-end
  for SES + Postal providers.
- DKIM signing verified by an external reflector
  (`mail-tester.com` or equivalent) returning a 10/10 score.
- DMARC reports flow into the audit chain within 48h of first
  send.

## Rollback

Parent disables the API. Manual operator-driven onboarding
per runbook.

## References

- ADR-0201, ADR-0064.
- IP-005 DKIM key rotation pipeline.
- ADR-0202 OpenTofu DNS module.
