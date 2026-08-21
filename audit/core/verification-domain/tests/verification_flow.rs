// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` to
// assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! End-to-end coverage of `audit_verification_domain::verify` against real
//! Ed25519 keys (`audit_chain_domain::Ed25519SigningKey`) and real Merkle
//! trees/proofs (`audit_chain_domain::MerkleTree`) — no faked crypto (L5):
//! every negative test tampers a genuinely-computed value or withholds a
//! genuinely-required port confirmation, rather than asserting a shape a
//! fabricated value would also satisfy.
//!
//! Each of the six closed `VerificationFailureReason` variants gets its own
//! dedicated test driving `verify` to exactly that verdict.

use audit_chain_domain::{Ed25519SigningKey, Ed25519VerificationKey, MerkleTree, Sha256Hash};
use audit_verification_domain::{
    ChainMerkleVerifier, KeyResolver, MerkleInclusionProof, PriorRootClaim, RedactionRegistry,
    RootRegistry, VerificationFailureReason, VerificationRequest, VerificationVerdict,
    verification_signing_payload, verify,
};

// ── real-crypto fixtures ────────────────────────────────────────────────

fn distinct_leaf(seed: u8) -> Sha256Hash {
    let mut bytes = [0_u8; 32];
    bytes[0] = seed;
    bytes[31] = seed.wrapping_add(7);
    bytes
}

/// Builds a real `n`-leaf Merkle tree and returns the leaf, a real
/// inclusion proof, and the real root for `leaf_index`.
fn merkle_fixture(n: u8, leaf_index: usize) -> (Sha256Hash, MerkleInclusionProof, Sha256Hash) {
    let leaves: Vec<Sha256Hash> = (0..n).map(distinct_leaf).collect();
    let tree = MerkleTree::new(leaves.clone());
    let root = tree.build_root();
    let audit_path = tree.proof_path(leaf_index);
    let proof = MerkleInclusionProof {
        audit_path,
        leaf_index: leaf_index as u64,
        leaf_count: tree.len() as u64,
    };
    (leaves[leaf_index], proof, root)
}

fn signer(seed: u8) -> Ed25519SigningKey {
    Ed25519SigningKey::from_seed_bytes("verification-test-key", [seed; 32])
        .expect("test seed builds a signing key")
}

/// A fully self-consistent request: real key, real signature over the
/// record's own identity + root, real Merkle proof, claimed as the first
/// period, not redacted, `context_*` and `record_*` identity matching.
fn valid_request(signing_key: &Ed25519SigningKey) -> VerificationRequest {
    let (leaf, proof, root) = merkle_fixture(5, 2);
    let payload = verification_signing_payload("pack-a", "tenant-1", "2026-08", &root);
    let signature = signing_key.sign(&payload);
    VerificationRequest {
        context_pack: "pack-a".to_string(),
        context_tenant_partition: "tenant-1".to_string(),
        context_period_id: "2026-08".to_string(),
        record_pack: "pack-a".to_string(),
        record_tenant_partition: "tenant-1".to_string(),
        record_period_id: "2026-08".to_string(),
        leaf,
        proof,
        merkle_root: root,
        prior_root: PriorRootClaim::First,
        signature,
        redacted: false,
    }
}

// ── test-double ports ───────────────────────────────────────────────────

/// Always resolves the one fixed key, regardless of what identity it is
/// asked for — models a real (if pack-unaware) key resolver, not a stub
/// that fakes signature math: the actual Ed25519 check still runs for real
/// against whatever key this returns.
struct FixedKeyResolver(Ed25519VerificationKey);
impl KeyResolver for FixedKeyResolver {
    type Error = ();
    fn resolve_key(
        &self,
        _pack: &str,
        _tenant_partition: &str,
        _period_id: &str,
    ) -> Result<Ed25519VerificationKey, ()> {
        Ok(self.0.clone())
    }
}

struct NoKeyCoversThisRequest;
impl KeyResolver for NoKeyCoversThisRequest {
    type Error = ();
    fn resolve_key(&self, _: &str, _: &str, _: &str) -> Result<Ed25519VerificationKey, ()> {
        Err(())
    }
}

