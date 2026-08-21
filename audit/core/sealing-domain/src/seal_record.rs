//! Seal-record construction: builds a [`SealRecord`] from accumulated
//! leaves, enforcing the invariants a persisted seal must satisfy before any
//! `IndexWriter` / `ObjectStoreWriter` adapter is allowed to write it.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use audit_chain_domain::Sha256Hash;
use audit_sealing_kernel::{MerkleEngine, SealRecord, SealStatus, SigningKeyRef};

use crate::SealingDomainError;

/// Caller's attestation of what precedes this seal period for the same
/// `(pack, tenant_partition)`.
///
/// A pure domain crate performs no I/O and so cannot itself look up whether
/// an earlier sealed period exists; the caller (who DOES have a read path —
/// e.g. via whatever backs the `IndexWriter` adapter) must say so
/// explicitly. [`PriorPeriod::First`] carrying no root value does NOT, by
/// itself, make a false firstness claim impossible — a unit variant is just
/// as freely constructible as `Option::None` was. [`build_seal_record`]
/// closes that gap the only way a pure domain crate can: by taking a
/// [`PriorPeriodLookup`] port and calling it to VERIFY the claim (the same
/// pattern already used for `MerkleEngine` — a caller-supplied port the
/// domain calls and checks, not a value the domain trusts on its word).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PriorPeriod {
    /// This is the first period ever sealed for this `(pack, tenant_partition)`.
    /// [`build_seal_record`] verifies this via [`PriorPeriodLookup`] before
    /// accepting it — see [`SealingDomainError::FalseFirstPeriodClaim`].
    First,
    /// The immediately preceding sealed period's root, for chaining.
    Preceding { root: String },
}

/// Port this domain owns for verifying a [`PriorPeriod::First`] claim.
///
/// This crate has no read path of its own (it is pure / I/O-free), so it
/// cannot look up whether a sealed period already exists for
/// `(pack, tenant_partition)`. The caller — composed with whatever backs the
/// real seal-index read path — answers that question here, and
/// [`build_seal_record`] refuses to trust an unverified `First` claim: it
/// calls this port and rejects the claim with
/// [`SealingDomainError::FalseFirstPeriodClaim`] when it comes back false.
///
/// This is the "PORT the domain owns ... a caller-supplied value the domain
/// VERIFIES" shape, mirroring how [`audit_sealing_kernel::MerkleEngine`] is
/// already threaded through this same function.
pub trait PriorPeriodLookup {
    type Error;

    /// Returns `Ok(true)` only when no sealed period already exists for
    /// `(pack, tenant_partition)` — i.e. this genuinely would be the first
    /// period ever sealed for it.
    fn is_first_period(&self, pack: &str, tenant_partition: &str) -> Result<bool, Self::Error>;
}

/// Caller-supplied inputs for [`build_seal_record`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SealRecordInput {
    pub pack: String,
    pub tenant_partition: String,
    pub period_id: String,
    /// Ordered leaf hashes committed by this seal period.
    pub leaves: Vec<Sha256Hash>,
    /// The leaf count the caller's upstream accumulator believes it handed
    /// over. Checked against `leaves.len()` rather than trusted outright —
    /// see [`SealingDomainError::LeafCountMismatch`].
    pub declared_leaf_count: u64,
    pub prior_period: PriorPeriod,
    pub signing_key: SigningKeyRef,
}

