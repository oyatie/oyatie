---
doc_class: ContractSpec
template_id: TPL-CONTRACT-SPEC
microservice: payments
status: Accepted
date: 2026-05-20
owner_team: axis-payments + axis-shared-sdk
related_adrs: [ADR-0145, ADR-0246]
companion_docs:
  - microservices/payments/ARCHITECTURE.md
  - microservices/payments/contracts/openapi-v1.yaml
  - microservices/payments/contracts/payments-v1.proto
diataxis_quadrant: reference
doc_status: published
---

# PSP Adapter Trait — interface contract

> The stable interface every per-PSP adapter crate must implement. New PSPs (Klarna, PayPal, Coinbase, etc.) join the substrate by adding an adapter crate that implements this trait.

---

## §1. Trait shape

```rust
// crates/oya-payments-charge-kernel/src/ports.rs
use async_trait::async_trait;
use crate::entities::*;
use crate::errors::*;

#[async_trait]
pub trait PspAdapter: Send + Sync {
    fn psp_id(&self) -> PspId;
    fn supported_regions(&self) -> &[Region];
    fn supported_currencies(&self) -> &[Currency];
    fn supported_payment_methods(&self) -> &[PaymentMethodKind];
    fn supported_capabilities(&self) -> PspCapabilities;

    /// Authorise (and optionally capture) a charge. Idempotency-key required.
    async fn authorize(
        &self,
        ctx: &PspCallContext,
        req: AuthorizeRequest,
    ) -> Result<AuthorizeResponse, PspError>;

    /// Capture a previously-authorised charge.
    async fn capture(
        &self,
        ctx: &PspCallContext,
        req: CaptureRequest,
    ) -> Result<CaptureResponse, PspError>;

    /// Void an authorised-but-not-captured charge.
    async fn void(
        &self,
        ctx: &PspCallContext,
        req: VoidRequest,
    ) -> Result<VoidResponse, PspError>;

    /// Issue a refund against a captured charge.
    async fn refund(
        &self,
        ctx: &PspCallContext,
        req: RefundRequest,
    ) -> Result<RefundResponse, PspError>;

    /// Schedule a payout to a bank-account.
    async fn payout(
        &self,
        ctx: &PspCallContext,
        req: PayoutRequest,
    ) -> Result<PayoutResponse, PspError>;

    /// Onboard a sub-merchant (/ MarketPay-equivalent).
    async fn onboard_sub_merchant(
        &self,
        ctx: &PspCallContext,
        req: OnboardSubMerchantRequest,
    ) -> Result<OnboardSubMerchantResponse, PspError>;

    /// Process an inbound webhook: verify HMAC, parse, emit domain events.
    async fn handle_webhook(
        &self,
        ctx: &PspCallContext,
        payload: WebhookPayload,
    ) -> Result<Vec<DomainEvent>, PspError>;

    /// Reconcile vs PSP settlement report.
    async fn fetch_settlement_report(
        &self,
        ctx: &PspCallContext,
        date: SettlementDate,
    ) -> Result<SettlementReport, PspError>;
}
```

## §2. Required types

```rust
pub struct PspCallContext<'a> {
    pub tenant_id: TenantId,
    pub principal_svid: PrincipalSvid,
    pub trace_context: TraceContext,
    pub provider_credential: ProviderCredential,  // Per ADR-0255 §D-4 tenant-BYOK
    pub region_hint: Option<Region>,
    pub idempotency_key: IdempotencyKey,
    pub deadline: Instant,
}

pub enum ProviderCredential {
    Stripe { secret_key: SecretRef, webhook_secret: SecretRef },
    Adyen { api_key: SecretRef, hmac_key: SecretRef, merchant_account: String },
    Toss { secret_key: SecretRef, webhook_key: SecretRef, merchant_id: String },
    KakaoPay { secret_key: SecretRef, ... },
    LinePay { channel_id: String, channel_secret: SecretRef, ... },
    WechatPay { mch_id: String, api_v3_key: SecretRef, ... },
    Alipay { app_id: String, private_key: SecretRef, alipay_public_key: SecretRef, ... },
}

pub struct PspCapabilities {
    pub supports_subscriptions: bool,
    pub supports_marketplace: bool,
    pub supports_partial_capture: bool,
    pub supports_partial_refund: bool,
    pub max_currency_count: usize,
    pub rate_limit_per_second: u32,
}

pub enum PspError {
    Network(NetworkError),
    Authentication(AuthError),
    Authorization(AuthzError),
    InvalidRequest(ValidationError),
    Declined(DeclineReason),
    Conflict(ConflictReason),
    RateLimited { retry_after_seconds: u32 },
    PspUnavailable,
    Timeout,
    UnknownPspResponse(String),
}
```

