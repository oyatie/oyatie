---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-openbao-secretreference-substrate
impl_plan_id: IP-007-resolver-rest-and-sdk-rust
status: pending
owner: axis-cloud-secrets
acceptance_lanes: [contract-test, sdk-smoke]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: resolver-rest + SDK (Rust)

## Intent

Ship two crates:
1. `-rest` — admin REST endpoints per `contracts/openapi/cloud-secrets.yaml` (NOT the hot path; admin only).
2. `-sdk` — the canonical Rust client SDK; the primary integration surface.

## ChangeSet boundary

Two new crates. SDK is the public surface; REST is admin only.

## Concrete File Targets

| Path | Action |
|---|---|
| `…/oya-cloud-secrets-secret-reference-resolver-rest/Cargo.toml` | create |
| `…/oya-cloud-secrets-secret-reference-resolver-rest/src/lib.rs` | create |
| `…/oya-cloud-secrets-secret-reference-resolver-rest/src/handlers/*.rs` | create — one handler per OpenAPI path |
| `…/oya-cloud-secrets-secret-reference-resolver-sdk/Cargo.toml` | create |
| `…/oya-cloud-secrets-secret-reference-resolver-sdk/src/lib.rs` | create — `SecretClient` |
| `…/src/secret_wrapper.rs` | create — `Secret<T>` newtype with Zeroize + redacted Debug |
| `…/src/with_secret.rs` | create — scoped callback API |
| `microservices/cloud-secrets/catalog/oya-cloud-secrets-secret-reference-resolver-rest.yaml` | create |
| `microservices/cloud-secrets/catalog/oya-cloud-secrets-secret-reference-resolver-sdk.yaml` | create |

## Code Shape

```rust
// SDK public API
pub struct SecretClient { /* … */ }

impl SecretClient {
    pub fn builder() -> SecretClientBuilder { /* … */ }

    pub async fn resolve(&self, reference: &SecretReference) -> Result<Secret<Vec<u8>>, ResolveError>;
    pub async fn with_secret<F, T>(&self, reference: &SecretReference, f: F) -> Result<T, ResolveError>
    where F: FnOnce(&[u8]) -> Result<T, ResolveError>;
    pub async fn subscribe_revocations(&self) -> Result<RevocationStream, RevocationError>;
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-cloud-secrets-secret-reference-resolver-rest
cargo nextest run -p oya-cloud-secrets-secret-reference-resolver-sdk
cargo run -p oya-dev-cli -- gate validate sdk-contract-conformance --sdk-lang rust
```

## Test Plan

- Contract tests against OpenAPI: every handler honours schema.
- SDK smoke: end-to-end resolve via SDK → sandbox OpenBao.
- `Secret<T>` Debug returns `[REDACTED]`.
- `with_secret` callback zeroises after return.

## Halt Conditions

- SDK exposes raw value type — BLOCKER.

## Next IP

`IP-008-sdk-ts-python-bindings.md`