/// Build a [`SealRecord`] from `input`, computing its Merkle root via
/// `engine` (typically [`crate::merkle_engine::MerkleTreeEngine`]) and
/// verifying any [`PriorPeriod::First`] claim via `prior_period_lookup`
/// (see [`PriorPeriodLookup`]).
///
/// The returned record's `status` is always [`SealStatus::Sealed`]: fixing
/// a `merkle_root` over a closed leaf set IS the sealing act this function
/// performs. The earlier `Accepted` and `Unsealed` lifecycle states describe
/// a pre-image accumulation phase this constructor does not model (that
/// phase precedes having a leaf set to build a tree from at all); later
/// states are reached only via [`crate::status::apply_seal_status_transition`].
///
/// Validation runs in this order — pack/tenant identity first, then the
/// leaf-count contract, then prior-period chaining (including the
/// [`PriorPeriodLookup`] call for a `First` claim), then the Merkle build
/// itself, then a self-reference check on the finished root — so the first
/// structural problem in `input` is always the one reported:
///
/// # Errors
/// - [`SealingDomainError::EmptyPack`] — `input.pack` is empty/whitespace.
/// - [`SealingDomainError::EmptyTenantPartition`] — `input.tenant_partition`
///   is empty/whitespace.
/// - [`SealingDomainError::LeafCountMismatch`] — `input.leaves.len()` does
///   not equal `input.declared_leaf_count`.
/// - [`SealingDomainError::FalseFirstPeriodClaim`] — `input.prior_period` is
///   [`PriorPeriod::First`] but `prior_period_lookup` reports a sealed
///   period already exists for `(input.pack, input.tenant_partition)`.
/// - [`SealingDomainError::EmptyPriorRoot`] — `input.prior_period` is
///   [`PriorPeriod::Preceding`] with an empty/whitespace `root`.
/// - [`SealingDomainError::MalformedPriorRoot`] — `input.prior_period` is
///   [`PriorPeriod::Preceding`] with a non-empty `root` that is not shaped
///   like `sha256:` + 64 lowercase hex characters.
/// - [`SealingDomainError::InvalidLeafCount`] — `input.leaves` is empty
///   (propagated from `engine.root`).
/// - [`SealingDomainError::SelfReferentialPriorRoot`] — `input.prior_period`
///   is [`PriorPeriod::Preceding`] with a well-formed `root` equal to this
///   record's own freshly computed `merkle_root`.
pub fn build_seal_record<E, L>(
    input: SealRecordInput,
    engine: &E,
    prior_period_lookup: &L,
) -> Result<SealRecord, SealingDomainError>
where
    E: MerkleEngine<Leaf = Sha256Hash, Root = Sha256Hash, Error = SealingDomainError>,
    L: PriorPeriodLookup<Error = SealingDomainError>,
{
    if input.pack.trim().is_empty() {
        return Err(SealingDomainError::EmptyPack);
    }
    if input.tenant_partition.trim().is_empty() {
        return Err(SealingDomainError::EmptyTenantPartition);
    }
    let actual_leaf_count = input.leaves.len() as u64;
    if actual_leaf_count != input.declared_leaf_count {
        return Err(SealingDomainError::LeafCountMismatch {
            declared: input.declared_leaf_count,
            actual: actual_leaf_count,
        });
    }
    let prior_root = match &input.prior_period {
        PriorPeriod::First => {
            let is_first =
                prior_period_lookup.is_first_period(&input.pack, &input.tenant_partition)?;
            if !is_first {
                return Err(SealingDomainError::FalseFirstPeriodClaim {
                    pack: input.pack.clone(),
                    tenant_partition: input.tenant_partition.clone(),
                });
            }
            None
        }
        PriorPeriod::Preceding { root } => {
            if root.trim().is_empty() {
                return Err(SealingDomainError::EmptyPriorRoot);
            }
            if !is_well_formed_root(root) {
                return Err(SealingDomainError::MalformedPriorRoot { root: root.clone() });
            }
            Some(root.clone())
        }
    };
    let root = engine.root(&input.leaves)?;
    let merkle_root = encode_root(root);
    if prior_root.as_deref() == Some(merkle_root.as_str()) {
        return Err(SealingDomainError::SelfReferentialPriorRoot);
    }
    Ok(SealRecord {
        pack: input.pack,
        tenant_partition: input.tenant_partition,
        period_id: input.period_id,
        leaf_count: input.declared_leaf_count,
        merkle_root,
        prior_root,
        signing_key: input.signing_key,
        status: SealStatus::Sealed,
    })
}

