#[test]
fn classification_level_ordering_is_unrestricted_lt_critical() {
    assert!(ClassificationLevel::Unrestricted < ClassificationLevel::Restricted);
    assert!(ClassificationLevel::Restricted < ClassificationLevel::Sensitive);
    assert!(ClassificationLevel::Sensitive < ClassificationLevel::Critical);
}

#[test]
fn hard_deny_data_classes_map_to_critical_level() {
    for data_class in [
        DataClass::Phi,
        DataClass::Pci,
        DataClass::PipaArticle23,
        DataClass::SensitivePipaArticle23,
        DataClass::Children,
    ] {
        assert_eq!(
            ClassificationLevel::from_data_class(data_class),
            ClassificationLevel::Critical,
            "{data_class:?} must map to Critical"
        );
        assert!(ClassificationLevel::from_data_class(data_class).is_hard_deny_tier());
    }
}

#[test]
fn public_maps_to_unrestricted_level() {
    assert_eq!(
        ClassificationLevel::from_data_class(DataClass::Public),
        ClassificationLevel::Unrestricted
    );
    assert!(!ClassificationLevel::Unrestricted.is_hard_deny_tier());
}

#[test]
fn classification_level_labels_are_stable() {
    assert_eq!(ClassificationLevel::Unrestricted.label(), "UNRESTRICTED");
    assert_eq!(ClassificationLevel::Restricted.label(), "RESTRICTED");
    assert_eq!(ClassificationLevel::Sensitive.label(), "SENSITIVE");
    assert_eq!(ClassificationLevel::Critical.label(), "CRITICAL");
}

#[test]
fn data_class_matcher_hard_deny_set_covers_exactly_phi_pci_pipa_children() {
    let hard_deny = [
        DataClass::Phi,
        DataClass::Pci,
        DataClass::PipaArticle23,
        DataClass::SensitivePipaArticle23,
        DataClass::Children,
    ];
    for dc in hard_deny {
        assert!(
            DataClassMatcher::HardDenySet.matches(dc),
            "{dc:?} must be in HardDenySet"
        );
    }
    for dc in [
        DataClass::Public,
        DataClass::InternalOnly,
        DataClass::PiiIdentifying,
        DataClass::Financial,
    ] {
        assert!(
            !DataClassMatcher::HardDenySet.matches(dc),
            "{dc:?} must not be in HardDenySet"
        );
    }
}

#[test]
fn data_class_matcher_regulated_financial_covers_financial_variants() {
    assert!(DataClassMatcher::RegulatedFinancial.matches(DataClass::Financial));
    assert!(DataClassMatcher::RegulatedFinancial.matches(DataClass::FinancialRegulatedCredit));
    assert!(!DataClassMatcher::RegulatedFinancial.matches(DataClass::Phi));
    assert!(!DataClassMatcher::RegulatedFinancial.matches(DataClass::Public));
}

#[test]
fn data_class_matcher_direct_pii_covers_identifying_variants() {
    for dc in [
        DataClass::PiiIdentifying,
        DataClass::PiiSensitive,
        DataClass::PiiQuasiIdentifier,
    ] {
        assert!(
            DataClassMatcher::DirectPii.matches(dc),
            "{dc:?} must be DirectPii"
        );
    }
    assert!(!DataClassMatcher::DirectPii.matches(DataClass::Phi));
    assert!(!DataClassMatcher::DirectPii.matches(DataClass::BehavioralAds));
}

#[test]
fn data_class_matcher_search_index_restricted_covers_phi_pci_pipa_financial() {
    for dc in [
        DataClass::Phi,
        DataClass::Pci,
        DataClass::PipaArticle23,
        DataClass::SensitivePipaArticle23,
        DataClass::Financial,
        DataClass::FinancialRegulatedCredit,
    ] {
        assert!(
            DataClassMatcher::SearchIndexRestricted.matches(dc),
            "{dc:?} must be SearchIndexRestricted"
        );
    }
    assert!(!DataClassMatcher::SearchIndexRestricted.matches(DataClass::Public));
    assert!(!DataClassMatcher::SearchIndexRestricted.matches(DataClass::PiiIdentifying));
}

#[test]
fn retention_policy_critical_class_uses_crypto_shred_30_days() {
    let policy = RetentionPolicy::from_data_class(DataClass::Phi);
    assert_eq!(policy.level, ClassificationLevel::Critical);
    assert_eq!(policy.purge_action, PurgeAction::CryptoShred);
    assert_eq!(policy.retention_window.as_secs(), 30 * 24 * 3600);
}

#[test]
fn retention_policy_sensitive_class_uses_secure_erase_90_days() {
    let policy = RetentionPolicy::from_data_class(DataClass::PiiIdentifying);
    assert_eq!(policy.level, ClassificationLevel::Sensitive);
    assert_eq!(policy.purge_action, PurgeAction::SecureErase);
    assert_eq!(policy.retention_window.as_secs(), 90 * 24 * 3600);
}

#[test]
fn retention_policy_restricted_class_uses_logical_delete_365_days() {
    let policy = RetentionPolicy::from_data_class(DataClass::InternalOnly);
    assert_eq!(policy.level, ClassificationLevel::Restricted);
    assert_eq!(policy.purge_action, PurgeAction::LogicalDelete);
    assert_eq!(policy.retention_window.as_secs(), 365 * 24 * 3600);
}

#[test]
fn retention_policy_unrestricted_class_uses_logical_delete_730_days() {
    let policy = RetentionPolicy::from_data_class(DataClass::Public);
    assert_eq!(policy.level, ClassificationLevel::Unrestricted);
    assert_eq!(policy.purge_action, PurgeAction::LogicalDelete);
    assert_eq!(policy.retention_window.as_secs(), 730 * 24 * 3600);
}

#[test]
fn retention_policy_from_level_is_consistent_with_from_data_class() {
    for data_class in [
        DataClass::Phi,
        DataClass::PiiIdentifying,
        DataClass::InternalOnly,
        DataClass::Public,
    ] {
        let level = ClassificationLevel::from_data_class(data_class);
        assert_eq!(
            RetentionPolicy::from_data_class(data_class),
            RetentionPolicy::from_level(level)
        );
    }
}
