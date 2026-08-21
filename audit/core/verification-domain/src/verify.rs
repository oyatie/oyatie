//! The verifier: resolve key, verify signature, verify proof, walk prior
//! root, reject cross-pack mixtures, report redaction honestly.
//!
//! [`verify`] is pure and never mutates any of its arguments or any state
//! reachable through the ports it is given — it is a function from
//! `(request, key_resolver, root_registry, merkle_verifier)` to a
//! [`VerificationVerdict`], nothing more.

use audit_chain_domain::Sha256Hash;
use audit_verification_api::{VerificationFailureReason, VerificationVerdict};

use crate::ports::{KeyResolver, MerkleVerifier, RedactionRegistry, RootRegistry};
use crate::request::{PriorRootClaim, VerificationRequest};

/// The exact canonical bytes a `record_pack` / `record_tenant_partition` /
/// `record_period_id` / `merkle_root` combination must be signed over for
/// [`verify`] to accept the signature. A real signer (whatever composes
/// `audit_sealing_domain::build_seal_record`'s output with a `SignerPort`
/// adapter) MUST sign exactly this construction — a different framing,
/// field order, or byte value produces a signature `verify` will reject as
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
///
/// ## Injective by construction (L9)
///
/// This crate is the sole producer of this byte format — nothing else in
/// `audit/` mints it (see the crate doc's "what this crate depends on"
/// section) — so it is this crate's job, not some downstream reader's, to
/// make the encoding unambiguous. An earlier version of this function
/// joined `"field=value"` strings with `\n` separators; because `pack` /
/// `tenant_partition` / `period_id` are arbitrary caller strings with no
/// charset restriction, a `\n` embedded in one field let content migrate
/// across a field boundary, so two DIFFERENT `(pack, tenant_partition,
/// period_id)` triples could serialize to byte-identical payloads — e.g.
/// `("pack-a", "t", "X\nperiod_id=Y")` and `("pack-a", "t\nperiod_id=X",
/// "Y")` both produced the same bytes, so a signature genuinely minted for
/// one triple verified as valid for the other. Fixed here by length-
/// prefixing every variable-length field with its exact byte count (a
/// fixed-width 8-byte big-endian `u64`) instead of scanning for a
/// delimiter: a reader consumes exactly `len` bytes for each field, so no
/// byte sequence inside a field — `\n`, `=`, or anything else — can ever be
/// misread as a boundary. `merkle_root` needs no length prefix of its own
/// because [`Sha256Hash`] is already a fixed-size `[u8; 32]`, so its extent
/// is never ambiguous either. The version tag changed from `v1` to `v2`
/// alongside the wire-format change so the two encodings can never be
/// confused with each other.
pub fn verification_signing_payload(
    record_pack: &str,
    record_tenant_partition: &str,
    record_period_id: &str,
    merkle_root: &Sha256Hash,
) -> Vec<u8> {
    const DOMAIN_TAG: &[u8] = b"audit-verification-domain-v2";
    let mut out = Vec::with_capacity(
        DOMAIN_TAG.len()
            + 8 * 3
            + record_pack.len()
            + record_tenant_partition.len()
            + record_period_id.len()
            + merkle_root.len(),
    );
    out.extend_from_slice(DOMAIN_TAG);
    push_length_prefixed(&mut out, record_pack.as_bytes());
    push_length_prefixed(&mut out, record_tenant_partition.as_bytes());
    push_length_prefixed(&mut out, record_period_id.as_bytes());
    out.extend_from_slice(merkle_root); // fixed-size: no length prefix needed
    out
}

/// Appends `field` to `out`, preceded by its exact byte length as a
/// fixed-width 8-byte big-endian integer. This is what makes
/// [`verification_signing_payload`] injective (L9/L3): a reader never
/// scans `field` for a delimiter, so nothing inside `field` — including a
/// byte sequence that looks like another field's own length prefix — can
/// ever be misinterpreted as a boundary.
fn push_length_prefixed(out: &mut Vec<u8>, field: &[u8]) {
    out.extend_from_slice(&(field.len() as u64).to_be_bytes());
    out.extend_from_slice(field);
}

/// Verify `request` against the trust material `key_resolver`,
/// `root_registry`, `merkle_verifier`, and `redaction_registry` supply, and
/// return the resulting [`VerificationVerdict`]. Never panics, never
/// mutates `request` or anything reachable through the four ports, and
/// never returns `VerificationVerdict::Verified` unless every one of the
/// following holds:
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
/// 6. **Report redaction honestly.** If every check above passed, the
///    leaf's inclusion has genuinely been proven — it is not silently
///    reported as verified when it has actually been redacted.
///    `request.redacted` alone is never trusted for this (L8: a `bool`
///    field is exactly as free to construct as a unit enum variant): it is
///    confirmed against `redaction_registry.is_redacted`, and only a
///    confirmed `Ok(false)` (and `request.redacted == false`) lets this
///    step pass. `request.redacted == true`, `Ok(true)`, or `Err` from the
///    registry all fail the same way (L4: fail closed, never take an
///    unconfirmed or unreachable "clean" answer as a pass) →
///    [`VerificationFailureReason::RedactedEvent`].
///
/// Only when none of the above fires does `verify` return
/// [`VerificationVerdict::Verified`].
pub fn verify<KR, RR, MV, RG>(
    request: &VerificationRequest,
    key_resolver: &KR,
    root_registry: &RR,
    merkle_verifier: &MV,
    redaction_registry: &RG,
) -> VerificationVerdict
where
    KR: KeyResolver,
    RR: RootRegistry,
    MV: MerkleVerifier,
    RG: RedactionRegistry,
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
    // been proven above. `request.redacted` is a free-to-construct caller
    // claim (L8) and is never trusted alone: it must be confirmed by
    // `redaction_registry`. Anything other than a confirmed "clean" fails
    // closed into `RedactedEvent` rather than a silent `Verified` (L4), so
    // an unreachable or erroring registry can never be used to launder a
    // genuinely redacted leaf through as verified.
    let confirmed_not_redacted = matches!(
        redaction_registry.is_redacted(
            &request.context_pack,
            &request.context_tenant_partition,
            &request.context_period_id,
            &request.leaf,
        ),
        Ok(false)
    );
    if request.redacted || !confirmed_not_redacted {
        return VerificationVerdict::Failed(VerificationFailureReason::RedactedEvent);
    }

    VerificationVerdict::Verified
}
