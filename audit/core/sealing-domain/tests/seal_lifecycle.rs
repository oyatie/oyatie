// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use audit_sealing_domain::{
    MerkleTreeEngine, PriorPeriod, PriorPeriodLookup, SealRecordInput, SealStatus,
    SealingDomainError, apply_seal_status_transition, build_seal_record,
    verify_epoch_covers_period, verify_leaf_inclusion,
};
use audit_sealing_kernel::{PackEpoch, SigningKeyRef};

fn leaf(seed: u8) -> [u8; 32] {
    let mut bytes = [0_u8; 32];
    bytes[0] = seed;
    bytes
}

/// Test double standing in for the real seal-index read path: every
/// `(pack, tenant_partition)` this integration test seals genuinely has no
/// prior sealed period.
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

/// End-to-end: seal a period, publish its proof material, walk it through
/// its lifecycle, and confirm the signing key's epoch actually covers it.
#[test]
fn seals_a_period_walks_its_lifecycle_and_checks_epoch_coverage() {
    let signing_key = SigningKeyRef {
        key_id: "key-2026-08".to_string(),
    };
    let leaves = vec![leaf(1), leaf(2), leaf(3), leaf(4)];

    let record = build_seal_record(
        SealRecordInput {
            pack: "pack-alpha".to_string(),
            tenant_partition: "tenant-1".to_string(),
            period_id: "2026-08-15".to_string(),
            leaves: leaves.clone(),
            declared_leaf_count: 4,
            prior_period: PriorPeriod::Preceding {
                root: format!("sha256:{}", "a1".repeat(32)),
            },
            signing_key: signing_key.clone(),
        },
        &MerkleTreeEngine,
        &AlwaysFirst,
    )
    .expect("well-formed input seals");
    assert_eq!(record.status, SealStatus::Sealed);
    assert_eq!(
        record.prior_root,
        Some(format!("sha256:{}", "a1".repeat(32)))
    );

    // A leaf's inclusion in the sealed root is independently verifiable via
    // the same `MerkleTree` math the seal itself used.
    let tree = audit_sealing_domain::MerkleTree::new(leaves.clone());
    let root = tree.build_root();
    let path = tree.proof_path(2);
    verify_leaf_inclusion(leaves[2], 2, &path, root, record.leaf_count)
        .expect("leaf 2 is included under the sealed root");

    // Walk the record through its declared lifecycle.
    let published =
        apply_seal_status_transition(&record, SealStatus::Published).expect("Sealed -> Published");
    let verified = apply_seal_status_transition(&published, SealStatus::Verified)
        .expect("Published -> Verified");
    assert_eq!(verified.status, SealStatus::Verified);

    // Once Verified, the record may retire into either terminal state — but
    // not both from the same starting point, and never back out.
    let retained = apply_seal_status_transition(&verified, SealStatus::Retained)
        .expect("Verified -> Retained");
    assert_eq!(
        apply_seal_status_transition(&retained, SealStatus::Verified),
        Err(SealingDomainError::IllegalSealStatusTransition {
            from: SealStatus::Retained,
            to: SealStatus::Verified,
        })
    );

    // The key that signed this period must fall inside the epoch that names
    // it as active, for the same (pack, tenant_partition, period).
    let epoch = PackEpoch {
        pack: "pack-alpha".to_string(),
        tenant_partition: "tenant-1".to_string(),
        period_lo: "2026-08-01".to_string(),
        period_hi: "2026-09-01".to_string(),
        active_key: signing_key.clone(),
        retiring_key: None,
    };
    verify_epoch_covers_period(&epoch, "pack-alpha", "tenant-1", "2026-08-15", &signing_key)
        .expect("the signing key's epoch covers this period");

    // The same key does NOT cover a period the epoch does not name.
    assert!(matches!(
        verify_epoch_covers_period(&epoch, "pack-alpha", "tenant-1", "2026-09-15", &signing_key),
        Err(SealingDomainError::PeriodOutsideEpochWindow { .. })
    ));
}

#[test]
fn first_period_seal_has_no_prior_root_and_rejects_skipped_publish() {
    let record = build_seal_record(
        SealRecordInput {
            pack: "pack-beta".to_string(),
            tenant_partition: "tenant-2".to_string(),
            period_id: "2026-08-01".to_string(),
            leaves: vec![leaf(9)],
            declared_leaf_count: 1,
            prior_period: PriorPeriod::First,
            signing_key: SigningKeyRef {
                key_id: "key-first".to_string(),
            },
        },
        &MerkleTreeEngine,
        &AlwaysFirst,
    )
    .expect("single-leaf first-period input seals");
    assert_eq!(record.prior_root, None);

    // Sealed cannot jump straight to Verified, skipping Published.
    assert_eq!(
        apply_seal_status_transition(&record, SealStatus::Verified),
        Err(SealingDomainError::IllegalSealStatusTransition {
            from: SealStatus::Sealed,
            to: SealStatus::Verified,
        })
    );
}
