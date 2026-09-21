#[test]
fn ad_targeting_purpose_blocks_non_ad_safe_classes_even_with_grants() {
    let scope = super::ConsentScope::default()
        .allow(Purpose::AdsTargeting, privacy(DataClass::PiiIdentifying))
        .allow(Purpose::AdsTargeting, privacy(DataClass::SearchQuery))
        .allow(Purpose::AdsTargeting, privacy(DataClass::InternalOnly));

    for data_class in [
        DataClass::PiiIdentifying,
        DataClass::SearchQuery,
        DataClass::InternalOnly,
    ] {
        assert!(!scope.allows(Purpose::AdsTargeting, privacy(data_class)));
        assert_eq!(
            evaluate_legacy_data_use(LegacyDataUseAttributes {
                purpose: Purpose::AdsTargeting,
                data_class,
                subject_class: SubjectClass::Adult,
            }),
            Err(DataUseDenialReason::HardDeniedDataClass)
        );
    }
}

#[test]
fn declared_preference_is_the_only_declared_ad_targeting_class() {
    let scope = super::ConsentScope::default().allow(
        Purpose::AdTargetingDeclared,
        privacy(DataClass::DeclaredPreference),
    );

    assert!(scope.allows(
        Purpose::AdTargetingDeclared,
        privacy(DataClass::DeclaredPreference)
    ));
    assert_eq!(
        evaluate_legacy_data_use(LegacyDataUseAttributes {
            purpose: Purpose::AdTargetingDeclared,
            data_class: DataClass::PiiIdentifying,
            subject_class: SubjectClass::Adult,
        }),
        Err(DataUseDenialReason::HardDeniedDataClass)
    );
}

#[test]
fn public_search_index_accepts_only_public_class() {
    assert_eq!(
        evaluate_legacy_data_use(LegacyDataUseAttributes {
            purpose: Purpose::SearchIndexPublic,
            data_class: DataClass::Public,
            subject_class: SubjectClass::Adult,
        }),
        Ok(())
    );
    assert_eq!(
        evaluate_legacy_data_use(LegacyDataUseAttributes {
            purpose: Purpose::SearchIndexPublic,
            data_class: DataClass::PiiIdentifying,
            subject_class: SubjectClass::Adult,
        }),
        Err(DataUseDenialReason::HardDeniedDataClass)
    );
}

#[test]
fn model_training_purposes_block_direct_and_regulated_classes_even_with_grants() {
    let denied_classes = [
        DataClass::PiiIdentifying,
        DataClass::PiiQuasiIdentifier,
        DataClass::Phi,
        DataClass::Pci,
        DataClass::SensitivePipaArticle23,
        DataClass::FinancialRegulatedCredit,
        DataClass::SearchQuery,
    ];
    let mut scope = super::ConsentScope::default();
    for purpose in [Purpose::ModelTrainingOya, Purpose::ModelTrainingThirdParty] {
        for data_class in denied_classes {
            scope = scope.allow(purpose, privacy(data_class));
            assert!(!scope.allows(purpose, privacy(data_class)));
            assert_eq!(
                evaluate_legacy_data_use(LegacyDataUseAttributes {
                    purpose,
                    data_class,
                    subject_class: SubjectClass::Adult,
                }),
                Err(DataUseDenialReason::HardDeniedDataClass)
            );
        }
        assert!(
            scope
                .clone()
                .try_allow_legacy_data_class(purpose, DataClass::Audit)
                .is_err()
        );
        assert!(!scope.allows_legacy_data_class(purpose, DataClass::Audit));
    }

    let allowed_scope = super::ConsentScope::default()
        .allow(Purpose::ModelTrainingOya, privacy(DataClass::Public))
        .allow(Purpose::ModelTrainingThirdParty, privacy(DataClass::Public));
    assert!(allowed_scope.allows(Purpose::ModelTrainingOya, privacy(DataClass::Public)));
    assert!(allowed_scope.allows(Purpose::ModelTrainingThirdParty, privacy(DataClass::Public)));
}
