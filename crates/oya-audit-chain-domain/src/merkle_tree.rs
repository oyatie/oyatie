//! Public `MerkleTree` struct with proof-path support.
//!
//! The existing internal `merkle_root()` helper in `lib.rs` computes roots
//! in-place during chain append; this module exposes the same SHA-256 Merkle
//! algorithm as a named, testable type for callers that need inclusion proofs
//! (e.g. the future `oya-audit-chain-segments-application` seal use-case).
//!
//! Leaf ordering: callers MUST sort leaves before constructing `MerkleTree`
//! when determinism across independent nodes is required (per Bominal ADR-0028).
//! This type is ordering-agnostic to keep the type minimal.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use sha2::{Digest, Sha256};

/// A 32-byte SHA-256 hash.
pub type Sha256Hash = [u8; 32];

/// Deterministic binary Merkle tree over SHA-256 leaf hashes.
///
/// Odd-length levels duplicate the last node before pairing, matching the
/// internal algorithm already used by `AuditChain`'s Merkle-root computation.
///
/// # Panics
///
/// [`MerkleTree::new`] panics when `leaves` is empty — a tree with no leaves
/// has no defined root and callers must guard against this.
#[derive(Clone, Debug)]
pub struct MerkleTree {
    leaves: Vec<Sha256Hash>,
}

impl MerkleTree {
    /// Construct a `MerkleTree` from a non-empty slice of leaf hashes.
    ///
    /// # Panics
    ///
    /// Panics when `leaves` is empty.
    pub fn new(leaves: Vec<Sha256Hash>) -> Self {
        assert!(!leaves.is_empty(), "MerkleTree requires at least one leaf");
        Self { leaves }
    }

    /// Return the number of leaf hashes.
    pub fn len(&self) -> usize {
        self.leaves.len()
    }

    /// Returns `true` when there are no leaves (always `false` after `new`).
    pub fn is_empty(&self) -> bool {
        self.leaves.is_empty()
    }

    /// Compute the Merkle root by reducing leaves pairwise bottom-up.
    pub fn build_root(&self) -> Sha256Hash {
        let mut level = self.leaves.clone();
        while level.len() > 1 {
            level = level
                .chunks(2)
                .map(|pair| {
                    let mut hasher = Sha256::new();
                    hasher.update(pair[0]);
                    // Duplicate last leaf when count is odd.
                    hasher.update(pair.get(1).copied().unwrap_or(pair[0]));
                    hasher.finalize().into()
                })
                .collect();
        }
        level[0]
    }

    /// Return the sibling hashes from `leaf_index` up to the root (proof of
    /// inclusion). The caller combines this path with the target leaf to
    /// recompute the root and confirm inclusion.
    ///
    /// Returns an empty `Vec` when there is exactly one leaf (the root equals
    /// the single leaf hash; no siblings needed).
    ///
    /// # Panics
    ///
    /// Panics when `leaf_index >= self.len()`.
    pub fn proof_path(&self, leaf_index: usize) -> Vec<Sha256Hash> {
        assert!(
            leaf_index < self.leaves.len(),
            "leaf_index {leaf_index} out of bounds (len={})",
            self.leaves.len()
        );
        let mut proof = Vec::new();
        let mut level = self.leaves.clone();
        let mut idx = leaf_index;
        while level.len() > 1 {
            let sibling = if idx.is_multiple_of(2) {
                // Right sibling; duplicate if this is the last (odd) leaf.
                level.get(idx + 1).copied().unwrap_or(level[idx])
            } else {
                // Left sibling always exists when index is odd.
                level[idx - 1]
            };
            proof.push(sibling);
            level = level
                .chunks(2)
                .map(|pair| {
                    let mut hasher = Sha256::new();
                    hasher.update(pair[0]);
                    hasher.update(pair.get(1).copied().unwrap_or(pair[0]));
                    hasher.finalize().into()
                })
                .collect();
            idx /= 2;
        }
        proof
    }

