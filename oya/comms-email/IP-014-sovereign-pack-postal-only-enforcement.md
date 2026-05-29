# IP-014 — Sovereign pack: Postal-only enforcement

> ADR anchor: ADR-0201, ADR-0064, ADR-0173.
> Owner: `oya-substrate-comms`.
> Estimate: 2 days.

## Goal

For sovereign packs (KSA, UAE, and any future sovereign overlay),
force the `comms.email.provider` to `postal`. SES / Mailgun /
SMTP adapters are rejected at config-load time.

## Why this IP

Sovereign deployments cannot use commercial-SaaS providers.
Even an air-gap cluster could mis-configure `provider = ses`
and silently push traffic over the open internet. This IP
makes that impossible by raising the violation at
config-load, before the µservice serves any traffic.

## Pre-conditions

- IP-001..IP-004 adapters land.
- ADR-0064 packs structure exists.

## Tasks

### 1. Pack overlay schema

- Each pack overlay under
  `microservices/comms-email/iac/packs/<pack>/` declares
  `allowed_providers: [postal]` (sovereign) or
  `allowed_providers: [ses, postal, mailgun, smtp]` (default).

### 2. Config-load validator

- On µservice boot:
  - Load the active pack overlay.
  - Read the manifest's `comms.email.provider` value.
  - Reject + exit-non-zero if the value is not in
    `allowed_providers`.

### 3. Cargo lane check

- `oya-check-iac-tier-discipline` (this batch) gains a
  sub-check (or a sibling discipline check) that runs at CI
  asserting sovereign packs declare `allowed_providers:
  [postal]` only. Implementation: parent-wired as a follow-up.

### 4. Tests

- Unit test: KSA pack with `provider = ses` fails at boot.
- Unit test: KSA pack with `provider = postal` boots clean.
- Unit test: default pack with `provider = ses` boots clean.

## Failure modes

- Mis-configured sovereign cluster boots with
  `provider = ses`: caught at config-load; the µservice
  refuses to serve.

## Acceptance criteria

- KSA / UAE clusters cannot start with non-Postal provider.
- Default clusters retain full provider flexibility.

## Rollback

Parent removes the sovereign tag from the pack to allow
broader provider set; this is an explicit decision recorded
in the audit chain.

## References

- ADR-0201.
- ADR-0064 packs.
- ADR-0173 vendor lock-in avoidance.
