//! Pure DSR domain logic: Merkle aggregation of erasure receipts, proof
//! verification, and the per-pack legal SLA. No I/O, no clock, no
//! randomness — every function here is a total function of its arguments.
//!
//! # Merkle encoding (normative)
//!
//! The tree is RFC 6962-style, with explicit domain separation so a leaf
//! hash can never be mistaken for an internal node hash.
//!
//! ```text
//! field(x)  := be_u64(len(x)) || x                    (length-prefixed)
//! leaf(r)   := SHA256(0x00 || field(r.tenant)
//!                          || field(r.request)
//!                          || field(r.microservice)
//!                          || field(r.merkle_leaf))
//! node(l,r) := SHA256(0x01 || l || r)
//! ```
//!
//! `canonical_receipt_bytes` returns the WHOLE preimage including the `0x00`
//! tag, so [`leaf_hash`] is `SHA256(canonical_receipt_bytes(r))` — the tag is
//! applied exactly once. The crate's test suite pins one leaf to a literal hex
//! digest, so a third party can reimplement this verifier from the four
//! lines above and check their implementation against a stated value.
//!
//! Four deliberate choices:
//!
//! 1. **Length prefixes.** Without them `("ab","c")` and `("a","bc")`
//!    encode identically, so two different receipt sets could share a leaf.
//! 2. **Tag bytes.** `0x00` for leaves and `0x01` for nodes is the RFC 6962
//!    second-preimage defence: an attacker cannot present an internal node
//!    as if it were a leaf.
//! 3. **Tenant binding.** The tenant is the first field, so a receipt
//!    produced for one tenant's subject cannot be replayed as evidence for
//!    another tenant that happens to use the same request id.
//! 4. **Odd leaf rule: PROMOTE, never duplicate.** When a level has an odd
//!    count the last hash is carried up unchanged. The classic
//!    duplicate-last-leaf rule (Bitcoin CVE-2012-2459) makes `[a,b,c]` and
//!    `[a,b,c,c]` collide; promotion cannot, because a promoted value is a
//!    `0x00`-tagged leaf hash while the duplicated variant would be a
//!    `0x01`-tagged node hash.
//!
//! Leaves are ordered by microservice name ascending, NOT by arrival order,
//! so the root is independent of the order in which the fan-out completes.
//! Two receipts for the same microservice are refused rather than
//! deduplicated: silently dropping one would shrink the proof.
//!
//! # Coverage is a SET, not a count
//!
//! A certificate asserts coverage of a NAMED plan
//! ([`crate::kernel::ProofOfErasure::covered_microservices`]), and
//! completeness means every named microservice reported. Receipts from
//! services the current plan no longer names — a microservice decommissioned
//! inside the statutory window — are surplus EVIDENCE, not a defect: they
//! are carried in the tree and never block sealing. Comparing counts instead
//! would make a request whose registry shrank permanently unsealable, with
//! the subject fully erased and no certificate obtainable by any path,
//! including the DPO waiver.

use std::collections::{BTreeMap, BTreeSet};

use crate::digest::sha256;
use crate::kernel::{
    DpoOverride, DsrKernelError, DsrRequestKey, ErasureReceipt, ProofOfErasure, RegulatoryPack,
    Timestamp,
};

/// Domain-separation tag for leaf hashes.
const LEAF_TAG: u8 = 0x00;

/// Domain-separation tag for internal node hashes.
const NODE_TAG: u8 = 0x01;

/// Seconds in a calendar day, as used by every statutory window here.
const SECONDS_PER_DAY: i64 = 86_400;

/// Fraction of the SLA window, in fifths, after which the DPO is alerted.
const AT_RISK_NUMERATOR: i128 = 4;

/// Denominator matching [`AT_RISK_NUMERATOR`] (4/5 == 80% of the window).
const AT_RISK_DENOMINATOR: i128 = 5;

