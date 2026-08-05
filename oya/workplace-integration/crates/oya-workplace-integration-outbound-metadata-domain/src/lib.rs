//! Metadata-only workplace-integration outbound env-tier guardrails.
//!
//! This crate intentionally owns no e-sign provider call, workplace external
//! delivery, payment execution, webhook delivery, production traffic, runtime
//! audit-chain writer, worker, persistence, or provider adapter. It only models
//! the fail-closed metadata contract for future workplace/e-sign/offer/roster
//! and payment-adjacent integration emission plans.
#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::expect_used, clippy::panic, clippy::unwrap_used))]

const TENANT_ID_PREFIX: &str = "ten_";
const DESTINATION_BINDING_NAMESPACE: &str = "workplace-integration-destination";
const POLICY_EVIDENCE_NAMESPACE: &str = "policy-evidence/workplace-integration";
const TENANCY_EVIDENCE_NAMESPACE: &str = "tenancy-evidence/workplace-integration";
const PROD_TIER_MARKER: &str = "prod";
const ENV_TIER_MARKER: &str = "env-tier";
const SECRET_MARKERS: [&str; 15] = [
    "secret",
    "token",
    "credential",
    "bearer",
    "authorization:",
    "api_key",
    "api-key",
    "private-key",
    "private_key",
    "password",
    "client_secret",
    "webhook-secret",
    "signing-secret",
    "provider-key",
    "sk_",
];
const CONSENT_OR_AUTHORIZATION_MARKERS: [&str; 3] = ["consent", "authorization", "policy"];

/// Canonical tenant environment tier for metadata-only workplace integration plans.
/// The serialized fixture field is `env_tier`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum WorkplaceEnvTier {
    Test,
    Staging,
    Prod,
}

impl WorkplaceEnvTier {
    /// Returns the only allowed derived `outbound_mode` for this tier.
    #[must_use]
    pub const fn derived_outbound_mode(self) -> WorkplaceOutboundMode {
        match self {
            Self::Test => WorkplaceOutboundMode::Intercept,
            Self::Staging => WorkplaceOutboundMode::TestRecipients,
            Self::Prod => WorkplaceOutboundMode::Live,
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Test => "test",
            Self::Staging => "staging",
            Self::Prod => "prod",
        }
    }
}

/// Derived outbound behavior. The serialized fixture field is `outbound_mode`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum WorkplaceOutboundMode {
    /// test => intercept/log-only; no e-sign, workplace, payment, or webhook side effect.
    Intercept,
    /// staging => test_recipients; tenant QA recipient/endpoint only.
    TestRecipients,
    /// prod => live metadata only with tenancy/env-tier policy plus consent evidence.
    Live,
}

impl WorkplaceOutboundMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Intercept => "intercept",
            Self::TestRecipients => "test_recipients",
            Self::Live => "live",
        }
    }
}

/// Workplace/e-sign/offer/roster/payment-adjacent action classes covered by the guardrail.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum WorkplaceIntegrationActionClass {
    ESignSessionInitiation,
    ESignSignatureCapture,
    OfferGeneration,
    RosterBinding,
    WorkplaceExternalNotification,
    PaymentAdjacentWebhook,
}

/// Raw create shape used by RED fixtures before validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkplaceOutboundEmissionPlanCreate {
    pub tenant_id: String,                             // data_class: INTERNAL_ONLY
    pub env_tier: Option<WorkplaceEnvTier>,            // data_class: INTERNAL_ONLY
    pub outbound_mode: WorkplaceOutboundMode, // data_class: INTERNAL_ONLY; derived from env_tier
    pub action_class: WorkplaceIntegrationActionClass, // data_class: INTERNAL_ONLY
    pub destination_binding_ref: Option<String>, // data_class: INTERNAL_ONLY; metadata ref only
    pub consent_policy_evidence_ref: String, // data_class: AUDIT; consent/authorization/policy ref only
    pub tenancy_env_tier_evidence_ref: String, // data_class: AUDIT; tenancy/env-tier ref only
    pub runtime_delivery_authorized: bool, // data_class: INTERNAL_ONLY; must remain false in this lane
}

/// Validated metadata-only plan. This is not a runtime delivery, e-sign, payment, or webhook claim.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkplaceOutboundEmissionPlan {
    pub tenant_id: String,                             // data_class: INTERNAL_ONLY
    pub env_tier: WorkplaceEnvTier,                    // data_class: INTERNAL_ONLY
    pub outbound_mode: WorkplaceOutboundMode,          // data_class: INTERNAL_ONLY
    pub action_class: WorkplaceIntegrationActionClass, // data_class: INTERNAL_ONLY
    pub destination_binding_ref: Option<String>,       // data_class: INTERNAL_ONLY
    pub consent_policy_evidence_ref: String,           // data_class: AUDIT
    pub tenancy_env_tier_evidence_ref: String,         // data_class: AUDIT
    pub runtime_delivery_authorized: bool,             // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum WorkplaceOutboundMetadataError {
    MissingEnvTier,
    InvalidOutboundModeForTier,
    TestTierExternalSideEffectForbidden,
    MissingQaDestination,
    DestinationBindingRequired,
    InvalidDestinationBindingForTier,
    InvalidEvidenceRefForTier,
    ProdTenancyEnvTierEvidenceRequired,
    ProdConsentAuthorizationEvidenceRequired,
    TenantMismatch,
    RawSecretOrCredentialInFixture,
    RuntimeDeliveryClaimForbidden,
}

