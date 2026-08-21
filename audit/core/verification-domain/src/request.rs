//! Caller-supplied inputs for [`crate::verify`].
//!
//! Mirrors the shape `audit_sealing_domain::seal_record::SealRecordInput`
//! uses: plain public fields, no constructor, and no invariant claimed by
//! this module (L1) — every check [`crate::verify`] performs runs inside
//! `verify` itself and is reported through [`VerificationVerdict`], never
//! silently skipped by a constructor that never got called.
//!
//! [`VerificationVerdict`]: audit_verification_api::VerificationVerdict

use audit_chain_domain::{Ed25519Signature, Sha256Hash};

/// A bundle of the three RFC 6962 §2.1 values
/// `audit_chain_domain::MerkleTree::verify_proof` needs beyond the leaf and
/// root themselves. Opaque to [`crate::ports::MerkleVerifier`] — plain
/// public fields, no invariant claimed here either: `verify_proof` itself
/// already fails closed on every malformed combination (see its own doc).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MerkleInclusionProof {
    /// The audit path from the leaf up to the root, leaf-to-root order —
    /// see `audit_chain_domain::MerkleTree::proof_path`.
    pub audit_path: Vec<Sha256Hash>, // data_class: INTERNAL_ONLY
    /// The leaf's position among `leaf_count` leaves.
    pub leaf_index: u64, // data_class: PUBLIC
    /// The total number of leaves the committed tree covered.
    pub leaf_count: u64, // data_class: PUBLIC
}

/// What the record under verification claims about the period immediately
/// before it, for the same `(pack, tenant_partition)`.
///
/// Mirrors `audit_sealing_domain::seal_record::PriorPeriod` exactly
/// (`First` / `Preceding { root }`) — see that type's doc for why a bare
/// `First` claim is never trusted on its own: [`crate::verify`] always
/// checks it against [`crate::ports::RootRegistry::is_first_period`] (L8)
/// rather than accepting the variant as proof of anything.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PriorRootClaim {
    /// This is claimed to be the first period ever sealed for
    /// `(pack, tenant_partition)`. Verified via
    /// [`crate::ports::RootRegistry::is_first_period`], never taken as-is.
    First,
    /// The immediately preceding sealed period's published root, for
    /// chaining. Verified against
    /// [`crate::ports::RootRegistry::resolve_root`], never taken as-is.
    Preceding { root: Sha256Hash },
}

/// Caller-supplied inputs to [`crate::verify`].
///
/// Two identity triples travel together here, and [`crate::verify`]
/// deliberately keeps them distinct instead of collapsing them into one:
///
/// - `context_pack` / `context_tenant_partition` / `context_period_id` —
///   the `(pack, tenant_partition, period_id)` this verification call is
///   actually being invoked for. This is the identity every port
///   ([`crate::ports::KeyResolver`], [`crate::ports::RootRegistry`]) is
///   resolved against — the trust anchor.
/// - `record_pack` / `record_tenant_partition` / `record_period_id` — the
///   sealed artifact's OWN self-asserted identity. Nothing in the Ed25519
///   signature bytes or the Merkle proof authenticates `context_*` against
///   `record_*` on its own (the signing payload
///   [`crate::verification_signing_payload`] builds is keyed off
///   `record_*`, precisely so a genuinely valid signature and a genuinely
///   valid Merkle proof for pack A tell you nothing about whether they were
///   submitted under a pack A or pack B verification request) — so
///   [`crate::verify`] checks the two triples against each other
///   explicitly and reports any leg's mismatch as
///   `VerificationFailureReason::PackMismatch` (L7: every leg of the tuple
///   participates, not just `pack`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerificationRequest {
    pub context_pack: String,             // data_class: PUBLIC
    pub context_tenant_partition: String, // data_class: INTERNAL_ONLY
    pub context_period_id: String,        // data_class: INTERNAL_ONLY

    pub record_pack: String,             // data_class: PUBLIC
    pub record_tenant_partition: String, // data_class: INTERNAL_ONLY
    pub record_period_id: String,        // data_class: INTERNAL_ONLY

    /// The individual leaf under verification (e.g. one audit event's
    /// commitment hash within the sealed period's tree).
    pub leaf: Sha256Hash, // data_class: INTERNAL_ONLY
    pub proof: MerkleInclusionProof, // data_class: INTERNAL_ONLY
    /// The sealed period's own Merkle root, as attested by the signature
    /// (mirrors `audit_sealing_kernel::SealRecord::merkle_root`).
    pub merkle_root: Sha256Hash, // data_class: INTERNAL_ONLY
    pub prior_root: PriorRootClaim,  // data_class: INTERNAL_ONLY
    pub signature: Ed25519Signature, // data_class: INTERNAL_ONLY

    /// The caller's claim that the leaf under verification belongs to an
    /// event a retention-cascade redaction has already erased the payload
    /// of. `audit/policy/retention-matrix.yaml` sets `preserve_merkle_proof:
    /// true` for every class it defines, so a redacted event's leaf hash is
    /// never removed from the tree — [`crate::verify`] still runs the full
    /// signature and Merkle-inclusion check against it (it must still PROVE
    /// inclusion).
    ///
    /// This field alone is NOT the enforcement: a plain `bool` is exactly
    /// as free to construct as [`PriorRootClaim::First`] is (L8), so
    /// [`crate::verify`] never takes it at face value either. It confirms
    /// the true redaction status via
    /// [`crate::ports::RedactionRegistry::is_redacted`] — see that port's
    /// doc for why — and only turns a would-be `Verified` into
    /// `VerificationFailureReason::RedactedEvent` once genuine inclusion
    /// has been proven, so a redacted leaf is reported honestly rather than
    /// silently verified, and setting this field to `false` cannot by
    /// itself launder a genuinely redacted leaf through as `Verified`.
    pub redacted: bool, // data_class: PUBLIC
}
