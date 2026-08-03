//! Payments charge-BC domain — `Charge` aggregate, invariants, and
//! `ChargeRepository` port.
//!
//! Wave 15-IMPL-truth-up scaffold; full state-machine + COPPA invariant
//! implementation in IP-002. Zero I/O. Domain events emitted through the
//! `DomainEventEnvelope` shape declared here.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use oya_payments_charge_kernel::{ChargeId, ChargeState};

/// Charge aggregate root. State-machine guards land in IP-002.
#[allow(dead_code)]
pub struct Charge {
    id: ChargeId,
    state: ChargeState,
}

/// Repository port. Adapter impls land in IP-004 / IP-018 lanes.
pub trait ChargeRepository {
    type Error;
    fn save(&self, charge: &Charge) -> Result<(), Self::Error>;
}

const TENANT_ID_PREFIX: &str = "ten_";
const PROD_POLICY_MARKER: &str = "env-tier";
const QA_DESTINATION_MARKERS: [&str; 6] =
    ["/qa/", "/sandbox/", "/test/", "qa-", "sandbox-", "test-"];
const SECRET_MARKERS: [&str; 9] = [
    "secret",
    "token",
    "credential",
    "bearer",
    "api_key",
    "api-key",
    "private-key",
    "private_key",
    "password",
];
const API_KEY_PREFIXES: [&str; 3] = ["sk_test_", "sk_stage_", "sk_live_"];

/// Canonical tenant environment tier for metadata-only payment emission plans.
/// The serialized fixture field is `env_tier`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum PaymentEnvTier {
    Test,
    Staging,
    Prod,
}

impl PaymentEnvTier {
    /// Returns the only allowed derived `outbound_mode` for this tier.
    #[must_use]
    pub const fn derived_outbound_mode(self) -> PaymentOutboundMode {
        match self {
            Self::Test => PaymentOutboundMode::Intercept,
            Self::Staging => PaymentOutboundMode::TestRecipients,
            Self::Prod => PaymentOutboundMode::Live,
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

    #[must_use]
    pub const fn expected_api_key_prefix(self) -> &'static str {
        match self {
            Self::Test => "sk_test_",
            Self::Staging => "sk_stage_",
            Self::Prod => "sk_live_",
        }
    }
}

/// Derived outbound behavior. The serialized fixture field is `outbound_mode`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum PaymentOutboundMode {
    /// test => intercept/log-only; no PSP, webhook, invoice, or delivery side effect.
    Intercept,
    /// staging => test_recipients; tenant QA PSP sandbox/webhook endpoint only.
    TestRecipients,
    /// prod => live; requires env-tier policy evidence plus production acknowledgments.
    Live,
}

impl PaymentOutboundMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Intercept => "intercept",
            Self::TestRecipients => "test_recipients",
            Self::Live => "live",
        }
    }
}

/// Payments action classes covered by the metadata-only guardrail.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum PaymentActionClass {
    ChargeCapture,
    Refund,
    Payout,
    Subscription,
    Invoice,
    WebhookDelivery,
}

/// Raw create shape used by RED fixtures before validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentOutboundEmissionPlanCreate {
    pub tenant_id: String,                            // data_class: INTERNAL_ONLY
    pub env_tier: Option<PaymentEnvTier>,             // data_class: INTERNAL_ONLY
    pub outbound_mode: PaymentOutboundMode, // data_class: INTERNAL_ONLY; derived from env_tier
    pub payment_action_class: PaymentActionClass, // data_class: INTERNAL_ONLY
    pub destination_binding_ref: Option<String>, // data_class: INTERNAL_ONLY; metadata ref only
    pub pci_safe_evidence_ref: String,      // data_class: AUDIT; never raw PSP credentials
    pub tenancy_cedar_policy_evidence_ref: String, // data_class: AUDIT
    pub financial_acknowledgment_ref: Option<String>, // data_class: AUDIT
    pub prod_acknowledgment_ref: Option<String>, // data_class: AUDIT
    pub api_key_prefix_evidence_ref: Option<String>, // data_class: AUDIT; prefix evidence only
}

