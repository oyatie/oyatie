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

/// A `leaf_count` above this bound is rejected before ever reaching
/// `audit_chain_domain::MerkleTree::verify_proof` — see [`ChainMerkleVerifier::verify`]'s
/// doc for why. `2^40` (a little over one trillion) is already many orders
/// of magnitude beyond any tree this system could plausibly ever build (a
/// trillion 32-byte leaves alone is 32 terabytes of raw leaf data before
/// even hashing), yet it sits comfortably below the value at which
/// `MerkleTree`'s own internal split-point search can overflow — the bound
/// is chosen with a wide margin so it does not depend on knowing that
/// arithmetic's exact overflow threshold.
const MAX_PLAUSIBLE_LEAF_COUNT: u64 = 1 << 40;

/// Concrete, stateless [`MerkleVerifier`] over
/// `audit_chain_domain::MerkleTree::verify_proof`.
#[derive(Clone, Copy, Debug, Default)]
pub struct ChainMerkleVerifier;

impl MerkleVerifier for ChainMerkleVerifier {
    /// Delegates to `MerkleTree::verify_proof`, which already fails closed
    /// on an out-of-range index, a wrong-length path, and a zero leaf count
    /// (see its own doc). This adapter itself must refuse to widen two
    /// further things (L4), both fixed by rejecting rather than by
    /// truncating or saturating:
    ///
    /// - `MerkleInclusionProof`'s `leaf_index` / `leaf_count` are `u64` but
    ///   `verify_proof` takes `usize`. On a 32-bit target, a `u64` value
    ///   that does not fit in `usize` returns `false` here — a value that
    ///   cannot even be represented can never be verified as in-range. (On
    ///   the 64-bit targets this crate actually ships on, `usize` and
    ///   `u64` have the same range, so this particular conversion never
    ///   fails there; the guard exists for portability.)
    /// - `leaf_count` near `u64::MAX` reaches `MerkleTree::verify_proof`
    ///   even on a 64-bit target (the `usize` conversion above succeeds
    ///   there) and used to overflow inside `MerkleTree`'s private
    ///   split-point search — an unauthenticated, caller-controlled panic
    ///   (a DoS), the exact inverse of "never panics" this type promises.
    ///   `leaf_count > `[`MAX_PLAUSIBLE_LEAF_COUNT`] is rejected with
    ///   `false` before ever calling into `MerkleTree`, so no value this
    ///   adapter passes on can reach that overflow.
    fn verify(&self, leaf: &Sha256Hash, proof: &MerkleInclusionProof, root: &Sha256Hash) -> bool {
        if proof.leaf_count > MAX_PLAUSIBLE_LEAF_COUNT {
            return false;
        }
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
        // tree size must never be accepted by widening or truncation.
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

    #[test]
    fn rejects_a_leaf_count_near_u64_max_without_panicking() {
        // Regression test for a real defect: `leaf_count` values close to
        // `u64::MAX` pass this adapter's `usize::try_from` guard cleanly on
        // a 64-bit target (the conversion always succeeds there), reach
        // `MerkleTree::verify_proof`, and used to overflow inside
        // `MerkleTree`'s private split-point search — an unauthenticated,
        // caller-controlled panic, in direct contradiction of this type's
        // "never panics" doc contract. `catch_unwind` proves no panic
        // escapes; the outer assert proves the verdict is still a clean
        // `false`, not a crash disguised as one.
        let leaves: Vec<Sha256Hash> = (0_u8..3).map(leaf).collect();
        let tree = MerkleTree::new(leaves.clone());
        let root = tree.build_root();
        let path = tree.proof_path(0);
        let proof = MerkleInclusionProof {
            audit_path: path,
            leaf_index: 0,
            leaf_count: u64::MAX,
        };
        let outcome =
            std::panic::catch_unwind(|| ChainMerkleVerifier.verify(&leaves[0], &proof, &root));
        assert!(
            matches!(outcome, Ok(false)),
            "a leaf_count near u64::MAX must be rejected cleanly, never panic"
        );
    }
}
