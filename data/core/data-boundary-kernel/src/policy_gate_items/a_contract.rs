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
