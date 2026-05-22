---
ip_id: IP-004
title: "IP-004: webhook-receiver domain crate"
microservice: connect
bounded_context: webhook-receiver
layers: [domain]
acceptance_status: design-ready
date: 2026-05-20
related_adrs: [ADR-0056, ADR-0105, ADR-0243, ADR-0253, ADR-0263, ADR-0273, ADR-0296]
companion_docs:
  - microservices/connect/catalog/oya-connect-webhook-receiver-domain.yaml
  - microservices/connect/policy/webhook-receiver-gating.cedar
  - microservices/connect/policy/payload-signature-verification.cedar
  - microservices/connect/runbooks/webhook-replay-attack-detected.md
doc_status: published
---

# IP-004: webhook-receiver domain crate

## Purpose

Implement `oya-connect-webhook-receiver-domain` — per-tenant webhook endpoint provisioning, inbound HMAC signature verification (constant-time), replay-window enforcement (≤5min), idempotency-key deduplication, and payload enqueue.

## Acceptance criteria

1. `WebhookReceiverService::receive(endpoint_id, headers, body)` verifies signature, checks replay window, deduplicates by idempotency-key, enqueues to downstream (payload-canonicalization → workflow-engine dispatch).
2. Signature verification: **constant-time HMAC comparison** (`subtle::ConstantTimeEq`) for all algorithms; returns `SignatureVerifyFailed` error without revealing which comparison failed.
3. Replay-window: `timestamp` extracted from vendor-specific header; if `|now - timestamp| > 300s` → `WebhookError::ReplayWindowExpired`; emit `WebhookReplayBlocked` audit event.
4. Idempotency: `key_hash = SHA-256(idempotency_key || tenant_id)` stored in `connect.idempotency_keys` table with 5min TTL; duplicate → `WebhookError::Duplicate` (idempotent 200 OK).
5. Per-vendor signing schemes: `Shopify` (HMAC-SHA256 over body), `Stripe` (HMAC-SHA256 with timestamp), `GitHub` (HMAC-SHA256 `hub.signature-256`), `Slack` (HMAC-SHA256 v0 prefix), `Generic` (HMAC-SHA256).
6. Ack latency ≤200ms p99 under 10,000 events/s (verified by load test in CI).
7. `WebhookReceived` and `WebhookSignatureVerifyFailed` audit events emitted per ADR-0263.
8. Cedar gate `webhook-receiver-gating.cedar` consulted for endpoint registration; receive path is Cedar-exempt (latency-critical; signature IS the gate).

## Key types

```rust
pub enum WebhookSigningScheme {
    HmacSha256 { header: String },
    ShopifyHmacSha256,
    StripeSignature,
    GitHubHmacSha256,
    SlackV0,
}

pub struct InboundWebhook {
    pub endpoint_id: WebhookEndpointId,
    pub tenant_id: TenantId,
    pub connector_name: ConnectorName,
    pub idempotency_key: IdempotencyKey,
    pub received_at: DateTime<Utc>,
    pub body: Bytes,
    pub headers: HeaderMap,
}

impl WebhookReceiverService {
    pub async fn receive(&self, webhook: InboundWebhook) -> Result<WebhookAck, WebhookError>;
    pub async fn register_endpoint(&self, req: RegisterEndpointRequest) -> Result<WebhookEndpoint, WebhookError>;
}
```

## Failure modes

1. **Signing secret not found in OpenBao** → return 500 (internal error, not 401); emit `WebhookSecretMissing` audit event; never 200 without verification.
2. **Replay attack spike** → circuit-breaker on per-tenant replay-block count; `WebhookReplayBlocked` event; alert on `oya_connect_webhook_replay_blocked_total > 50/5min`.
3. **Idempotency table unavailable** → fail-open (process webhook; log warning); idempotency is best-effort on DB failure.
4. **Body too large** → return 413; configurable max body size (default 5MB).

## Definition of done

- [ ] Property test: `forall (key, body) → constant_time_verify(key, body) takes same wall-time as failed verify`
- [ ] Load test: 10,000 webhooks/s for 60s → p99 ack ≤200ms
- [ ] Unit test: per-vendor signing scheme (Shopify, Stripe, GitHub, Slack)
- [ ] `cargo clippy -- -D warnings` passes


