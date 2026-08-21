//! Real [`crate::ports::MerkleVerifier`] implementation, backed by
//! `audit_chain_domain::MerkleTree::verify_proof`.
//!
//! This is pure math (no I/O), so — unlike [`crate::ports::KeyResolver`] and
//! [`crate::ports::RootRegistry`], which need a real read path this crate
//! cannot provide — this crate supplies its own concrete adapter directly,
//! the same way `audit_sealing_domain::merkle_engine::MerkleTreeEngine`
//! supplies a concrete `MerkleEngine` for sealing.

use audit_chain_domain::{MerkleTree, Sha256Hash};

use crate::ports::MerkleVerifier;
use crate::request::MerkleInclusionProof;

/// Concrete, stateless [`MerkleVerifier`] over
/// `audit_chain_domain::MerkleTree::verify_proof`.
#[derive(Clone, Copy, Debug, Default)]
pub struct ChainMerkleVerifier;

impl MerkleVerifier for ChainMerkleVerifier {
    /// Delegates to `MerkleTree::verify_proof`, which already fails closed
    /// on an out-of-range index, a wrong-length path, and a zero leaf count
    /// (see its own doc). The one thing this adapter must itself refuse to
    /// widen (L4): `MerkleInclusionProof`'s `leaf_index` / `leaf_count` are
    /// `u64` but `verify_proof` takes `usize`. On a 32-bit target, a `u64`
    /// value that does not fit in `usize` returns `false` here rather than
    /// truncating or saturating — a value that cannot even be represented
    /// can never be verified as in-range. (On the 64-bit targets this crate
    /// actually ships on, `usize` and `u64` have the same range, so this
    /// conversion never fails there; the guard exists for portability, not
    /// because it is reachable on every target.)
    fn verify(&self, leaf: &Sha256Hash, proof: &MerkleInclusionProof, root: &Sha256Hash) -> bool {
        let Ok(leaf_index) = usize::try_from(proof.leaf_index) else {
            return false;
        };
        let Ok(leaf_count) = usize::try_from(proof.leaf_count) else {
            return false;
        };
        MerkleTree::verify_proof(*leaf, leaf_index, &proof.audit_path, *root, leaf_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leaf(seed: u8) -> Sha256Hash {
        let mut bytes = [0_u8; 32];
        bytes[0] = seed;
        bytes
    }

    #[test]
    fn accepts_a_real_inclusion_proof() {
        let leaves: Vec<Sha256Hash> = (0_u8..5).map(leaf).collect();
        let tree = MerkleTree::new(leaves.clone());
        let root = tree.build_root();
        let path = tree.proof_path(2);
        let proof = MerkleInclusionProof {
            audit_path: path,
            leaf_index: 2,
            leaf_count: tree.len() as u64,
        };
        assert!(ChainMerkleVerifier.verify(&leaves[2], &proof, &root));
    }

    #[test]
    fn rejects_a_tampered_leaf() {
        let leaves: Vec<Sha256Hash> = (0_u8..5).map(leaf).collect();
        let tree = MerkleTree::new(leaves.clone());
        let root = tree.build_root();
        let path = tree.proof_path(2);
        let proof = MerkleInclusionProof {
            audit_path: path,
            leaf_index: 2,
            leaf_count: tree.len() as u64,
        };
        let mut bad_leaf = leaves[2];
        bad_leaf[0] ^= 0xff;
        assert!(!ChainMerkleVerifier.verify(&bad_leaf, &proof, &root));
    }

    #[test]
    fn rejects_an_absurd_leaf_count_instead_of_ever_passing() {
        // L4 end-to-end: a `leaf_count` that cannot possibly be the real
        // tree size must never be accepted by widening or truncation —
        // whether that is caught by this adapter's `usize::try_from` guard
        // (32-bit targets) or by `verify_proof`'s own depth check (64-bit
        // targets, where the conversion itself always succeeds), the
        // observable result from this port is the same: reject.
        //
        // Deliberately NOT `u64::MAX`: `MerkleTree`'s own
        // `largest_power_of_two_less_than` helper multiplies `leaf_count` by
        // two while searching for a split point, which legitimately
        // overflows for a value that close to `u64::MAX` — that is a
        // panic-on-adversarial-input question for `audit-chain-domain`
        // itself, out of scope for this crate to fix or paper over here. A
        // leaf count several orders of magnitude too large already proves
        // the point this test exists for without tripping that overflow.
        let leaves: Vec<Sha256Hash> = (0_u8..3).map(leaf).collect();
        let tree = MerkleTree::new(leaves.clone());
        let root = tree.build_root();
        let path = tree.proof_path(0);
        let proof = MerkleInclusionProof {
            audit_path: path,
            leaf_index: 0,
            leaf_count: 1_000_000_000_000,
        };
        assert!(!ChainMerkleVerifier.verify(&leaves[0], &proof, &root));
    }
}
