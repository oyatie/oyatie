fn privacy(data_class: DataClass) -> PrivacyDataClass {
    PrivacyDataClass::try_from(data_class).expect("test fixture must use privacy data class")
}

#[test]
fn purpose_pascal_labels_round_trip_for_file_ledger_compatibility() {
    for purpose in [
        Purpose::CoreService,
        Purpose::CapabilityInvocation,
        Purpose::SearchIndex,
        Purpose::AdsTargeting,
        Purpose::Analytics,
        Purpose::Support,
        Purpose::TenantAnalyticsFirstParty,
        Purpose::CrossTenantAggregateAnonymous,
        Purpose::PersonalizationInProduct,
        Purpose::SearchIndexPrivate,
        Purpose::SearchIndexPublic,
        Purpose::AdTargetingDeclared,
        Purpose::AdTargetingBehavioral,
        Purpose::ModelTrainingOya,
        Purpose::ModelTrainingThirdParty,
    ] {
        assert_eq!(
            super::parse_purpose_pascal_label(purpose.pascal_label()),
            Some(purpose)
        );
    }
}

#[test]
fn typed_consent_scope_grants_only_privacy_classifications() {
    let scope =
        super::ConsentScope::default().allow(Purpose::Analytics, privacy(DataClass::InternalOnly));

    assert!(
        scope
            .clone()
            .try_allow_legacy_data_class(Purpose::Analytics, DataClass::Audit)
            .is_err()
    );

    assert!(scope.allows_classification(
        Purpose::Analytics,
        DataClassification::from(DataClass::InternalOnly)
    ));
    assert!(!scope.allows_classification(
        Purpose::Analytics,
        DataClassification::from(OperationalDataClass::Audit)
    ));
    assert!(!scope.allows_classification(
        Purpose::Analytics,
        DataClassification::from(SubjectDataMarker::Children)
    ));
}

#[test]
fn legacy_consent_scope_allow_fails_closed_for_non_privacy_markers() {
    let scope = super::ConsentScope::default()
        .try_allow_legacy_data_class(Purpose::CapabilityInvocation, DataClass::InternalOnly)
        .expect("internal-only is a privacy data class");

    assert!(scope.allows_legacy_data_class(Purpose::CapabilityInvocation, DataClass::InternalOnly));
    for data_class in [DataClass::Audit, DataClass::Secret, DataClass::Children] {
        assert!(
            scope
                .clone()
                .try_allow_legacy_data_class(Purpose::CapabilityInvocation, data_class)
                .is_err()
        );
        assert!(!scope.allows_legacy_data_class(Purpose::CapabilityInvocation, data_class));
    }
}
