---
doc_class: APIReference
microservice: payments
version: 1.0.0-mvp
status: Proposed
date: 2026-05-20
owner: axis-payments + council-finance + ops-fraud + ops-treasury
openapi_version: 3.2.0
asyncapi_version: 3.1.0
proto3: true
---

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

# payments API Reference

Canonical REST, gRPC, and AsyncAPI reference for the `payments` microservice.
The service is the multi-PSP, multi-currency, marketplace-facilitator substrate
for charges, refunds, payouts, disputes, subscriptions, sub-merchants, and PSP
webhooks.

## Quick Start

Named example: `ChargeCaptureAndSubscribe`.

1. Create an authorization with `POST /v1/charges`.
2. Capture it with `POST /v1/charges/{charge_id}/capture`.
3. Subscribe to `payment-events.{tenant_id}` for `ChargeCaptured` and `ChargeErrored`.

Minimum headers:

- `Authorization: Bearer <oidc-token>`
- `X-Tenant-Id: <uuid-v7>` where the contract requires tenant scoping
- `Idempotency-Key: <ulid>` on every mutating request
- `X-Request-Id: <ulid>` for reconciliation
- `Content-Type: application/json`

Example:

```http
POST /v1/charges HTTP/2
Host: payments.oyatie.com
Authorization: Bearer eyJ...
X-Tenant-Id: 018f7a54-3ef5-7c42-a111-a2c4ad7f88f0
Idempotency-Key: 01HYPAYAUTH0000000000000
Content-Type: application/json
```

## Authentication & Authorization

Authentication patterns:

- OIDC bearer for tenant operators and product surfaces.
- SPIFFE SVID for service-to-service gRPC.
- HMAC verification for inbound PSP webhooks.
- OpenBao SecretReference for provider-BYOK credential resolution.

Principal types:

- `PaymentBuyer`: consumer or employee initiating a payment.
- `MerchantOperator`: tenant operator managing charges, refunds, and disputes.
- `SubMerchantOperator`: marketplace seller or vendor principal.
- `TreasuryOperator`: payout, settlement, FX, and reconciliation operator.
- `FraudAnalyst`: restricted fraud review principal.
- `PspWebhookPrincipal`: verified Stripe, Adyen, Toss, KakaoPay, LINE Pay, or Checkout webhook source.
- `PaymentsSettlementWorker`: internal worker moving funds and ledger postings.
- `PaymentsAuditor`: scoped read-only evidence and dispute principal.

Named Cedar policy patterns:

- `payments::tenant_scope_match`: tenant id binds token, account, and ledger rows.
- `payments::charge_create`: buyer and merchant must be allowed for payment method and pack.
- `payments::charge_capture`: capture must follow authorization ownership and amount bounds.
- `payments::refund_authorization`: refund requires role, settlement state, and amount ceiling.
- `payments::payout_authorization`: payout requires treasury scope and KYC/KYB pass.
- `payments::dispute_evidence_submit`: evidence submit requires merchant or delegated operator.
- `payments::sub_merchant_onboarding`: KYB/KYC data class and pack checks must pass.
- `payments::emergency_services_bypass`: medical/emergency payment exception, audit-sealed.
- `payments::abuse_defence`: blocks fraud, card testing, velocity abuse, and sanctioned accounts.

Authorization failure shape:

```json
{
  "error": {
    "code": "PAYMENTS_AUTHZ_DENIED",
    "message": "Cedar policy denied payment action",
    "request_id": "01HYREQ...",
    "details": [{"policy": "payments::charge_create"}]
  }
}
```

## REST Endpoints

Base URLs:

- Production: `https://payments.oyatie.com`
- Sandbox: `https://payments-sandbox.oyatie.com`

Status note:

- `contract-bound` means the endpoint exists in the current OpenAPI file.
- `reference-planned` means the PRD/tier surface names the API but the OpenAPI
  contract must still be expanded before runtime exposure.

### Charges

#### 1. `POST /v1/charges` (`contract-bound`)
- Resource: Charge collection.
- Request schema: `CreateChargeRequest` with amount, currency, buyer, merchant, payment method, and capture mode.
- Response schema: `Charge`.
- Status codes: `201`, `400`, `401`, `403`, `409`, `422`, `429`, `503`.
- Error shape: `CHARGE_VALIDATION_FAILED`, `PAYMENT_METHOD_REFUSED`, `IDEMPOTENCY_REPLAY_CONFLICT`.

