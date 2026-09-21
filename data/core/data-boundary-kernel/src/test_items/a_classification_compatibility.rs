/// Compiles only when both arguments share one Rust type.
fn same_type<T>(_: T, _: T) {}

#[test]
fn classification_values_are_the_same_type_through_both_namespaces() {
    same_type(super::DataClass::Phi, data_classification::DataClass::Phi);
    same_type(
        super::OperationalDataClass::Audit,
        data_classification::OperationalDataClass::Audit,
    );
    same_type(
        super::SubjectDataMarker::Children,
        data_classification::SubjectDataMarker::Children,
    );
    same_type(
        super::DataClassification::from(super::DataClass::Pci),
        data_classification::DataClassification::from(data_classification::DataClass::Pci),
    );
    same_type(
        super::PrivacyDataClass::internal_only(),
        data_classification::PrivacyDataClass::internal_only(),
    );
    same_type(
        super::PrivacyDataClass::new(super::DataClass::Audit).unwrap_err(),
        data_classification::NonPrivacyDataClass {
            data_class: data_classification::DataClass::Audit,
        },
    );
    same_type(
        super::Classified::new("value", super::DataClass::Public),
        data_classification::Classified::new("value", data_classification::DataClass::Public),
    );
    same_type(
        super::PRIVACY_PROGRAM_DATA_CLASS_LABELS,
        data_classification::PRIVACY_PROGRAM_DATA_CLASS_LABELS,
    );
}

#[test]
fn classification_functions_are_the_same_items_through_both_namespaces() {
    same_type(
        super::parse_data_class_label,
        data_classification::parse_data_class_label,
    );
    same_type(
        super::parse_data_class_pascal_label,
        data_classification::parse_data_class_pascal_label,
    );
    same_type(
        super::parse_operational_data_class_label,
        data_classification::parse_operational_data_class_label,
    );
    same_type(
        super::parse_subject_data_marker_label,
        data_classification::parse_subject_data_marker_label,
    );
    same_type(
        super::privacy_data_classes_from,
        data_classification::privacy_data_classes_from,
    );
    same_type(
        super::data_classes_from_privacy_data_classes,
        data_classification::data_classes_from_privacy_data_classes,
    );
    same_type(
        super::most_restrictive_privacy_data_class,
        data_classification::most_restrictive_privacy_data_class,
    );
}

#[test]
fn a_port_value_is_accepted_by_the_legacy_policy_evaluator_unchanged() {
    let attributes = DataUseAttributes {
        purpose: Purpose::AdsTargeting,
        data_classification: data_classification::DataClassification::from(
            data_classification::DataClass::Phi,
        ),
        subject_class: SubjectClass::Adult,
    };
    assert_eq!(
        evaluate_data_use(attributes),
        Err(DataUseDenialReason::HardDeniedDataClass)
    );
    assert_eq!(
        ClassificationLevel::from_data_class(data_classification::DataClass::Phi),
        ClassificationLevel::Critical
    );
    assert!(DataClassMatcher::HardDenySet.matches(data_classification::DataClass::Phi));
}
