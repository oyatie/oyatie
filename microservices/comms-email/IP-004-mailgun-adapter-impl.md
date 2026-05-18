# IP-004 — Mailgun adapter implementation

> ADR anchor: ADR-0201, ADR-0173.
> Crate: `crates/oya-shared-email-comms-kernel` (real `MailgunEmailComms` impl).
> Owner: `oya-substrate-comms`.
> Estimate: 3 days.

## Goal

Implement the Mailgun adapter behind the `EmailComms` trait.
Mailgun is the alternative-SaaS second-source per ADR-0201 — it
ensures SES is never the only commercial path so tenants can
swap without re-engineering.

## Why this IP

Vendor-lock-in avoidance per ADR-0173. Even when SES is the
default for AWS-hosted clusters, having a second commercial
adapter ready to go gives operators a fast switch when SES has
a regional outage or quota issue, and gives customers who
explicitly choose Mailgun a first-class path.

## Pre-conditions

- `crates/oya-shared-email-comms-kernel` lands.
- ADR-0201 ratified.

## Tasks

### 1. Wire the HTTP client

- Add the Mailgun adapter sub-crate
  `crates/oya-shared-email-comms-kernel-adapter-mailgun`.
- Depend on `reqwest` workspace-pinned, rustls TLS only.

### 2. Implement `EmailComms::send`

- POST `https://api.mailgun.net/v3/{domain}/messages` with:
  - `from`, `to`, `subject`, `html`, `text`, `h:Reply-To`.
  - `v:tenant_id`, `v:idempotency_key`,
    `v:audit_correlation_id` (Mailgun "variables" feature).
- Authenticate with the per-tenant Mailgun API key from
  OpenBao.

### 3. DKIM identity binding

- Mailgun signs DKIM on the receiving side using its own keys
  by default. To meet ADR-0201's "every send DKIM signed by
  the tenant's key" requirement, the adapter uploads the
  per-tenant DKIM private key to Mailgun's per-domain key
  store at tenant onboarding (IP-011), and Mailgun signs with
  the tenant-supplied key.

### 4. Webhook ingest

- Mailgun webhooks POST to the comms-email µservice ingest URL.
- Map Mailgun event names to canonical `DeliveryEventKind`:
  - `accepted` → `Sent`
  - `delivered` → `Delivered`
  - `failed` (`severity=permanent`) → `Bounced`
  - `failed` (`severity=temporary`) → soft bounce; routed back
    through retry logic.
  - `opened` → `Opened`
  - `clicked` → `Clicked`
  - `complained` → `Complained`
  - `unsubscribed` → `Suppressed`.

### 5. Idempotency

- Mailgun accepts a client-supplied `Message-Id` header.
  Emit the ADR-0149 idempotency-key fingerprint as that header
  so retries collapse server-side.

### 6. Errors

- Map Mailgun 4xx → `ProviderError` non-retryable.
- Map Mailgun 5xx → `ProviderError` retryable.
- Map Mailgun 429 → `RateCeilingExceeded`.

### 7. Observability

- OTEL spans `comms.email.mailgun.send`.
- Prometheus counter `comms_email_mailgun_sends_total` labeled
  `{result=ok|rejected|throttled|error}`.

### 8. Tests

- Unit tests for the HTTP request builder.
- Integration test against a fixture Mailgun mock server.
- Webhook event ingest test.

## Failure modes

- Mailgun regional outage: tenant flips to alternate adapter
  (SES or Postal); SLO `send-success rate` stays green.
- Mailgun account suspension: emits a critical alert; runbook
  `mailgun-account-recovery.md` (placeholder; covered by
  blacklist-recovery runbook as the generic recovery path).

## Acceptance criteria

- `cargo test -p oya-shared-email-comms-kernel-adapter-mailgun`
  passes.
- A test send through a Mailgun sandbox account succeeds and
  the webhook delivery event arrives in the audit chain.

## Rollback

Flag flip.

## References

- ADR-0201.
- ADR-0173 vendor lock-in avoidance.
- Mailgun upstream documentation.
