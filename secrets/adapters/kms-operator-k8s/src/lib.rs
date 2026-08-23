//! Transient kube-rs adapter for the Cloud KMS operator.
//!
//! ADR-0510 boundary marker: kube-rs and k8s-openapi live only in this crate.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::{
    collections::BTreeMap,
    fmt, fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use futures::StreamExt;
use kube::{
    Client,
    api::{Api, ApiResource, ListParams, Patch, PatchParams},
    core::{DynamicObject, GroupVersionKind},
    runtime::{Controller, controller::Action as ControllerAction, watcher},
};
use network_residency::ResidencyClass;
use data_boundary_kernel::DataClass;
use secrets_kms_domain::{
    CloudKmsDirectory, CloudKmsError, HsmValidation as DomainHsmValidation, KeyDestructionReceipt,
    KeyDestructionRequest, KeyRingQuarantineRequest, KeyVersionDemotionRequest, KmsDecryptRequest,
    KmsEncryptRequest, KmsKey, KmsKeyCreate, KmsKeyId, KmsKeyOrigin as DomainKeyOrigin,
    KmsKeyState, KmsKeyUsage as DomainKeyUsage, KmsKeyVersionLifecycle,
    KmsKeyVersionLifecycleState, KmsRepo, KmsSealingRoot, KmsSealingRootCreate, KmsUseReceipt,
};
use secrets_kms_operator_kernel::{
    Action, Clock, DataClassLabel, DesiredState, HsmValidation, KeyOrigin, KeyRing, KeyUsage,
    KeyVersionState, ObservedHealth, ObservedKeyRing, ObservedKeyVersion, ObservedSealingRoot,
    ObservedState, ReadConsistency, ResidencyMode, SealingRoot, reconcile,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use tracing::{error, info, warn};

pub const ADR_0510_TRANSIENT_KUBE_ADAPTER: &str =
    "ADR-0510 transient kube-rs adapter; owned destination is cloud-k8s";
pub const KMS_OPERATOR_GROUP: &str = "kms.oyatie.com";
pub const KMS_OPERATOR_VERSION: &str = "v1alpha1";
pub const KMS_KEY_RING_KIND: &str = "KmsKeyRing";
pub const KMS_KEY_RING_PLURAL: &str = "kmskeyrings";
pub const KMS_SEALING_ROOT_KIND: &str = "KmsSealingRoot";
pub const KMS_SEALING_ROOT_PLURAL: &str = "kmssealingroots";

const KEY_RING_CRD: &str = r#"apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: kmskeyrings.kms.oyatie.com
spec:
  group: kms.oyatie.com
  scope: Namespaced
  names:
    plural: kmskeyrings
    singular: kmskeyring
    kind: KmsKeyRing
  versions:
    - name: v1alpha1
      served: true
      storage: true
      schema:
        openAPIV3Schema:
          type: object
          x-kubernetes-preserve-unknown-fields: true
"#;

const SEALING_ROOT_CRD: &str = r#"apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: kmssealingroots.kms.oyatie.com
spec:
  group: kms.oyatie.com
  scope: Namespaced
  names:
    plural: kmssealingroots
    singular: kmssealingroot
    kind: KmsSealingRoot
  versions:
    - name: v1alpha1
      served: true
      storage: true
      schema:
        openAPIV3Schema:
          type: object
          x-kubernetes-preserve-unknown-fields: true
"#;

pub fn crd_manifests() -> [&'static str; 2] {
    [KEY_RING_CRD, SEALING_ROOT_CRD]
}

pub fn key_ring_api_resource() -> ApiResource {
    ApiResource::from_gvk_with_plural(
        &GroupVersionKind::gvk(KMS_OPERATOR_GROUP, KMS_OPERATOR_VERSION, KMS_KEY_RING_KIND),
        KMS_KEY_RING_PLURAL,
    )
}

pub fn sealing_root_api_resource() -> ApiResource {
    ApiResource::from_gvk_with_plural(
        &GroupVersionKind::gvk(
            KMS_OPERATOR_GROUP,
            KMS_OPERATOR_VERSION,
            KMS_SEALING_ROOT_KIND,
        ),
        KMS_SEALING_ROOT_PLURAL,
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdapterError {
    InvalidCrdObject(String),
    PartialObservedState(String),
    DomainActuation(String),
}

impl std::fmt::Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdapterError::InvalidCrdObject(message) => write!(f, "invalid CRD object: {message}"),
            AdapterError::PartialObservedState(message) => {
                write!(f, "partial observed state: {message}")
            }
            AdapterError::DomainActuation(message) => {
                write!(f, "domain actuation failed: {message}")
            }
        }
    }
}

impl std::error::Error for AdapterError {}

#[derive(Debug)]
pub enum KubeOperatorError {
    Adapter(AdapterError),
    Kube(String),
    ActuatorLockPoisoned,
}

impl fmt::Display for KubeOperatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KubeOperatorError::Adapter(error) => write!(f, "{error}"),
            KubeOperatorError::Kube(message) => write!(f, "kube api error: {message}"),
            KubeOperatorError::ActuatorLockPoisoned => write!(f, "actuator lock poisoned"),
        }
    }
}

impl std::error::Error for KubeOperatorError {}

