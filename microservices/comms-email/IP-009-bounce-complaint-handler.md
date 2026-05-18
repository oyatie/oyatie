# IP-009 — Bounce / complaint handler

> ADR anchor: ADR-0201, ADR-0145.
> Owner: `oya-substrate-comms`.
> Estimate: 3 days.

## Goal

React to `Bounced` and `Complained` events from the webhook
pipeline (IP-008) by classifying hard vs soft, escalating
critical patterns, and adding hard-bounce / complaint addresses
to the suppression list (IP-010).

## Why this IP

Continuing to send to addresses that bounce or complain is the
fastest path to a domain being blocklisted and to a tenant's
reputation eroding across the receiver matrix. Industry standard
is to suppress immediately on hard bounce + on complaint.

## Pre-conditions

- IP-008 webhook delivery pipeline lands.
- IP-010 suppression list lands.

## Tasks

### 1. Classification

- Hard bounce signals (any one):
  - SES: `bounceType=Permanent`
  - Postal: `MessageBounced` with `severity=hard`
  - Mailgun: `failed` with `severity=permanent`
  - SMTP: 5xx permanent response code (5.1.1, 5.1.10, etc.)
- Soft bounce: temporary failure; the µservice retries via
  ADR-0149 idempotency-key for ≤ 3 attempts over ≤ 24h before
  declaring soft-bounce-as-hard.
- Complaint: any provider event tagged `complained`.

### 2. Suppression handoff

- On hard bounce: insert into suppression list with
  `reason = HardBounce` and `provider = <provider>`.
- On complaint: insert with `reason = Complained`. Complaint
  is **never** removed automatically — only by operator action
  via runbook `dmarc-policy-tune.md`.

### 3. Escalation

- If hard-bounce rate exceeds 5% over a 1h window per tenant
  per pack, emit `comms.email.bouncestorm.detected` event and
  page on-call. Runbook `bounce-storm-mitigation.md` covers
  the response.
- If complaint rate exceeds 0.1% over a 1h window, throttle
  the tenant's send rate to 25% and page on-call.

### 4. Audit chain emission

- Every bounce + complaint event becomes an ADR-0145 chain
  entry with full context (tenant_id, recipient, reason,
  provider_message_id).

### 5. Tests

- Unit tests for each provider's classification path.
- Integration test that simulates a bounce storm and asserts
  the on-call alert fires.
- Soft-bounce → retry → hard-bounce promotion test.

## Failure modes

- Misclassification (false hard-bounce → permanent suppression
  of valid recipient): operator removes from suppression via
  runbook; emit an audit chain entry for the removal.
- Storm escalation deadlock (alert fires repeatedly on the
  same root cause): runbook freezes the tenant's send rate
  + opens an incident.

## Acceptance criteria

- 100% of hard bounces land in suppression within 5 s of
  provider webhook arrival.
- Bounce-storm detection fires within 60 s of crossing
  threshold.
- No silent retries of suppressed addresses.

## Rollback

Parent disables auto-suppression for the affected tenant; on-call
performs manual suppression curation.

## References

- ADR-0201.
- ADR-0145 audit chain.
- IP-008 webhook delivery pipeline.
- IP-010 suppression list.
