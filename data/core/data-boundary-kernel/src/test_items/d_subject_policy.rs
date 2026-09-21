#[test]
fn typed_classification_policy_preserves_legacy_marker_denials() {
    let cases = [
        (
            Purpose::Analytics,
            DataClass::Secret,
            DataClassification::from(OperationalDataClass::Secret),
        ),
        (
            Purpose::ModelTrainingOya,
            DataClass::Audit,
            DataClassification::from(OperationalDataClass::Audit),
        ),
        (
            Purpose::SearchIndex,
            DataClass::Children,
            DataClassification::from(SubjectDataMarker::Children),
        ),
        (
            Purpose::CapabilityInvocation,
            DataClass::Audit,
            DataClassification::from(OperationalDataClass::Audit),
        ),
    ];

    for (purpose, legacy_class, classification) in cases {
        assert_eq!(
            super::is_hard_denied(purpose, legacy_class),
            super::is_hard_denied_classification(purpose, classification)
        );
    }

    assert_eq!(
        evaluate_data_use(DataUseAttributes {
            purpose: Purpose::Analytics,
            data_classification: DataClassification::from(OperationalDataClass::Secret),
            subject_class: SubjectClass::Adult,
        }),
        Err(DataUseDenialReason::HardDeniedDataClass)
    );
    assert_eq!(
        evaluate_data_use_classification(DataUseClassificationAttributes {
            purpose: Purpose::ModelTrainingThirdParty,
            data_classification: DataClassification::from(OperationalDataClass::Audit),
            subject_class: SubjectClass::Adult,
        }),
        Err(DataUseDenialReason::HardDeniedDataClass)
    );
    assert_eq!(
        evaluate_data_use_classification(DataUseClassificationAttributes {
            purpose: Purpose::SearchIndexPrivate,
            data_classification: DataClassification::from(SubjectDataMarker::Children),
            subject_class: SubjectClass::Adult,
        }),
        Err(DataUseDenialReason::HardDeniedDataClass)
    );
    assert_eq!(
        evaluate_data_use_classification(DataUseClassificationAttributes {
            purpose: Purpose::Analytics,
            data_classification: DataClassification::from(OperationalDataClass::Audit),
            subject_class: SubjectClass::Adult,
        }),
        Ok(())
    );
}

#[test]
fn data_use_evaluator_composes_data_class_and_subject_class_denies() {
    assert_eq!(
        evaluate_legacy_data_use(LegacyDataUseAttributes {
            purpose: Purpose::AdsTargeting,
            data_class: DataClass::Public,
            subject_class: SubjectClass::Minor {
                age_band: AgeBand::Under14
            },
        }),
        Err(DataUseDenialReason::MinorSubjectAds)
    );
    assert_eq!(
        evaluate_legacy_data_use(LegacyDataUseAttributes {
            purpose: Purpose::AdsTargeting,
            data_class: DataClass::Phi,
            subject_class: SubjectClass::Adult,
        }),
        Err(DataUseDenialReason::HardDeniedDataClass)
    );
    assert_eq!(
        evaluate_legacy_data_use(LegacyDataUseAttributes {
            purpose: Purpose::AdsTargeting,
            data_class: DataClass::Public,
            subject_class: SubjectClass::Adult,
        }),
        Ok(())
    );
}
