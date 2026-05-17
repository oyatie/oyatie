//! Public `MerkleTree` struct with proof-path support.
//!
//! The existing internal `merkle_root()` helper in `lib.rs` computes roots
//! in-place during chain append; this module exposes the same SHA-256 Merkle
//! algorithm as a named, testable type for callers that need inclusion proofs
//! (e.g. the future `oya-audit-chain-segments-application` seal use-case).
//!
//! ## Domain separation
//!
//! Leaf and internal-node hashes are domain-separated using the same
//! length-prefixed scheme as `lib.rs`'s private `digest_prefixed` helper:
//!
//! - Leaves: `SHA-256( len("merkle-leaf") || "merkle-leaf" || len(field) || field )`
//! - Nodes:  `SHA-256( len("merkle-node") || "merkle-node" || len(left) || left
//!                                                            || len(right) || right )`
//!
//! where `len(x)` is the byte-length of `x` encoded as a big-endian `u64`,
//! and `field` / `left` / `right` are the lower-hex representations of the
//! 32-byte input arrays. This matches the `AuditEvent.merkle_root` computation
//! so that roots and proofs produced by `MerkleTree` are directly comparable
//! to values persisted by `AuditChain`.
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

/// A domain-separated leaf node hash computed from a raw leaf value.
///
/// Callers never construct this directly; it is an internal type used to
/// ensure that leaf hashes cannot be mistaken for internal-node hashes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DomainHash([u8; 32]);

/// Compute a domain-separated hash matching `lib.rs`'s `digest_prefixed`.
///
/// Produces `SHA-256( len(domain) || domain || len(field0) || field0 || … )`
/// where each `len` is a big-endian `u64`.
fn digest_prefixed_bytes<I: IntoIterator<Item = S>, S: AsRef<[u8]>>(
    domain: &str,
    fields: I,
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    let dom = domain.as_bytes();
    hasher.update((dom.len() as u64).to_be_bytes());
    hasher.update(dom);
    for field in fields {
        let f = field.as_ref();
        hasher.update((f.len() as u64).to_be_bytes());
        hasher.update(f);
    }
    hasher.finalize().into()
}