## A. Problem
`IP-004: webhook-receiver domain crate` closes a concrete `connect` integration-substrate gap, not a generic planning slot. The issue is that connector behavior spans catalog metadata, OAuth or webhook trust, vendor rate limits, DLQ replay, policy decisions, and SLO evidence; a short line-count shell cannot prove those boundaries. Domain vocabulary for this IP: ConnectorCatalog, OAuthBrokerService, WebhookReceiverService, ConnectorAdapter, DLQ, provider-BYOK, per-tenant webhook DNS, vendor rate-limit profile.

## B. Approach
Webhook receive correctness: per-vendor HMAC schemes, replay window, idempotency digest, fast acknowledgement, payload canonicalization handoff, and DLQ fallback. The implementation remains substrate-only: `workflow-engine` orchestrates, while `connect` supplies connector directory, credential broker, webhook receive, adapter invocation, mapping, retry/DLQ, and audit evidence.

## C. Deliverables
- `microservices/connect/PRD.md` — concrete artifact to verify or update.
- `microservices/connect/ARCHITECTURE.md` — concrete artifact to verify or update.
- `microservices/connect/contracts/openapi/connect-integration.yaml` — concrete artifact to verify or update.
- `microservices/connect/contracts/proto/connect_integration.proto` — concrete artifact to verify or update.
- `microservices/connect/contracts/asyncapi/connect-integration-events.yaml` — concrete artifact to verify or update.
- `microservices/connect/policy/connector-authorization.cedar` — concrete artifact to verify or update.
- `microservices/connect/slos/connector-availability.openslo.yaml` — concrete artifact to verify or update.
- `microservices/connect/competitor-parity-matrix.md` — concrete artifact to verify or update.
- `microservices/connect/policy/payload-signature-verification.cedar` — concrete artifact to verify or update.
- `microservices/connect/capabilities/webhook-endpoint-register.yaml` — concrete artifact to verify or update.
- `microservices/connect/runbooks/webhook-replay-attack-detected.md` — concrete artifact to verify or update.
- `microservices/connect/catalog/oya-connect-webhook-receiver-domain.yaml` — concrete artifact to verify or update.
- Declared Rust crates/types such as `ConnectorCatalog`, `OAuthBrokerService`, `WebhookReceiverService`, `ConnectorAdapter`, or `DlqService` must be added only by implementation PRs that also add tests; this documentation scrub does not fake source existence.

## D. Implementation Steps
1. Confirm the bounded-context row in `microservices/connect/PRD.md` and the retirement/substrate boundary in `microservices/connect/ARCHITECTURE.md`.
2. Trace each public command or event to `contracts/openapi/connect-integration.yaml`, `contracts/proto/connect_integration.proto`, or `contracts/asyncapi/connect-integration-events.yaml`.
3. Check the relevant Cedar policy before adding publish, OAuth, webhook, invoke, replay, or catalog mutation behavior.
4. Bind credentials through `iac/openbao-policy.hcl` and never through raw tenant tokens in docs, tests, or examples.
5. Attach an SLO, dashboard, runbook, or audit-event class for every failure mode named in this IP.
6. Run the IP-specific cargo/gate/contract/load command when source exists; otherwise record the missing crate as implementation debt.

## E. Acceptance
- Artifact links above resolve in this checkout.
- Vendor-specific probes include at least one real connector catalog entry, not a hypothetical vendor.
- Credential, webhook, and DLQ paths have policy plus audit evidence before runtime claims.
- The counterpart matrix row is updated when parity changes.

## F. Evidence
- `microservices/connect/PRD.md`
- `microservices/connect/ARCHITECTURE.md`
- `microservices/connect/contracts/openapi/connect-integration.yaml`
- `microservices/connect/contracts/proto/connect_integration.proto`
- `microservices/connect/contracts/asyncapi/connect-integration-events.yaml`
- `microservices/connect/policy/connector-authorization.cedar`
- `microservices/connect/slos/connector-availability.openslo.yaml`
- `microservices/connect/competitor-parity-matrix.md`
- `microservices/connect/policy/payload-signature-verification.cedar`
- `microservices/connect/capabilities/webhook-endpoint-register.yaml`

## G. Counterparts
| Counterpart pressure | Oyatie closure for this IP |
|---|---|
| Zapier, n8n, Workato, Boomi, MuleSoft, Tray.io, Pipedream, AWS EventBridge, Stripe, Salesforce, Slack, GitHub, GitLab, HubSpot, Notion, Linear, Snowflake, and Twilio | Stripe/GitHub/Slack webhook signing schemes are the vendor probes; AWS EventBridge supplies the event-ingest durability pressure. This IP binds `004 webhook receiver domain` to concrete connect contracts, catalog records, policies, SLOs, runbooks, and IaC instead of a reusable stamp. |
