#[test]
fn derived_feature_lineage_inherits_hard_deny_source_class() {
    let override_pack = MicroserviceOverridePack::new("analytics");
    let tenant_policy = tenant_allows(Purpose::AdTargetingDeclared, DataClass::DeclaredPreference);
    let lineage = DerivedFeatureLineage::from_sources([
        DataClassification::from(DataClass::Public),
        DataClassification::from(DataClass::Phi),
    ]);

    let decision = evaluate_data_use_gate(DataUseGateRequest {
        attributes: crate::DataUseAttributes {
            purpose: Purpose::AdTargetingDeclared,
            data_classification: DataClassification::from(DataClass::DeclaredPreference),
            subject_class: SubjectClass::Adult,
        },
        override_pack: Some(&override_pack),
        tenant_policy: &tenant_policy,
        derived_lineage: Some(&lineage),
        eu_ai_risk: None,
    });

    assert_eq!(
        decision,
        Err(DataUseGateDenialReason::DerivedLineageDenied {
            inherited_classification: DataClassification::from(DataClass::Phi),
        })
    );
}

#[test]
fn derived_feature_lineage_checks_every_source_before_effective_class() {
    let override_pack = MicroserviceOverridePack::new("analytics");
    let tenant_policy = tenant_allows(Purpose::Analytics, DataClass::PiiIdentifying);
    let lineage = DerivedFeatureLineage::from_sources([
        DataClassification::from(OperationalDataClass::Secret),
        DataClassification::from(DataClass::PiiIdentifying),
    ]);

    let decision = evaluate_data_use_gate(DataUseGateRequest {
        attributes: crate::DataUseAttributes {
            purpose: Purpose::Analytics,
            data_classification: DataClassification::from(DataClass::Public),
            subject_class: SubjectClass::Adult,
        },
        override_pack: Some(&override_pack),
        tenant_policy: &tenant_policy,
        derived_lineage: Some(&lineage),
        eu_ai_risk: None,
    });

    assert_eq!(
        decision,
        Err(DataUseGateDenialReason::DerivedLineageDenied {
            inherited_classification: DataClassification::from(OperationalDataClass::Secret),
        })
    );
}

#[test]
fn derived_feature_lineage_requires_tenant_grants_for_each_source_class() {
    let override_pack = MicroserviceOverridePack::new("analytics");
    let tenant_policy = tenant_allows(Purpose::CapabilityInvocation, DataClass::DeclaredPreference);
    let lineage = DerivedFeatureLineage::from_sources([
        DataClassification::from(DataClass::PiiIdentifying),
        DataClassification::from(DataClass::DeclaredPreference),
    ]);

    let decision = evaluate_data_use_gate(DataUseGateRequest {
        attributes: crate::DataUseAttributes {
            purpose: Purpose::CapabilityInvocation,
            data_classification: DataClassification::from(DataClass::DeclaredPreference),
            subject_class: SubjectClass::Adult,
        },
        override_pack: Some(&override_pack),
        tenant_policy: &tenant_policy,
        derived_lineage: Some(&lineage),
        eu_ai_risk: None,
    });

    assert_eq!(decision, Err(DataUseGateDenialReason::TenantPolicyDenied));
}

#[test]
fn override_pack_denies_matching_lineage_source_before_effective_class() {
    let override_pack = MicroserviceOverridePack::new("privacy-gate").deny(
        OverrideDenyRule::new(DataClassification::from(DataClass::PiiIdentifying))
            .for_purpose(Purpose::CapabilityInvocation)
            .with_scope(HardDenyScope::AnyMicroserviceExceptHome),
    );
    let tenant_policy = TenantDataUsePolicy::default()
        .allow(
            Purpose::CapabilityInvocation,
            privacy(DataClass::PiiIdentifying),
        )
        .allow(
            Purpose::CapabilityInvocation,
            privacy(DataClass::DeclaredPreference),
        );
    let lineage = DerivedFeatureLineage::from_sources([
        DataClassification::from(DataClass::PiiIdentifying),
        DataClassification::from(DataClass::DeclaredPreference),
    ]);

    let decision = evaluate_data_use_gate(DataUseGateRequest {
        attributes: crate::DataUseAttributes {
            purpose: Purpose::CapabilityInvocation,
            data_classification: DataClassification::from(DataClass::DeclaredPreference),
            subject_class: SubjectClass::Adult,
        },
        override_pack: Some(&override_pack),
        tenant_policy: &tenant_policy,
        derived_lineage: Some(&lineage),
        eu_ai_risk: None,
    });

    assert_eq!(
        decision,
        Err(DataUseGateDenialReason::OverridePackDenied {
            microservice_id: "privacy-gate",
            scope: HardDenyScope::AnyMicroserviceExceptHome,
        })
    );
}

#[test]
fn requested_classification_cannot_be_downgraded_by_lineage() {
    let override_pack = MicroserviceOverridePack::new("ads");
    let tenant_policy = TenantDataUsePolicy::default()
        .allow(Purpose::AdsTargeting, privacy(DataClass::Public))
        .allow(Purpose::AdsTargeting, privacy(DataClass::Phi));
    let lineage =
        DerivedFeatureLineage::from_sources([DataClassification::from(DataClass::Public)]);

    let decision = evaluate_data_use_gate(DataUseGateRequest {
        attributes: crate::DataUseAttributes {
            purpose: Purpose::AdsTargeting,
            data_classification: DataClassification::from(DataClass::Phi),
            subject_class: SubjectClass::Adult,
        },
        override_pack: Some(&override_pack),
        tenant_policy: &tenant_policy,
        derived_lineage: Some(&lineage),
        eu_ai_risk: None,
    });

    assert_eq!(
        decision,
        Err(DataUseGateDenialReason::DataUseBoundaryDenied(
            DataUseDenialReason::HardDeniedDataClass,
        ))
    );
}
