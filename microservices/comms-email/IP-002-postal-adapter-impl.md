# IP-002 — Postal adapter implementation

> ADR anchor: ADR-0201.
> Crate: `crates/oya-shared-email-comms-kernel` (real `PostalEmailComms` impl).
> Owner: `oya-substrate-comms`.
> Estimate: 6 days.

## Goal

Implement the Postal adapter behind the `EmailComms` trait.
Postal is the canonical provider for the sovereign / air-gapped
tier per ADR-0201 §"Adapter set". The adapter targets the
Postal HTTP API (the upstream Ruby on Rails Postal server) using
the per-tenant credentials stored in OpenBao.

## Why this IP

Sovereign / air-gapped deployments cannot use AWS SES. Postal is
the AGPL self-hosted option that ships as a Helm chart in this
batch. Without this adapter, the sovereign tier has no working
email path.

## Pre-conditions

- `crates/oya-shared-email-comms-kernel` lands.
- ADR-0201 ratified.
- Postal Helm chart at
  `microservices/comms-email/iac/helm/postal/` installs on a
  reference kind cluster.

## Tasks

### 1. Wire the HTTP client

- Add the Postal adapter sub-crate
  `crates/oya-shared-email-comms-kernel-adapter-postal`.
- Depend on `reqwest` for HTTP (workspace-pinned, rustls
  TLS only — no native-tls).

### 2. Implement `EmailComms::send`

- POST `https://{postal-base}/api/v1/send/message` with:
  - `from`, `to`, `subject`, `html_body`, `plain_body`,
    `reply_to`.
  - `tag: oya-tenant-{tenant_id}`.
  - `x-oya-correlation-id` header populated from the audit chain
    correlation id.
- Authenticate with the per-tenant Postal credentials from
  OpenBao.

### 3. DKIM identity binding

- DKIM signing happens at the Postal MTA per-tenant. The
  per-tenant DKIM private key is provisioned into Postal via
  the Postal API on tenant onboarding (IP-011).
- DKIM rotation (IP-005) re-provisions Postal's DKIM secret
  store.

### 4. Webhook ingest

- Postal webhooks POST to the comms-email µservice ingest URL
  (configured via Helm `values.yaml.webhooks.ingestUrl`).
- Map Postal event names to canonical `DeliveryEventKind`:
  - `MessageSent` → `Sent`
  - `MessageDelivered` → `Delivered`
  - `MessageBounced` → `Bounced`
  - `MessageHeld` → `Suppressed`
  - `MessageLinkClicked` → `Clicked`
  - `MessageOpened` → `Opened`
  - `MessageDeliveryFailed` → `Bounced` (with hard/soft flag).
  - `MessageMarkedSpam` → `Complained`.

### 5. Idempotency

- Postal accepts a client-supplied message id. Use the ADR-0149
  idempotency-key fingerprint as the Postal-side message tag so
  that retries produce identical Postal records.

### 6. Errors

- Map Postal HTTP 4xx / 5xx to `EmailCommsError::ProviderError`.
- Map Postal HTTP 429 → `RateCeilingExceeded`.
- Map Postal queue-full → `ProviderError` with explicit
  `code = postal.queue.full`.

### 7. Observability

- OTEL spans `comms.email.postal.send`.
- Prometheus counter `comms_email_postal_sends_total` labeled
  `{result=ok|rejected|throttled|error}`.

### 8. Tests

- Unit tests for the request builder.
- Integration test against a local Postal instance run in CI
  (Postal docker-compose harness).
- Webhook event ingest test asserting the normalized event
  shape.

## Failure modes

- Postal MTA outage: comms-email µservice degrades gracefully —
  sends queue locally for retry; SLO `send-latency p99` widens
  but `send-success rate` stays green for the queued backlog.
- Postal MariaDB exhaustion: handled by the Postal Helm chart's
  PVC autoscale; runbook `postal-failover.md` covers cold-side
  failover.

## Acceptance criteria

- `cargo test -p oya-shared-email-comms-kernel-adapter-postal`
  passes.
- A test send through a local Postal yields a DKIM-signed
  delivery event in the audit chain.
- Sovereign packs (ksa, uae) install the Postal chart and run
  the integration test green.

## Rollback

If the Postal adapter regresses, parent flips
`comms.email.provider` for affected tenants back to the SMTP
fallback (IP-003) so messages still leave the cluster, just
without per-event webhook telemetry. SLO `webhook-delivery-rate`
will drop until the regression is repaired.

## References

- ADR-0201.
- Postal upstream documentation (Postal HTTP API).
- IP-005 DKIM key rotation pipeline.
- IP-008 Webhook delivery pipeline.
