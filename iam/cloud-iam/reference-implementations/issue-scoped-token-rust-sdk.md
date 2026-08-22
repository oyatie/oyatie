# Reference implementation — Issue + introspect a scoped token with `cloud-iam-sdk`

Runnable Rust program that authenticates a service account, issues a scoped token with fingerprint binding, exercises the token
against the Cedar evaluator, and verifies the audit-chain anchor.

## `Cargo.toml`

```toml
[package]
name = "iam-scoped-token-example"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
cloud-iam-sdk = "0.42.0"
trace = "0.42.0"
serde = { version = "1", features = ["derive"] }
tokio = { version = "1.43", features = ["macros", "rt-multi-thread"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

## `src/main.rs`

```rust
use anyhow::{Context, Result};
use cloud_iam_sdk::{
    AuthorizeDecision, IamClient, IamConfig, PrincipalUid, ResourceUid, ScopedTokenRequest, Tenant,
    TokenBinding,
};
use trace::TraceContext;
use std::time::Duration;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let trace = TraceContext::new_root();

    let cfg = IamConfig::builder()
        .endpoint("https://loopback.cloud-iam.oyatie.local".parse()?)
        .tenant(Tenant::parse("oyatie.b2b.smb.acme-software")?)
        .service_account_credentials_path("/etc/oya/iam/sa-creds.json")
        .request_timeout(Duration::from_secs(5))
        .authorize_deadline(Duration::from_millis(50)) // very strict — Cedar should respond ≤ 3 ms
        .build()?;

    let client = IamClient::connect(cfg).await?;
    info!("connected to cloud-iam");

    // 1. Issue a scoped token for an end-user
    let principal = PrincipalUid::parse(
        "User::\"oyatie.b2b.smb.acme-software/alice@acme-software.io\"",
    )?;
    let fingerprint = client.session_fingerprint().await?; // bound to the calling process

    let token_req = ScopedTokenRequest::builder()
        .principal(principal.clone())
        .scopes(vec!["read:tasks".to_string(), "read:projects".to_string()])
        .ttl(Duration::from_secs(3600))
        .binding(TokenBinding::Fingerprint(fingerprint))
        .reason("user-clicked-login-on-web-app")
        .build()?;

    let token = client.issue_token(token_req, trace.child()).await?;
    info!(
        token_id = %token.id(),
        principal = %token.principal(),
        expires_at = %token.expires_at(),
        scope_count = token.scopes().len(),
        "token issued"
    );

    // 2. Exercise the token against the Cedar evaluator
    let resource = ResourceUid::parse(
        "Application::\"oyatie.b2b.smb.acme-software/tasks\"",
    )?;
    let decision = client
        .authorize_with_token(
            token.bearer(),
            "read",
            &resource,
            serde_json::json!({
                "session": {
                    "mfa_verified": true,
                    "federation_idp": "okta-workforce"
                }
            }),
            trace.child(),
        )
        .await?;

    match decision {
        AuthorizeDecision::Allow {
            determining_policies,
            eval_micros,
            audit_chain_event_id,
        } => {
            info!(
                ?determining_policies,
                eval_micros,
                %audit_chain_event_id,
                "authorize allow"
            );
        }
        AuthorizeDecision::Deny { reasons, eval_micros } => {
            warn!(?reasons, eval_micros, "authorize deny");
            return Ok(());
        }
    }

    // 3. Introspect the token (e.g. for a downstream resource server)
    let introspected = client.introspect_token(token.bearer(), trace.child()).await?;
    info!(
        active = introspected.active(),
        principal = %introspected.principal(),
        bound = introspected.binding().is_some(),
        "token introspected"
    );

    // 4. Revoke after use
    client
        .revoke_token(token.bearer(), "example-complete", trace.child())
        .await
        .context("revoke")?;
    info!("token revoked");

    // 5. Verify the audit-chain anchor
    let anchor = client.audit_chain_anchor_for(token.id(), trace.child()).await?;
    info!(
        chain_root = %anchor.root(),
        signed_by = %anchor.hsm_key_id(),
        range = ?anchor.range(),
        "audit-chain anchor verified"
    );

    Ok(())
}
```

## Run it

```bash
cargo run --release
```

Expected output (trimmed):
```
INFO  connected to cloud-iam
INFO  token issued token_id=tok-… principal=User::"oyatie.b2b.smb.acme-software/alice@acme-software.io" expires_at=2026-05-20T11:07:33Z scope_count=2
INFO  authorize allow determining_policies=["acme-software/tasks-read-allow"] eval_micros=174 audit_chain_event_id=ce-2026-…
INFO  token introspected active=true principal=User::"oyatie.b2b.smb.acme-software/alice@acme-software.io" bound=true
INFO  token revoked
INFO  audit-chain anchor verified chain_root=blake3-256:… signed_by=hsm-key-08 range=ts-2026-05-20T10:00…ts-2026-05-20T11:00
```

## SDK correctness guarantees

1. `IamConfig::authorize_deadline` is strict; the SDK returns `IamError::DeadlineExceeded` rather than block longer (Cedar should
   never exceed 5 ms on paid tenant_class).
2. `ScopedTokenRequest::binding` is mandatory; tokens with no binding are refused by `cloud-iam` (paid tenant_class enforce
   `lean-a-token-binding`).
3. `authorize_with_token` is idempotent — replaying the same `(token, action, resource, context)` returns the same decision
   from the entity-store snapshot at `token.issued_at`.
4. `revoke_token` propagates within ≤ 80 ms p95 across runners; calling `introspect_token` after revoke returns `active: false`.
5. `audit_chain_anchor_for(token_id)` returns the BLAKE3 anchor + HSM signature for the hour containing the token's issuance —
   the canonical compliance evidence pointer.

## Tests

```bash
cargo test --features hermetic
```

The `hermetic` feature uses `cloud_iam_sdk::testkit::Hermetic` to spin a single-process loopback `cloud-iam` cell with an
in-memory Cedar entity store; tests finish in ≤ 30 s.

## Error budget

If the SDK returns `IamError::CedarEvalSlow { took_micros }`, treat as a Tier SLO breach signal — file a
`cloud_iam.slo.cedar_eval_slow` event so the observability lane picks it up; do not retry, retry will not help (cache miss).
