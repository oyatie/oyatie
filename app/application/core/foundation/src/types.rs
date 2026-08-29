//! Request and registration value types accepted by the foundation.

use crate::*;

use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRegistration {
    pub tenant_id: String,
    pub legal_name: String,
    pub home_region: String,
    pub residency_class: String,
    pub regulatory_packs: Vec<String>,
    pub autonomy_ceiling: AutonomyTier,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentityRegistration {
    pub tenant_id: String,
    pub user_id: String,
    pub primary_identifier: String,
    pub display_name: String,
    pub roles: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenRequest {
    pub tenant_id: String,
    pub user_id: String,
    pub purpose: Purpose,
    pub ttl_seconds: u64,
    pub issued_at_epoch_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegionalPackRegistration {
    pub pack_id: String,
    pub region: String,
    pub residency_class: String,
    pub controls: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectPropertyInput {
    pub name: String,
    pub value: String,
    pub tier: PropertyTier,
    pub privacy_data_class: PrivacyDataClass, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectEntityUpsert {
    pub tenant_id: String,
    pub entity_id: String,
    pub entity_type: String,
    pub properties: Vec<ObjectPropertyInput>,
}

impl ObjectPropertyInput {
    pub fn new(
        name: String,
        value: String,
        tier: PropertyTier,
        privacy_data_class: PrivacyDataClass,
    ) -> Self {
        Self {
            name,
            value,
            tier,
            privacy_data_class,
        }
    }

    /// Compatibility constructor for legacy callers that still supply raw
    /// `DataClass` labels; fails closed for operational and subject markers.
    pub fn try_from_legacy_data_class(
        name: String,
        value: String,
        tier: PropertyTier,
        data_class: DataClass,
    ) -> Result<Self, FoundationError> {
        let privacy_data_class =
            PrivacyDataClass::try_from(data_class).map_err(|_| FoundationError::InvalidInput)?;
        Ok(Self::new(name, value, tier, privacy_data_class))
    }

    /// Legacy object-property compatibility label for call sites that still
    /// traffic in raw `DataClass` payloads. The source of truth is
    /// `privacy_data_class`, and construction fails closed for operational and
    /// subject markers.
    pub fn legacy_data_class(&self) -> DataClass {
        self.privacy_data_class.data_class()
    }

    #[deprecated(
        note = "use privacy_data_class for canonical typed access or legacy_data_class for the compatibility projection"
    )]
    pub fn data_class(&self) -> DataClass {
        self.legacy_data_class()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxPublish {
    pub tenant_id: String,
    pub topic: String,
    pub idempotency_key: String,
    pub payload_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationRequest {
    pub tenant_id: String,
    pub user_id: String,
    pub action: String,
    pub resource: String,
    pub attributes: Vec<(String, String)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityRegistration {
    pub capability_id: String,
    pub namespace: String,
    pub action: CapabilityAction, // data_class: INTERNAL_ONLY
    pub required_tier: AutonomyTier,
    pub touched_privacy_data_classes: Vec<PrivacyDataClass>,
    pub evidence_topic: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantCapabilityGrant {
    pub tenant_id: String,
    pub capability_id: String,
    pub mcp_visible: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpDiscoveryRequest {
    pub tenant_id: String,
    pub access_token: McpAccessTokenClaims,
    pub now_epoch_seconds: u64,
    pub tld: String,
    pub authorization_server: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpToolCallRequest {
    pub tenant_id: String,
    pub user_id: String,
    pub tool_name: String,
    pub access_token: McpAccessTokenClaims,
    pub tld: String,
    pub authorization_server: String,
    pub purpose: Purpose,
    pub subject_class: SubjectClass, // data_class: INTERNAL_ONLY
    pub budget_window_id: String,
    pub projected_cost_micros: u64,
    pub started_at_epoch_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CostBudgetRegistration {
    pub tenant_id: String,
    pub capability_id: Option<String>,
    pub window_id: String,
    pub monthly_limit_micros: u64,
    pub per_invocation_limit_micros: u64,
    pub warning_threshold_percent: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityInvocationRequest {
    pub tenant_id: String,
    pub user_id: String,
    pub capability_id: String,
    pub purpose: Purpose,
    pub subject_class: SubjectClass, // data_class: INTERNAL_ONLY
    pub budget_window_id: String,
    pub projected_cost_micros: u64,
    pub started_at_epoch_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityInvocationPrincipal {
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub user_id: String,                // data_class: INTERNAL_ONLY
    pub autonomy_ceiling: AutonomyTier, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvocationReceipt {
    pub tenant_id: String,
    pub user_id: String,
    pub capability_id: String,
    pub evidence_event_hash: String,
    pub cost_reservation_id: Option<String>,
    pub cost_budget_warning: Option<BudgetWarning>,
    pub run_id: Option<String>,
    pub foundry_step_id: Option<String>,
    pub foundry_evidence_id: Option<String>,
}
