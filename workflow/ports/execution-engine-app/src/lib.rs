//! Workflow-engine execution-engine app foundation.
//!
//! This crate provides a source-level app composition root for the preview
//! execution-engine REST, SDK, worker, and in-memory adapter seams. It validates
//! metadata refs for later Postgres, Valkey, Cedar, OpenBao, event-bus, audit,
//! and Oyatie Cloud dogfood workload binding; exposes Kubernetes-shaped
//! startup/liveness/readiness probe decisions and OpenTelemetry-shaped service
//! resource attributes; and supports in-process preview execution for tests. It
//! performs no binary startup, environment loading, HTTP serving, socket/DNS I/O,
//! database connection, Valkey lease coordination, Cedar evaluation, OpenBao
//! secret materialization, event-bus publishing, audit-chain sealing, filesystem
//! access, random/UUID generation, wall-clock reads, Kubernetes API calls,
//! container orchestration, cloud deployment, or tenant workload scheduling.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use workflow_execution_engine_adapter::{
    ExecutionAdapterAction, ExecutionAdapterActionKind, WorkflowExecutionMemoryAdapter,
};
pub use workflow_execution_engine_rest::{
    ExecutionEngineRestBody, ExecutionEngineRestResponse, ExecutionEngineRestService,
};
pub use workflow_execution_engine_sdk::{
    EXECUTION_ENGINE_SDK_DECLARED_VERSION, ExecutionEngineSdkClient, ExecutionEngineSdkConfig,
    ExecutionEngineSdkRequestContext, ExecutionEngineSdkRunDescriptor,
    ExecutionEngineSdkStepDescriptor, ExecutionEngineSdkTimerDescriptor,
};
pub use workflow_execution_engine_worker::{
    ExecutionEngineWorker, ExecutionWorkerResumeCandidate, ExecutionWorkerResumePlan,
    ExecutionWorkerResumeThrottle, WorkflowExecutionStatus,
};

pub const EXECUTION_ENGINE_APP_SURFACE: &str = "workflow-engine.execution-engine.app";
pub const EXECUTION_ENGINE_APP_SERVICE_NAMESPACE: &str = "oyatie.workflow-engine";
pub const EXECUTION_ENGINE_APP_SERVICE_NAME: &str = "workflow-engine-execution-engine";
pub const EXECUTION_ENGINE_APP_STARTUP_PROBE_PATH: &str = "/healthz/startup";
pub const EXECUTION_ENGINE_APP_LIVENESS_PROBE_PATH: &str = "/healthz/live";
pub const EXECUTION_ENGINE_APP_READINESS_PROBE_PATH: &str = "/healthz/ready";
pub const EXECUTION_ENGINE_APP_DOGFOOD_SUBSTRATE_REF_PREFIX: &str = "oyatie-cloud:";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExecutionEngineAppComponent {
    RestApi,
    SdkFacade,
    Worker,
    RunStore,
    Dispatcher,
    RetryPolicy,
    TimerStore,
    CedarPolicyBundle,
    PostgresRunStoreRef,
    ValkeyLeaseRef,
    EventBusPublisherRef,
    OpenBaoCredentialRef,
    AuditChainRef,
    OyatieCloudTenantWorkload,
}

