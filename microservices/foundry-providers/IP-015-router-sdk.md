---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-provider-adapter-substrate
impl_plan_id: IP-015-router-sdk
status: pending
execution_unit: ChangeSet
owner: axis-foundry + gtm
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, layer-correctness]
---

# IP-015: oya-foundry-providers-router-sdk

## Intent

First-party Rust SDK (`-sdk` crate) per `sdk-plan.md` Rust SDK section. Wraps the REST + gRPC surface with: OIDC token provider trait, SPIFFE workload-identity helper, automatic retry w/ backoff, streaming support, idiomatic error handling.

## File Targets

### `oya-foundry-providers-router-sdk`

| Path | Action |
|---|---|
| `.../Cargo.toml` | create — `tonic` (gRPC), `reqwest` (REST), kernel + api deps |
| `.../src/lib.rs` | create |
| `.../src/client.rs` | create — `Client::new(opts)`; `Client::invoke`; `Client::decide`; `Client::invoke_stream` |
| `.../src/auth.rs` | create — OIDC token provider trait + SPIFFE helper |
| `.../src/retry.rs` | create — exponential backoff for 5xx + 429 |
| `.../src/error.rs` | create — public error surface |

## TypeScript SDK scaffold

`microservices/foundry-providers/sdk-generation/typescript/`:

| Path | Action |
|---|---|
| `package.json` | create |
| `tsconfig.json` | create |
| `src/index.ts` | create — main client surface |
| `src/auth.ts` | create — OIDC helper |
| `src/generated/` | create — `openapi-generator-cli` output dir |
| `scripts/generate.sh` | create — runs openapi-generator-cli against `contracts/openapi/provider-router.yaml` |

(Full TS implementation lands at M01+1 per `sdk-plan.md`; this IP scaffolds the layout and CI lane.)

## Client Surface

```rust
pub struct Client {
    rest_endpoint: String,
    grpc_endpoint: String,
    token_provider: Arc<dyn TokenProvider>,
    tenant_id: String,
    pack: String,
}

impl Client {
    pub fn new(opts: ClientOpts) -> Self { /* ... */ }

    pub async fn invoke(&self, req: InvokeRequest) -> Result<InvokeResponse, Error> { /* gRPC primary */ }

    pub async fn decide(&self, req: DecideRequest) -> Result<RouterDecision, Error> { /* gRPC primary */ }

    pub fn invoke_stream(&self, req: InvokeRequest) -> impl Stream<Item = Result<InvokeStreamChunk, Error>> { /* ... */ }

    pub async fn providers_health(&self, filter: HealthFilter) -> Result<Vec<ProviderHealthSnapshot>, Error> { /* ... */ }

    pub async fn list_capabilities(&self) -> Result<Vec<CapabilityProfile>, Error> { /* ... */ }
}
```

## Constraints

- SDK NEVER carries credential bytes; tenants reference `SecretReference` URIs as opaque strings.
- SDK has `#![deny(unsafe_code)]`.
- SDK logs do not emit prompt text by default (debug-trace can opt-in but is off by default).

## Test Plan

| Test | Verifies |
|---|---|
| `test_client_invoke_against_mock_router_rest` | integration |
| `test_client_invoke_against_mock_router_grpc` | integration |
| `test_client_retry_backoff_for_5xx` | retry policy |
| `test_client_retry_honors_retry_after` | spec |
| `test_client_streaming_terminates_on_done_chunk` | streaming |
| `tests/integration/sdk_compatibility_n_minus_1.rs` | compat |
| `test_sdk_does_not_log_prompt_by_default` | privacy invariant |

## Acceptance Gates

Standard + per-language CI lane build + SDK compatibility test (N-1, N, N+1).

## Next IP

(Phase exit gate per PHASE-01.)
