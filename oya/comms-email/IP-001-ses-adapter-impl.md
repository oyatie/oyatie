# IP-001 — SES adapter implementation

> ADR anchor: ADR-0201, ADR-0173.
> Crate: `crates/oya-shared-email-comms-kernel` (real `SesEmailComms` impl).
> Owner: `oya-substrate-comms`.
> Estimate: 5 days.

## Goal

Implement the AWS SES adapter behind the `EmailComms` trait in
`oya-shared-email-comms-kernel`. The adapter sits behind the
canonical preflight (DKIM / SPF / DMARC / suppression / rate
ceiling) so its surface area is small: send a message and emit a
provider message id.

## Why this IP

SES is the default provider for cloud-hosted clusters (ADR-0201).
Every existing one-off SES integration across six oyatie
µservices migrates onto this adapter. The migration removes six
bespoke SES clients in favor of one.

## Pre-conditions

- `crates/oya-shared-email-comms-kernel` lands (delivered this
  batch).
- ADR-0201 ratified (delivered this batch).
- OpenBao mount path for DKIM keys exists per ADR-0173.

## Tasks

### 1. Wire the SDK

- Add the SES adapter sub-crate
  `crates/oya-shared-email-comms-kernel-adapter-ses` (parent
  wires the workspace member; this IP defines the contents).
- Depend on `aws-sdk-sesv2` behind the workspace pin. Live-source
  verification of the latest stable version is owed at wire time;
  the floor is the 1.x line as of 2026-05-18.

### 2. Implement `EmailComms::send`

- Translate the `OutboundMessage` into a SESv2 `SendEmail`
  request:
  - `FromEmailAddress` from `OutboundMessage::from`.
  - `Destination::ToAddresses` from `OutboundMessage::to`.
  - `Content::Simple::Subject::Data` from `subject`.
  - `Content::Simple::Body::Html::Data` from `html_body`.
  - `Content::Simple::Body::Text::Data` from `plain_body` (if
    present).
  - `ReplyToAddresses` from `reply_to` (if present).
  - `EmailTags`: `{tenant_id, locale, idempotency_key,
    audit_correlation_id}`.

### 3. DKIM identity binding

- Use the SES per-domain "EmailIdentity" with a DKIM signing
  attribute. The per-tenant DKIM key is published into SES via
  the SES Configuration Set + DKIM signing attribute path.
- The OpenBao-stored DKIM private key is the source of truth.
  The SES side imports it via `aws-sdk-sesv2` PutEmailIdentityDkimAttributes.

### 4. Configuration sets

- Create a single configuration set per tenant pack named
  `oya-tenant-{tenant_id}` that:
  - Forwards delivery events (bounce, complaint, delivery,
    open, click) to an SNS topic that the comms-email µservice
    consumes.
  - Applies the IP pool assigned to the tenant.

### 5. Idempotency

- ADR-0149 idempotency key is propagated into the `EmailTags`
  collection and the comms-email µservice's idempotency store.
- Duplicate sends with identical fingerprint return the cached
  message id; conflicts return `IdempotencyConflict`.

### 6. Errors

- Map SESv2 errors to `EmailCommsError::ProviderError { ... }`.
- Specifically map the SES "MessageRejected" → never silent;
  surface to the caller.
- Map the SES throttling error → `RateCeilingExceeded` and
  emit a `back-off` hint.

### 7. Observability

- Emit OTEL spans `comms.email.ses.send` carrying
  `tenant_id`, `idempotency_key`, `dkim_selector`,
  `from_domain`, and the resulting `message_id` once the SDK
  responds.
- Emit a Prometheus counter `comms_email_ses_sends_total`
  labeled `{result=ok|rejected|throttled|error}`.

### 8. Tests

- Unit tests for the SESv2 request builder (no SDK call;
  asserts on the constructed request shape).
- Integration test against `localstack` that runs in CI.
- Conformance test that runs against a real SES sandbox
  account (opt-in CI lane; secrets via OpenBao).

## Failure modes

- SES quota exhaustion: handled by Mailgun second-source +
  Postal sovereign fallback. See `runbooks/ses-failover.md`.
- DKIM identity not verified: preflight blocks; runbook
  `dkim-key-rotation.md` covers re-verification.
- Configuration set drift: the comms-email µservice
  reconciles configuration sets every five minutes and emits
  a drift event into the audit chain.

## Acceptance criteria

- `cargo test -p oya-shared-email-comms-kernel-adapter-ses`
  passes (unit + localstack integration).
- A test send against a real SES sandbox account succeeds with
  DKIM-signed delivery and the recipient receives the message.
- Webhook event for `delivered` arrives in the audit chain
  within p99 ≤ 5 s end-to-end.

## Rollback

If the adapter introduces a regression, parent wiring flips
`comms.email.provider` from `ses` to `postal` per tenant pack.
Tenants in sovereign packs are not affected (they always run on
Postal). Cloud-hosted tenants experience a brief deliverability
gap of ≤ 5 minutes (the configuration cache TTL).

## References

- ADR-0201 (this batch).
- ADR-0149 idempotency keys.
- ADR-0173 vendor lock-in avoidance.
- AWS SESv2 API documentation (Anthropic Context7 if needed
  during implementation).