/// Confirms every identity as a genuine first period, and has no
/// predecessor root on file for anything (a `Preceding` claim against this
/// double always fails to resolve).
struct ConfirmsFirstPeriod;
impl RootRegistry for ConfirmsFirstPeriod {
    type Error = ();
    fn resolve_root(&self, _: &str, _: &str, _: &str) -> Result<Sha256Hash, ()> {
        Err(())
    }
    fn is_first_period(&self, _: &str, _: &str, _: &str) -> Result<bool, ()> {
        Ok(true)
    }
}

/// Denies every first-period claim (a predecessor is known to exist) and
/// still has no predecessor root on file (simulates: registry knows a
/// predecessor exists but cannot produce its root).
struct DeniesFirstPeriodNoRootOnFile;
impl RootRegistry for DeniesFirstPeriodNoRootOnFile {
    type Error = ();
    fn resolve_root(&self, _: &str, _: &str, _: &str) -> Result<Sha256Hash, ()> {
        Err(())
    }
    fn is_first_period(&self, _: &str, _: &str, _: &str) -> Result<bool, ()> {
        Ok(false)
    }
}

/// Resolves a fixed predecessor root for a `Preceding` claim to chain
/// against.
struct ChainsToFixedPriorRoot(Sha256Hash);
impl RootRegistry for ChainsToFixedPriorRoot {
    type Error = ();
    fn resolve_root(&self, _: &str, _: &str, _: &str) -> Result<Sha256Hash, ()> {
        Ok(self.0)
    }
    fn is_first_period(&self, _: &str, _: &str, _: &str) -> Result<bool, ()> {
        Ok(false)
    }
}

/// Affirmatively confirms the leaf under verification is NOT redacted —
/// models a registry that has genuine retention-cascade state on file and
/// reports it accurately.
struct ConfirmsNotRedacted;
impl RedactionRegistry for ConfirmsNotRedacted {
    type Error = ();
    fn is_redacted(&self, _: &str, _: &str, _: &str, _: &Sha256Hash) -> Result<bool, ()> {
        Ok(false)
    }
}

/// Affirmatively confirms the leaf under verification HAS been redacted —
/// used to prove `verify` reports `RedactedEvent` from the registry's own
/// authority, independent of whatever the caller's `request.redacted`
/// claim happens to say (L8).
struct ConfirmsRedacted;
impl RedactionRegistry for ConfirmsRedacted {
    type Error = ();
    fn is_redacted(&self, _: &str, _: &str, _: &str, _: &Sha256Hash) -> Result<bool, ()> {
        Ok(true)
    }
}

/// Models an unreachable/erroring redaction registry — L4: an inability to
/// confirm "clean" must never be treated as a pass.
struct RedactionLookupFails;
impl RedactionRegistry for RedactionLookupFails {
    type Error = ();
    fn is_redacted(&self, _: &str, _: &str, _: &str, _: &Sha256Hash) -> Result<bool, ()> {
        Err(())
    }
}

fn flip_first_hex_char(hex: &str) -> String {
    if let Some(rest) = hex.strip_prefix('0') {
        format!("1{rest}")
    } else {
        format!("0{}", &hex[1..])
    }
}

// ── happy path ──────────────────────────────────────────────────────────

#[test]
fn verified_when_every_check_genuinely_passes() {
    let signing_key = signer(1);
    let request = valid_request(&signing_key);
    let verdict = verify(
        &request,
        &FixedKeyResolver(signing_key.verification_key()),
        &ConfirmsFirstPeriod,
        &ChainMerkleVerifier,
        &ConfirmsNotRedacted,
    );
    assert_eq!(verdict, VerificationVerdict::Verified);
}

