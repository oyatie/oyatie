---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-guardrails-safety-and-policy-enforcement
impl_plan_id: IP-015-sdk-rust-and-typescript
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-guardrails + gtm
acceptance_lanes: [cargo-check, cargo-nextest, npm-build, sdk-compatibility-matrix]
---

# IP-015: SDK — Rust first-party + TypeScript via OpenAPI generator

## Intent

Author `oya-foundry-guardrails-prompt-classifier-sdk` (Rust; first-party) + scaffold TypeScript SDK via `openapi-generator-cli` 7.x. Per `sdk-plan.md` Rust is M01; TS is M01+1 but pipeline lands now. Companion SDK crates for each BC `-sdk` ship as part of this IP for the canonical Rust surface.

## ChangeSet boundary

One Rust SDK crate per BC (× 6) + TS-generation pipeline configuration. Per-language CI lanes added per `sdk-plan.md`.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-foundry-guardrails-<bc>-sdk/Cargo.toml` + `src/{lib.rs,client.rs,retry.rs}` (× 6 BCs) | create |
| `sdk-generation/typescript/openapi-generator.yaml` | create |
| `sdk-generation/typescript/package.json` | create |
| `sdk-generation/typescript/src/wrappers/` | create (hand-authored ergonomic wrapper layer) |
| `sdk-generation/Makefile` | create — `make typescript` + `make python` + `make go` (TS only enabled at M01+1; others scaffolded) |
| `Cargo.toml` workspace | update |
| `catalog/oya-foundry-guardrails-<bc>-sdk.yaml` | create (× 6) |

## Code Shape

```rust
// prompt-classifier-sdk/src/client.rs
pub struct Client {
    http: reqwest::Client,
    base_url: url::Url,
    token_provider: Arc<dyn TokenProvider>,
    tenant_id: TenantId,
    retry: RetryConfig,
}

impl Client {
    pub fn new(opts: ClientOpts) -> Result<Self, ClientError> { ... }

    pub async fn classify_prompt(&self, prompt: &str, ctx: &ClassifyCtx) -> Result<Classification, ClientError> {
        let token = self.token_provider.fetch().await?;
        let resp = retry_with_backoff(&self.retry, || async {
            self.http
                .post(self.base_url.join("/v1/classify-prompt")?)
                .header("X-Scope-OrgID", self.tenant_id.x_scope())
                .bearer_auth(token.expose())
                .json(&ClassifyPromptRequest::from_prompt(prompt, ctx))
                .send().await
        }).await?;
        Ok(resp.json().await?)
    }

    pub fn stream_decisions(&self, filter: DecisionFilter) -> impl Stream<Item = Result<GuardrailDecision, ClientError>> {
        // gRPC streaming
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-foundry-guardrails-<bc>-sdk (× 6) --all-features
cargo nextest run -p oya-foundry-guardrails-<bc>-sdk (× 6) --all-features --test integration
make -C microservices/foundry/sdk-generation typescript-validate
cargo run -p oya-dev-cli -- gate validate sdk-compatibility-matrix --microservice foundry-guardrails
```

## Test Plan

Per sdk class: 1 per public client method (happy + retry + auth-fail) + ≥ 2 against rest crate + 0 e2e. Coverage 90% / 80%.

| Test | Verifies |
|---|---|
| `test_classify_prompt_happy` | matches REST contract |
| `test_classify_prompt_retry_on_5xx` | exponential backoff |
| `test_classify_prompt_auth_fail` | 401 returned cleanly |
| `test_tenant_binding` | X-Scope-OrgID populated |
| `test_typescript_generation` | TS SDK generated + lints |

## Halt Conditions

- Generated TS does not pass lint — escalate generator config.
- Rust SDK lacks retry / auth-error surface — refactor.

## Next IP

End of phase. Promotion-readiness gate must be green: HG-FGUARD + coupling lane + all µservice lanes.

## References

- `sdk-plan.md`.
- `contracts/openapi/guardrails.yaml`.
- `contracts/proto/guardrails.proto`.
- ADR-0105 (sdk layer canonical).
- OpenAPI Generator — `openapi-generator.tech`.
- Stripe SDK precedent.
- Twilio SDK precedent.
