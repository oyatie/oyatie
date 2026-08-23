//! Code-backed safety, evidence, and redaction primitives for intelligence-app.
//!
//! This module is deliberately pure kernel code: it defines platform-invariant
//! policy decisions without binding to any concrete policy engine, secret
//! provider, review queue, or model adapter.

use std::time::Duration;

/// Platform-owned evidence/data taxonomy. Tenant labels must map into these
/// classes before enforcement; tenants may tighten but not weaken the platform
/// floor.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum EvidenceDataClass {
    PublicOrOperationalMetadata,
    PersonalData,
    RegulatedHealthData,
    PaymentOrFinancialData,
    CredentialOrSecret,
    TenantBoundaryViolation,
    PromptInjectionOrJailbreak,
    SecurityExploitOrBreach,
    SelfHarmOrHarmToOthers,
    ChildSafetyOrAbuse,
    FraudOrHostilePattern,
    FaultOrAnomaly,
}

impl EvidenceDataClass {
    const fn sensitivity_rank(self) -> u8 {
        match self {
            Self::PublicOrOperationalMetadata => 0,
            Self::PersonalData => 10,
            Self::FaultOrAnomaly => 15,
            Self::FraudOrHostilePattern => 30,
            Self::PromptInjectionOrJailbreak => 40,
            Self::RegulatedHealthData => 50,
            Self::PaymentOrFinancialData => 50,
            Self::SelfHarmOrHarmToOthers => 55,
            Self::TenantBoundaryViolation => 60,
            Self::SecurityExploitOrBreach => 70,
            Self::CredentialOrSecret => 80,
            Self::ChildSafetyOrAbuse => 90,
        }
    }

