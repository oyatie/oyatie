---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-first-paying-tenant
phase: P01-application-shell-landing
impl_plan_id: IP-011-module-loader-kernel-domain
status: pending
execution_unit: ChangeSet
owner: axis-application
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, port-location, layer-correctness, data-class]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-011: module-loader kernel + domain

## Intent

Kernel: port traits (ManifestRepository, BundleFetcher, SignatureVerifier,
PublisherKeyResolver) + entities (Module, ModuleManifest, IntegrityClaim,
SignerKey). Domain: SRI hash verification (sha384), Ed25519 signature
verification, canonical CBOR encoding for signing.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/application/src/crates/oya-application-module-loader-kernel/{Cargo.toml,src/{lib,entities,ports,errors}.rs}` | create |
| `microservices/application/src/crates/oya-application-module-loader-domain/{Cargo.toml,src/{lib,sri,signature,canonical}.rs}` | create |
| 2 × catalog rows | create |
| `Cargo.toml` (workspace) | update |

## Code Shape

```rust
#[derive(Clone, Debug)]
pub struct ModuleManifest {
    #[data_class(INTERNAL_ONLY)] pub module: String,
    #[data_class(INTERNAL_ONLY)] pub version: String,
    #[data_class(INTERNAL_ONLY)] pub sri_hash: String,      // sha384-<base64>
    #[data_class(INTERNAL_ONLY)] pub signer_key_id: String,
    #[data_class(INTERNAL_ONLY)] pub signature: String,     // base64url Ed25519
    #[data_class(INTERNAL_ONLY)] pub routes: Vec<RouteRegistration>,
    #[data_class(INTERNAL_ONLY)] pub bundle_url: String,
    #[data_class(AUDIT)] pub published_at: chrono::DateTime<chrono::Utc>,
}

// domain
pub fn verify_sri(content: &[u8], expected: &str) -> bool {
    let hash = Sha384::digest(content);
    let actual = format!("sha384-{}", base64::encode(hash));
    constant_time_eq(actual.as_bytes(), expected.as_bytes())
}

pub fn verify_signature(manifest: &ModuleManifest, pubkey: &VerifyingKey) -> Result<(), SignatureError> {
    let body = canonical_cbor_encode(manifest)?;
    let sig = base64url_decode(&manifest.signature)?;
    pubkey.verify(&body, &Signature::from_slice(&sig)?)?;
    Ok(())
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-application-module-loader-kernel --all-features
cargo nextest run -p oya-application-module-loader-domain --all-features
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_sri_mismatch_rejected` | flip 1 bit → reject |
| `test_sri_constant_time` | timing channel absent |
| `test_signature_invalid_rejected` | bad sig → reject |
| `test_signature_signer_mismatch_rejected` | manifest claims one signer; verify uses another |
| `test_canonical_cbor_deterministic` | same manifest → same bytes |
| `test_data_class_complete` | every field annotated |

Coverage: 95 % / 90 %.

## Next IP

[`IP-012-module-loader-usecase-adapter-cdn.md`](IP-012-module-loader-usecase-adapter-cdn.md)
