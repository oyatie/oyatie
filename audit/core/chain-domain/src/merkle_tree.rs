//! Public `MerkleTree`: RFC 6962 §2.1 Merkle Tree Hash, with inclusion
//! (audit-path) proofs.
//!
//! This is a standalone, general-purpose Merkle tree over opaque 32-byte
//! leaf values, for callers that need inclusion proofs (e.g.
//! `audit-sealing-domain`'s seal records). It is **not** the same
//! construction as `lib.rs`'s private `merkle_root()` helper, which computes
//! a different, length-prefixed-domain-string scheme used only to chain
//! `AuditEvent.merkle_root` values inside this crate's own append-only event
//! log. The two are intentionally independent: `AuditChain`'s own hash chain
//! is out of scope for this module.
//!
//! ## The RFC 6962 §2.1 Merkle Tree Hash (MTH)
//!
//! For a list of `n` leaves `D[n] = {d(0), d(1), ..., d(n-1)}`:
//!
//! ```text
//! MTH({})       = SHA-256("")
//! MTH({d(0)})   = SHA-256(0x00 || d(0))
//! MTH(D[n])     = SHA-256(0x01 || MTH(D[0:k]) || MTH(D[k:n]))   for n > 1
//! ```
//!
//! where `k` is the **largest power of two strictly less than `n`** (so the
//! left subtree `D[0:k]` is always a perfect power-of-two tree and the split
//! point depends only on `n`, never on leaf content). The single-byte `0x00`
//! / `0x01` prefixes domain-separate leaf hashes from internal-node hashes:
//! no internal-node hash can ever be presented as a valid leaf hash (or vice
//! versa), because they are the outputs of disjoint hash-input spaces.
//!
//! Crucially, this k-split is **not** the same tree produced by naive
//! pairwise reduction with promotion of an odd trailing element (i.e.
//! `right = pair.get(1).unwrap_or(left)`). That construction is the
//! CVE-2012-2459 shape: it lets an `n`-leaf list and an `(n+1)`-leaf list
//! (formed by literally repeating the last leaf) reduce to the same pair at
//! some level and therefore commit to the **same root**, even though they
//! commit to different leaf counts. RFC 6962's k-split has no such
//! collision: appending a leaf always changes which subtree it falls under
//! and how many leaves that subtree covers, so the two trees diverge
//! structurally from the first level that differs, and the `0x00`/`0x01`
//! domain separation prevents a leaf-hash and a node-hash from ever
//! coinciding as an escape hatch.
//!
//! ## Audit (inclusion) paths
//!
//! `PATH(m, D[n])`, the audit path for leaf index `m`:
//!
//! ```text
//! PATH(0, {d(0)}) = {}
//! PATH(m, D[n])   = PATH(m, D[0:k])   : MTH(D[k:n])   for n > 1, m < k
//!                 = PATH(m - k, D[k:n]) : MTH(D[0:k]) for n > 1, m >= k
//! ```
//!
//! (`:` is list-append onto the end.) Because the split point `k` depends
//! only on `n`, the path length varies with leaf index whenever `n` is not a
//! power of two — there is no single uniform "tree depth" the way there is
//! for a perfect binary tree. [`MerkleTree::verify_proof`] computes the
//! expected path length as a function of *both* `leaf_index` and
//! `leaf_count` and rejects any proof of the wrong length before attempting
//! to recompute a root, so a shortened or extended proof always fails
//! closed rather than silently verifying against the wrong structure.
//!
//! ## Leaf ordering
//!
//! Callers MUST sort leaves before constructing `MerkleTree` when
//! determinism across independent nodes is required (per Bominal ADR-0028).
//! This type is ordering-agnostic to keep the type minimal.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use sha2::{Digest, Sha256};

/// A 32-byte SHA-256 hash. Used both for raw (caller-supplied) leaf values
/// and for the leaf/node hashes this module computes from them.
pub type Sha256Hash = [u8; 32];