/// Returns `true` when `s` has exactly the shape [`encode_root`] emits:
/// the literal prefix `sha256:` followed by exactly 64 lowercase hex digits.
///
/// This crate cannot verify that a `PriorPeriod::Preceding` root is the
/// REAL prior period's root (it has no read path to confirm that), but it
/// is the sole producer of the root string format and can and does reject
/// values that could never have come from it — garbage, non-hex text,
/// wrong length, or non-`White_Space` filler (e.g. U+200B ZERO WIDTH SPACE)
/// that `str::trim` does not strip.
fn is_well_formed_root(s: &str) -> bool {
    match s.strip_prefix("sha256:") {
        Some(hex) => {
            hex.len() == 64 && hex.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        }
        None => false,
    }
}

/// Encode a raw Merkle root as the `SealRecord.merkle_root` string form:
/// `sha256:<lower-hex>`. This is a plain hex encoding local to this crate —
/// distinct from `audit_chain_domain`'s own `"merkle-sha256:…"` ledger-root
/// format, which describes a different value (the append-only event chain's
/// root, not a sealed pack period's root).
fn encode_root(root: Sha256Hash) -> String {
    format!("sha256:{}", encode_hex(&root))
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::merkle_engine::MerkleTreeEngine;

    fn leaf(seed: u8) -> Sha256Hash {
        let mut bytes = [0_u8; 32];
        bytes[0] = seed;
        bytes
    }

    /// Test double: every `(pack, tenant_partition)` genuinely has no prior
    /// sealed period.
    struct AlwaysFirst;
    impl PriorPeriodLookup for AlwaysFirst {
        type Error = SealingDomainError;
        fn is_first_period(
            &self,
            _pack: &str,
            _tenant_partition: &str,
        ) -> Result<bool, SealingDomainError> {
            Ok(true)
        }
    }

    /// Test double: every `(pack, tenant_partition)` already has a sealed
    /// period — any [`PriorPeriod::First`] claim against it is false.
    struct NeverFirst;
    impl PriorPeriodLookup for NeverFirst {
        type Error = SealingDomainError;
        fn is_first_period(
            &self,
            _pack: &str,
            _tenant_partition: &str,
        ) -> Result<bool, SealingDomainError> {
            Ok(false)
        }
    }

    /// A well-formed (but fake) prior-root string: `sha256:` + 64 lowercase
    /// hex digits, distinct from any root this test module's leaf sets
    /// actually build.
    fn well_formed_fake_root() -> String {
        format!("sha256:{}", "b".repeat(64))
    }

    fn valid_input() -> SealRecordInput {
        SealRecordInput {
            pack: "pack-alpha".to_string(),
            tenant_partition: "tenant-1".to_string(),
            period_id: "2026-08".to_string(),
            leaves: vec![leaf(1), leaf(2), leaf(3)],
            declared_leaf_count: 3,
            prior_period: PriorPeriod::First,
            signing_key: SigningKeyRef {
                key_id: "key-1".to_string(),
            },
        }
    }

    #[test]
    fn builds_sealed_record_with_first_period_prior_root_none() {
        let record = build_seal_record(valid_input(), &MerkleTreeEngine, &AlwaysFirst)
            .expect("valid input builds a record");
        assert_eq!(record.status, SealStatus::Sealed);
        assert_eq!(record.prior_root, None);
        assert_eq!(record.leaf_count, 3);
        assert!(record.merkle_root.starts_with("sha256:"));
        assert_eq!(record.merkle_root.len(), "sha256:".len() + 64);
    }

    /// Known-answer anchor: pins the exact `merkle_root` a fixed 3-leaf set
    /// must produce under RFC 6962 §2.1 (`audit_chain_domain::MerkleTree`).
    /// Independently confirmed by running
    /// `audit_chain_domain::MerkleTree::new(leaves).build_root()` over the
    /// same three leaves and hex-encoding the result outside this crate's
    /// own `encode_root`. Unlike a shape-only assertion
    /// (`starts_with("sha256:")` + `len() == 71`, which any fabricated
    /// 64-hex string satisfies), this test fails if `build_seal_record` ever
    /// stops committing to the actual leaf set — e.g. sorts leaves, drops a
    /// leaf, or hashes a fixed placeholder.
    #[test]
    fn merkle_root_is_a_known_answer_for_a_fixed_leaf_set() {
        let record = build_seal_record(valid_input(), &MerkleTreeEngine, &AlwaysFirst)
            .expect("valid input builds a record");
        assert_eq!(
            record.merkle_root,
            "sha256:d300790a304c441999aa258bb23cd0ad8f2b6ce25f9546894788201bb9bd6ff7"
        );
    }

    #[test]
    fn merkle_root_is_order_dependent() {
        let mut reordered = valid_input();
        reordered.leaves = vec![leaf(3), leaf(2), leaf(1)];
        let forward = build_seal_record(valid_input(), &MerkleTreeEngine, &AlwaysFirst)
            .expect("forward order builds");
        let backward = build_seal_record(reordered, &MerkleTreeEngine, &AlwaysFirst)
            .expect("reversed order builds");
        assert_ne!(forward.merkle_root, backward.merkle_root);
    }

    #[test]
    fn builds_chained_record_with_preceding_root() {
        let mut input = valid_input();
        let prior = well_formed_fake_root();
        input.prior_period = PriorPeriod::Preceding {
            root: prior.clone(),
        };
        let record = build_seal_record(input, &MerkleTreeEngine, &AlwaysFirst)
            .expect("valid chained input builds a record");
        assert_eq!(record.prior_root, Some(prior));
    }

    #[test]
    fn rejects_empty_pack() {
        let mut input = valid_input();
        input.pack = "   ".to_string();
        assert_eq!(
            build_seal_record(input, &MerkleTreeEngine, &AlwaysFirst),
            Err(SealingDomainError::EmptyPack)
        );
    }

    #[test]
    fn rejects_empty_tenant_partition() {
        let mut input = valid_input();
        input.tenant_partition = String::new();
        assert_eq!(
            build_seal_record(input, &MerkleTreeEngine, &AlwaysFirst),
            Err(SealingDomainError::EmptyTenantPartition)
        );
    }

    #[test]
    fn rejects_leaf_count_mismatch_when_declared_is_higher() {
        let mut input = valid_input();
        input.declared_leaf_count = 4;
        assert_eq!(
            build_seal_record(input, &MerkleTreeEngine, &AlwaysFirst),
            Err(SealingDomainError::LeafCountMismatch {
                declared: 4,
                actual: 3,
            })
        );
    }

    #[test]
    fn rejects_leaf_count_mismatch_when_declared_is_lower() {
        let mut input = valid_input();
        input.declared_leaf_count = 2;
        assert_eq!(
            build_seal_record(input, &MerkleTreeEngine, &AlwaysFirst),
            Err(SealingDomainError::LeafCountMismatch {
                declared: 2,
                actual: 3,
            })
        );
    }

    #[test]
    fn rejects_empty_preceding_root() {
        let mut input = valid_input();
        input.prior_period = PriorPeriod::Preceding {
            root: "  ".to_string(),
        };
        assert_eq!(
            build_seal_record(input, &MerkleTreeEngine, &AlwaysFirst),
            Err(SealingDomainError::EmptyPriorRoot)
        );
    }

    #[test]
    fn rejects_malformed_preceding_root_plain_garbage() {
        let mut input = valid_input();
        input.prior_period = PriorPeriod::Preceding {
            root: "not-a-root-at-all".to_string(),
        };
        assert_eq!(
            build_seal_record(input, &MerkleTreeEngine, &AlwaysFirst),
            Err(SealingDomainError::MalformedPriorRoot {
                root: "not-a-root-at-all".to_string(),
            })
        );
    }

    #[test]
    fn rejects_malformed_preceding_root_wrong_length_hex() {
        let mut input = valid_input();
        input.prior_period = PriorPeriod::Preceding {
            root: "sha256:deadbeef".to_string(),
        };
        assert_eq!(
            build_seal_record(input, &MerkleTreeEngine, &AlwaysFirst),
            Err(SealingDomainError::MalformedPriorRoot {
                root: "sha256:deadbeef".to_string(),
            })
        );
    }

    #[test]
    fn rejects_malformed_preceding_root_non_hex_after_prefix() {
        let mut input = valid_input();
        let bad = format!("sha256:{}", "z".repeat(64));
        input.prior_period = PriorPeriod::Preceding { root: bad.clone() };
        assert_eq!(
            build_seal_record(input, &MerkleTreeEngine, &AlwaysFirst),
            Err(SealingDomainError::MalformedPriorRoot { root: bad })
        );
    }

    #[test]
    fn rejects_malformed_preceding_root_zero_width_space() {
        // U+200B ZERO WIDTH SPACE is not `White_Space`, so `str::trim` does
        // not strip it and `EmptyPriorRoot` does not fire — the format check
        // must catch it instead.
        let mut input = valid_input();
        input.prior_period = PriorPeriod::Preceding {
            root: "\u{200b}".to_string(),
        };
        assert_eq!(
            build_seal_record(input, &MerkleTreeEngine, &AlwaysFirst),
            Err(SealingDomainError::MalformedPriorRoot {
                root: "\u{200b}".to_string(),
            })
        );
    }

    #[test]
    fn rejects_self_referential_preceding_root() {
        // First compute what this input's own merkle_root will be, then
        // claim that exact value as the prior root.
        let sealed = build_seal_record(valid_input(), &MerkleTreeEngine, &AlwaysFirst)
            .expect("first pass builds a record");
        let mut input = valid_input();
        input.prior_period = PriorPeriod::Preceding {
            root: sealed.merkle_root,
        };
        assert_eq!(
            build_seal_record(input, &MerkleTreeEngine, &AlwaysFirst),
            Err(SealingDomainError::SelfReferentialPriorRoot)
        );
    }

    #[test]
    fn rejects_false_first_period_claim() {
        assert_eq!(
            build_seal_record(valid_input(), &MerkleTreeEngine, &NeverFirst),
            Err(SealingDomainError::FalseFirstPeriodClaim {
                pack: "pack-alpha".to_string(),
                tenant_partition: "tenant-1".to_string(),
            })
        );
    }

    #[test]
    fn accepts_true_first_period_claim() {
        assert!(build_seal_record(valid_input(), &MerkleTreeEngine, &AlwaysFirst).is_ok());
    }

    #[test]
    fn repeated_first_claims_for_the_same_pack_are_each_independently_checked() {
        // Regression anchor: an operator cannot re-seal every period as
        // `First` and get a free pass — each call is checked against the
        // lookup port, so a lookup that (correctly) reports "not first"
        // after the first successful seal rejects every subsequent claim.
        for period in ["2026-09", "2026-10", "2026-11", "2027-01"] {
            let mut input = valid_input();
            input.period_id = period.to_string();
            assert_eq!(
                build_seal_record(input, &MerkleTreeEngine, &NeverFirst),
                Err(SealingDomainError::FalseFirstPeriodClaim {
                    pack: "pack-alpha".to_string(),
                    tenant_partition: "tenant-1".to_string(),
                }),
                "period {period} must not be sealable as First once the lookup says otherwise"
            );
        }
    }

    #[test]
    fn rejects_empty_leaf_set_via_engine() {
        let mut input = valid_input();
        input.leaves = Vec::new();
        input.declared_leaf_count = 0;
        assert_eq!(
            build_seal_record(input, &MerkleTreeEngine, &AlwaysFirst),
            Err(SealingDomainError::InvalidLeafCount)
        );
    }

    #[test]
    fn accepts_trailing_duplicate_leaf_via_engine() {
        // RFC 6962's k-split needs no leaf-shape restriction (see
        // merkle_engine::tests for the collision-freedom regression); a
        // trailing duplicate leaf builds a record like any other input.
        let mut input = valid_input();
        input.leaves = vec![leaf(1), leaf(2), leaf(3), leaf(3)];
        input.declared_leaf_count = 4;
        assert!(build_seal_record(input, &MerkleTreeEngine, &AlwaysFirst).is_ok());
    }

    #[test]
    fn empty_pack_takes_priority_over_leaf_count_mismatch() {
        // Ordering contract: pack/tenant checks run before the leaf-count
        // contract, so a record broken in both ways reports the pack error.
        let mut input = valid_input();
        input.pack = String::new();
        input.declared_leaf_count = 99;
        assert_eq!(
            build_seal_record(input, &MerkleTreeEngine, &AlwaysFirst),
            Err(SealingDomainError::EmptyPack)
        );
    }
}