/// Validated metadata-only plan. This is not a runtime delivery or PSP call claim.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentOutboundEmissionPlan {
    pub tenant_id: String,                            // data_class: INTERNAL_ONLY
    pub env_tier: PaymentEnvTier,                     // data_class: INTERNAL_ONLY
    pub outbound_mode: PaymentOutboundMode,           // data_class: INTERNAL_ONLY
    pub payment_action_class: PaymentActionClass,     // data_class: INTERNAL_ONLY
    pub destination_binding_ref: Option<String>,      // data_class: INTERNAL_ONLY
    pub pci_safe_evidence_ref: String,                // data_class: AUDIT
    pub tenancy_cedar_policy_evidence_ref: String,    // data_class: AUDIT
    pub financial_acknowledgment_ref: Option<String>, // data_class: AUDIT
    pub prod_acknowledgment_ref: Option<String>,      // data_class: AUDIT
    pub api_key_prefix_evidence_ref: Option<String>,  // data_class: AUDIT
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum PaymentOutboundMetadataError {
    MissingEnvTier,
    InvalidOutboundModeForTier,
    ExternalSideEffectNotAllowedForTier,
    MissingQaDestination,
    ProdPolicyEvidenceRequired,
    ProdAcknowledgmentRequired,
    ApiKeyPrefixTierMismatch,
    InvalidApiKeyPrefixEvidence,
    TenantMismatch,
    RawSecretOrCredentialInFixture,
}

impl PaymentOutboundEmissionPlan {
    /// Validates the metadata contract without performing I/O, delivery, PSP calls,
    /// webhook delivery, invoice generation, or production side effects.
    pub fn new(
        create: PaymentOutboundEmissionPlanCreate,
    ) -> Result<Self, PaymentOutboundMetadataError> {
        let env_tier = create
            .env_tier
            .ok_or(PaymentOutboundMetadataError::MissingEnvTier)?;

        if create.outbound_mode != env_tier.derived_outbound_mode() {
            return Err(PaymentOutboundMetadataError::InvalidOutboundModeForTier);
        }

        validate_no_raw_secret_or_credential(create.destination_binding_ref.as_deref())?;
        validate_no_raw_secret_or_credential(Some(create.pci_safe_evidence_ref.as_str()))?;
        validate_no_raw_secret_or_credential(Some(
            create.tenancy_cedar_policy_evidence_ref.as_str(),
        ))?;
        validate_no_raw_secret_or_credential(create.financial_acknowledgment_ref.as_deref())?;
        validate_no_raw_secret_or_credential(create.prod_acknowledgment_ref.as_deref())?;
        validate_api_key_prefix(
            env_tier,
            create.tenant_id.as_str(),
            create.api_key_prefix_evidence_ref.as_deref(),
        )?;

        match env_tier {
            PaymentEnvTier::Test => {
                if create.destination_binding_ref.is_some() {
                    return Err(PaymentOutboundMetadataError::ExternalSideEffectNotAllowedForTier);
                }
            }
            PaymentEnvTier::Staging => {
                let destination = create
                    .destination_binding_ref
                    .as_deref()
                    .ok_or(PaymentOutboundMetadataError::MissingQaDestination)?;
                validate_tenant_scoped_ref(create.tenant_id.as_str(), destination)?;
                if !has_qa_destination_marker(destination) {
                    return Err(PaymentOutboundMetadataError::MissingQaDestination);
                }
            }
            PaymentEnvTier::Prod => {
                let destination = create
                    .destination_binding_ref
                    .as_deref()
                    .ok_or(PaymentOutboundMetadataError::MissingQaDestination)?;
                validate_tenant_scoped_ref(create.tenant_id.as_str(), destination)?;
                validate_prod_policy_evidence(
                    create.tenant_id.as_str(),
                    create.tenancy_cedar_policy_evidence_ref.as_str(),
                )?;
                validate_prod_acknowledgment(
                    create.tenant_id.as_str(),
                    create.financial_acknowledgment_ref.as_deref(),
                    "financial",
                )?;
                validate_prod_acknowledgment(
                    create.tenant_id.as_str(),
                    create.prod_acknowledgment_ref.as_deref(),
                    "prod",
                )?;
            }
        }

        Ok(Self {
            tenant_id: create.tenant_id,
            env_tier,
            outbound_mode: create.outbound_mode,
            payment_action_class: create.payment_action_class,
            destination_binding_ref: create.destination_binding_ref,
            pci_safe_evidence_ref: create.pci_safe_evidence_ref,
            tenancy_cedar_policy_evidence_ref: create.tenancy_cedar_policy_evidence_ref,
            financial_acknowledgment_ref: create.financial_acknowledgment_ref,
            prod_acknowledgment_ref: create.prod_acknowledgment_ref,
            api_key_prefix_evidence_ref: create.api_key_prefix_evidence_ref,
        })
    }
}

