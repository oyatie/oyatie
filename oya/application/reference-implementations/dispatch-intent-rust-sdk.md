# Reference implementation — Dispatch an intent with the `oya-application-sdk` Rust crate

A complete, runnable Rust program that authenticates with a tenant scope, fires three intents, waits for completion, and prints a
summary plus the audit-chain hash chain. Copy-paste into a `cargo new` and run against either a loopback dev cell or production.

Crates:
- `oya-application-sdk = "0.42.0"` (re-exports `tokio`, `serde`, `tracing`)
- `oya-trace = "0.42.0"` (W3C Trace Context emitter)
- `anyhow = "1"`
- `clap = { version = "4", features = ["derive"] }`

## `Cargo.toml`

```toml
[package]
name = "dispatch-intent-example"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
clap = { version = "4", features = ["derive"] }
oya-application-sdk = "0.42.0"
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
use clap::Parser;
use oya_application_sdk::{
    ApplicationClient, ApplicationConfig, Intent, IntentKind, Tenant, TierExpectation,
};
use oya_trace::TraceContext;
use serde::Deserialize;
use std::time::Duration;
use tracing::{info, instrument};

#[derive(Parser, Debug)]
#[command(
    name = "dispatch-intent-example",
    about = "Reference dispatcher for the oya-application surface"
)]
struct Args {
    /// Tenant scope, e.g. `oyatie.community.dev-sample`
    #[arg(long)]
    tenant: String,

    /// Service endpoint; defaults to loopback dev cell
    #[arg(long, default_value = "https://loopback.application.oyatie.local")]
    endpoint: String,

    /// Optional Cedar principal id for break-glass / agent flows
    #[arg(long)]
    principal: Option<String>,

    /// Number of intents to fire
    #[arg(long, default_value = "3")]
    count: usize,
}

#[derive(Deserialize, Debug)]
struct DispatchOutcome {
    intent_hash: String,
    cell_id: String,
    dispatch_target: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let trace = TraceContext::new_root();

    let cfg = ApplicationConfig::builder()
        .endpoint(args.endpoint.parse()?)
        .tenant(Tenant::parse(&args.tenant)?)
        .principal(args.principal.clone())
        .tier_expectation(TierExpectation::AtLeasttenant_class demo_trial)
        .request_timeout(Duration::from_secs(10))
        .build()
        .context("invalid application config")?;

    let client = ApplicationClient::connect(cfg).await?;
    info!(tenant = %args.tenant, endpoint = %args.endpoint, "connected");

    let mut outcomes = Vec::with_capacity(args.count);
    for i in 0..args.count {
        let intent = Intent::new(
            IntentKind::parse("application::Intent::CreateWorkspace")?,
            serde_json::json!({
                "name": format!("ref-impl-ws-{i}"),
                "owner_email": format!("alice+{i}@example.com")
            }),
        );
        let outcome: DispatchOutcome = dispatch_one(&client, &trace, intent).await?;
        outcomes.push(outcome);
    }

    info!(?outcomes, "dispatch complete");

    // Walk the audit chain to prove tamper-evidence
    let chain = client.audit_chain_segment(args.count).await?;
    chain.verify_links().context("audit chain link mismatch")?;
    info!(
        first = %chain.first().intent_hash,
        last = %chain.last().intent_hash,
        links = chain.len(),
        "audit chain segment verified"
    );

    Ok(())
}

#[instrument(skip(client, trace))]
async fn dispatch_one(
    client: &ApplicationClient,
    trace: &TraceContext,
    intent: Intent,
) -> Result<DispatchOutcome> {
    let resp = client
        .dispatch(intent, trace.child())
        .await
        .context("dispatch failed")?;
    Ok(DispatchOutcome {
        intent_hash: resp.intent_hash().to_string(),
        cell_id: resp.cell_id().to_string(),
        dispatch_target: resp.dispatch_target().to_string(),
    })
}
```

## Run it

```bash
cargo run -- --tenant oyatie.community.dev-sample --count 3
```

Expected stdout (trimmed):
```
INFO  connected tenant=oyatie.community.dev-sample endpoint=https://loopback.application.oyatie.local
INFO  dispatch_one outcome=DispatchOutcome { intent_hash: "blake3-256:9f3c…", cell_id: "loopback-1", dispatch_target: "workflow-engine" }
INFO  dispatch_one outcome=DispatchOutcome { intent_hash: "blake3-256:a201…", cell_id: "loopback-2", dispatch_target: "workflow-engine" }
INFO  dispatch_one outcome=DispatchOutcome { intent_hash: "blake3-256:7e8d…", cell_id: "loopback-1", dispatch_target: "workflow-engine" }
INFO  audit chain segment verified first=blake3-256:9f3c… last=blake3-256:7e8d… links=3
```

## How the SDK enforces correctness

1. `ApplicationConfig::tenant` is non-optional; you cannot build a client without a tenant.
2. `Intent::new(kind, payload)` validates against the closed `Intent` enum at construction time — typos fail compile (string-typed
   `IntentKind::parse` is a deliberate escape hatch for forward-compat).
3. `ApplicationClient::dispatch` returns `Result<DispatchResponse, ApplicationError>`. `ApplicationError` variants include
   `PermitDenied`, `DeadlineExceeded`, `CellUnavailable`, `PackSetIncoherent`, and `LegacyProtocolForbidden`.
4. `audit_chain_segment(n).verify_links()` re-hashes the BLAKE3 chain locally and returns `LinkMismatch` if any audit row was tampered.
5. `TraceContext::new_root()` + `.child()` propagate W3C traceparent across hops automatically.

## Production-only flags

- `ApplicationConfig::require_mtls(true)` — enforces mTLS; client must present an attested cert from the `kms` µservice.
- `ApplicationConfig::byok_credentials(creds)` — pipe in your own provider key per `feedback_byok_everywhere_credentials.md`.
- `ApplicationConfig::pack_expectation(["pack.soc2-type-ii-v2017"])` — refuse to dispatch unless the named pack is active.

## Tests

```bash
cargo test --features integration
```
The integration suite includes `dispatch_one` against a hermetic loopback cell launched via `oya_application_sdk::testkit::Hermetic`.
