pub fn is_hard_denied(purpose: Purpose, data_class: DataClass) -> bool {
    is_hard_denied_classification(purpose, data_class)
}

pub fn is_hard_denied_classification(
    purpose: Purpose,
    classification: impl Into<DataClassification>,
) -> bool {
    match normalize_classification(classification.into()) {
        DataClassification::Privacy(data_class) => {
            is_hard_denied_privacy_class(purpose, data_class.data_class())
        }
        DataClassification::Operational(OperationalDataClass::Audit) => {
            is_audit_marker_hard_denied(purpose)
        }
        DataClassification::Operational(OperationalDataClass::Secret) => {
            is_secret_marker_hard_denied(purpose)
        }
        DataClassification::SubjectMarker(SubjectDataMarker::Children) => {
            is_child_subject_marker_hard_denied(purpose)
        }
    }
}

fn normalize_classification(classification: DataClassification) -> DataClassification {
    classification
}

fn is_hard_denied_privacy_class(purpose: Purpose, data_class: DataClass) -> bool {
    match purpose {
        Purpose::SearchIndex | Purpose::SearchIndexPrivate => {
            is_search_index_privacy_hard_denied(data_class)
        }
        Purpose::SearchIndexPublic => !matches!(data_class, DataClass::Public),
        Purpose::AdsTargeting => is_ads_targeting_privacy_hard_denied(data_class),
        Purpose::AdTargetingDeclared => !matches!(
            data_class,
            DataClass::DeclaredPreference | DataClass::Public
        ),
        Purpose::AdTargetingBehavioral => !matches!(
            data_class,
            DataClass::BehavioralAds | DataClass::DeclaredPreference | DataClass::Public
        ),
        Purpose::Analytics | Purpose::TenantAnalyticsFirstParty => {
            matches!(data_class, DataClass::Pci)
        }
        Purpose::CrossTenantAggregateAnonymous => {
            matches!(data_class, DataClass::Pci | DataClass::SearchQuery)
        }
        Purpose::PersonalizationInProduct => is_regulated_privacy_class(data_class),
        Purpose::ModelTrainingOya => is_model_training_privacy_hard_denied(data_class),
        Purpose::ModelTrainingThirdParty => is_model_training_privacy_hard_denied(data_class),
        Purpose::CoreService | Purpose::CapabilityInvocation | Purpose::Support => false,
    }
}

fn is_search_index_privacy_hard_denied(data_class: DataClass) -> bool {
    matches!(
        data_class,
        DataClass::Phi
            | DataClass::Pci
            | DataClass::PipaArticle23
            | DataClass::SensitivePipaArticle23
            | DataClass::Financial
            | DataClass::FinancialRegulatedCredit
    )
}

fn is_ads_targeting_privacy_hard_denied(data_class: DataClass) -> bool {
    matches!(
        data_class,
        DataClass::InternalOnly
            | DataClass::PiiIdentifying
            | DataClass::PiiSensitive
            | DataClass::PiiQuasiIdentifier
            | DataClass::Phi
            | DataClass::Pci
            | DataClass::PipaArticle23
            | DataClass::SensitivePipaArticle23
            | DataClass::Financial
            | DataClass::FinancialRegulatedCredit
            | DataClass::Usage
            | DataClass::BehavioralTenantProduct
            | DataClass::SearchQuery
    )
}

fn is_regulated_privacy_class(data_class: DataClass) -> bool {
    matches!(
        data_class,
        DataClass::Phi
            | DataClass::Pci
            | DataClass::PipaArticle23
            | DataClass::SensitivePipaArticle23
            | DataClass::Financial
            | DataClass::FinancialRegulatedCredit
    )
}

fn is_model_training_privacy_hard_denied(data_class: DataClass) -> bool {
    matches!(
        data_class,
        DataClass::PiiIdentifying
            | DataClass::PiiSensitive
            | DataClass::PiiQuasiIdentifier
            | DataClass::Phi
            | DataClass::Pci
            | DataClass::PipaArticle23
            | DataClass::SensitivePipaArticle23
            | DataClass::Financial
            | DataClass::FinancialRegulatedCredit
            | DataClass::SearchQuery
    )
}

fn is_audit_marker_hard_denied(purpose: Purpose) -> bool {
    matches!(
        purpose,
        Purpose::SearchIndexPublic
            | Purpose::AdsTargeting
            | Purpose::AdTargetingDeclared
            | Purpose::AdTargetingBehavioral
            | Purpose::ModelTrainingOya
            | Purpose::ModelTrainingThirdParty
    )
}

fn is_secret_marker_hard_denied(purpose: Purpose) -> bool {
    matches!(
        purpose,
        Purpose::SearchIndex
            | Purpose::SearchIndexPrivate
            | Purpose::SearchIndexPublic
            | Purpose::AdsTargeting
            | Purpose::AdTargetingDeclared
            | Purpose::AdTargetingBehavioral
            | Purpose::Analytics
            | Purpose::TenantAnalyticsFirstParty
            | Purpose::CrossTenantAggregateAnonymous
            | Purpose::PersonalizationInProduct
            | Purpose::ModelTrainingOya
            | Purpose::ModelTrainingThirdParty
    )
}

fn is_child_subject_marker_hard_denied(purpose: Purpose) -> bool {
    matches!(
        purpose,
        Purpose::SearchIndex
            | Purpose::SearchIndexPrivate
            | Purpose::SearchIndexPublic
            | Purpose::AdsTargeting
            | Purpose::AdTargetingDeclared
            | Purpose::AdTargetingBehavioral
            | Purpose::PersonalizationInProduct
            | Purpose::ModelTrainingOya
            | Purpose::ModelTrainingThirdParty
    )
}

pub fn evaluate_data_use(attributes: DataUseAttributes) -> Result<(), DataUseDenialReason> {
    if is_hard_denied_classification(attributes.purpose, attributes.data_classification) {
        return Err(DataUseDenialReason::HardDeniedDataClass);
    }
    if is_subject_hard_denied(attributes.purpose, attributes.subject_class) {
        return Err(DataUseDenialReason::MinorSubjectAds);
    }
    Ok(())
}

pub fn evaluate_data_use_classification(
    attributes: DataUseClassificationAttributes,
) -> Result<(), DataUseDenialReason> {
    evaluate_data_use(attributes)
}

pub fn evaluate_legacy_data_use(
    attributes: LegacyDataUseAttributes,
) -> Result<(), DataUseDenialReason> {
    evaluate_data_use(DataUseAttributes {
        purpose: attributes.purpose,
        data_classification: attributes.data_class.into(),
        subject_class: attributes.subject_class,
    })
}

pub fn is_subject_hard_denied(purpose: Purpose, subject_class: SubjectClass) -> bool {
    matches!(purpose, Purpose::AdsTargeting) && matches!(subject_class, SubjectClass::Minor { .. })
}
