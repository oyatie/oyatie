//! Data Use Boundary policy-gate slice.
//!
//! This module keeps the first PRIVACY-001 gate fixture close to the DUB value
//! types without claiming a full Cedar/runtime integration.

use crate::{
    ClassificationLevel, ConsentScope, DataClass, DataClassification, DataUseAttributes,
    DataUseDenialReason, PrivacyDataClass, Purpose,
};

/// DUB purpose/data-class matrix facade.
///
/// This is intentionally a thin wrapper over the existing kernel hard-deny
/// functions. PRIVACY-001 needs a named fixture/gate seam, not a second policy
/// engine that can drift away from ADR-0008.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DataUseBoundaryMatrix {
    _private: (),
}

impl DataUseBoundaryMatrix {
    pub fn is_hard_denied(
        self,
        purpose: Purpose,
        classification: impl Into<DataClassification>,
    ) -> bool {
        crate::is_hard_denied_classification(purpose, classification)
    }
}

/// ADR-0034 hard-deny scopes owned by a microservice override pack.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum HardDenyScope {
    AdSourcing,
    CrossTenantSharing,
    CrossRegionTransfer,
    AnyMicroserviceExceptHome,
    All,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct OverrideDenyRule {
    classification: DataClassification, // data_class: INTERNAL_ONLY
    purpose: Option<Purpose>,           // data_class: INTERNAL_ONLY
    scope: HardDenyScope,               // data_class: INTERNAL_ONLY
}

impl OverrideDenyRule {
    pub const fn new(classification: DataClassification) -> Self {
        Self {
            classification,
            purpose: None,
            scope: HardDenyScope::All,
        }
    }

    pub const fn for_purpose(mut self, purpose: Purpose) -> Self {
        self.purpose = Some(purpose);
        self
    }

    pub const fn with_scope(mut self, scope: HardDenyScope) -> Self {
        self.scope = scope;
        self
    }

    fn applies_to(self, purpose: Purpose, classification: DataClassification) -> bool {
        canonical_policy_classification(self.classification)
            == canonical_policy_classification(classification)
            && self.purpose.is_none_or(|p| p == purpose)
    }
}

/// Immutable microservice override pack loaded before tenant policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MicroserviceOverridePack {
    microservice_id: &'static str,      // data_class: INTERNAL_ONLY
    hard_denies: Vec<OverrideDenyRule>, // data_class: INTERNAL_ONLY
}

impl MicroserviceOverridePack {
    pub fn new(microservice_id: &'static str) -> Self {
        Self {
            microservice_id,
            hard_denies: Vec::new(),
        }
    }

    pub fn deny(mut self, rule: OverrideDenyRule) -> Self {
        self.hard_denies.push(rule);
        self
    }

    fn denial_for(
        &self,
        purpose: Purpose,
        classification: DataClassification,
    ) -> Option<HardDenyScope> {
        self.hard_denies
            .iter()
            .find(|rule| rule.applies_to(purpose, classification))
            .map(|rule| rule.scope)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TenantDataUsePolicy {
    consent_scope: ConsentScope, // data_class: INTERNAL_ONLY
}

impl TenantDataUsePolicy {
    pub fn allow(mut self, purpose: Purpose, data_class: PrivacyDataClass) -> Self {
        self.consent_scope = self.consent_scope.allow(purpose, data_class);
        self
    }

    fn allows_classification(&self, purpose: Purpose, classification: DataClassification) -> bool {
        self.consent_scope
            .allows_classification(purpose, classification)
    }
}

/// Derived feature lineage carries the source classifications that must be
/// inherited by a model feature or computed attribute. Policy evaluation keeps
/// every source classification live so equal-severity classes cannot erase
/// each other's purpose-bound consent or override-pack rows.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DerivedFeatureLineage {
    sources: Vec<DataClassification>, // data_class: INTERNAL_ONLY
}

impl DerivedFeatureLineage {
    pub fn from_sources(sources: impl IntoIterator<Item = DataClassification>) -> Self {
        Self {
            sources: sources.into_iter().collect(),
        }
    }

    pub fn effective_classification(&self) -> Option<DataClassification> {
        most_restrictive_policy_classification(self.source_classifications())
    }

    fn source_classifications(&self) -> impl Iterator<Item = DataClassification> + '_ {
        self.sources
            .iter()
            .copied()
            .map(canonical_policy_classification)
    }

    fn hard_denied_source(&self, purpose: Purpose) -> Option<DataClassification> {
        self.source_classifications().find(|classification| {
            DataUseBoundaryMatrix::default().is_hard_denied(purpose, *classification)
        })
    }
}

fn classification_level(classification: DataClassification) -> ClassificationLevel {
    ClassificationLevel::from_data_class(classification.compatibility_data_class())
}

fn most_restrictive_policy_classification(
    classifications: impl IntoIterator<Item = DataClassification>,
) -> Option<DataClassification> {
    classifications.into_iter().max_by_key(|classification| {
        (
            classification_level(*classification),
            classification.compatibility_data_class(),
        )
    })
}

/// ADR-0144 graduated EU AI Act risk tier.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum EuAiRiskTier {
    Minimal,
    Limited,
    GeneralPurpose,
    HighRisk,
    Unacceptable,
}

