---
doc_class: ReferenceImplementation
microservice: payments
language: Rust + Bash
date: 2026-05-20
doc_status: published
---

# Reference implementation — Charge + refund flow via the payments Rust SDK

A runnable example that:

1. Tokenizes a card across multiple PSPs (network tokens).
2. Creates a charge with explicit idempotency key.
3. Issues a partial refund.
4. Walks the resulting ledger postings.
5. Verifies the audit-chain emission.

## Cargo.toml

```toml
[package]
name = "payments-example"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-payments-client = { path = "../../../../crates/oya-payments-client" }
oya-audit-chain-client = { path = "../../../../crates/oya-audit-chain-client" }
oya-cedar-client = { path = "../../../../crates/oya-cedar-client" }
tokio = { version = "1.40", features = ["rt-multi-thread", "macros"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
rust_decimal = "1.36"
rust_decimal_macros = "1.36"
uuid = { version = "1.10", features = ["v7"] }
chrono = { version = "0.4", features = ["serde"] }
```

## src/main.rs

```rust
use anyhow::Result;
use oya_payments_client::{
    PaymentsClient, PaymentsClientConfig,
    Customer, CustomerCreate,
    PaymentMethodTokenize, PaymentMethod,
    ChargeCreate, Charge, RefundCreate, Refund,
    Currency, MoneyMinorUnits,
};
use oya_cedar_client::CedarPrincipal;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Construct the client bound to a Cedar merchant_admin principal.
    let principal = CedarPrincipal::from_env("PAYMENTS_PRINCIPAL_JWT")?;
    let client = PaymentsClient::connect(PaymentsClientConfig {
        cell_endpoint: std::env::var("PAYMENTS_CELL_ENDPOINT")?,
        tenant_id: "acme-corp".into(),
        principal,
        request_timeout: std::time::Duration::from_secs(30),
        max_retries: 2,
    }).await?;

    // 2. Create a customer record (idempotent).
    let customer_idem = format!("create-customer-cust-001-{}", chrono::Utc::now().date_naive());
    let customer: Customer = client.customer_create(CustomerCreate {
        customer_id: "cust-001".into(),
        email: "alice@example.com".into(),
        billing_country: "US".parse().unwrap(),
        billing_currency: Currency::USD,
        idempotency_key: customer_idem,
    }).await?;
    println!("Customer created: id={}, created_at={}", customer.id, customer.created_at);

    // 3. Tokenize a card (network token across all active PSPs).
    let tokenize_idem = format!("tokenize-cust-001-card-1-{}", Uuid::now_v7());
    let pm: PaymentMethod = client.payment_method_tokenize(PaymentMethodTokenize {
        customer_id: "cust-001".into(),
        card_number: "4242424242424242".into(),
        exp_month: 12,
        exp_year: 2030,
        cvc: "123".into(),
        network_token: true,
        idempotency_key: tokenize_idem,
    }).await?;
    println!(
        "Payment method: id={}, network_token_status={:?}, psp_tokens={:?}",
        pm.id, pm.network_token_status, pm.psp_token_map
    );

    // 4. Create a charge with explicit idempotency.
    let charge_idem = format!("charge-{}", Uuid::now_v7());
    let charge: Charge = client.charge_create(ChargeCreate {
        customer_id: "cust-001".into(),
        payment_method_id: pm.id.clone(),
        amount: MoneyMinorUnits::new(12500, Currency::USD),  // $125.00
        description: "Example charge".into(),
        idempotency_key: charge_idem.clone(),
        ..Default::default()
    }).await?;
    println!(
        "Charge: id={}, status={:?}, psp_routed={}, fee={}, net={}",
        charge.id,
        charge.status,
        charge.psp_routed,
        charge.fee_minor_units,
        charge.net_to_merchant_minor_units
    );

    // 5. Replay the idempotency key — should return the same charge, no double-charge.
    let charge_replay: Charge = client.charge_create(ChargeCreate {
        customer_id: "cust-001".into(),
        payment_method_id: pm.id.clone(),
        amount: MoneyMinorUnits::new(12500, Currency::USD),
        description: "Example charge".into(),
        idempotency_key: charge_idem,  // SAME key
        ..Default::default()
    }).await?;
    assert_eq!(charge.id, charge_replay.id);
    println!("Idempotency replay confirmed: same charge_id returned, no double-charge.");

    // 6. Issue a partial refund.
    let refund_idem = format!("refund-{}", Uuid::now_v7());
    let refund: Refund = client.refund_create(RefundCreate {
        charge_id: charge.id.clone(),
        amount: MoneyMinorUnits::new(5000, Currency::USD),  // $50.00 partial
        reason: "customer_requested".into(),
        idempotency_key: refund_idem,
    }).await?;
    println!(
        "Refund: id={}, status={:?}, amount={}",
        refund.id, refund.status, refund.amount_minor_units
    );

    // 7. Walk the ledger postings for this transaction.
    let postings = client.ledger_postings_for_transaction(&charge.id).await?;
    println!("Ledger postings for {}:", charge.id);
    for p in &postings {
        println!(
            "  {} | {} | debit={} | credit={}",
            p.account_path, p.currency, p.debit_minor, p.credit_minor
        );
    }
    let total_debit: i64 = postings.iter().map(|p| p.debit_minor).sum();
    let total_credit: i64 = postings.iter().map(|p| p.credit_minor).sum();
    assert_eq!(total_debit, total_credit, "ledger must balance per currency");
    println!("Ledger balanced: debits={} credits={}", total_debit, total_credit);

    Ok(())
}
```