#### 2. `GET /v1/charges` (`contract-bound`)
- Resource: Charge collection.
- Request schema: query `tenant_id`, `status`, `created_after`, `cursor`, `limit`.
- Response schema: `ListChargesResponse`.
- Status codes: `200`, `400`, `401`, `403`, `422`, `429`, `503`.
- Error shape: `CHARGE_CURSOR_INVALID`, `TENANT_SCOPE_MISMATCH`.

#### 3. `GET /v1/charges/{charge_id}` (`contract-bound`)
- Resource: Charge entity.
- Request schema: path `charge_id`.
- Response schema: `Charge`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `503`.
- Error shape: `CHARGE_NOT_FOUND`, `CHARGE_READ_DENIED`.

#### 4. `POST /v1/charges/{charge_id}/capture` (`contract-bound`)
- Resource: Charge capture action.
- Request schema: `CaptureChargeRequest` with amount, final capture flag, and ledger note.
- Response schema: `Charge`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `409`, `422`, `429`, `503`.
- Error shape: `CHARGE_NOT_AUTHORIZED`, `CAPTURE_AMOUNT_EXCEEDS_AUTH`, `PSP_TIMEOUT`.

#### 5. `POST /v1/charges/{charge_id}/void` (`contract-bound`)
- Resource: Charge void action.
- Request schema: `VoidChargeRequest` with reason and expected status.
- Response schema: `Charge`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `409`, `429`, `503`.
- Error shape: `CHARGE_ALREADY_CAPTURED`, `VOID_WINDOW_CLOSED`.

### Refunds

#### 6. `POST /v1/refunds` (`contract-bound`)
- Resource: Refund collection.
- Request schema: `CreateRefundRequest` with charge id, amount, reason, and idempotency key.
- Response schema: `Refund`.
- Status codes: `201`, `400`, `401`, `403`, `404`, `409`, `422`, `429`, `503`.
- Error shape: `REFUND_AMOUNT_EXCEEDS_CAPTURED`, `REFUND_NOT_ALLOWED`.

#### 7. `GET /v1/refunds/{refund_id}` (`contract-bound`)
- Resource: Refund entity.
- Request schema: path `refund_id`.
- Response schema: `Refund`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `503`.
- Error shape: `REFUND_NOT_FOUND`, `REFUND_READ_DENIED`.

#### 8. `GET /v1/refunds` (`reference-planned`)
- Resource: Refund collection.
- Request schema: query `charge_id`, `status`, `created_after`, `cursor`, `limit`.
- Response schema: `ListRefundsResponse`.
- Status codes: `200`, `400`, `401`, `403`, `422`, `429`, `503`.
- Error shape: `REFUND_CURSOR_INVALID`, `TENANT_SCOPE_MISMATCH`.

### Payouts And Settlement

#### 9. `POST /v1/payouts` (`contract-bound`)
- Resource: Payout collection.
- Request schema: `SchedulePayoutRequest` with destination account, amount, currency, settlement rail, and date.
- Response schema: `Payout`.
- Status codes: `201`, `400`, `401`, `403`, `409`, `422`, `429`, `503`.
- Error shape: `PAYOUT_DESTINATION_UNVERIFIED`, `PAYOUT_KYC_REQUIRED`.

#### 10. `GET /v1/payouts/{payout_id}` (`contract-bound`)
- Resource: Payout entity.
- Request schema: path `payout_id`.
- Response schema: `Payout`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `503`.
- Error shape: `PAYOUT_NOT_FOUND`, `PAYOUT_READ_DENIED`.

#### 11. `GET /v1/payouts` (`reference-planned`)
- Resource: Payout collection.
- Request schema: query `status`, `destination_ref`, `currency`, `cursor`, `limit`.
- Response schema: `ListPayoutsResponse`.
- Status codes: `200`, `400`, `401`, `403`, `422`, `429`, `503`.
- Error shape: `PAYOUT_CURSOR_INVALID`, `TENANT_SCOPE_MISMATCH`.

