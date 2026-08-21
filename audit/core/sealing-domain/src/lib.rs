//! Audit-chain sealing domain: seal-record construction, `SealStatus`
//! lifecycle transitions, and `PackEpoch` signing-key coverage checks.
//!
//! ## What this crate owns
//!
//! `audit-sealing-domain` is the pure (I/O-free) `core/*-domain` crate for
//! the sealing capability. It sits on top of two crates it does not
//! reimplement:
//!
//! - [`audit_chain_domain::MerkleTree`] — the SHA-256 Merkle-tree math (root
//!   computation and inclusion proofs). This crate wraps it behind the
//!   `MerkleEngine` port ([`merkle_engine::MerkleTreeEngine`]) instead of
//!   re-deriving any hashing.
//! - `audit_sealing_kernel` — the port-only crate holding the pure
//!   `SigningKeyRef` / `PackEpoch` / `SealStatus` / `SealRecord` types and
//!   the `MerkleEngine` / `SignerPort` / `RootPublisher` / `IndexWriter` /
//!   `ObjectStoreWriter` trait ports this crate's callers compose against.
//!
//! ## Merkle scheme actually in use (RFC 6962 §2.1, via `audit_chain_domain`)
//!
//! [`audit_chain_domain::MerkleTree`] implements RFC 6962 §2.1's Merkle Tree
//! Hash (`MTH`) exactly: single-byte `0x00` (leaf) / `0x01` (node) domain
//! prefixes, and the `MTH(D[n]) = SHA-256(0x01 || MTH(D[0:k]) || MTH(D[k:n]))`
//! split at `k` = the largest power of two strictly less than `n`, rather
//! than a naive pairwise reduction that promotes or duplicates a lone
//! odd-level node. An earlier revision of `audit_chain_domain` used the
//! latter (duplicate-the-lone-node) construction, which reproduced the
//! CVE-2012-2459 shape: a 3-leaf set `[a,b,c]` and the 4-leaf set
//! `[a,b,c,c]` reduced to the same root despite committing to different
//! leaf counts. That has been fixed at the source (`audit_chain_domain`'s
//! own test suite pins the RFC 6962 `MTH`/`PATH` recursion against an
//! independent reference implementation for every leaf count from 0 through
//! at least 33, and regression-tests that a trailing-duplicate extension no
//! longer collides with its base set), so this crate no longer needs — and
//! no longer applies — a leaf-shape workaround at the seal-build boundary:
//! [`merkle_engine::MerkleTreeEngine::root`] passes every non-empty leaf
//! slice straight through to `MerkleTree::build_root`, including one whose
//! last two leaves happen to be identical. A `SealRecord`'s whole purpose is
//! that its `merkle_root` uniquely attests to the leaf set (and
//! `leaf_count`) it was built from; RFC 6962's domain-separated k-split
//! provides that for a fixed, non-adversarially-mutated leaf set — the
//! construction this crate's [`build_seal_record`] and
//! [`merkle_engine::verify_leaf_inclusion`] rely on.
//!
//! `audit/capabilities/seal-mint.yaml` describes this Merkle tree as
//! "RFC-6962-shaped" with
//! `eval_metric: merkle-correctness-against-RFC-6962-reference-vectors`.
//! That description is now accurate for `audit_chain_domain::MerkleTree`;
//! reconciling the capability file itself is outside this crate's
//! ownership.
//!
//! ## What this crate does NOT do
//!
//! No PKCS#11, S3, Postgres, Mimir, or HTTP calls — those live behind the
//! adapter crates that implement the `SignerPort` / `RootPublisher` /
//! `IndexWriter` / `ObjectStoreWriter` ports from `audit_sealing_kernel`.
//! Every function in this crate is a pure function of its arguments, or
//! validates an explicit, typed attestation the caller supplies (e.g.
//! [`seal_record::PriorPeriod`]) — a pure domain crate has no read path of
//! its own to confirm what actually happened before.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![allow(dead_code)]

pub mod epoch;
pub mod merkle_engine;
pub mod seal_record;
pub mod status;

pub use audit_chain_domain::{MerkleTree, Sha256Hash};
pub use audit_sealing_kernel::{PackEpoch, SealRecord, SealStatus, SigningKeyRef};

