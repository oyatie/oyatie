//! Foundry policy kernel: runtime autonomy ceiling checks.

use intelligence_capability_domain::{AutonomyTier, Capability, CapabilityAction};
use data_boundary_kernel::{DataClass, PrivacyDataClass, SubjectClass};

const HEALTH_REGULATED_PACK_MARKERS: &[&str] = &[
    "clinical",
    "healthcare",
    "health-regulated",
    "protected-health",
    "regulated-health",
];
const FINANCIAL_REGULATED_PACK_MARKERS: &[&str] = &[
    "cardholder",
    "financial",
    "fintech",
    "payment-card",
    "regulated-credit",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantPolicy {
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub autonomy_ceiling: AutonomyTier, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AutonomyVerdict {
    Allow,
    Deny,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AutonomyCapSource {
    TenantConfigured,
    Principal,
    AgenticAds,
    VerticalPack,
    SubjectClass,
    CapabilityRequired,
}

impl AutonomyCapSource {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TenantConfigured => "tenant_configured",
            Self::Principal => "principal",
            Self::AgenticAds => "agentic_ads",
            Self::VerticalPack => "vertical_pack",
            Self::SubjectClass => "subject_class",
            Self::CapabilityRequired => "capability_required",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AutonomyCapReason {
    TenantConfiguredCeiling,
    PrincipalInheritedCeiling,
    AgenticAdsDefault,
    VerticalPackRegulatedData,
    SubjectClassRisk,
    CapabilityRequiredTier,
}

impl AutonomyCapReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TenantConfiguredCeiling => "tenant_configured_ceiling",
            Self::PrincipalInheritedCeiling => "principal_inherited_ceiling",
            Self::AgenticAdsDefault => "agentic_ads_default",
            Self::VerticalPackRegulatedData => "vertical_pack_regulated_data",
            Self::SubjectClassRisk => "subject_class_risk",
            Self::CapabilityRequiredTier => "capability_required_tier",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutonomyCeilingInputs {
    pub tenant_id: String,                       // data_class: INTERNAL_ONLY
    pub capability_id: String,                   // data_class: INTERNAL_ONLY
    pub tenant_configured_ceiling: AutonomyTier, // data_class: INTERNAL_ONLY
    pub principal_ceiling: AutonomyTier,         // data_class: INTERNAL_ONLY
    pub capability_required_cap: AutonomyTier,   // data_class: INTERNAL_ONLY
    pub agentic_ads_cap: AutonomyTier,           // data_class: INTERNAL_ONLY
    pub vertical_pack_cap: AutonomyTier,         // data_class: INTERNAL_ONLY
    pub subject_class: SubjectClass,             // data_class: INTERNAL_ONLY
    pub subject_class_cap: AutonomyTier,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutonomyDecision {
    pub tenant_id: String,                              // data_class: INTERNAL_ONLY
    pub capability_id: String,                          // data_class: INTERNAL_ONLY
    pub configured_ceiling: AutonomyTier,               // data_class: INTERNAL_ONLY
    pub tenant_configured_ceiling: AutonomyTier,        // data_class: INTERNAL_ONLY
    pub principal_ceiling: AutonomyTier,                // data_class: INTERNAL_ONLY
    pub capability_required_cap: AutonomyTier,          // data_class: INTERNAL_ONLY
    pub agentic_ads_cap: AutonomyTier,                  // data_class: INTERNAL_ONLY
    pub vertical_pack_cap: AutonomyTier,                // data_class: INTERNAL_ONLY
    pub subject_class: SubjectClass,                    // data_class: INTERNAL_ONLY
    pub subject_class_cap: AutonomyTier,                // data_class: INTERNAL_ONLY
    pub denial_threshold: AutonomyTier,                 // data_class: INTERNAL_ONLY
    pub effective_ceiling: AutonomyTier,                // data_class: INTERNAL_ONLY
    pub required_tier: AutonomyTier,                    // data_class: INTERNAL_ONLY
    pub verdict: AutonomyVerdict,                       // data_class: INTERNAL_ONLY
    pub blocking_cap_source: Option<AutonomyCapSource>, // data_class: INTERNAL_ONLY
    pub blocking_cap_reason: Option<AutonomyCapReason>, // data_class: INTERNAL_ONLY
    pub lowering_cap_source: AutonomyCapSource,         // data_class: INTERNAL_ONLY
    pub lowering_cap_reason: AutonomyCapReason,         // data_class: INTERNAL_ONLY
}

impl TenantPolicy {
    pub fn new(tenant_id: String, autonomy_ceiling: AutonomyTier) -> Self {
        Self {
            tenant_id,
            autonomy_ceiling,
        }
    }

    pub fn permits(&self, capability: &Capability) -> bool {
        self.evaluate(capability).allowed()
    }

    pub fn evaluate(&self, capability: &Capability) -> AutonomyDecision {
        self.evaluate_with_principal_ceiling(capability, self.autonomy_ceiling)
    }

    pub fn evaluate_with_principal_ceiling(
        &self,
        capability: &Capability,
        principal_ceiling: AutonomyTier,
    ) -> AutonomyDecision {
        self.evaluate_with_context(capability, principal_ceiling, &[], SubjectClass::Adult)
    }

    pub fn evaluate_with_context(
        &self,
        capability: &Capability,
        principal_ceiling: AutonomyTier,
        regulatory_packs: &[String],
        subject_class: SubjectClass,
    ) -> AutonomyDecision {
        self.evaluate_inputs(AutonomyCeilingInputs::new(
            self.tenant_id.clone(),
            capability.id.clone(),
            self.autonomy_ceiling,
            principal_ceiling,
            capability.required_tier,
            agentic_ads_cap(capability),
            vertical_pack_cap(regulatory_packs, capability),
            subject_class,
            subject_class_cap(subject_class),
        ))
    }

    pub fn evaluate_inputs(&self, inputs: AutonomyCeilingInputs) -> AutonomyDecision {
        evaluate_autonomy_inputs(inputs)
    }
}

impl AutonomyCeilingInputs {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenant_id: String,
        capability_id: String,
        tenant_configured_ceiling: AutonomyTier,
        principal_ceiling: AutonomyTier,
        capability_required_cap: AutonomyTier,
        agentic_ads_cap: AutonomyTier,
        vertical_pack_cap: AutonomyTier,
        subject_class: SubjectClass,
        subject_class_cap: AutonomyTier,
    ) -> Self {
        Self {
            tenant_id,
            capability_id,
            tenant_configured_ceiling,
            principal_ceiling,
            capability_required_cap,
            agentic_ads_cap,
            vertical_pack_cap,
            subject_class,
            subject_class_cap,
        }
    }
}

impl AutonomyDecision {
    pub fn allowed(&self) -> bool {
        self.verdict == AutonomyVerdict::Allow
    }
}

pub fn evaluate_autonomy_inputs(inputs: AutonomyCeilingInputs) -> AutonomyDecision {
    let denial_threshold = min_tier(&[
        inputs.tenant_configured_ceiling,
        inputs.principal_ceiling,
        inputs.agentic_ads_cap,
        inputs.vertical_pack_cap,
        inputs.subject_class_cap,
    ]);
    let effective_ceiling = min_tier(&[
        inputs.tenant_configured_ceiling,
        inputs.principal_ceiling,
        inputs.capability_required_cap,
        inputs.agentic_ads_cap,
        inputs.vertical_pack_cap,
        inputs.subject_class_cap,
    ]);
    let verdict = if inputs.capability_required_cap <= denial_threshold {
        AutonomyVerdict::Allow
    } else {
        AutonomyVerdict::Deny
    };
    let blocking_cap = if verdict == AutonomyVerdict::Deny {
        first_cap_below_required(&inputs)
    } else {
        None
    };
    let lowering_cap = first_min_cap(&inputs, effective_ceiling);

    AutonomyDecision {
        tenant_id: inputs.tenant_id,
        capability_id: inputs.capability_id,
        configured_ceiling: inputs.tenant_configured_ceiling,
        tenant_configured_ceiling: inputs.tenant_configured_ceiling,
        principal_ceiling: inputs.principal_ceiling,
        capability_required_cap: inputs.capability_required_cap,
        agentic_ads_cap: inputs.agentic_ads_cap,
        vertical_pack_cap: inputs.vertical_pack_cap,
        subject_class: inputs.subject_class,
        subject_class_cap: inputs.subject_class_cap,
        denial_threshold,
        effective_ceiling,
        required_tier: inputs.capability_required_cap,
        verdict,
        blocking_cap_source: blocking_cap.map(|(source, _)| source),
        blocking_cap_reason: blocking_cap.map(|(_, reason)| reason),
        lowering_cap_source: lowering_cap.0,
        lowering_cap_reason: lowering_cap.1,
    }
}

pub fn agentic_ads_cap(capability: &Capability) -> AutonomyTier {
    if is_agentic_ads_action(capability) {
        AutonomyTier::T1ViewOnly
    } else {
        AutonomyTier::T4AutoExecute
    }
}

pub fn vertical_pack_cap(regulatory_packs: &[String], capability: &Capability) -> AutonomyTier {
    if has_healthcare_pack(regulatory_packs) && capability_touches_health_regulated_data(capability)
    {
        return AutonomyTier::T2Advisory;
    }
    if has_fintech_pack(regulatory_packs) && capability_touches_financial_regulated_data(capability)
    {
        return AutonomyTier::T2Advisory;
    }
    AutonomyTier::T4AutoExecute
}

pub const fn subject_class_cap(subject_class: SubjectClass) -> AutonomyTier {
    match subject_class {
        SubjectClass::Minor { .. } => AutonomyTier::T1ViewOnly,
        SubjectClass::Elderly | SubjectClass::Vulnerable => AutonomyTier::T2Advisory,
        SubjectClass::Adult | SubjectClass::Authority => AutonomyTier::T4AutoExecute,
    }
}

fn min_tier(tiers: &[AutonomyTier]) -> AutonomyTier {
    tiers
        .iter()
        .copied()
        .min()
        .unwrap_or(AutonomyTier::T4AutoExecute)
}

fn first_cap_below_required(
    inputs: &AutonomyCeilingInputs,
) -> Option<(AutonomyCapSource, AutonomyCapReason)> {
    ordered_caps(inputs)
        .into_iter()
        .find(|(_, _, cap)| *cap < inputs.capability_required_cap)
        .map(|(source, reason, _)| (source, reason))
}

fn first_min_cap(
    inputs: &AutonomyCeilingInputs,
    effective_ceiling: AutonomyTier,
) -> (AutonomyCapSource, AutonomyCapReason) {
    ordered_caps(inputs)
        .into_iter()
        .find(|(_, _, cap)| *cap == effective_ceiling)
        .map(|(source, reason, _)| (source, reason))
        .unwrap_or((
            AutonomyCapSource::CapabilityRequired,
            AutonomyCapReason::CapabilityRequiredTier,
        ))
}

fn ordered_caps(
    inputs: &AutonomyCeilingInputs,
) -> [(AutonomyCapSource, AutonomyCapReason, AutonomyTier); 6] {
    [
        (
            AutonomyCapSource::TenantConfigured,
            AutonomyCapReason::TenantConfiguredCeiling,
            inputs.tenant_configured_ceiling,
        ),
        (
            AutonomyCapSource::Principal,
            AutonomyCapReason::PrincipalInheritedCeiling,
            inputs.principal_ceiling,
        ),
        (
            AutonomyCapSource::AgenticAds,
            AutonomyCapReason::AgenticAdsDefault,
            inputs.agentic_ads_cap,
        ),
        (
            AutonomyCapSource::VerticalPack,
            AutonomyCapReason::VerticalPackRegulatedData,
            inputs.vertical_pack_cap,
        ),
        (
            AutonomyCapSource::SubjectClass,
            AutonomyCapReason::SubjectClassRisk,
            inputs.subject_class_cap,
        ),
        (
            AutonomyCapSource::CapabilityRequired,
            AutonomyCapReason::CapabilityRequiredTier,
            inputs.capability_required_cap,
        ),
    ]
}

fn has_healthcare_pack(regulatory_packs: &[String]) -> bool {
    has_pack_marker(regulatory_packs, HEALTH_REGULATED_PACK_MARKERS)
}

fn has_fintech_pack(regulatory_packs: &[String]) -> bool {
    has_pack_marker(regulatory_packs, FINANCIAL_REGULATED_PACK_MARKERS)
}

fn has_pack_marker(regulatory_packs: &[String], accepted_markers: &[&str]) -> bool {
    regulatory_packs.iter().any(|pack| {
        let normalized = normalize_policy_marker(pack);
        accepted_markers
            .iter()
            .any(|marker| normalized.contains(marker))
    })
}

fn normalize_policy_marker(marker: &str) -> String {
    marker.trim().to_ascii_lowercase().replace('_', "-")
}

fn capability_touches_health_regulated_data(capability: &Capability) -> bool {
    capability
        .touched_privacy_data_classes()
        .iter()
        .copied()
        .any(is_health_regulated_privacy_class)
}

fn capability_touches_financial_regulated_data(capability: &Capability) -> bool {
    capability
        .touched_privacy_data_classes()
        .iter()
        .copied()
        .any(is_financial_regulated_privacy_class)
}

fn is_health_regulated_privacy_class(data_class: PrivacyDataClass) -> bool {
    matches!(
        data_class.data_class(),
        DataClass::Phi | DataClass::SensitivePipaArticle23 | DataClass::PipaArticle23
    )
}

fn is_financial_regulated_privacy_class(data_class: PrivacyDataClass) -> bool {
    matches!(
        data_class.data_class(),
        DataClass::Pci | DataClass::FinancialRegulatedCredit | DataClass::Financial
    )
}

fn is_agentic_ads_action(capability: &Capability) -> bool {
    matches!(
        capability.action,
        CapabilityAction::AdsBid | CapabilityAction::AdsBudgetAdjust
    )
}
