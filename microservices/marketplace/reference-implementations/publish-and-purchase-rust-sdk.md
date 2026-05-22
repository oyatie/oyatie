# Reference implementation — Publish + Purchase + Escrow + Payout, end-to-end in Rust

A runnable Rust program that takes two tenants (seller Alice, buyer Bob), publishes a workflow listing, executes a purchase, waits
for escrow release, and triggers the payout.

## `Cargo.toml`

```toml
[package]
name = "marketplace-flow-example"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
oya-marketplace-sdk = "0.42.0"
oya-trace = "0.42.0"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1.43", features = ["macros", "rt-multi-thread"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

## `src/main.rs`

```rust
use anyhow::{Context, Result};
use oya_marketplace_sdk::{
    EscrowState, ListingCategory, ListingCreate, MarketplaceClient, MarketplaceConfig,
    PaymentMethodRef, PricingModel, PurchaseSpec, Tenant,
};
use oya_trace::TraceContext;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let trace = TraceContext::new_root();

    let alice = Tenant::parse("oyatie.b2c.indie.alice")?;
    let bob = Tenant::parse("oyatie.b2c.indie.bob")?;

    let cfg = MarketplaceConfig::builder()
        .endpoint("https://loopback.marketplace.oyatie.local".parse()?)
        .api_key(std::env::var("OYA_API_KEY").context("OYA_API_KEY missing")?)
        .request_timeout(Duration::from_secs(10))
        .build()?;
    let client = MarketplaceClient::connect(cfg).await?;
    info!("connected to marketplace");

    // 1. Alice publishes a listing
    let listing = client
        .listing_create(
            &alice,
            ListingCreate {
                category: ListingCategory::Workflow,
                title: "Daily Standup Summarizer".into(),
                description:
                    "Aggregates yesterday's git activity, calendar events, and outstanding PRs into a 5-bullet summary."
                        .into(),
                pricing: PricingModel::OneTime { amount_minor: 1_900, currency: "USD".into() },
                region_availability: vec![
                    "US".into(), "CA".into(), "GB".into(), "EU".into(),
                    "KR".into(), "JP".into(), "AU".into(),
                ],
                payload_json: serde_json::from_str(include_str!(
                    "../samples/daily-standup-summarizer.workflow.json"
                ))?,
                screenshots: vec![include_str!("../samples/screenshot-1.png.b64").into()],
                license_spdx: "MIT".into(),
            },
            trace.child(),
        )
        .await
        .context("listing create failed")?;
    info!(listing_id = %listing.id(), slug = %listing.slug(), "listing published");

    // 2. Bob purchases
    let purchase = client
        .purchase(
            &bob,
            PurchaseSpec {
                listing_id: listing.id().to_string(),
                payment_method: PaymentMethodRef::Token("test-card-visa".into()),
                idempotency_key: format!("ref-impl-purchase-{}", listing.id()),
            },
            trace.child(),
        )
        .await
        .context("purchase failed")?;
    info!(
        purchase_id = %purchase.id(),
        escrow_state = ?purchase.escrow_state(),
        escrow_until = %purchase.escrow_until(),
        net_payable = purchase.net_payable_minor(),
        "purchase complete"
    );
    assert_eq!(purchase.escrow_state(), EscrowState::Held);

    // 3. Fast-forward time on the dev cell (real time would just sleep)
    client
        .dev_advance_time(Duration::from_secs(7 * 24 * 60 * 60), trace.child())
        .await
        .context("dev_advance_time failed (only valid on dev cell)")?;

    // 4. Poll for escrow release
    let mut backoff = Duration::from_millis(500);
    let released = loop {
        let p = client.purchase_get(&bob, purchase.id(), trace.child()).await?;
        if p.escrow_state() == EscrowState::Released {
            break p;
        }
        if backoff >= Duration::from_secs(10) {
            warn!("escrow not released after 10 s of polling on dev cell; check time advance");
            break p;
        }
        sleep(backoff).await;
        backoff *= 2;
    };
    info!(escrow_state = ?released.escrow_state(), "escrow released");

    // 5. Trigger payout
    let payout = client
        .payout_run(
            &alice,
            "ref-impl-weekly-cycle",
            trace.child(),
        )
        .await
        .context("payout run failed")?;
    info!(
        payout_id = %payout.id(),
        amount_minor = payout.amount_minor(),
        rail = %payout.rail(),
        expected_settle = %payout.expected_settle(),
        "payout initiated"
    );

    // 6. Inspect ledger
    let ledger = client
        .ledger_show(&alice, Duration::from_secs(8 * 24 * 60 * 60), trace.child())
        .await?;
    info!(
        entries = ledger.entries().len(),
        net_balance_minor = ledger.net_balance_minor(),
        "ledger snapshot"
    );

    Ok(())
}
```

## Run it

```bash
OYA_API_KEY=$(./bin/oya creds dev-token --tenant oyatie.b2c.indie.alice) \
  cargo run --release
```

Expected stdout:
```
INFO  connected to marketplace
INFO  listing published listing_id=lst-… slug=daily-standup-summarizer
INFO  purchase complete purchase_id=pur-… escrow_state=Held escrow_until=2026-05-27T08:31:14Z net_payable=1650
INFO  escrow released escrow_state=Released
INFO  payout initiated payout_id=po-… amount_minor=1650 rail=ACH expected_settle=2026-05-23T17:00:00Z
INFO  ledger snapshot entries=4 net_balance_minor=1650
```

## SDK correctness guarantees

1. `ListingCategory` is a closed enum — adding a new category requires an ADR.
2. `PricingModel` is a discriminated union; subscriptions, usage, tiered, enterprise are first-class variants.
3. `purchase` returns immediately with `escrow_state: Held`; release is asynchronous and observable via `purchase_get`.
4. `dev_advance_time` is dev-cell only — production refuses with `OperationOnDevCellOnly`.
5. `payout_run` is idempotent on `(tenant, cycle_id)`; double-calls return the same payout.
6. `ledger_show` is read-only; no side effects.

## Tests

```bash
cargo test --features hermetic
```

The hermetic feature uses `oya_marketplace_sdk::testkit::Hermetic` with a single-process loopback marketplace + mocked payment
rails for fast tests.
