use oya_foundry_catalog_kernel::{CatalogIndex, CatalogRecordInput};
use oya_foundry_claim_ceiling_kernel::{ClaimCeilingError, FoundationClaimCeiling};

#[test]
fn foundation_claim_ceiling_accepts_preview_source_only_records() {
    let index = CatalogIndex::from_records(vec![record(
        "oya-foundry-capability-kernel",
        "preview",
        "unreviewed",
        "source-only",
    )
    .build()
    .unwrap()])
    .expect("catalog index is valid");

    assert_eq!(
        FoundationClaimCeiling::preview_foundation().validate_catalog(&index),
        Ok(())
    );
}

#[test]
fn foundation_claim_ceiling_blocks_unshipped_stability_security_and_supply_chain_claims() {
    let stable_api = CatalogIndex::from_records(vec![record(
        "oya-foundry-capability-kernel",
        "stable",
        "unreviewed",
        "source-only",
    )
    .build()
    .unwrap()])
    .expect("catalog index is valid");
    assert_eq!(
        FoundationClaimCeiling::preview_foundation().validate_catalog(&stable_api),
        Err(ClaimCeilingError::ApiStabilityAboveFoundation)
    );

    let security_review = CatalogIndex::from_records(vec![record(
        "oya-foundry-capability-kernel",
        "preview",
        "independent",
        "source-only",
    )
    .build()
    .unwrap()])
    .expect("catalog index is valid");
    assert_eq!(
        FoundationClaimCeiling::preview_foundation().validate_catalog(&security_review),
        Err(ClaimCeilingError::SecurityReviewAboveFoundation)
    );

    let signed_supply_chain = CatalogIndex::from_records(vec![record(
        "oya-foundry-capability-kernel",
        "preview",
        "unreviewed",
        "signed-provenance",
    )
    .build()
    .unwrap()])
    .expect("catalog index is valid");
    assert_eq!(
        FoundationClaimCeiling::preview_foundation().validate_catalog(&signed_supply_chain),
        Err(ClaimCeilingError::SupplyChainAboveFoundation)
    );
}

fn record(
    crate_id: &str,
    api_stability: &str,
    security_review: &str,
    supply_chain: &str,
) -> CatalogRecordInput {
    CatalogRecordInput {
        crate_id: crate_id.into(),
        context: "foundry".into(),
        role: "kernel".into(),
        capability: "capability".into(),
        plane: "control".into(),
        data_classes_owned: vec!["INTERNAL_ONLY".into()],
        operational_classes_owned: Vec::new(),
        api_stability: api_stability.into(),
        security_review: security_review.into(),
        supply_chain: supply_chain.into(),
    }
}
