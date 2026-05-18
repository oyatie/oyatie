---
doc_class: ImplementationPlan
impl_plan_id: IP-007-sealing-domain-merkle
status: pending
owner: axis-audit-chain
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: oya-audit-chain-sealing-domain (Merkle math)

## Intent

Pure Merkle-tree math (RFC 6962 SHA-256 binary tree) + root chaining (`prior_root_hash` link). Zero I/O. Property-tested against RFC 6962 reference vectors.

## Crate Naming

`oya-audit-chain-sealing-domain`.

## Concrete File Targets

| Path | Action |
|---|---|
| `.../src/lib.rs` | create |
| `.../src/merkle.rs` | create — leaf-hash + internal-node + proof-extract |
| `.../src/chain.rs` | create — `compute_chained_root(period_root, prior_root) -> CommittedRoot` |
| `.../tests/rfc6962_test_vectors.rs` | create — property-test against RFC 6962 §Test Vectors |
| `.../tests/proptest_tampers.rs` | create — 10k random trees × 10k random mutations; every mutation must invalidate proof |

## Code Shape

```rust
// merkle.rs
pub fn leaf_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = sha2::Sha256::new();
    hasher.update(&[0x00]);  // leaf prefix per RFC 6962 §2.1
    hasher.update(data);
    hasher.finalize().into()
}

pub fn internal_node(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut hasher = sha2::Sha256::new();
    hasher.update(&[0x01]);  // internal prefix per RFC 6962 §2.1
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

pub fn build_tree(leaves: &[Vec<u8>]) -> MerkleTree { ... }
pub fn build_proof(tree: &MerkleTree, leaf_index: usize) -> MerkleProof { ... }
pub fn verify_proof(leaf: &[u8; 32], proof: &MerkleProof, root: &[u8; 32]) -> bool { ... }
```

## Acceptance Gates

```bash
cargo nextest run -p oya-audit-chain-sealing-domain
cargo nextest run -p oya-audit-chain-sealing-domain --features proptest -- proptest_tampers
# coverage ≥ 95% line / 90% branch
cargo run -p oya-dev-cli -- gate validate audit-chain-merkle-shape
```

## Halt Conditions

- Any RFC 6962 test vector fails — block.
- Any 10k-tamper test classifies tampered tree as `verified: true` — fundamental correctness bug.

## References

- RFC 6962 §"Merkle Tree" + §"Test Vectors".
- Bominal ADR-0028 §"Sealing process".
- `microservices/audit-chain/policy/seal-integrity.md` §"SI-01..SI-03".