pub use epoch::verify_epoch_covers_period;
pub use merkle_engine::{MerkleTreeEngine, verify_leaf_inclusion};
pub use seal_record::{PriorPeriod, PriorPeriodLookup, SealRecordInput, build_seal_record};
pub use status::{apply_seal_status_transition, transition_seal_status};

/// Domain-level seal error variants.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SealingDomainError {
    /// [`build_seal_record`]: `pack` was empty or all-whitespace.
    EmptyPack,
    /// [`build_seal_record`]: `tenant_partition` was empty or all-whitespace.
    EmptyTenantPartition,
    /// [`MerkleTreeEngine::root`]: the leaf set was empty — a tree with no
    /// leaves has no defined root ([`MerkleTree::new`] would panic on it).
    InvalidLeafCount,
    /// [`verify_leaf_inclusion`]: the supplied inclusion proof did not
    /// recompute to the claimed root — a tampered leaf, a tampered or
    /// truncated proof path, an out-of-range leaf index, or a proof depth
    /// that does not match the committed leaf count.
    InvalidProofPath,
    /// [`build_seal_record`]: the caller's declared leaf count did not equal
    /// `leaves.len()`. Rejected rather than silently trusting either value:
    /// a mismatch means the caller's batch accumulator and the leaves it
    /// actually handed over have already disagreed before any hash runs.
    LeafCountMismatch { declared: u64, actual: u64 },
    /// [`build_seal_record`]: [`PriorPeriod::Preceding`] carried an empty or
    /// all-whitespace root string, which is not a valid chain reference.
    EmptyPriorRoot,
    /// [`build_seal_record`]: [`PriorPeriod::Preceding`] carried a non-empty
    /// root string that is not shaped like a root this crate's own
    /// `encode_root` could ever emit (`sha256:` followed by exactly 64
    /// lowercase hex characters). This crate cannot verify that a prior-root
    /// string is the REAL prior period's root (it has no read path), but it
    /// can and does reject values that are structurally impossible chain
    /// references.
    MalformedPriorRoot { root: String },
    /// [`build_seal_record`]: [`PriorPeriod::Preceding`]'s root string was
    /// well-formed but identical to the record's own freshly computed
    /// `merkle_root` — a record cannot chain to itself.
    SelfReferentialPriorRoot,
    /// [`build_seal_record`]: the caller claimed [`PriorPeriod::First`] for
    /// `(pack, tenant_partition)`, but the supplied [`PriorPeriodLookup`]
    /// reports a sealed period already exists there. A false firstness claim
    /// would otherwise seal a record with `prior_root: None` that is not
    /// actually the start of the chain, defeating tamper-evidence between
    /// periods.
    FalseFirstPeriodClaim {
        pack: String,
        tenant_partition: String,
    },
    /// [`transition_seal_status`]: `to` is not a legal successor of `from`
    /// in the `Accepted -> Unsealed -> Sealed -> Published -> Verified`
    /// lifecycle (no backward moves, no skipped stages, no exits from the
    /// terminal `Redacted` / `Retained` states).
    IllegalSealStatusTransition { from: SealStatus, to: SealStatus },
    /// [`verify_epoch_covers_period`]: the epoch's `pack` does not match the
    /// pack being checked. An epoch scoped to one pack must never be treated
    /// as covering a period for another, regardless of its window.
    EpochPackMismatch {
        epoch_pack: String,
        record_pack: String,
    },
    /// [`verify_epoch_covers_period`]: the epoch's `tenant_partition` does
    /// not match the tenant partition being checked, for the same reason.
    EpochTenantPartitionMismatch {
        epoch_tenant_partition: String,
        record_tenant_partition: String,
    },
    /// [`verify_epoch_covers_period`]: the `active_key` signed a period
    /// outside the epoch's half-open `[period_lo, period_hi)` window.
    PeriodOutsideEpochWindow {
        period: String,
        period_lo: String,
        period_hi: String,
    },
    /// [`verify_epoch_covers_period`]: the `retiring_key` signed a period
    /// outside the epoch's own `[period_lo, period_hi)` window. A retiring
    /// key's grace period never extends past the epoch that names it.
    RetiringKeyOutsideEpochWindow {
        key_id: String,
        period: String,
        period_lo: String,
        period_hi: String,
    },
    /// [`verify_epoch_covers_period`]: the signing key is neither the
    /// epoch's `active_key` nor its `retiring_key`.
    SigningKeyNotInEpoch { key_id: String },
}