    pub const fn requires_blocking_in_transit(self) -> bool {
        matches!(
            self,
            Self::RegulatedHealthData
                | Self::PaymentOrFinancialData
                | Self::CredentialOrSecret
                | Self::TenantBoundaryViolation
                | Self::SecurityExploitOrBreach
                | Self::SelfHarmOrHarmToOthers
                | Self::ChildSafetyOrAbuse
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum CriticalSafetyCategory {
    PromptInjectionOrJailbreak,
    DataExfiltrationOrBreach,
    CredentialOrSecretProbe,
    SandboxEscapeOrDestructiveAction,
    SelfHarmOrHarmToOthers,
    PrivacyViolation,
    TenantBoundaryViolation,
    FraudOrHostilePattern,
    FaultOrAnomaly,
    UnsafeScheduledOrDelegatedExecution,
    ChildSafetyOrAbuse,
    SecurityExploitOrBreach,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SafetySignal {
    pub category: CriticalSafetyCategory, // data_class: INTERNAL_ONLY
    pub data_class: EvidenceDataClass,    // data_class: INTERNAL_ONLY
    pub critical: bool,                   // data_class: INTERNAL_ONLY
}

impl SafetySignal {
    pub const fn critical(category: CriticalSafetyCategory, data_class: EvidenceDataClass) -> Self {
        Self {
            category,
            data_class,
            critical: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManualReviewState {
    NotRequired,
    Required,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SafetyEnforcementDecision {
    pub blocked: bool,                           // data_class: INTERNAL_ONLY
    pub quarantined: bool,                       // data_class: INTERNAL_ONLY
    pub secondary_agentic_review_required: bool, // data_class: INTERNAL_ONLY
    pub secondary_review_completed: bool,        // data_class: INTERNAL_ONLY
    pub secondary_review_flagged_unsafe: bool,   // data_class: INTERNAL_ONLY
    pub manual_review: ManualReviewState,        // data_class: INTERNAL_ONLY
    pub tenant_may_override: bool,               // data_class: INTERNAL_ONLY
    pub signals_to_tenant_policy: bool,          // data_class: INTERNAL_ONLY
}

pub fn enforce_safety_signals(signals: &[SafetySignal]) -> SafetyEnforcementDecision {
    let has_critical = signals.iter().any(|signal| signal.critical);
    SafetyEnforcementDecision {
        blocked: has_critical,
        quarantined: has_critical,
        secondary_agentic_review_required: has_critical,
        secondary_review_completed: false,
        secondary_review_flagged_unsafe: false,
        manual_review: if has_critical {
            ManualReviewState::Required
        } else {
            ManualReviewState::NotRequired
        },
        tenant_may_override: !has_critical,
        signals_to_tenant_policy: true,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecondaryReviewVerdict {
    Safe,
    Unsafe,
    Inconclusive,
}

pub fn apply_secondary_review(
    decision: &SafetyEnforcementDecision,
    verdict: SecondaryReviewVerdict,
) -> SafetyEnforcementDecision {
    let mut reviewed = decision.clone();
    reviewed.secondary_review_completed = true;
    reviewed.secondary_review_flagged_unsafe = matches!(
        verdict,
        SecondaryReviewVerdict::Unsafe | SecondaryReviewVerdict::Inconclusive
    );

    if decision.blocked || reviewed.secondary_review_flagged_unsafe {
        reviewed.blocked = true;
        reviewed.quarantined = true;
        reviewed.manual_review = ManualReviewState::Required;
        reviewed.tenant_may_override = false;
    }

    reviewed
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceRetentionPolicy {
    pub data_class: EvidenceDataClass, // data_class: INTERNAL_ONLY
    pub ttl: Duration,                 // data_class: INTERNAL_ONLY
    pub raw_access_requires_audited_break_glass: bool,
    pub jurisdiction_pack_required: bool,
}

pub fn default_retention_policy_for(data_class: EvidenceDataClass) -> EvidenceRetentionPolicy {
    let days = 24 * 60 * 60;
    let ttl = match data_class {
        EvidenceDataClass::PublicOrOperationalMetadata => Duration::from_secs(0),
        EvidenceDataClass::CredentialOrSecret => Duration::from_secs(days),
        EvidenceDataClass::PersonalData | EvidenceDataClass::PaymentOrFinancialData => {
            Duration::from_secs(7 * days)
        }
        EvidenceDataClass::FaultOrAnomaly => Duration::from_secs(14 * days),
        EvidenceDataClass::RegulatedHealthData
        | EvidenceDataClass::TenantBoundaryViolation
        | EvidenceDataClass::PromptInjectionOrJailbreak
        | EvidenceDataClass::SelfHarmOrHarmToOthers => Duration::from_secs(30 * days),
        EvidenceDataClass::SecurityExploitOrBreach
        | EvidenceDataClass::ChildSafetyOrAbuse
        | EvidenceDataClass::FraudOrHostilePattern => Duration::from_secs(90 * days),
    };
    EvidenceRetentionPolicy {
        data_class,
        ttl,
        raw_access_requires_audited_break_glass: true,
        jurisdiction_pack_required: matches!(
            data_class,
            EvidenceDataClass::RegulatedHealthData
                | EvidenceDataClass::PaymentOrFinancialData
                | EvidenceDataClass::SelfHarmOrHarmToOthers
                | EvidenceDataClass::ChildSafetyOrAbuse
        ),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceVisibility {
    RedactedStructuredEvidenceOnly,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceCapturePolicy {
    pub raw_payload_stored: bool,
    pub encrypted_evidence_handle: Option<String>, // data_class: SECRET_REFERENCE
    pub ttl: Duration,                             // data_class: INTERNAL_ONLY
    pub visibility: EvidenceVisibility,
    pub raw_access_requires_audited_break_glass: bool,
}

pub fn normal_evidence_capture_policy() -> EvidenceCapturePolicy {
    EvidenceCapturePolicy {
        raw_payload_stored: false,
        encrypted_evidence_handle: None,
        ttl: Duration::from_secs(0),
        visibility: EvidenceVisibility::RedactedStructuredEvidenceOnly,
        raw_access_requires_audited_break_glass: true,
    }
}

pub fn triggered_evidence_capture_policy(data_class: EvidenceDataClass) -> EvidenceCapturePolicy {
    let retention = default_retention_policy_for(data_class);
    EvidenceCapturePolicy {
        raw_payload_stored: true,
        encrypted_evidence_handle: Some(format!("evidence-ref://sealed/{:?}", data_class)),
        ttl: retention.ttl,
        visibility: EvidenceVisibility::RedactedStructuredEvidenceOnly,
        raw_access_requires_audited_break_glass: true,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RetentionOverlay {
    ShorterTtl(Duration),
    RawAccessWithoutBreakGlass,
}

impl RetentionOverlay {
    pub const fn shorter_ttl(ttl: Duration) -> Self {
        Self::ShorterTtl(ttl)
    }

    pub const fn raw_access_without_break_glass() -> Self {
        Self::RawAccessWithoutBreakGlass
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantDataClassOverlay {
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub source_class: EvidenceDataClass,     // data_class: INTERNAL_ONLY
    pub mapped_class: EvidenceDataClass,     // data_class: INTERNAL_ONLY
    pub retention_overlay: RetentionOverlay, // data_class: INTERNAL_ONLY
}

impl TenantDataClassOverlay {
    pub fn new(
        tenant_id: &str,
        source_class: EvidenceDataClass,
        mapped_class: EvidenceDataClass,
        retention_overlay: RetentionOverlay,
    ) -> Result<Self, SafetyPolicyError> {
        if tenant_id.trim().is_empty() {
            return Err(SafetyPolicyError::InvalidTenantId);
        }
        Ok(Self {
            tenant_id: tenant_id.to_string(),
            source_class,
            mapped_class,
            retention_overlay,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SafetyPolicyError {
    InvalidTenantId,
    DataClassDowngrade,
    RawAccessExpansion,
    RetentionWeakening,
    MissingWorkflow,
}

#[derive(Clone, Debug, Default)]
pub struct CentralDataClassTaxonomy;

impl CentralDataClassTaxonomy {
    pub fn validate_overlay(
        &self,
        overlay: &TenantDataClassOverlay,
    ) -> Result<(), SafetyPolicyError> {
        if overlay.mapped_class.sensitivity_rank() < overlay.source_class.sensitivity_rank() {
            return Err(SafetyPolicyError::DataClassDowngrade);
        }
        match overlay.retention_overlay {
            RetentionOverlay::RawAccessWithoutBreakGlass => {
                Err(SafetyPolicyError::RawAccessExpansion)
            }
            RetentionOverlay::ShorterTtl(ttl) => {
                let default_ttl = default_retention_policy_for(overlay.mapped_class).ttl;
                if ttl <= default_ttl {
                    Ok(())
                } else {
                    Err(SafetyPolicyError::RetentionWeakening)
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RedactionDecision {
    Allow,
    Redact,
    ReversibleTokenize,
    BlockAndQuarantine,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InTransitRedactionPlan {
    pub decision: RedactionDecision,
    pub model_receives_raw_value: bool,
    pub routing_advisor_receives_raw_value: bool,
    pub secondary_review_receives_raw_value: bool,
}

pub fn classify_in_transit_payload(
    _payload_sample: &str,
    data_class: EvidenceDataClass,
) -> InTransitRedactionPlan {
    let decision = if data_class.requires_blocking_in_transit() {
        RedactionDecision::BlockAndQuarantine
    } else if matches!(data_class, EvidenceDataClass::PersonalData) {
        RedactionDecision::ReversibleTokenize
    } else if matches!(
        data_class,
        EvidenceDataClass::FaultOrAnomaly | EvidenceDataClass::FraudOrHostilePattern
    ) {
        RedactionDecision::Redact
    } else {
        RedactionDecision::Allow
    };

    InTransitRedactionPlan {
        decision,
        model_receives_raw_value: matches!(decision, RedactionDecision::Allow),
        routing_advisor_receives_raw_value: false,
        secondary_review_receives_raw_value: false,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenLifetime {
    EphemeralRun,
    NamedWorkflow { workflow: String, ttl: Duration },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenizationPolicy {
    pub tenant_id: String, // data_class: INTERNAL_ONLY
    pub purpose: String,   // data_class: INTERNAL_ONLY
    pub tenant_policy_approved: bool,
    pub lifetime: TokenLifetime,
    pub restore_only_after_model_output: bool,
    pub provider_visible: bool,
    pub routing_advisor_visible: bool,
}

impl TokenizationPolicy {
    pub fn tenant_approved_default(tenant_id: &str, purpose: &str) -> Self {
        Self {
            tenant_id: tenant_id.to_string(),
            purpose: purpose.to_string(),
            tenant_policy_approved: true,
            lifetime: TokenLifetime::EphemeralRun,
            restore_only_after_model_output: true,
            provider_visible: false,
            routing_advisor_visible: false,
        }
    }

    pub fn long_lived_without_named_workflow(tenant_id: &str) -> Result<Self, SafetyPolicyError> {
        let _ = tenant_id;
        Err(SafetyPolicyError::MissingWorkflow)
    }

    pub fn tenant_approved_named_workflow(
        tenant_id: &str,
        workflow: &str,
        ttl: Duration,
    ) -> Result<Self, SafetyPolicyError> {
        if workflow.trim().is_empty() {
            return Err(SafetyPolicyError::MissingWorkflow);
        }
        Ok(Self {
            tenant_id: tenant_id.to_string(),
            purpose: workflow.to_string(),
            tenant_policy_approved: true,
            lifetime: TokenLifetime::NamedWorkflow {
                workflow: workflow.to_string(),
                ttl,
            },
            restore_only_after_model_output: true,
            provider_visible: false,
            routing_advisor_visible: false,
        })
    }
}
