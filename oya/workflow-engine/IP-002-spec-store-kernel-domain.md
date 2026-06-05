---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02b-substrate-ready
phase: P01-durable-execution-substrate
impl_plan_id: IP-002-spec-store-kernel-domain
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, oya-governance-per-microservice-layout]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-002: oya-workflow-engine-spec-store-{kernel,domain}

## Intent

Scaffold the spec-store kernel + domain layers per ADR-0105: kernel = port traits (sealed) + entity types + value objects + error types (zero I/O, zero business logic); domain = pure spec compilation + canonicalization + signature-verification math. Foundation for IP-008 (spec-store usecase/adapter/rest/sdk/app).

## ChangeSet boundary

Two new Rust crates at `microservices/workflow-engine/src/crates/oya-workflow-engine-spec-store-{kernel,domain}/`. Workspace members added to root `Cargo.toml`. Catalog rows under `microservices/workflow-engine/catalog/`. No downstream consumers in this IP.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/workflow-engine/src/crates/oya-workflow-engine-spec-store-kernel/Cargo.toml` | create | `[package]` + minimal deps |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-spec-store-kernel/src/lib.rs` | create | module declarations |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-spec-store-kernel/src/entities.rs` | create | `WorkflowSpec`, `SpecVersion`, `SpecSignature`, `SpecLifecycle`, `SignerIdentity` with `data_class` annotations |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-spec-store-kernel/src/ports.rs` | create | port trait declarations (sealed): `WorkflowSpecRepository`, `SpecCompiler` |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-spec-store-kernel/src/errors.rs` | create | error variants |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-spec-store-domain/Cargo.toml` | create | depends on kernel |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-spec-store-domain/src/canonicalize.rs` | create | JSON canonicalization (sorted keys, NFC normalize, no floats) |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-spec-store-domain/src/signature.rs` | create | Ed25519 verification logic; pure |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-spec-store-domain/src/version_sha.rs` | create | SHA-256 over canonical body + metadata |
| `microservices/workflow-engine/src/crates/oya-workflow-engine-spec-store-domain/src/forbidden_constructs.rs` | create | per `policy/spec-integrity.md` Forbidden Constructs list |
| `Cargo.toml` (workspace) | update | add both crates to `[workspace.members]` |
| `microservices/workflow-engine/catalog/oya-workflow-engine-spec-store-kernel.yaml` | create | catalog row |
| `microservices/workflow-engine/catalog/oya-workflow-engine-spec-store-domain.yaml` | create | catalog row |

## Crate Naming

```
NAME: oya-workflow-engine-spec-store-{kernel,domain}
JUSTIFICATION:
- microservice = workflow-engine
- bc-tokens = spec-store (primary BC per PRD)
- layer = kernel | domain (ADR-0105)
- exemptions: none
```

## Code Shape

```rust
// kernel/src/ports.rs
#[async_trait]
pub trait WorkflowSpecRepository: Send + Sync + Sealed {
    async fn store(&self, spec: &WorkflowSpec) -> Result<SpecVersion, RepositoryError>;
    async fn load(&self, tenant_id: &TenantId, spec_id: &SpecId, version_sha: &VersionSha)
        -> Result<WorkflowSpec, RepositoryError>;
    async fn list(&self, tenant_id: &TenantId, filter: SpecFilter)
        -> Result<Vec<SpecVersion>, RepositoryError>;
    async fn transition_lifecycle(&self, tenant_id: &TenantId, spec_id: &SpecId,
        version_sha: &VersionSha, lifecycle_to: SpecLifecycle, reason: &str)
        -> Result<(), RepositoryError>;
}

#[async_trait]
pub trait SpecCompiler: Send + Sync + Sealed {
    async fn compile(&self, body: &str) -> Result<WorkflowSpec, CompilerError>;
}
```

```rust
// domain/src/canonicalize.rs
pub fn canonicalize(body: &serde_json::Value) -> Result<String, CanonicalizationError> {
    // sort keys; normalize NFC; reject floats; return canonical JSON string
}
```

## Acceptance Gates

```bash
cargo check -p oya-workflow-engine-spec-store-kernel --all-features
cargo check -p oya-workflow-engine-spec-store-domain --all-features
cargo clippy -p oya-workflow-engine-spec-store-kernel --all-features -- -D warnings
cargo clippy -p oya-workflow-engine-spec-store-domain --all-features -- -D warnings
cargo nextest run -p oya-workflow-engine-spec-store-kernel --all-features
cargo nextest run -p oya-workflow-engine-spec-store-domain --all-features
buck2 build //:quality-lane-registry-authority-check # lane=port-location --crate oya-workflow-engine-spec-store-kernel
buck2 build //:quality-lane-registry-authority-check # lane=layer-correctness --crate oya-workflow-engine-spec-store-domain
buck2 build //:quality-lane-registry-authority-check # lane=data-class --crate oya-workflow-engine-spec-store-kernel
```

## Test Plan

Per PHASE-01 thresholds: kernel 90% line, 80% branch; domain 95% line, 90% branch + property tests.

| Test | Verifies |
|---|---|
| `test_workflow_spec_construction` | entity invariants |
| `test_port_traits_sealed` | external crates cannot impl sealed traits |
| `test_canonicalize_deterministic` | property: canonicalize(json) idempotent |
| `test_canonicalize_keys_sorted` | property: keys in NFC-normalized canonical form |
| `test_version_sha_deterministic` | property: same body → same SHA |
| `test_signature_verification_happy` | Ed25519 signature verified |
| `test_signature_verification_tampered` | tampered body → verify fails |
| `test_forbidden_constructs_rejected` | spec containing system-time access refused |

## Halt Conditions

- BNF v4.1 naming violation.
- Any port trait introduces business logic.
- Any I/O reachable from kernel.

## Next IP

[`IP-003-state-machine-kernel-domain.md`](IP-003-state-machine-kernel-domain.md)

## References

- ADR-0056 BNF v4.1; ADR-0105 13-layer enum.
- PRD §"Bounded Contexts" port-trait table.
- `policy/spec-integrity.md` §"Forbidden Spec Constructs".
- Bominal ADR-0028 (data-class taxonomy).
