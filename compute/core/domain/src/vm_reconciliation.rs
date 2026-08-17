//! Provider-neutral VM desired/observed reconciliation contracts.
//!
//! Provider identifiers, regions, shapes, and image OCIDs stay behind an opaque
//! [`VmProviderBindingRef`]. Plans bind the observed generation and ETag so an
//! allocation or destructive action cannot execute against stale inventory.

#![forbid(unsafe_code)]

use sha2::{Digest, Sha256};

const SHA256_PREFIX: &str = "sha256:";
const SHA256_HEX_LEN: usize = 64;
const MIN_BOOT_VOLUME_GIB: u32 = 50;
const MAX_BOOT_VOLUME_GIB: u32 = 200;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum VmTargetRole {
    ProvisioningConsole,
    GenesisDisasterRecovery,
}

impl VmTargetRole {
    const fn label(self) -> &'static str {
        match self {
            Self::ProvisioningConsole => "provisioning_console",
            Self::GenesisDisasterRecovery => "genesis_disaster_recovery",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum VmLifecycleIntent {
    CreateIfAbsent,
    Retire,
}

impl VmLifecycleIntent {
    const fn label(self) -> &'static str {
        match self {
            Self::CreateIfAbsent => "create_if_absent",
            Self::Retire => "retire",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct VmPlacement {
    pub environment: String,    // data_class: PUBLIC
    pub control_domain: String, // data_class: INTERNAL_ONLY
    pub locality: String,       // data_class: PUBLIC
    pub ordinal: u16,           // data_class: PUBLIC
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VmResourceEnvelope {
    pub instance_limit: u32,        // data_class: INTERNAL_ONLY
    pub active_instances: u32,      // data_class: INTERNAL_ONLY
    pub storage_limit_gib: u32,     // data_class: INTERNAL_ONLY
    pub allocated_storage_gib: u32, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VmIntent {
    pub intent_id: String,            // data_class: INTERNAL_ONLY
    pub role: VmTargetRole,           // data_class: PUBLIC
    pub lifecycle: VmLifecycleIntent, // data_class: PUBLIC
    pub class_ref: String,            // data_class: INTERNAL_ONLY
    pub release_ref: String,          // data_class: INTERNAL_ONLY
    pub placement: VmPlacement,       // data_class: INTERNAL_ONLY
    pub endpoint_name: String,        // data_class: PUBLIC
    pub boot_volume_gib: u32,         // data_class: INTERNAL_ONLY
    pub generation: u64,              // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VmIntentError {
    EmptyField { field: &'static str },
    InvalidDigest { field: &'static str },
    InvalidEndpoint,
    InvalidBootVolume,
    UnsupportedTargetRole,
    InvalidGeneration,
}

impl std::fmt::Display for VmIntentError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyField { field } => write!(formatter, "{field} must not be empty"),
            Self::InvalidDigest { field } => {
                write!(formatter, "{field} must be a lowercase sha256 digest")
            }
            Self::InvalidEndpoint => formatter.write_str("endpoint name is invalid"),
            Self::InvalidBootVolume => formatter.write_str("boot volume size is outside policy"),
            Self::UnsupportedTargetRole => {
                formatter.write_str("target role is not permitted by this VM reconciler")
            }
            Self::InvalidGeneration => formatter.write_str("intent generation must be non-zero"),
        }
    }
}

impl std::error::Error for VmIntentError {}

impl VmIntent {
    pub fn validate(&self) -> Result<(), VmIntentError> {
        for (field, value) in [
            ("intent_id", self.intent_id.as_str()),
            ("class_ref", self.class_ref.as_str()),
            ("environment", self.placement.environment.as_str()),
            ("control_domain", self.placement.control_domain.as_str()),
            ("locality", self.placement.locality.as_str()),
        ] {
            if !valid_token(value) {
                return Err(VmIntentError::EmptyField { field });
            }
        }
        if self.role != VmTargetRole::ProvisioningConsole {
            return Err(VmIntentError::UnsupportedTargetRole);
        }
        if !valid_sha256(&self.release_ref) {
            return Err(VmIntentError::InvalidDigest {
                field: "release_ref",
            });
        }
        if self.endpoint_name != "console.oyatie.dev" {
            return Err(VmIntentError::InvalidEndpoint);
        }
        if !(MIN_BOOT_VOLUME_GIB..=MAX_BOOT_VOLUME_GIB).contains(&self.boot_volume_gib) {
            return Err(VmIntentError::InvalidBootVolume);
        }
        if self.generation == 0 {
            return Err(VmIntentError::InvalidGeneration);
        }
        Ok(())
    }

    fn canonical(&self) -> String {
        format!(
            "intent_id={}|role={}|lifecycle={}|class_ref={}|release_ref={}|environment={}|control_domain={}|locality={}|ordinal={}|endpoint={}|boot_volume_gib={}|generation={}",
            self.intent_id,
            self.role.label(),
            self.lifecycle.label(),
            self.class_ref,
            self.release_ref,
            self.placement.environment,
            self.placement.control_domain,
            self.placement.locality,
            self.placement.ordinal,
            self.endpoint_name,
            self.boot_volume_gib,
            self.generation,
        )
    }
}

/// Opaque reference to provider-owned binding data.
///
/// The domain can compare identity and immutable class/release bindings without
/// learning provider regions, shapes, OCIDs, tenancy identifiers, or credentials.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct VmProviderBindingRef {
    pub binding_ref: String,      // data_class: INTERNAL_ONLY
    pub intent_id: String,        // data_class: INTERNAL_ONLY
    pub class_ref: String,        // data_class: INTERNAL_ONLY
    pub release_ref: String,      // data_class: INTERNAL_ONLY
    pub placement_digest: String, // data_class: INTERNAL_ONLY
}

impl VmProviderBindingRef {
    pub fn for_intent(
        binding_ref: impl Into<String>,
        intent: &VmIntent,
    ) -> Result<Self, VmIntentError> {
        intent.validate()?;
        let binding_ref = binding_ref.into();
        if !valid_token(&binding_ref) {
            return Err(VmIntentError::EmptyField {
                field: "binding_ref",
            });
        }
        let placement_digest = digest(&format!(
            "environment={}|control_domain={}|locality={}|ordinal={}",
            intent.placement.environment,
            intent.placement.control_domain,
            intent.placement.locality,
            intent.placement.ordinal,
        ));
        Ok(Self {
            binding_ref,
            intent_id: intent.intent_id.clone(),
            class_ref: intent.class_ref.clone(),
            release_ref: intent.release_ref.clone(),
            placement_digest,
        })
    }

    fn matches_intent(&self, intent: &VmIntent) -> bool {
        Self::for_intent(self.binding_ref.clone(), intent).is_ok_and(|expected| expected == *self)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum VmObservedLifecycle {
    Absent,
    Provisioning,
    Running,
    Stopped,
    Terminated,
    Ambiguous,
}

impl VmObservedLifecycle {
    const fn label(self) -> &'static str {
        match self {
            Self::Absent => "absent",
            Self::Provisioning => "provisioning",
            Self::Running => "running",
            Self::Stopped => "stopped",
            Self::Terminated => "terminated",
            Self::Ambiguous => "ambiguous",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VmObservedState {
    pub binding: VmProviderBindingRef,  // data_class: INTERNAL_ONLY
    pub lifecycle: VmObservedLifecycle, // data_class: PUBLIC
    pub generation: u64,                // data_class: INTERNAL_ONLY
    pub etag: String,                   // data_class: INTERNAL_ONLY
    pub matching_resource_count: u32,   // data_class: INTERNAL_ONLY
    pub boot_volume_gib: Option<u32>,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum VmActionKind {
    AllocateBootVolume,
    LaunchInstance,
    BindNetwork,
    StartInstance,
    StopInstance,
    TerminateInstance,
    DeleteBootVolume,
    VerifyInstance,
}

impl VmActionKind {
    const fn label(self) -> &'static str {
        match self {
            Self::AllocateBootVolume => "allocate_boot_volume",
            Self::LaunchInstance => "launch_instance",
            Self::BindNetwork => "bind_network",
            Self::StartInstance => "start_instance",
            Self::StopInstance => "stop_instance",
            Self::TerminateInstance => "terminate_instance",
            Self::DeleteBootVolume => "delete_boot_volume",
            Self::VerifyInstance => "verify_instance",
        }
    }

    pub const fn destructive(self) -> bool {
        matches!(
            self,
            Self::StopInstance | Self::TerminateInstance | Self::DeleteBootVolume
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VmReconcileAction {
    pub sequence: u16,                // data_class: PUBLIC
    pub kind: VmActionKind,           // data_class: PUBLIC
    pub exact_binding_required: bool, // data_class: PUBLIC
    pub observed_etag_required: bool, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VmReconcilePlan {
    pub intent_id: String,               // data_class: INTERNAL_ONLY
    pub intent_generation: u64,          // data_class: INTERNAL_ONLY
    pub binding: VmProviderBindingRef,   // data_class: INTERNAL_ONLY
    pub observed_generation: u64,        // data_class: INTERNAL_ONLY
    pub observed_etag: String,           // data_class: INTERNAL_ONLY
    pub actions: Vec<VmReconcileAction>, // data_class: INTERNAL_ONLY
    pub digest: String,                  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VmApplyAuthorization {
    pub actor: String,                 // data_class: INTERNAL_ONLY
    pub plan_digest: String,           // data_class: INTERNAL_ONLY
    pub nonce_hash: String,            // data_class: SECRET
    pub issued_at_epoch_seconds: u64,  // data_class: INTERNAL_ONLY
    pub expires_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub consumed: bool,                // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VmReceipt {
    pub operation_id: String,                 // data_class: INTERNAL_ONLY
    pub plan_digest: String,                  // data_class: INTERNAL_ONLY
    pub binding_ref: String,                  // data_class: INTERNAL_ONLY
    pub applied_actions: Vec<VmActionKind>,   // data_class: INTERNAL_ONLY
    pub final_generation: u64,                // data_class: INTERNAL_ONLY
    pub final_etag_hash: String,              // data_class: INTERNAL_ONLY
    pub provider_receipt_hashes: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VmVerification {
    pub plan_digest: String,            // data_class: INTERNAL_ONLY
    pub observed_generation: u64,       // data_class: INTERNAL_ONLY
    pub observed_etag_hash: String,     // data_class: INTERNAL_ONLY
    pub lifecycle: VmObservedLifecycle, // data_class: PUBLIC
    pub exact_match: bool,              // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VmReconcileError {
    Intent(VmIntentError),
    BindingMismatch,
    StaleObservation,
    AmbiguousObservation,
    QuotaExceeded,
    MissingAuthorization,
    AuthorizationExpired,
    AuthorizationConsumed,
    PlanDigestMismatch,
    Provider {
        operation: &'static str,
        detail: String,
    },
    VerificationFailed,
}

impl std::fmt::Display for VmReconcileError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Intent(error) => write!(formatter, "invalid VM intent: {error}"),
            Self::BindingMismatch => formatter.write_str("provider binding does not match intent"),
            Self::StaleObservation => formatter.write_str("observed generation or ETag is stale"),
            Self::AmbiguousObservation => formatter.write_str("provider observation is ambiguous"),
            Self::QuotaExceeded => formatter.write_str("resource envelope would be exceeded"),
            Self::MissingAuthorization => {
                formatter.write_str("destructive plan lacks authorization")
            }
            Self::AuthorizationExpired => {
                formatter.write_str("authorization is expired or not yet valid")
            }
            Self::AuthorizationConsumed => {
                formatter.write_str("authorization was already consumed")
            }
            Self::PlanDigestMismatch => {
                formatter.write_str("authorization does not bind this plan")
            }
            Self::Provider { operation, detail } => {
                write!(formatter, "provider {operation} failed: {detail}")
            }
            Self::VerificationFailed => formatter.write_str("GET-after-write verification failed"),
        }
    }
}

impl std::error::Error for VmReconcileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Intent(error) => Some(error),
            _ => None,
        }
    }
}

impl From<VmIntentError> for VmReconcileError {
    fn from(error: VmIntentError) -> Self {
        Self::Intent(error)
    }
}

impl VmReconcilePlan {
    pub fn build(
        intent: &VmIntent,
        binding: &VmProviderBindingRef,
        observed: &VmObservedState,
        envelope: VmResourceEnvelope,
    ) -> Result<Self, VmReconcileError> {
        intent.validate()?;
        if !binding.matches_intent(intent) || observed.binding != *binding {
            return Err(VmReconcileError::BindingMismatch);
        }
        if observed.lifecycle == VmObservedLifecycle::Ambiguous
            || observed.matching_resource_count > 1
        {
            return Err(VmReconcileError::AmbiguousObservation);
        }
        if observed.etag.trim().is_empty() {
            return Err(VmReconcileError::StaleObservation);
        }

        let kinds = match (intent.lifecycle, observed.lifecycle) {
            (VmLifecycleIntent::CreateIfAbsent, VmObservedLifecycle::Absent)
            | (VmLifecycleIntent::CreateIfAbsent, VmObservedLifecycle::Terminated) => {
                if envelope.active_instances >= envelope.instance_limit
                    || envelope
                        .allocated_storage_gib
                        .saturating_add(intent.boot_volume_gib)
                        > envelope.storage_limit_gib
                {
                    return Err(VmReconcileError::QuotaExceeded);
                }
                vec![
                    VmActionKind::AllocateBootVolume,
                    VmActionKind::LaunchInstance,
                    VmActionKind::BindNetwork,
                    VmActionKind::StartInstance,
                    VmActionKind::VerifyInstance,
                ]
            }
            (VmLifecycleIntent::CreateIfAbsent, VmObservedLifecycle::Stopped) => {
                vec![VmActionKind::StartInstance, VmActionKind::VerifyInstance]
            }
            (VmLifecycleIntent::CreateIfAbsent, VmObservedLifecycle::Provisioning) => {
                vec![VmActionKind::VerifyInstance]
            }
            (VmLifecycleIntent::CreateIfAbsent, VmObservedLifecycle::Running)
            | (VmLifecycleIntent::Retire, VmObservedLifecycle::Absent)
            | (VmLifecycleIntent::Retire, VmObservedLifecycle::Terminated) => Vec::new(),
            (VmLifecycleIntent::Retire, VmObservedLifecycle::Provisioning)
            | (VmLifecycleIntent::Retire, VmObservedLifecycle::Running) => vec![
                VmActionKind::StopInstance,
                VmActionKind::TerminateInstance,
                VmActionKind::DeleteBootVolume,
                VmActionKind::VerifyInstance,
            ],
            (VmLifecycleIntent::Retire, VmObservedLifecycle::Stopped) => vec![
                VmActionKind::TerminateInstance,
                VmActionKind::DeleteBootVolume,
                VmActionKind::VerifyInstance,
            ],
            (_, VmObservedLifecycle::Ambiguous) => {
                return Err(VmReconcileError::AmbiguousObservation);
            }
        };

        let actions: Vec<_> = kinds
            .into_iter()
            .enumerate()
            .map(|(index, kind)| VmReconcileAction {
                sequence: u16::try_from(index + 1).unwrap_or(u16::MAX),
                kind,
                exact_binding_required: kind.destructive(),
                observed_etag_required: kind.destructive(),
            })
            .collect();
        let mut plan = Self {
            intent_id: intent.intent_id.clone(),
            intent_generation: intent.generation,
            binding: binding.clone(),
            observed_generation: observed.generation,
            observed_etag: observed.etag.clone(),
            actions,
            digest: String::new(),
        };
        plan.digest = digest(&plan.canonical(intent));
        Ok(plan)
    }

    pub fn validate_fresh_observation(
        &self,
        observed: &VmObservedState,
    ) -> Result<(), VmReconcileError> {
        if observed.binding != self.binding
            || observed.generation != self.observed_generation
            || observed.etag != self.observed_etag
        {
            return Err(VmReconcileError::StaleObservation);
        }
        Ok(())
    }

    pub fn validate_authorization(
        &self,
        authorization: Option<&VmApplyAuthorization>,
        now_epoch_seconds: u64,
    ) -> Result<(), VmReconcileError> {
        if !self.actions.iter().any(|action| action.kind.destructive()) {
            return Ok(());
        }
        let Some(authorization) = authorization else {
            return Err(VmReconcileError::MissingAuthorization);
        };
        if authorization.consumed {
            return Err(VmReconcileError::AuthorizationConsumed);
        }
        if authorization.issued_at_epoch_seconds > now_epoch_seconds
            || now_epoch_seconds >= authorization.expires_at_epoch_seconds
        {
            return Err(VmReconcileError::AuthorizationExpired);
        }
        if authorization.actor.trim().is_empty()
            || !valid_sha256(&authorization.nonce_hash)
            || authorization.plan_digest != self.digest
        {
            return Err(VmReconcileError::PlanDigestMismatch);
        }
        Ok(())
    }

    pub fn is_noop(&self) -> bool {
        self.actions.is_empty()
    }

    fn canonical(&self, intent: &VmIntent) -> String {
        let actions = self
            .actions
            .iter()
            .map(|action| {
                format!(
                    "{}:{}:{}:{}",
                    action.sequence,
                    action.kind.label(),
                    action.exact_binding_required,
                    action.observed_etag_required,
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{}|binding_ref={}|observed_generation={}|observed_etag={}|actions={actions}",
            intent.canonical(),
            self.binding.binding_ref,
            self.observed_generation,
            self.observed_etag,
        )
    }
}

pub trait VmReconciliationPort {
    fn observe(
        &self,
        intent: &VmIntent,
        binding: &VmProviderBindingRef,
    ) -> Result<VmObservedState, VmReconcileError>;

    fn plan(
        &self,
        intent: &VmIntent,
        binding: &VmProviderBindingRef,
        observed: &VmObservedState,
        envelope: VmResourceEnvelope,
    ) -> Result<VmReconcilePlan, VmReconcileError> {
        VmReconcilePlan::build(intent, binding, observed, envelope)
    }

    fn apply(
        &self,
        intent: &VmIntent,
        plan: &VmReconcilePlan,
        authorization: Option<&VmApplyAuthorization>,
        now_epoch_seconds: u64,
    ) -> Result<VmReceipt, VmReconcileError>;

    fn poll(
        &self,
        intent: &VmIntent,
        plan: &VmReconcilePlan,
    ) -> Result<VmObservedState, VmReconcileError>;

    fn verify(
        &self,
        intent: &VmIntent,
        plan: &VmReconcilePlan,
        receipt: &VmReceipt,
    ) -> Result<VmVerification, VmReconcileError>;
}

fn valid_token(value: &str) -> bool {
    !value.trim().is_empty()
        && value.len() <= 256
        && !value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace())
}

fn valid_sha256(value: &str) -> bool {
    value.strip_prefix(SHA256_PREFIX).is_some_and(|hex| {
        hex.len() == SHA256_HEX_LEN
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn digest(value: &str) -> String {
    let bytes = Sha256::digest(value.as_bytes());
    let mut output = String::with_capacity(SHA256_PREFIX.len() + SHA256_HEX_LEN);
    output.push_str(SHA256_PREFIX);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn intent(lifecycle: VmLifecycleIntent) -> VmIntent {
        VmIntent {
            intent_id: "bootstrap-development-provisioning-console-1".to_string(),
            role: VmTargetRole::ProvisioningConsole,
            lifecycle,
            class_ref: "machine-class/e2-micro-v1".to_string(),
            release_ref: DIGEST.to_string(),
            placement: VmPlacement {
                environment: "development".to_string(),
                control_domain: "control".to_string(),
                locality: "kr-chuncheon".to_string(),
                ordinal: 1,
            },
            endpoint_name: "console.oyatie.dev".to_string(),
            boot_volume_gib: 50,
            generation: 7,
        }
    }

    fn observed(
        intent: &VmIntent,
        lifecycle: VmObservedLifecycle,
    ) -> (VmProviderBindingRef, VmObservedState) {
        let binding =
            VmProviderBindingRef::for_intent("binding/console-1", intent).expect("valid binding");
        let observed = VmObservedState {
            binding: binding.clone(),
            lifecycle,
            generation: 4,
            etag: "etag-generation-4".to_string(),
            matching_resource_count: u32::from(lifecycle != VmObservedLifecycle::Absent),
            boot_volume_gib: None,
        };
        (binding, observed)
    }

    fn envelope() -> VmResourceEnvelope {
        VmResourceEnvelope {
            instance_limit: 2,
            active_instances: 1,
            storage_limit_gib: 200,
            allocated_storage_gib: 147,
        }
    }

    #[test]
    fn create_plan_is_deterministic_and_ordered() {
        let intent = intent(VmLifecycleIntent::CreateIfAbsent);
        let (binding, observed) = observed(&intent, VmObservedLifecycle::Absent);
        let first =
            VmReconcilePlan::build(&intent, &binding, &observed, envelope()).expect("plan builds");
        let second = VmReconcilePlan::build(&intent, &binding, &observed, envelope())
            .expect("same plan builds");

        assert_eq!(first, second);
        assert!(valid_sha256(&first.digest));
        assert_eq!(
            first
                .actions
                .iter()
                .map(|action| action.kind)
                .collect::<Vec<_>>(),
            vec![
                VmActionKind::AllocateBootVolume,
                VmActionKind::LaunchInstance,
                VmActionKind::BindNetwork,
                VmActionKind::StartInstance,
                VmActionKind::VerifyInstance,
            ]
        );
    }

    #[test]
    fn repeated_create_intent_for_running_vm_is_noop() {
        let intent = intent(VmLifecycleIntent::CreateIfAbsent);
        let (binding, observed) = observed(&intent, VmObservedLifecycle::Running);
        let plan =
            VmReconcilePlan::build(&intent, &binding, &observed, envelope()).expect("plan builds");
        assert!(plan.is_noop());
    }

    #[test]
    fn genesis_target_is_rejected() {
        let mut intent = intent(VmLifecycleIntent::CreateIfAbsent);
        intent.role = VmTargetRole::GenesisDisasterRecovery;
        assert_eq!(intent.validate(), Err(VmIntentError::UnsupportedTargetRole));
    }

    #[test]
    fn wrong_binding_is_rejected() {
        let intent = intent(VmLifecycleIntent::CreateIfAbsent);
        let (binding, mut observed) = observed(&intent, VmObservedLifecycle::Absent);
        observed.binding.binding_ref = "binding/wrong".to_string();
        assert_eq!(
            VmReconcilePlan::build(&intent, &binding, &observed, envelope()),
            Err(VmReconcileError::BindingMismatch)
        );
    }

    #[test]
    fn ambiguous_observation_is_rejected() {
        let intent = intent(VmLifecycleIntent::CreateIfAbsent);
        let (binding, mut observed) = observed(&intent, VmObservedLifecycle::Running);
        observed.matching_resource_count = 2;
        assert_eq!(
            VmReconcilePlan::build(&intent, &binding, &observed, envelope()),
            Err(VmReconcileError::AmbiguousObservation)
        );
    }

    #[test]
    fn quota_breach_is_rejected() {
        let intent = intent(VmLifecycleIntent::CreateIfAbsent);
        let (binding, observed) = observed(&intent, VmObservedLifecycle::Absent);
        let mut exhausted = envelope();
        exhausted.allocated_storage_gib = 151;
        assert_eq!(
            VmReconcilePlan::build(&intent, &binding, &observed, exhausted),
            Err(VmReconcileError::QuotaExceeded)
        );
    }

    #[test]
    fn stale_etag_is_rejected_before_apply() {
        let intent = intent(VmLifecycleIntent::Retire);
        let (binding, observed) = observed(&intent, VmObservedLifecycle::Stopped);
        let plan =
            VmReconcilePlan::build(&intent, &binding, &observed, envelope()).expect("plan builds");
        let mut changed = observed;
        changed.etag = "etag-generation-5".to_string();
        assert_eq!(
            plan.validate_fresh_observation(&changed),
            Err(VmReconcileError::StaleObservation)
        );
    }

    #[test]
    fn destructive_plan_requires_fresh_digest_bound_authorization() {
        let intent = intent(VmLifecycleIntent::Retire);
        let (binding, observed) = observed(&intent, VmObservedLifecycle::Stopped);
        let plan =
            VmReconcilePlan::build(&intent, &binding, &observed, envelope()).expect("plan builds");
        assert_eq!(
            plan.validate_authorization(None, 100),
            Err(VmReconcileError::MissingAuthorization)
        );

        let mut authorization = VmApplyAuthorization {
            actor: "principal/operator-1".to_string(),
            plan_digest: plan.digest.clone(),
            nonce_hash: DIGEST.to_string(),
            issued_at_epoch_seconds: 90,
            expires_at_epoch_seconds: 120,
            consumed: false,
        };
        plan.validate_authorization(Some(&authorization), 100)
            .expect("bound authorization accepted");
        authorization.consumed = true;
        assert_eq!(
            plan.validate_authorization(Some(&authorization), 100),
            Err(VmReconcileError::AuthorizationConsumed)
        );
    }

    #[test]
    fn changed_plan_after_approval_is_rejected() {
        let intent = intent(VmLifecycleIntent::Retire);
        let (binding, observed) = observed(&intent, VmObservedLifecycle::Stopped);
        let plan =
            VmReconcilePlan::build(&intent, &binding, &observed, envelope()).expect("plan builds");
        let mut authorization = VmApplyAuthorization {
            actor: "principal/operator-1".to_string(),
            plan_digest: plan.digest.clone(),
            nonce_hash: DIGEST.to_string(),
            issued_at_epoch_seconds: 90,
            expires_at_epoch_seconds: 120,
            consumed: false,
        };
        authorization.plan_digest = digest("different-plan");
        assert_eq!(
            plan.validate_authorization(Some(&authorization), 100),
            Err(VmReconcileError::PlanDigestMismatch)
        );
    }
}
