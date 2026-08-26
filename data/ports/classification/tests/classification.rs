use data_classification::{
    Classified, DataClass, DataClassification, NonPrivacyDataClass, OperationalDataClass,
    PRIVACY_PROGRAM_DATA_CLASS_LABELS, PrivacyDataClass, SubjectDataMarker,
    data_classes_from_privacy_data_classes, most_restrictive_privacy_data_class,
    parse_data_class_label, parse_data_class_pascal_label, parse_operational_data_class_label,
    parse_subject_data_marker_label, privacy_data_classes_from,
};

#[test]
fn privacy_program_labels_are_parseable_without_operational_leakage() {
    for label in PRIVACY_PROGRAM_DATA_CLASS_LABELS {
        assert_eq!(
            parse_data_class_label(label).map(DataClass::label),
            Some(label)
        );
    }
    for label in ["AUDIT", "SECRET", "CHILDREN"] {
        assert_eq!(parse_data_class_label(label), None);
    }
    assert_eq!(
        parse_operational_data_class_label("AUDIT"),
        Some(OperationalDataClass::Audit)
    );
    assert_eq!(
        parse_subject_data_marker_label("CHILDREN"),
        Some(SubjectDataMarker::Children)
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
fn pascal_labels_round_trip_for_file_ledger_compatibility() {
    for data_class in [
        DataClass::Public,
        DataClass::InternalOnly,
        DataClass::PiiIdentifying,
        DataClass::PiiSensitive,
        DataClass::Phi,
        DataClass::Pci,
        DataClass::PipaArticle23,
        DataClass::Children,
        DataClass::Financial,
        DataClass::Usage,
        DataClass::Secret,
        DataClass::Audit,
        DataClass::PiiQuasiIdentifier,
        DataClass::FinancialRegulatedCredit,
        DataClass::BehavioralTenantProduct,
        DataClass::BehavioralAds,
        DataClass::DeclaredPreference,
        DataClass::SearchQuery,
        DataClass::SensitivePipaArticle23,
    ] {
        assert_eq!(
            parse_data_class_pascal_label(data_class.pascal_label()),
            Some(data_class)
        );
    }
}

#[test]
fn classified_values_keep_privacy_operational_and_subject_axes_distinct() {
    let privacy = Classified::new("tenant-id", DataClass::InternalOnly);
    assert_eq!(
        privacy.data_class.privacy_data_class(),
        PrivacyDataClass::try_from(DataClass::InternalOnly).ok()
    );

    let audit = Classified::new("audit-hash", OperationalDataClass::Audit);
    assert_eq!(audit.data_class.label(), "AUDIT");
    assert_eq!(audit.data_class.privacy_data_class(), None);
    assert_eq!(
        audit.data_class.compatibility_data_class(),
        DataClass::Audit
    );

    let child = Classified::new("minor", SubjectDataMarker::Children);
    assert_eq!(child.data_class.label(), "CHILDREN");
    assert_eq!(
        child.data_class,
        DataClassification::from(DataClass::Children)
    );
}