/// Append a length-prefixed field to a canonical encoding.
fn push_field(out: &mut Vec<u8>, bytes: &[u8]) -> Result<(), DsrKernelError> {
    let length = u64::try_from(bytes.len()).map_err(|_| DsrKernelError::ReceiptEncodingTooLarge)?;
    out.extend_from_slice(&length.to_be_bytes());
    out.extend_from_slice(bytes);
    Ok(())
}

/// The canonical byte encoding of a receipt — the COMPLETE leaf preimage,
/// `0x00` tag included.
///
/// Documented in the module header; changing it changes every root, so it
/// is part of the crate's published contract.
///
/// # Errors
/// [`DsrKernelError::ReceiptEncodingTooLarge`] if a field length exceeds
/// `u64` (unreachable on any supported target, but never assumed).
pub fn canonical_receipt_bytes(receipt: &ErasureReceipt) -> Result<Vec<u8>, DsrKernelError> {
    let mut out = Vec::with_capacity(
        96 + receipt.tenant.len() + receipt.microservice.len() + receipt.request.0.len(),
    );
    out.push(LEAF_TAG);
    push_field(&mut out, receipt.tenant.as_bytes())?;
    push_field(&mut out, receipt.request.0.as_bytes())?;
    push_field(&mut out, receipt.microservice.as_bytes())?;
    push_field(&mut out, &receipt.merkle_leaf)?;
    Ok(out)
}

/// The tree leaf for one receipt: `SHA256(canonical_receipt_bytes(receipt))`.
///
/// The `0x00` leaf tag is the first byte of those canonical bytes and is NOT
/// applied a second time here.
///
/// # Errors
/// Propagates [`canonical_receipt_bytes`].
pub fn leaf_hash(receipt: &ErasureReceipt) -> Result<[u8; 32], DsrKernelError> {
    Ok(sha256(&canonical_receipt_bytes(receipt)?))
}

/// Hash one internal node: `SHA256(0x01 || left || right)`.
#[must_use]
pub fn node_hash(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut preimage = Vec::with_capacity(65);
    preimage.push(NODE_TAG);
    preimage.extend_from_slice(left);
    preimage.extend_from_slice(right);
    sha256(&preimage)
}

/// Reduce leaves to a Merkle root, promoting a lone node at each level.
///
/// # Errors
/// [`DsrKernelError::MerkleAggregationFailed`] for an empty leaf set: there
/// is no such thing as a proof of erasure over nothing.
pub fn merkle_root(leaves: &[[u8; 32]]) -> Result<[u8; 32], DsrKernelError> {
    let mut level: Vec<[u8; 32]> = leaves.to_vec();
    let Some(first) = level.first().copied() else {
        return Err(DsrKernelError::MerkleAggregationFailed);
    };
    if level.len() == 1 {
        return Ok(first);
    }
    while level.len() > 1 {
        let mut next = Vec::with_capacity(level.len().div_ceil(2));
        for pair in level.chunks(2) {
            match pair {
                [left, right] => next.push(node_hash(left, right)),
                // Odd tail: promote unchanged (see module header).
                [lone] => next.push(*lone),
                _ => return Err(DsrKernelError::MerkleAggregationFailed),
            }
        }
        level = next;
    }
    level
        .first()
        .copied()
        .ok_or(DsrKernelError::MerkleAggregationFailed)
}

/// Order receipts canonically and reject anything that would make the root
/// ambiguous: a foreign tenant or request, or two receipts from one
/// microservice.
fn canonical_receipts(
    key: &DsrRequestKey,
    receipts: &[ErasureReceipt],
) -> Result<Vec<ErasureReceipt>, DsrKernelError> {
    let mut ordered: BTreeMap<String, ErasureReceipt> = BTreeMap::new();
    for receipt in receipts {
        if receipt.tenant != key.tenant || receipt.request != key.request {
            return Err(DsrKernelError::ForeignReceipt);
        }
        if ordered
            .insert(receipt.microservice.clone(), receipt.clone())
            .is_some()
        {
            return Err(DsrKernelError::DuplicateMicroserviceReceipt);
        }
    }
    if ordered.is_empty() {
        return Err(DsrKernelError::EmptyReceiptSet);
    }
    Ok(ordered.into_values().collect())
}