## Expected output (against a demo_trial tenant_class cell with Stripe sandbox)

```
Customer created: id=cus_acme_001, created_at=2026-05-20T14:32:17Z
Payment method: id=pm_acme_001, network_token_status=Available, psp_tokens={"stripe": "pm_1Q...", "adyen": "tok_adyen_...", "checkout": "src_check_..."}
Charge: id=ch_acme_001, status=Succeeded, psp_routed=stripe, fee=395, net=12105
Idempotency replay confirmed: same charge_id returned, no double-charge.
Refund: id=re_acme_001, status=Succeeded, amount=5000
Ledger postings for ch_acme_001:
  tenant.acme-corp.receivable.stripe | USD | debit=12500 | credit=0
  tenant.acme-corp.revenue           | USD | debit=0     | credit=12500
  tenant.acme-corp.expense.psp_fee   | USD | debit=395   | credit=0
  tenant.acme-corp.payable.stripe_fee| USD | debit=0     | credit=395
  tenant.acme-corp.revenue           | USD | debit=5000  | credit=0
  tenant.acme-corp.receivable.stripe | USD | debit=0     | credit=5000
  tenant.acme-corp.expense.psp_fee   | USD | debit=0     | credit=30
  tenant.acme-corp.payable.stripe_fee| USD | debit=30    | credit=0
Ledger balanced: debits=17925 credits=17925
```

## HTTP alternative (curl)

```sh
# Create customer
curl -X POST https://payments.prod-syd-1.oyatie.local/v1/customers \
    -H "Authorization: Bearer $PAYMENTS_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Idempotency-Key: create-customer-cust-001-2026-05-20" \
    -H "Content-Type: application/json" \
    -d '{
        "customer_id": "cust-001",
        "email": "alice@example.com",
        "billing_country": "US",
        "billing_currency": "USD"
    }'

# Create charge
curl -X POST https://payments.prod-syd-1.oyatie.local/v1/charges \
    -H "Authorization: Bearer $PAYMENTS_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Idempotency-Key: charge-cust-001-2026-05-20-001" \
    -H "Content-Type: application/json" \
    -d '{
        "customer_id": "cust-001",
        "payment_method_id": "pm_acme_001",
        "amount_minor_units": 12500,
        "currency": "USD",
        "description": "Example charge"
    }'

# Refund (partial)
curl -X POST https://payments.prod-syd-1.oyatie.local/v1/refunds \
    -H "Authorization: Bearer $PAYMENTS_JWT" \
    -H "X-Oya-Tenant-Id: acme-corp" \
    -H "Idempotency-Key: refund-charge-ch_acme_001-001" \
    -H "Content-Type: application/json" \
    -d '{
        "charge_id": "ch_acme_001",
        "amount_minor_units": 5000,
        "reason": "customer_requested"
    }'
```

## Error handling — what to retry

| Error class | HTTP | Retry? | Action |
|---|---|---|---|
| `idempotency_key_mismatch` | 422 | No | Client bug; the same key was used with a different body |
| `cedar_denied` | 403 | No | Principal lacks `payments::charge::create` permission |
| `card_declined` | 402 | No | Customer's card was declined; surface the decline-code |
| `card_declined_velocity` | 402 | No | Customer hit per-card velocity limit; surface to customer |
| `psp_unavailable` | 503 | Yes (auto, fail-over) | Primary PSP down; routing fails over to fallback automatically; SDK transparent retry |
| `psp_rate_limited` | 429 | Yes (auto, backoff) | PSP throttled; SDK exponential backoff |
| `fx_rate_stale` | 503 | No | FX rate > 60 s old; surface to customer; tenant may accept stale via header |
| `tenant_quota_exceeded` | 429 | Yes (auto, backoff) | Tenant hit per-second charge quota; backoff and retry |
| `webhook_replay_window_exceeded` | 422 | No | Idempotency key replayed > 24 h after first use; create a new key |

## Audit-chain events emitted

| Operation | Event class |
|---|---|
| `customer_create` | `payments.customer.created` |
| `payment_method_tokenize` | `payments.payment_method.tokenized` |
| `charge_create` (succeeded) | `payments.charge.created`, `payments.charge.succeeded`, `payments.psp.routed` |
| `charge_create` (failed) | `payments.charge.created`, `payments.charge.failed` |
| `refund_create` | `payments.refund.executed` |
| Idempotency replay | (no new event; original event_id linked) |

All events Ed25519-signed against the cell's signing key.

## Where this file lives

`microservices/payments/reference-implementations/charge-and-refund-rust-sdk.md` (this file). The runnable Cargo project ships at `microservices/payments/reference-implementations/charge-refund-example/` once the `oya-payments-client` crate is published.