impl WorkplaceOutboundEmissionPlan {
    /// Validates the metadata contract without performing I/O, provider calls,
    /// webhook delivery, payment execution, production traffic, or audit-chain emission.
    pub fn new(
        create: WorkplaceOutboundEmissionPlanCreate,
    ) -> Result<Self, WorkplaceOutboundMetadataError> {
        let env_tier = create
            .env_tier
            .ok_or(WorkplaceOutboundMetadataError::MissingEnvTier)?;

        if create.outbound_mode != env_tier.derived_outbound_mode() {
            return Err(WorkplaceOutboundMetadataError::InvalidOutboundModeForTier);
        }

        validate_no_raw_secret_or_credential(create.destination_binding_ref.as_deref())?;
        validate_no_raw_secret_or_credential(Some(create.consent_policy_evidence_ref.as_str()))?;
        validate_no_raw_secret_or_credential(Some(create.tenancy_env_tier_evidence_ref.as_str()))?;
        validate_tenant_scoped_ref(
            create.tenant_id.as_str(),
            create.consent_policy_evidence_ref.as_str(),
        )?;
        validate_tenant_scoped_ref(
            create.tenant_id.as_str(),
            create.tenancy_env_tier_evidence_ref.as_str(),
        )?;
        validate_evidence_ref_prefixes(
            create.tenant_id.as_str(),
            env_tier,
            create.consent_policy_evidence_ref.as_str(),
            create.tenancy_env_tier_evidence_ref.as_str(),
        )?;

        if create.runtime_delivery_authorized {
            return Err(WorkplaceOutboundMetadataError::RuntimeDeliveryClaimForbidden);
        }

        match env_tier {
            WorkplaceEnvTier::Test => {
                if create.destination_binding_ref.is_some() {
                    return Err(
                        WorkplaceOutboundMetadataError::TestTierExternalSideEffectForbidden,
                    );
                }
            }
            WorkplaceEnvTier::Staging => {
                let destination = create
                    .destination_binding_ref
                    .as_deref()
                    .ok_or(WorkplaceOutboundMetadataError::MissingQaDestination)?;
                validate_staging_destination_binding_ref(create.tenant_id.as_str(), destination)?;
            }
            WorkplaceEnvTier::Prod => {
                let destination = create
                    .destination_binding_ref
                    .as_deref()
                    .ok_or(WorkplaceOutboundMetadataError::DestinationBindingRequired)?;
                validate_prod_destination_binding_ref(create.tenant_id.as_str(), destination)?;
                validate_prod_tenancy_env_tier_evidence(
                    create.tenant_id.as_str(),
                    create.tenancy_env_tier_evidence_ref.as_str(),
                )?;
                validate_prod_consent_authorization_evidence(
                    create.tenant_id.as_str(),
                    create.consent_policy_evidence_ref.as_str(),
                )?;
            }
        }

        Ok(Self {
            tenant_id: create.tenant_id,
            env_tier,
            outbound_mode: create.outbound_mode,
            action_class: create.action_class,
            destination_binding_ref: create.destination_binding_ref,
            consent_policy_evidence_ref: create.consent_policy_evidence_ref,
            tenancy_env_tier_evidence_ref: create.tenancy_env_tier_evidence_ref,
            runtime_delivery_authorized: create.runtime_delivery_authorized,
        })
    }
}

fn validate_no_raw_secret_or_credential(
    value: Option<&str>,
) -> Result<(), WorkplaceOutboundMetadataError> {
    let Some(value) = value else {
        return Ok(());
    };
    let normalized = value.to_ascii_lowercase();
    if SECRET_MARKERS
        .iter()
        .any(|marker| normalized.contains(marker))
    {
        return Err(WorkplaceOutboundMetadataError::RawSecretOrCredentialInFixture);
    }
    Ok(())
}

fn validate_tenant_scoped_ref(
    tenant_id: &str,
    value: &str,
) -> Result<(), WorkplaceOutboundMetadataError> {
    if !tenant_id.starts_with(TENANT_ID_PREFIX) || !value.contains(tenant_id) {
        return Err(WorkplaceOutboundMetadataError::TenantMismatch);
    }
    Ok(())
}