#[test]
fn verified_via_a_genuinely_chained_preceding_root_too() {
    let signing_key = signer(2);
    let mut request = valid_request(&signing_key);
    let prior_root = distinct_leaf(200); // stand-in 32-byte "prior root" value
    request.prior_root = PriorRootClaim::Preceding { root: prior_root };
    let verdict = verify(
        &request,
        &FixedKeyResolver(signing_key.verification_key()),
        &ChainsToFixedPriorRoot(prior_root),
        &ChainMerkleVerifier,
        &ConfirmsNotRedacted,
    );
    assert_eq!(verdict, VerificationVerdict::Verified);
}

// ── the six closed failure reasons, each independently reachable ─────────

#[test]
fn key_epoch_mismatch_when_no_key_covers_the_request() {
    let signing_key = signer(3);
    let request = valid_request(&signing_key);
    let verdict = verify(
        &request,
        &NoKeyCoversThisRequest,
        &ConfirmsFirstPeriod,
        &ChainMerkleVerifier,
        &ConfirmsNotRedacted,
    );
    assert_eq!(
        verdict,
        VerificationVerdict::Failed(VerificationFailureReason::KeyEpochMismatch)
    );
}

#[test]
fn signature_invalid_when_the_signature_bytes_are_tampered() {
    let signing_key = signer(4);
    let mut request = valid_request(&signing_key);
    request.signature.signature_hex = flip_first_hex_char(&request.signature.signature_hex);
    let verdict = verify(
        &request,
        &FixedKeyResolver(signing_key.verification_key()),
        &ConfirmsFirstPeriod,
        &ChainMerkleVerifier,
        &ConfirmsNotRedacted,
    );
    assert_eq!(
        verdict,
        VerificationVerdict::Failed(VerificationFailureReason::SignatureInvalid)
    );
}

#[test]
fn signature_invalid_when_the_resolved_key_is_the_wrong_key() {
    let signing_key = signer(5);
    let request = valid_request(&signing_key);
    let attacker_key = signer(6).verification_key();
    let verdict = verify(
        &request,
        &FixedKeyResolver(attacker_key),
        &ConfirmsFirstPeriod,
        &ChainMerkleVerifier,
        &ConfirmsNotRedacted,
    );
    assert_eq!(
        verdict,
        VerificationVerdict::Failed(VerificationFailureReason::SignatureInvalid)
    );
}

#[test]
fn proof_invalid_when_the_leaf_is_tampered() {
    let signing_key = signer(7);
    let mut request = valid_request(&signing_key);
    request.leaf[0] ^= 0xff;
    let verdict = verify(
        &request,
        &FixedKeyResolver(signing_key.verification_key()),
        &ConfirmsFirstPeriod,
        &ChainMerkleVerifier,
        &ConfirmsNotRedacted,
    );
    assert_eq!(
        verdict,
        VerificationVerdict::Failed(VerificationFailureReason::ProofInvalid)
    );
}

#[test]
fn proof_invalid_when_the_audit_path_is_truncated() {
    let signing_key = signer(8);
    let mut request = valid_request(&signing_key);
    request.proof.audit_path.pop();
    let verdict = verify(
        &request,
        &FixedKeyResolver(signing_key.verification_key()),
        &ConfirmsFirstPeriod,
        &ChainMerkleVerifier,
        &ConfirmsNotRedacted,
    );
    assert_eq!(
        verdict,
        VerificationVerdict::Failed(VerificationFailureReason::ProofInvalid)
    );
}

#[test]
fn prior_root_missing_when_a_preceding_claim_cannot_be_confirmed() {
    let signing_key = signer(9);
    let mut request = valid_request(&signing_key);
    request.prior_root = PriorRootClaim::Preceding {
        root: distinct_leaf(201),
    };
    let verdict = verify(
        &request,
        &FixedKeyResolver(signing_key.verification_key()),
        &DeniesFirstPeriodNoRootOnFile,
        &ChainMerkleVerifier,
        &ConfirmsNotRedacted,
    );
    assert_eq!(
        verdict,
        VerificationVerdict::Failed(VerificationFailureReason::PriorRootMissing)
    );
}