impl From<AdapterError> for KubeOperatorError {
    fn from(error: AdapterError) -> Self {
        Self::Adapter(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedKmsObject {
    value: Value,
}

impl ProjectedKmsObject {
    pub fn from_json(value: Value) -> Self {
        Self { value }
    }
}

pub fn project_observed_state(
    objects: &[ProjectedKmsObject],
) -> Result<ObservedState, AdapterError> {
    let mut key_rings = Vec::new();
    let mut sealing_roots = Vec::new();

    for object in objects {
        let raw: RawObject = serde_json::from_value(object.value.clone())
            .map_err(|e| AdapterError::InvalidCrdObject(e.to_string()))?;
        match raw.kind.as_str() {
            "KmsKeyRing" => key_rings.push(project_key_ring(raw)?),
            "KmsSealingRoot" => sealing_roots.push(project_sealing_root(raw)?),
            other => {
                return Err(AdapterError::InvalidCrdObject(format!(
                    "unknown kind {other}"
                )));
            }
        }
    }

    Ok(ObservedState {
        read_consistency: ReadConsistency::Complete,
        key_rings,
        sealing_roots,
    })
}

pub fn desired_state_from_observed(observed: &ObservedState) -> DesiredState {
    DesiredState {
        key_rings: observed
            .key_rings
            .iter()
            .map(|key_ring| key_ring.desired.clone())
            .collect(),
        sealing_roots: observed
            .sealing_roots
            .iter()
            .map(|sealing_root| sealing_root.desired.clone())
            .collect(),
    }
}

pub trait ObservedStateProvider {
    fn observe(&self) -> Result<ObservedState, AdapterError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InMemoryObservedStateProvider {
    result: Result<ObservedState, AdapterError>,
}

impl InMemoryObservedStateProvider {
    pub fn complete(observed: ObservedState) -> Self {
        Self {
            result: Ok(observed),
        }
    }

    pub fn partial(reason: impl Into<String>) -> Self {
        Self {
            result: Err(AdapterError::PartialObservedState(reason.into())),
        }
    }
}

impl ObservedStateProvider for InMemoryObservedStateProvider {
    fn observe(&self) -> Result<ObservedState, AdapterError> {
        self.result.clone()
    }
}

pub trait KmsOperatorActuator {
    fn desired_state_for_observed(&self, observed: &ObservedState) -> DesiredState {
        desired_state_from_observed(observed)
    }

    fn execute(&mut self, action: &Action) -> Result<(), AdapterError>;

    fn status_patches_for_actions(
        &self,
        observed: &ObservedState,
        actions: &[Action],
    ) -> Result<Vec<KmsStatusPatch>, AdapterError> {
        status_patches_for_actions(observed, actions)
    }

    fn remember_status_patches(&mut self, _actions: &[Action], _patches: &[KmsStatusPatch]) {}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KmsStatusPatchKind {
    KeyRing,
    SealingRoot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KmsStatusPatch {
    pub kind: KmsStatusPatchKind,
    pub name: String,  // data_class: INTERNAL_ONLY
    pub status: Value, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReconcileWideEvent {
    pub event_name: String,          // data_class: INTERNAL_ONLY
    pub status: String,              // data_class: PUBLIC
    pub action_count: usize,         // data_class: INTERNAL_ONLY
    pub executed_count: usize,       // data_class: INTERNAL_ONLY
    pub error_class: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReconcileCycleReport {
    pub planned_actions: usize,         // data_class: INTERNAL_ONLY
    pub executed_actions: usize,        // data_class: INTERNAL_ONLY
    pub wide_event: ReconcileWideEvent, // data_class: INTERNAL_ONLY
    pub actions: Vec<Action>,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReconcileCycleFailure {
    pub planned_actions: usize,              // data_class: INTERNAL_ONLY
    pub executed_actions: usize,             // data_class: INTERNAL_ONLY
    pub wide_event: Box<ReconcileWideEvent>, // data_class: INTERNAL_ONLY
    pub error: AdapterError,                 // data_class: INTERNAL_ONLY
}

pub fn run_reconcile_cycle<P, A, C>(
    provider: &P,
    desired: &DesiredState,
    clock: &C,
    actuator: &mut A,
) -> Result<ReconcileCycleReport, ReconcileCycleFailure>
where
    P: ObservedStateProvider,
    A: KmsOperatorActuator,
    C: Clock,
{
    let observed = provider
        .observe()
        .map_err(|error| reconcile_failure(0, 0, error))?;
    if observed.read_consistency != ReadConsistency::Complete {
        return Err(reconcile_failure(
            0,
            0,
            AdapterError::PartialObservedState("observed state was not complete".to_owned()),
        ));
    }

    let actions = reconcile(&observed, desired, clock);
    let mut executed_actions = 0;
    for action in &actions {
        actuator
            .execute(action)
            .map_err(|error| reconcile_failure(actions.len(), executed_actions, error))?;
        executed_actions += 1;
    }

    Ok(ReconcileCycleReport {
        planned_actions: actions.len(),
        executed_actions,
        wide_event: ReconcileWideEvent {
            event_name: "secrets_kms_operator_reconcile".to_owned(),
            status: "succeeded".to_owned(),
            action_count: actions.len(),
            executed_count: executed_actions,
            error_class: None,
        },
        actions,
    })
}

pub struct KubeOperatorRuntime<A, C> {
    context: Arc<KubeReconcileContext<A, C>>,
}

impl<A, C> KubeOperatorRuntime<A, C>
where
    A: KmsOperatorActuator + Send + 'static,
    C: Clock + Send + Sync + 'static,
{
    pub fn new(
        client: Client,
        namespace: String,
        actuator: A,
        clock: C,
        backoff: ExponentialBackoff,
    ) -> Self {
        let key_ring_resource = key_ring_api_resource();
        let sealing_root_resource = sealing_root_api_resource();
        let key_ring_api = Api::namespaced_with(client.clone(), &namespace, &key_ring_resource);
        let sealing_root_api = Api::namespaced_with(client, &namespace, &sealing_root_resource);
        Self {
            context: Arc::new(KubeReconcileContext {
                namespace,
                key_ring_api,
                sealing_root_api,
                key_ring_resource,
                sealing_root_resource,
                actuator: Mutex::new(actuator),
                clock,
                backoff,
                backoff_attempt: Mutex::new(0),
            }),
        }
    }

    pub async fn run(self) {
        let key_ring_controller = Controller::new_with(
            self.context.key_ring_api.clone(),
            watcher::Config::default(),
            self.context.key_ring_resource.clone(),
        )
        .shutdown_on_signal()
        .run(
            reconcile_kms_dynamic_object::<A, C>,
            kube_operator_error_policy::<A, C>,
            self.context.clone(),
        )
        .for_each(|result| async move {
            match result {
                Ok((object_ref, action)) => {
                    info!(object = ?object_ref, requeue = ?action, "cloud-kms key-ring reconcile complete");
                }
                Err(error) => {
                    error!(error = %error, "cloud-kms key-ring reconcile stream error");
                }
            }
        });

        let sealing_root_controller = Controller::new_with(
            self.context.sealing_root_api.clone(),
            watcher::Config::default(),
            self.context.sealing_root_resource.clone(),
        )
        .shutdown_on_signal()
        .run(
            reconcile_kms_dynamic_object::<A, C>,
            kube_operator_error_policy::<A, C>,
            self.context.clone(),
        )
        .for_each(|result| async move {
            match result {
                Ok((object_ref, action)) => {
                    info!(object = ?object_ref, requeue = ?action, "cloud-kms sealing-root reconcile complete");
                }
                Err(error) => {
                    error!(error = %error, "cloud-kms sealing-root reconcile stream error");
                }
            }
        });

        info!(namespace = %self.context.namespace, "starting cloud-kms kube-rs operator");
        tokio::select! {
            _ = key_ring_controller => {}
            _ = sealing_root_controller => {}
        }
    }
}

struct KubeReconcileContext<A, C> {
    namespace: String,
    key_ring_api: Api<DynamicObject>,
    sealing_root_api: Api<DynamicObject>,
    key_ring_resource: ApiResource,
    sealing_root_resource: ApiResource,
    actuator: Mutex<A>,
    clock: C,
    backoff: ExponentialBackoff,
    backoff_attempt: Mutex<u32>,
}

impl<A, C> KubeReconcileContext<A, C> {
    async fn observe_current_state(&self) -> Result<ObservedState, KubeOperatorError> {
        let mut objects = Vec::new();
        objects.extend(
            list_projected_objects(&self.key_ring_api, KMS_KEY_RING_KIND)
                .await
                .map_err(KubeOperatorError::from)?,
        );
        objects.extend(
            list_projected_objects(&self.sealing_root_api, KMS_SEALING_ROOT_KIND)
                .await
                .map_err(KubeOperatorError::from)?,
        );
        project_observed_state(&objects).map_err(KubeOperatorError::from)
    }

    fn next_backoff_delay_seconds(&self) -> u64 {
        match self.backoff_attempt.lock() {
            Ok(mut attempt) => {
                let delay = self.backoff.delay_seconds(*attempt);
                *attempt = attempt.saturating_add(1);
                delay
            }
            Err(_) => self.backoff.max_seconds,
        }
    }

    fn reset_backoff_attempt(&self) {
        if let Ok(mut attempt) = self.backoff_attempt.lock() {
            *attempt = 0;
        }
    }

    async fn patch_statuses(
        &self,
        status_patches: Vec<KmsStatusPatch>,
    ) -> Result<(), KubeOperatorError> {
        for status_patch in status_patches {
            let patch = Patch::Merge(json!({ "status": status_patch.status }));
            let params = PatchParams::default();
            match status_patch.kind {
                KmsStatusPatchKind::KeyRing => {
                    self.key_ring_api
                        .patch_status(&status_patch.name, &params, &patch)
                        .await
                        .map_err(|error| KubeOperatorError::Kube(error.to_string()))?;
                }
                KmsStatusPatchKind::SealingRoot => {
                    self.sealing_root_api
                        .patch_status(&status_patch.name, &params, &patch)
                        .await
                        .map_err(|error| KubeOperatorError::Kube(error.to_string()))?;
                }
            }
        }
        Ok(())
    }
}

async fn reconcile_kms_dynamic_object<A, C>(
    _object: Arc<DynamicObject>,
    context: Arc<KubeReconcileContext<A, C>>,
) -> Result<ControllerAction, KubeOperatorError>
where
    A: KmsOperatorActuator + Send + 'static,
    C: Clock + Send + Sync + 'static,
{
    let started = Instant::now();
    let observed = match context.observe_current_state().await {
        Ok(observed) => observed,
        Err(error) => {
            let failure =
                reconcile_observation_failure(adapter_error_from_kube_operator_error(&error));
            emit_reconcile_wide_event(&failure.wide_event, started.elapsed());
            return Err(error);
        }
    };
    let provider = InMemoryObservedStateProvider::complete(observed.clone());
    let cycle_result = {
        let mut actuator = context
            .actuator
            .lock()
            .map_err(|_| KubeOperatorError::ActuatorLockPoisoned)?;
        let desired = actuator.desired_state_for_observed(&observed);
        match run_reconcile_cycle(&provider, &desired, &context.clock, &mut *actuator) {
            Ok(report) => match actuator.status_patches_for_actions(&observed, &report.actions) {
                Ok(status_patches) => {
                    actuator.remember_status_patches(&report.actions, &status_patches);
                    Ok((report, status_patches))
                }
                Err(error) => Err(reconcile_failure(
                    report.planned_actions,
                    report.executed_actions,
                    error,
                )),
            },
            Err(failure) => Err(failure),
        }
    };
    match cycle_result {
        Ok((report, status_patches)) => {
            if let Err(error) = context.patch_statuses(status_patches).await {
                let failure = reconcile_failure(
                    report.planned_actions,
                    report.executed_actions,
                    adapter_error_from_kube_operator_error(&error),
                );
                emit_reconcile_wide_event(&failure.wide_event, started.elapsed());
                return Err(error);
            }
            context.reset_backoff_attempt();
            emit_reconcile_wide_event(&report.wide_event, started.elapsed());
            Ok(ControllerAction::requeue(Duration::from_secs(
                context.backoff.base_seconds,
            )))
        }
        Err(failure) => {
            emit_reconcile_wide_event(&failure.wide_event, started.elapsed());
            Err(KubeOperatorError::Adapter(failure.error))
        }
    }
}

fn kube_operator_error_policy<A, C>(
    _object: Arc<DynamicObject>,
    error: &KubeOperatorError,
    context: Arc<KubeReconcileContext<A, C>>,
) -> ControllerAction {
    warn!(error = %error, "cloud-kms reconcile failed closed");
    ControllerAction::requeue(Duration::from_secs(context.next_backoff_delay_seconds()))
}

async fn list_projected_objects(
    api: &Api<DynamicObject>,
    fallback_kind: &str,
) -> Result<Vec<ProjectedKmsObject>, AdapterError> {
    let listed = api
        .list(&ListParams::default())
        .await
        .map_err(|e| AdapterError::PartialObservedState(e.to_string()))?;
    listed
        .items
        .into_iter()
        .map(|object| project_dynamic_object(object, fallback_kind))
        .collect()
}

fn project_dynamic_object(
    object: DynamicObject,
    fallback_kind: &str,
) -> Result<ProjectedKmsObject, AdapterError> {
    let name = object
        .metadata
        .name
        .ok_or_else(|| AdapterError::InvalidCrdObject("metadata.name is missing".to_owned()))?;
    let kind = match object.types.map(|types| types.kind) {
        Some(value) if !value.is_empty() => value,
        _ => fallback_kind.to_owned(),
    };
    let mut body = match object.data {
        Value::Object(map) => map,
        _ => Map::new(),
    };
    body.insert("kind".to_owned(), Value::String(kind));
    body.insert("metadata".to_owned(), json!({ "name": name }));
    Ok(ProjectedKmsObject::from_json(Value::Object(body)))
}

pub fn emit_reconcile_wide_event(event: &ReconcileWideEvent, elapsed: Duration) {
    info!(
        event_name = %event.event_name,
        status = %event.status,
        action_count = event.action_count,
        executed_count = event.executed_count,
        error_class = event.error_class.as_deref(),
        metric_name = "secrets_kms_operator_reconcile_convergence_seconds",
        convergence_seconds = elapsed.as_secs_f64(),
        "cloud-kms operator reconcile cycle"
    );
}

fn reconcile_failure(
    planned_actions: usize,
    executed_actions: usize,
    error: AdapterError,
) -> ReconcileCycleFailure {
    let error_class = match &error {
        AdapterError::InvalidCrdObject(_) => "invalid_crd_object",
        AdapterError::PartialObservedState(_) => "partial_observed_state",
        AdapterError::DomainActuation(_) => "domain_actuation",
    };
    ReconcileCycleFailure {
        planned_actions,
        executed_actions,
        wide_event: Box::new(ReconcileWideEvent {
            event_name: "secrets_kms_operator_reconcile".to_owned(),
            status: "failed".to_owned(),
            action_count: planned_actions,
            executed_count: executed_actions,
            error_class: Some(error_class.to_owned()),
        }),
        error,
    }
}

pub fn reconcile_observation_failure(error: AdapterError) -> ReconcileCycleFailure {
    reconcile_failure(0, 0, error)
}

fn adapter_error_from_kube_operator_error(error: &KubeOperatorError) -> AdapterError {
    match error {
        KubeOperatorError::Adapter(error) => error.clone(),
        KubeOperatorError::Kube(message) => AdapterError::PartialObservedState(message.clone()),
        KubeOperatorError::ActuatorLockPoisoned => {
            AdapterError::DomainActuation("actuator lock poisoned".to_owned())
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExponentialBackoff {
    pub base_seconds: u64,
    pub max_seconds: u64,
}

impl ExponentialBackoff {
    pub fn delay_seconds(&self, attempt: u32) -> u64 {
        let multiplier = 1_u64.checked_shl(attempt).unwrap_or(u64::MAX);
        self.base_seconds
            .saturating_mul(multiplier)
            .min(self.max_seconds)
    }
}

pub struct DomainKmsOperatorActuator<R> {
    repo: R,
    remembered_key_ring_statuses: BTreeMap<String, RememberedKeyRingStatusPatch>,
    remembered_sealing_root_statuses: BTreeMap<String, Value>,
}

impl<R> DomainKmsOperatorActuator<R> {
    pub fn new(repo: R) -> Self {
        Self {
            repo,
            remembered_key_ring_statuses: BTreeMap::new(),
            remembered_sealing_root_statuses: BTreeMap::new(),
        }
    }

    pub fn into_inner(self) -> R {
        self.repo
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomainKeySnapshot {
    pub current_version: u32,          // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

pub trait KmsOperatorDomainRepo {
    fn key_snapshot(&self, key_id: &KmsKeyId) -> Option<DomainKeySnapshot>;
    fn key_id_for_key_ring(&self, key_ring_name: &str, tenant_id: &str) -> Option<KmsKeyId>;
    fn sealing_root_exists(&self, root_ref: &str) -> bool;
    fn create_operator_key(&mut self, input: KmsKeyCreate) -> Result<(), AdapterError>;
    fn rotate_operator_key(
        &mut self,
        key_id: &KmsKeyId,
        updated_at_epoch_seconds: u64,
    ) -> Result<(), AdapterError>;
    fn create_operator_sealing_root(
        &mut self,
        input: KmsSealingRootCreate,
    ) -> Result<(), AdapterError>;
    fn demote_operator_key_version(
        &mut self,
        input: KeyVersionDemotionRequest,
    ) -> Result<(), AdapterError>;
    fn quarantine_operator_key_ring(
        &mut self,
        input: KeyRingQuarantineRequest,
    ) -> Result<(), AdapterError>;
}

impl KmsOperatorDomainRepo for CloudKmsDirectory {
    fn key_snapshot(&self, key_id: &KmsKeyId) -> Option<DomainKeySnapshot> {
        self.keys()
            .find(|key| &key.key_id.value == key_id)
            .map(|key| DomainKeySnapshot {
                current_version: key.current_version.value,
                created_at_epoch_seconds: key.created_at_epoch_seconds.value,
                updated_at_epoch_seconds: key.updated_at_epoch_seconds.value,
            })
    }

    fn key_id_for_key_ring(&self, key_ring_name: &str, tenant_id: &str) -> Option<KmsKeyId> {
        self.keys()
            .find(|key| {
                key.tenant_id.value == tenant_id
                    && key
                        .key_id
                        .value
                        .name()
                        .is_ok_and(|name| name == key_ring_name)
            })
            .map(|key| key.key_id.value.clone())
    }

    fn sealing_root_exists(&self, root_ref: &str) -> bool {
        self.sealing_roots()
            .any(|sealing_root| sealing_root.root_ref.value.value == root_ref)
    }

    fn create_operator_key(&mut self, input: KmsKeyCreate) -> Result<(), AdapterError> {
        self.create_key(input).map_err(map_domain_error)?;
        Ok(())
    }

    fn rotate_operator_key(
        &mut self,
        key_id: &KmsKeyId,
        updated_at_epoch_seconds: u64,
    ) -> Result<(), AdapterError> {
        self.rotate_key(key_id, updated_at_epoch_seconds)
            .map_err(map_domain_error)?;
        Ok(())
    }

    fn create_operator_sealing_root(
        &mut self,
        input: KmsSealingRootCreate,
    ) -> Result<(), AdapterError> {
        self.create_sealing_root(input).map_err(map_domain_error)?;
        Ok(())
    }

    fn demote_operator_key_version(
        &mut self,
        input: KeyVersionDemotionRequest,
    ) -> Result<(), AdapterError> {
        self.demote_key_version(input).map_err(map_domain_error)?;
        Ok(())
    }

    fn quarantine_operator_key_ring(
        &mut self,
        input: KeyRingQuarantineRequest,
    ) -> Result<(), AdapterError> {
        self.quarantine_key_ring(input).map_err(map_domain_error)?;
        Ok(())
    }
}

pub struct PersistentCloudKmsDirectory {
    path: PathBuf,
    directory: CloudKmsDirectory,
}

impl PersistentCloudKmsDirectory {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, AdapterError> {
        let path = path.as_ref().to_path_buf();
        let directory = if path.is_file() {
            let raw = fs::read_to_string(&path).map_err(persistence_error)?;
            if raw.trim().is_empty() {
                CloudKmsDirectory::default()
            } else {
                PersistentCloudKmsDirectorySnapshot::from_json(&raw)?.into_directory()?
            }
        } else {
            CloudKmsDirectory::default()
        };
        Ok(Self { path, directory })
    }

    pub fn keys(&self) -> impl Iterator<Item = &KmsKey> {
        self.directory.keys()
    }

    pub fn key_version_lifecycle(&self) -> impl Iterator<Item = &KmsKeyVersionLifecycle> {
        self.directory.key_version_lifecycle()
    }

    pub fn sealing_roots(&self) -> impl Iterator<Item = &KmsSealingRoot> {
        self.directory.sealing_roots()
    }

    fn persist(&self) -> Result<(), AdapterError> {
        if let Some(parent) = self.path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent).map_err(persistence_error)?;
        }
        let encoded = PersistentCloudKmsDirectorySnapshot::from_directory(&self.directory)?
            .to_json()
            .map_err(persistence_error)?;
        let mut tmp_path = self.path.clone();
        tmp_path.set_extension("json.tmp");
        fs::write(&tmp_path, encoded).map_err(persistence_error)?;
        fs::rename(&tmp_path, &self.path).map_err(persistence_error)?;
        Ok(())
    }
}

impl KmsOperatorDomainRepo for PersistentCloudKmsDirectory {
    fn key_snapshot(&self, key_id: &KmsKeyId) -> Option<DomainKeySnapshot> {
        self.directory.key_snapshot(key_id)
    }

    fn key_id_for_key_ring(&self, key_ring_name: &str, tenant_id: &str) -> Option<KmsKeyId> {
        self.directory.key_id_for_key_ring(key_ring_name, tenant_id)
    }

    fn sealing_root_exists(&self, root_ref: &str) -> bool {
        self.directory.sealing_root_exists(root_ref)
    }

    fn create_operator_key(&mut self, input: KmsKeyCreate) -> Result<(), AdapterError> {
        self.directory.create_operator_key(input)?;
        self.persist()
    }

    fn rotate_operator_key(
        &mut self,
        key_id: &KmsKeyId,
        updated_at_epoch_seconds: u64,
    ) -> Result<(), AdapterError> {
        self.directory
            .rotate_operator_key(key_id, updated_at_epoch_seconds)?;
        self.persist()
    }

    fn create_operator_sealing_root(
        &mut self,
        input: KmsSealingRootCreate,
    ) -> Result<(), AdapterError> {
        self.directory.create_operator_sealing_root(input)?;
        self.persist()
    }

    fn demote_operator_key_version(
        &mut self,
        input: KeyVersionDemotionRequest,
    ) -> Result<(), AdapterError> {
        self.directory.demote_operator_key_version(input)?;
        self.persist()
    }

    fn quarantine_operator_key_ring(
        &mut self,
        input: KeyRingQuarantineRequest,
    ) -> Result<(), AdapterError> {
        self.directory.quarantine_operator_key_ring(input)?;
        self.persist()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct PersistentCloudKmsDirectorySnapshot {
    schema_version: u32,
    keys: Vec<PersistentKeySnapshot>,
    sealing_roots: Vec<PersistentSealingRootSnapshot>,
    key_versions: Vec<PersistentKeyVersionSnapshot>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct PersistentKeySnapshot {
    resource_id: String,
    key_id: String,
    tenant_id: String,
    region: String,
    cell_id: String,
    hsm_partition_ref: String,
    origin: String,
    usage: String,
    hsm_validation: String,
    residency: String,
    data_class: String,
    state: String,
    current_version: u32,
    rotation_period_days: Option<u16>,
    created_at_epoch_seconds: u64,
    updated_at_epoch_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct PersistentSealingRootSnapshot {
    root_ref: String,
    tenant_id: String,
    region: String,
    cell_id: String,
    active_version: u32,
    rotate_after_seconds: u64,
    created_at_epoch_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct PersistentKeyVersionSnapshot {
    key_id: String,
    tenant_id: String,
    version: u32,
    state: String,
    reason: Option<String>,
    activated_at_epoch_seconds: u64,
    decrypt_only_since_epoch_seconds: Option<u64>,
    quarantined_since_epoch_seconds: Option<u64>,
}

impl PersistentCloudKmsDirectorySnapshot {
    fn from_json(raw: &str) -> Result<Self, AdapterError> {
        serde_json::from_str(raw)
            .map_err(|error| AdapterError::DomainActuation(format!("state decode failed: {error}")))
    }

    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    fn from_directory(directory: &CloudKmsDirectory) -> Result<Self, AdapterError> {
        Ok(Self {
            schema_version: 1,
            keys: directory
                .keys()
                .map(PersistentKeySnapshot::from_key)
                .collect::<Result<Vec<_>, _>>()?,
            sealing_roots: directory
                .sealing_roots()
                .map(PersistentSealingRootSnapshot::from_sealing_root)
                .collect(),
            key_versions: directory
                .key_version_lifecycle()
                .map(PersistentKeyVersionSnapshot::from_lifecycle)
                .collect(),
        })
    }

    fn into_directory(self) -> Result<CloudKmsDirectory, AdapterError> {
        if self.schema_version != 1 {
            return Err(AdapterError::DomainActuation(format!(
                "unsupported persistent KMS directory schema version {}",
                self.schema_version
            )));
        }
        let mut directory = CloudKmsDirectory::default();
        for sealing_root in self.sealing_roots {
            directory
                .create_sealing_root(sealing_root.into_create())
                .map_err(map_domain_error)?;
        }
        for key in self.keys {
            let key_id = KmsKeyId::new(key.key_id.clone()).map_err(map_domain_error)?;
            let final_state = key_state_from_label(&key.state)?;
            let mut create = key.to_create()?;
            if key.current_version > 1 && final_state == KmsKeyState::Disabled {
                create.state = KmsKeyState::Enabled;
            }
            directory.create_key(create).map_err(map_domain_error)?;
            for version in 2..=key.current_version {
                let activation = self
                    .key_versions
                    .iter()
                    .find(|candidate| {
                        candidate.key_id == key.key_id && candidate.version == version
                    })
                    .map(|candidate| candidate.activated_at_epoch_seconds)
                    .unwrap_or(key.updated_at_epoch_seconds.saturating_add(version as u64));
                directory
                    .rotate_key(&key_id, activation)
                    .map_err(map_domain_error)?;
            }
            if final_state == KmsKeyState::Disabled {
                directory
                    .quarantine_key_ring(KeyRingQuarantineRequest {
                        key_id: key.key_id,
                        tenant_id: key.tenant_id,
                        reason: "restored disabled key state".to_owned(),
                        effective_at_epoch_seconds: key.updated_at_epoch_seconds,
                    })
                    .map_err(map_domain_error)?;
            }
        }
        for version in self.key_versions {
            match key_version_lifecycle_state_from_label(&version.state)? {
                KmsKeyVersionLifecycleState::DecryptOnly => {
                    directory
                        .demote_key_version(KeyVersionDemotionRequest {
                            key_id: version.key_id,
                            tenant_id: version.tenant_id,
                            version: version.version,
                            reason: version
                                .reason
                                .unwrap_or_else(|| "restored decrypt-only state".to_owned()),
                            effective_at_epoch_seconds: version
                                .decrypt_only_since_epoch_seconds
                                .unwrap_or(version.activated_at_epoch_seconds),
                        })
                        .map_err(map_domain_error)?;
                }
                KmsKeyVersionLifecycleState::Quarantined => {
                    directory
                        .quarantine_key_ring(KeyRingQuarantineRequest {
                            key_id: version.key_id,
                            tenant_id: version.tenant_id,
                            reason: version
                                .reason
                                .unwrap_or_else(|| "restored quarantined state".to_owned()),
                            effective_at_epoch_seconds: version
                                .quarantined_since_epoch_seconds
                                .unwrap_or(version.activated_at_epoch_seconds),
                        })
                        .map_err(map_domain_error)?;
                }
                KmsKeyVersionLifecycleState::Active | KmsKeyVersionLifecycleState::Destroyed => {}
            }
        }
        Ok(directory)
    }
}

impl PersistentKeySnapshot {
    fn from_key(key: &KmsKey) -> Result<Self, AdapterError> {
        Ok(Self {
            resource_id: key.resource_id.value.value.clone(),
            key_id: key.key_id.value.value.clone(),
            tenant_id: key.tenant_id.value.clone(),
            region: key.region.value.value.clone(),
            cell_id: key.cell_id.value.value.clone(),
            hsm_partition_ref: key.hsm_partition_ref.value.value.clone(),
            origin: domain_origin_label(key.origin.value).to_owned(),
            usage: domain_usage_label(key.usage.value).to_owned(),
            hsm_validation: domain_hsm_validation_label(key.hsm_validation.value).to_owned(),
            residency: domain_residency_label(&key.residency.value)?.to_owned(),
            data_class: key.data_class.value.data_class().label().to_owned(),
            state: key_state_label(key.state.value).to_owned(),
            current_version: key.current_version.value,
            rotation_period_days: key.rotation_period_days.value,
            created_at_epoch_seconds: key.created_at_epoch_seconds.value,
            updated_at_epoch_seconds: key.updated_at_epoch_seconds.value,
        })
    }

    fn to_create(&self) -> Result<KmsKeyCreate, AdapterError> {
        Ok(KmsKeyCreate {
            resource_id: self.resource_id.clone(),
            key_id: self.key_id.clone(),
            tenant_id: self.tenant_id.clone(),
            region: self.region.clone(),
            cell_id: self.cell_id.clone(),
            hsm_partition_ref: self.hsm_partition_ref.clone(),
            origin: domain_origin_from_label(&self.origin)?,
            usage: domain_usage_from_label(&self.usage)?,
            hsm_validation: domain_hsm_validation_from_label(&self.hsm_validation)?,
            residency: domain_residency_from_label(&self.residency)?,
            data_class: domain_data_class_from_label(&self.data_class)?,
            state: key_state_from_label(&self.state)?,
            rotation_period_days: self.rotation_period_days,
            created_at_epoch_seconds: self.created_at_epoch_seconds,
        })
    }
}

impl PersistentSealingRootSnapshot {
    fn from_sealing_root(sealing_root: &KmsSealingRoot) -> Self {
        Self {
            root_ref: sealing_root.root_ref.value.value.clone(),
            tenant_id: sealing_root.tenant_id.value.clone(),
            region: sealing_root.region.value.value.clone(),
            cell_id: sealing_root.cell_id.value.value.clone(),
            active_version: sealing_root.active_version.value,
            rotate_after_seconds: sealing_root.rotate_after_seconds.value,
            created_at_epoch_seconds: sealing_root.created_at_epoch_seconds.value,
        }
    }

    fn into_create(self) -> KmsSealingRootCreate {
        KmsSealingRootCreate {
            root_ref: self.root_ref,
            tenant_id: self.tenant_id,
            region: self.region,
            cell_id: self.cell_id,
            active_version: self.active_version,
            rotate_after_seconds: self.rotate_after_seconds,
            created_at_epoch_seconds: self.created_at_epoch_seconds,
        }
    }
}

impl PersistentKeyVersionSnapshot {
    fn from_lifecycle(lifecycle: &KmsKeyVersionLifecycle) -> Self {
        Self {
            key_id: lifecycle.key_id.value.value.clone(),
            tenant_id: lifecycle.tenant_id.value.clone(),
            version: lifecycle.version.value,
            state: key_version_lifecycle_state_label(lifecycle.state.value).to_owned(),
            reason: lifecycle.reason.value.clone(),
            activated_at_epoch_seconds: lifecycle.activated_at_epoch_seconds.value,
            decrypt_only_since_epoch_seconds: lifecycle.decrypt_only_since_epoch_seconds.value,
            quarantined_since_epoch_seconds: lifecycle.quarantined_since_epoch_seconds.value,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RememberedKeyRingStatusPatch {
    observed_active_version: Option<u32>,
    status: Value,
}

impl<R: KmsOperatorDomainRepo> DomainKmsOperatorActuator<R> {
    fn key_snapshot_for_key_ring(
        &self,
        key_ring: &KeyRing,
    ) -> Result<Option<DomainKeySnapshot>, AdapterError> {
        let key_id = domain_key_id_for_key_ring(key_ring)?;
        Ok(self.repo.key_snapshot(&key_id))
    }

    fn remembered_patch(
        &self,
        key_ring_name: &str,
        observed_active_version: Option<u32>,
    ) -> Option<KmsStatusPatch> {
        self.remembered_key_ring_statuses
            .get(key_ring_name)
            .filter(|patch| patch.observed_active_version == observed_active_version)
            .map(|patch| KmsStatusPatch {
                kind: KmsStatusPatchKind::KeyRing,
                name: key_ring_name.to_owned(),
                status: patch.status.clone(),
            })
    }

    fn remembered_sealing_root_patch(&self, name: &str) -> Option<KmsStatusPatch> {
        self.remembered_sealing_root_statuses
            .get(name)
            .map(|status| KmsStatusPatch {
                kind: KmsStatusPatchKind::SealingRoot,
                name: name.to_owned(),
                status: status.clone(),
            })
    }
}

impl<R: KmsOperatorDomainRepo> KmsOperatorActuator for DomainKmsOperatorActuator<R> {
    fn desired_state_for_observed(&self, observed: &ObservedState) -> DesiredState {
        desired_state_from_observed(observed)
    }

    fn execute(&mut self, action: &Action) -> Result<(), AdapterError> {
        match action {
            Action::CreateKeyRing {
                key_ring,
                requested_at_epoch_seconds,
            } => {
                if self.key_snapshot_for_key_ring(key_ring)?.is_some() {
                    return Ok(());
                }
                self.repo.create_operator_key(key_create_from_key_ring(
                    key_ring,
                    *requested_at_epoch_seconds,
                )?)?;
                Ok(())
            }
            Action::RotateKeyVersion {
                key_ring,
                observed_active_version,
                requested_at_epoch_seconds,
                ..
            } => {
                let key_id = domain_key_id_for_key_ring(key_ring)?;
                if self
                    .repo
                    .key_snapshot(&key_id)
                    .is_some_and(|snapshot| snapshot.current_version > *observed_active_version)
                {
                    return Ok(());
                }
                self.repo
                    .rotate_operator_key(&key_id, *requested_at_epoch_seconds)?;
                Ok(())
            }
            Action::CreateSealingRoot { sealing_root } => {
                if self.repo.sealing_root_exists(&sealing_root.root_ref) {
                    return Ok(());
                }
                self.repo
                    .create_operator_sealing_root(sealing_root_create(sealing_root)?)?;
                Ok(())
            }
            Action::DemoteKeyVersionToDecryptOnly {
                key_ring_name,
                tenant_id,
                version,
                reason,
                effective_at_epoch_seconds,
            } => {
                let key_id = self
                    .repo
                    .key_id_for_key_ring(key_ring_name, tenant_id)
                    .ok_or_else(|| {
                        AdapterError::DomainActuation(format!(
                            "domain key for KmsKeyRing {key_ring_name} tenant {tenant_id} was not found"
                        ))
                    })?;
                self.repo
                    .demote_operator_key_version(KeyVersionDemotionRequest {
                        key_id: key_id.value,
                        tenant_id: tenant_id.clone(),
                        version: *version,
                        reason: reason.clone(),
                        effective_at_epoch_seconds: *effective_at_epoch_seconds,
                    })?;
                Ok(())
            }
            Action::QuarantineKeyRing {
                key_ring_name,
                tenant_id,
                reason,
                effective_at_epoch_seconds,
            } => {
                let key_id = self
                    .repo
                    .key_id_for_key_ring(key_ring_name, tenant_id)
                    .ok_or_else(|| {
                        AdapterError::DomainActuation(format!(
                            "domain key for KmsKeyRing {key_ring_name} tenant {tenant_id} was not found"
                        ))
                    })?;
                self.repo
                    .quarantine_operator_key_ring(KeyRingQuarantineRequest {
                        key_id: key_id.value,
                        tenant_id: tenant_id.clone(),
                        reason: reason.clone(),
                        effective_at_epoch_seconds: *effective_at_epoch_seconds,
                    })?;
                Ok(())
            }
            Action::QuarantineObservedState { .. } => Err(AdapterError::PartialObservedState(
                "refusing to act on quarantined observed state".to_owned(),
            )),
        }
    }

    fn status_patches_for_actions(
        &self,
        observed: &ObservedState,
        actions: &[Action],
    ) -> Result<Vec<KmsStatusPatch>, AdapterError> {
        let mut patches = Vec::new();
        for action in actions {
            match action {
                Action::CreateKeyRing { key_ring, .. } => {
                    if let Some(patch) = self.remembered_patch(&key_ring.name, None) {
                        patches.push(patch);
                    } else if let Some(snapshot) = self.key_snapshot_for_key_ring(key_ring)? {
                        let active_at_epoch_seconds = if snapshot.current_version == 1 {
                            snapshot.created_at_epoch_seconds
                        } else {
                            snapshot.updated_at_epoch_seconds
                        };
                        patches.push(key_ring_active_status_patch(
                            key_ring,
                            snapshot.current_version,
                            active_at_epoch_seconds,
                        ));
                    } else {
                        patches.extend(status_patches_for_actions(
                            observed,
                            std::slice::from_ref(action),
                        )?);
                    }
                }
                Action::RotateKeyVersion {
                    key_ring,
                    observed_active_version,
                    ..
                } => {
                    if let Some(patch) =
                        self.remembered_patch(&key_ring.name, Some(*observed_active_version))
                    {
                        patches.push(patch);
                    } else if let Some(snapshot) = self.key_snapshot_for_key_ring(key_ring)? {
                        patches.push(key_ring_rotated_status_patch_with_version(
                            observed,
                            key_ring,
                            snapshot.current_version,
                            snapshot.updated_at_epoch_seconds,
                        )?);
                    } else {
                        patches.extend(status_patches_for_actions(
                            observed,
                            std::slice::from_ref(action),
                        )?);
                    }
                }
                Action::CreateSealingRoot { sealing_root } => {
                    if let Some(patch) = self.remembered_sealing_root_patch(&sealing_root.name) {
                        patches.push(patch);
                    } else {
                        patches.push(sealing_root_created_status_patch(sealing_root));
                    }
                }
                Action::DemoteKeyVersionToDecryptOnly { .. } | Action::QuarantineKeyRing { .. } => {
                    patches.extend(status_patches_for_actions(
                        observed,
                        std::slice::from_ref(action),
                    )?);
                }
                Action::QuarantineObservedState { .. } => {}
            }
        }
        Ok(patches)
    }

    fn remember_status_patches(&mut self, actions: &[Action], patches: &[KmsStatusPatch]) {
        for action in actions {
            let (key_ring_name, observed_active_version) = match action {
                Action::CreateKeyRing { key_ring, .. } => (&key_ring.name, None),
                Action::RotateKeyVersion {
                    key_ring,
                    observed_active_version,
                    ..
                } => (&key_ring.name, Some(*observed_active_version)),
                Action::CreateSealingRoot { sealing_root } => {
                    if let Some(patch) = patches.iter().find(|patch| {
                        patch.kind == KmsStatusPatchKind::SealingRoot
                            && patch.name == sealing_root.name
                    }) {
                        self.remembered_sealing_root_statuses
                            .insert(sealing_root.name.clone(), patch.status.clone());
                    }
                    continue;
                }
                Action::DemoteKeyVersionToDecryptOnly { .. }
                | Action::QuarantineKeyRing { .. }
                | Action::QuarantineObservedState { .. } => continue,
            };
            if let Some(patch) = patches.iter().find(|patch| {
                patch.kind == KmsStatusPatchKind::KeyRing && patch.name == *key_ring_name
            }) {
                self.remembered_key_ring_statuses.insert(
                    key_ring_name.clone(),
                    RememberedKeyRingStatusPatch {
                        observed_active_version,
                        status: patch.status.clone(),
                    },
                );
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawObject {
    kind: String,
    metadata: RawMetadata,
    spec: Value,
    status: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct RawMetadata {
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawKeyRingSpec {
    tenant_id: String,
    region: String,
    cell_id: String,
    hsm_partition_ref: String,
    origin: String,
    usage: String,
    hsm_validation: String,
    residency: String,
    data_class: String,
    rotation_policy: RawRotationPolicy,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawRotationPolicy {
    rotate_after_seconds: u64,
    decrypt_only_grace_seconds: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawKeyRingStatus {
    health: Option<RawHealth>,
    versions: Option<Vec<RawKeyVersion>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawKeyVersion {
    version: u32,
    state: String,
    created_at_epoch_seconds: u64,
    activated_at_epoch_seconds: u64,
    decrypt_only_since_epoch_seconds: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawSealingRootSpec {
    tenant_id: String,
    region: String,
    cell_id: String,
    root_ref: String,
    active_version: u32,
    rotate_after_seconds: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawSealingRootStatus {
    observed_version: Option<u32>,
    health: Option<RawHealth>,
}

#[derive(Debug, Deserialize)]
struct RawHealth {
    state: String,
    reason: Option<String>,
}

fn project_key_ring(raw: RawObject) -> Result<ObservedKeyRing, AdapterError> {
    let spec: RawKeyRingSpec = serde_json::from_value(raw.spec)
        .map_err(|e| AdapterError::InvalidCrdObject(e.to_string()))?;
    let status = match raw.status {
        Some(status) => Some(
            serde_json::from_value::<RawKeyRingStatus>(status)
                .map_err(|e| AdapterError::InvalidCrdObject(e.to_string()))?,
        ),
        None => None,
    };
    let (raw_versions, health) = match status {
        Some(status) => (
            status.versions.ok_or_else(|| {
                AdapterError::PartialObservedState(format!(
                    "KmsKeyRing {} status.versions is missing",
                    raw.metadata.name
                ))
            })?,
            Some(status.health.ok_or_else(|| {
                AdapterError::PartialObservedState(format!(
                    "KmsKeyRing {} status.health is missing",
                    raw.metadata.name
                ))
            })?),
        ),
        None => (Vec::new(), None),
    };
    let versions = raw_versions
        .into_iter()
        .map(project_key_version)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ObservedKeyRing {
        desired: KeyRing {
            name: raw.metadata.name,
            tenant_id: spec.tenant_id,
            region: spec.region,
            cell_id: spec.cell_id,
            hsm_partition_ref: spec.hsm_partition_ref,
            origin: parse_key_origin(&spec.origin)?,
            usage: parse_key_usage(&spec.usage)?,
            hsm_validation: parse_hsm_validation(&spec.hsm_validation)?,
            residency: parse_residency(&spec.residency)?,
            data_class: parse_data_class(&spec.data_class)?,
            rotation_policy: secrets_kms_operator_kernel::KeyVersionRotationPolicy {
                rotate_after_seconds: spec.rotation_policy.rotate_after_seconds,
                decrypt_only_grace_seconds: spec.rotation_policy.decrypt_only_grace_seconds,
            },
        },
        versions,
        health: project_health(health)?,
    })
}

fn project_sealing_root(raw: RawObject) -> Result<ObservedSealingRoot, AdapterError> {
    let name = raw.metadata.name;
    let spec: RawSealingRootSpec = serde_json::from_value(raw.spec)
        .map_err(|e| AdapterError::InvalidCrdObject(e.to_string()))?;
    let status = match raw.status {
        Some(status) => Some(
            serde_json::from_value::<RawSealingRootStatus>(status)
                .map_err(|e| AdapterError::InvalidCrdObject(e.to_string()))?,
        ),
        None => None,
    };
    let sealing_root = SealingRoot {
        name: name.clone(),
        tenant_id: spec.tenant_id,
        region: spec.region,
        cell_id: spec.cell_id,
        root_ref: spec.root_ref,
        active_version: spec.active_version,
        rotate_after_seconds: spec.rotate_after_seconds,
    };
    let (observed_version, health) = match status {
        Some(status) => (
            status.observed_version.ok_or_else(|| {
                AdapterError::PartialObservedState(format!(
                    "KmsSealingRoot {} status.observedVersion is missing",
                    name
                ))
            })?,
            Some(status.health.ok_or_else(|| {
                AdapterError::PartialObservedState(format!(
                    "KmsSealingRoot {} status.health is missing",
                    name
                ))
            })?),
        ),
        None => (0, None),
    };
    Ok(ObservedSealingRoot {
        observed_version,
        desired: sealing_root,
        health: project_health(health)?,
    })
}

fn project_key_version(raw: RawKeyVersion) -> Result<ObservedKeyVersion, AdapterError> {
    Ok(ObservedKeyVersion {
        version: raw.version,
        state: parse_key_version_state(&raw.state)?,
        created_at_epoch_seconds: raw.created_at_epoch_seconds,
        activated_at_epoch_seconds: raw.activated_at_epoch_seconds,
        decrypt_only_since_epoch_seconds: raw.decrypt_only_since_epoch_seconds,
    })
}

fn project_health(raw: Option<RawHealth>) -> Result<ObservedHealth, AdapterError> {
    match raw {
        Some(health) if health.state == "Healthy" || health.state == "healthy" => {
            Ok(ObservedHealth::Healthy)
        }
        Some(health) if health.state == "Ambiguous" || health.state == "ambiguous" => {
            Ok(ObservedHealth::Ambiguous(match health.reason {
                Some(reason) => reason,
                None => "ambiguous".to_owned(),
            }))
        }
        Some(health) if health.state == "Compromised" || health.state == "compromised" => {
            Ok(ObservedHealth::Compromised(match health.reason {
                Some(reason) => reason,
                None => "compromised".to_owned(),
            }))
        }
        Some(health) => Err(AdapterError::InvalidCrdObject(format!(
            "invalid health state {}",
            health.state
        ))),
        None => Ok(ObservedHealth::Healthy),
    }
}

fn parse_key_origin(value: &str) -> Result<KeyOrigin, AdapterError> {
    match value {
        "OyatieManaged" | "oyatieManaged" | "oyatie_managed" => Ok(KeyOrigin::OyatieManaged),
        "Byok" | "BYOK" | "byok" => Ok(KeyOrigin::Byok),
        "Hyok" | "HYOK" | "hyok" => Ok(KeyOrigin::Hyok),
        other => Err(AdapterError::InvalidCrdObject(format!(
            "invalid origin {other}"
        ))),
    }
}

fn parse_key_usage(value: &str) -> Result<KeyUsage, AdapterError> {
    match value {
        "EncryptDecrypt" | "encrypt_decrypt" => Ok(KeyUsage::EncryptDecrypt),
        "SignVerify" | "sign_verify" => Ok(KeyUsage::SignVerify),
        other => Err(AdapterError::InvalidCrdObject(format!(
            "invalid usage {other}"
        ))),
    }
}

fn parse_hsm_validation(value: &str) -> Result<HsmValidation, AdapterError> {
    match value {
        "PackEnhancedFips1403Level3" | "pack_enhanced_fips1403_level3" => {
            Ok(HsmValidation::PackEnhancedFips1403Level3)
        }
        "Fips1403Level3" | "fips1403_level3" => Ok(HsmValidation::Fips1403Level3),
        "Cryptrec" | "cryptrec" => Ok(HsmValidation::Cryptrec),
        "CommonCriteriaEal4" | "common_criteria_eal4" => Ok(HsmValidation::CommonCriteriaEal4),
        "PciHsm" | "pci_hsm" => Ok(HsmValidation::PciHsm),
        other => Err(AdapterError::InvalidCrdObject(format!(
            "invalid hsm validation {other}"
        ))),
    }
}

fn parse_residency(value: &str) -> Result<ResidencyMode, AdapterError> {
    match value {
        "StrictHomeRegion" | "strict_home_region" => Ok(ResidencyMode::StrictHomeRegion),
        "HomeWithRecoveryFailover" | "home_with_recovery_failover" => {
            Ok(ResidencyMode::HomeWithRecoveryFailover)
        }
        "Global" | "global" => Ok(ResidencyMode::Global),
        other => Err(AdapterError::InvalidCrdObject(format!(
            "invalid residency {other}"
        ))),
    }
}

fn parse_data_class(value: &str) -> Result<DataClassLabel, AdapterError> {
    match value {
        "Public" | "PUBLIC" | "public" => Ok(DataClassLabel::Public),
        "InternalOnly" | "INTERNAL_ONLY" | "internal_only" => Ok(DataClassLabel::InternalOnly),
        "PiiIdentifying" | "PII_IDENTIFYING" | "pii_identifying" => {
            Ok(DataClassLabel::PiiIdentifying)
        }
        "Phi" | "PHI" | "phi" => Ok(DataClassLabel::Phi),
        "Pci" | "PCI" | "pci" => Ok(DataClassLabel::Pci),
        "Secret" | "SECRET" | "secret" => Ok(DataClassLabel::Secret),
        "Audit" | "AUDIT" | "audit" => Ok(DataClassLabel::Audit),
        other => Err(AdapterError::InvalidCrdObject(format!(
            "invalid data class {other}"
        ))),
    }
}

fn parse_key_version_state(value: &str) -> Result<KeyVersionState, AdapterError> {
    match value {
        "Pending" | "pending" => Ok(KeyVersionState::Pending),
        "Active" | "active" => Ok(KeyVersionState::Active),
        "DecryptOnly" | "decrypt_only" => Ok(KeyVersionState::DecryptOnly),
        "Quarantined" | "quarantined" => Ok(KeyVersionState::Quarantined),
        "Destroyed" | "destroyed" => Ok(KeyVersionState::Destroyed),
        other => Err(AdapterError::InvalidCrdObject(format!(
            "invalid key version state {other}"
        ))),
    }
}

pub fn status_patches_for_actions(
    observed: &ObservedState,
    actions: &[Action],
) -> Result<Vec<KmsStatusPatch>, AdapterError> {
    actions
        .iter()
        .filter_map(|action| match action {
            Action::CreateKeyRing {
                key_ring,
                requested_at_epoch_seconds,
            } => Some(Ok(key_ring_created_status_patch(
                key_ring,
                *requested_at_epoch_seconds,
            ))),
            Action::RotateKeyVersion {
                key_ring,
                requested_at_epoch_seconds,
                ..
            } => Some(key_ring_rotated_status_patch(
                observed,
                key_ring,
                *requested_at_epoch_seconds,
            )),
            Action::CreateSealingRoot { sealing_root } => {
                Some(Ok(sealing_root_created_status_patch(sealing_root)))
            }
            Action::DemoteKeyVersionToDecryptOnly {
                key_ring_name,
                tenant_id,
                version,
                effective_at_epoch_seconds,
                ..
            } => Some(key_ring_demoted_status_patch(
                observed,
                key_ring_name,
                tenant_id,
                *version,
                *effective_at_epoch_seconds,
            )),
            Action::QuarantineKeyRing {
                key_ring_name,
                tenant_id,
                reason,
                effective_at_epoch_seconds,
            } => Some(key_ring_quarantined_status_patch(
                observed,
                key_ring_name,
                tenant_id,
                reason,
                *effective_at_epoch_seconds,
            )),
            Action::QuarantineObservedState { .. } => None,
        })
        .collect()
}

fn key_ring_created_status_patch(
    key_ring: &KeyRing,
    requested_at_epoch_seconds: u64,
) -> KmsStatusPatch {
    key_ring_active_status_patch(key_ring, 1, requested_at_epoch_seconds)
}

fn key_ring_active_status_patch(
    key_ring: &KeyRing,
    active_version: u32,
    active_at_epoch_seconds: u64,
) -> KmsStatusPatch {
    KmsStatusPatch {
        kind: KmsStatusPatchKind::KeyRing,
        name: key_ring.name.clone(),
        status: json!({
            "health": {"state": "Healthy"},
            "versions": [{
                "version": active_version,
                "state": "Active",
                "createdAtEpochSeconds": active_at_epoch_seconds,
                "activatedAtEpochSeconds": active_at_epoch_seconds
            }]
        }),
    }
}

fn key_ring_rotated_status_patch(
    observed: &ObservedState,
    key_ring: &KeyRing,
    requested_at_epoch_seconds: u64,
) -> Result<KmsStatusPatch, AdapterError> {
    let observed_ring = observed
        .key_rings
        .iter()
        .find(|observed_ring| {
            observed_ring.desired.name == key_ring.name
                && observed_ring.desired.tenant_id == key_ring.tenant_id
        })
        .ok_or_else(|| {
            AdapterError::PartialObservedState(format!(
                "KmsKeyRing {} was not present for status rotation",
                key_ring.name
            ))
        })?;
    let next_version = observed_ring
        .versions
        .iter()
        .map(|version| version.version)
        .max()
        .unwrap_or(0)
        .saturating_add(1);
    key_ring_rotated_status_patch_with_version(
        observed,
        key_ring,
        next_version,
        requested_at_epoch_seconds,
    )
}

fn key_ring_rotated_status_patch_with_version(
    observed: &ObservedState,
    key_ring: &KeyRing,
    next_version: u32,
    requested_at_epoch_seconds: u64,
) -> Result<KmsStatusPatch, AdapterError> {
    let observed_ring = observed
        .key_rings
        .iter()
        .find(|observed_ring| {
            observed_ring.desired.name == key_ring.name
                && observed_ring.desired.tenant_id == key_ring.tenant_id
        })
        .ok_or_else(|| {
            AdapterError::PartialObservedState(format!(
                "KmsKeyRing {} was not present for status rotation",
                key_ring.name
            ))
        })?;
    let mut versions = observed_ring
        .versions
        .iter()
        .map(|version| {
            let mut projected = version.clone();
            if projected.state == KeyVersionState::Active {
                projected.state = KeyVersionState::DecryptOnly;
                projected.decrypt_only_since_epoch_seconds = Some(requested_at_epoch_seconds);
            }
            key_version_status_value(&projected)
        })
        .collect::<Vec<_>>();
    versions.push(json!({
        "version": next_version,
        "state": "Active",
        "createdAtEpochSeconds": requested_at_epoch_seconds,
        "activatedAtEpochSeconds": requested_at_epoch_seconds
    }));
    Ok(KmsStatusPatch {
        kind: KmsStatusPatchKind::KeyRing,
        name: key_ring.name.clone(),
        status: json!({
            "health": {"state": "Healthy"},
            "versions": versions
        }),
    })
}

fn sealing_root_created_status_patch(sealing_root: &SealingRoot) -> KmsStatusPatch {
    KmsStatusPatch {
        kind: KmsStatusPatchKind::SealingRoot,
        name: sealing_root.name.clone(),
        status: json!({
            "health": {"state": "Healthy"},
            "observedVersion": sealing_root.active_version
        }),
    }
}

fn key_ring_demoted_status_patch(
    observed: &ObservedState,
    key_ring_name: &str,
    tenant_id: &str,
    version: u32,
    effective_at_epoch_seconds: u64,
) -> Result<KmsStatusPatch, AdapterError> {
    let observed_ring = observed_key_ring(observed, key_ring_name, tenant_id)?;
    let versions = observed_ring
        .versions
        .iter()
        .map(|observed_version| {
            let mut projected = observed_version.clone();
            if projected.version == version {
                projected.state = KeyVersionState::DecryptOnly;
                projected.decrypt_only_since_epoch_seconds = Some(effective_at_epoch_seconds);
            }
            key_version_status_value(&projected)
        })
        .collect::<Vec<_>>();
    Ok(KmsStatusPatch {
        kind: KmsStatusPatchKind::KeyRing,
        name: key_ring_name.to_owned(),
        status: json!({
            "health": {"state": "Healthy"},
            "versions": versions
        }),
    })
}

fn key_ring_quarantined_status_patch(
    observed: &ObservedState,
    key_ring_name: &str,
    tenant_id: &str,
    reason: &str,
    effective_at_epoch_seconds: u64,
) -> Result<KmsStatusPatch, AdapterError> {
    let observed_ring = observed_key_ring(observed, key_ring_name, tenant_id)?;
    let versions = observed_ring
        .versions
        .iter()
        .map(|observed_version| {
            let mut projected = observed_version.clone();
            projected.state = KeyVersionState::Quarantined;
            projected.decrypt_only_since_epoch_seconds = Some(
                projected
                    .decrypt_only_since_epoch_seconds
                    .unwrap_or(effective_at_epoch_seconds),
            );
            key_version_status_value(&projected)
        })
        .collect::<Vec<_>>();
    Ok(KmsStatusPatch {
        kind: KmsStatusPatchKind::KeyRing,
        name: key_ring_name.to_owned(),
        status: json!({
            "health": {"state": "Compromised", "reason": reason},
            "versions": versions
        }),
    })
}

fn observed_key_ring<'a>(
    observed: &'a ObservedState,
    key_ring_name: &str,
    tenant_id: &str,
) -> Result<&'a ObservedKeyRing, AdapterError> {
    observed
        .key_rings
        .iter()
        .find(|observed_ring| {
            observed_ring.desired.name == key_ring_name
                && observed_ring.desired.tenant_id == tenant_id
        })
        .ok_or_else(|| {
            AdapterError::PartialObservedState(format!(
                "KmsKeyRing {key_ring_name} was not present for status patch"
            ))
        })
}

fn key_version_status_value(version: &ObservedKeyVersion) -> Value {
    let mut value = json!({
        "version": version.version,
        "state": key_version_state_label(version.state),
        "createdAtEpochSeconds": version.created_at_epoch_seconds,
        "activatedAtEpochSeconds": version.activated_at_epoch_seconds
    });
    if let Some(decrypt_only_since) = version.decrypt_only_since_epoch_seconds
        && let Value::Object(fields) = &mut value
    {
        fields.insert(
            "decryptOnlySinceEpochSeconds".to_owned(),
            Value::from(decrypt_only_since),
        );
    }
    value
}

fn key_version_state_label(state: KeyVersionState) -> &'static str {
    match state {
        KeyVersionState::Pending => "Pending",
        KeyVersionState::Active => "Active",
        KeyVersionState::DecryptOnly => "DecryptOnly",
        KeyVersionState::Quarantined => "Quarantined",
        KeyVersionState::Destroyed => "Destroyed",
    }
}

fn key_create_from_key_ring(
    key_ring: &KeyRing,
    created_at_epoch_seconds: u64,
) -> Result<KmsKeyCreate, AdapterError> {
    Ok(KmsKeyCreate {
        resource_id: resource_id_for(key_ring),
        key_id: key_id_for(
            key_ring.origin,
            &key_ring.region,
            &key_ring.tenant_id,
            &key_ring.name,
        ),
        tenant_id: key_ring.tenant_id.clone(),
        region: key_ring.region.clone(),
        cell_id: key_ring.cell_id.clone(),
        hsm_partition_ref: key_ring.hsm_partition_ref.clone(),
        origin: domain_origin(key_ring.origin),
        usage: domain_usage(key_ring.usage),
        hsm_validation: domain_hsm_validation(key_ring.hsm_validation),
        residency: domain_residency(key_ring.residency),
        data_class: domain_data_class(key_ring.data_class),
        state: KmsKeyState::Enabled,
        rotation_period_days: rotation_days(key_ring.rotation_policy.rotate_after_seconds),
        created_at_epoch_seconds,
    })
}

fn sealing_root_create(sealing_root: &SealingRoot) -> Result<KmsSealingRootCreate, AdapterError> {
    Ok(KmsSealingRootCreate {
        root_ref: sealing_root.root_ref.clone(),
        tenant_id: sealing_root.tenant_id.clone(),
        region: sealing_root.region.clone(),
        cell_id: sealing_root.cell_id.clone(),
        active_version: sealing_root.active_version,
        rotate_after_seconds: sealing_root.rotate_after_seconds,
        created_at_epoch_seconds: 0,
    })
}

fn domain_key_id_for_key_ring(key_ring: &KeyRing) -> Result<KmsKeyId, AdapterError> {
    KmsKeyId::new(key_id_for(
        key_ring.origin,
        &key_ring.region,
        &key_ring.tenant_id,
        &key_ring.name,
    ))
    .map_err(map_domain_error)
}

fn resource_id_for(key_ring: &KeyRing) -> String {
    format!(
        "oyatie:cloud:{}:{}:kms-key:{}",
        key_ring.region, key_ring.tenant_id, key_ring.name
    )
}

fn key_id_for(origin: KeyOrigin, region: &str, tenant_id: &str, name: &str) -> String {
    let prefix = match origin {
        KeyOrigin::OyatieManaged => "kms",
        KeyOrigin::Byok => "byok",
        KeyOrigin::Hyok => "hyok",
    };
    format!("{prefix}/{region}/{tenant_id}/{name}")
}

fn rotation_days(seconds: u64) -> Option<u16> {
    let days = seconds / 86_400;
    if days == 0 || days > u16::MAX as u64 {
        None
    } else {
        Some(days as u16)
    }
}

fn domain_origin(origin: KeyOrigin) -> DomainKeyOrigin {
    match origin {
        KeyOrigin::OyatieManaged => DomainKeyOrigin::OyatieManaged,
        KeyOrigin::Byok => DomainKeyOrigin::Byok,
        KeyOrigin::Hyok => DomainKeyOrigin::Hyok,
    }
}

fn domain_usage(usage: KeyUsage) -> DomainKeyUsage {
    match usage {
        KeyUsage::EncryptDecrypt => DomainKeyUsage::EncryptDecrypt,
        KeyUsage::SignVerify => DomainKeyUsage::SignVerify,
    }
}

fn domain_hsm_validation(validation: HsmValidation) -> DomainHsmValidation {
    match validation {
        HsmValidation::PackEnhancedFips1403Level3 => {
            DomainHsmValidation::PackEnhancedFips1403Level3
        }
        HsmValidation::Fips1403Level3 => DomainHsmValidation::Fips1403Level3,
        HsmValidation::Cryptrec => DomainHsmValidation::Cryptrec,
        HsmValidation::CommonCriteriaEal4 => DomainHsmValidation::CommonCriteriaEal4,
        HsmValidation::PciHsm => DomainHsmValidation::PciHsm,
    }
}

fn domain_residency(residency: ResidencyMode) -> ResidencyClass {
    match residency {
        ResidencyMode::StrictHomeRegion => ResidencyClass::StrictHomeRegion,
        ResidencyMode::HomeWithRecoveryFailover => ResidencyClass::HomeWithRecoveryFailover,
        ResidencyMode::Global => ResidencyClass::Global,
    }
}

fn domain_data_class(data_class: DataClassLabel) -> DataClass {
    match data_class {
        DataClassLabel::Public => DataClass::Public,
        DataClassLabel::InternalOnly => DataClass::InternalOnly,
        DataClassLabel::PiiIdentifying => DataClass::PiiIdentifying,
        DataClassLabel::Phi => DataClass::Phi,
        DataClassLabel::Pci => DataClass::Pci,
        DataClassLabel::Secret => DataClass::Secret,
        DataClassLabel::Audit => DataClass::Audit,
    }
}

fn domain_origin_label(origin: DomainKeyOrigin) -> &'static str {
    match origin {
        DomainKeyOrigin::OyatieManaged => "oyatie_managed",
        DomainKeyOrigin::Byok => "byok",
        DomainKeyOrigin::Hyok => "hyok",
    }
}

fn domain_origin_from_label(value: &str) -> Result<DomainKeyOrigin, AdapterError> {
    match value {
        "oyatie_managed" => Ok(DomainKeyOrigin::OyatieManaged),
        "byok" => Ok(DomainKeyOrigin::Byok),
        "hyok" => Ok(DomainKeyOrigin::Hyok),
        other => Err(AdapterError::DomainActuation(format!(
            "invalid persisted key origin {other}"
        ))),
    }
}

fn domain_usage_label(usage: DomainKeyUsage) -> &'static str {
    match usage {
        DomainKeyUsage::EncryptDecrypt => "encrypt_decrypt",
        DomainKeyUsage::SignVerify => "sign_verify",
    }
}

fn domain_usage_from_label(value: &str) -> Result<DomainKeyUsage, AdapterError> {
    match value {
        "encrypt_decrypt" => Ok(DomainKeyUsage::EncryptDecrypt),
        "sign_verify" => Ok(DomainKeyUsage::SignVerify),
        other => Err(AdapterError::DomainActuation(format!(
            "invalid persisted key usage {other}"
        ))),
    }
}

fn domain_hsm_validation_label(validation: DomainHsmValidation) -> &'static str {
    match validation {
        DomainHsmValidation::PackEnhancedFips1403Level3 => "pack_enhanced_fips1403_level3",
        DomainHsmValidation::Fips1403Level3 => "fips1403_level3",
        DomainHsmValidation::Cryptrec => "cryptrec",
        DomainHsmValidation::CommonCriteriaEal4 => "common_criteria_eal4",
        DomainHsmValidation::PciHsm => "pci_hsm",
    }
}

fn domain_hsm_validation_from_label(value: &str) -> Result<DomainHsmValidation, AdapterError> {
    match value {
        "pack_enhanced_fips1403_level3" => Ok(DomainHsmValidation::PackEnhancedFips1403Level3),
        "fips1403_level3" => Ok(DomainHsmValidation::Fips1403Level3),
        "cryptrec" => Ok(DomainHsmValidation::Cryptrec),
        "common_criteria_eal4" => Ok(DomainHsmValidation::CommonCriteriaEal4),
        "pci_hsm" => Ok(DomainHsmValidation::PciHsm),
        other => Err(AdapterError::DomainActuation(format!(
            "invalid persisted HSM validation {other}"
        ))),
    }
}

fn domain_residency_label(residency: &ResidencyClass) -> Result<&'static str, AdapterError> {
    match residency {
        ResidencyClass::StrictHomeRegion => Ok("strict_home_region"),
        ResidencyClass::HomeWithRecoveryFailover => Ok("home_with_recovery_failover"),
        ResidencyClass::Global => Ok("global"),
        ResidencyClass::PerPack(_) => Err(AdapterError::DomainActuation(
            "per-pack residency persistence is unsupported by the cloud KMS operator".to_owned(),
        )),
    }
}

fn domain_residency_from_label(value: &str) -> Result<ResidencyClass, AdapterError> {
    match value {
        "strict_home_region" => Ok(ResidencyClass::StrictHomeRegion),
        "home_with_recovery_failover" => Ok(ResidencyClass::HomeWithRecoveryFailover),
        "global" => Ok(ResidencyClass::Global),
        other => Err(AdapterError::DomainActuation(format!(
            "invalid persisted residency {other}"
        ))),
    }
}

fn domain_data_class_from_label(value: &str) -> Result<DataClass, AdapterError> {
    match value {
        "PUBLIC" => Ok(DataClass::Public),
        "INTERNAL_ONLY" => Ok(DataClass::InternalOnly),
        "PII_IDENTIFYING" => Ok(DataClass::PiiIdentifying),
        "PII_SENSITIVE" => Ok(DataClass::PiiSensitive),
        "PHI" => Ok(DataClass::Phi),
        "PCI" => Ok(DataClass::Pci),
        "PIPA_ARTICLE_23" => Ok(DataClass::PipaArticle23),
        "CHILDREN" => Ok(DataClass::Children),
        "FINANCIAL" => Ok(DataClass::Financial),
        "USAGE" => Ok(DataClass::Usage),
        "SECRET" => Ok(DataClass::Secret),
        "AUDIT" => Ok(DataClass::Audit),
        "PII_QUASI_IDENTIFIER" => Ok(DataClass::PiiQuasiIdentifier),
        "FINANCIAL_REGULATED_CREDIT" => Ok(DataClass::FinancialRegulatedCredit),
        "BEHAVIORAL_TENANT_PRODUCT" => Ok(DataClass::BehavioralTenantProduct),
        "BEHAVIORAL_ADS" => Ok(DataClass::BehavioralAds),
        "DECLARED_PREFERENCE" => Ok(DataClass::DeclaredPreference),
        "SEARCH_QUERY" => Ok(DataClass::SearchQuery),
        "SENSITIVE_PIPA_ART23" => Ok(DataClass::SensitivePipaArticle23),
        other => Err(AdapterError::DomainActuation(format!(
            "invalid persisted data class {other}"
        ))),
    }
}

fn key_state_label(state: KmsKeyState) -> &'static str {
    match state {
        KmsKeyState::PendingImport => "pending_import",
        KmsKeyState::Enabled => "enabled",
        KmsKeyState::Disabled => "disabled",
        KmsKeyState::PendingDeletion => "pending_deletion",
        KmsKeyState::Destroyed => "destroyed",
    }
}

fn key_state_from_label(value: &str) -> Result<KmsKeyState, AdapterError> {
    match value {
        "pending_import" => Ok(KmsKeyState::PendingImport),
        "enabled" => Ok(KmsKeyState::Enabled),
        "disabled" => Ok(KmsKeyState::Disabled),
        "pending_deletion" => Ok(KmsKeyState::PendingDeletion),
        "destroyed" => Ok(KmsKeyState::Destroyed),
        other => Err(AdapterError::DomainActuation(format!(
            "invalid persisted key state {other}"
        ))),
    }
}

fn key_version_lifecycle_state_label(state: KmsKeyVersionLifecycleState) -> &'static str {
    match state {
        KmsKeyVersionLifecycleState::Active => "active",
        KmsKeyVersionLifecycleState::DecryptOnly => "decrypt_only",
        KmsKeyVersionLifecycleState::Quarantined => "quarantined",
        KmsKeyVersionLifecycleState::Destroyed => "destroyed",
    }
}

fn key_version_lifecycle_state_from_label(
    value: &str,
) -> Result<KmsKeyVersionLifecycleState, AdapterError> {
    match value {
        "active" => Ok(KmsKeyVersionLifecycleState::Active),
        "decrypt_only" => Ok(KmsKeyVersionLifecycleState::DecryptOnly),
        "quarantined" => Ok(KmsKeyVersionLifecycleState::Quarantined),
        "destroyed" => Ok(KmsKeyVersionLifecycleState::Destroyed),
        other => Err(AdapterError::DomainActuation(format!(
            "invalid persisted key version lifecycle state {other}"
        ))),
    }
}

fn persistence_error(error: impl std::fmt::Display) -> AdapterError {
    AdapterError::DomainActuation(format!("persistent KMS operator state error: {error}"))
}

fn map_domain_error(error: CloudKmsError) -> AdapterError {
    AdapterError::DomainActuation(format!("{error:?}"))
}