/// Encode 32 bytes as a lower-hex string (same as `lib.rs`'s `encode_hex`).
fn encode_hex_32(bytes: &[u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(64);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

/// Hash a raw leaf value into a domain-separated leaf node.
fn leaf_domain_hash(raw: &Sha256Hash) -> DomainHash {
    let hex = encode_hex_32(raw);
    DomainHash(digest_prefixed_bytes("merkle-leaf", [hex.as_str()]))
}

/// Hash two domain-hashes into a parent domain-separated node.
fn node_domain_hash(left: &DomainHash, right: &DomainHash) -> DomainHash {
    let left_hex = encode_hex_32(&left.0);
    let right_hex = encode_hex_32(&right.0);
    DomainHash(digest_prefixed_bytes(
        "merkle-node",
        [left_hex.as_str(), right_hex.as_str()],
    ))
}

/// Deterministic binary Merkle tree over SHA-256 leaf hashes.
///
/// Odd-length levels duplicate the last node before pairing, matching the
/// internal algorithm already used by `AuditChain`'s Merkle-root computation.
///
/// Leaf and internal-node hashes use the same domain-separated scheme as
/// `AuditChain` so that `build_root()` produces a root directly comparable
/// to `AuditEvent.merkle_root`.
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
    ///
    /// The root is domain-separated (using "merkle-leaf" / "merkle-node"
    /// prefixes) to match `AuditChain`'s persisted `merkle_root` values.
    pub fn build_root(&self) -> Sha256Hash {
        let mut level: Vec<DomainHash> = self.leaves.iter().map(leaf_domain_hash).collect();
        while level.len() > 1 {
            level = level
                .chunks(2)
                .map(|pair| {
                    let left = pair[0];
                    let right = pair.get(1).copied().unwrap_or(left);
                    node_domain_hash(&left, &right)
                })
                .collect();
        }
        level[0].0
    }

    /// Return the sibling hashes from `leaf_index` up to the root (proof of
    /// inclusion). The caller combines this path with the target leaf to
    /// recompute the root and confirm inclusion.
    ///
    /// Returns an empty `Vec` when there is exactly one leaf (the root equals
    /// the domain-hashed single leaf; no siblings needed).
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
        let mut level: Vec<DomainHash> = self.leaves.iter().map(leaf_domain_hash).collect();
        let mut idx = leaf_index;
        while level.len() > 1 {
            let sibling = if idx.is_multiple_of(2) {
                // Right sibling; duplicate if this is the last (odd) leaf.
                level.get(idx + 1).copied().unwrap_or(level[idx])
            } else {
                // Left sibling always exists when index is odd.
                level[idx - 1]
            };
            proof.push(sibling.0);
            level = level
                .chunks(2)
                .map(|pair| {
                    let left = pair[0];
                    let right = pair.get(1).copied().unwrap_or(left);
                    node_domain_hash(&left, &right)
                })
                .collect();
            idx /= 2;
        }
        proof
    }

    /// Verify that `leaf` at `leaf_index` is included in `root` using the
    /// given `proof_path` (as returned by [`MerkleTree::proof_path`]).
    ///
    /// Returns `false` when:
    /// - `leaf_index >= leaf_count` (out-of-range index for the committed tree
    ///   size), preventing index-999 false positives on single-leaf trees.
    /// - The proof depth does not match the expected depth for `leaf_count`
    ///   (prevents shortened proofs from certifying internal nodes as leaves).
    /// - The recomputed root does not equal `root`.
    pub fn verify_proof(
        leaf: Sha256Hash,
        leaf_index: usize,
        proof_path: &[Sha256Hash],
        root: Sha256Hash,
        leaf_count: usize,
    ) -> bool {
        // Reject out-of-range leaf indexes.
        if leaf_count == 0 || leaf_index >= leaf_count {
            return false;
        }
        // Reject proofs whose depth doesn't match the tree size.
        // A tree of `n` leaves has ⌈log2(n)⌉ levels above the leaf row.
        let expected_depth = expected_proof_depth(leaf_count);
        if proof_path.len() != expected_depth {
            return false;
        }
        // Recompute from leaf domain-hash upward.
        let mut current = leaf_domain_hash(&leaf);
        let mut idx = leaf_index;
        for &sibling_raw in proof_path {
            let sibling = DomainHash(sibling_raw);
            current = if idx.is_multiple_of(2) {
                node_domain_hash(&current, &sibling)
            } else {
                node_domain_hash(&sibling, &current)
            };
            idx /= 2;
        }
        current.0 == root
    }
}