#### 12. `POST /v1/transfers` (`reference-planned`)
- Resource: Marketplace transfer collection.
- Request schema: `CreateTransferRequest` with source charge, seller account, platform fee, and currency.
- Response schema: `Transfer`.
- Status codes: `201`, `400`, `401`, `403`, `409`, `422`, `429`, `503`.
- Error shape: `TRANSFER_BALANCE_INSUFFICIENT`, `SUB_MERCHANT_RESTRICTED`.

#### 13. `GET /v1/settlements/{settlement_id}` (`reference-planned`)
- Resource: Settlement entity.
- Request schema: path `settlement_id`.
- Response schema: `Settlement`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `503`.
- Error shape: `SETTLEMENT_NOT_FOUND`, `SETTLEMENT_READ_DENIED`.

### Disputes

#### 14. `GET /v1/disputes` (`contract-bound`)
- Resource: Dispute collection.
- Request schema: query `status`, `charge_id`, `due_before`, `cursor`, `limit`.
- Response schema: `ListDisputesResponse`.
- Status codes: `200`, `400`, `401`, `403`, `422`, `429`, `503`.
- Error shape: `DISPUTE_CURSOR_INVALID`, `TENANT_SCOPE_MISMATCH`.

#### 15. `POST /v1/disputes/{dispute_id}/evidence` (`contract-bound`)
- Resource: Dispute evidence bundle.
- Request schema: `SubmitDisputeEvidenceRequest` with documents, narrative, and file refs.
- Response schema: `Dispute`.
- Status codes: `202`, `400`, `401`, `403`, `404`, `409`, `413`, `422`, `429`, `503`.
- Error shape: `DISPUTE_EVIDENCE_WINDOW_CLOSED`, `EVIDENCE_FILE_INVALID`.

#### 16. `GET /v1/disputes/{dispute_id}` (`reference-planned`)
- Resource: Dispute entity.
- Request schema: path `dispute_id`.
- Response schema: `Dispute`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `503`.
- Error shape: `DISPUTE_NOT_FOUND`, `DISPUTE_READ_DENIED`.

### Subscriptions

#### 17. `POST /v1/subscriptions` (`contract-bound`)
- Resource: Subscription collection.
- Request schema: `CreateSubscriptionRequest` with plan, customer, billing cadence, trial, and payment method.
- Response schema: `Subscription`.
- Status codes: `201`, `400`, `401`, `403`, `409`, `422`, `429`, `503`.
- Error shape: `SUBSCRIPTION_PLAN_INVALID`, `PAYMENT_METHOD_REQUIRED`.

#### 18. `GET /v1/subscriptions/{subscription_id}` (`reference-planned`)
- Resource: Subscription entity.
- Request schema: path `subscription_id`.
- Response schema: `Subscription`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `503`.
- Error shape: `SUBSCRIPTION_NOT_FOUND`, `SUBSCRIPTION_READ_DENIED`.

#### 19. `POST /v1/subscriptions/{subscription_id}/cancel` (`reference-planned`)
- Resource: Subscription cancel action.
- Request schema: `CancelSubscriptionRequest` with reason, effective_at, and proration policy.
- Response schema: `Subscription`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `409`, `422`, `429`.
- Error shape: `SUBSCRIPTION_ALREADY_CANCELLED`, `PRORATION_POLICY_INVALID`.

### Sub-Merchants And Payment Methods

#### 20. `POST /v1/sub-merchants` (`contract-bound`)
- Resource: Sub-merchant collection.
- Request schema: `OnboardSubMerchantRequest` with legal entity, beneficial owners, country, and PSP target.
- Response schema: `SubMerchant`.
- Status codes: `201`, `400`, `401`, `403`, `409`, `422`, `429`, `503`.
- Error shape: `SUB_MERCHANT_KYB_REQUIRED`, `UNSUPPORTED_COUNTRY`.

#### 21. `GET /v1/sub-merchants/{sub_merchant_id}` (`reference-planned`)
- Resource: Sub-merchant entity.
- Request schema: path `sub_merchant_id`.
- Response schema: `SubMerchant`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `503`.
- Error shape: `SUB_MERCHANT_NOT_FOUND`, `SUB_MERCHANT_READ_DENIED`.

#### 22. `POST /v1/payment-methods` (`reference-planned`)
- Resource: Payment method token collection.
- Request schema: `CreatePaymentMethodRequest` with type, PSP token, billing details, and consent.
- Response schema: `PaymentMethod`.
- Status codes: `201`, `400`, `401`, `403`, `409`, `422`, `429`, `503`.
- Error shape: `PAYMENT_METHOD_TOKEN_INVALID`, `PCI_SCOPE_REFUSED`.

