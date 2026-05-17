---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-ci-fitness-consolidation
impl_plan_id: IP-008-evidence-emitter-kernel-domain
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: ops-security
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, lean-a1, port-location, layer-correctness, data-class]
---

# IP-008: oya-governance-evidence-emitter-{kernel,domain}

## Intent

Fill in the kernel + domain layers of the `evidence-emitter` BC. Kernel = `Finding`, `EvidenceRecord`, `AuditSeal`, `ReplayCursor` entities + ports. Domain = canonical-JSON serialisation + Merkle-tree composition.

## ChangeSet boundary

2 crates.

## Concrete File Targets

| Path | Action |
|---|---|
| `…/-kernel/src/entities.rs` | `Finding`, `EvidenceRecord`, `AuditSeal`, `ReplayCursor`; data_class annotated |
| `…/-kernel/src/ports.rs` | `FindingPersistence`, `AuditChainSealer`, `ReplayQuery` (sealed) |
| `…/-domain/src/canonical_json.rs` | RFC8785-style canonical-JSON serialisation |
| `…/-domain/src/merkle.rs` | Merkle-tree composition for batch-seal |
| `…/-domain/src/ed25519_compose.rs` | Ed25519 signature composition (no key access here; that's adapter) |

## Code Shape

```rust
// kernel/src/entities.rs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Finding {
    #[data_class(AUDIT)] pub finding_id: uuid::Uuid,
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)] pub microservice: String,
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)] pub tenant_id: Option<String>,
    #[data_class(AUDIT)] pub lane_id: oya_governance_lane_runtime_kernel::LaneId,
    #[data_class(AUDIT)] pub severity: oya_governance_policy_engine_kernel::Severity,
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)] pub sha: String,
    #[data_class(AUDIT)] pub citation: oya_governance_policy_engine_kernel::BaselineCitation,
    #[data_class(AUDIT)] pub evidence_blob_id: Option<String>,
    #[data_class(AUDIT)] pub signature: String,
    #[data_class(AUDIT)] pub signed_by: String,
    #[data_class(AUDIT)] pub sealed_at: Option<chrono::DateTime<chrono::Utc>>,
    #[data_class(AUDIT)] pub finding_hash: String,
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)] pub pr_number: Option<u64>,
    #[data_class(PII_IDENTIFYING)] pub pr_author_subject: Option<String>,
}
```

```rust
// domain/src/canonical_json.rs
pub fn canonicalize(value: &serde_json::Value) -> Vec<u8> {
    // RFC8785-compatible canonicalization (sorted keys; UTF-8; no whitespace)
    todo!()
}
```

```rust
// domain/src/merkle.rs
pub fn compose_root(leaves: &[[u8; 32]]) -> [u8; 32] {
    // SHA256-based Merkle tree
    todo!()
}
```

## Acceptance Gates

```bash
cargo check -p oya-governance-evidence-emitter-{kernel,domain}
cargo nextest run -p oya-governance-evidence-emitter-kernel
cargo nextest run -p oya-governance-evidence-emitter-domain
cargo run -p oya-dev-cli -- gate validate data-class --crate oya-governance-evidence-emitter-kernel
```

## Test Plan

| Test | Verifies |
|---|---|
| `kernel::test_finding_serde_roundtrip` | entity stability |
| `domain::test_canonical_json_determinism` | byte-identical across re-runs |
| `domain::test_merkle_root_stability` | RFC9162-style Merkle determinism |

Coverage 90% / 80%.

## Halt Conditions

- Kernel imports a key-management crate → refactor to adapter.
- Canonical JSON non-deterministic → refactor.

## Next IP

[`IP-009-evidence-emitter-adapter-rest-worker.md`](IP-009-evidence-emitter-adapter-rest-worker.md)

## References

- Bominal ADR-0028 (data-class taxonomy).
- RFC 8785 (canonical JSON).
- RFC 9162 (Merkle trees).
- `microservices/governance/PRD.md` §"Bounded Contexts" evidence-emitter.