fn validate_no_raw_secret_or_credential(
    value: Option<&str>,
) -> Result<(), PaymentOutboundMetadataError> {
    let Some(value) = value else {
        return Ok(());
    };
    let normalized = value.to_ascii_lowercase();
    if SECRET_MARKERS
        .iter()
        .any(|marker| normalized.contains(marker))
        || API_KEY_PREFIXES.iter().any(|prefix| value.contains(prefix))
    {
        return Err(PaymentOutboundMetadataError::RawSecretOrCredentialInFixture);
    }
    Ok(())
}

fn validate_api_key_prefix(
    env_tier: PaymentEnvTier,
    tenant_id: &str,
    value: Option<&str>,
) -> Result<(), PaymentOutboundMetadataError> {
    let Some(value) = value else {
        return Ok(());
    };

    validate_tenant_scoped_ref(tenant_id, value)?;
    if raw_key_material_after_allowed_prefix(value) {
        return Err(PaymentOutboundMetadataError::RawSecretOrCredentialInFixture);
    }

    let found_prefix = API_KEY_PREFIXES
        .iter()
        .copied()
        .find(|prefix| value.contains(prefix))
        .ok_or(PaymentOutboundMetadataError::InvalidApiKeyPrefixEvidence)?;

    if found_prefix != env_tier.expected_api_key_prefix() {
        return Err(PaymentOutboundMetadataError::ApiKeyPrefixTierMismatch);
    }

    Ok(())
}

fn raw_key_material_after_allowed_prefix(value: &str) -> bool {
    API_KEY_PREFIXES.iter().any(|prefix| {
        value.find(prefix).is_some_and(|index| {
            value[index + prefix.len()..]
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_alphanumeric() || character == '_')
        })
    })
}

fn validate_tenant_scoped_ref(
    tenant_id: &str,
    value: &str,
) -> Result<(), PaymentOutboundMetadataError> {
    if !tenant_id.starts_with(TENANT_ID_PREFIX) || !value.contains(tenant_id) {
        return Err(PaymentOutboundMetadataError::TenantMismatch);
    }
    Ok(())
}

fn has_qa_destination_marker(destination: &str) -> bool {
    let normalized = destination.to_ascii_lowercase();
    QA_DESTINATION_MARKERS
        .iter()
        .any(|marker| normalized.contains(marker))
}

fn validate_prod_policy_evidence(
    tenant_id: &str,
    policy_evidence_ref: &str,
) -> Result<(), PaymentOutboundMetadataError> {
    let normalized = policy_evidence_ref.to_ascii_lowercase();
    if !policy_evidence_ref.contains(tenant_id)
        || !normalized.contains(PaymentEnvTier::Prod.as_str())
        || !normalized.contains(PROD_POLICY_MARKER)
        || normalized.contains("missing")
    {
        return Err(PaymentOutboundMetadataError::ProdPolicyEvidenceRequired);
    }
    Ok(())
}

fn validate_prod_acknowledgment(
    tenant_id: &str,
    acknowledgment_ref: Option<&str>,
    required_marker: &str,
) -> Result<(), PaymentOutboundMetadataError> {
    let Some(acknowledgment_ref) = acknowledgment_ref else {
        return Err(PaymentOutboundMetadataError::ProdAcknowledgmentRequired);
    };
    let normalized = acknowledgment_ref.to_ascii_lowercase();
    if !acknowledgment_ref.contains(tenant_id)
        || !normalized.contains(PaymentEnvTier::Prod.as_str())
        || !normalized.contains(required_marker)
        || normalized.contains("missing")
    {
        return Err(PaymentOutboundMetadataError::ProdAcknowledgmentRequired);
    }
    Ok(())
}
