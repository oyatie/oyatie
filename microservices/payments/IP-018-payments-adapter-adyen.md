---
doc_class: ImplementationPlan
id: IP-018
title: "oya-payments-adapter-adyen — PspAdapter impl for Adyen MarketPay"
microservice: payments
bounded_context: charge
layer: adapter
status: accepted
date: 2026-05-20
owner_team: axis-payments
pr_size_estimate: "≤600 LOC"
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

# IP-018 — oya-payments-adapter-adyen

## Purpose

Implement `oya-payments-adapter-adyen`: `PspAdapter` for Adyen, including MarketPay platform-facilitator pattern, interchange-plus routing, and HMAC-SHA256 webhook verification.

## Acceptance criteria

- [ ] `AdyenAdapter` implements `PspAdapter` trait from `oya-payments-charge-kernel`.
- [ ] `authorize()` calls `POST https://checkout-test.adyen.com/v71/payments`; uses tenant's Adyen API key from OpenBao `secret/<tenant_id>/payments/adyen/api_key` per ADR-0296.
- [ ] `capture()` calls `POST /v71/payments/{pspReference}/captures`.
- [ ] `refund()` calls `POST /v71/payments/{pspReference}/refunds`.
- [ ] `payout()` calls Adyen Fund Transfer API `POST /fund/v6/transferFunds` (MarketPay) or `POST /pal/servlet/Payout/v68/payout` (standalone payout).
- [ ] `handle_webhook()` verifies `HmacSignature` in Adyen notification using HMAC-SHA256 over the notification fields; returns `PspError::WebhookReplayRejected` if signature invalid.
- [ ] HTTP/3 + QUIC for all outbound Adyen API calls per ADR-0253; fallback to HTTP/2.
- [ ] Rate-limit: 200 req/s per tenant (Adyen limit); backpressure via `PspError::RateLimited`.
- [ ] EU-specific: Adyen supports 3DS2 SCA (EU PSD2); `authorize()` checks `sca_required` flag and sets `additionalData.executeThreeD = true` if needed.
- [ ] `cargo clippy` zero warnings.
- [ ] Integration tests (Adyen test environment): authorize + capture + refund; HMAC webhook verification.

## Dependencies

- IP-001 (kernel) must be merged first.

## Hyperscaler precedent

Adyen MarketPay: `accountHolder` creation (sub-merchant onboarding), `transferFunds` (platform-to-sub-merchant), `payoutAccountHolder` (payout to bank). Interchange-plus pricing via `processingType=InterchangePlus`.

## Cross-references

- `IP-003-payments-usecase-charge.md` — wires this adapter alongside Stripe.
- `ARCHITECTURE.md §D` — PSP adapter table (Adyen row).
- `compliance.md §3` — EU PSD2 + SCA (Adyen handles 3DS2).
- `contracts/psp-adapter-trait.md` — trait spec.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/payments/IP-018-payments-adapter-adyen.md` matched `payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: PCI-DSS-L1-v4(86400s/3600s), SOX-404(14400s/3600s), HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/payments/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/payments/slos/charge-api-availability.openslo.yaml`, `microservices/payments/slos/charge-api-latency.openslo.yaml`, `microservices/payments/slos/payout-completion-success.openslo.yaml`, `microservices/payments/slos/dispute-response-latency.openslo.yaml`, `microservices/payments/policy/abuse-defence.cedar`.
