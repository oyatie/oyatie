# IP-008 — Webhook delivery pipeline

> ADR anchor: ADR-0201, ADR-0145, ADR-0166.
> Owner: `oya-substrate-comms`.
> Estimate: 5 days.

## Goal

Receive provider webhooks (SES, Postal, Mailgun) and normalize
them into canonical `DeliveryEvent`s that flow into the
ADR-0145 audit chain on a ADR-0166 schema-versioned shape.

## Why this IP

Delivery telemetry (sent, delivered, opened, clicked, bounced,
complained, suppressed) is the foundation of the comms-email
substrate's value proposition. Without normalized webhook
ingest, callers can send mail but cannot prove (or learn) what
happened to it. Auditors cannot prove regulatory disclosure
delivered. Suppression-list maintenance (IP-010) cannot react.

## Pre-conditions

- ADR-0145 audit chain available.
- ADR-0166 schema registry available.
- Adapters IP-001 / IP-002 / IP-004 land.

## Tasks

### 1. Webhook ingest surface

- HTTPS-only ingest at `https://comms-email.<region>/v1/webhooks/{provider}`.
- TLS 1.3, mTLS optional (provider-specific).
- Authentication:
  - SES: SNS signature verification.
  - Postal: per-tenant HMAC shared secret from OpenBao.
  - Mailgun: HMAC-SHA256 signature header verification.

### 2. Schema versioning

- Each `DeliveryEvent` emitted into the audit chain carries an
  ADR-0166 schema id, e.g.
  `oya/comms-email/delivery-event/v1`.
- v1 is the schema for this IP. Future schema versions go
  through ADR-0166 versioning + sunset.

### 3. Normalization rules

- Provider event → `DeliveryEventKind` mappings live in each
  adapter IP (IP-001, IP-002, IP-004).
- IP-003 SMTP emits only `Sent` (no downstream visibility).

### 4. Idempotency

- Each provider webhook fingerprint is recorded; duplicates
  collapse. Replay attacks (provider re-delivers the same
  event) are absorbed cleanly.

### 5. At-least-once + retry

- The audit-chain emission path is retried with exponential
  back-off (max 8 retries, 1s → 256s) on transient failure.
- After max retries the event lands in a dead-letter store
  (`comms-email-webhook-dlq` Postgres table) for manual
  intervention.

### 6. Bounce / complaint handoff

- Hard bounces and complaints feed IP-009 (bounce handler) +
  IP-010 (suppression list).

### 7. Tests

- Unit tests for signature verification per provider.
- Unit tests for the v1 schema serialization.
- Integration test: provider-fixture → normalized event in
  audit chain.
- Replay test: same event delivered twice produces one
  audit-chain entry.

## Failure modes

- Audit chain unavailable: events land in the DLQ. SLO
  `webhook-success rate` widens. Runbook `webhook-replay.md`.
- Signature verification fails: drop + emit
  `comms.email.webhook.signature.failed` Prometheus counter.
  Persistent failure → page on-call.

## Acceptance criteria

- All four adapters route normalized events through this
  pipeline.
- p99 event arrival in audit chain ≤ 5 s end-to-end.
- DLQ depth alert fires at > 100 events in 5 min.
- Schema v1 is registered in the ADR-0166 registry.

## Rollback

Parent disables the affected provider's webhook ingest.
Downstream telemetry degrades; sends continue.

## References

- ADR-0201, ADR-0145, ADR-0166.
- IP-001, IP-002, IP-004 adapter implementations.
- IP-009 bounce handler, IP-010 suppression list.
