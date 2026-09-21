#[test]
fn classified_metadata_splits_operational_and_subject_markers_from_privacy_classes() {
    let privacy = Classified::new("tenant-id", DataClass::InternalOnly);
    assert_eq!(
        privacy.data_class,
        DataClassification::from(DataClass::InternalOnly)
    );
    assert_eq!(
        privacy.data_class.privacy_data_class(),
        PrivacyDataClass::try_from(DataClass::InternalOnly).ok()
    );

    let audit = Classified::new("audit-hash", OperationalDataClass::Audit);
    assert_eq!(
        audit.data_class,
        DataClassification::Operational(OperationalDataClass::Audit)
    );
    assert_eq!(audit.data_class.label(), "AUDIT");
    assert_eq!(audit.data_class.privacy_data_class(), None);
    assert_eq!(
        audit.data_class.compatibility_data_class(),
        DataClass::Audit
    );

    let secret = Classified::new("secret-ref", OperationalDataClass::Secret);
    assert_eq!(
        secret.data_class,
        DataClassification::Operational(OperationalDataClass::Secret)
    );
    assert_eq!(secret.data_class.label(), "SECRET");
    assert_eq!(
        secret.data_class.compatibility_data_class(),
        DataClass::Secret
    );

    let child_subject_marker = Classified::new("minor", SubjectDataMarker::Children);
    assert_eq!(
        child_subject_marker.data_class,
        DataClassification::SubjectMarker(SubjectDataMarker::Children)
    );
    assert_eq!(child_subject_marker.data_class.label(), "CHILDREN");
    assert_eq!(
        child_subject_marker.data_class.compatibility_data_class(),
        DataClass::Children
    );
    assert_eq!(
        DataClassification::from(DataClass::Audit),
        DataClassification::from(OperationalDataClass::Audit)
    );
    assert_eq!(
        DataClassification::from(DataClass::Secret),
        DataClassification::from(OperationalDataClass::Secret)
    );
    assert_eq!(
        DataClassification::from(DataClass::Children),
        DataClassification::from(SubjectDataMarker::Children)
    );
    assert_eq!(
        DataClassification::from(DataClass::PiiIdentifying).normalized(),
        DataClassification::from(DataClass::PiiIdentifying)
    );
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
