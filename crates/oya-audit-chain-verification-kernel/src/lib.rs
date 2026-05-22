//! Audit-chain verification kernel: pure proof + signature ports.
//!
//! Wave 15-IMPL-truth-up scaffold (2026-05-21). Resolver and registry traits
//! land fully in IP-011.
#![allow(dead_code)]

/// Port: resolves a published root by (pack, tenant_partition, period_id).
pub trait RootRegistry {
    type Root;
    type Error;
    fn resolve_root(
        &self,
        pack: &str,
        tenant_partition: &str,
        period_id: &str,
    ) -> Result<Self::Root, Self::Error>;
}

/// Port: resolves an Ed25519 public verification key by (pack, tenant_partition, period_id).
pub trait KeyResolver {
    type Key;
    type Error;
    fn resolve_key(
        &self,
        pack: &str,
        tenant_partition: &str,
        period_id: &str,
    ) -> Result<Self::Key, Self::Error>;
}

/// Port: pure Merkle inclusion verifier.
pub trait MerkleVerifier {
    type Proof;
    type Root;
    type Leaf;
    fn verify(&self, leaf: &Self::Leaf, proof: &Self::Proof, root: &Self::Root) -> bool;
}
