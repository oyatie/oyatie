// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use data_boundary_kernel::{DataClass, OperationalDataClass, privacy_data_classes_from};
use intelligence_catalog_domain::{
    ApiStability, CatalogError, CatalogIndex, CatalogRecordInput, CatalogRole, SecurityReview,
    SupplyChainAttestation,
};

#[test]
fn catalog_record_validates_existing_registry_shape() {
    let record = valid_record("intelligence-capability-kernel")
        .build()
        .expect("catalog record is valid");

    assert_eq!(record.crate_id.value, "intelligence-capability-kernel");
    assert_eq!(record.role.value, CatalogRole::Kernel);
    assert_eq!(record.api_stability.value, ApiStability::Preview);
    assert_eq!(record.security_review.value, SecurityReview::Unreviewed);
    assert_eq!(
        record.supply_chain.value,
        SupplyChainAttestation::SourceOnly
    );
    assert_eq!(
        record.privacy_data_classes_owned().value,
        privacy_data_classes_from(&[DataClass::InternalOnly]).unwrap()
    );
    assert_eq!(
        record.legacy_data_classes_owned().value,
        vec![DataClass::InternalOnly]
    );
    #[allow(deprecated)]
    {
        assert_eq!(
            record.data_classes_owned(),
            record.legacy_data_classes_owned()
        );
    }
    assert_eq!(
        record.operational_classes_owned.value,
        vec![OperationalDataClass::Audit]
    );
    assert_eq!(record.schema_version.value, 1);
}

#[test]
fn catalog_record_accepts_de_branded_crate_ids() {
    // Catalog crate-ids mirror live, de-branded workspace crate names
    // (e.g. `marketplace/core/plugin-kernel`).
    for crate_id in [
        "marketplace-plugin-kernel",
        "observability-aggregate",
        "iac-core-domain",
        "intelligence-capability-kernel",
    ] {
        let record = valid_record(crate_id)
            .build()
            .unwrap_or_else(|error| panic!("{crate_id} should be valid, got {error:?}"));
        assert_eq!(record.crate_id.value, crate_id);
    }
}

#[test]
fn catalog_record_rejects_invalid_crate_id() {
    for crate_id in ["", "-foo", "Foo", "foo_bar"] {
        let invalid = CatalogRecordInput {
            ..valid_record(crate_id)
        };
        assert_eq!(
            invalid.build(),
            Err(CatalogError::InvalidCrateId),
            "{crate_id:?} should be rejected"
        );
    }
}

#[test]
fn catalog_record_accepts_usecase_role() {
    let record = CatalogRecordInput {
        role: "usecase".into(),
        ..valid_record("intelligence-subagent-runtime-usecase")
    }
    .build()
    .expect("usecase role is valid");

    assert_eq!(record.role.value, CatalogRole::Usecase);
}

#[test]
fn catalog_record_rejects_non_privacy_owned_data_class_labels() {
    for data_class in ["AUDIT", "SECRET", "CHILDREN"] {
        let invalid = CatalogRecordInput {
            data_classes_owned: vec![data_class.into()],
            ..valid_record("intelligence-capability-kernel")
        };

        assert_eq!(invalid.build(), Err(CatalogError::InvalidDataClass));
    }
}

#[test]
fn catalog_record_rejects_invalid_role_plane_context_and_missing_classes() {
    let invalid_role = CatalogRecordInput {
        role: "service".into(),
        ..valid_record("intelligence-capability-kernel")
    };
    assert_eq!(invalid_role.build(), Err(CatalogError::InvalidRole));

    let invalid_plane = CatalogRecordInput {
        plane: "frontend".into(),
        ..valid_record("intelligence-capability-kernel")
    };
    assert_eq!(invalid_plane.build(), Err(CatalogError::InvalidPlane));

    let empty_context = CatalogRecordInput {
        context: "".into(),
        ..valid_record("intelligence-capability-kernel")
    };
    assert_eq!(empty_context.build(), Err(CatalogError::EmptyContext));

    let missing_classes = CatalogRecordInput {
        data_classes_owned: Vec::new(),
        ..valid_record("intelligence-capability-kernel")
    };
    assert_eq!(
        missing_classes.build(),
        Err(CatalogError::MissingDataClasses)
    );

    let invalid_operational_class = CatalogRecordInput {
        operational_classes_owned: vec!["CHILDREN".into()],
        ..valid_record("intelligence-capability-kernel")
    };
    assert_eq!(
        invalid_operational_class.build(),
        Err(CatalogError::InvalidDataClass)
    );

    let invalid_api_stability = CatalogRecordInput {
        api_stability: "production".into(),
        ..valid_record("intelligence-capability-kernel")
    };
    assert_eq!(
        invalid_api_stability.build(),
        Err(CatalogError::InvalidApiStability)
    );

    let invalid_security_review = CatalogRecordInput {
        security_review: "rubber-stamped".into(),
        ..valid_record("intelligence-capability-kernel")
    };
    assert_eq!(
        invalid_security_review.build(),
        Err(CatalogError::InvalidSecurityReview)
    );

    let invalid_supply_chain = CatalogRecordInput {
        supply_chain: "magic".into(),
        ..valid_record("intelligence-capability-kernel")
    };
    assert_eq!(
        invalid_supply_chain.build(),
        Err(CatalogError::InvalidSupplyChain)
    );
}

#[test]
fn catalog_index_rejects_duplicates_and_missing_workspace_records() {
    let first = valid_record("intelligence-capability-kernel")
        .build()
        .unwrap();
    let duplicate = first.clone();
    assert_eq!(
        CatalogIndex::from_records(vec![first.clone(), duplicate]),
        Err(CatalogError::DuplicateCrateRecord)
    );

    let index = CatalogIndex::from_records(vec![first]).expect("index is valid");
    assert!(index.lookup("intelligence-capability-kernel").is_some());
    assert_eq!(
        index.validate_required_crates([
            "intelligence-capability-kernel",
            "intelligence-run-kernel"
        ]),
        Err(CatalogError::MissingCrateRecord {
            crate_id: "intelligence-run-kernel".into(),
        })
    );
}

#[test]
fn catalog_index_requires_review_for_plane_class_changes() {
    let baseline = CatalogIndex::from_records(vec![
        valid_record_with_plane("intelligence-capability-kernel", "control")
            .build()
            .unwrap(),
    ])
    .expect("baseline index is valid");
    let current = CatalogIndex::from_records(vec![
        valid_record_with_plane("intelligence-capability-kernel", "data")
            .build()
            .unwrap(),
    ])
    .expect("current index is valid");

    assert_eq!(
        current.validate_plane_stability(&baseline, std::iter::empty::<&str>()),
        Err(CatalogError::PlaneChanged)
    );
    assert_eq!(
        current.validate_plane_stability(&baseline, ["intelligence-capability-kernel"]),
        Ok(())
    );
}

fn valid_record(crate_id: &str) -> CatalogRecordInput {
    valid_record_with_plane(crate_id, "control")
}

fn valid_record_with_plane(crate_id: &str, plane: &str) -> CatalogRecordInput {
    CatalogRecordInput {
        crate_id: crate_id.into(),
        context: "foundry".into(),
        role: "kernel".into(),
        capability: "capability".into(),
        plane: plane.into(),
        data_classes_owned: vec!["INTERNAL_ONLY".into()],
        operational_classes_owned: vec!["AUDIT".into()],
        api_stability: "preview".into(),
        security_review: "unreviewed".into(),
        supply_chain: "source-only".into(),
    }
}
