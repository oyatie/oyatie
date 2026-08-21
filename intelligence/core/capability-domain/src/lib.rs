//! Foundry capability kernel: capability records and autonomy tier requirements.

use std::collections::BTreeMap;

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AutonomyTier {
    T1ViewOnly = 1,
    T2Advisory = 2,
    T3ExecuteWithApproval = 3,
    T4AutoExecute = 4,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CapabilityAction {
    Other,
    AdsBid,
    AdsBudgetAdjust,
}

pub const DEFAULT_FOUNDATION_LOCAL_PROVIDER_ID: &str = "foundation-local";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityCostProfile {
    pub per_invocation_limit_micros: Classified<u64>, // data_class: INTERNAL_ONLY
    pub per_tenant_monthly_limit_micros: Classified<u64>, // data_class: INTERNAL_ONLY
    pub provider_preference: Classified<Vec<String>>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityMcpContract {
    pub agent_readable_description: Classified<String>, // data_class: PUBLIC
    pub human_readable_description: Classified<String>, // data_class: PUBLIC
    pub input_schema: Classified<String>,               // data_class: PUBLIC
    pub output_schema: Classified<String>,              // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Capability {
    pub id: String,
    pub namespace: Classified<String>,
    pub action: CapabilityAction, // data_class: INTERNAL_ONLY
    pub required_tier: AutonomyTier,
    touched_data_classes: Vec<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub cost_profile: CapabilityCostProfile,     // data_class: INTERNAL_ONLY
    pub mcp_contract: CapabilityMcpContract,     // data_class: PUBLIC
    pub evidence_topic: Classified<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CapabilityError {
    InvalidCapabilityId,
    InvalidTenantId,
    EmptyNamespace,
    EmptyEvidenceTopic,
    MissingDataClasses,
    NonPrivacyDataClass,
    InvalidCostProfile,
    MissingProviderPreference,
    InvalidProviderPreference,
    InvalidMcpContract,
    DuplicateCapability,
    CapabilityNotFound,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantCapabilityBinding {
    pub tenant_id: Classified<String>,
    pub capability_id: Classified<String>,
    pub mcp_visible: Classified<bool>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityRegistry {
    capabilities: Classified<BTreeMap<String, Capability>>,
    tenant_bindings: Classified<BTreeMap<(String, String), TenantCapabilityBinding>>,
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self {
            capabilities: Classified::new(BTreeMap::new(), DataClass::InternalOnly),
            tenant_bindings: Classified::new(BTreeMap::new(), DataClass::InternalOnly),
        }
    }
}

impl Capability {
    pub fn new(
        id: String,
        namespace: String,
        required_tier: AutonomyTier,
        touched_data_classes: Vec<PrivacyDataClass>,
        evidence_topic: String,
    ) -> Result<Self, CapabilityError> {
        Self::new_with_action(
            id,
            namespace,
            CapabilityAction::Other,
            required_tier,
            touched_data_classes,
            evidence_topic,
        )
    }

    pub fn new_with_action(
        id: String,
        namespace: String,
        action: CapabilityAction,
        required_tier: AutonomyTier,
        touched_data_classes: Vec<PrivacyDataClass>,
        evidence_topic: String,
    ) -> Result<Self, CapabilityError> {
        Self::new_with_action_and_cost_profile(
            id,
            namespace,
            action,
            required_tier,
            touched_data_classes,
            evidence_topic,
            CapabilityCostProfile::foundation_local_default(),
        )
    }

    pub fn new_with_cost_profile(
        id: String,
        namespace: String,
        required_tier: AutonomyTier,
        touched_data_classes: Vec<PrivacyDataClass>,
        evidence_topic: String,
        cost_profile: CapabilityCostProfile,
    ) -> Result<Self, CapabilityError> {
        Self::new_with_action_and_cost_profile(
            id,
            namespace,
            CapabilityAction::Other,
            required_tier,
            touched_data_classes,
            evidence_topic,
            cost_profile,
        )
    }

    pub fn new_with_action_and_cost_profile(
        id: String,
        namespace: String,
        action: CapabilityAction,
        required_tier: AutonomyTier,
        touched_data_classes: Vec<PrivacyDataClass>,
        evidence_topic: String,
        cost_profile: CapabilityCostProfile,
    ) -> Result<Self, CapabilityError> {
        let mcp_contract = CapabilityMcpContract::default_for(&id, &namespace)?;
        Self::new_with_action_and_profiles(
            id,
            namespace,
            action,
            required_tier,
            touched_data_classes,
            evidence_topic,
            (cost_profile, mcp_contract),
        )
    }

    pub fn new_with_mcp_contract(
        id: String,
        namespace: String,
        required_tier: AutonomyTier,
        touched_data_classes: Vec<PrivacyDataClass>,
        evidence_topic: String,
        mcp_contract: CapabilityMcpContract,
    ) -> Result<Self, CapabilityError> {
        Self::new_with_action_and_profiles(
            id,
            namespace,
            CapabilityAction::Other,
            required_tier,
            touched_data_classes,
            evidence_topic,
            (
                CapabilityCostProfile::foundation_local_default(),
                mcp_contract,
            ),
        )
    }

    pub fn new_with_cost_profile_and_mcp_contract(
        id: String,
        namespace: String,
        required_tier: AutonomyTier,
        touched_data_classes: Vec<PrivacyDataClass>,
        evidence_topic: String,
        cost_profile: CapabilityCostProfile,
        mcp_contract: CapabilityMcpContract,
    ) -> Result<Self, CapabilityError> {
        Self::new_with_action_and_profiles(
            id,
            namespace,
            CapabilityAction::Other,
            required_tier,
            touched_data_classes,
            evidence_topic,
            (cost_profile, mcp_contract),
        )
    }

    fn new_with_action_and_profiles(
        id: String,
        namespace: String,
        action: CapabilityAction,
        required_tier: AutonomyTier,
        touched_data_classes: Vec<PrivacyDataClass>,
        evidence_topic: String,
        profiles: (CapabilityCostProfile, CapabilityMcpContract),
    ) -> Result<Self, CapabilityError> {
        if !id.starts_with("cap.") {
            return Err(CapabilityError::InvalidCapabilityId);
        }
        if namespace.trim().is_empty() {
            return Err(CapabilityError::EmptyNamespace);
        }
        if evidence_topic.trim().is_empty() {
            return Err(CapabilityError::EmptyEvidenceTopic);
        }
        if touched_data_classes.is_empty() {
            return Err(CapabilityError::MissingDataClasses);
        }
        let (cost_profile, mcp_contract) = profiles;
        Ok(Self {
            id,
            namespace: Classified::new(namespace, DataClass::InternalOnly),
            action,
            required_tier,
            touched_data_classes,
            cost_profile,
            mcp_contract,
            evidence_topic: Classified::new(evidence_topic, DataClass::InternalOnly),
        })
    }

    /// Compatibility constructor for import/config seams that still carry raw
    /// `DataClass` labels. Canonical capability construction takes
    /// `PrivacyDataClass` and this path fails closed for operational markers
    /// and subject markers.
    pub fn try_from_legacy_data_classes(
        id: String,
        namespace: String,
        required_tier: AutonomyTier,
        touched_data_classes: Vec<DataClass>,
        evidence_topic: String,
    ) -> Result<Self, CapabilityError> {
        Self::try_from_legacy_action_data_classes(
            id,
            namespace,
            CapabilityAction::Other,
            required_tier,
            touched_data_classes,
            evidence_topic,
        )
    }

    /// Compatibility constructor for legacy raw `DataClass` labels plus an
    /// explicit action. See [`Self::try_from_legacy_data_classes`].
    pub fn try_from_legacy_action_data_classes(
        id: String,
        namespace: String,
        action: CapabilityAction,
        required_tier: AutonomyTier,
        touched_data_classes: Vec<DataClass>,
        evidence_topic: String,
    ) -> Result<Self, CapabilityError> {
        let touched_data_classes = touched_data_classes
            .into_iter()
            .map(PrivacyDataClass::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| CapabilityError::NonPrivacyDataClass)?;
        Self::new_with_action(
            id,
            namespace,
            action,
            required_tier,
            touched_data_classes,
            evidence_topic,
        )
    }

    pub fn new_with_privacy_data_classes(
        id: String,
        namespace: String,
        required_tier: AutonomyTier,
        touched_data_classes: Vec<PrivacyDataClass>,
        evidence_topic: String,
    ) -> Result<Self, CapabilityError> {
        Self::new(
            id,
            namespace,
            required_tier,
            touched_data_classes,
            evidence_topic,
        )
    }

    pub fn new_with_action_and_privacy_data_classes(
        id: String,
        namespace: String,
        action: CapabilityAction,
        required_tier: AutonomyTier,
        touched_data_classes: Vec<PrivacyDataClass>,
        evidence_topic: String,
    ) -> Result<Self, CapabilityError> {
        Self::new_with_action(
            id,
            namespace,
            action,
            required_tier,
            touched_data_classes,
            evidence_topic,
        )
    }

    pub fn cost_profile(&self) -> &CapabilityCostProfile {
        &self.cost_profile
    }

    pub fn mcp_contract(&self) -> &CapabilityMcpContract {
        &self.mcp_contract
    }

    pub fn provider_preference(&self) -> &[String] {
        &self.cost_profile.provider_preference.value
    }

    pub fn allows_projected_invocation_cost(&self, projected_cost_micros: u64) -> bool {
        projected_cost_micros <= self.cost_profile.per_invocation_limit_micros.value
    }

    pub fn touched_privacy_data_classes(&self) -> &[PrivacyDataClass] {
        &self.touched_data_classes
    }

    /// Legacy capability-record projection for audit/config surfaces that
    /// still persist raw `DataClass` labels. Capability construction stores
    /// typed privacy classes only, so this projection remains lossless and
    /// cannot smuggle operational or subject markers into the privacy set.
    pub fn legacy_touched_data_classes(&self) -> Vec<DataClass> {
        self.touched_data_classes
            .iter()
            .map(|data_class| data_class.data_class())
            .collect()
    }

    #[deprecated(
        note = "use touched_privacy_data_classes for canonical typed access or legacy_touched_data_classes for the compatibility projection"
    )]
    pub fn touched_data_classes(&self) -> Vec<DataClass> {
        self.legacy_touched_data_classes()
    }
}

impl CapabilityCostProfile {
    pub fn new(
        per_invocation_limit_micros: u64,
        per_tenant_monthly_limit_micros: u64,
        provider_preference: Vec<String>,
    ) -> Result<Self, CapabilityError> {
        if per_invocation_limit_micros == 0
            || per_tenant_monthly_limit_micros == 0
            || per_invocation_limit_micros > per_tenant_monthly_limit_micros
        {
            return Err(CapabilityError::InvalidCostProfile);
        }
        if provider_preference.is_empty() {
            return Err(CapabilityError::MissingProviderPreference);
        }
        let mut seen = BTreeMap::new();
        for provider_id in &provider_preference {
            validate_provider_id(provider_id)?;
            if seen.insert(provider_id.clone(), ()).is_some() {
                return Err(CapabilityError::InvalidProviderPreference);
            }
        }
        Ok(Self {
            per_invocation_limit_micros: Classified::new(
                per_invocation_limit_micros,
                DataClass::InternalOnly,
            ),
            per_tenant_monthly_limit_micros: Classified::new(
                per_tenant_monthly_limit_micros,
                DataClass::InternalOnly,
            ),
            provider_preference: Classified::new(provider_preference, DataClass::InternalOnly),
        })
    }

    pub fn foundation_local_default() -> Self {
        // ADR-0083 Tier 1: bypass the fallible `Self::new` validator for the
        // statically known-valid foundation-local defaults. The values
        // (`u64::MAX` ceilings + the single `DEFAULT_FOUNDATION_LOCAL_PROVIDER_ID`
        // constant) trivially satisfy every check in `Self::new`
        // (`per_invocation_limit_micros <= per_tenant_monthly_limit_micros`,
        // non-empty provider preference, valid `cap.` / kebab-case provider id,
        // no duplicates), so a direct struct construction is correct and
        // removes the `.expect()` previously required at this site.
        Self {
            per_invocation_limit_micros: Classified::new(u64::MAX, DataClass::InternalOnly),
            per_tenant_monthly_limit_micros: Classified::new(u64::MAX, DataClass::InternalOnly),
            provider_preference: Classified::new(
                vec![DEFAULT_FOUNDATION_LOCAL_PROVIDER_ID.to_string()],
                DataClass::InternalOnly,
            ),
        }
    }
}

impl CapabilityMcpContract {
    pub fn new(
        agent_readable_description: String,
        human_readable_description: String,
        input_schema: String,
        output_schema: String,
    ) -> Result<Self, CapabilityError> {
        validate_mcp_description(&agent_readable_description)?;
        validate_mcp_description(&human_readable_description)?;
        validate_mcp_schema_object(&input_schema)?;
        validate_mcp_schema_object(&output_schema)?;
        Ok(Self {
            agent_readable_description: Classified::new(
                agent_readable_description,
                DataClass::Public,
            ),
            human_readable_description: Classified::new(
                human_readable_description,
                DataClass::Public,
            ),
            input_schema: Classified::new(input_schema, DataClass::Public),
            output_schema: Classified::new(output_schema, DataClass::Public),
        })
    }

    pub fn default_for(capability_id: &str, namespace: &str) -> Result<Self, CapabilityError> {
        Self::new(
            format!(
                "Invoke Oyatie capability {capability_id} in namespace {namespace} under the tenant MCP endpoint."
            ),
            format!(
                "Capability {capability_id} is documented for operators in the {namespace} namespace."
            ),
            r#"{"type":"object","additionalProperties":false}"#.to_string(),
            r#"{"type":"object"}"#.to_string(),
        )
    }
}

impl CapabilityRegistry {
    pub fn publish(&mut self, capability: Capability) -> Result<(), CapabilityError> {
        if self.capabilities.value.contains_key(&capability.id) {
            return Err(CapabilityError::DuplicateCapability);
        }
        self.capabilities
            .value
            .insert(capability.id.clone(), capability);
        Ok(())
    }

    pub fn get(&self, capability_id: &str) -> Option<&Capability> {
        self.capabilities.value.get(capability_id)
    }

    pub fn grant_to_tenant(
        &mut self,
        tenant_id: String,
        capability_id: String,
        mcp_visible: bool,
    ) -> Result<TenantCapabilityBinding, CapabilityError> {
        validate_tenant_id(&tenant_id)?;
        if !self.capabilities.value.contains_key(&capability_id) {
            return Err(CapabilityError::CapabilityNotFound);
        }
        let binding = TenantCapabilityBinding {
            tenant_id: Classified::new(tenant_id.clone(), DataClass::InternalOnly),
            capability_id: Classified::new(capability_id.clone(), DataClass::InternalOnly),
            mcp_visible: Classified::new(mcp_visible, DataClass::InternalOnly),
        };
        self.tenant_bindings
            .value
            .insert((tenant_id, capability_id), binding.clone());
        Ok(binding)
    }

    pub fn is_licensed_for_tenant(&self, tenant_id: &str, capability_id: &str) -> bool {
        self.tenant_bindings
            .value
            .contains_key(&(tenant_id.to_string(), capability_id.to_string()))
    }

    pub fn discover_for_tenant(
        &self,
        tenant_id: &str,
        autonomy_ceiling: AutonomyTier,
    ) -> Result<Vec<Capability>, CapabilityError> {
        validate_tenant_id(tenant_id)?;
        Ok(self
            .tenant_bindings
            .value
            .iter()
            .filter(|((binding_tenant_id, _), binding)| {
                binding_tenant_id == tenant_id && binding.mcp_visible.value
            })
            .filter_map(|((_, capability_id), _)| self.capabilities.value.get(capability_id))
            .filter(|capability| capability.required_tier <= autonomy_ceiling)
            .cloned()
            .collect())
    }
}

fn validate_tenant_id(tenant_id: &str) -> Result<(), CapabilityError> {
    if !tenant_id.starts_with("ten_") {
        return Err(CapabilityError::InvalidTenantId);
    }
    Ok(())
}

fn validate_provider_id(provider_id: &str) -> Result<(), CapabilityError> {
    if provider_id.is_empty()
        || !provider_id.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
    {
        return Err(CapabilityError::InvalidProviderPreference);
    }
    Ok(())
}

fn validate_mcp_description(description: &str) -> Result<(), CapabilityError> {
    if description.trim().is_empty() {
        return Err(CapabilityError::InvalidMcpContract);
    }
    Ok(())
}

fn validate_mcp_schema_object(schema: &str) -> Result<(), CapabilityError> {
    let normalized = schema
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    if normalized.is_empty()
        || !normalized.starts_with('{')
        || !normalized.ends_with('}')
        || !normalized.contains(r#""type":"object""#)
    {
        return Err(CapabilityError::InvalidMcpContract);
    }
    Ok(())
}
