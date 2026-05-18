---
doc_class: ImplementationPlan
impl_plan_id: IP-006-sealing-kernel
status: pending
owner: axis-audit-chain
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, port-location, layer-correctness, data-class]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006: oya-audit-chain-sealing-kernel

## Intent

Port traits + entity types for `sealing` BC. Zero I/O. Foundation for sealing-domain + adapters + worker.

## Crate Naming

`oya-audit-chain-sealing-kernel`.

## Concrete File Targets

| Path | Action |
|---|---|
| `.../src/crates/oya-audit-chain-sealing-kernel/Cargo.toml` | create |
| `.../src/lib.rs` | create |
| `.../src/entities.rs` | create — `MerkleTree`, `MerkleRoot`, `SealRecord`, `SigningKey` (handle, not material), `PackEpoch`, `KeyFingerprint` |
| `.../src/ports.rs` | create — port traits: `MerkleEngine`, `SignerPort`, `RootPublisher`, `ObjectStoreWriter`, `IndexWriter` |
| `.../src/errors.rs` | create |

## Code Shape

```rust
// src/ports.rs
#[async_trait]
pub trait MerkleEngine: Send + Sync + Sealed {
    fn build_tree(&self, leaves: &[Vec<u8>]) -> Result<MerkleTree, KernelError>;
    fn build_proof(&self, tree: &MerkleTree, leaf_index: usize) -> Result<MerkleProof, KernelError>;
}

#[async_trait]
pub trait SignerPort: Send + Sync + Sealed {
    async fn sign(&self, root_hash: &[u8; 32]) -> Result<Signature, KernelError>;
    async fn public_key_fingerprint(&self) -> Result<KeyFingerprint, KernelError>;
}

#[async_trait]
pub trait RootPublisher: Send + Sync + Sealed {
    async fn publish(&self, record: &SealRecord) -> Result<PublicationStatus, KernelError>;
}

#[async_trait]
pub trait ObjectStoreWriter: Send + Sync + Sealed {
    async fn write_tree(&self, pack: &PackId, period_id: &PeriodId, tree: &MerkleTree) -> Result<(), KernelError>;
    async fn write_record(&self, record: &SealRecord) -> Result<(), KernelError>;
}

#[async_trait]
pub trait IndexWriter: Send + Sync + Sealed {
    async fn write_seal_record(&self, record: &SealRecord) -> Result<(), KernelError>;
}
```

## Acceptance Gates

Same shape as IP-003 kernel-class gates. Coverage 90% line / 80% branch.

## References

- Bominal ADR-0028 §"Sealing process".
- ADR-0105.
