#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Purpose {
    CoreService,
    CapabilityInvocation,
    SearchIndex,
    AdsTargeting,
    Analytics,
    Support,
    TenantAnalyticsFirstParty,
    CrossTenantAggregateAnonymous,
    PersonalizationInProduct,
    SearchIndexPrivate,
    SearchIndexPublic,
    AdTargetingDeclared,
    AdTargetingBehavioral,
    ModelTrainingOya,
    ModelTrainingThirdParty,
}

impl Purpose {
    /// Historical PascalCase label used by existing file-ledger records.
    pub const fn pascal_label(self) -> &'static str {
        match self {
            Self::CoreService => "CoreService",
            Self::CapabilityInvocation => "CapabilityInvocation",
            Self::SearchIndex => "SearchIndex",
            Self::AdsTargeting => "AdsTargeting",
            Self::Analytics => "Analytics",
            Self::Support => "Support",
            Self::TenantAnalyticsFirstParty => "TenantAnalyticsFirstParty",
            Self::CrossTenantAggregateAnonymous => "CrossTenantAggregateAnonymous",
            Self::PersonalizationInProduct => "PersonalizationInProduct",
            Self::SearchIndexPrivate => "SearchIndexPrivate",
            Self::SearchIndexPublic => "SearchIndexPublic",
            Self::AdTargetingDeclared => "AdTargetingDeclared",
            Self::AdTargetingBehavioral => "AdTargetingBehavioral",
            Self::ModelTrainingOya => "ModelTrainingOya",
            Self::ModelTrainingThirdParty => "ModelTrainingThirdParty",
        }
    }
}

pub fn parse_purpose_pascal_label(label: &str) -> Option<Purpose> {
    match label.trim() {
        "CoreService" => Some(Purpose::CoreService),
        "CapabilityInvocation" => Some(Purpose::CapabilityInvocation),
        "SearchIndex" => Some(Purpose::SearchIndex),
        "AdsTargeting" => Some(Purpose::AdsTargeting),
        "Analytics" => Some(Purpose::Analytics),
        "Support" => Some(Purpose::Support),
        "TenantAnalyticsFirstParty" => Some(Purpose::TenantAnalyticsFirstParty),
        "CrossTenantAggregateAnonymous" => Some(Purpose::CrossTenantAggregateAnonymous),
        "PersonalizationInProduct" => Some(Purpose::PersonalizationInProduct),
        "SearchIndexPrivate" => Some(Purpose::SearchIndexPrivate),
        "SearchIndexPublic" => Some(Purpose::SearchIndexPublic),
        "AdTargetingDeclared" => Some(Purpose::AdTargetingDeclared),
        "AdTargetingBehavioral" => Some(Purpose::AdTargetingBehavioral),
        "ModelTrainingOya" => Some(Purpose::ModelTrainingOya),
        "ModelTrainingThirdParty" => Some(Purpose::ModelTrainingThirdParty),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum AgeBand {
    Under13,
    Under14,
    Under16,
    Under19,
    UnknownMinor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum SubjectClass {
    Adult,
    Minor { age_band: AgeBand },
    Elderly,
    Vulnerable,
    Authority,
}
