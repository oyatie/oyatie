---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-first-paying-tenant
phase: P01-application-shell-landing
impl_plan_id: IP-012-module-loader-usecase-adapter-cdn
status: pending
execution_unit: ChangeSet
owner: axis-application
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, layer-correctness]
---

# IP-012: module-loader usecase + adapter + adapter-cdn + rest + sdk

## Intent

Combined IP delivering the module-loader functional path:
- usecase: LoadModuleUseCase (fetch manifest → verify SRI + signature → register routes → emit event); PublishManifestUseCase.
- adapter: Postgres ManifestRepository + OpenBao PublisherKeyResolver.
- adapter-cdn: backend-qualified CDN client (OCI CDN HTTP API; Cloudflare overlay).
- rest: handlers per `contracts/openapi/application.yaml` for `/modules/{m}/manifest` GET + POST.
- sdk: client crate for product µservices to publish + register modules.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/application/src/crates/oya-application-module-loader-usecase/{Cargo.toml,src/{lib,load,publish}.rs}` | create |
| `microservices/application/src/crates/oya-application-module-loader-adapter/{Cargo.toml,src/{lib,postgres,openbao}.rs}` | create |
| `microservices/application/src/crates/oya-application-module-loader-adapter-cdn/{Cargo.toml,src/{lib,oci,cloudflare}.rs}` | create |
| `microservices/application/src/crates/oya-application-module-loader-api/{Cargo.toml,src/lib.rs}` | create |
| `microservices/application/src/crates/oya-application-module-loader-rest/{Cargo.toml,src/{lib,router,handlers}.rs}` | create |
| `microservices/application/src/crates/oya-application-module-loader-sdk/{Cargo.toml,src/{lib,client,builder}.rs}` | create |
| 6 × catalog rows | create |
| `Cargo.toml` (workspace) | update |

## Code Shape

```rust
// usecase
pub struct LoadModuleUseCase<R, F, V, K> { repo: R, fetcher: F, verifier: V, keys: K, audit: AuditEmitter }

impl<R, F, V, K> LoadModuleUseCase<R, F, V, K>
where R: ManifestRepository, F: BundleFetcher, V: SignatureVerifier, K: PublisherKeyResolver
{
    pub async fn load(&self, principal: &Principal, module: &str, version: &str)
        -> Result<ModuleManifest, UseCaseError>
    {
        let manifest = self.repo.fetch(module, version).await?;
        let pubkey = self.keys.resolve(&manifest.signer_key_id).await?;
        if pubkey.revoked() {
            self.audit.emit(ModuleLoadRejected::new(principal, module, "signer_revoked"));
            return Err(UseCaseError::SignerRevoked);
        }
        verify_signature(&manifest, &pubkey)?;
        let bundle = self.fetcher.fetch(&manifest.bundle_url).await?;
        if !verify_sri(&bundle, &manifest.sri_hash) {
            self.audit.emit(ModuleLoadRejected::new(principal, module, "sri_mismatch"));
            return Err(UseCaseError::SriMismatch);
        }
        self.audit.emit(ModuleLoaded::new(principal, &manifest));
        Ok(manifest)
    }
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-application-module-loader-usecase --all-features
cargo nextest run -p oya-application-module-loader-adapter --all-features
cargo nextest run -p oya-application-module-loader-adapter-cdn --all-features
cargo nextest run -p oya-application-module-loader-rest --all-features
cargo nextest run -p oya-application-module-loader-sdk --all-features
cargo run -p oya-dev-cli -- gate validate openapi-conformance --crate oya-application-module-loader-rest
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_load_happy_path` | valid manifest + bundle loads |
| `test_load_sri_mismatch_rejected` | T-01 |
| `test_load_signature_invalid_rejected` | T-02 |
| `test_load_signer_revoked_rejected` | R-03 |
| `test_publish_via_sdk` | signed publish round-trip |
| `test_cdn_purge_after_revert` | adapter-cdn purge call |

Coverage: 90 % / 80 %.

## Next IP

[`IP-013-frontend-bundle-serve.md`](IP-013-frontend-bundle-serve.md)
