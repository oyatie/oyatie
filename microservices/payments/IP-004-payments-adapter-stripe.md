---
doc_class: ImplementationPlan
id: IP-004
title: "oya-payments-adapter-stripe — PspAdapter impl for Stripe Connect"
microservice: payments
bounded_context: charge
layer: adapter
status: accepted
date: 2026-05-20
owner_team: axis-payments
pr_size_estimate: "≤700 LOC"
related_adrs:
  - ADR-0056
  - ADR-0105
  - ADR-0145
  - ADR-0253
  - ADR-0255
  - ADR-0296
diataxis_quadrant: how-to
doc_status: published
---

# IP-004 — oya-payments-adapter-stripe

## Purpose

Implement `oya-payments-adapter-stripe`: `PspAdapter` for Stripe, including Connect platform-facilitator pattern, webhook-signature verification, and provider-BYOK credential retrieval from OpenBao.

## Acceptance criteria

- [ ] `StripeAdapter` implements `PspAdapter` trait from `oya-payments-charge-kernel`.
- [ ] `authorize()` calls `POST https://api.stripe.com/v1/charges` (or `POST /v1/payment_intents` for SCA flows); uses tenant's Stripe secret key fetched from OpenBao `secret/<tenant_id>/payments/stripe/secret_key` per ADR-0296.
- [ ] `capture()` calls `POST /v1/charges/{id}/capture`.
- [ ] `refund()` calls `POST /v1/refunds`.
- [ ] `payout()` calls `POST /v1/payouts` using Connect transfer → payout pattern.
- [ ] `handle_webhook()` verifies `Stripe-Signature` header HMAC-SHA256 per Stripe docs; returns `PspError::WebhookReplayRejected` if `Stripe-Timestamp` older than 300s per `policy/abuse-defence.cedar`.
- [ ] HTTP/3 + QUIC for all outbound Stripe API calls; fallback to HTTP/2 per ADR-0253.
- [ ] Credential fetch: sidecar TTL ≤60s; never log or persist the key in-process beyond request lifecycle per ADR-0296.
- [ ] Rate-limit: 100 req/s per tenant (Stripe rate-limit); backpressure via `PspError::RateLimited` with `retry_after_ms`.
- [ ] Integration tests (with Stripe test-mode keys): authorize + capture + refund happy path; webhook replay rejection.
- [ ] `cargo clippy` zero warnings.

## Dependencies

- IP-001 (kernel) must be merged first.

## Implementation notes

```toml
[dependencies]
oya-payments-charge-kernel = { path = "../oya-payments-charge-kernel" }
reqwest = { version = "0.12", features = ["http3", "json", "rustls-tls"] }
hmac = "0.12"
sha2 = "0.10"
serde_json = "1"
oya-shared-secrets = { path = "../../shared/oya-shared-secrets" }
```

Webhook verification per Stripe docs:
```rust
fn verify_stripe_signature(payload: &[u8], sig_header: &str, secret: &[u8]) -> Result<(), PspError> {
    // Parse t= and v1= components from Stripe-Signature header
    // Compute HMAC-SHA256 of "<timestamp>.<payload>" with secret
    // Compare constant-time; reject if timestamp delta > 300s
}
```

## Hyperscaler precedent

Stripe Connect platform-facilitator: charge to `on_behalf_of: <connected_account>` with `application_fee_amount`. Payout via `Stripe.transfers.create` then `Stripe.payouts.create` per connected-account flow.

## Cross-references

- `IP-003-payments-usecase-charge.md` — wires this adapter.
- `policy/abuse-defence.cedar` — ANTI-SPOOF webhook-replay gate.
- `ARCHITECTURE.md §D` — PSP adapter table.
- `contracts/psp-adapter-trait.md` — trait spec.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/payments/IP-004-payments-adapter-stripe.md` matched `payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: PCI-DSS-L1-v4(86400s/3600s), SOX-404(14400s/3600s), HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/payments/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/policy/abuse-defence.cedar`.
