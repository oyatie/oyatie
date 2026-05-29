---
doc_class: SDKPlan
template_id: TPL-SDK-PLAN
microservice: payments
status: Accepted
date: 2026-05-20
owner_team: axis-payments + axis-shared-sdk
related_adrs: [ADR-0145, ADR-0244, ADR-0253, ADR-0258]
companion_docs:
  - microservices/payments/PRD.md
  - microservices/payments/contracts/openapi-v1.yaml
  - microservices/payments/contracts/asyncapi-v1.yaml
  - microservices/payments/contracts/payments-v1.proto
diataxis_quadrant: how-to
doc_status: published
---

# SDK Plan — payments µservice

> Rust + TypeScript + iOS + Android + Python client SDKs. Per-language idioms preserved; type-safe; HTTP/3-aware; PCI-scope-aware.

---

## §1. SDK roster

| Language | Crate / package | Use-case | First ship |
|---|---|---|---|
| Rust | `oya-payments-sdk-rust` | Internal cross-µservice + Rust-tenant-services | M02-foundation |
| TypeScript | `@oyatie/payments-sdk` (npm) | Web tenant surfaces, Node.js backends | M02-foundation |
| iOS (Swift) | `OyatiePaymentsSDK` (SPM) | iOS app (PSP SDK passthrough; we collect tokenised refs only) | M02-foundation |
| Android (Kotlin) | `dev.oyatie.payments-sdk` (Maven) | Android app | M02-foundation |
| Python | `oyatie-payments` (PyPI) | Tenant-data-science notebooks, B2B integrations | Wave-4 |
| Go | `dev.oyatie/payments-sdk-go` | Tenant-Go-services | Wave-4 |

## §2. SDK design principles

1. **Type-safe contracts** — generated from `contracts/openapi-v1.yaml` + `contracts/payments-v1.proto`.
2. **Idempotency-by-default** — every mutating call auto-generates idempotency-key if not provided.
3. **HTTP/3-aware** — falls back to HTTP/2 → HTTP/1.1; never HTTP/1.0.
4. **Tenant-scoped** — every call carries `tenant_id` from auth context; cross-tenant impossible.
5. **PCI-scope-aware** — SDK refuses to accept raw card-data; only PSP-tokenised payment-method-ids.
6. **Per-call observability** — every call emits trace span + optional audit-event-receipt to caller.
7. **No mock-PSP** — SDK never simulates PSP responses; integration tests use PSP sandbox.

## §3. Rust SDK (`oya-payments-sdk-rust`)

```rust
use oya_payments_sdk::{PaymentsClient, Money, CreateChargeRequest};
use rust_decimal::Decimal;

let client = PaymentsClient::builder()
    .tenant_id("tenant_acme")
    .api_base("https://payments.oyatie.com")
    .auth(BearerToken::from_oidc())
    .build()?;

let charge = client.charges()
    .create(CreateChargeRequest {
        amount: Money::new(Decimal::from(1099), Currency::Usd),
        payment_method_id: "pm_tokenised_id".into(),
        description: Some("Sticker pack X".into()),
        idempotency_key: None,  // SDK auto-generates
        ..Default::default()
    })
    .await?;

println!("Charge state: {:?}", charge.state);
```

- Crate location: `microservices/payments/src/crates/oya-payments-charge-sdk/`.
- Re-exports: domain types from `oya-payments-charge-kernel`.
- Async runtime: tokio.
- HTTP transport: hyper + h3 (HTTP/3 preferred).
- Tracing: tracing crate; OTel exporter optional.

## §4. TypeScript SDK (`@oyatie/payments-sdk`)

```ts
import { PaymentsClient, Money, Currency } from '@oyatie/payments-sdk';

const client = new PaymentsClient({
  tenantId: 'tenant_acme',
  apiBase: 'https://payments.oyatie.com',
  auth: { type: 'oidc', token: getOidcToken() },
});

const charge = await client.charges.create({
  amount: Money.of(1099, Currency.USD),
  paymentMethodId: 'pm_tokenised_id',
  description: 'Sticker pack X',
});

console.log('Charge state:', charge.state);
```