#[test]
fn prior_root_missing_when_a_preceding_claim_does_not_match_the_published_root() {
    let signing_key = signer(10);
    let mut request = valid_request(&signing_key);
    request.prior_root = PriorRootClaim::Preceding {
        root: distinct_leaf(202),
    };
    // Registry has a genuine predecessor root on file — just not the one
    // the record claims.
    let verdict = verify(
        &request,
        &FixedKeyResolver(signing_key.verification_key()),
        &ChainsToFixedPriorRoot(distinct_leaf(203)),
        &ChainMerkleVerifier,
        &ConfirmsNotRedacted,
    );
    assert_eq!(
        verdict,
        VerificationVerdict::Failed(VerificationFailureReason::PriorRootMissing)
    );
}

#[test]
fn prior_root_missing_when_a_first_period_claim_is_not_confirmed() {
    // L8: `PriorRootClaim::First` is a unit variant, free to construct
    // regardless of whether it is true. `verify` must not take it at its
    // word — a registry that denies the claim must fail closed.
    let signing_key = signer(11);
    let request = valid_request(&signing_key); // prior_root: First
    let verdict = verify(
        &request,
        &FixedKeyResolver(signing_key.verification_key()),
        &DeniesFirstPeriodNoRootOnFile,
        &ChainMerkleVerifier,
        &ConfirmsNotRedacted,
    );
    assert_eq!(
        verdict,
        VerificationVerdict::Failed(VerificationFailureReason::PriorRootMissing)
    );
}

#[test]
fn pack_mismatch_even_though_the_signature_and_merkle_proof_both_check_out() {
    // The security property from the assignment, verbatim: a record that is
    // entirely genuine for pack-a (real key, real signature, real Merkle
    // proof, all internally consistent) must still be rejected with
    // `PackMismatch` when the verification CONTEXT names a different pack —
    // even though `FixedKeyResolver` and `ConfirmsFirstPeriod` below are
    // both pack-agnostic test doubles that would happily let the crypto
    // checks pass regardless of which pack they are asked about (a
    // realistic misconfiguration: trust material that is not actually
    // pack-partitioned). `verify` catches the mismatch itself, independent
    // of whether the crypto noticed anything wrong.
    let signing_key = signer(12);
    let mut request = valid_request(&signing_key); // record_* == "pack-a"/"tenant-1"/"2026-08"
    request.context_pack = "pack-b".to_string();
    let verdict = verify(
        &request,
        &FixedKeyResolver(signing_key.verification_key()),
        &ConfirmsFirstPeriod,
        &ChainMerkleVerifier,
        &ConfirmsNotRedacted,
    );
    assert_eq!(
        verdict,
        VerificationVerdict::Failed(VerificationFailureReason::PackMismatch)
    );
}

#[test]
fn pack_mismatch_on_a_period_id_leg_mismatch_alone() {
    // L7: every leg of the (pack, tenant_partition, period_id) tuple
    // participates, not just `pack`. Pack and tenant_partition match here;
    // only period_id differs.
    let signing_key = signer(13);
    let mut request = valid_request(&signing_key);
    request.context_period_id = "2026-09".to_string();
    let verdict = verify(
        &request,
        &FixedKeyResolver(signing_key.verification_key()),
        &ConfirmsFirstPeriod,
        &ChainMerkleVerifier,
        &ConfirmsNotRedacted,
    );
    assert_eq!(
        verdict,
        VerificationVerdict::Failed(VerificationFailureReason::PackMismatch)
    );
}

#[test]
fn pack_mismatch_on_a_tenant_partition_leg_mismatch_alone() {
    let signing_key = signer(14);
    let mut request = valid_request(&signing_key);
    request.context_tenant_partition = "tenant-2".to_string();
    let verdict = verify(
        &request,
        &FixedKeyResolver(signing_key.verification_key()),
        &ConfirmsFirstPeriod,
        &ChainMerkleVerifier,
        &ConfirmsNotRedacted,
    );
    assert_eq!(
        verdict,
        VerificationVerdict::Failed(VerificationFailureReason::PackMismatch)
    );
}