## §3. Invariants per ADR-0145

Every adapter implementation MUST:

1. **No shared mutable state across calls.** Each call is stateless modulo SDK connection-pool.
2. **No shared secret coupling beyond `provider_credential`.** The credential is the only dependency; no per-tenant config exists in the adapter binary.
3. **Direct gRPC / HTTP only.** No queue-as-middleman; no shared cache; no broker.
4. **Idempotency-key propagated.** Every request MUST set the PSP-side idempotency-key from `ctx.idempotency_key`.
5. **Trace-context propagated.** `traceparent` header set per W3C Trace Context.

## §4. Webhook handling contract

```rust
pub struct WebhookPayload {
    pub raw_body: Bytes,
    pub headers: HashMap<String, String>,
    pub received_at: Timestamp,
}

// Adapter MUST:
//   1. Verify HMAC signature using `ctx.provider_credential` webhook secret.
//   2. Reject if `received_at - payload.event_time > 300s` (replay-window).
//   3. Look up idempotency-key in `payments.webhook_events`; reject if duplicate.
//   4. Parse PSP-specific event format into oyatie domain events.
//   5. Return Vec<DomainEvent>; caller persists + emits to AsyncAPI channel.
```

## §5. Per-PSP adapter implementations (in M02-foundation)

| PSP | Crate | Key dependencies | Notes |
|---|---|---|---|
| Stripe | `oya-payments-adapter-stripe` | `stripe-rust` v0.30+ | platform-facilitator pattern |
| Adyen | `oya-payments-adapter-adyen` | per-PSP REST client | MarketPay equivalent |
| Toss Payments | `oya-payments-adapter-toss` | per-PSP REST client | KR-FSS-licensed; KRW-only |
| KakaoPay | `oya-payments-adapter-kakaopay` | per-PSP REST client | KR; wallet-only |
| LINE Pay | `oya-payments-adapter-line-pay` | per-PSP REST client | JP / TW / TH |
| WeChat Pay | `oya-payments-adapter-wechat-pay` | per-PSP REST client | CN-PIPL hard-pin |
| Alipay | `oya-payments-adapter-alipay` | per-PSP REST client | CN / global |

## §6. Adapter conformance tests

Every adapter ships with:

- Unit tests for parse / serialise of each PSP-specific event type.
- Integration tests against PSP sandbox (CI-injected sandbox credentials).
- Property tests for idempotency-key handling.
- Contract tests against the trait shape (`assert_impl_all!`).
- Replay-window tests (reject post-window).
- HMAC signature tests (positive + negative).

## §7. Versioning

- Trait is owned by `oya-payments-charge-kernel`; SemVer.
- Per-adapter crate is independent; SemVer.
- Major bump on trait change triggers per-adapter rebuild + audit.

## §8. References

- [`ARCHITECTURE.md`](../ARCHITECTURE.md) §D — per-PSP adapter pattern.
- [`openapi-v1.yaml`](openapi-v1.yaml).
- [`payments-v1.proto`](payments-v1.proto).
- [ADR-0145 — inter-microservice communication](../../../docs/decisions/ADR-0145-inter-microservice-communication-reform.md).
- [ADR-0246 + amendment — library-first policy](../../../docs/decisions/ADR-0246-policy-engine-library-first.md).
- [ADR-0255 §D-4 — provider-BYOK](../../../docs/decisions/ADR-0255-intelligence-two-layer-substrate.md).
- Stripe Rust crate — `crates.io/crates/stripe-rust`.
- Adyen API docs — `docs.adyen.com/api-explorer`.
- Toss Payments API docs — `docs.tosspayments.com`.
