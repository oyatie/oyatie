//! Port traits this crate calls to verify a sealed period.
//!
//! `audit-verification-domain` depends only on `audit-chain-domain` and
//! `audit-verification-api` (see the crate doc for why) — it may NOT depend
//! on `audit_verification_kernel`, the port-only crate that already declares
//! `RootRegistry`, `KeyResolver`, and `MerkleVerifier`. Three of the four
//! traits below are this crate's OWN declarations of those equivalent
//! shapes, written independently so this crate never needs the dependency
//! edge; the fourth, [`RedactionRegistry`], has no `audit_verification_kernel`
//! counterpart at all and is this crate's own addition (see its doc for
//! why). Anywhere a mirrored shape differs from `audit_verification_kernel`'s,
//! the doc comment on that item says so and why.

use audit_chain_domain::{Ed25519VerificationKey, Sha256Hash};

/// Mirrors `audit_verification_kernel::KeyResolver` exactly: resolves the
/// Ed25519 verification key that is trusted to have signed `period_id` for
/// `(pack, tenant_partition)`.
///
/// Fixed to `Key = Ed25519VerificationKey` (rather than a free associated
/// type) so [`crate::verify`] can call
/// `audit_chain_domain::Ed25519Signature::verify_with_trusted_key` directly
/// on the result — real Ed25519 verification, not a second port this crate
/// would otherwise have to invent to wrap it.
///
/// An `Err` here means "no key epoch this crate can find covers this
/// request" and [`crate::verify`] reports
/// [`VerificationFailureReason::KeyEpochMismatch`](audit_verification_api::VerificationFailureReason::KeyEpochMismatch)
/// for it, never a pass (L4: fail closed).
pub trait KeyResolver {
    type Error;

    fn resolve_key(
        &self,
        pack: &str,
        tenant_partition: &str,
        period_id: &str,
    ) -> Result<Ed25519VerificationKey, Self::Error>;
}

/// Mirrors `audit_verification_kernel::MerkleVerifier`'s shape
/// (`verify(leaf, proof, root) -> bool`) exactly, fixed to this crate's real
/// value types: `Leaf`/`Root` are raw RFC 6962 leaf/root hashes
/// ([`Sha256Hash`]) and `Proof` is [`crate::MerkleInclusionProof`] — an
/// opaque (to this trait) bundle of whatever an implementation needs to
/// recompute a root from a leaf. [`crate::ChainMerkleVerifier`] is this
/// crate's own real implementation, delegating to
/// `audit_chain_domain::MerkleTree::verify_proof`.
pub trait MerkleVerifier {
    fn verify(
        &self,
        leaf: &Sha256Hash,
        proof: &crate::request::MerkleInclusionProof,
        root: &Sha256Hash,
    ) -> bool;
}

/// Mirrors `audit_verification_kernel::RootRegistry`'s `resolve_root` method
/// exactly, PLUS one addition beyond the mirrored shape:
/// [`RootRegistry::is_first_period`].
///
/// ## Why the addition (L8)
///
/// [`crate::PriorRootClaim::First`] asserting "no predecessor exists" is,
/// by itself, exactly as unenforced as `Option::None` was before
/// `audit-sealing-domain` closed the same gap for
/// `audit_sealing_domain::seal_record::PriorPeriod::First` — a unit variant
/// is free to construct regardless of whether a predecessor genuinely
/// exists. `audit_sealing_domain::seal_record::PriorPeriodLookup` closes
/// that gap with an explicit port method returning `Result<bool, Error>`
/// (an affirmative/negative answer, not a value lookup) that the domain
/// calls and checks before honoring the claim. `is_first_period` here is
/// the same pattern, because `verification-kernel`'s three named ports
/// don't cover it: `resolve_root` alone cannot distinguish "confirmed no
/// predecessor" from "predecessor lookup failed for some other reason" —
/// both are just `Err`, and treating either as a pass would let a forged
/// `First` claim through on infrastructure flakiness alone (L4).
pub trait RootRegistry {
    type Error;

    /// Resolves the trusted, previously published root that
    /// `(pack, tenant_partition, period_id)`'s immediate predecessor
    /// committed. `Err` when no such root can be confirmed (unreachable
    /// registry, no record on file, etc.) — [`crate::verify`] never treats
    /// that as a pass.
    fn resolve_root(
        &self,
        pack: &str,
        tenant_partition: &str,
        period_id: &str,
    ) -> Result<Sha256Hash, Self::Error>;

    /// Returns `Ok(true)` only when the registry can affirmatively confirm
    /// that no period precedes `period_id` for `(pack, tenant_partition)` —
    /// i.e. a [`crate::PriorRootClaim::First`] claim against it is
    /// genuinely true. `Ok(false)` and `Err` are both treated identically
    /// by [`crate::verify`]: neither is a confirmed "yes", so neither is a
    /// pass.
    fn is_first_period(
        &self,
        pack: &str,
        tenant_partition: &str,
        period_id: &str,
    ) -> Result<bool, Self::Error>;
}

/// Confirms whether the leaf under verification has genuinely been redacted
/// by a retention-cascade action, per `audit/policy/retention-matrix.yaml`'s
/// `preserve_merkle_proof: true` policy (every class it defines sets this:
/// a redacted event's leaf hash stays in the tree even though its payload
/// is gone elsewhere).
///
/// ## Why this port exists at all (L8)
///
/// [`crate::VerificationRequest::redacted`] is a plain `bool` field — every
/// bit as free to construct as [`crate::PriorRootClaim::First`] is free to
/// construct, and for the exact same reason [`RootRegistry::is_first_period`]
/// exists: a caller (or an attacker replaying a genuinely-signed record)
/// could otherwise simply set `redacted: false` and have a truly redacted
/// leaf's `RedactedEvent` verdict silently laundered into `Verified`,
/// because nothing about the Ed25519 signature or the Merkle proof says
/// anything about redaction status — [`crate::verification_signing_payload`]
/// does not cover it, deliberately, because redaction is a fact about
/// retention-cascade state *after* sealing, not something the record itself
/// could have attested to at signing time. [`crate::verify`] therefore never
/// takes `request.redacted` as the whole story: it also asks this registry,
/// and only a confirmed `Ok(false)` answer lets a would-be `Verified`
/// verdict actually pass step 6. `Ok(true)` and `Err` are both treated as
/// "not confirmed clean" and fail closed into
/// [`crate::VerificationFailureReason::RedactedEvent`] (L4) — an
/// unreachable or erroring registry can never be used to launder a
/// genuinely redacted leaf through as verified, and `request.redacted` on
/// its own can never override a registry that says otherwise.
pub trait RedactionRegistry {
    type Error;

    /// Returns `Ok(true)` only when the registry can affirmatively confirm
    /// that the leaf at `(pack, tenant_partition, period_id)` has been
    /// redacted by a retention-cascade action. `Ok(false)` means
    /// affirmatively confirmed clean. `Err` means the registry could not
    /// answer at all — [`crate::verify`] treats that identically to
    /// `Ok(true)`: neither is a confirmed "clean", so neither lets
    /// [`crate::verify`] report [`crate::VerificationVerdict::Verified`].
    fn is_redacted(
        &self,
        pack: &str,
        tenant_partition: &str,
        period_id: &str,
        leaf: &Sha256Hash,
    ) -> Result<bool, Self::Error>;
}