- npm package: `@oyatie/payments-sdk`.
- TypeScript strict-mode; ESM + CJS dual-build.
- Generated from OpenAPI 3.2.0 via `openapi-typescript-codegen`.
- Bundles `fetch`-based HTTP client (Node 18+ / Browser).
- Browser-safe: refuses any code path that would accept raw card-data.

## §5. iOS SDK (Swift, `OyatiePaymentsSDK`)

```swift
import OyatiePaymentsSDK

let client = PaymentsClient(
    tenantId: "tenant_acme",
    apiBase: URL(string: "https://payments.oyatie.com")!,
    auth: .oidc(token: oidcToken)
)

let charge = try await client.charges.create(
    request: CreateChargeRequest(
        amount: Money(amount: 1099, currency: .usd),
        paymentMethodId: "pm_tokenised_id",
        description: "Sticker pack X"
    )
)
```

- Distribution: Swift Package Manager.
- Min iOS: 16.0.
- Async/await native.
- Includes integration with Apple Pay (PKPaymentRequest) for tokenisation; SDK never accepts raw card-data.
- App Attest device-attestation per documentation-rigor §3.2.3.

## §6. Android SDK (Kotlin, `dev.oyatie.payments-sdk`)

```kotlin
val client = PaymentsClient.builder()
    .tenantId("tenant_acme")
    .apiBase("https://payments.oyatie.com")
    .auth(Auth.oidc(getOidcToken()))
    .build()

val charge = client.charges.create(
    CreateChargeRequest(
        amount = Money(1099, Currency.USD),
        paymentMethodId = "pm_tokenised_id",
        description = "Sticker pack X"
    )
)
```

- Distribution: Maven Central (`dev.oyatie:payments-sdk:1.0.0`).
- Min Android: SDK 28 (Android 9).
- Coroutines-based.
- Includes integration with Google Pay (PaymentDataRequest) for tokenisation.
- Play Integrity device-attestation per documentation-rigor §3.2.3.

## §7. Python SDK (`oyatie-payments`) — Wave-4

```python
from oyatie_payments import PaymentsClient, Money, Currency

client = PaymentsClient(
    tenant_id="tenant_acme",
    api_base="https://payments.oyatie.com",
    auth=OidcAuth(token=oidc_token),
)

charge = await client.charges.create(
    amount=Money(1099, Currency.USD),
    payment_method_id="pm_tokenised_id",
    description="Sticker pack X",
)
print("Charge state:", charge.state)
```

- PyPI package: `oyatie-payments`.
- Python ≥ 3.11.
- asyncio + httpx.
- Pydantic v2 models generated from OpenAPI.

## §8. Versioning + deprecation

Per ADR-0258:

- **MAJOR**: breaking change to OpenAPI / proto; 18-month sunset of prior MAJOR.
- **MINOR**: backward-compatible addition; no sunset.
- **PATCH**: bug fix.

Each SDK release ships with:

- Generated changelog from OpenAPI / proto diff.
- Migration guide for breaking changes.
- Test-matrix vs server contract versions.

## §9. Auth modes

| Mode | Use-case | SDK support |
|---|---|---|
| OIDC bearer | Web / browser tenant operators | All SDKs |
| Service-account JWT | Tenant backend services | All SDKs |
| SPIFFE SVID (mTLS) | Cross-µservice in-cluster | Rust SDK (M02), others Wave-4 |
| Step-up auth (passkey / WebAuthn) | High-value payout authorisation | TS SDK (browser); native SDKs via WebAuthn-via-OS |

## §10. Test coverage

- Unit tests on all SDK methods.
- Contract-tests against OpenAPI server-spec.
- Integration tests against PSP sandbox (Stripe / Adyen / Toss test-mode).
- Property-based tests on idempotency-key derivation.

## §11. References

- [`contracts/openapi-v1.yaml`](contracts/openapi-v1.yaml).
- [`contracts/asyncapi-v1.yaml`](contracts/asyncapi-v1.yaml).
- [`contracts/payments-v1.proto`](contracts/payments-v1.proto).
- [ADR-0258 — API versioning + deprecation cadence](../../docs/decisions/ADR-0258-api-versioning-deprecation-cadence.md).
- Stripe Web Elements (SDK reference) — `stripe.com/docs/stripe-js`.
- Adyen Web Drop-in — `docs.adyen.com/online-payments/web-drop-in`.
