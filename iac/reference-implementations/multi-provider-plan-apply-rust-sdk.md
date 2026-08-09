# Reference implementation — Multi-provider plan/apply with `oya-cloud-iac-sdk`

Runnable Rust program that authors an in-memory module set, fires a plan, waits for the reviewer-agent decision, applies it,
and reports the audit-chain anchor.

## `Cargo.toml`

```toml
[package]
name = "iac-multiprovider-example"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
oya-cloud-iac-sdk = "0.42.0"
oya-trace = "0.42.0"
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
tokio = { version = "1.43", features = ["macros", "rt-multi-thread"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

## `src/main.rs`

```rust
use anyhow::{Context, Result};
use oya_cloud_iac_sdk::{
    ApplyDecision, IacClient, IacConfig, ModuleSet, PlanInputs, ReviewOutcome, Tenant,
};
use oya_trace::TraceContext;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let trace = TraceContext::new_root();

    let cfg = IacConfig::builder()
        .endpoint("https://loopback.cloud-iac.oyatie.local".parse()?)
        .tenant(Tenant::parse("oyatie.b2b.smb.acme-software")?)
        .module_set(ModuleSet::parse("oya-iac-modules-paid-per-usage-v1")?)
        .request_timeout(Duration::from_secs(30))
        .plan_deadline(Duration::from_secs(120))
        .apply_deadline(Duration::from_secs(360))
        .build()?;

    let client = IacClient::connect(cfg).await?;
    info!("connected to cloud-iac");

    let inputs_yaml = include_str!("../inputs/acme-website-stack.yaml");
    let inputs: PlanInputs = serde_yaml::from_str(inputs_yaml).context("inputs parse")?;

    // 1. Plan
    let plan = client.plan(inputs, trace.child()).await?;
    info!(
        plan_id = %plan.id(),
        resources_to_add = plan.resources_to_add(),
        resources_to_change = plan.resources_to_change(),
        resources_to_destroy = plan.resources_to_destroy(),
        graph_signature = %plan.graph_signature(),
        "plan complete"
    );

    if plan.is_empty() {
        info!("empty plan; nothing to apply");
        return Ok(());
    }

    // 2. Wait for reviewer-agent decision
    let mut backoff = Duration::from_millis(500);
    let review = loop {
        match client.review_status(plan.id()).await? {
            ReviewOutcome::Pending => {
                sleep(backoff).await;
                backoff = (backoff * 2).min(Duration::from_secs(10));
            }
            outcome => break outcome,
        }
    };

    match review {
        ReviewOutcome::Approve { facets_passed } => {
            info!(facets_passed, "reviewer-agent APPROVED");
        }
        ReviewOutcome::Block { facet_failures } => {
            warn!(?facet_failures, "reviewer-agent BLOCKED; aborting");
            return Ok(());
        }
        ReviewOutcome::NeedsHuman { reason } => {
            warn!(reason, "reviewer-agent escalated to human; aborting auto-flow");
            return Ok(());
        }
        ReviewOutcome::Pending => unreachable!(),
    }

    // 3. Apply
    let apply = client.apply(plan.id(), trace.child()).await?;
    match apply.decision() {
        ApplyDecision::Success => {
            info!(
                apply_id = %apply.id(),
                duration_ms = apply.duration().as_millis() as u64,
                audit_chain_event_id = %apply.audit_chain_event_id(),
                "apply success"
            );
        }
        ApplyDecision::PartialFailure { rolled_back } => {
            warn!(rolled_back, "apply partial failure; runner auto-rolled-back");
        }
        ApplyDecision::Failure { reason } => {
            warn!(reason, "apply failed; nothing changed");
        }
    }

    Ok(())
}
```

## `inputs/acme-website-stack.yaml`

```yaml
module_set: oya-iac-modules-paid-per-usage-v1
modules:
  - name: aws-s3-static-site
    inputs:
      bucket_name: acme-software-website-prod
      region: us-east-2
      acl: private
  - name: aws-cloudfront-distribution
    inputs:
      origin: ${module.aws-s3-static-site.bucket_regional_domain_name}
      aliases: ["www.acme-software.io"]
      certificate_arn: ${ref:cloud_secrets.acme_software_io_cert_arn}
  - name: cloudflare-zone
    inputs:
      zone_name: acme-software.io
      plan: free
  - name: cloudflare-record
    inputs:
      zone_id: ${module.cloudflare-zone.zone_id}
      name: www
      type: CNAME
      value: ${module.aws-cloudfront-distribution.domain_name}
      proxied: true
```

## Run it

```bash
cargo run --release
```

Expected output (trimmed):
```
INFO  connected to cloud-iac
INFO  plan complete plan_id=plan-… resources_to_add=4 resources_to_change=0 resources_to_destroy=0 graph_signature=blake3-256:5b4a…
INFO  reviewer-agent APPROVED facets_passed=11
INFO  apply success apply_id=apply-… duration_ms=15870 audit_chain_event_id=ce-2026-…
```

## SDK correctness guarantees

1. `PlanInputs` is strict-deserialised; unknown fields are rejected (Pulumi-style YAML laxity is **not** allowed).
2. `client.plan()` returns `PlanError::ModuleNotInSet` if any referenced module is outside the tenant's subscribed set.
3. `client.review_status()` is the only legitimate way to check the gate; polling is exponential-backoff.
4. `client.apply()` is idempotent on `plan_id` — retries do not double-apply.
5. `apply.audit_chain_event_id()` is the canonical chain anchor for compliance evidence.

## Tests

```bash
cargo test --features hermetic
```

The hermetic feature uses `oya_cloud_iac_sdk::testkit::Hermetic` to spin a single-process loopback cell with mock providers; tests
finish in ≤ 60 s.