fn validate_destination_tenant_scoped_ref(
    tenant_id: &str,
    destination: &str,
) -> Result<(), WorkplaceOutboundMetadataError> {
    if !tenant_id.starts_with(TENANT_ID_PREFIX) {
        return Err(WorkplaceOutboundMetadataError::TenantMismatch);
    }
    let tenant_destination_prefix = format!("{DESTINATION_BINDING_NAMESPACE}/{tenant_id}/");
    if !destination.starts_with(tenant_destination_prefix.as_str()) {
        return Err(WorkplaceOutboundMetadataError::TenantMismatch);
    }
    Ok(())
}

fn validate_staging_destination_binding_ref(
    tenant_id: &str,
    destination: &str,
) -> Result<(), WorkplaceOutboundMetadataError> {
    validate_destination_tenant_scoped_ref(tenant_id, destination)?;
    let allowed_prefixes = [
        format!("{DESTINATION_BINDING_NAMESPACE}/{tenant_id}/staging/qa/"),
        format!("{DESTINATION_BINDING_NAMESPACE}/{tenant_id}/staging/test/"),
        format!("{DESTINATION_BINDING_NAMESPACE}/{tenant_id}/staging/sandbox/"),
    ];
    if !allowed_prefixes
        .iter()
        .any(|prefix| destination.starts_with(prefix.as_str()))
    {
        return Err(WorkplaceOutboundMetadataError::InvalidDestinationBindingForTier);
    }
    Ok(())
}

fn validate_prod_destination_binding_ref(
    tenant_id: &str,
    destination: &str,
) -> Result<(), WorkplaceOutboundMetadataError> {
    validate_destination_tenant_scoped_ref(tenant_id, destination)?;
    let allowed_prefix = format!("{DESTINATION_BINDING_NAMESPACE}/{tenant_id}/prod/live/");
    if !destination.starts_with(allowed_prefix.as_str()) {
        return Err(WorkplaceOutboundMetadataError::InvalidDestinationBindingForTier);
    }
    Ok(())
}

fn validate_evidence_ref_prefixes(
    tenant_id: &str,
    env_tier: WorkplaceEnvTier,
    consent_policy_evidence_ref: &str,
    tenancy_env_tier_evidence_ref: &str,
) -> Result<(), WorkplaceOutboundMetadataError> {
    validate_tiered_evidence_ref(
        POLICY_EVIDENCE_NAMESPACE,
        tenant_id,
        env_tier,
        consent_policy_evidence_ref,
        false,
    )?;
    validate_tiered_evidence_ref(
        TENANCY_EVIDENCE_NAMESPACE,
        tenant_id,
        env_tier,
        tenancy_env_tier_evidence_ref,
        true,
    )
}

fn validate_tiered_evidence_ref(
    namespace: &str,
    tenant_id: &str,
    env_tier: WorkplaceEnvTier,
    value: &str,
    requires_env_tier_segment: bool,
) -> Result<(), WorkplaceOutboundMetadataError> {
    if !tenant_id.starts_with(TENANT_ID_PREFIX) {
        return Err(WorkplaceOutboundMetadataError::TenantMismatch);
    }

    let tenant_prefix = format!("{namespace}/{tenant_id}/");
    if !value.starts_with(tenant_prefix.as_str()) {
        return Err(WorkplaceOutboundMetadataError::TenantMismatch);
    }

    let tier_prefix = if requires_env_tier_segment {
        format!("{tenant_prefix}{}/env-tier/", env_tier.as_str())
    } else {
        format!("{tenant_prefix}{}/", env_tier.as_str())
    };
    if !value.starts_with(tier_prefix.as_str()) {
        return Err(WorkplaceOutboundMetadataError::InvalidEvidenceRefForTier);
    }

    Ok(())
}

fn validate_prod_tenancy_env_tier_evidence(
    tenant_id: &str,
    tenancy_env_tier_evidence_ref: &str,
) -> Result<(), WorkplaceOutboundMetadataError> {
    let normalized = tenancy_env_tier_evidence_ref.to_ascii_lowercase();
    if !tenancy_env_tier_evidence_ref.contains(tenant_id)
        || !normalized.contains(PROD_TIER_MARKER)
        || !normalized.contains(ENV_TIER_MARKER)
        || normalized.contains("missing")
    {
        return Err(WorkplaceOutboundMetadataError::ProdTenancyEnvTierEvidenceRequired);
    }
    Ok(())
}

fn validate_prod_consent_authorization_evidence(
    tenant_id: &str,
    consent_policy_evidence_ref: &str,
) -> Result<(), WorkplaceOutboundMetadataError> {
    let normalized = consent_policy_evidence_ref.to_ascii_lowercase();
    if !consent_policy_evidence_ref.contains(tenant_id)
        || !normalized.contains(PROD_TIER_MARKER)
        || normalized.contains("missing")
        || !CONSENT_OR_AUTHORIZATION_MARKERS
            .iter()
            .any(|marker| normalized.contains(marker))
    {
        return Err(WorkplaceOutboundMetadataError::ProdConsentAuthorizationEvidenceRequired);
    }
    Ok(())
}