#[test]
fn redacted_event_reported_after_genuinely_proving_inclusion() {
    let signing_key = signer(15);
    let mut request = valid_request(&signing_key);
    request.redacted = true;
    let verdict = verify(
        &request,
        &FixedKeyResolver(signing_key.verification_key()),
        &ConfirmsFirstPeriod,
        &ChainMerkleVerifier,
        &ConfirmsNotRedacted,
    );
    assert_eq!(
        verdict,
        VerificationVerdict::Failed(VerificationFailureReason::RedactedEvent)
    );
}

#[test]
fn redacted_event_still_requires_a_genuinely_valid_inclusion_proof() {
    // The redacted flag must never mask a real proof failure: a redacted
    // request whose leaf is tampered must still report `ProofInvalid`, not
    // `RedactedEvent` — inclusion genuinely was not proven.
    let signing_key = signer(16);
    let mut request = valid_request(&signing_key);
    request.redacted = true;
    request.leaf[0] ^= 0xff;
    let verdict = verify(
        &request,
        &FixedKeyResolver(signing_key.verification_key()),
        &ConfirmsFirstPeriod,
        &ChainMerkleVerifier,
        &ConfirmsNotRedacted,
    );
    assert_eq!(
        verdict,
        VerificationVerdict::Failed(VerificationFailureReason::ProofInvalid)
    );
}

#[test]
fn redacted_event_when_registry_confirms_redaction_even_though_caller_claims_clean() {
    // L8, the exact defect this port exists to close: `request.redacted` is
    // a plain `bool`, exactly as free to construct as
    // `PriorRootClaim::First` is. A caller (or an attacker replaying a
    // genuinely-signed record) setting `redacted: false` must NOT be able
    // to launder a genuinely redacted leaf through as `Verified` — the
    // registry's own confirmation governs, not the caller's say-so.
    let signing_key = signer(17);
    let mut request = valid_request(&signing_key);
    request.redacted = false; // the caller's (false) claim of "not redacted"
    let verdict = verify(
        &request,
        &FixedKeyResolver(signing_key.verification_key()),
        &ConfirmsFirstPeriod,
        &ChainMerkleVerifier,
        &ConfirmsRedacted, // the registry's (true) ground truth
    );
    assert_eq!(
        verdict,
        VerificationVerdict::Failed(VerificationFailureReason::RedactedEvent),
        "an unauthenticated `redacted: false` claim must never override a \
         registry that affirmatively confirms the leaf was redacted"
    );
}

#[test]
fn redacted_event_when_the_registry_cannot_be_reached() {
    // L4: an unconfirmable "clean" answer must never be treated as a pass.
    let signing_key = signer(18);
    let mut request = valid_request(&signing_key);
    request.redacted = false;
    let verdict = verify(
        &request,
        &FixedKeyResolver(signing_key.verification_key()),
        &ConfirmsFirstPeriod,
        &ChainMerkleVerifier,
        &RedactionLookupFails,
    );
    assert_eq!(
        verdict,
        VerificationVerdict::Failed(VerificationFailureReason::RedactedEvent),
        "an erroring redaction registry must fail closed, never pass through as Verified"
    );
}

// ── signing payload is bound to the actual root, not a fixed shape (L5) ──

#[test]
fn signing_payload_changes_when_the_merkle_root_changes() {
    let (_, _, root_a) = merkle_fixture(4, 0);
    let (_, _, root_b) = merkle_fixture(5, 0);
    assert_ne!(
        root_a, root_b,
        "test fixture sanity: distinct trees, distinct roots"
    );
    let payload_a = verification_signing_payload("pack-a", "tenant-1", "2026-08", &root_a);
    let payload_b = verification_signing_payload("pack-a", "tenant-1", "2026-08", &root_b);
    assert_ne!(payload_a, payload_b);
}