/// The canonical form of a cascade plan: sorted, deduplicated names.
///
/// # Errors
/// [`DsrKernelError::EmptyCascadePlan`] when no microservice is named.
pub fn canonical_microservices(microservices: &[String]) -> Result<Vec<String>, DsrKernelError> {
    let canonical: BTreeSet<String> = microservices.iter().cloned().collect();
    if canonical.is_empty() {
        return Err(DsrKernelError::EmptyCascadePlan);
    }
    Ok(canonical.into_iter().collect())
}

/// The Merkle root over a receipt set, in canonical order.
///
/// # Errors
/// [`DsrKernelError::EmptyReceiptSet`], [`DsrKernelError::ForeignReceipt`]
/// or [`DsrKernelError::DuplicateMicroserviceReceipt`] for an unusable set.
pub fn receipts_merkle_root(
    key: &DsrRequestKey,
    receipts: &[ErasureReceipt],
) -> Result<[u8; 32], DsrKernelError> {
    let ordered = canonical_receipts(key, receipts)?;
    let leaves = ordered
        .iter()
        .map(leaf_hash)
        .collect::<Result<Vec<_>, _>>()?;
    merkle_root(&leaves)
}

/// Which of `covered` has no receipt, in canonical order.
fn missing_coverage(covered: &[String], receipts: &[ErasureReceipt]) -> Vec<String> {
    let reported: BTreeSet<&str> = receipts
        .iter()
        .map(|receipt| receipt.microservice.as_str())
        .collect();
    covered
        .iter()
        .filter(|name| !reported.contains(name.as_str()))
        .cloned()
        .collect()
}

/// Seal a proof of erasure over `receipts`.
///
/// `covered_microservices` is the cascade plan the certificate asserts
/// coverage of. When a named microservice has NOT reported, a validated
/// [`DpoOverride`] is mandatory — the IP-009 halt condition "proof emitted
/// with received < expected and no DPO override — refuse". Receipts beyond
/// the plan are kept, not refused: see the module header.
///
/// # Errors
/// - [`DsrKernelError::EmptyReceiptSet`] / [`DsrKernelError::ForeignReceipt`]
///   / [`DsrKernelError::DuplicateMicroserviceReceipt`] — unusable set.
/// - [`DsrKernelError::EmptyCascadePlan`] — nothing named as covered.
/// - [`DsrKernelError::DpoOverrideRequired`] — incomplete without a waiver.
/// - [`DsrKernelError::InvalidDpoOverride`] — waiver is not dual control.
pub fn compute_proof_of_erasure(
    key: &DsrRequestKey,
    receipts: &[ErasureReceipt],
    covered_microservices: &[String],
    sealed_at: Timestamp,
    dpo_override: Option<DpoOverride>,
) -> Result<ProofOfErasure, DsrKernelError> {
    let ordered = canonical_receipts(key, receipts)?;
    let covered = canonical_microservices(covered_microservices)?;
    let complete = missing_coverage(&covered, &ordered).is_empty();
    match (&dpo_override, complete) {
        (None, false) => return Err(DsrKernelError::DpoOverrideRequired),
        (Some(waiver), _) => waiver.validate()?,
        (None, true) => {}
    }

    let leaves = ordered
        .iter()
        .map(leaf_hash)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ProofOfErasure {
        tenant: key.tenant.clone(),
        request: key.request.clone(),
        merkle_root: merkle_root(&leaves)?,
        receipts: ordered,
        expected_microservices: covered.len(),
        covered_microservices: covered,
        sealed_at,
        dpo_override,
    })
}

