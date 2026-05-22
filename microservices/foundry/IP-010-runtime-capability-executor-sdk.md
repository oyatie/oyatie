---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agent-runtime-and-capability-execution
impl_plan_id: IP-010-capability-executor-sdk
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-runtime
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-010: oya-foundry-runtime-capability-executor-sdk

## Intent

First-party Rust SDK per `sdk-plan.md`. Public surface: `Client::new(opts); client.dispatch(...); client.stream_invocation(...); client.get_session(...); client.get_autonomy_ceiling(...)`. Built-in OIDC token provider abstraction; tenant-context binding at construction; retry policy with exponential backoff for 5xx/429/503; idempotency-key auto-generation if not supplied.

## ChangeSet boundary

One new Rust crate.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-foundry-runtime-capability-executor-sdk/Cargo.toml` | create |
| `.../src/lib.rs` | create |
| `.../src/client.rs` | create (Client struct + builder) |
| `.../src/auth.rs` | create (OIDC token provider trait) |
| `.../src/retry.rs` | create (exponential backoff) |
| `.../src/streaming.rs` | create (gRPC streaming wrapper) |
| `.../src/errors.rs` | create (SDK-specific error variants) |
| `.../examples/dispatch_simple.rs` | create |
| `.../examples/stream_invocation.rs` | create |
| `.../README.md` | create (quick-start) |

## Crate Naming

```
NAME: oya-foundry-runtime-capability-executor-sdk
JUSTIFICATION:
- microservice = foundry-runtime; bc-tokens = capability-executor
- layer = sdk per ADR-0105 (client library)
- exemptions claimed: none
```

## Code Shape

```rust
// src/client.rs
pub struct Client {
    inner: reqwest::Client,
    base_url: Url,
    auth: Box<dyn OidcTokenProvider>,
    tenant_id: String,
    retry: RetryPolicy,
}

impl Client {
    pub fn builder() -> ClientBuilder { ClientBuilder::new() }

    pub async fn dispatch(
        &self,
        capability_id: &str,
        input: serde_json::Value,
        opts: DispatchOptions,
    ) -> Result<Invocation, SdkError> {
        let body = DispatchRequest {
            capability_id: capability_id.into(),
            input,
            autonomy_level_declared: opts.autonomy_level_declared,
            timeout_seconds: opts.timeout_seconds,
            idempotency_key: opts.idempotency_key.unwrap_or_else(generate_idempotency_key),
            ..Default::default()
        };
        let url = self.base_url.join(&format!("/api/v1/capabilities/{capability_id}/dispatch"))?;
        let response = self.retry.execute(|| async {
            self.inner.post(url.clone())
                .bearer_auth(self.auth.token().await?)
                .header("X-Scope-OrgID", &self.tenant_id)
                .header("Idempotency-Key", &body.idempotency_key)
                .json(&body)
                .send().await
        }).await?;
        Ok(response.json::<Invocation>().await?)
    }

    pub fn stream_invocation(
        &self,
        invocation_id: &str,
    ) -> impl Stream<Item = Result<Invocation, SdkError>> {
        // gRPC streaming wrapper per proto contract
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-foundry-runtime-capability-executor-sdk --all-features
cargo nextest run -p oya-foundry-runtime-capability-executor-sdk --all-features
cargo doc -p oya-foundry-runtime-capability-executor-sdk --no-deps
cargo test --example dispatch_simple --features test-against-staging
```

## Test Plan

Per PHASE-01 sdk class: 1 test per public method (happy + retry + auth-fail) + ≥2 against rest crate. 90% line / 80% branch.

| Test | Verifies |
|---|---|
| `test_dispatch_happy_path_against_rest` | end-to-end through rest crate (in-process) |
| `test_dispatch_retry_on_503` | retry policy correctness |
| `test_dispatch_auth_fail_returns_unauthorized` | OIDC token expired surfaced as SdkError::Unauthenticated |
| `test_stream_invocation_until_terminal` | streaming until Completed/Failed/Cancelled |
| `test_idempotency_key_auto_generated` | unique key per call when not supplied |
| `test_tenant_context_immutable_after_construction` | client cannot be re-pointed to different tenant |

## Halt Conditions

- `unsafe` code present — refactor (`#![deny(unsafe_code)]`).
- Tenant-binding mutable after construction — refactor (security risk).

## Next IP

[`IP-011-capability-executor-app.md`](IP-011-capability-executor-app.md)

## References

- `sdk-plan.md`.
- `contracts/openapi/foundry-runtime.yaml`.
- `contracts/proto/foundry-runtime.proto`.

## Wave 15 counterpart anchor

- Counterparts: OpenAI Assistants, AWS Bedrock Agents, and Cloudflare Workers sandboxing.
- Gap closure: this IP closes session/run execution, capability isolation, and sandbox accounting with Oyatie tenant, Cedar, and evidence-chain controls.
- Evidence source: `microservices/foundry/competitor-parity-matrix.md` plus the BC-local parity archive under `microservices/foundry/bc-sources/` when present.
