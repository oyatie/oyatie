#[test]
fn privacy_data_class_newtype_rejects_operational_and_subject_markers() {
    for data_class in [
        DataClass::Public,
        DataClass::InternalOnly,
        DataClass::PiiIdentifying,
        DataClass::PiiQuasiIdentifier,
        DataClass::FinancialRegulatedCredit,
        DataClass::BehavioralAds,
        DataClass::SensitivePipaArticle23,
    ] {
        let privacy_class = PrivacyDataClass::try_from(data_class)
            .expect("privacy-program data classes should construct");
        assert_eq!(privacy_class.data_class(), data_class);
        assert_eq!(privacy_class.label(), data_class.label());
    }

    for data_class in [DataClass::Audit, DataClass::Secret, DataClass::Children] {
        assert_eq!(
            PrivacyDataClass::try_from(data_class),
            Err(NonPrivacyDataClass { data_class })
        );
    }
    assert_eq!(
        privacy_data_classes_from(&[DataClass::InternalOnly, DataClass::Audit]),
        Err(NonPrivacyDataClass {
            data_class: DataClass::Audit
        })
    );
    let privacy_classes =
        privacy_data_classes_from(&[DataClass::Public, DataClass::Phi, DataClass::BehavioralAds])
            .expect("privacy classes construct");
    assert_eq!(
        data_classes_from_privacy_data_classes(&privacy_classes),
        vec![DataClass::Public, DataClass::Phi, DataClass::BehavioralAds]
    );
    assert_eq!(
        most_restrictive_privacy_data_class(&privacy_classes),
        Some(DataClass::BehavioralAds)
    );
}

#[test]
fn privacy_refinement_rejects_operational_and_subject_markers() {
    for data_class in [
        DataClass::Public,
        DataClass::InternalOnly,
        DataClass::PiiIdentifying,
        DataClass::PiiQuasiIdentifier,
        DataClass::FinancialRegulatedCredit,
        DataClass::BehavioralAds,
        DataClass::SensitivePipaArticle23,
    ] {
        let privacy_class = PrivacyDataClass::try_from(data_class)
            .expect("privacy-program data classes should construct");
        assert_eq!(privacy_class.data_class(), data_class);
        assert_eq!(privacy_class.label(), data_class.label());
    }

    for data_class in [DataClass::Audit, DataClass::Secret, DataClass::Children] {
        assert_eq!(
            PrivacyDataClass::try_from(data_class),
            Err(NonPrivacyDataClass { data_class })
        );
    }
}

#[test]
fn privacy_collection_conversions_preserve_order_and_restriction() {
    assert_eq!(
        privacy_data_classes_from(&[DataClass::InternalOnly, DataClass::Audit]),
        Err(NonPrivacyDataClass {
            data_class: DataClass::Audit
        })
    );
    let privacy_classes =
        privacy_data_classes_from(&[DataClass::Public, DataClass::Phi, DataClass::BehavioralAds])
            .expect("privacy classes construct");
    assert_eq!(
        data_classes_from_privacy_data_classes(&privacy_classes),
        vec![DataClass::Public, DataClass::Phi, DataClass::BehavioralAds]
    );
    assert_eq!(
        most_restrictive_privacy_data_class(&privacy_classes),
        Some(DataClass::BehavioralAds)
    );
}

#[test]
fn privacy_conversion_error_carries_the_rejected_class_and_orders_like_data_class() {
    let error = PrivacyDataClass::new(DataClass::Secret).unwrap_err();
    assert_eq!(
        error,
        NonPrivacyDataClass {
            data_class: DataClass::Secret
        }
    );
    assert_eq!(error.data_class, DataClass::Secret);
    assert!(
        NonPrivacyDataClass {
            data_class: DataClass::Audit
        } > NonPrivacyDataClass {
            data_class: DataClass::Secret
        }
    );
    assert_eq!(
        PrivacyDataClass::internal_only(),
        PrivacyDataClass::new(DataClass::InternalOnly).unwrap()
    );
    assert_eq!(
        PrivacyDataClass::pii_identifying().data_class(),
        DataClass::PiiIdentifying
    );
    assert_eq!(
        PrivacyDataClass::pii_quasi_identifier().label(),
        "PII_QUASI_IDENTIFIER"
    );
    assert_eq!(
        DataClass::FinancialCredit,
        DataClass::FinancialRegulatedCredit
    );
}
