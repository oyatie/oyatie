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
