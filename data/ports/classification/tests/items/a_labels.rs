#[test]
fn privacy_program_data_class_labels_are_parseable() {
    for label in PRIVACY_PROGRAM_DATA_CLASS_LABELS {
        assert!(
            parse_data_class_label(label).is_some(),
            "privacy-program label must parse: {label}"
        );
    }
    assert_eq!(DataClass::Financial.label(), "FINANCIAL");
    assert_eq!(
        DataClass::Financial.privacy_program_label(),
        Some("FINANCIAL")
    );
    assert_eq!(
        PrivacyDataClass::new(DataClass::Financial)
            .expect("financial compatibility class is a privacy class")
            .label(),
        "FINANCIAL"
    );
    assert_eq!(DataClass::Audit.privacy_program_label(), None);
    for operational_or_subject_label in ["AUDIT", "SECRET", "CHILDREN"] {
        assert_eq!(
            parse_data_class_label(operational_or_subject_label),
            None,
            "non-privacy label must not parse as a privacy data class"
        );
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
fn data_class_pascal_labels_round_trip_for_file_ledger_compatibility() {
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
fn privacy_parser_trims_whitespace_and_rejects_unknown_and_non_privacy_labels() {
    assert_eq!(parse_data_class_label("  PHI\n"), Some(DataClass::Phi));
    assert_eq!(
        parse_data_class_pascal_label("\tPiiIdentifying "),
        Some(DataClass::PiiIdentifying)
    );
    assert_eq!(
        parse_operational_data_class_label(" SECRET "),
        Some(OperationalDataClass::Secret)
    );
    assert_eq!(
        parse_subject_data_marker_label(" CHILDREN "),
        Some(SubjectDataMarker::Children)
    );
    for label in ["", " ", "phi", "PHI_", "UNKNOWN", "Phi", "PII IDENTIFYING"] {
        assert_eq!(parse_data_class_label(label), None, "{label:?}");
        assert_eq!(parse_operational_data_class_label(label), None, "{label:?}");
        assert_eq!(parse_subject_data_marker_label(label), None, "{label:?}");
    }
    for label in ["AUDIT", "SECRET", "CHILDREN", "Audit", "Secret", "Children"] {
        assert_eq!(parse_data_class_label(label), None, "{label:?}");
    }
    assert_eq!(parse_data_class_pascal_label("PHI"), None);
    assert_eq!(
        parse_data_class_pascal_label("Audit"),
        Some(DataClass::Audit)
    );
}