    /// Verify that `leaf` at `leaf_index` is included in `root` using the
    /// given `proof_path` (as returned by [`MerkleTree::proof_path`]).
    pub fn verify_proof(
        leaf: Sha256Hash,
        leaf_index: usize,
        proof_path: &[Sha256Hash],
        root: Sha256Hash,
    ) -> bool {
        let mut current = leaf;
        let mut idx = leaf_index;
        for &sibling in proof_path {
            let mut hasher = Sha256::new();
            if idx.is_multiple_of(2) {
                hasher.update(current);
                hasher.update(sibling);
            } else {
                hasher.update(sibling);
                hasher.update(current);
            }
            current = hasher.finalize().into();
            idx /= 2;
        }
        current == root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    fn leaf(data: &[u8]) -> Sha256Hash {
        Sha256::digest(data).into()
    }

    #[test]
    fn single_leaf_root_equals_leaf() {
        let l = leaf(b"only");
        let tree = MerkleTree::new(vec![l]);
        assert_eq!(tree.build_root(), l);
    }

    #[test]
    fn two_leaves_deterministic_root() {
        let a = leaf(b"a");
        let b = leaf(b"b");
        let tree = MerkleTree::new(vec![a, b]);
        let root = tree.build_root();

        // Recompute manually.
        let mut hasher = Sha256::new();
        hasher.update(a);
        hasher.update(b);
        let expected: Sha256Hash = hasher.finalize().into();
        assert_eq!(root, expected);
    }

    #[test]
    fn same_leaves_same_order_same_root() {
        let leaves: Vec<Sha256Hash> = (0u8..8).map(|i| leaf(&[i])).collect();
        let t1 = MerkleTree::new(leaves.clone());
        let t2 = MerkleTree::new(leaves);
        assert_eq!(t1.build_root(), t2.build_root());
    }

    #[test]
    fn odd_leaf_count_duplicates_last() {
        let leaves: Vec<Sha256Hash> = (0u8..3).map(|i| leaf(&[i])).collect();
        // Build manually: leaves = [L0, L1, L2]
        // Level 1: [H(L0‖L1), H(L2‖L2)]
        // Level 2 (root): H(H01 ‖ H22)
        let mut h01 = Sha256::new();
        h01.update(leaves[0]);
        h01.update(leaves[1]);
        let n01: Sha256Hash = h01.finalize().into();

        let mut h22 = Sha256::new();
        h22.update(leaves[2]);
        h22.update(leaves[2]);
        let n22: Sha256Hash = h22.finalize().into();

        let mut root_h = Sha256::new();
        root_h.update(n01);
        root_h.update(n22);
        let expected: Sha256Hash = root_h.finalize().into();

        let tree = MerkleTree::new(leaves);
        assert_eq!(tree.build_root(), expected);
    }

    #[test]
    fn proof_path_single_leaf_is_empty() {
        let l = leaf(b"solo");
        let tree = MerkleTree::new(vec![l]);
        let path = tree.proof_path(0);
        assert!(path.is_empty());
    }

    #[test]
    fn proof_path_verifies_all_leaves_two_leaf_tree() {
        let leaves: Vec<Sha256Hash> = (0u8..2).map(|i| leaf(&[i])).collect();
        let tree = MerkleTree::new(leaves.clone());
        let root = tree.build_root();
        for (idx, &leaf_hash) in leaves.iter().enumerate() {
            let path = tree.proof_path(idx);
            assert!(
                MerkleTree::verify_proof(leaf_hash, idx, &path, root),
                "proof failed for leaf {idx}"
            );
        }
    }

    #[test]
    fn proof_path_verifies_all_leaves_eight_leaf_tree() {
        let leaves: Vec<Sha256Hash> = (0u8..8).map(|i| leaf(&[i])).collect();
        let tree = MerkleTree::new(leaves.clone());
        let root = tree.build_root();
        for (idx, &leaf_hash) in leaves.iter().enumerate() {
            let path = tree.proof_path(idx);
            assert!(
                MerkleTree::verify_proof(leaf_hash, idx, &path, root),
                "proof failed for leaf {idx}"
            );
        }
    }

    #[test]
    fn proof_path_verifies_all_leaves_odd_tree() {
        let leaves: Vec<Sha256Hash> = (0u8..5).map(|i| leaf(&[i])).collect();
        let tree = MerkleTree::new(leaves.clone());
        let root = tree.build_root();
        for (idx, &leaf_hash) in leaves.iter().enumerate() {
            let path = tree.proof_path(idx);
            assert!(
                MerkleTree::verify_proof(leaf_hash, idx, &path, root),
                "proof failed for leaf {idx} (odd tree)"
            );
        }
    }

    #[test]
    fn tampered_leaf_fails_verification() {
        let leaves: Vec<Sha256Hash> = (0u8..4).map(|i| leaf(&[i])).collect();
        let tree = MerkleTree::new(leaves.clone());
        let root = tree.build_root();
        let path = tree.proof_path(0);
        // Flip one bit in the leaf.
        let mut bad_leaf = leaves[0];
        bad_leaf[0] ^= 0xff;
        assert!(
            !MerkleTree::verify_proof(bad_leaf, 0, &path, root),
            "tampered leaf must fail proof"
        );
    }

    #[test]
    fn tampered_root_fails_verification() {
        let leaves: Vec<Sha256Hash> = (0u8..4).map(|i| leaf(&[i])).collect();
        let tree = MerkleTree::new(leaves.clone());
        let mut root = tree.build_root();
        let path = tree.proof_path(1);
        root[31] ^= 0xff;
        assert!(
            !MerkleTree::verify_proof(leaves[1], 1, &path, root),
            "tampered root must fail proof"
        );
    }
}