#### 23. `DELETE /v1/payment-methods/{payment_method_id}` (`reference-planned`)
- Resource: Payment method token.
- Request schema: path `payment_method_id`.
- Response schema: empty success envelope.
- Status codes: `204`, `401`, `403`, `404`, `409`, `429`.
- Error shape: `PAYMENT_METHOD_IN_USE`, `PAYMENT_METHOD_NOT_FOUND`.

### Tax, Ledger, Webhooks

#### 24. `POST /v1/tax/quotes` (`reference-planned`)
- Resource: Tax quote.
- Request schema: `CreateTaxQuoteRequest` with taxable lines, buyer location, seller location, and product codes.
- Response schema: `TaxQuote`.
- Status codes: `200`, `400`, `401`, `403`, `422`, `429`, `503`.
- Error shape: `TAX_JURISDICTION_UNSUPPORTED`, `TAX_PROVIDER_UNAVAILABLE`.

#### 25. `GET /v1/ledger-entries` (`reference-planned`)
- Resource: Double-entry ledger collection.
- Request schema: query `entity_ref`, `currency`, `period`, `cursor`, `limit`.
- Response schema: `LedgerEntryPage`.
- Status codes: `200`, `400`, `401`, `403`, `422`, `429`, `503`.
- Error shape: `LEDGER_CURSOR_INVALID`, `LEDGER_READ_DENIED`.

#### 26. `POST /v1/webhooks/{psp}/v1` (`contract-bound`)
- Resource: PSP webhook ingress.
- Request schema: provider-specific `PspWebhookEnvelope` with HMAC signature headers.
- Response schema: `WebhookAck`.
- Status codes: `200`, `202`, `400`, `401`, `403`, `409`, `422`, `429`, `503`.
- Error shape: `WEBHOOK_SIGNATURE_INVALID`, `WEBHOOK_REPLAY_DETECTED`, `PSP_EVENT_UNSUPPORTED`.

## gRPC Methods

Package: `oyatie.payments.v1`.

### `ChargeService`

- `rpc CreateCharge(CreateChargeRequest) returns (Charge);`
- `rpc CaptureCharge(CaptureChargeRequest) returns (Charge);`
- `rpc VoidCharge(VoidChargeRequest) returns (Charge);`
- `rpc GetCharge(GetChargeRequest) returns (Charge);`
- `rpc ListCharges(ListChargesRequest) returns (ListChargesResponse);`

### `RefundService`

- `rpc CreateRefund(CreateRefundRequest) returns (Refund);`

### `PayoutService`

- `rpc SchedulePayout(SchedulePayoutRequest) returns (Payout);`

### `DisputeService`

- `rpc SubmitEvidence(SubmitDisputeEvidenceRequest) returns (Dispute);`

### `SubscriptionService`

- `rpc CreateSubscription(CreateSubscriptionRequest) returns (Subscription);`

### `SubMerchantService`

- `rpc OnboardSubMerchant(OnboardSubMerchantRequest) returns (SubMerchant);`

Message families:

- `Money`
- `Charge`
- `Refund`
- `Payout`
- `Dispute`
- `DisputeEvidenceBundle`
- `Subscription`
- `SubMerchant`

## AsyncAPI Channels

Delivery defaults:

- AMQP over QUIC where supported, AMQP/1.0 fallback.
- Per-tenant topic shape `payment-events.{tenant_id}`.
- At-least-once delivery with idempotent consumers by event id and PSP idempotency key.
- Every payload carries `tenant_id`, `audit_chain_seq`, `principal_svid`, and `audit_event_class`.

Publish channels:

