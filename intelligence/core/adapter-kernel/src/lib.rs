//! Foundry multi-provider adapter kernel.
//!
//! Pure provider/auth/policy value objects plus deterministic route resolution.
//! Concrete network adapters live in adapter crates; this kernel never resolves
//! or exposes secret material.

use std::collections::BTreeSet;
use std::fmt;

use oya_check_cost_budget::BudgetSnapshot;
use data_boundary_kernel::{
    Classified, DataClass, PrivacyDataClass, data_classes_from_privacy_data_classes,
    privacy_data_classes_from,
};
use intelligence_capability_domain::Capability;
use secrets_domain::SecretRef;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdapterError {
    InvalidProviderId,
    InvalidTenantId,
    EmptyProviderAccount,
    EmptyFailoverChain,
    MissingDataClassAllowlist,
    MissingProviderRegion,
    MissingProviderCapability,
    AuthModeMismatch,
    InvalidCostCeiling,
    InvalidRequiredRegion,
    EmptyProviderCallIdempotencyKey,
    EmptyProviderModelRef,
    InvalidProviderCallAttempt,
    EmptyProviderRequestId,
    EmptyProviderPromptRef,
    EmptyProviderToolName,
    ProviderAdapterMismatch,
    InvalidProviderEventSequence,
    ProviderRetryableFailure,
    ProviderNonRetryableFailure,
    ProviderCallRegionMismatch,
    InvalidDataClass,
    DataClassNotAllowed,
    NoProviderAvailable,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProviderId {
    pub value: Classified<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderMode {
    Api,
    Subscription,
}

#[derive(Clone, Eq, PartialEq)]
pub enum ProviderAuth {
    Api {
        secret_ref: SecretRef,
        billing_account: String,
    },
    Subscription {
        session_token_ref: SecretRef,
        provider_account: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderProfile {
    pub id: ProviderId,
    pub mode: ProviderMode,
    pub auth: ProviderAuth,
    pub privacy_data_class_allowlist: Classified<Vec<PrivacyDataClass>>,
    pub regions_available: Classified<Vec<String>>,
    pub projected_invocation_cost_micros: Classified<u64>,
    pub p95_latency_ms: Classified<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CostCeiling {
    pub monthly_spend_micros: u64,
    pub monthly_limit_micros: u64,
    pub max_invocation_micros: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvocationPolicy {
    pub tenant_id: Classified<String>,
    pub allowed_privacy_data_classes: Vec<PrivacyDataClass>,
    pub required_region: Classified<String>,
    pub ceiling: CostCeiling,
    pub max_latency_ms: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRoutePreference {
    pub ordered_provider_ids: Classified<Vec<ProviderId>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRoute {
    pub providers: Vec<ProviderProfile>,
    pub selected_region: Classified<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderCallReceipt {
    pub provider_id: ProviderId,                 // data_class: INTERNAL_ONLY
    pub provider_mode: Classified<ProviderMode>, // data_class: INTERNAL_ONLY
    pub receipt_id: Classified<String>,          // data_class: INTERNAL_ONLY
    pub provider_region: Classified<String>,     // data_class: INTERNAL_ONLY
    pub model_ref: Classified<String>,           // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>,     // data_class: INTERNAL_ONLY
    pub attempt: Classified<u32>,                // data_class: INTERNAL_ONLY
    pub projected_cost_micros: Classified<u64>,  // data_class: INTERNAL_ONLY
    pub p95_latency_ms: Classified<u32>,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderStreamEndReason {
    Complete,
    ToolHandoff,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderFailureKind {
    Retryable,
    NonRetryable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptEnvelope {
    pub request_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub prompt_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub privacy_data_classes: Classified<Vec<PrivacyDataClass>>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolSchemaSet {
    pub tool_names: Classified<Vec<String>>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderInvocation {
    pub prompt: PromptEnvelope,              // data_class: INTERNAL_ONLY
    pub tools: ToolSchemaSet,                // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub model_ref: Classified<String>,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderInvocationRequest {
    pub provider_id: ProviderId,             // data_class: INTERNAL_ONLY
    pub prompt: PromptEnvelope,              // data_class: INTERNAL_ONLY
    pub tools: ToolSchemaSet,                // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<String>, // data_class: INTERNAL_ONLY
    pub model_ref: Classified<String>,       // data_class: INTERNAL_ONLY
    pub attempt: Classified<u32>,            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProviderEvent {
    StreamStart {
        provider_id: ProviderId,        // data_class: INTERNAL_ONLY
        request_id: Classified<String>, // data_class: INTERNAL_ONLY
    },
    Token {
        text: Classified<String>, // data_class: INTERNAL_ONLY
    },
    ToolCall {
        tool_name: Classified<String>, // data_class: INTERNAL_ONLY
    },
    Usage {
        input_tokens: Classified<u32>,  // data_class: INTERNAL_ONLY
        output_tokens: Classified<u32>, // data_class: INTERNAL_ONLY
        cost_micros: Classified<u64>,   // data_class: INTERNAL_ONLY
    },
    StreamEnd {
        reason: Classified<ProviderStreamEndReason>, // data_class: INTERNAL_ONLY
    },
    Error {
        kind: Classified<ProviderFailureKind>, // data_class: INTERNAL_ONLY
        message: Classified<String>,           // data_class: INTERNAL_ONLY
    },
    ResponseRestart {
        from_provider_id: ProviderId, // data_class: INTERNAL_ONLY
        to_provider_id: ProviderId,   // data_class: INTERNAL_ONLY
        reason: Classified<String>,   // data_class: INTERNAL_ONLY
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderAttemptTrace {
    pub provider_id: ProviderId,                // data_class: INTERNAL_ONLY
    pub attempt: Classified<u32>,               // data_class: INTERNAL_ONLY
    pub events: Classified<Vec<ProviderEvent>>, // data_class: INTERNAL_ONLY
    pub discarded_response: Classified<bool>,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderInvocationTrace {
    pub attempts: Vec<ProviderAttemptTrace>, // data_class: INTERNAL_ONLY
    pub failover_events: Vec<ProviderEvent>, // data_class: INTERNAL_ONLY
    pub final_provider_id: ProviderId,       // data_class: INTERNAL_ONLY
    pub final_events: Classified<Vec<ProviderEvent>>, // data_class: INTERNAL_ONLY
}

pub trait ProviderAdapter: Send + Sync {
    fn profile(&self) -> &ProviderProfile;

    fn invoke(
        &self,
        request: ProviderInvocationRequest,
    ) -> Result<Vec<ProviderEvent>, AdapterError>;
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SubscriptionBinding {
    pub tenant_id: Classified<String>,
    pub provider_id: ProviderId,
    pub provider_account: Classified<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SubscriptionBindingRegistry {
    bindings: BTreeSet<SubscriptionBinding>,
}

pub struct ProviderRouteRequest<'a> {
    pub capability: &'a Capability,
    pub policy: InvocationPolicy,
    pub preference: ProviderRoutePreference,
    pub profiles: &'a [ProviderProfile],
    pub subscription_bindings: &'a SubscriptionBindingRegistry,
}

impl ProviderId {
    pub fn new(value: String) -> Result<Self, AdapterError> {
        if value.is_empty()
            || !value.chars().all(|character| {
                character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
            })
        {
            return Err(AdapterError::InvalidProviderId);
        }
        Ok(Self {
            value: Classified::new(value, DataClass::InternalOnly),
        })
    }
}

impl ProviderAuth {
    pub fn mode(&self) -> ProviderMode {
        match self {
            Self::Api { .. } => ProviderMode::Api,
            Self::Subscription { .. } => ProviderMode::Subscription,
        }
    }
}

impl ProviderAuth {
    fn provider_account(&self) -> &str {
        match self {
            Self::Api {
                billing_account, ..
            } => billing_account,
            Self::Subscription {
                provider_account, ..
            } => provider_account,
        }
    }
}

impl fmt::Debug for ProviderAuth {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Api {
                billing_account, ..
            } => formatter
                .debug_struct("ProviderAuth::Api")
                .field("secret_ref", &"REDACTED")
                .field("billing_account", billing_account)
                .finish(),
            Self::Subscription {
                provider_account, ..
            } => formatter
                .debug_struct("ProviderAuth::Subscription")
                .field("session_token_ref", &"REDACTED")
                .field("provider_account", provider_account)
                .finish(),
        }
    }
}

impl ProviderProfile {
    pub fn new(
        id: ProviderId,
        mode: ProviderMode,
        auth: ProviderAuth,
        data_class_allowlist: Vec<PrivacyDataClass>,
        regions_available: Vec<String>,
        projected_invocation_cost_micros: u64,
        p95_latency_ms: u32,
    ) -> Result<Self, AdapterError> {
        Self::new_with_privacy_data_classes(
            id,
            mode,
            auth,
            data_class_allowlist,
            regions_available,
            projected_invocation_cost_micros,
            p95_latency_ms,
        )
    }

    /// Compatibility constructor for provider config seams that still carry
    /// raw `DataClass` labels. Canonical provider profiles take
    /// `PrivacyDataClass` and this path fails closed for operational markers
    /// and subject markers.
    pub fn try_from_legacy_data_class_allowlist(
        id: ProviderId,
        mode: ProviderMode,
        auth: ProviderAuth,
        data_class_allowlist: Vec<DataClass>,
        regions_available: Vec<String>,
        projected_invocation_cost_micros: u64,
        p95_latency_ms: u32,
    ) -> Result<Self, AdapterError> {
        let data_class_allowlist = privacy_data_classes_from(&data_class_allowlist)
            .map_err(|_| AdapterError::InvalidDataClass)?;
        Self::new(
            id,
            mode,
            auth,
            data_class_allowlist,
            regions_available,
            projected_invocation_cost_micros,
            p95_latency_ms,
        )
    }

    pub fn new_with_privacy_data_classes(
        id: ProviderId,
        mode: ProviderMode,
        auth: ProviderAuth,
        privacy_data_class_allowlist: Vec<PrivacyDataClass>,
        regions_available: Vec<String>,
        projected_invocation_cost_micros: u64,
        p95_latency_ms: u32,
    ) -> Result<Self, AdapterError> {
        if auth.mode() != mode {
            return Err(AdapterError::AuthModeMismatch);
        }
        if privacy_data_class_allowlist.is_empty() {
            return Err(AdapterError::MissingDataClassAllowlist);
        }
        if regions_available.is_empty()
            || regions_available
                .iter()
                .any(|region| region.trim().is_empty())
        {
            return Err(AdapterError::MissingProviderRegion);
        }
        Ok(Self {
            id,
            mode,
            auth,
            privacy_data_class_allowlist: Classified::new(
                privacy_data_class_allowlist,
                DataClass::InternalOnly,
            ),
            regions_available: Classified::new(regions_available, DataClass::InternalOnly),
            projected_invocation_cost_micros: Classified::new(
                projected_invocation_cost_micros,
                DataClass::InternalOnly,
            ),
            p95_latency_ms: Classified::new(p95_latency_ms, DataClass::InternalOnly),
        })
    }

    pub fn privacy_data_class_allowlist(&self) -> &[PrivacyDataClass] {
        &self.privacy_data_class_allowlist.value
    }

    /// Legacy provider-config projection for config writers/readers that still
    /// persist raw `DataClass` labels. The profile stores a typed
    /// [`PrivacyDataClass`] allowlist, so the projection is derived from
    /// validated state and remains fail-closed at construction time.
    pub fn legacy_data_class_allowlist(&self) -> Vec<DataClass> {
        data_classes_from_privacy_data_classes(self.privacy_data_class_allowlist())
    }

    #[deprecated(
        note = "use privacy_data_class_allowlist for canonical typed access or legacy_data_class_allowlist for the compatibility projection"
    )]
    pub fn data_class_allowlist(&self) -> Vec<DataClass> {
        self.legacy_data_class_allowlist()
    }
}

impl InvocationPolicy {
    pub fn new(
        tenant_id: Classified<String>,
        allowed_data_classes: Vec<PrivacyDataClass>,
        required_region: Classified<String>,
        ceiling: CostCeiling,
        max_latency_ms: u32,
    ) -> Result<Self, AdapterError> {
        Ok(Self::new_with_privacy_data_classes(
            tenant_id,
            allowed_data_classes,
            required_region,
            ceiling,
            max_latency_ms,
        ))
    }

    /// Compatibility constructor for policy config seams that still carry raw
    /// `DataClass` labels. Canonical invocation policy construction takes
    /// `PrivacyDataClass` and this path fails closed for operational markers
    /// and subject markers.
    pub fn try_from_legacy_allowed_data_classes(
        tenant_id: Classified<String>,
        allowed_data_classes: Vec<DataClass>,
        required_region: Classified<String>,
        ceiling: CostCeiling,
        max_latency_ms: u32,
    ) -> Result<Self, AdapterError> {
        let allowed_privacy_data_classes = privacy_data_classes_from(&allowed_data_classes)
            .map_err(|_| AdapterError::InvalidDataClass)?;
        Self::new(
            tenant_id,
            allowed_privacy_data_classes,
            required_region,
            ceiling,
            max_latency_ms,
        )
    }

    pub fn new_with_privacy_data_classes(
        tenant_id: Classified<String>,
        allowed_privacy_data_classes: Vec<PrivacyDataClass>,
        required_region: Classified<String>,
        ceiling: CostCeiling,
        max_latency_ms: u32,
    ) -> Self {
        Self {
            tenant_id,
            allowed_privacy_data_classes,
            required_region,
            ceiling,
            max_latency_ms,
        }
    }

    pub fn allowed_privacy_data_classes(&self) -> &[PrivacyDataClass] {
        &self.allowed_privacy_data_classes
    }

    /// Legacy invocation-policy projection for config writers/readers that
    /// still persist raw `DataClass` labels. The policy stores typed privacy
    /// classes, so this projection is derived from validated state and cannot
    /// widen policy scope to operational or subject markers.
    pub fn legacy_allowed_data_classes(&self) -> Vec<DataClass> {
        data_classes_from_privacy_data_classes(&self.allowed_privacy_data_classes)
    }

    #[deprecated(
        note = "use allowed_privacy_data_classes for canonical typed access or legacy_allowed_data_classes for the compatibility projection"
    )]
    pub fn allowed_data_classes(&self) -> Vec<DataClass> {
        self.legacy_allowed_data_classes()
    }
}

impl ProviderRoutePreference {
    pub fn ordered(ordered_provider_ids: Vec<ProviderId>) -> Result<Self, AdapterError> {
        if ordered_provider_ids.is_empty() {
            return Err(AdapterError::EmptyFailoverChain);
        }
        Ok(Self {
            ordered_provider_ids: Classified::new(ordered_provider_ids, DataClass::InternalOnly),
        })
    }
}

impl SubscriptionBindingRegistry {
    pub fn bind(
        &mut self,
        tenant_id: String,
        provider_id: ProviderId,
        provider_account: String,
    ) -> Result<SubscriptionBinding, AdapterError> {
        validate_tenant_id(&tenant_id)?;
        validate_provider_account(&provider_account)?;
        let binding = SubscriptionBinding {
            tenant_id: Classified::new(tenant_id, DataClass::InternalOnly),
            provider_id,
            provider_account: Classified::new(provider_account, DataClass::InternalOnly),
        };
        self.bindings.insert(binding.clone());
        Ok(binding)
    }

    pub fn is_active(
        &self,
        tenant_id: &str,
        provider_id: &ProviderId,
        provider_account: &str,
    ) -> bool {
        self.bindings.contains(&SubscriptionBinding {
            tenant_id: Classified::new(tenant_id.to_string(), DataClass::InternalOnly),
            provider_id: provider_id.clone(),
            provider_account: Classified::new(
                provider_account.to_string(),
                DataClass::InternalOnly,
            ),
        })
    }
}

impl CostCeiling {
    pub fn from_budget_snapshot(snapshot: &BudgetSnapshot) -> Self {
        Self {
            monthly_spend_micros: snapshot.running_spend_micros.value,
            monthly_limit_micros: snapshot.ceiling.monthly_limit_micros.value,
            max_invocation_micros: snapshot.ceiling.per_invocation_limit_micros.value,
        }
    }

    fn allows(&self, projected_invocation_cost_micros: u64) -> bool {
        projected_invocation_cost_micros <= self.max_invocation_micros
            && self
                .monthly_spend_micros
                .saturating_add(projected_invocation_cost_micros)
                <= self.monthly_limit_micros
    }
}

impl ProviderRoute {
    pub fn primary(&self) -> Result<&ProviderProfile, AdapterError> {
        self.providers
            .first()
            .ok_or(AdapterError::NoProviderAvailable)
    }
}

impl ProviderCallReceipt {
    pub fn from_route(
        route: &ProviderRoute,
        idempotency_key: String,
        attempt: u32,
        model_ref: String,
        provider_region: String,
    ) -> Result<Self, AdapterError> {
        let provider = route.primary()?;
        if idempotency_key.trim().is_empty() {
            return Err(AdapterError::EmptyProviderCallIdempotencyKey);
        }
        if model_ref.trim().is_empty() {
            return Err(AdapterError::EmptyProviderModelRef);
        }
        if attempt == 0 {
            return Err(AdapterError::InvalidProviderCallAttempt);
        }
        if provider_region.trim().is_empty() {
            return Err(AdapterError::MissingProviderRegion);
        }
        if provider_region != route.selected_region.value {
            return Err(AdapterError::ProviderCallRegionMismatch);
        }
        if !provider
            .regions_available
            .value
            .contains(&route.selected_region.value)
        {
            return Err(AdapterError::MissingProviderRegion);
        }
        Ok(Self {
            provider_id: provider.id.clone(),
            provider_mode: Classified::new(provider.mode, DataClass::InternalOnly),
            receipt_id: Classified::new(
                format!("provider-call-receipt:{idempotency_key}"),
                DataClass::InternalOnly,
            ),
            provider_region: route.selected_region.clone(),
            model_ref: Classified::new(model_ref, DataClass::InternalOnly),
            idempotency_key: Classified::new(idempotency_key, DataClass::InternalOnly),
            attempt: Classified::new(attempt, DataClass::InternalOnly),
            projected_cost_micros: provider.projected_invocation_cost_micros.clone(),
            p95_latency_ms: provider.p95_latency_ms.clone(),
        })
    }
}

impl PromptEnvelope {
    pub fn new(
        request_id: String,
        prompt_ref: String,
        privacy_data_classes: Vec<PrivacyDataClass>,
    ) -> Result<Self, AdapterError> {
        if request_id.trim().is_empty() {
            return Err(AdapterError::EmptyProviderRequestId);
        }
        if prompt_ref.trim().is_empty() {
            return Err(AdapterError::EmptyProviderPromptRef);
        }
        if privacy_data_classes.is_empty() {
            return Err(AdapterError::MissingDataClassAllowlist);
        }
        Ok(Self {
            request_id: Classified::new(request_id, DataClass::InternalOnly),
            prompt_ref: Classified::new(prompt_ref, DataClass::InternalOnly),
            privacy_data_classes: Classified::new(privacy_data_classes, DataClass::InternalOnly),
        })
    }
}

impl ToolSchemaSet {
    pub fn new(tool_names: Vec<String>) -> Result<Self, AdapterError> {
        if tool_names
            .iter()
            .any(|tool_name| tool_name.trim().is_empty())
        {
            return Err(AdapterError::EmptyProviderToolName);
        }
        Ok(Self {
            tool_names: Classified::new(tool_names, DataClass::InternalOnly),
        })
    }
}

impl ProviderInvocation {
    pub fn new(
        prompt: PromptEnvelope,
        tools: ToolSchemaSet,
        idempotency_key: String,
        model_ref: String,
    ) -> Result<Self, AdapterError> {
        if idempotency_key.trim().is_empty() {
            return Err(AdapterError::EmptyProviderCallIdempotencyKey);
        }
        if model_ref.trim().is_empty() {
            return Err(AdapterError::EmptyProviderModelRef);
        }
        Ok(Self {
            prompt,
            tools,
            idempotency_key: Classified::new(idempotency_key, DataClass::InternalOnly),
            model_ref: Classified::new(model_ref, DataClass::InternalOnly),
        })
    }

    fn request_for_provider(
        &self,
        provider_id: ProviderId,
        attempt: u32,
    ) -> Result<ProviderInvocationRequest, AdapterError> {
        if attempt == 0 {
            return Err(AdapterError::InvalidProviderCallAttempt);
        }
        Ok(ProviderInvocationRequest {
            provider_id,
            prompt: self.prompt.clone(),
            tools: self.tools.clone(),
            idempotency_key: self.idempotency_key.clone(),
            model_ref: self.model_ref.clone(),
            attempt: Classified::new(attempt, DataClass::InternalOnly),
        })
    }
}

impl ProviderEvent {
    pub fn stream_start(provider_id: ProviderId, request_id: String) -> Self {
        Self::StreamStart {
            provider_id,
            request_id: Classified::new(request_id, DataClass::InternalOnly),
        }
    }

    pub fn token(text: String) -> Self {
        Self::Token {
            text: Classified::new(text, DataClass::InternalOnly),
        }
    }

    pub fn tool_call(tool_name: String) -> Self {
        Self::ToolCall {
            tool_name: Classified::new(tool_name, DataClass::InternalOnly),
        }
    }

    pub fn usage(input_tokens: u32, output_tokens: u32, cost_micros: u64) -> Self {
        Self::Usage {
            input_tokens: Classified::new(input_tokens, DataClass::InternalOnly),
            output_tokens: Classified::new(output_tokens, DataClass::InternalOnly),
            cost_micros: Classified::new(cost_micros, DataClass::InternalOnly),
        }
    }

    pub fn stream_end(reason: ProviderStreamEndReason) -> Self {
        Self::StreamEnd {
            reason: Classified::new(reason, DataClass::InternalOnly),
        }
    }

    pub fn error(kind: ProviderFailureKind, message: String) -> Self {
        Self::Error {
            kind: Classified::new(kind, DataClass::InternalOnly),
            message: Classified::new(message, DataClass::InternalOnly),
        }
    }

    pub fn response_restart(
        from_provider_id: ProviderId,
        to_provider_id: ProviderId,
        reason: String,
    ) -> Self {
        Self::ResponseRestart {
            from_provider_id,
            to_provider_id,
            reason: Classified::new(reason, DataClass::InternalOnly),
        }
    }

    fn terminal_failure_kind(&self) -> Option<ProviderFailureKind> {
        match self {
            Self::StreamEnd { .. } => None,
            Self::Error { kind, .. } => Some(kind.value),
            _ => None,
        }
    }

    fn is_terminal(&self) -> bool {
        matches!(self, Self::StreamEnd { .. } | Self::Error { .. })
    }
}

pub fn invoke_provider_route(
    route: &ProviderRoute,
    adapters: &[&dyn ProviderAdapter],
    invocation: ProviderInvocation,
) -> Result<ProviderInvocationTrace, AdapterError> {
    let mut attempts = Vec::new();
    let mut failover_events = Vec::new();
    let mut pending_restart_from = None;
    let mut attempt_number = 1;
    let mut saw_route_provider = false;

    for profile in &route.providers {
        let Some(adapter) = adapters
            .iter()
            .copied()
            .find(|adapter| adapter.profile().id == profile.id)
        else {
            continue;
        };
        saw_route_provider = true;
        if adapter.profile().mode != profile.mode {
            return Err(AdapterError::ProviderAdapterMismatch);
        }
        if let Some(from_provider_id) = pending_restart_from.take() {
            failover_events.push(ProviderEvent::response_restart(
                from_provider_id,
                profile.id.clone(),
                "retryable_provider_error".to_string(),
            ));
        }

        let request = invocation.request_for_provider(profile.id.clone(), attempt_number)?;
        let events = adapter.invoke(request)?;
        let terminal = validate_provider_event_sequence(
            &profile.id,
            &invocation.prompt.request_id.value,
            &events,
        )?;
        match terminal {
            ProviderTerminal::Success => {
                attempts.push(ProviderAttemptTrace {
                    provider_id: profile.id.clone(),
                    attempt: Classified::new(attempt_number, DataClass::InternalOnly),
                    events: Classified::new(events.clone(), DataClass::InternalOnly),
                    discarded_response: Classified::new(false, DataClass::InternalOnly),
                });
                return Ok(ProviderInvocationTrace {
                    attempts,
                    failover_events,
                    final_provider_id: profile.id.clone(),
                    final_events: Classified::new(events, DataClass::InternalOnly),
                });
            }
            ProviderTerminal::RetryableFailure => {
                attempts.push(ProviderAttemptTrace {
                    provider_id: profile.id.clone(),
                    attempt: Classified::new(attempt_number, DataClass::InternalOnly),
                    events: Classified::new(events, DataClass::InternalOnly),
                    discarded_response: Classified::new(true, DataClass::InternalOnly),
                });
                pending_restart_from = Some(profile.id.clone());
                attempt_number += 1;
            }
            ProviderTerminal::NonRetryableFailure => {
                attempts.push(ProviderAttemptTrace {
                    provider_id: profile.id.clone(),
                    attempt: Classified::new(attempt_number, DataClass::InternalOnly),
                    events: Classified::new(events, DataClass::InternalOnly),
                    discarded_response: Classified::new(false, DataClass::InternalOnly),
                });
                return Err(AdapterError::ProviderNonRetryableFailure);
            }
        }
    }

    if saw_route_provider {
        Err(AdapterError::ProviderRetryableFailure)
    } else {
        Err(AdapterError::NoProviderAvailable)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProviderTerminal {
    Success,
    RetryableFailure,
    NonRetryableFailure,
}

fn validate_provider_event_sequence(
    provider_id: &ProviderId,
    request_id: &str,
    events: &[ProviderEvent],
) -> Result<ProviderTerminal, AdapterError> {
    let Some(first) = events.first() else {
        return Err(AdapterError::InvalidProviderEventSequence);
    };
    match first {
        ProviderEvent::StreamStart {
            provider_id: event_provider_id,
            request_id: event_request_id,
        } if event_provider_id == provider_id && event_request_id.value == request_id => {}
        _ => return Err(AdapterError::InvalidProviderEventSequence),
    }

    let Some((terminal_index, terminal_event)) = events
        .iter()
        .enumerate()
        .find(|(_, event)| event.is_terminal())
    else {
        return Err(AdapterError::InvalidProviderEventSequence);
    };
    if terminal_index != events.len().saturating_sub(1) {
        return Err(AdapterError::InvalidProviderEventSequence);
    }
    if events
        .iter()
        .skip(1)
        .take(terminal_index.saturating_sub(1))
        .any(|event| {
            matches!(
                event,
                ProviderEvent::StreamStart { .. } | ProviderEvent::ResponseRestart { .. }
            )
        })
    {
        return Err(AdapterError::InvalidProviderEventSequence);
    }

    match terminal_event.terminal_failure_kind() {
        None => Ok(ProviderTerminal::Success),
        Some(ProviderFailureKind::Retryable) => Ok(ProviderTerminal::RetryableFailure),
        Some(ProviderFailureKind::NonRetryable) => Ok(ProviderTerminal::NonRetryableFailure),
    }
}

pub fn resolve_route(request: ProviderRouteRequest<'_>) -> Result<ProviderRoute, AdapterError> {
    validate_tenant_id(&request.policy.tenant_id.value)?;
    if request.policy.ceiling.monthly_limit_micros == 0 {
        return Err(AdapterError::InvalidCostCeiling);
    }
    if request.policy.required_region.value.trim().is_empty() {
        return Err(AdapterError::InvalidRequiredRegion);
    }
    let capability_data_classes = request.capability.touched_privacy_data_classes();
    if !capability_data_classes.iter().all(|data_class| {
        request
            .policy
            .allowed_privacy_data_classes
            .contains(data_class)
    }) {
        return Err(AdapterError::DataClassNotAllowed);
    }

    let mut providers = Vec::new();
    for desired_provider in &request.preference.ordered_provider_ids.value {
        let Some(profile) = request
            .profiles
            .iter()
            .find(|profile| profile.id == *desired_provider)
        else {
            continue;
        };
        if !capability_data_classes.iter().all(|data_class| {
            profile
                .privacy_data_class_allowlist
                .value
                .contains(data_class)
        }) {
            continue;
        }
        if !profile
            .regions_available
            .value
            .contains(&request.policy.required_region.value)
        {
            continue;
        }
        if !request
            .policy
            .ceiling
            .allows(profile.projected_invocation_cost_micros.value)
        {
            continue;
        }
        if profile.p95_latency_ms.value > request.policy.max_latency_ms {
            continue;
        }
        if profile.mode == ProviderMode::Subscription
            && !request.subscription_bindings.is_active(
                &request.policy.tenant_id.value,
                &profile.id,
                profile.auth.provider_account(),
            )
        {
            continue;
        }
        providers.push(profile.clone());
    }

    if providers.is_empty() {
        return Err(AdapterError::NoProviderAvailable);
    }
    Ok(ProviderRoute {
        providers,
        selected_region: request.policy.required_region,
    })
}

fn validate_tenant_id(tenant_id: &str) -> Result<(), AdapterError> {
    if !tenant_id.starts_with("ten_") {
        return Err(AdapterError::InvalidTenantId);
    }
    Ok(())
}

fn validate_provider_account(provider_account: &str) -> Result<(), AdapterError> {
    if provider_account.trim().is_empty() {
        return Err(AdapterError::EmptyProviderAccount);
    }
    Ok(())
}
