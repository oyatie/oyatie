---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-studio-preview
phase: P01-visual-authoring-substrate
impl_plan_id: IP-007-node-library-registry-full
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow
acceptance_lanes: [cargo-check, cargo-nextest, helm-lint, oya-governance-node-library-signature-verification, oya-governance-node-library-determinism]
depends_on: [IP-001]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: node-library-registry — kernel through app

## Intent

Author all 9 layers of the `node-library-registry` BC: per-pack signed node libraries distributed via CDN, signature verification at every Studio open, determinism enforced (3x re-load byte-identical), revocation propagation ≤ 60s. Foundation for FR-11 (per-pack node libraries) + threat-model T-S-04 (publisher impersonation) + T-T-03 (CDN binary tampering).

## ChangeSet boundary

Nine crates:
- `oya-workflow-studio-node-library-registry-{kernel,domain,usecase,api,adapter,adapter-cdn,rest,sdk,app}`

Per ADR-0105 Amendment 3 backend-qualified `adapter-cdn`.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-workflow-studio-node-library-registry-kernel/{Cargo.toml,src/{lib.rs,entities.rs,ports.rs}}` | create |
| `src/crates/oya-workflow-studio-node-library-registry-domain/{Cargo.toml,src/{lib.rs,signature.rs,determinism.rs},tests/{determinism.rs,signature.rs}}` | create |
| `src/crates/oya-workflow-studio-node-library-registry-usecase/{Cargo.toml,src/{lib.rs,orchestrator.rs}}` | create |
| `src/crates/oya-workflow-studio-node-library-registry-api/{Cargo.toml,src/{lib.rs,contracts.rs}}` | create |
| `src/crates/oya-workflow-studio-node-library-registry-adapter/{Cargo.toml,src/{lib.rs,impl.rs}}` | create |
| `src/crates/oya-workflow-studio-node-library-registry-adapter-cdn/{Cargo.toml,src/{lib.rs,cdn_impl.rs}}` | create |
| `src/crates/oya-workflow-studio-node-library-registry-rest/{Cargo.toml,src/{lib.rs,routes.rs,main.rs}}` | create |
| `src/crates/oya-workflow-studio-node-library-registry-sdk/{Cargo.toml,src/{lib.rs,client.rs}}` | create |
| `src/crates/oya-workflow-studio-node-library-registry-app/{Cargo.toml,src/main.rs}` | create |
| `microservices/workflow-studio/catalog/oya-workflow-studio-node-library-registry-*.yaml` | create | 9 catalog records |
| `microservices/workflow-studio/iac/terraform/node-library-publishers.tf` | update | wired into this IP |

## Code Shape

`node-library-registry-domain/src/signature.rs`:

```rust
use ed25519_dalek::{Verifier, VerifyingKey, Signature};

pub fn verify_library_signature(
    library_bytes: &[u8],
    signature_b64: &str,
    public_key_b64: &str,
) -> Result<(), SignatureError> {
    let signature = decode_signature(signature_b64)?;
    let pk = decode_public_key(public_key_b64)?;
    pk.verify(library_bytes, &signature)
        .map_err(|_| SignatureError::Invalid)
}

#[derive(thiserror::Error, Debug)]
pub enum SignatureError {
    #[error("invalid signature")] Invalid,
    #[error("malformed signature")] Malformed,
    #[error("malformed public key")] MalformedPublicKey,
}
```

`node-library-registry-domain/tests/determinism.rs`:

```rust
#[test]
fn test_load_determinism_3x() {
    let lib_path = "tests/fixtures/sample-library.signed.json";
    let bytes = std::fs::read(lib_path).unwrap();
    let load1 = oya_workflow_studio_node_library_registry_domain::load_descriptor(&bytes).unwrap();
    let load2 = oya_workflow_studio_node_library_registry_domain::load_descriptor(&bytes).unwrap();
    let load3 = oya_workflow_studio_node_library_registry_domain::load_descriptor(&bytes).unwrap();
    assert_eq!(load1, load2);
    assert_eq!(load2, load3);
}
```

## Acceptance Gates

```bash
cargo check -p oya-workflow-studio-node-library-registry-kernel \
  -p oya-workflow-studio-node-library-registry-domain \
  -p oya-workflow-studio-node-library-registry-usecase \
  -p oya-workflow-studio-node-library-registry-api \
  -p oya-workflow-studio-node-library-registry-adapter \
  -p oya-workflow-studio-node-library-registry-adapter-cdn \
  -p oya-workflow-studio-node-library-registry-rest \
  -p oya-workflow-studio-node-library-registry-sdk \
  -p oya-workflow-studio-node-library-registry-app
cargo nextest run -p oya-workflow-studio-node-library-registry-domain --test determinism
cargo nextest run -p oya-workflow-studio-node-library-registry-domain --test signature
cargo run -p oya-dev-cli -- gate validate node-library-signature-verification --microservice workflow-studio
cargo run -p oya-dev-cli -- gate validate node-library-determinism --microservice workflow-studio
helm lint microservices/workflow-studio/iac/helm/node-library-registry-rest
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_load_determinism_3x` | AC-11; 3x re-load byte-identical |
| `test_signature_verify_happy` | valid signed library accepted |
| `test_signature_verify_tampered` | tampered bytes rejected |
| `test_signature_verify_wrong_pack_key` | pack-eu library signed with pack-kr key rejected |
| `test_revocation_propagation_60s` | revoked library refuses load within 60s of revocation event |
| `test_descriptor_xss_safe` | data-class hint fields never contain executable HTML |

## Halt Conditions

- determinism test fails — STOP. Anti-pattern non_deterministic_node_library_load.
- signature verify accepts tampered bytes — STOP. T-T-03 supply-chain breach.

## Next IP

[`IP-008-llm-assist-adapter.md`](IP-008-llm-assist-adapter.md)

## References

- threat-model.md T-S-04, T-T-03, T-E-04.
- PRD FR-11.
- ed25519-dalek docs — `docs.rs/ed25519-dalek`.
- SLSA Level 3 spec — `slsa.dev/spec/v1.0/levels`.
- in-toto attestations — `in-toto.io`.

## Counterpart Anchors
This workflow-studio IP is measured against the local Workflow Studio benchmark envelope: n8n for visual workflow authoring depth, Zapier for broad trigger/action accessibility, Make for visual branching and scenario ergonomics, and Workato for enterprise workflow governance. The IP must keep Oyatie's differentiator intact: canonical workflow_spec.v1 round-trip, Cedar-gated save/publish, tenant-scoped collaboration, and audit evidence rather than counterpart-specific runtime authority.

## Pod runtime tier (per ADR-0338)

- pod_runtime_tier: `0`.
- runtime_requirement: Kata Containers plus Cloud Hypervisor REQUIRED.
- justification: tenant-customer code exists in this IP execution path; trigger_terms: [`workflow-studio`].
- surface_evidence_paths: [`microservices/workflow-studio/IP-007-node-library-registry-full.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/templates/index.json`, `microservices/workflow-studio/templates/schemas/workflow-template.schema.json`, `microservices/workflow-studio/PRD.md`, `microservices/workflow-studio/ARCHITECTURE.md`].