- `payment-events.{tenant_id}` with `ChargeAuthorized`.
- `payment-events.{tenant_id}` with `ChargeCaptured`.
- `payment-events.{tenant_id}` with `ChargeDeclined`.
- `payment-events.{tenant_id}` with `ChargeErrored`.
- `payment-events.{tenant_id}` with `RefundIssued`.
- `payment-events.{tenant_id}` with `RefundFailed`.
- `payment-events.{tenant_id}` with `PayoutScheduled`.
- `payment-events.{tenant_id}` with `PayoutInitiated`.
- `payment-events.{tenant_id}` with `PayoutCompleted`.
- `payment-events.{tenant_id}` with `PayoutFailed`.
- `payment-events.{tenant_id}` with `DisputeOpened`.
- `payment-events.{tenant_id}` with `DisputeEvidenceSubmitted`.
- `payment-events.{tenant_id}` with `DisputeResolved`.
- `payment-events.{tenant_id}` with `SubscriptionCreated`.
- `payment-events.{tenant_id}` with `SubscriptionDunningAttempted`.
- `payment-events.{tenant_id}` with `SubscriptionCancelled`.
- `payment-events.{tenant_id}` with `SubMerchantOnboarded`.
- `payment-events.{tenant_id}` with `SubMerchantRestricted`.

Subscribe channels:

- `audit-chain.seal.confirmed`: payload `AuditSealConfirmed`.
- `tenancy.capability-tier.changed`: payload `CapabilityTierChanged`.
- `governance.pack-policy.changed`: payload `CompliancePackPolicyChanged`.
- `intelligence.fraud-score.completed`: payload `FraudScoreCompleted`.
- `drive.dispute-evidence.file-ready`: payload `EvidenceFileReady`.

## Webhooks Inbound

- `stripe.charge.succeeded`: payload `StripeChargeSucceeded`, maps to `ChargeCaptured`.
- `stripe.charge.failed`: payload `StripeChargeFailed`, maps to `ChargeDeclined` or `ChargeErrored`.
- `stripe.dispute.created`: payload `StripeDisputeCreated`, maps to `DisputeOpened`.
- `adyen.AUTHORISATION`: payload `AdyenAuthorisation`, maps to charge state.
- `adyen.CAPTURE`: payload `AdyenCapture`, maps to `ChargeCaptured`.
- `adyen.REFUND`: payload `AdyenRefund`, maps to `RefundIssued`.
- `checkout.payment_approved`: payload `CheckoutPaymentApproved`, maps to charge authorization.
- `toss.payment.approved`: payload `TossPaymentApproved`, maps to KR PSP charge state.
- `kakaopay.payment.approved`: payload `KakaoPayPaymentApproved`, maps to KR wallet state.
- `linepay.payment.captured`: payload `LinePayPaymentCaptured`, maps to APAC wallet capture.
- `paypal.dispute.created`: payload `BraintreeDisputeCreated`, maps to dispute lifecycle.
- `banking.payout.settled`: payload `BankingPayoutSettled`, maps to payout completion.

## SDK Quick Reference

Rust:

```rust
let client = PaymentsClient::connect(endpoint, token)?;
let charge = client.create_charge(CreateChargeRequest::authorize(amount, "USD")).await?;
let captured = client.capture_charge(charge.id(), amount).await?;
let refund = client.create_refund(captured.id(), amount).await?;
```

TypeScript:

```ts
const payments = new PaymentsClient({ endpoint, token, tenantId });
const charge = await payments.createCharge({ amount, currency: "USD", captureMode: "manual" });
await payments.captureCharge({ chargeId: charge.chargeId, amount });
await payments.subscribePaymentEvents({ tenantId });
```

Python:

```python
payments = PaymentsClient(endpoint=endpoint, token=token, tenant_id=tenant_id)
charge = payments.create_charge(amount=amount, currency="USD", capture_mode="manual")
payments.capture_charge(charge_id=charge.charge_id, amount=amount)
payments.create_refund(charge_id=charge.charge_id, amount=amount)
```

Named SDK functions:

- `create_charge(input)`
- `list_charges(filters, cursor=None)`
- `get_charge(charge_id)`
- `capture_charge(charge_id, amount=None)`
- `void_charge(charge_id, reason)`
- `create_refund(input)`
- `schedule_payout(input)`
- `list_disputes(filters)`
- `submit_dispute_evidence(dispute_id, evidence)`
- `create_subscription(input)`
- `onboard_sub_merchant(input)`
- `receive_psp_webhook(psp, payload, signature)`

## Error Catalogue