/// Compute the expected proof depth for a tree with `n` leaves.
///
/// This is the number of levels above the leaf row, i.e. ⌈log2(n)⌉
/// (with the special case that a single-leaf tree has depth 0).
fn expected_proof_depth(n: usize) -> usize {
    if n <= 1 {
        return 0;
    }
    // Count levels: keep halving (rounding up for odd) until we reach 1.
    let mut count = n;
    let mut depth = 0;
    while count > 1 {
        count = count.div_ceil(2);
        depth += 1;
    }
    depth
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    fn leaf(data: &[u8]) -> Sha256Hash {
        Sha256::digest(data).into()
    }

    // ── build_root tests ───────────────────────────────────────────────────

    #[test]
    fn single_leaf_root_equals_leaf_domain_hash() {
        let l = leaf(b"only");
        let tree = MerkleTree::new(vec![l]);
        // Single-leaf root = leaf_domain_hash(l).0
        assert_eq!(tree.build_root(), leaf_domain_hash(&l).0);
    }

    #[test]
    fn two_leaves_deterministic_root() {
        let a = leaf(b"a");
        let b = leaf(b"b");
        let tree = MerkleTree::new(vec![a, b]);
        let root = tree.build_root();

        // Recompute manually using the same domain-separation.
        let la = leaf_domain_hash(&a);
        let lb = leaf_domain_hash(&b);
        let expected = node_domain_hash(&la, &lb).0;
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
        // Level 1: [node(leaf(L0), leaf(L1)), node(leaf(L2), leaf(L2))]
        // Level 2 (root): node(N01, N22)
        let dl: Vec<DomainHash> = leaves.iter().map(leaf_domain_hash).collect();
        let n01 = node_domain_hash(&dl[0], &dl[1]);
        let n22 = node_domain_hash(&dl[2], &dl[2]);
        let expected = node_domain_hash(&n01, &n22).0;

        let tree = MerkleTree::new(leaves);
        assert_eq!(tree.build_root(), expected);
    }

    // ── proof_path + verify_proof tests ───────────────────────────────────

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
                MerkleTree::verify_proof(leaf_hash, idx, &path, root, tree.len()),
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
                MerkleTree::verify_proof(leaf_hash, idx, &path, root, tree.len()),
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
                MerkleTree::verify_proof(leaf_hash, idx, &path, root, tree.len()),
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
        let mut bad_leaf = leaves[0];
        bad_leaf[0] ^= 0xff;
        assert!(
            !MerkleTree::verify_proof(bad_leaf, 0, &path, root, tree.len()),
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
            !MerkleTree::verify_proof(leaves[1], 1, &path, root, tree.len()),
            "tampered root must fail proof"
        );
    }

    // ── security / adversarial tests (Threads 2 & 3 regression anchors) ──

    #[test]
    fn out_of_range_leaf_index_rejected() {
        // Thread 2: index 999 on a single-leaf tree must not verify.
        let l = leaf(b"solo");
        let tree = MerkleTree::new(vec![l]);
        let root = tree.build_root();
        // Empty proof + leaf == root would have passed the old implementation.
        assert!(
            !MerkleTree::verify_proof(root, 999, &[], root, 1),
            "out-of-range index 999 must be rejected"
        );
    }

    #[test]
    fn internal_node_cannot_verify_as_leaf() {
        // Thread 3: a shortened proof must not certify an internal node as a
        // leaf. In a 4-leaf tree, H(L0||L1) with proof [H(L2||L3)] must fail.
        let leaves: Vec<Sha256Hash> = (0u8..4).map(|i| leaf(&[i])).collect();
        let tree = MerkleTree::new(leaves.clone());
        let root = tree.build_root();
        // Compute the internal node H(L0, L1) using domain hashing.
        let dl: Vec<DomainHash> = leaves.iter().map(leaf_domain_hash).collect();
        let internal_01 = node_domain_hash(&dl[0], &dl[1]).0;
        let internal_23 = node_domain_hash(&dl[2], &dl[3]).0;
        // A shortened proof with only 1 sibling (instead of required 2 for 4-leaf tree).
        let shortened_proof = vec![internal_23];
        assert!(
            !MerkleTree::verify_proof(internal_01, 0, &shortened_proof, root, 4),
            "internal node with shortened proof must not verify as leaf"
        );
    }

    #[test]
    fn expected_proof_depth_values() {
        assert_eq!(expected_proof_depth(1), 0);
        assert_eq!(expected_proof_depth(2), 1);
        assert_eq!(expected_proof_depth(3), 2);
        assert_eq!(expected_proof_depth(4), 2);
        assert_eq!(expected_proof_depth(5), 3);
        assert_eq!(expected_proof_depth(8), 3);
        assert_eq!(expected_proof_depth(9), 4);
    }

    #[test]
    fn zero_leaf_count_rejected_by_verify() {
        let l = leaf(b"x");
        assert!(
            !MerkleTree::verify_proof(l, 0, &[], l, 0),
            "leaf_count=0 must always return false"
        );
    }
}