impl ExecutionEngineAppComponent {
    pub const fn as_ref(self) -> &'static str {
        match self {
            Self::RestApi => "component:execution-engine:rest-api",
            Self::SdkFacade => "component:execution-engine:sdk-facade",
            Self::Worker => "component:execution-engine:worker",
            Self::RunStore => "component:execution-engine:run-store",
            Self::Dispatcher => "component:execution-engine:dispatcher",
            Self::RetryPolicy => "component:execution-engine:retry-policy",
            Self::TimerStore => "component:execution-engine:timer-store",
            Self::CedarPolicyBundle => "component:execution-engine:cedar-policy-bundle",
            Self::PostgresRunStoreRef => "component:execution-engine:postgres-run-store-ref",
            Self::ValkeyLeaseRef => "component:execution-engine:valkey-lease-ref",
            Self::EventBusPublisherRef => "component:execution-engine:event-bus-publisher-ref",
            Self::OpenBaoCredentialRef => "component:execution-engine:openbao-credential-ref",
            Self::AuditChainRef => "component:execution-engine:audit-chain-ref",
            Self::OyatieCloudTenantWorkload => {
                "component:execution-engine:oyatie-cloud-tenant-workload"
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExecutionEngineAppProbeKind {
    Startup,
    Liveness,
    Readiness,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExecutionEngineAppProbeStatus {
    Serving,
    NotReady,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEngineAppProbeReport {
    pub kind: ExecutionEngineAppProbeKind,     // data_class: PUBLIC
    pub status: ExecutionEngineAppProbeStatus, // data_class: PUBLIC
    pub path: String,                          // data_class: PUBLIC
    pub checked_components: Vec<String>,       // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEngineAppTelemetryAttribute {
    pub key: String,   // data_class: PUBLIC
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEngineAppWorkloadPlan {
    pub service_namespace: String,     // data_class: PUBLIC
    pub service_name: String,          // data_class: PUBLIC
    pub service_instance_ref: String,  // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub cell_id: String,               // data_class: INTERNAL_ONLY
    pub cloud_substrate_ref: String,   // data_class: INTERNAL_ONLY
    pub dogfood_tenant_workload: bool, // data_class: PUBLIC
    pub startup_probe_path: String,    // data_class: PUBLIC
    pub liveness_probe_path: String,   // data_class: PUBLIC
    pub readiness_probe_path: String,  // data_class: PUBLIC
    pub telemetry_attributes: Vec<ExecutionEngineAppTelemetryAttribute>, // data_class: INTERNAL_ONLY
    pub component_refs: Vec<String>, // data_class: INTERNAL_ONLY
    pub non_claim_refs: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionEngineAppConfig {
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub cell_id: String,                      // data_class: INTERNAL_ONLY
    pub service_instance_ref: String,         // data_class: INTERNAL_ONLY
    pub deployment_environment_ref: String,   // data_class: INTERNAL_ONLY
    pub cloud_substrate_ref: String,          // data_class: INTERNAL_ONLY
    pub cedar_bundle_ref: String,             // data_class: INTERNAL_ONLY
    pub postgres_run_store_ref: String,       // data_class: INTERNAL_ONLY
    pub valkey_lease_ref: String,             // data_class: INTERNAL_ONLY
    pub event_bus_publisher_ref: String,      // data_class: INTERNAL_ONLY
    pub openbao_credential_ref: String,       // data_class: INTERNAL_ONLY
    pub audit_chain_ref: String,              // data_class: INTERNAL_ONLY
    pub sdk_config: ExecutionEngineSdkConfig, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutionEngineAppError {
    InvalidConfig { evidence_ref: String },
    SdkConfigRejected { evidence_ref: String },
    UnsafeMetadata { evidence_ref: String },
    RestRejected { evidence_ref: String },
}

impl ExecutionEngineAppError {
    pub fn primary_evidence_ref(&self) -> &str {
        match self {
            Self::InvalidConfig { evidence_ref }
            | Self::SdkConfigRejected { evidence_ref }
            | Self::UnsafeMetadata { evidence_ref }
            | Self::RestRejected { evidence_ref } => evidence_ref,
        }
    }
}

pub struct ExecutionEngineApp {
    config: ExecutionEngineAppConfig,
    sdk: ExecutionEngineSdkClient,
    rest: ExecutionEngineRestService,
    worker: ExecutionEngineWorker,
    store: WorkflowExecutionMemoryAdapter,
    dispatcher: WorkflowExecutionMemoryAdapter,
    retry_policy: WorkflowExecutionMemoryAdapter,
    timers: WorkflowExecutionMemoryAdapter,
    accepting_traffic: bool,
}

impl ExecutionEngineApp {
    pub fn new(config: ExecutionEngineAppConfig) -> Result<Self, ExecutionEngineAppError> {
        validate_config(&config)?;
        let sdk = ExecutionEngineSdkClient::new(config.sdk_config.clone()).map_err(|error| {
            ExecutionEngineAppError::SdkConfigRejected {
                evidence_ref: error.primary_evidence_ref().to_owned(),
            }
        })?;
        Ok(Self {
            config,
            sdk,
            rest: ExecutionEngineRestService::default(),
            worker: ExecutionEngineWorker::default(),
            store: WorkflowExecutionMemoryAdapter::default(),
            dispatcher: WorkflowExecutionMemoryAdapter::default(),
            retry_policy: WorkflowExecutionMemoryAdapter::default(),
            timers: WorkflowExecutionMemoryAdapter::default(),
            accepting_traffic: true,
        })
    }

    pub fn workload_plan(&self) -> ExecutionEngineAppWorkloadPlan {
        ExecutionEngineAppWorkloadPlan {
            service_namespace: EXECUTION_ENGINE_APP_SERVICE_NAMESPACE.to_owned(),
            service_name: EXECUTION_ENGINE_APP_SERVICE_NAME.to_owned(),
            service_instance_ref: self.config.service_instance_ref.clone(),
            tenant_id: self.config.tenant_id.clone(),
            cell_id: self.config.cell_id.clone(),
            cloud_substrate_ref: self.config.cloud_substrate_ref.clone(),
            dogfood_tenant_workload: self
                .config
                .cloud_substrate_ref
                .starts_with(EXECUTION_ENGINE_APP_DOGFOOD_SUBSTRATE_REF_PREFIX),
            startup_probe_path: EXECUTION_ENGINE_APP_STARTUP_PROBE_PATH.to_owned(),
            liveness_probe_path: EXECUTION_ENGINE_APP_LIVENESS_PROBE_PATH.to_owned(),
            readiness_probe_path: EXECUTION_ENGINE_APP_READINESS_PROBE_PATH.to_owned(),
            telemetry_attributes: vec![
                attr("service.namespace", EXECUTION_ENGINE_APP_SERVICE_NAMESPACE),
                attr("service.name", EXECUTION_ENGINE_APP_SERVICE_NAME),
                attr("service.instance.id", &self.config.service_instance_ref),
                attr(
                    "deployment.environment.ref",
                    &self.config.deployment_environment_ref,
                ),
                attr("oyatie.cell.ref", &self.config.cell_id),
            ],
            component_refs: sorted_unique(all_component_refs()),
            non_claim_refs: sorted_unique(vec![
                "workflow-execution-app:source-only-composition".to_owned(),
                "workflow-execution-app:no-cloud-runtime-deployment".to_owned(),
                "workflow-execution-app:no-durable-postgres-valkey-io".to_owned(),
                "workflow-execution-app:no-hyperscaler-claim".to_owned(),
            ]),
        }
    }

    pub fn probe(&self, kind: ExecutionEngineAppProbeKind) -> ExecutionEngineAppProbeReport {
        match kind {
            ExecutionEngineAppProbeKind::Startup => probe_report(
                kind,
                ExecutionEngineAppProbeStatus::Serving,
                EXECUTION_ENGINE_APP_STARTUP_PROBE_PATH,
                vec![
                    ExecutionEngineAppComponent::RestApi,
                    ExecutionEngineAppComponent::SdkFacade,
                    ExecutionEngineAppComponent::Worker,
                ],
                vec!["workflow-execution-app:startup-configured".to_owned()],
            ),
            ExecutionEngineAppProbeKind::Liveness => probe_report(
                kind,
                ExecutionEngineAppProbeStatus::Serving,
                EXECUTION_ENGINE_APP_LIVENESS_PROBE_PATH,
                vec![
                    ExecutionEngineAppComponent::RestApi,
                    ExecutionEngineAppComponent::Worker,
                ],
                vec!["workflow-execution-app:liveness-shallow".to_owned()],
            ),
            ExecutionEngineAppProbeKind::Readiness => {
                let status = if self.accepting_traffic {
                    ExecutionEngineAppProbeStatus::Serving
                } else {
                    ExecutionEngineAppProbeStatus::NotReady
                };
                probe_report(
                    kind,
                    status,
                    EXECUTION_ENGINE_APP_READINESS_PROBE_PATH,
                    vec![
                        ExecutionEngineAppComponent::PostgresRunStoreRef,
                        ExecutionEngineAppComponent::ValkeyLeaseRef,
                        ExecutionEngineAppComponent::CedarPolicyBundle,
                        ExecutionEngineAppComponent::EventBusPublisherRef,
                        ExecutionEngineAppComponent::OpenBaoCredentialRef,
                        ExecutionEngineAppComponent::AuditChainRef,
                        ExecutionEngineAppComponent::OyatieCloudTenantWorkload,
                    ],
                    vec![if self.accepting_traffic {
                        "workflow-execution-app:readiness-configured".to_owned()
                    } else {
                        "workflow-execution-app:readiness-draining".to_owned()
                    }],
                )
            }
        }
    }

    pub fn mark_draining(&mut self, reason_ref: &str) -> Result<(), ExecutionEngineAppError> {
        if !is_safe_ref(reason_ref) {
            return Err(ExecutionEngineAppError::UnsafeMetadata {
                evidence_ref: "workflow-execution-app:unsafe-drain-reason".to_owned(),
            });
        }
        self.accepting_traffic = false;
        Ok(())
    }

    pub fn mark_accepting_traffic(&mut self) {
        self.accepting_traffic = true;
    }

    pub fn start_run_in_process(
        &mut self,
        context: ExecutionEngineSdkRequestContext,
        run: ExecutionEngineSdkRunDescriptor,
        first_step: ExecutionEngineSdkStepDescriptor,
    ) -> Result<ExecutionEngineRestResponse, ExecutionEngineAppError> {
        let plan = self
            .sdk
            .start_run(context, run, first_step)
            .map_err(|error| ExecutionEngineAppError::UnsafeMetadata {
                evidence_ref: error.primary_evidence_ref().to_owned(),
            })?;
        self.sdk
            .execute_in_process(
                &mut self.rest,
                &mut self.store,
                &mut self.dispatcher,
                &self.retry_policy,
                &mut self.timers,
                plan,
            )
            .map_err(|error| ExecutionEngineAppError::RestRejected {
                evidence_ref: error.primary_evidence_ref().to_owned(),
            })
    }

    pub fn dispatch_step_in_process(
        &mut self,
        context: ExecutionEngineSdkRequestContext,
        run: ExecutionEngineSdkRunDescriptor,
        step: ExecutionEngineSdkStepDescriptor,
    ) -> Result<ExecutionEngineRestResponse, ExecutionEngineAppError> {
        let plan = self
            .sdk
            .dispatch_step(context, run, step)
            .map_err(|error| ExecutionEngineAppError::UnsafeMetadata {
                evidence_ref: error.primary_evidence_ref().to_owned(),
            })?;
        self.sdk
            .execute_in_process(
                &mut self.rest,
                &mut self.store,
                &mut self.dispatcher,
                &self.retry_policy,
                &mut self.timers,
                plan,
            )
            .map_err(|error| ExecutionEngineAppError::RestRejected {
                evidence_ref: error.primary_evidence_ref().to_owned(),
            })
    }

    pub fn arm_timer_in_process(
        &mut self,
        context: ExecutionEngineSdkRequestContext,
        run: ExecutionEngineSdkRunDescriptor,
        timer: ExecutionEngineSdkTimerDescriptor,
    ) -> Result<ExecutionEngineRestResponse, ExecutionEngineAppError> {
        let plan = self
            .sdk
            .arm_sla_timer(context, run, timer)
            .map_err(|error| ExecutionEngineAppError::UnsafeMetadata {
                evidence_ref: error.primary_evidence_ref().to_owned(),
            })?;
        self.sdk
            .execute_in_process(
                &mut self.rest,
                &mut self.store,
                &mut self.dispatcher,
                &self.retry_policy,
                &mut self.timers,
                plan,
            )
            .map_err(|error| ExecutionEngineAppError::RestRejected {
                evidence_ref: error.primary_evidence_ref().to_owned(),
            })
    }

    pub fn plan_worker_resume(
        &self,
        throttle: ExecutionWorkerResumeThrottle,
        candidates: Vec<ExecutionWorkerResumeCandidate>,
    ) -> Result<ExecutionWorkerResumePlan, ExecutionEngineAppError> {
        self.worker
            .plan_cold_start_resume(throttle, candidates)
            .map_err(|evidence_ref| ExecutionEngineAppError::UnsafeMetadata { evidence_ref })
    }

    pub fn run_count(&self) -> usize {
        self.store.run_count()
    }

    pub fn dispatched_action_count(&self) -> usize {
        self.dispatcher.recorded_actions().len()
    }

    pub fn timer_count(&self) -> usize {
        self.timers.timer_count()
    }

    pub fn rest_cached_response_count(&self) -> usize {
        self.rest.api_cached_response_count()
    }

    pub fn store_actions(&self) -> &[ExecutionAdapterAction] {
        self.store.recorded_actions()
    }
}

fn validate_config(config: &ExecutionEngineAppConfig) -> Result<(), ExecutionEngineAppError> {
    if config.tenant_id != config.sdk_config.tenant_id {
        return Err(ExecutionEngineAppError::InvalidConfig {
            evidence_ref: "workflow-execution-app:tenant-sdk-drift".to_owned(),
        });
    }
    if config.cell_id != config.sdk_config.default_cell_id {
        return Err(ExecutionEngineAppError::InvalidConfig {
            evidence_ref: "workflow-execution-app:cell-sdk-drift".to_owned(),
        });
    }
    if !is_safe_tenant(&config.tenant_id)
        || !is_safe_ref(&config.cell_id)
        || !is_safe_ref(&config.service_instance_ref)
        || !is_safe_ref(&config.deployment_environment_ref)
        || !is_safe_ref(&config.cloud_substrate_ref)
        || !is_safe_ref(&config.cedar_bundle_ref)
        || !is_safe_ref(&config.postgres_run_store_ref)
        || !is_safe_ref(&config.valkey_lease_ref)
        || !is_safe_ref(&config.event_bus_publisher_ref)
        || !is_safe_ref(&config.openbao_credential_ref)
        || !is_safe_ref(&config.audit_chain_ref)
    {
        return Err(ExecutionEngineAppError::InvalidConfig {
            evidence_ref: "workflow-execution-app:invalid-config-metadata".to_owned(),
        });
    }
    if !config
        .cloud_substrate_ref
        .starts_with(EXECUTION_ENGINE_APP_DOGFOOD_SUBSTRATE_REF_PREFIX)
    {
        return Err(ExecutionEngineAppError::InvalidConfig {
            evidence_ref: "workflow-execution-app:cloud-substrate-ref-required".to_owned(),
        });
    }
    Ok(())
}

fn probe_report(
    kind: ExecutionEngineAppProbeKind,
    status: ExecutionEngineAppProbeStatus,
    path: &str,
    checked_components: Vec<ExecutionEngineAppComponent>,
    evidence_refs: Vec<String>,
) -> ExecutionEngineAppProbeReport {
    ExecutionEngineAppProbeReport {
        kind,
        status,
        path: path.to_owned(),
        checked_components: checked_components
            .into_iter()
            .map(|component| component.as_ref().to_owned())
            .collect(),
        evidence_refs: sorted_unique(evidence_refs),
    }
}

fn attr(key: &str, value: &str) -> ExecutionEngineAppTelemetryAttribute {
    ExecutionEngineAppTelemetryAttribute {
        key: key.to_owned(),
        value: value.to_owned(),
    }
}

fn all_component_refs() -> Vec<String> {
    vec![
        ExecutionEngineAppComponent::RestApi.as_ref().to_owned(),
        ExecutionEngineAppComponent::SdkFacade.as_ref().to_owned(),
        ExecutionEngineAppComponent::Worker.as_ref().to_owned(),
        ExecutionEngineAppComponent::RunStore.as_ref().to_owned(),
        ExecutionEngineAppComponent::Dispatcher.as_ref().to_owned(),
        ExecutionEngineAppComponent::RetryPolicy.as_ref().to_owned(),
        ExecutionEngineAppComponent::TimerStore.as_ref().to_owned(),
        ExecutionEngineAppComponent::CedarPolicyBundle
            .as_ref()
            .to_owned(),
        ExecutionEngineAppComponent::PostgresRunStoreRef
            .as_ref()
            .to_owned(),
        ExecutionEngineAppComponent::ValkeyLeaseRef
            .as_ref()
            .to_owned(),
        ExecutionEngineAppComponent::EventBusPublisherRef
            .as_ref()
            .to_owned(),
        ExecutionEngineAppComponent::OpenBaoCredentialRef
            .as_ref()
            .to_owned(),
        ExecutionEngineAppComponent::AuditChainRef
            .as_ref()
            .to_owned(),
        ExecutionEngineAppComponent::OyatieCloudTenantWorkload
            .as_ref()
            .to_owned(),
    ]
}

fn is_safe_tenant(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.starts_with("ten_") && value == trimmed && is_safe_metadata(value)
}

fn is_safe_ref(value: &str) -> bool {
    is_safe_metadata(value) && value.contains(':')
}

fn is_safe_metadata(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && value == trimmed
        && !value.chars().any(char::is_whitespace)
        && !contains_raw_secret_material(value)
        && !contains_raw_content_material(value)
}

fn contains_raw_secret_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with("sk-")
        || lower.contains("sk-")
        || lower.contains("bearer")
        || lower.contains("authorization:")
        || lower.contains("api_key=")
        || lower.contains("openai_api_key")
        || lower.contains("private key")
        || lower.contains("-----begin")
        || lower.contains("secret=")
}

fn contains_raw_content_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("raw prompt")
        || lower.contains("raw model")
        || lower.contains("write an email")
        || lower.contains("customer message")
        || lower.contains("model answer")
        || lower.contains("raw output")
        || lower.contains("payload")
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sdk_config() -> ExecutionEngineSdkConfig {
        ExecutionEngineSdkConfig {
            tenant_id: "ten_execution_app".to_owned(),
            principal_id: "principal:workflow-app:runtime".to_owned(),
            authorization_decision_id: "authz:workflow-app:allow".to_owned(),
            authorization_evidence_ref: "policy:workflow-app:allow".to_owned(),
            default_spec_id: "spec:workflow:invoice".to_owned(),
            default_version_sha: "sha:workflow:invoice:v1".to_owned(),
            default_cell_id: "cell:us-east-1a".to_owned(),
            spec_integrity_ref: "integrity:workflow:invoice".to_owned(),
            replay_epoch_ref: "replay:epoch:20260525".to_owned(),
            scheduler_epoch_ref: "scheduler:epoch:20260525".to_owned(),
            trace_context_ref: "trace:workflow-app:root".to_owned(),
            oyatie_version: EXECUTION_ENGINE_SDK_DECLARED_VERSION.to_owned(),
        }
    }

    fn app_config() -> ExecutionEngineAppConfig {
        ExecutionEngineAppConfig {
            tenant_id: "ten_execution_app".to_owned(),
            cell_id: "cell:us-east-1a".to_owned(),
            service_instance_ref: "instance:workflow-execution-app:pod-1".to_owned(),
            deployment_environment_ref: "env:dev:preview".to_owned(),
            cloud_substrate_ref: "oyatie-cloud:dogfood:dev-cell".to_owned(),
            cedar_bundle_ref: "cedar:bundle:workflow-execution".to_owned(),
            postgres_run_store_ref: "postgres:run-store:workflow-execution".to_owned(),
            valkey_lease_ref: "valkey:lease:workflow-execution".to_owned(),
            event_bus_publisher_ref: "event-bus:workflow-events:publisher".to_owned(),
            openbao_credential_ref: "openbao:secret:workflow-execution".to_owned(),
            audit_chain_ref: "audit-chain:workflow-execution".to_owned(),
            sdk_config: sdk_config(),
        }
    }

    fn new_app_error(config: ExecutionEngineAppConfig) -> ExecutionEngineAppError {
        match ExecutionEngineApp::new(config) {
            Ok(_) => panic!("expected execution-engine app config rejection"),
            Err(error) => error,
        }
    }

    fn context(seq: u32) -> ExecutionEngineSdkRequestContext {
        ExecutionEngineSdkRequestContext {
            request_id: format!("req:workflow-app:{seq}"),
            idempotency_key: format!("idem:workflow-app:{seq}"),
            trace_context_ref: None,
        }
    }

    fn run(status: &str, version: u64) -> ExecutionEngineSdkRunDescriptor {
        ExecutionEngineSdkRunDescriptor {
            run_id: "run:workflow-app:invoice:1".to_owned(),
            spec_id: None,
            version_sha: None,
            active_cell_id: None,
            current_run_status: status.to_owned(),
            current_run_version: version,
            current_step_index: Some(1),
            input_ref: Some("input:workflow-app:invoice:1".to_owned()),
            evidence_refs: vec!["evidence:workflow-app:invoice:1".to_owned()],
        }
    }

    fn step(status: &str) -> ExecutionEngineSdkStepDescriptor {
        ExecutionEngineSdkStepDescriptor {
            step_id: "step:workflow-app:approve".to_owned(),
            step_index: 1,
            step_attempt: 1,
            step_status: status.to_owned(),
            side_effect_ref: Some("effect:workflow-app:approval".to_owned()),
            last_error_ref: None,
        }
    }

    #[test]
    fn workload_plan_models_oyatie_cloud_dogfood_and_otel_service_identity() {
        let app = ExecutionEngineApp::new(app_config()).expect("valid app");
        let plan = app.workload_plan();
        assert_eq!(
            plan.service_namespace,
            EXECUTION_ENGINE_APP_SERVICE_NAMESPACE
        );
        assert_eq!(plan.service_name, EXECUTION_ENGINE_APP_SERVICE_NAME);
        assert!(plan.dogfood_tenant_workload);
        assert_eq!(
            plan.startup_probe_path,
            EXECUTION_ENGINE_APP_STARTUP_PROBE_PATH
        );
        assert!(
            plan.telemetry_attributes
                .iter()
                .any(|attr| attr.key == "service.name"
                    && attr.value == EXECUTION_ENGINE_APP_SERVICE_NAME)
        );
        assert!(
            plan.telemetry_attributes
                .iter()
                .any(|attr| attr.key == "service.instance.id")
        );
        assert!(
            plan.component_refs.contains(
                &ExecutionEngineAppComponent::OyatieCloudTenantWorkload
                    .as_ref()
                    .to_owned()
            )
        );
        assert!(
            plan.non_claim_refs
                .contains(&"workflow-execution-app:no-hyperscaler-claim".to_owned())
        );
    }

    #[test]
    fn config_validation_rejects_missing_cloud_ref_tenant_drift_and_raw_secret_without_echo() {
        let missing_cloud = ExecutionEngineAppConfig {
            cloud_substrate_ref: "substrate:external-k8s".to_owned(),
            ..app_config()
        };
        let error = new_app_error(missing_cloud);
        assert_eq!(
            error.primary_evidence_ref(),
            "workflow-execution-app:cloud-substrate-ref-required"
        );

        let tenant_drift = ExecutionEngineAppConfig {
            sdk_config: ExecutionEngineSdkConfig {
                tenant_id: "ten_other".to_owned(),
                ..sdk_config()
            },
            ..app_config()
        };
        let error = new_app_error(tenant_drift);
        assert_eq!(
            error.primary_evidence_ref(),
            "workflow-execution-app:tenant-sdk-drift"
        );

        let raw_secret = ExecutionEngineAppConfig {
            openbao_credential_ref: "secret=super-secret-token".to_owned(),
            ..app_config()
        };
        let error = new_app_error(raw_secret);
        assert_eq!(
            error.primary_evidence_ref(),
            "workflow-execution-app:invalid-config-metadata"
        );
        assert!(!format!("{error:?}").contains("super-secret-token"));
    }

    #[test]
    fn startup_liveness_and_readiness_are_distinct_and_readiness_tracks_draining() {
        let mut app = ExecutionEngineApp::new(app_config()).expect("valid app");
        let startup = app.probe(ExecutionEngineAppProbeKind::Startup);
        assert_eq!(startup.status, ExecutionEngineAppProbeStatus::Serving);
        assert!(
            startup
                .checked_components
                .contains(&ExecutionEngineAppComponent::SdkFacade.as_ref().to_owned())
        );

        let liveness = app.probe(ExecutionEngineAppProbeKind::Liveness);
        assert_eq!(liveness.status, ExecutionEngineAppProbeStatus::Serving);
        assert!(
            !liveness.checked_components.contains(
                &ExecutionEngineAppComponent::PostgresRunStoreRef
                    .as_ref()
                    .to_owned()
            )
        );

        let readiness = app.probe(ExecutionEngineAppProbeKind::Readiness);
        assert_eq!(readiness.status, ExecutionEngineAppProbeStatus::Serving);
        assert!(
            readiness.checked_components.contains(
                &ExecutionEngineAppComponent::PostgresRunStoreRef
                    .as_ref()
                    .to_owned()
            )
        );
        app.mark_draining("drain:workflow-app:rolling-update")
            .expect("safe drain");
        let readiness = app.probe(ExecutionEngineAppProbeKind::Readiness);
        assert_eq!(readiness.status, ExecutionEngineAppProbeStatus::NotReady);
        assert!(
            readiness
                .evidence_refs
                .contains(&"workflow-execution-app:readiness-draining".to_owned())
        );
    }

    #[test]
    fn app_composes_sdk_rest_api_and_memory_adapters_for_in_process_preview() {
        let mut app = ExecutionEngineApp::new(app_config()).expect("valid app");
        let start = app
            .start_run_in_process(context(1), run("pending", 1), step("pending"))
            .expect("start response");
        assert_eq!(start.status_code, 201);
        assert_eq!(app.run_count(), 1);
        assert!(
            app.store_actions()
                .iter()
                .any(|action| action.kind == ExecutionAdapterActionKind::CreateRun)
        );

        let dispatch = app
            .dispatch_step_in_process(context(2), run("running", 2), step("pending"))
            .expect("dispatch response");
        assert_eq!(dispatch.status_code, 202);
        assert_eq!(app.dispatched_action_count(), 2);
        assert_eq!(app.rest_cached_response_count(), 2);
    }

    #[test]
    fn app_arms_timer_and_exposes_timer_store_preview_without_wall_clock_reads() {
        let mut app = ExecutionEngineApp::new(app_config()).expect("valid app");
        let start = app
            .start_run_in_process(context(3), run("pending", 1), step("pending"))
            .expect("start response");
        assert_eq!(start.status_code, 201);

        let response = app
            .arm_timer_in_process(
                context(4),
                run("running", 2),
                ExecutionEngineSdkTimerDescriptor {
                    timer_id: "timer:workflow-app:sla".to_owned(),
                    armed_at_epoch_seconds: 10,
                    deadline_epoch_seconds: 60,
                    step_index: Some(1),
                },
            )
            .expect("timer response");
        assert_eq!(response.status_code, 202);
        assert_eq!(app.timer_count(), 1);
    }

    #[test]
    fn app_delegates_worker_cold_start_resume_planning_without_queue_or_kubernetes_io() {
        let app = ExecutionEngineApp::new(app_config()).expect("valid app");
        let plan = app
            .plan_worker_resume(
                ExecutionWorkerResumeThrottle {
                    max_resumes_per_tick: 1,
                },
                vec![
                    ExecutionWorkerResumeCandidate {
                        tenant_id: "ten_execution_app".to_owned(),
                        run_id: "run:workflow-app:2".to_owned(),
                        run_status: WorkflowExecutionStatus::Running,
                        observed_run_version: 3,
                        resume_priority: 10,
                        resume_evidence_ref: "resume:workflow-app:2".to_owned(),
                    },
                    ExecutionWorkerResumeCandidate {
                        tenant_id: "ten_execution_app".to_owned(),
                        run_id: "run:workflow-app:1".to_owned(),
                        run_status: WorkflowExecutionStatus::Paused,
                        observed_run_version: 2,
                        resume_priority: 1,
                        resume_evidence_ref: "resume:workflow-app:1".to_owned(),
                    },
                ],
            )
            .expect("resume plan");
        assert_eq!(plan.accepted.len(), 1);
        assert_eq!(plan.accepted[0].run_id, "run:workflow-app:1");
        assert_eq!(plan.deferred.len(), 1);
    }

    #[test]
    fn unsafe_drain_reason_is_denied_without_state_change() {
        let mut app = ExecutionEngineApp::new(app_config()).expect("valid app");
        let error = app
            .mark_draining("raw prompt: drain for customer message")
            .expect_err("unsafe drain denied");
        assert_eq!(
            error.primary_evidence_ref(),
            "workflow-execution-app:unsafe-drain-reason"
        );
        assert_eq!(
            app.probe(ExecutionEngineAppProbeKind::Readiness).status,
            ExecutionEngineAppProbeStatus::Serving
        );
    }
}