impl EuAiRiskTier {
    pub const fn blocks_deployment(self) -> bool {
        matches!(self, Self::HighRisk | Self::Unacceptable)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct EuAiRiskRegistryEntry {
    archetype: &'static str, // data_class: INTERNAL_ONLY
    tier: EuAiRiskTier,      // data_class: INTERNAL_ONLY
}

impl EuAiRiskRegistryEntry {
    pub const fn new(archetype: &'static str, tier: EuAiRiskTier) -> Self {
        Self { archetype, tier }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DataUseGateRequest<'a> {
    pub attributes: DataUseAttributes, // data_class: INTERNAL_ONLY
    pub override_pack: Option<&'a MicroserviceOverridePack>, // data_class: INTERNAL_ONLY
    pub tenant_policy: &'a TenantDataUsePolicy, // data_class: INTERNAL_ONLY
    pub derived_lineage: Option<&'a DerivedFeatureLineage>, // data_class: INTERNAL_ONLY
    pub eu_ai_risk: Option<&'a EuAiRiskRegistryEntry>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DataUseGateDenialReason {
    OverridePackMissing,
    OverridePackDenied {
        microservice_id: &'static str,
        scope: HardDenyScope,
    },
    DerivedLineageDenied {
        inherited_classification: DataClassification,
    },
    DataUseBoundaryDenied(DataUseDenialReason),
    EuAiRiskRegistryMissing,
    EuAiRiskTierDenied {
        archetype: &'static str,
        tier: EuAiRiskTier,
    },
    TenantPolicyDenied,
}

fn policy_classifications_for_request(
    request: &DataUseGateRequest<'_>,
    requested_classification: DataClassification,
) -> Vec<DataClassification> {
    let mut classifications = vec![requested_classification];
    if let Some(lineage) = request.derived_lineage {
        classifications.extend(lineage.source_classifications());
    }
    classifications.sort();
    classifications.dedup();
    classifications
}

pub fn evaluate_data_use_gate(
    request: DataUseGateRequest<'_>,
) -> Result<(), DataUseGateDenialReason> {
    let override_pack = request
        .override_pack
        .ok_or(DataUseGateDenialReason::OverridePackMissing)?;
    let requested_classification =
        canonical_policy_classification(request.attributes.data_classification);
    let policy_classifications =
        policy_classifications_for_request(&request, requested_classification);
    let effective_classification =
        most_restrictive_policy_classification(policy_classifications.iter().copied())
            .unwrap_or(requested_classification);

    let lineage_denial = request
        .derived_lineage
        .and_then(|lineage| lineage.hard_denied_source(request.attributes.purpose));
    if let Some(inherited_classification) = lineage_denial {
        return Err(DataUseGateDenialReason::DerivedLineageDenied {
            inherited_classification,
        });
    }

    if DataUseBoundaryMatrix::default()
        .is_hard_denied(request.attributes.purpose, requested_classification)
    {
        return Err(DataUseGateDenialReason::DataUseBoundaryDenied(
            DataUseDenialReason::HardDeniedDataClass,
        ));
    }

    if let Some(scope) = policy_classifications
        .iter()
        .copied()
        .find_map(|classification| {
            override_pack.denial_for(request.attributes.purpose, classification)
        })
    {
        return Err(DataUseGateDenialReason::OverridePackDenied {
            microservice_id: override_pack.microservice_id,
            scope,
        });
    }

    let effective_attributes = DataUseAttributes {
        data_classification: effective_classification,
        ..request.attributes
    };
    crate::evaluate_data_use(effective_attributes)
        .map_err(DataUseGateDenialReason::DataUseBoundaryDenied)?;

    if requires_eu_ai_risk_tier(request.attributes.purpose) {
        let risk = request
            .eu_ai_risk
            .ok_or(DataUseGateDenialReason::EuAiRiskRegistryMissing)?;
        if risk.tier.blocks_deployment() {
            return Err(DataUseGateDenialReason::EuAiRiskTierDenied {
                archetype: risk.archetype,
                tier: risk.tier,
            });
        }
    }

    if !policy_classifications
        .iter()
        .copied()
        .all(|classification| {
            request
                .tenant_policy
                .allows_classification(request.attributes.purpose, classification)
        })
    {
        return Err(DataUseGateDenialReason::TenantPolicyDenied);
    }

    Ok(())
}

fn canonical_policy_classification(classification: DataClassification) -> DataClassification {
    match classification {
        DataClassification::Privacy(data_class) => {
            DataClassification::from(match data_class.data_class() {
                DataClass::PiiSensitive => DataClass::PiiQuasiIdentifier,
                DataClass::Usage => DataClass::BehavioralTenantProduct,
                DataClass::PipaArticle23 => DataClass::SensitivePipaArticle23,
                canonical => canonical,
            })
        }
        DataClassification::Operational(_) | DataClassification::SubjectMarker(_) => classification,
    }
}

fn requires_eu_ai_risk_tier(purpose: Purpose) -> bool {
    matches!(
        purpose,
        Purpose::ModelTrainingOya | Purpose::ModelTrainingThirdParty
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AgeBand, DataClass, DataClassification, OperationalDataClass, PrivacyDataClass, Purpose,
        SubjectClass,
    };

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
    fn derived_feature_lineage_inherits_hard_deny_source_class() {
        let override_pack = MicroserviceOverridePack::new("analytics");
        let tenant_policy =
            tenant_allows(Purpose::AdTargetingDeclared, DataClass::DeclaredPreference);
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
        let tenant_policy =
            tenant_allows(Purpose::CapabilityInvocation, DataClass::DeclaredPreference);
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

    #[test]
    fn eu_ai_high_risk_registry_entry_denies_model_training_even_for_public_data() {
        let override_pack = MicroserviceOverridePack::new("intelligence");
        let tenant_policy = tenant_allows(Purpose::ModelTrainingOya, DataClass::Public);
        let risk =
            EuAiRiskRegistryEntry::new("auto-employment-decisioning", EuAiRiskTier::HighRisk);

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
}
