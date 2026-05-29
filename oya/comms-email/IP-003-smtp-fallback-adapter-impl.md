# IP-003 — SMTP fallback adapter implementation

> ADR anchor: ADR-0201.
> Crate: `crates/oya-shared-email-comms-kernel` (real `SmtpEmailComms` impl).
> Owner: `oya-substrate-comms`.
> Estimate: 3 days.

## Goal

Implement the generic RFC 5321 SMTP fallback adapter for the
`EmailComms` trait. This is the last-resort path used when no
other adapter is available: partner-managed MTAs, isolated lab
environments, or relay-only smarthosts.

## Why this IP

Adapter coverage matters. Even when SES, Postal, and Mailgun are
all unavailable in a given deployment topology, the substrate
must still emit transactional email. The SMTP adapter is also
the easiest path for emergency-onboarding a tenant that hasn't
yet provisioned a proper Postal or SES identity.

## Pre-conditions

- `crates/oya-shared-email-comms-kernel` lands.
- ADR-0201 ratified.

## Tasks

### 1. Wire the SMTP client

- Add the SMTP adapter sub-crate
  `crates/oya-shared-email-comms-kernel-adapter-smtp`.
- Depend on `lettre` (workspace-pinned; floor 0.11.x as of
  2026-05-18). `lettre` is the Rust-native SMTP client most
  widely adopted in the ecosystem; license is MIT/Apache-2.0.
- Use `rustls` for TLS, never `native-tls`.

### 2. Implement `EmailComms::send`

- Construct a `lettre::Message`:
  - `From`, `To`, `Reply-To`, `Subject`.
  - HTML + plain alternative bodies as MIME multipart/alternative.
- Open an SMTP connection per send (no pooling at adapter
  level; the comms-email µservice handles pooling at a higher
  layer).
- TLS: enforce STARTTLS where the peer advertises it; otherwise
  require TLS-on-connect (port 465). Plain-text SMTP (port 25
  unencrypted) is **rejected** at preflight.

### 3. DKIM signing

- The kernel preflight requires every send be DKIM signed.
  Because the SMTP adapter often talks to a downstream relay
  the substrate does not own, DKIM signing happens **inside**
  the adapter using the per-tenant key from OpenBao before the
  message is handed to the SMTP transport.
- Use `lettre`'s DKIM signing support when available; otherwise
  use the `mail-auth` Rust crate or an in-tree signing path.
  Implementation choice owed at IP-005 (DKIM rotation).

### 4. Idempotency

- SMTP does not have a native client-message-id. Emit the
  `Message-ID:` header with the ADR-0149 idempotency-key
  fingerprint so downstream MTAs deduplicate on retry.

### 5. Errors

- Map SMTP 4xx (transient) → retryable provider error.
- Map SMTP 5xx (permanent) → `ProviderError` non-retryable.
- Map TLS handshake failure → `ProviderError` with explicit
  `code = smtp.tls.handshake-failed`.

### 6. Observability

- OTEL spans `comms.email.smtp.send`.
- Prometheus counter `comms_email_smtp_sends_total` labeled
  `{result=ok|tls-fail|retryable|permanent}`.

### 7. Webhook delivery events

- SMTP does not emit webhooks. The adapter emits exactly one
  synchronous `Sent` event into the audit chain on successful
  RCPT-TO ack; subsequent delivery status is **best-effort
  unknown** (DSNs may arrive at the configured Reply-To inbox,
  which is out of scope for Phase 1).

### 8. Tests

- Unit tests for the message builder.
- Integration test against a local `Mailpit` test SMTP
  container (in CI).
- TLS-rejection test that asserts plaintext-port-25 send is
  rejected at preflight.

## Failure modes

- Downstream relay refuses STARTTLS: adapter rejects send;
  surface a `ProviderError` with explicit operator guidance.
- DKIM signing key not present: preflight rejects.
- Connection timeout: tenant retries via ADR-0149 idempotency
  key; the higher-level µservice queue handles back-off.

## Acceptance criteria

- `cargo test -p oya-shared-email-comms-kernel-adapter-smtp`
  passes.
- A test send through `Mailpit` arrives DKIM-signed with the
  correct Message-ID.
- TLS-rejection assertion passes.

## Rollback

The SMTP adapter is rollback-by-feature-flag: parent flips
`comms.email.provider` away from `smtp` for affected tenants.

## References

- ADR-0201.
- RFC 5321 (SMTP).
- RFC 5322 (Internet Message Format).
- RFC 6376 (DKIM signatures).
- `lettre` upstream documentation.
