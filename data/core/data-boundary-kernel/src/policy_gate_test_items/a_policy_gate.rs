fn privacy(data_class: DataClass) -> PrivacyDataClass {
    PrivacyDataClass::try_from(data_class).expect("test fixture uses privacy class")
}

fn tenant_allows(purpose: Purpose, data_class: DataClass) -> TenantDataUsePolicy {
    TenantDataUsePolicy::default().allow(purpose, privacy(data_class))
}

#[test]
fn override_pack_deny_short_circuits_tenant_allow_before_tenant_policy() {
    let override_pack = MicroserviceOverridePack::new("payroll").deny(
        OverrideDenyRule::new(DataClassification::from(DataClass::PiiIdentifying))
            .for_purpose(Purpose::CapabilityInvocation)
            .with_scope(HardDenyScope::All),
    );
    let tenant_policy = tenant_allows(Purpose::CapabilityInvocation, DataClass::PiiIdentifying);

    let decision = evaluate_data_use_gate(DataUseGateRequest {
        attributes: crate::DataUseAttributes {
            purpose: Purpose::CapabilityInvocation,
            data_classification: DataClassification::from(DataClass::PiiIdentifying),
            subject_class: SubjectClass::Adult,
        },
        override_pack: Some(&override_pack),
        tenant_policy: &tenant_policy,
        derived_lineage: None,
        eu_ai_risk: None,
    });

    assert_eq!(
        decision,
        Err(DataUseGateDenialReason::OverridePackDenied {
            microservice_id: "payroll",
            scope: HardDenyScope::All,
        })
    );
}

#[test]
fn override_pack_deny_normalizes_legacy_privacy_aliases() {
    for (denied_class, request_class) in [
        (DataClass::PiiQuasiIdentifier, DataClass::PiiSensitive),
        (DataClass::BehavioralTenantProduct, DataClass::Usage),
        (DataClass::SensitivePipaArticle23, DataClass::PipaArticle23),
    ] {
        let override_pack = MicroserviceOverridePack::new("privacy-gate").deny(
            OverrideDenyRule::new(DataClassification::from(denied_class))
                .for_purpose(Purpose::CapabilityInvocation),
        );
        let tenant_policy = tenant_allows(Purpose::CapabilityInvocation, request_class);

        let decision = evaluate_data_use_gate(DataUseGateRequest {
            attributes: crate::DataUseAttributes {
                purpose: Purpose::CapabilityInvocation,
                data_classification: DataClassification::from(request_class),
                subject_class: SubjectClass::Adult,
            },
            override_pack: Some(&override_pack),
            tenant_policy: &tenant_policy,
            derived_lineage: None,
            eu_ai_risk: None,
        });

        assert_eq!(
            decision,
            Err(DataUseGateDenialReason::OverridePackDenied {
                microservice_id: "privacy-gate",
                scope: HardDenyScope::All,
            })
        );
    }
}

#[test]
fn missing_override_pack_fails_closed_before_tenant_allow() {
    let tenant_policy = tenant_allows(Purpose::CapabilityInvocation, DataClass::Public);

    let decision = evaluate_data_use_gate(DataUseGateRequest {
        attributes: crate::DataUseAttributes {
            purpose: Purpose::CapabilityInvocation,
            data_classification: DataClassification::from(DataClass::Public),
            subject_class: SubjectClass::Adult,
        },
        override_pack: None,
        tenant_policy: &tenant_policy,
        derived_lineage: None,
        eu_ai_risk: None,
    });

    assert_eq!(decision, Err(DataUseGateDenialReason::OverridePackMissing));
}

#[test]
fn eu_ai_high_risk_registry_entry_denies_model_training_even_for_public_data() {
    let override_pack = MicroserviceOverridePack::new("intelligence");
    let tenant_policy = tenant_allows(Purpose::ModelTrainingOya, DataClass::Public);
    let risk = EuAiRiskRegistryEntry::new("auto-employment-decisioning", EuAiRiskTier::HighRisk);

    let decision = evaluate_data_use_gate(DataUseGateRequest {
        attributes: crate::DataUseAttributes {
            purpose: Purpose::ModelTrainingOya,
            data_classification: DataClassification::from(DataClass::Public),
            subject_class: SubjectClass::Adult,
        },
        override_pack: Some(&override_pack),
        tenant_policy: &tenant_policy,
        derived_lineage: None,
        eu_ai_risk: Some(&risk),
    });

    assert_eq!(
        decision,
        Err(DataUseGateDenialReason::EuAiRiskTierDenied {
            archetype: "auto-employment-decisioning",
            tier: EuAiRiskTier::HighRisk,
        })
    );
}

#[test]
fn dub_matrix_fixture_covers_hard_deny_operational_and_subject_rows() {
    let matrix = DataUseBoundaryMatrix::default();

    assert!(matrix.is_hard_denied(
        Purpose::AdsTargeting,
        DataClassification::from(DataClass::Phi)
    ));
    assert!(matrix.is_hard_denied(
        Purpose::Analytics,
        DataClassification::from(OperationalDataClass::Secret)
    ));
    assert!(matrix.is_hard_denied(
        Purpose::SearchIndexPrivate,
        DataClassification::from(DataClass::Children)
    ));
    assert!(!matrix.is_hard_denied(
        Purpose::CapabilityInvocation,
        DataClassification::from(DataClass::Public)
    ));
}
