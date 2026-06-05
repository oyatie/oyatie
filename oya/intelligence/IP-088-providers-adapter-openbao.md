---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-provider-adapter-substrate
impl_plan_id: IP-013-adapter-openbao
status: pending
execution_unit: ChangeSet
owner: axis-foundry + ops-security (2-person rule on publish per CI-INV-09)
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, credential-isolation, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-013: oya-foundry-providers-adapter-openbao

## Intent

The OpenBao SecretReference resolver. Reads `openbao://<pack>/<tenant>/providers/<vendor>/<credential-name>` URIs via the local OpenBao agent socket; returns a `ResolvedCredential` opaque value that crosses no FFI boundary as raw bytes and zeroises on drop.

This is the **most security-sensitive** crate in the µservice; CI-INV-01..CI-INV-10 all touch it. 2-person review mandatory on every change.

## File Targets

| Path | Action |
|---|---|
| `.../Cargo.toml` | create — `zeroize`, `tokio`, `serde`, OpenBao agent client deps |
| `.../src/lib.rs` | create |
| `.../src/resolver.rs` | create — `OpenBaoResolver` impl of `CredentialResolver` |
| `.../src/resolved_credential.rs` | create — opaque type w/ Debug=REDACTED + Drop=zeroize |
| `.../src/lease_cache.rs` | create — per-tenant lease cache w/ TTL |
| `.../src/audit_emitter.rs` | create — emits `CredentialResolved` event |

## Core Type

```rust
// resolved_credential.rs
use zeroize::Zeroize;

pub struct ResolvedCredential {
    inner: SecretBytes,
    vendor: Vendor,
    lease_id: String,
}

#[derive(Zeroize)]
#[zeroize(drop)]
struct SecretBytes(Vec<u8>);

impl std::fmt::Debug for ResolvedCredential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ResolvedCredential {{ vendor: {:?}, redacted: true }}", self.vendor)
    }
}
// NO Display impl.
// NO Serialize impl.
// NO Deserialize impl.
// NO Clone impl (per-call lifetime).

impl ResolvedCredential {
    pub fn with_credential<R, F>(&self, f: F) -> R
    where F: FnOnce(&[u8]) -> R {
        f(&self.inner.0)
    }
}
```

## Resolver

```rust
#[async_trait]
impl CredentialResolver for OpenBaoResolver {
    type Credential = ResolvedCredential;
    async fn resolve(&self, secret_ref: &SecretReference, caller_ctx: &CallerCtx)
        -> Result<ResolvedCredential, ResolverError>
    {
        // CI-INV-05: token-tightening — agent token only has read on providers/*
        let agent_response = self.agent_client.read(&secret_ref.uri).await?;
        let credential_bytes = SecretBytes(agent_response.data);
        let lease_id = agent_response.lease_id;

        self.audit_emitter.emit_credential_resolved(secret_ref, &lease_id, caller_ctx).await?;

        Ok(ResolvedCredential {
            inner: credential_bytes,
            vendor: secret_ref.vendor()?,
            lease_id,
        })
    }
}
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_resolved_credential_debug_is_redacted` | CI-INV-02 (Debug emits REDACTED) |
| `test_resolved_credential_no_serialize_impl` | compile-fails-test asserts no Serialize |
| `test_resolved_credential_drop_zeroises` | CI-INV-02 (Drop zeroizes) |
| `test_with_credential_closure_accessor` | CI-INV-02 |
| `tests/integration/openbao_lease_lifecycle.rs` | CI-INV-04 |
| `tests/integration/openbao_resolve_emits_audit` | CI-INV-10 |
| `tests/integration/rotation_zero_downtime.rs` | CI-INV-07 |
| `tests/integration/cross_tenant_resolve_denied.rs` | Cedar deny |
| `test_resolver_handles_lease_expiry_re_fetch` | TTL behaviour |

## Acceptance Gates

```bash
buck2 build //:quality-lane-registry-authority-check # lane=credential-isolation --crate oya-foundry-providers-adapter-openbao
```

This crate's PR REQUIRES 2-person review per CI-INV-09 (CODEOWNERS rule).

## Next IP

[`IP-014-router-rest-worker-app.md`](IP-014-router-rest-worker-app.md)

## Wave 15 counterpart anchor

- Counterparts: OpenAI API, Anthropic API, Google Vertex Model Garden, LiteLLM, and OpenRouter.
- Gap closure: this IP closes provider-neutral routing, credential isolation, API/subscription adapters, and provider health/cost decisions.
- Evidence source: `microservices/intelligence/competitor-parity-matrix.md` plus the BC-local parity archive under `microservices/intelligence/bc-sources/` when present.
