//! Audit-chain verification domain: pure proof + signature verification.
//!
//! This crate is the read-side counterpart to `audit-sealing-domain`'s
//! write side. Given a claim about one sealed period — an Ed25519
//! signature, a Merkle inclusion proof for one leaf, a claimed prior root,
//! and the `(pack, tenant_partition, period_id)` identity the claim was
//! sealed under — [`verify`] answers exactly one question: does this claim
//! hold, and if not, which of a closed set of ways does it fail in.
//!
//! [`verify`] runs, in order: resolve the trusted signing key for the
//! request's verification context, verify the Ed25519 signature over the
//! record's own canonical payload, verify the Merkle inclusion proof,
//! confirm the prior period's published root actually chains (or that the
//! record's claim to be the first period is independently confirmed —
//! never taken on the claim's word alone), reject any mismatch across the
//! full `(pack, tenant_partition, period_id)` identity tuple between the
//! verification context and the record's own self-asserted identity, and
//! finally — only once every one of those has genuinely passed — report a
//! redacted leaf's proven inclusion honestly instead of as a silent
//! `Verified`. See [`verify`]'s own doc for the full breakdown and
//! [`request`]'s module doc for why steps 2 and 5 are distinct checks
//! rather than one.
//!
//! `verify` never mutates anything: not `request`, not any state reachable
//! through the three ports it is given. It is a pure function to a
//! [`VerificationVerdict`].
//!
//! ## What this crate depends on, and what it does not
//!
//! This crate depends only on `audit-chain-domain` (for
//! [`Sha256Hash`](audit_chain_domain::Sha256Hash), Ed25519 signing/verifying
//! types, and `MerkleTree::verify_proof`) and `audit-verification-api` (for
//! [`VerificationFailureReason`] / [`VerificationVerdict`] themselves). It
//! does **not** depend on `audit_verification_kernel` — the port-only crate
//! that already declares `RootRegistry` / `KeyResolver` / `MerkleVerifier`
//! traits shaped just like the ones in [`ports`] — because that dependency
//! edge is out of scope here. [`ports::RootRegistry`],
//! [`ports::KeyResolver`], and [`ports::MerkleVerifier`] are this crate's
//! own, independently declared equivalents of those three shapes; see
//! [`ports`]'s module doc for exactly how each one mirrors (or,
//! documented case by case, deliberately extends) its
//! `audit_verification_kernel` counterpart.
//!
//! Key resolution and prior-root lookup need a real read path this pure
//! crate does not have, so [`ports::KeyResolver`] and
//! [`ports::RootRegistry`] stay caller-implemented ports with no concrete
//! adapter here — exactly the same shape
//! `audit_sealing_domain::seal_record::PriorPeriodLookup` uses, and for the
//! same reason (see [`ports::RootRegistry::is_first_period`]'s doc).
//! Merkle-proof verification, by contrast, is pure math with no I/O, so
//! this crate DOES supply a real, concrete [`ports::MerkleVerifier`]
//! implementation — [`ChainMerkleVerifier`] — the same way
//! `audit_sealing_domain::merkle_engine::MerkleTreeEngine` supplies a
//! concrete `MerkleEngine` for sealing.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![allow(dead_code)]

mod merkle_adapter;
mod ports;
mod request;
mod verify;

pub use audit_verification_api::{VerificationFailureReason, VerificationVerdict};

pub use merkle_adapter::ChainMerkleVerifier;
pub use ports::{KeyResolver, MerkleVerifier, RootRegistry};
pub use request::{MerkleInclusionProof, PriorRootClaim, VerificationRequest};
pub use verify::{verification_signing_payload, verify};