#[test]
fn signing_payload_changes_when_any_identity_leg_changes() {
    let (_, _, root) = merkle_fixture(4, 0);
    let base = verification_signing_payload("pack-a", "tenant-1", "2026-08", &root);
    assert_ne!(
        base,
        verification_signing_payload("pack-b", "tenant-1", "2026-08", &root)
    );
    assert_ne!(
        base,
        verification_signing_payload("pack-a", "tenant-2", "2026-08", &root)
    );
    assert_ne!(
        base,
        verification_signing_payload("pack-a", "tenant-1", "2026-09", &root)
    );
}

// ── payload injectivity: no field-boundary confusion (L3/L9) ─────────────
//
// Regression coverage for a real, proven defect: an earlier version of
// `verification_signing_payload` joined `"field=value"` strings with `\n`
// separators. Because none of `pack` / `tenant_partition` / `period_id` are
// restricted in what characters they may contain, a `\n` embedded in one
// field let bytes migrate across a field boundary, so two DIFFERENT
// identity triples serialized to byte-identical payloads — and a signature
// genuinely minted for one verified as valid for the other. Fixed by
// length-prefixing every field instead of delimiting it; these tests prove
// the fix holds using the exact triples that used to collide.

#[test]
fn no_field_boundary_confusion_across_the_tenant_period_seam() {
    let (_, _, root) = merkle_fixture(3, 0);
    let a = verification_signing_payload("pack-a", "tenant-1\nperiod_id=2026-08", "9999-12", &root);
    let b = verification_signing_payload("pack-a", "tenant-1", "2026-08\nperiod_id=9999-12", &root);
    assert_ne!(
        a, b,
        "two distinct (pack, tenant_partition, period_id) triples must never \
         serialize to the same bytes"
    );
}

#[test]
fn no_field_boundary_confusion_across_the_pack_tenant_seam() {
    let (_, _, root) = merkle_fixture(3, 0);
    let a = verification_signing_payload("pack-a\ntenant_partition=t2", "t1", "2026-08", &root);
    let b = verification_signing_payload("pack-a", "t2", "2026-08", &root);
    // Not a claim that these two SHOULD collide — a sanity check that the
    // encoding does not accidentally make the first triple look like the
    // second by dropping the embedded newline; the real assertion is the
    // dedicated forgery test below.
    assert_ne!(a, b);
}

#[test]
fn a_signature_minted_for_one_identity_triple_does_not_verify_as_another() {
    // The end-to-end security property: sign over one (pack,
    // tenant_partition, period_id) triple with real Ed25519, then present
    // that exact signature under a DIFFERENT triple engineered to have
    // collided under the old `\n`-joined encoding. Must be rejected as
    // `SignatureInvalid`, never accepted.
    let signing_key = signer(19);
    let (leaf, proof, root) = merkle_fixture(3, 0);

    let signed_pack = "pack-a";
    let signed_tenant = "tenant-1\nperiod_id=2026-08";
    let signed_period = "9999-12";
    let payload = verification_signing_payload(signed_pack, signed_tenant, signed_period, &root);
    let signature = signing_key.sign(&payload);

    // The triple that used to collide with the one above under the old
    // `\n`-joined format.
    let presented_tenant = "tenant-1";
    let presented_period = "2026-08\nperiod_id=9999-12";

    let request = VerificationRequest {
        context_pack: signed_pack.to_string(),
        context_tenant_partition: presented_tenant.to_string(),
        context_period_id: presented_period.to_string(),
        record_pack: signed_pack.to_string(),
        record_tenant_partition: presented_tenant.to_string(),
        record_period_id: presented_period.to_string(),
        leaf,
        proof,
        merkle_root: root,
        prior_root: PriorRootClaim::First,
        signature,
        redacted: false,
    };

    let verdict = verify(
        &request,
        &FixedKeyResolver(signing_key.verification_key()),
        &ConfirmsFirstPeriod,
        &ChainMerkleVerifier,
        &ConfirmsNotRedacted,
    );
    assert_eq!(
        verdict,
        VerificationVerdict::Failed(VerificationFailureReason::SignatureInvalid),
        "a signature minted for one identity triple must never verify as a \
         different triple, even one that collided under the old encoding"
    );
}
