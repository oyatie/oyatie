# Reference implementation — Foundry claim/work/done/verify/promote protocol in Rust

A runnable Rust program that demonstrates the full Foundry protocol from inside an agent. This is the canonical pattern an
autonomous agent uses to land work without operator intervention.

## `Cargo.toml`

```toml
[package]
name = "foundry-agent-example"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
oya-foundry-sdk = "0.42.0"
oya-trace = "0.42.0"
serde = { version = "1", features = ["derive"] }
tokio = { version = "1.43", features = ["macros", "rt-multi-thread"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

## `src/main.rs`

```rust
use anyhow::{Context, Result};
use oya_foundry_sdk::{
    AgentIdentity, ClaimSpec, FoundryClient, FoundryConfig, IntentDescriptor, PromoteEnvironment,
    PromoteSpec, Scope, VerifyEvidence,
};
use oya_trace::TraceContext;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let trace = TraceContext::new_root();

    // Connect with the agent identity (typically populated by the agent harness)
    let cfg = FoundryConfig::builder()
        .endpoint("https://loopback.foundry.oyatie.local".parse()?)
        .agent_identity(AgentIdentity::parse("agent.example.reference-impl")?)
        .request_timeout(Duration::from_secs(10))
        .build()?;

    let client = FoundryClient::connect(cfg).await?;
    info!("connected to foundry");

    let intent = IntentDescriptor::new(
        "agent-example-reference-impl-2026-05-20",
        "Demonstrate the full claim protocol",
    );

    // 1. claim
    let claim = client
        .claim(ClaimSpec {
            intent: intent.clone(),
            scopes: vec![
                Scope::path("docs/architecture/notes"),
                Scope::path("microservices/foundry/scratch"),
            ],
        }, trace.child())
        .await
        .context("claim failed")?;
    info!(claim_id = %claim.id(), "claim accepted");

    // 2. work — actually edit files
    let scratch_dir = PathBuf::from("microservices/foundry/scratch");
    fs::create_dir_all(&scratch_dir).await?;
    fs::write(
        scratch_dir.join("agent-example.json"),
        r#"{"actor":"agent.example","status":"in-progress"}"#,
    )
    .await?;
    info!("work: wrote scratch file");

    // 3. verify
    let verify = client
        .verify(VerifyEvidence::builder()
            .add_metric("files_modified", 1.0)
            .add_lane_outcome("lean-a5-doc-coverage", true)
            .add_lane_outcome("lean-a4-secret-cleartext", true)
            .build(),
            &claim,
            trace.child())
        .await
        .context("verify failed")?;
    match verify.outcome() {
        oya_foundry_sdk::VerifyOutcome::Green { lanes_passed } => {
            info!(lanes_passed, "verify green");
        }
        oya_foundry_sdk::VerifyOutcome::Yellow { lanes_failed } => {
            warn!(?lanes_failed, "verify yellow; not fatal but worth checking");
        }
        oya_foundry_sdk::VerifyOutcome::Red { lanes_failed } => {
            warn!(?lanes_failed, "verify red; aborting");
            client.release_claim(&claim, trace.child()).await.ok();
            return Ok(());
        }
    }

    // 4. done
    let done = client
        .done(&claim,
              VerifyEvidence::builder()
                  .add_metric("files_modified", 1.0)
                  .add_lane_outcome("lean-a5-doc-coverage", true)
                  .add_lane_outcome("lean-a4-secret-cleartext", true)
                  .add_metric("verify", 1.0)
                  .build(),
              trace.child())
        .await
        .context("done failed")?;
    info!(audit_event_id = %done.audit_event_id(), "claim closed");

    // 5. open PR (use the SDK's helper which calls `gh pr create` semantics under Cedar)
    let pr = client
        .open_pull_request("dev", "agent-example: reference impl walkthrough", trace.child())
        .await?;
    info!(pr_url = %pr.url(), "pr opened");

    // 6. wait for admission gate
    let admit = client.wait_for_admission(&pr, Duration::from_secs(15 * 60), trace.child()).await?;
    match admit {
        oya_foundry_sdk::AdmissionOutcome::Merged { sha } => {
            info!(merged_sha = %sha, "merged into dev");
        }
        oya_foundry_sdk::AdmissionOutcome::Blocked { reasons } => {
            warn!(?reasons, "admission blocked; aborting");
            return Ok(());
        }
        oya_foundry_sdk::AdmissionOutcome::TimedOut => {
            warn!("admission timed out");
            return Ok(());
        }
    }

    // 7. promote to dev
    let promote = client
        .promote(PromoteSpec {
            bundle_id: "agent-example-reference-impl-bundle-2026-05-20".into(),
            environment: PromoteEnvironment::Dev,
            evidence: VerifyEvidence::builder()
                .add_metric("merged", 1.0)
                .add_lane_outcome("admission", true)
                .build(),
        }, trace.child())
        .await?;
    info!(
        bundle = %promote.bundle_id(),
        env = ?promote.environment(),
        audit_event = %promote.audit_event_id(),
        "promoted"
    );

    Ok(())
}
```

## Run it

```bash
cargo run --release
```

Expected stdout (trimmed):
```
INFO  connected to foundry
INFO  claim accepted claim_id=cl-…
INFO  work: wrote scratch file
INFO  verify green lanes_passed=2
INFO  claim closed audit_event_id=ce-…
INFO  pr opened pr_url=https://github.com/oyatie/oyatie/pull/19873
INFO  merged into dev merged_sha=9f3c4a7…
INFO  promoted bundle=agent-example-… env=Dev audit_event=ce-…
```

## Correctness guarantees

1. `AgentIdentity` is mandatory; anonymous calls are refused (Foundry is principal-aware).
2. `ClaimSpec.scopes` is a closed `Vec<Scope>`; you cannot pass arbitrary strings — only paths, glob-patterns, and crate IDs.
3. `verify` runs validator lanes server-side; client-side caches the result for 60 s.
4. `done` is idempotent on `(agent, claim_id)`; double-`done` returns the same audit event.
5. `wait_for_admission` polls with exponential backoff up to the deadline.
6. `promote` is Cedar-gated separately per environment; promoting to `staging` from a `dev`-only principal returns `PermitDenied`.

## Tests

```bash
cargo test --features hermetic
```

The hermetic feature runs against a single-process loopback Foundry cell. Tests in CI use the same SDK.
