//! The verifier: resolve key, verify signature, verify proof, walk prior
//! root, reject cross-pack mixtures, report redaction honestly.
//!
//! [`verify`] is pure and never mutates any of its arguments or any state
//! reachable through the ports it is given — it is a function from
//! `(request, key_resolver, root_registry, merkle_verifier)` to a
//! [`VerificationVerdict`], nothing more.

use audit_chain_domain::Sha256Hash;
use audit_verification_api::{VerificationFailureReason, VerificationVerdict};

use crate::ports::{KeyResolver, MerkleVerifier, RootRegistry};
use crate::request::{PriorRootClaim, VerificationRequest};

/// The exact canonical bytes a `record_pack` / `record_tenant_partition` /
/// `record_period_id` / `merkle_root` combination must be signed over for
/// [`verify`] to accept the signature. A real signer (whatever composes
/// `audit_sealing_domain::build_seal_record`'s output with a `SignerPort`
/// adapter) MUST sign exactly this construction — a different framing,
/// field order, or separator produces a signature `verify` will reject as
/// [`VerificationFailureReason::SignatureInvalid`], never as a pass.
///
/// Deliberately built from the record's OWN identity fields, not the
/// verification request's `context_*` identity: the signature attests to
/// what the record itself is (its `(pack, tenant_partition, period_id,
/// merkle_root)`), independent of which context a caller later asks to
/// verify it under. That separation is what lets [`verify`] tell a
/// [`VerificationFailureReason::SignatureInvalid`] apart from a
/// [`VerificationFailureReason::PackMismatch`] — see [`crate::request`]'s
/// module doc.
pub fn verification_signing_payload(
    record_pack: &str,
    record_tenant_partition: &str,
    record_period_id: &str,
    merkle_root: &Sha256Hash,
) -> Vec<u8> {
    [
        "audit-verification-domain-v1".to_string(),
        format!("pack={record_pack}"),
        format!("tenant_partition={record_tenant_partition}"),
        format!("period_id={record_period_id}"),
        format!("merkle_root=sha256:{}", encode_hex(merkle_root)),
    ]
    .join("\n")
    .into_bytes()
}

fn encode_hex(bytes: &Sha256Hash) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

/// Verify `request` against the trust material `key_resolver`,
/// `root_registry`, and `merkle_verifier` supply, and return the resulting
/// [`VerificationVerdict`]. Never panics, never mutates `request` or
/// anything reachable through the three ports, and never returns
/// `VerificationVerdict::Verified` unless every one of the following holds:
///
/// 1. **Resolve key.** `key_resolver.resolve_key` succeeds for
///    `request.context_*`. `Err` (or a key epoch that simply does not
///    cover this request) →
///    [`VerificationFailureReason::KeyEpochMismatch`].
/// 2. **Verify signature.** `request.signature` verifies, via
///    `audit_chain_domain::Ed25519Signature::verify_with_trusted_key`, over
///    exactly [`verification_signing_payload`] built from `request.record_*`
///    and `request.merkle_root`, under the resolved key. A failure here →
///    [`VerificationFailureReason::SignatureInvalid`].
/// 3. **Verify proof.** `merkle_verifier.verify(&request.leaf,
///    &request.proof, &request.merkle_root)` returns `true`. `false` →
///    [`VerificationFailureReason::ProofInvalid`].
/// 4. **Walk prior root.** [`PriorRootClaim::First`] must be confirmed by
///    `root_registry.is_first_period`; [`PriorRootClaim::Preceding`] must
///    match what `root_registry.resolve_root` returns. Either an
///    unconfirmed `First` claim or an unresolved/mismatched `Preceding`
///    claim → [`VerificationFailureReason::PriorRootMissing`] — never a
///    pass on `Err` alone (L4).
/// 5. **Reject cross-pack mixtures.** `request.record_pack` /
///    `record_tenant_partition` / `record_period_id` must equal
///    `request.context_pack` / `context_tenant_partition` /
///    `context_period_id`, leg for leg (L7). Any one leg differing →
///    [`VerificationFailureReason::PackMismatch`], even though steps 1
///    through 4 above all already passed using genuinely valid crypto (see
///    [`crate::request`]'s module doc for why the crypto alone cannot
///    catch this).
/// 6. **Report redaction honestly.** If every check above passed but
///    `request.redacted` is `true`, the leaf's inclusion has been proven —
///    it is not silently reported as verified anyway. →
///    [`VerificationFailureReason::RedactedEvent`].
///
/// Only when none of the above fires does `verify` return
/// [`VerificationVerdict::Verified`].
pub fn verify<KR, RR, MV>(
    request: &VerificationRequest,
    key_resolver: &KR,
    root_registry: &RR,
    merkle_verifier: &MV,
) -> VerificationVerdict
where
    KR: KeyResolver,
    RR: RootRegistry,
    MV: MerkleVerifier,
{
    // 1. Resolve key, against the verification CONTEXT — never the
    // record's own unauthenticated identity claim (see module doc).
    let Ok(key) = key_resolver.resolve_key(
        &request.context_pack,
        &request.context_tenant_partition,
        &request.context_period_id,
    ) else {
        return VerificationVerdict::Failed(VerificationFailureReason::KeyEpochMismatch);
    };

    // 2. Verify signature, over a payload built from the RECORD's own
    // identity (see `verification_signing_payload`'s doc for why).
    let payload = verification_signing_payload(
        &request.record_pack,
        &request.record_tenant_partition,
        &request.record_period_id,
        &request.merkle_root,
    );
    if request
        .signature
        .verify_with_trusted_key(&payload, &key)
        .is_err()
    {
        return VerificationVerdict::Failed(VerificationFailureReason::SignatureInvalid);
    }

    // 3. Verify proof.
    if !merkle_verifier.verify(&request.leaf, &request.proof, &request.merkle_root) {
        return VerificationVerdict::Failed(VerificationFailureReason::ProofInvalid);
    }

    // 4. Walk prior root.
    let prior_root_confirmed = match &request.prior_root {
        PriorRootClaim::First => matches!(
            root_registry.is_first_period(
                &request.context_pack,
                &request.context_tenant_partition,
                &request.context_period_id,
            ),
            Ok(true)
        ),
        PriorRootClaim::Preceding { root } => matches!(
            root_registry.resolve_root(
                &request.context_pack,
                &request.context_tenant_partition,
                &request.context_period_id,
            ),
            Ok(resolved) if resolved == *root
        ),
    };
    if !prior_root_confirmed {
        return VerificationVerdict::Failed(VerificationFailureReason::PriorRootMissing);
    }

    // 5. Reject cross-pack (and cross-tenant-partition, cross-period)
    // mixtures: every leg of the identity tuple, not just `pack` (L7).
    if request.record_pack != request.context_pack
        || request.record_tenant_partition != request.context_tenant_partition
        || request.record_period_id != request.context_period_id
    {
        return VerificationVerdict::Failed(VerificationFailureReason::PackMismatch);
    }

    // 6. Report redaction honestly — only after inclusion has genuinely
    // been proven above.
    if request.redacted {
        return VerificationVerdict::Failed(VerificationFailureReason::RedactedEvent);
    }

    VerificationVerdict::Verified
}