/// Re-derive a proof's root from its own receipts and check it.
///
/// This is the verifier a regulator runs: it catches a receipt that was
/// removed from, added to or altered inside a sealed certificate, a coverage
/// list that disagrees with its own stated count, and an incomplete
/// certificate carrying no waiver.
///
/// It CANNOT detect a certificate whose envelope was rewritten wholesale
/// (coverage list shrunk to match the receipts, waiver stripped and count
/// adjusted): only the receipts are bound by the root, and nothing here is
/// signed. That is the "no signature" gap in the crate header, and it is an
/// adapter's job, not this function's.
///
/// # Errors
/// [`DsrKernelError::RootMismatch`] on a root mismatch,
/// [`DsrKernelError::InconsistentProof`] when the coverage fields disagree,
/// [`DsrKernelError::DpoOverrideRequired`] /
/// [`DsrKernelError::InvalidDpoOverride`] for waiver defects, plus the
/// aggregation errors for an unusable receipt set.
pub fn verify_proof_of_erasure(proof: &ProofOfErasure) -> Result<(), DsrKernelError> {
    let key = proof.key();
    let recomputed = receipts_merkle_root(&key, &proof.receipts)?;
    if recomputed != proof.merkle_root {
        return Err(DsrKernelError::RootMismatch);
    }
    let covered = canonical_microservices(&proof.covered_microservices)?;
    if covered != proof.covered_microservices || covered.len() != proof.expected_microservices {
        return Err(DsrKernelError::InconsistentProof);
    }
    let complete = missing_coverage(&covered, &proof.receipts).is_empty();
    match &proof.dpo_override {
        Some(waiver) => waiver.validate(),
        None if complete => Ok(()),
        None => Err(DsrKernelError::DpoOverrideRequired),
    }
}

/// The statutory response window for a pack, in days.
#[must_use]
pub const fn sla_days(pack: RegulatoryPack) -> i64 {
    match pack {
        RegulatoryPack::Br => 15,
        RegulatoryPack::UsHc => 7,
        RegulatoryPack::Eu | RegulatoryPack::Kr | RegulatoryPack::In | RegulatoryPack::Default => {
            30
        }
    }
}

/// The deadline for a request under its pack.
///
/// # Errors
/// [`DsrKernelError::TimestampOverflow`] if the deadline leaves `i64`.
pub fn sla_deadline(
    pack: RegulatoryPack,
    requested_at: Timestamp,
) -> Result<Timestamp, DsrKernelError> {
    let window = sla_days(pack)
        .checked_mul(SECONDS_PER_DAY)
        .ok_or(DsrKernelError::TimestampOverflow)?;
    requested_at
        .0
        .checked_add(window)
        .map(Timestamp)
        .ok_or(DsrKernelError::TimestampOverflow)
}

/// Whether the DPO should be alerted: 80% or more of the window is spent.
///
/// Returns `true` for an already-breached deadline as well — an alert is
/// never wrong once past the line.
///
/// # Errors
/// [`DsrKernelError::TimestampOverflow`] if the window or the elapsed span
/// is not representable, or the deadline precedes the request.
pub fn sla_at_risk(
    requested_at: Timestamp,
    deadline: Timestamp,
    now: Timestamp,
) -> Result<bool, DsrKernelError> {
    let window = requested_at
        .seconds_until(deadline)
        .ok_or(DsrKernelError::TimestampOverflow)?;
    if window <= 0 {
        return Err(DsrKernelError::TimestampOverflow);
    }
    let elapsed = requested_at
        .seconds_until(now)
        .ok_or(DsrKernelError::TimestampOverflow)?;
    if elapsed <= 0 {
        return Ok(false);
    }
    // Integer comparison in i128 so no product can overflow: alert once
    // elapsed/window >= 4/5.
    Ok(i128::from(elapsed) * AT_RISK_DENOMINATOR >= i128::from(window) * AT_RISK_NUMERATOR)
}

/// Whether the deadline has passed.
#[must_use]
pub const fn sla_breached(deadline: Timestamp, now: Timestamp) -> bool {
    now.0 > deadline.0
}
