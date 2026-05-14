//! GitOps promotion controller for Oya VCS ChangeBundles.
//!
//! This crate is deliberately provider-free and network-free. It owns the pure
//! promotion state machine and the typed provider contract seams that adapters
//! must satisfy before a bundle can move from CI/CD admission through the
//! dev -> staging -> production release train and GitOps reconciliation.

use std::collections::BTreeMap;
use std::fmt;

use oya_foundry_vcs_changebundle_kernel::{
    BundleError, ChangeBundle, PromotionEvidence, PromotionStatus,
};
use oya_foundry_vcs_kernel::PromotionState as KernelPromotionState;
use oya_foundry_vcs_test_standard_gate::{AdmissionDecision, FixupReason};

const CONTROLLER_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct IdempotencyKey(String); // data_class: INTERNAL_ONLY

impl IdempotencyKey {
    pub fn new(value: impl Into<String>) -> Result<Self, PromotionError> {
        let value = normalize_non_empty(value.into(), PromotionError::InvalidIdempotencyKey)?;
        if value.len() > 160 || !value.chars().all(is_key_char) {
            return Err(PromotionError::InvalidIdempotencyKey);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Environment {
    Dev,
    Staging,
    Production,
}

impl Environment {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dev => "dev",
            Self::Staging => "staging",
            Self::Production => "production",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnvironmentHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnvironmentStatus {
    pub environment: Environment,       // data_class: INTERNAL_ONLY
    pub health: EnvironmentHealth,      // data_class: INTERNAL_ONLY
    pub observed_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub message: String,                // data_class: INTERNAL_ONLY
}

impl EnvironmentStatus {
    pub fn healthy(environment: Environment, observed_at_epoch_seconds: u64) -> Self {
        Self {
            environment,
            health: EnvironmentHealth::Healthy,
            observed_at_epoch_seconds,
            message: "healthy".into(),
        }
    }

    pub fn new(
        environment: Environment,
        health: EnvironmentHealth,
        observed_at_epoch_seconds: u64,
        message: impl Into<String>,
    ) -> Result<Self, PromotionError> {
        Ok(Self {
            environment,
            health,
            observed_at_epoch_seconds,
            message: normalize_non_empty(message.into(), PromotionError::InvalidProviderEvidence)?,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ProviderKind {
    Ci,
    GitHubActions,
    Trivy,
    ArgoGitOps,
    NativeManual,
}

impl ProviderKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ci => "ci",
            Self::GitHubActions => "github-actions",
            Self::Trivy => "trivy",
            Self::ArgoGitOps => "argo-gitops",
            Self::NativeManual => "native-manual",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderAvailability {
    Available,
    Outage,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderDecision {
    Passed,
    Failed,
    Pending,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderContractEvidence {
    pub provider: ProviderKind,             // data_class: INTERNAL_ONLY
    pub availability: ProviderAvailability, // data_class: INTERNAL_ONLY
    pub decision: ProviderDecision,         // data_class: INTERNAL_ONLY
    pub contract_fixture_id: String,        // data_class: INTERNAL_ONLY
    pub observed_generation: u64,           // data_class: INTERNAL_ONLY
    pub observed_at_epoch_seconds: u64,     // data_class: INTERNAL_ONLY
    pub details: String,                    // data_class: INTERNAL_ONLY
}

impl ProviderContractEvidence {
    pub fn new(
        provider: ProviderKind,
        availability: ProviderAvailability,
        decision: ProviderDecision,
        contract_fixture_id: impl Into<String>,
        observed_generation: u64,
        observed_at_epoch_seconds: u64,
        details: impl Into<String>,
    ) -> Result<Self, PromotionError> {
        if observed_generation == 0 || observed_at_epoch_seconds == 0 {
            return Err(PromotionError::InvalidProviderEvidence);
        }
        Ok(Self {
            provider,
            availability,
            decision,
            contract_fixture_id: normalize_non_empty(
                contract_fixture_id.into(),
                PromotionError::InvalidProviderEvidence,
            )?,
            observed_generation,
            observed_at_epoch_seconds,
            details: normalize_non_empty(details.into(), PromotionError::InvalidProviderEvidence)?,
        })
    }

    pub fn passed(provider: ProviderKind, fixture: impl Into<String>, generation: u64) -> Self {
        Self::new(
            provider,
            ProviderAvailability::Available,
            ProviderDecision::Passed,
            fixture,
            generation,
            1_800_000_000,
            "passed fixture",
        )
        .expect("valid provider fixture")
    }

    fn is_fresh_for(&self, index: &FreshnessEnvelope) -> bool {
        self.observed_generation == index.current_generation
            && self.observed_at_epoch_seconds <= index.now_epoch_seconds
            && index.now_epoch_seconds - self.observed_at_epoch_seconds <= index.max_age_seconds
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CiContractFixture {
    pub provider_evidence: ProviderContractEvidence, // data_class: INTERNAL_ONLY
    pub build_id: String,                            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitHubActionsContractFixture {
    pub provider_evidence: ProviderContractEvidence, // data_class: INTERNAL_ONLY
    pub workflow_run_id: String,                     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrivyContractFixture {
    pub provider_evidence: ProviderContractEvidence, // data_class: INTERNAL_ONLY
    pub critical_findings: u32,                      // data_class: INTERNAL_ONLY
    pub high_findings: u32,                          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArgoGitOpsContractFixture {
    pub provider_evidence: ProviderContractEvidence, // data_class: INTERNAL_ONLY
    pub application: String,                         // data_class: INTERNAL_ONLY
    pub target_revision: String,                     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderEvidenceSet {
    pub ci: CiContractFixture,                // data_class: INTERNAL_ONLY
    pub github: GitHubActionsContractFixture, // data_class: INTERNAL_ONLY
    pub trivy: TrivyContractFixture,          // data_class: INTERNAL_ONLY
    pub argo: ArgoGitOpsContractFixture,      // data_class: INTERNAL_ONLY
}

impl ProviderEvidenceSet {
    pub fn validate_slot_kinds(&self) -> Result<(), PromotionError> {
        validate_provider_slot(self.ci.provider_evidence.provider, ProviderKind::Ci)?;
        validate_provider_slot(
            self.github.provider_evidence.provider,
            ProviderKind::GitHubActions,
        )?;
        validate_provider_slot(self.trivy.provider_evidence.provider, ProviderKind::Trivy)?;
        validate_provider_slot(
            self.argo.provider_evidence.provider,
            ProviderKind::ArgoGitOps,
        )?;
        Ok(())
    }

    fn all(&self) -> [&ProviderContractEvidence; 4] {
        [
            &self.ci.provider_evidence,
            &self.github.provider_evidence,
            &self.trivy.provider_evidence,
            &self.argo.provider_evidence,
        ]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FreshnessEnvelope {
    pub index_digest: String,           // data_class: INTERNAL_ONLY
    pub cache_generation: u64,          // data_class: INTERNAL_ONLY
    pub current_generation: u64,        // data_class: INTERNAL_ONLY
    pub observed_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub now_epoch_seconds: u64,         // data_class: INTERNAL_ONLY
    pub max_age_seconds: u64,           // data_class: INTERNAL_ONLY
}

impl FreshnessEnvelope {
    pub fn fresh(
        index_digest: impl Into<String>,
        generation: u64,
        observed_at_epoch_seconds: u64,
        now_epoch_seconds: u64,
    ) -> Result<Self, PromotionError> {
        Self::new(
            index_digest,
            generation,
            generation,
            observed_at_epoch_seconds,
            now_epoch_seconds,
            86_400,
        )
    }

    pub fn new(
        index_digest: impl Into<String>,
        cache_generation: u64,
        current_generation: u64,
        observed_at_epoch_seconds: u64,
        now_epoch_seconds: u64,
        max_age_seconds: u64,
    ) -> Result<Self, PromotionError> {
        if cache_generation == 0
            || current_generation == 0
            || observed_at_epoch_seconds == 0
            || now_epoch_seconds == 0
            || max_age_seconds == 0
            || observed_at_epoch_seconds > now_epoch_seconds
        {
            return Err(PromotionError::InvalidFreshnessEvidence);
        }
        Ok(Self {
            index_digest: normalize_non_empty(
                index_digest.into(),
                PromotionError::InvalidFreshnessEvidence,
            )?,
            cache_generation,
            current_generation,
            observed_at_epoch_seconds,
            now_epoch_seconds,
            max_age_seconds,
        })
    }

    pub fn is_fresh(&self) -> bool {
        self.cache_generation == self.current_generation
            && self.now_epoch_seconds - self.observed_at_epoch_seconds <= self.max_age_seconds
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotionPolicy {
    pub allow_native_manual_fallback: bool, // data_class: INTERNAL_ONLY
    pub require_production_health: bool,    // data_class: INTERNAL_ONLY
    pub max_release_train_hops: usize,      // data_class: INTERNAL_ONLY
}

impl PromotionPolicy {
    pub fn strict() -> Self {
        Self {
            allow_native_manual_fallback: false,
            require_production_health: true,
            max_release_train_hops: 3,
        }
    }

    pub fn allow_degraded_native_manual() -> Self {
        Self {
            allow_native_manual_fallback: true,
            require_production_health: true,
            max_release_train_hops: 3,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromotionControllerState {
    Requested,
    AdmissionEvaluated,
    SecurityScanned,
    PublishedDev,
    PublishedStaging,
    PublishedProduction,
    Reconciled,
    RollbackRequested,
    RolledBack,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransitionRecord {
    pub from: PromotionControllerState, // data_class: INTERNAL_ONLY
    pub to: PromotionControllerState,   // data_class: INTERNAL_ONLY
    pub reason: String,                 // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseTrainRecord {
    pub environment: Environment, // data_class: INTERNAL_ONLY
    pub provider: ProviderKind,   // data_class: INTERNAL_ONLY
    pub degraded: bool,           // data_class: INTERNAL_ONLY
    pub evidence_ref: String,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotionRequest {
    pub request_id: String,                         // data_class: INTERNAL_ONLY
    pub idempotency_key: IdempotencyKey,            // data_class: INTERNAL_ONLY
    pub bundle: ChangeBundle,                       // data_class: INTERNAL_ONLY
    pub admission: AdmissionDecision,               // data_class: INTERNAL_ONLY
    pub freshness: FreshnessEnvelope,               // data_class: INTERNAL_ONLY
    pub providers: ProviderEvidenceSet,             // data_class: INTERNAL_ONLY
    pub environment_health: Vec<EnvironmentStatus>, // data_class: INTERNAL_ONLY
    pub policy: PromotionPolicy,                    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotionOutcome {
    pub schema_version: u32,                    // data_class: INTERNAL_ONLY
    pub request_id: String,                     // data_class: INTERNAL_ONLY
    pub idempotency_key: IdempotencyKey,        // data_class: INTERNAL_ONLY
    pub final_state: PromotionControllerState,  // data_class: INTERNAL_ONLY
    pub kernel_state: KernelPromotionState,     // data_class: INTERNAL_ONLY
    pub duplicate_collapsed: bool,              // data_class: INTERNAL_ONLY
    pub release_train: Vec<ReleaseTrainRecord>, // data_class: INTERNAL_ONLY
    pub transitions: Vec<TransitionRecord>,     // data_class: INTERNAL_ONLY
    pub degraded_path: Vec<String>,             // data_class: INTERNAL_ONLY
    pub rejected_reasons: Vec<PromotionError>,  // data_class: INTERNAL_ONLY
    pub bundle: ChangeBundle,                   // data_class: INTERNAL_ONLY
}

#[derive(Default)]
pub struct PromotionController {
    outcomes_by_key: BTreeMap<IdempotencyKey, PromotionOutcome>, // data_class: INTERNAL_ONLY
}

impl PromotionController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn promote(&mut self, request: PromotionRequest) -> PromotionOutcome {
        if let Some(previous) = self.outcomes_by_key.get(&request.idempotency_key).cloned() {
            let mut duplicate = previous;
            duplicate.duplicate_collapsed = true;
            return duplicate;
        }

        let outcome = reduce_promotion(request);
        self.outcomes_by_key
            .insert(outcome.idempotency_key.clone(), outcome.clone());
        outcome
    }

    pub fn rollback(
        &mut self,
        idempotency_key: IdempotencyKey,
        reason: impl Into<String>,
    ) -> Result<PromotionOutcome, PromotionError> {
        let reason = normalize_non_empty(reason.into(), PromotionError::InvalidRollbackReason)?;
        let previous = self
            .outcomes_by_key
            .get_mut(&idempotency_key)
            .ok_or(PromotionError::UnknownIdempotencyKey)?;
        let from = previous.final_state;
        previous.transitions.push(TransitionRecord {
            from,
            to: PromotionControllerState::RollbackRequested,
            reason: reason.clone(),
        });
        previous.transitions.push(TransitionRecord {
            from: PromotionControllerState::RollbackRequested,
            to: PromotionControllerState::RolledBack,
            reason,
        });
        previous.final_state = PromotionControllerState::RolledBack;
        previous.kernel_state = KernelPromotionState::RolledBack;
        Ok(previous.clone())
    }
}

pub fn reduce_promotion(request: PromotionRequest) -> PromotionOutcome {
    let mut builder = OutcomeBuilder::new(&request);

    builder.transition(
        PromotionControllerState::Requested,
        PromotionControllerState::AdmissionEvaluated,
        "request accepted for deterministic admission",
    );

    validate_admission(&request, &mut builder);
    validate_freshness(&request, &mut builder);
    validate_provider_contracts(&request, &mut builder);
    validate_environment_health(&request, &mut builder);

    if !builder.rejected_reasons.is_empty() {
        builder.transition(
            builder.current,
            PromotionControllerState::Rejected,
            "blocking promotion invariant failed",
        );
        return builder.finish(request.bundle, false);
    }

    builder.transition(
        builder.current,
        PromotionControllerState::SecurityScanned,
        "ci, github-actions and trivy contract fixtures admitted",
    );

    let mut bundle = request.bundle;
    for environment in [
        Environment::Dev,
        Environment::Staging,
        Environment::Production,
    ] {
        let degraded = builder
            .degraded_path
            .iter()
            .any(|entry| entry.contains("argo-gitops"));
        let provider = if degraded {
            ProviderKind::NativeManual
        } else {
            ProviderKind::ArgoGitOps
        };
        let evidence_ref = format!(
            "promo_{}_{}_{}",
            request.request_id.replace('-', "_"),
            environment.as_str(),
            request.freshness.current_generation
        );
        let status = if environment == Environment::Production {
            PromotionStatus::Published
        } else {
            PromotionStatus::Requested
        };
        let promotion_evidence = match PromotionEvidence::new(
            evidence_ref.clone(),
            environment.as_str(),
            status,
            format!("provider={} degraded={degraded}", provider.as_str()),
            false,
        ) {
            Ok(value) => value,
            Err(error) => {
                builder.rejected_reasons.push(PromotionError::Bundle(error));
                break;
            }
        };
        if environment == Environment::Production {
            if let Err(error) = bundle
                .publish_promotion_evidence(promotion_evidence, request.freshness.now_epoch_seconds)
            {
                builder.rejected_reasons.push(PromotionError::Bundle(error));
                break;
            }
        } else {
            bundle.promotion_evidence.push(promotion_evidence);
        }
        builder.release_train.push(ReleaseTrainRecord {
            environment,
            provider,
            degraded,
            evidence_ref,
        });
        let next = match environment {
            Environment::Dev => PromotionControllerState::PublishedDev,
            Environment::Staging => PromotionControllerState::PublishedStaging,
            Environment::Production => PromotionControllerState::PublishedProduction,
        };
        builder.transition(builder.current, next, "release-train hop recorded");
    }

    if !builder.rejected_reasons.is_empty() {
        builder.transition(
            builder.current,
            PromotionControllerState::Rejected,
            "bundle refused release-train evidence",
        );
        return builder.finish(bundle, false);
    }

    builder.transition(
        builder.current,
        PromotionControllerState::Reconciled,
        "gitops desired state reconciled",
    );
    builder.finish(bundle, false)
}

fn validate_provider_slot(
    actual: ProviderKind,
    expected: ProviderKind,
) -> Result<(), PromotionError> {
    if actual == expected {
        Ok(())
    } else {
        Err(PromotionError::ProviderSlotMismatch { expected, actual })
    }
}

fn validate_admission(request: &PromotionRequest, builder: &mut OutcomeBuilder) {
    if !request.admission.accepted {
        let mut emitted = Vec::<FixupReason>::new();
        for reason in request
            .admission
            .fixup_tasks
            .iter()
            .filter(|task| task.blocking)
            .map(|task| task.reason.clone())
        {
            if !emitted.contains(&reason) {
                emitted.push(reason.clone());
                builder
                    .rejected_reasons
                    .push(PromotionError::AdmissionFixup(reason));
            }
        }
        if emitted.is_empty() {
            builder
                .rejected_reasons
                .push(PromotionError::AdmissionRejected);
        }
    }
}

fn validate_freshness(request: &PromotionRequest, builder: &mut OutcomeBuilder) {
    if !request.freshness.is_fresh() {
        builder
            .rejected_reasons
            .push(PromotionError::StaleIndexEvidence);
    }
    for provider in request.providers.all() {
        if !provider.is_fresh_for(&request.freshness) {
            builder
                .rejected_reasons
                .push(PromotionError::StaleProviderEvidence(provider.provider));
        }
    }
}

fn validate_provider_contracts(request: &PromotionRequest, builder: &mut OutcomeBuilder) {
    if let Err(error) = request.providers.validate_slot_kinds() {
        builder.rejected_reasons.push(error);
    }
    for provider in request.providers.all() {
        match (provider.availability, provider.decision) {
            (ProviderAvailability::Available, ProviderDecision::Passed) => {}
            (
                ProviderAvailability::Available,
                ProviderDecision::Failed | ProviderDecision::Pending,
            ) => {
                builder
                    .rejected_reasons
                    .push(PromotionError::ProviderRejected(provider.provider));
            }
            (ProviderAvailability::Outage, _) => {
                if request.policy.allow_native_manual_fallback {
                    builder.degraded_path.push(format!(
                        "{} outage -> native/manual mode allowed by policy",
                        provider.provider.as_str()
                    ));
                } else {
                    builder
                        .rejected_reasons
                        .push(PromotionError::ProviderOutageNoFallback(provider.provider));
                }
            }
        }
    }
    if request.providers.trivy.critical_findings > 0 {
        builder
            .rejected_reasons
            .push(PromotionError::SecurityScanRejected);
    }
}

fn validate_environment_health(request: &PromotionRequest, builder: &mut OutcomeBuilder) {
    let health_by_env = request
        .environment_health
        .iter()
        .map(|status| (status.environment, status.health))
        .collect::<BTreeMap<_, _>>();
    for environment in [
        Environment::Dev,
        Environment::Staging,
        Environment::Production,
    ] {
        let health = health_by_env
            .get(&environment)
            .copied()
            .unwrap_or(EnvironmentHealth::Unknown);
        match health {
            EnvironmentHealth::Healthy => {}
            EnvironmentHealth::Degraded if environment != Environment::Production => {}
            EnvironmentHealth::Degraded if !request.policy.require_production_health => {}
            EnvironmentHealth::Degraded => builder
                .rejected_reasons
                .push(PromotionError::EnvironmentNotReady(environment)),
            EnvironmentHealth::Unhealthy | EnvironmentHealth::Unknown => builder
                .rejected_reasons
                .push(PromotionError::EnvironmentNotReady(environment)),
        }
    }
    if request.policy.max_release_train_hops < 3 {
        builder
            .rejected_reasons
            .push(PromotionError::ReleaseTrainTooShort);
    }
}

struct OutcomeBuilder {
    request_id: String,
    idempotency_key: IdempotencyKey,
    current: PromotionControllerState,
    transitions: Vec<TransitionRecord>,
    release_train: Vec<ReleaseTrainRecord>,
    degraded_path: Vec<String>,
    rejected_reasons: Vec<PromotionError>,
}

impl OutcomeBuilder {
    fn new(request: &PromotionRequest) -> Self {
        Self {
            request_id: request.request_id.clone(),
            idempotency_key: request.idempotency_key.clone(),
            current: PromotionControllerState::Requested,
            transitions: Vec::new(),
            release_train: Vec::new(),
            degraded_path: Vec::new(),
            rejected_reasons: Vec::new(),
        }
    }

    fn transition(
        &mut self,
        from: PromotionControllerState,
        to: PromotionControllerState,
        reason: impl Into<String>,
    ) {
        self.transitions.push(TransitionRecord {
            from,
            to,
            reason: reason.into(),
        });
        self.current = to;
    }

    fn finish(self, bundle: ChangeBundle, duplicate_collapsed: bool) -> PromotionOutcome {
        let final_state = self.current;
        let kernel_state = match final_state {
            PromotionControllerState::Reconciled => KernelPromotionState::PromotedProduction,
            PromotionControllerState::Rejected => KernelPromotionState::Rejected,
            PromotionControllerState::RolledBack => KernelPromotionState::RolledBack,
            PromotionControllerState::PublishedProduction => {
                KernelPromotionState::PromotedProduction
            }
            PromotionControllerState::PublishedStaging => KernelPromotionState::PromotedStaging,
            PromotionControllerState::PublishedDev => KernelPromotionState::PromotedDev,
            PromotionControllerState::AdmissionEvaluated
            | PromotionControllerState::SecurityScanned => KernelPromotionState::Admitted,
            PromotionControllerState::Requested | PromotionControllerState::RollbackRequested => {
                KernelPromotionState::Requested
            }
        };
        PromotionOutcome {
            schema_version: CONTROLLER_SCHEMA_VERSION,
            request_id: self.request_id,
            idempotency_key: self.idempotency_key,
            final_state,
            kernel_state,
            duplicate_collapsed,
            release_train: self.release_train,
            transitions: self.transitions,
            degraded_path: self.degraded_path,
            rejected_reasons: self.rejected_reasons,
            bundle,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PromotionError {
    InvalidIdempotencyKey,
    InvalidFreshnessEvidence,
    InvalidProviderEvidence,
    InvalidRollbackReason,
    UnknownIdempotencyKey,
    AdmissionRejected,
    AdmissionFixup(FixupReason),
    StaleIndexEvidence,
    StaleProviderEvidence(ProviderKind),
    ProviderRejected(ProviderKind),
    ProviderOutageNoFallback(ProviderKind),
    ProviderSlotMismatch {
        expected: ProviderKind,
        actual: ProviderKind,
    },
    SecurityScanRejected,
    EnvironmentNotReady(Environment),
    ReleaseTrainTooShort,
    Bundle(BundleError),
}

impl fmt::Display for PromotionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for PromotionError {}

fn normalize_non_empty(value: String, error: PromotionError) -> Result<String, PromotionError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        Err(error)
    } else {
        Ok(value)
    }
}

fn is_key_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':' | '.')
}