/// `SHA-256(0x00 || leaf)` — RFC 6962 §2.1's leaf hash.
fn leaf_hash(leaf: &Sha256Hash) -> Sha256Hash {
    let mut hasher = Sha256::new();
    hasher.update([0x00]);
    hasher.update(leaf);
    hasher.finalize().into()
}

/// `SHA-256(0x01 || left || right)` — RFC 6962 §2.1's internal-node hash.
fn node_hash(left: &Sha256Hash, right: &Sha256Hash) -> Sha256Hash {
    let mut hasher = Sha256::new();
    hasher.update([0x01]);
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

/// The largest power of two strictly less than `n`. Only meaningful for
/// `n > 1`; RFC 6962's MTH/PATH recursions never call this for `n <= 1`.
fn largest_power_of_two_less_than(n: usize) -> usize {
    debug_assert!(n > 1, "largest_power_of_two_less_than requires n > 1");
    let mut k = 1_usize;
    while k * 2 < n {
        k *= 2;
    }
    k
}

/// RFC 6962 §2.1 `MTH(D[n])` over `leaves`, computed directly from the
/// recursive definition. `leaves.len() == 0` returns `SHA-256("")` (the
/// empty-tree hash); `MerkleTree` itself never holds zero leaves (see
/// [`MerkleTree::new`]'s panic contract), but this free function is total
/// over every `n >= 0` so the empty case is exercised directly in tests.
fn mth(leaves: &[Sha256Hash]) -> Sha256Hash {
    match leaves.len() {
        0 => Sha256::digest([]).into(),
        1 => leaf_hash(&leaves[0]),
        n => {
            let k = largest_power_of_two_less_than(n);
            let left = mth(&leaves[..k]);
            let right = mth(&leaves[k..]);
            node_hash(&left, &right)
        }
    }
}

/// RFC 6962 §2.1 `PATH(leaf_index, leaves)`: the audit path from
/// `leaf_index` up to the root, in leaf-to-root order (the sibling closest
/// to the leaf comes first, the sibling at the outermost k-split comes
/// last). Requires `leaves.len() > 1`; the `n <= 1` base case (an empty
/// path) is handled by the caller before recursing.
fn audit_path(leaf_index: usize, leaves: &[Sha256Hash]) -> Vec<Sha256Hash> {
    let n = leaves.len();
    if n <= 1 {
        return Vec::new();
    }
    let k = largest_power_of_two_less_than(n);
    if leaf_index < k {
        let mut path = audit_path(leaf_index, &leaves[..k]);
        path.push(mth(&leaves[k..]));
        path
    } else {
        let mut path = audit_path(leaf_index - k, &leaves[k..]);
        path.push(mth(&leaves[..k]));
        path
    }
}

/// The exact RFC 6962 audit-path length for `leaf_index` in a tree of
/// `leaf_count` leaves. Unlike a perfect binary tree, this is **not** a
/// single function of `leaf_count` alone when `leaf_count` is not a power of
/// two — different leaf indices at the same `leaf_count` can have different
/// path lengths, because the k-split is unbalanced. Requires
/// `leaf_index < leaf_count`; callers must check that first (see
/// [`MerkleTree::verify_proof`]).
fn expected_proof_depth(leaf_index: usize, leaf_count: usize) -> usize {
    if leaf_count <= 1 {
        return 0;
    }
    let k = largest_power_of_two_less_than(leaf_count);
    if leaf_index < k {
        1 + expected_proof_depth(leaf_index, k)
    } else {
        1 + expected_proof_depth(leaf_index - k, leaf_count - k)
    }
}

/// Recompute the RFC 6962 root implied by `leaf` at `leaf_index`, given an
/// audit path of exactly the right length for `(leaf_index, leaf_count)`.
///
/// Mirrors [`audit_path`]'s own recursive structure in reverse: at each
/// level, the *last* remaining path entry is the sibling contributed at that
/// level's k-split (it was appended last while descending, so it is
/// consumed first while re-ascending), and every entry before it belongs to
/// the recursive call for the subtree `leaf_index` actually falls in.
///
/// # Panics
/// Never, when called with `path.len() == expected_proof_depth(leaf_index,
/// leaf_count)` as [`MerkleTree::verify_proof`] guarantees before calling
/// this: the split at each recursive level always leaves exactly one
/// trailing element for `split_at(path.len() - 1)`, by construction of
/// [`expected_proof_depth`].
fn reconstruct_root(
    leaf_index: usize,
    leaf_count: usize,
    path: &[Sha256Hash],
    leaf: Sha256Hash,
) -> Sha256Hash {
    if leaf_count <= 1 {
        return leaf_hash(&leaf);
    }
    let k = largest_power_of_two_less_than(leaf_count);
    let (inner_path, sibling) = path.split_at(path.len() - 1);
    let sibling = sibling[0];
    if leaf_index < k {
        let inner = reconstruct_root(leaf_index, k, inner_path, leaf);
        node_hash(&inner, &sibling)
    } else {
        let inner = reconstruct_root(leaf_index - k, leaf_count - k, inner_path, leaf);
        node_hash(&sibling, &inner)
    }
}

/// RFC 6962 §2.1 Merkle Tree Hash, with inclusion (audit-path) proofs, over
/// opaque 32-byte leaf values.
///
/// # Panics
///
/// [`MerkleTree::new`] panics when `leaves` is empty — a tree with no leaves
/// has no defined root under this type's contract, and callers must guard
/// against this (see `audit-sealing-domain::MerkleTreeEngine::root`, which
/// checks emptiness itself and returns a typed error before ever calling
/// `new`).
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

    /// Compute the RFC 6962 `MTH` root over this tree's leaves.
    pub fn build_root(&self) -> Sha256Hash {
        mth(&self.leaves)
    }

    /// Return the RFC 6962 audit path for `leaf_index` (the sibling hashes
    /// needed to recompute the root from that leaf), in leaf-to-root order.
    ///
    /// Returns an empty `Vec` when there is exactly one leaf (the root
    /// equals that leaf's hash directly; no siblings needed).
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
        audit_path(leaf_index, &self.leaves)
    }

    /// Verify that `leaf` at `leaf_index` is included in `root` using the
    /// given `proof_path` (as returned by [`MerkleTree::proof_path`]), for a
    /// tree that committed to `leaf_count` leaves in total.
    ///
    /// Fails closed (returns `false`, never panics or widens acceptance) on:
    /// - `leaf_count == 0` — an empty tree has no root any leaf can belong to.
    /// - `leaf_index >= leaf_count` — out-of-range index for the committed
    ///   tree size (prevents e.g. index-999 false positives on a
    ///   single-leaf tree).
    /// - `proof_path.len() != expected_proof_depth(leaf_index, leaf_count)`
    ///   — the exact RFC 6962 audit-path length for this `(leaf_index,
    ///   leaf_count)` pair, which (unlike a perfect binary tree) varies by
    ///   leaf index whenever `leaf_count` is not a power of two. A
    ///   truncated or extended proof is rejected before any hashing, so it
    ///   can never be padded or trimmed into certifying an internal node as
    ///   a leaf, or vice versa.
    /// - The recomputed root not equal to `root`.
    pub fn verify_proof(
        leaf: Sha256Hash,
        leaf_index: usize,
        proof_path: &[Sha256Hash],
        root: Sha256Hash,
        leaf_count: usize,
    ) -> bool {
        if leaf_count == 0 || leaf_index >= leaf_count {
            return false;
        }
        if proof_path.len() != expected_proof_depth(leaf_index, leaf_count) {
            return false;
        }
        reconstruct_root(leaf_index, leaf_count, proof_path, leaf) == root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    fn leaf(data: &[u8]) -> Sha256Hash {
        Sha256::digest(data).into()
    }

    fn leaves(n: u32) -> Vec<Sha256Hash> {
        (0..n).map(|i| leaf(&i.to_be_bytes())).collect()
    }

    fn to_hex(bytes: &Sha256Hash) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    // ── independent reference implementation (oracle) ─────────────────────
    //
    // A deliberately naive, allocating transcription of RFC 6962 §2.1,
    // written independently of `mth`/`node_hash`/`leaf_hash` above: it
    // builds full owned `Vec<u8>` buffers and hashes them in one shot,
    // rather than sharing any incremental-hasher plumbing with the
    // production path. Two structurally different implementations agreeing
    // on every leaf count from 0 through 33 is a much stronger oracle than
    // a handful of hand-copied vectors.
    fn reference_mth(leaves: &[Sha256Hash]) -> Vec<u8> {
        if leaves.is_empty() {
            return Sha256::digest([]).to_vec();
        }
        if leaves.len() == 1 {
            let mut buf = vec![0x00_u8];
            buf.extend_from_slice(&leaves[0]);
            return Sha256::digest(&buf).to_vec();
        }
        let n = leaves.len();
        let mut k = 1_usize;
        while k * 2 < n {
            k *= 2;
        }
        let left = reference_mth(&leaves[..k]);
        let right = reference_mth(&leaves[k..]);
        let mut buf = vec![0x01_u8];
        buf.extend_from_slice(&left);
        buf.extend_from_slice(&right);
        Sha256::digest(&buf).to_vec()
    }

    #[test]
    fn production_mth_matches_reference_for_every_count_zero_through_thirty_three() {
        for n in 0_u32..=33 {
            let ls = leaves(n);
            let reference = reference_mth(&ls);
            let production = mth(&ls);
            assert_eq!(production.to_vec(), reference, "mismatch at leaf count {n}");
        }
    }

    #[test]
    fn production_build_root_matches_reference_for_every_count_one_through_thirty_three() {
        for n in 1_u32..=33 {
            let ls = leaves(n);
            let reference = reference_mth(&ls);
            let tree = MerkleTree::new(ls);
            assert_eq!(
                tree.build_root().to_vec(),
                reference,
                "MerkleTree::build_root mismatch at leaf count {n}"
            );
        }
    }

    // ── golden values (pin against reference/production drifting together) ─

    #[test]
    fn empty_tree_root_is_sha256_of_empty_string() {
        // Well-known constant: SHA-256("") = e3b0c442...b855 is the standard
        // published test vector for the empty-input SHA-256 digest,
        // independent of anything in this crate.
        let expected = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        assert_eq!(to_hex(&Sha256::digest([]).into()), expected);
        assert_eq!(to_hex(&mth(&[])), expected);
    }

    #[test]
    fn golden_root_for_three_leaves() {
        // Pinned literal, independent of `reference_mth`: computed once from
        // this exact production implementation and frozen here so the
        // production code and the in-file reference oracle cannot silently
        // drift together and both be wrong the same way.
        let tree = MerkleTree::new(leaves(3));
        assert_eq!(
            to_hex(&tree.build_root()),
            "b1782769b8906e20ab9b2648e93f64b15e98d20ebdc70e9d5dd2737378bfa0cd"
        );
    }

    // ── the fixed CVE-2012-2459-shaped collision is gone ──────────────────

    #[test]
    fn three_leaves_and_trailing_duplicate_four_leaves_do_not_collide() {
        let three = leaves(3);
        let mut four = leaves(3);
        four.push(*four.last().expect("three leaves is non-empty"));

        let root_three = MerkleTree::new(three).build_root();
        let root_four = MerkleTree::new(four).build_root();
        assert_ne!(
            root_three, root_four,
            "RFC 6962 k-split must not let a 3-leaf set and its \
             trailing-duplicate 4-leaf extension collide"
        );
    }

    #[test]
    fn no_trailing_duplicate_collision_for_any_count_up_to_thirty_three() {
        for n in 1_u32..=33 {
            let base = leaves(n);
            let mut extended = base.clone();
            extended.push(*extended.last().expect("n >= 1"));

            let root_base = MerkleTree::new(base).build_root();
            let root_extended = MerkleTree::new(extended).build_root();
            assert_ne!(
                root_base, root_extended,
                "n={n} vs n+1 (trailing duplicate) must not collide"
            );
        }
    }

    // ── leaf/node domain separation ────────────────────────────────────────

    #[test]
    fn internal_node_hash_cannot_verify_as_a_leaf_hash() {
        let a = leaf(b"a");
        let b = leaf(b"b");
        let internal = node_hash(&leaf_hash(&a), &leaf_hash(&b));
        // A single-leaf tree containing exactly `internal` as its leaf value
        // has root == leaf_hash(internal), which the 0x01-prefixed
        // `internal` value must never equal.
        let single_leaf_root = MerkleTree::new(vec![internal]).build_root();
        assert_ne!(
            single_leaf_root, internal,
            "an internal-node hash re-hashed as a leaf must not equal the raw node hash"
        );
        assert_eq!(single_leaf_root, leaf_hash(&internal));
    }

    // ── build_root basics ──────────────────────────────────────────────────

    #[test]
    fn single_leaf_root_equals_leaf_hash() {
        let l = leaf(b"only");
        let tree = MerkleTree::new(vec![l]);
        assert_eq!(tree.build_root(), leaf_hash(&l));
    }

    #[test]
    fn same_leaves_same_order_same_root() {
        let ls = leaves(8);
        let t1 = MerkleTree::new(ls.clone());
        let t2 = MerkleTree::new(ls);
        assert_eq!(t1.build_root(), t2.build_root());
    }

    #[test]
    fn reordered_leaves_change_the_root() {
        let mut forward = leaves(4);
        let backward: Vec<_> = {
            forward.reverse();
            forward.clone()
        };
        let forward_root = MerkleTree::new(leaves(4)).build_root();
        let backward_root = MerkleTree::new(backward).build_root();
        assert_ne!(forward_root, backward_root);
    }

    // ── proof_path + verify_proof: round-trip for every n, every index ────

    #[test]
    fn proof_path_single_leaf_is_empty() {
        let l = leaf(b"solo");
        let tree = MerkleTree::new(vec![l]);
        assert!(tree.proof_path(0).is_empty());
    }

    #[test]
    fn every_leaf_at_every_count_up_to_thirty_three_round_trips() {
        for n in 1_u32..=33 {
            let ls = leaves(n);
            let tree = MerkleTree::new(ls.clone());
            let root = tree.build_root();
            for (idx, &l) in ls.iter().enumerate() {
                let path = tree.proof_path(idx);
                assert!(
                    MerkleTree::verify_proof(l, idx, &path, root, tree.len()),
                    "n={n} leaf={idx} must verify"
                );
            }
        }
    }

    #[test]
    fn tampered_leaf_fails_verification() {
        let ls = leaves(4);
        let tree = MerkleTree::new(ls.clone());
        let root = tree.build_root();
        let path = tree.proof_path(0);
        let mut bad_leaf = ls[0];
        bad_leaf[0] ^= 0xff;
        assert!(!MerkleTree::verify_proof(
            bad_leaf,
            0,
            &path,
            root,
            tree.len()
        ));
    }

    #[test]
    fn tampered_root_fails_verification() {
        let ls = leaves(4);
        let tree = MerkleTree::new(ls.clone());
        let mut root = tree.build_root();
        let path = tree.proof_path(1);
        root[31] ^= 0xff;
        assert!(!MerkleTree::verify_proof(ls[1], 1, &path, root, tree.len()));
    }

    #[test]
    fn wrong_index_fails_verification() {
        let ls = leaves(5);
        let tree = MerkleTree::new(ls.clone());
        let root = tree.build_root();
        let path = tree.proof_path(2);
        // Path for leaf 2 presented against leaf 2's value but a different
        // claimed index must fail (either a length mismatch, since path
        // length can vary by index for non-power-of-two n, or a root
        // mismatch if lengths happen to coincide).
        assert!(!MerkleTree::verify_proof(ls[2], 3, &path, root, tree.len()));
    }

    #[test]
    fn truncated_path_fails_verification() {
        let ls = leaves(8);
        let tree = MerkleTree::new(ls.clone());
        let root = tree.build_root();
        let mut path = tree.proof_path(5);
        path.pop();
        assert!(!MerkleTree::verify_proof(ls[5], 5, &path, root, tree.len()));
    }

    #[test]
    fn extended_path_fails_verification() {
        let ls = leaves(8);
        let tree = MerkleTree::new(ls.clone());
        let root = tree.build_root();
        let mut path = tree.proof_path(5);
        path.push(path[0]);
        assert!(!MerkleTree::verify_proof(ls[5], 5, &path, root, tree.len()));
    }

    // ── path length varies by leaf index for non-power-of-two n ───────────

    #[test]
    fn expected_proof_depth_varies_by_leaf_index_for_non_power_of_two_counts() {
        // n=3, k=2: leaves 0 and 1 sit two levels deep under the left
        // (perfect, size-2) subtree; leaf 2 sits alone under the right
        // subtree of size 1 (depth 1 total).
        assert_eq!(expected_proof_depth(0, 3), 2);
        assert_eq!(expected_proof_depth(1, 3), 2);
        assert_eq!(expected_proof_depth(2, 3), 1);

        // n=5, k=4: leaves 0..4 sit three levels deep under the left
        // (perfect, size-4) subtree; leaf 4 sits alone under the right
        // subtree of size 1 (depth 1 total).
        assert_eq!(expected_proof_depth(0, 5), 3);
        assert_eq!(expected_proof_depth(3, 5), 3);
        assert_eq!(expected_proof_depth(4, 5), 1);
    }

    #[test]
    fn expected_proof_depth_matches_actual_proof_path_length_for_every_index_up_to_thirty_three() {
        for n in 1_u32..=33 {
            let tree = MerkleTree::new(leaves(n));
            for idx in 0..n as usize {
                assert_eq!(
                    tree.proof_path(idx).len(),
                    expected_proof_depth(idx, n as usize),
                    "n={n} idx={idx}"
                );
            }
        }
    }

    // ── fail-closed edge cases (L4: never saturate, never widen) ──────────

    #[test]
    fn out_of_range_leaf_index_rejected() {
        let l = leaf(b"solo");
        let tree = MerkleTree::new(vec![l]);
        let root = tree.build_root();
        assert!(!MerkleTree::verify_proof(root, 999, &[], root, 1));
    }

    #[test]
    fn zero_leaf_count_rejected_by_verify() {
        let l = leaf(b"x");
        assert!(!MerkleTree::verify_proof(l, 0, &[], l, 0));
    }

    #[test]
    fn shortened_proof_cannot_certify_an_internal_node_as_a_leaf() {
        // In a 4-leaf tree, the internal node H(L0||L1) presented as leaf 0
        // with only the (shorter) sibling for the *other* half must fail.
        let ls = leaves(4);
        let tree = MerkleTree::new(ls.clone());
        let root = tree.build_root();
        let internal_01 = node_hash(&leaf_hash(&ls[0]), &leaf_hash(&ls[1]));
        let internal_23 = node_hash(&leaf_hash(&ls[2]), &leaf_hash(&ls[3]));
        let shortened_proof = vec![internal_23];
        assert!(!MerkleTree::verify_proof(
            internal_01,
            0,
            &shortened_proof,
            root,
            4
        ));
    }
}