- `PAYMENTS_AUTHN_MISSING`: no bearer or workload identity; do not retry unchanged.
- `PAYMENTS_AUTHZ_DENIED`: Cedar denied payment action; do not retry unchanged.
- `TENANT_SCOPE_MISMATCH`: tenant binding mismatch; fix token or tenant id.
- `IDEMPOTENCY_REPLAY_CONFLICT`: same key used with different body; create a new key.
- `PAYMENT_METHOD_REFUSED`: payment method not allowed or failed; ask for another method.
- `PCI_SCOPE_REFUSED`: request would enter forbidden PCI scope; do not retry.
- `CHARGE_VALIDATION_FAILED`: invalid amount, currency, or merchant data; fix request.
- `CHARGE_NOT_AUTHORIZED`: capture attempted without authorization; do not retry.
- `CAPTURE_AMOUNT_EXCEEDS_AUTH`: amount exceeds authorization; fix request.
- `REFUND_AMOUNT_EXCEEDS_CAPTURED`: refund amount invalid; fix request.
- `PAYOUT_DESTINATION_UNVERIFIED`: destination needs verification; retry after KYB/KYC.
- `SUB_MERCHANT_RESTRICTED`: seller restricted; do not pay out.
- `DISPUTE_EVIDENCE_WINDOW_CLOSED`: evidence deadline passed; do not retry.
- `WEBHOOK_SIGNATURE_INVALID`: webhook failed verification; reject and alert.
- `WEBHOOK_REPLAY_DETECTED`: duplicate or replayed PSP webhook; ignore after ack policy.
- `PSP_TIMEOUT`: provider timeout; retry with bounded exponential backoff.
- `PSP_RATE_LIMITED`: provider bucket exhausted; retry after provider hint.
- `FX_RATE_STALE`: stale exchange rate; refresh quote then retry.
- `TAX_PROVIDER_UNAVAILABLE`: tax quote provider unavailable; retry with jitter.
- `LEDGER_IMBALANCE`: double-entry invariant failed; stop and page treasury.
- `RATE_LIMIT`: tenant or principal bucket exhausted; retry after `Retry-After`.

## Pagination

Cursor pattern name: `payments_ledger_cursor_v1`.

- Cursor format: opaque, signed, tenant-bound token.
- Default page size: `50`.
- Maximum charge page size: `200`.
- Maximum refund page size: `200`.
- Maximum payout page size: `200`.
- Maximum dispute page size: `100`.
- Maximum ledger page size: `500`.
- Ordering: created time descending for operational APIs.
- Ledger ordering: posting sequence ascending within period.
- Cursor stability: binds to tenant, resource family, filter hash, and sequence watermark.

## Rate Limits per Tier

ADR-0316 payments tiers map to payment transaction and API envelopes.

| Tier | API reads | Mutating API | Transaction rate | Webhook ingress | Notes |
|---|---:|---:|---:|---:|---|

Rate-limit headers:

- `Retry-After`
- `oya-throttle-class`
- `oya-throttle-user-headroom`
- `oya-throttle-tenant-headroom`
- Provider-specific retry hints, preserved under `details.provider_retry_after`

## OpenAPI 3.2.0 Schema

Contract file: [`microservices/payments/contracts/openapi-v1.yaml`](../../microservices/payments/contracts/openapi-v1.yaml).

## AsyncAPI 3.1.0 Schema

Contract file: [`microservices/payments/contracts/asyncapi-v1.yaml`](../../microservices/payments/contracts/asyncapi-v1.yaml).

## proto3 Schema

Contract file: [`microservices/payments/contracts/payments-v1.proto`](../../microservices/payments/contracts/payments-v1.proto).

## Cross-References

- PRD: [`microservices/payments/PRD.md`](../../microservices/payments/PRD.md).
- Architecture: [`microservices/payments/ARCHITECTURE.md`](../../microservices/payments/ARCHITECTURE.md).
- SDK plan: [`microservices/payments/sdk-plan.md`](../../microservices/payments/sdk-plan.md).
- Capability tiers: [`microservices/payments/capability-tiers/tier-matrix.md`](../../microservices/payments/capability-tiers/tier-matrix.md).
- PSP adapter trait: [`microservices/payments/contracts/psp-adapter-trait.md`](../../microservices/payments/contracts/psp-adapter-trait.md).
- Policies: [`microservices/payments/policy/`](../../microservices/payments/policy/).
- API standard: [`docs/standards/api-design.md`](../standards/api-design.md).
- ADR-0316: [`docs/decisions/ADR-0709-general-live-apex.md`](../decisions/ADR-0316-capability-tier-over-product-fragmentation.md).
