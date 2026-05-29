# IP-005 — DKIM key rotation pipeline

> ADR anchor: ADR-0201, ADR-0173.
> Owner: `oya-substrate-comms`.
> Estimate: 4 days.

## Goal

Implement the per-tenant DKIM key generation, rotation, and
revocation pipeline. Every tenant from-domain has a DKIM signing
key in OpenBao. Rotation is annual by default and on-revocation
on demand. Selectors cycle deterministically so DNS records can
be pre-published.

## Why this IP

DKIM signing is mandatory on every send (ADR-0201). The signing
key is the single most sensitive secret in the comms-email
substrate. Rotation discipline + key isolation prevent forged-
mail attacks against tenant from-domains. Without this IP, the
kernel's "DKIM enforced at preflight" guarantee has no
operational backing.

## Pre-conditions

- ADR-0173 OpenBao storage substrate exists.
- `crates/oya-shared-email-comms-kernel` lands (delivered this
  batch).
- IP-011 per-tenant from-domain onboarding lands.

## Tasks

### 1. Key generation

- Generate Ed25519 (RFC 8463) DKIM keys preferentially, with
  RSA-2048 as the fallback for receivers that do not yet
  support Ed25519 (still the majority as of 2026-05-18; both
  records are published in parallel).
- Use `ring` or `aws-lc-rs` for crypto primitives (workspace
  pin). No custom crypto.

### 2. Key storage

- Private keys live at OpenBao path
  `kv/data/oya/comms-email/dkim/{tenant_id}/{selector}/`.
- ACL: only the comms-email µservice service account can
  read; no human ever sees the private key material.

### 3. Selector cycling

- Selectors follow the deterministic schedule
  `oya{YYYYMM}` (e.g. `oya202605`). On rotation, the new
  selector is generated 14 days in advance, both selectors are
  active in parallel for the overlap window, and DNS records
  for both are published.

### 4. DNS publication

- Tier-B OpenTofu DNS module (ADR-0202
  `microservices/cloud-iac/tofu/modules/dns/`) consumes the
  canonical DKIM record string per tenant and publishes the
  TXT record under the selector subdomain.
- Publication is idempotent; the comms-email µservice emits
  a `dkim.dns.publish.requested` event into the audit chain
  and the IaC pipeline picks it up.

### 5. Provider key sync

- For SES: `aws-sdk-sesv2 PutEmailIdentityDkimAttributes`
  with the new key per-domain.
- For Postal: Postal API per-domain DKIM update.
- For Mailgun: Mailgun per-domain DKIM key upload.
- For SMTP: no provider sync needed; signing happens in the
  adapter.

### 6. Overlap window

- Both old + new selectors are valid for 14 days.
- All new sends use the new selector starting at T+0.
- The old selector's DNS record is removed at T+14d.

### 7. Revocation

- On compromise: immediate selector retirement; new selector
  generated and published; old DNS record removed within ≤ 5
  minutes; suppression list flush for any in-flight retries
  using the compromised key.

### 8. Tests

- Unit tests for selector cycling logic.
- Integration test that generates a key, publishes the DNS
  record (against a test DNS provider), and verifies that a
  test send signs with the new selector.
- Revocation drill test asserting old selector is removed
  within SLA.

## Failure modes

- OpenBao unavailable: comms-email µservice rejects every
  send at preflight with `DkimBindingMissing`. Runbook
  `dkim-key-rotation.md` covers the OpenBao failover.
- DNS propagation delay: overlap window covers this. If a
  receiver still rejects after 14 days, manual extension via
  the runbook.

## Acceptance criteria

- A tenant onboarded today rotates its DKIM key in T+12mo
  automatically, with zero deliverability degradation across
  the overlap window.
- Revocation drill removes a compromised selector in ≤ 5 min.
- No private key material ever appears in logs, in audit
  chain events (key reference only, not key material), or in
  any non-OpenBao storage.

## Rollback

If the rotation pipeline regresses, parent disables auto-rotation
and operations team performs manual rotation per runbook. The
runbook is the authority for manual operation.

## References

- ADR-0201.
- ADR-0173.
- RFC 6376 (DKIM signatures).
- RFC 8463 (Ed25519 DKIM).
- IP-011 per-tenant from-domain onboarding.
