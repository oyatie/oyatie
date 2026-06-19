//! Workflow-engine trigger-orchestrator app foundation.
//!
//! This crate provides a source-level app composition root for the preview
//! trigger-orchestrator SDK, REST/API, and worker seams. It validates metadata
//! refs for later Postgres trigger registries, Valkey leases, Cedar bundles,
//! OpenBao credentials, scheduler/webhook/event ingress, execution-run starter
//! delegation, audit-chain emission, and Oyatie Cloud dogfood workload binding;
//! exposes Kubernetes-shaped startup/liveness/readiness probe decisions and
//! OpenTelemetry-shaped service resource attributes; and supports in-process
//! preview trigger evaluation for tests. It performs no binary startup,
//! environment loading, HTTP serving, socket/DNS I/O, database connection,
//! Valkey lease coordination, Cedar evaluation, OpenBao secret materialization,
//! scheduler execution, webhook serving, HMAC verification, event-bus
//! consumption, run creation, event-bus publishing, audit-chain sealing,
//! filesystem access, random/UUID generation, wall-clock reads, Kubernetes API
//! calls, container orchestration, cloud deployment, or tenant workload
//! scheduling.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use workflow_trigger_orchestrator_rest::{
    TriggerOrchestratorRestResponse, TriggerOrchestratorRestResponseBody,
    TriggerOrchestratorRestService,
};
pub use workflow_trigger_orchestrator_sdk::{
    TRIGGER_ORCHESTRATOR_SDK_DECLARED_VERSION, TriggerOrchestratorSdkClient,
    TriggerOrchestratorSdkConfig, TriggerOrchestratorSdkError,
    TriggerOrchestratorSdkEventDescriptor, TriggerOrchestratorSdkRequestContext,
    TriggerOrchestratorSdkScheduleDescriptor, TriggerOrchestratorSdkSource,
    TriggerOrchestratorSdkSourceMetadata, TriggerOrchestratorSdkTriggerDescriptor,
    TriggerOrchestratorSdkTriggerPlan, TriggerOrchestratorSdkWebhookDescriptor,
};
pub use workflow_trigger_orchestrator_worker::{
    TriggerOrchestratorWorker, TriggerOrchestratorWorkerJob, TriggerOrchestratorWorkerReceipt,
    TriggerOrchestratorWorkerResumeCandidate, TriggerOrchestratorWorkerResumePlan,
    TriggerOrchestratorWorkerResumeThrottle, TriggerOrchestratorWorkerStatus,
    plan_resume_candidates,
};

pub const TRIGGER_ORCHESTRATOR_APP_SURFACE: &str = "workflow-engine.trigger-orchestrator.app";
pub const TRIGGER_ORCHESTRATOR_APP_SERVICE_NAMESPACE: &str = "oyatie.workflow-engine";
pub const TRIGGER_ORCHESTRATOR_APP_SERVICE_NAME: &str = "workflow-engine-trigger-orchestrator";
pub const TRIGGER_ORCHESTRATOR_APP_STARTUP_PROBE_PATH: &str = "/healthz/startup";
pub const TRIGGER_ORCHESTRATOR_APP_LIVENESS_PROBE_PATH: &str = "/healthz/live";
pub const TRIGGER_ORCHESTRATOR_APP_READINESS_PROBE_PATH: &str = "/healthz/ready";
pub const TRIGGER_ORCHESTRATOR_APP_DOGFOOD_SUBSTRATE_REF_PREFIX: &str = "oyatie-cloud:";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TriggerOrchestratorAppComponent {
    RestApi,
    SdkFacade,
    Worker,
    TriggerRegistry,
    SchedulerCursorStore,
    WebhookVerifier,
    EventSubscriber,
    ExecutionRunStarter,
    CedarPolicyBundle,
    PostgresTriggerStoreRef,
    ValkeyLeaseRef,
    EventBusSubscriberRef,
    WebhookIngressRef,
    OpenBaoCredentialRef,
    AuditChainRef,
    OyatieCloudTenantWorkload,
}

impl TriggerOrchestratorAppComponent {
    pub const fn as_ref(self) -> &'static str {
        match self {
            Self::RestApi => "component:trigger-orchestrator:rest-api",
            Self::SdkFacade => "component:trigger-orchestrator:sdk-facade",
            Self::Worker => "component:trigger-orchestrator:worker",
            Self::TriggerRegistry => "component:trigger-orchestrator:trigger-registry",
            Self::SchedulerCursorStore => "component:trigger-orchestrator:scheduler-cursor-store",
            Self::WebhookVerifier => "component:trigger-orchestrator:webhook-verifier",
            Self::EventSubscriber => "component:trigger-orchestrator:event-subscriber",
            Self::ExecutionRunStarter => "component:trigger-orchestrator:execution-run-starter",
            Self::CedarPolicyBundle => "component:trigger-orchestrator:cedar-policy-bundle",
            Self::PostgresTriggerStoreRef => {
                "component:trigger-orchestrator:postgres-trigger-store-ref"
            }
            Self::ValkeyLeaseRef => "component:trigger-orchestrator:valkey-lease-ref",
            Self::EventBusSubscriberRef => {
                "component:trigger-orchestrator:event-bus-subscriber-ref"
            }
            Self::WebhookIngressRef => "component:trigger-orchestrator:webhook-ingress-ref",
            Self::OpenBaoCredentialRef => "component:trigger-orchestrator:openbao-credential-ref",
            Self::AuditChainRef => "component:trigger-orchestrator:audit-chain-ref",
            Self::OyatieCloudTenantWorkload => {
                "component:trigger-orchestrator:oyatie-cloud-tenant-workload"
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TriggerOrchestratorAppProbeKind {
    Startup,
    Liveness,
    Readiness,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TriggerOrchestratorAppProbeStatus {
    Serving,
    NotReady,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorAppProbeReport {
    pub kind: TriggerOrchestratorAppProbeKind, // data_class: PUBLIC
    pub status: TriggerOrchestratorAppProbeStatus, // data_class: PUBLIC
    pub path: String,                          // data_class: PUBLIC
    pub checked_components: Vec<String>,       // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorAppTelemetryAttribute {
    pub key: String,   // data_class: PUBLIC
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorAppWorkloadPlan {
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
    pub telemetry_attributes: Vec<TriggerOrchestratorAppTelemetryAttribute>, // data_class: INTERNAL_ONLY
    pub component_refs: Vec<String>, // data_class: INTERNAL_ONLY
    pub non_claim_refs: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorAppConfig {
    pub tenant_id: String,                        // data_class: INTERNAL_ONLY
    pub cell_id: String,                          // data_class: INTERNAL_ONLY
    pub service_instance_ref: String,             // data_class: INTERNAL_ONLY
    pub deployment_environment_ref: String,       // data_class: INTERNAL_ONLY
    pub cloud_substrate_ref: String,              // data_class: INTERNAL_ONLY
    pub cedar_bundle_ref: String,                 // data_class: INTERNAL_ONLY
    pub postgres_trigger_store_ref: String,       // data_class: INTERNAL_ONLY
    pub valkey_lease_ref: String,                 // data_class: INTERNAL_ONLY
    pub scheduler_cursor_ref: String,             // data_class: INTERNAL_ONLY
    pub event_bus_subscriber_ref: String,         // data_class: INTERNAL_ONLY
    pub webhook_ingress_ref: String,              // data_class: INTERNAL_ONLY
    pub execution_run_starter_ref: String,        // data_class: INTERNAL_ONLY
    pub openbao_credential_ref: String,           // data_class: INTERNAL_ONLY
    pub audit_chain_ref: String,                  // data_class: INTERNAL_ONLY
    pub sdk_config: TriggerOrchestratorSdkConfig, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TriggerOrchestratorAppError {
    InvalidConfig { evidence_ref: String },
    SdkConfigRejected { evidence_ref: String },
    UnsafeMetadata { evidence_ref: String },
    RestRejected { evidence_ref: String },
}

impl TriggerOrchestratorAppError {
    pub fn primary_evidence_ref(&self) -> &str {
        match self {
            Self::InvalidConfig { evidence_ref }
            | Self::SdkConfigRejected { evidence_ref }
            | Self::UnsafeMetadata { evidence_ref }
            | Self::RestRejected { evidence_ref } => evidence_ref,
        }
    }
}

pub struct TriggerOrchestratorApp {
    config: TriggerOrchestratorAppConfig,
    sdk: TriggerOrchestratorSdkClient,
    rest: TriggerOrchestratorRestService,
    worker: TriggerOrchestratorWorker,
    accepting_traffic: bool,
}

impl TriggerOrchestratorApp {
    pub fn new(config: TriggerOrchestratorAppConfig) -> Result<Self, TriggerOrchestratorAppError> {
        validate_config(&config)?;
        let sdk =
            TriggerOrchestratorSdkClient::new(config.sdk_config.clone()).map_err(|error| {
                TriggerOrchestratorAppError::SdkConfigRejected {
                    evidence_ref: error.primary_evidence_ref().to_owned(),
                }
            })?;
        Ok(Self {
            config,
            sdk,
            rest: TriggerOrchestratorRestService::default(),
            worker: TriggerOrchestratorWorker::default(),
            accepting_traffic: true,
        })
    }

    pub fn workload_plan(&self) -> TriggerOrchestratorAppWorkloadPlan {
        TriggerOrchestratorAppWorkloadPlan {
            service_namespace: TRIGGER_ORCHESTRATOR_APP_SERVICE_NAMESPACE.to_owned(),
            service_name: TRIGGER_ORCHESTRATOR_APP_SERVICE_NAME.to_owned(),
            service_instance_ref: self.config.service_instance_ref.clone(),
            tenant_id: self.config.tenant_id.clone(),
            cell_id: self.config.cell_id.clone(),
            cloud_substrate_ref: self.config.cloud_substrate_ref.clone(),
            dogfood_tenant_workload: self
                .config
                .cloud_substrate_ref
                .starts_with(TRIGGER_ORCHESTRATOR_APP_DOGFOOD_SUBSTRATE_REF_PREFIX),
            startup_probe_path: TRIGGER_ORCHESTRATOR_APP_STARTUP_PROBE_PATH.to_owned(),
            liveness_probe_path: TRIGGER_ORCHESTRATOR_APP_LIVENESS_PROBE_PATH.to_owned(),
            readiness_probe_path: TRIGGER_ORCHESTRATOR_APP_READINESS_PROBE_PATH.to_owned(),
            telemetry_attributes: vec![
                attr(
                    "service.namespace",
                    TRIGGER_ORCHESTRATOR_APP_SERVICE_NAMESPACE,
                ),
                attr("service.name", TRIGGER_ORCHESTRATOR_APP_SERVICE_NAME),
                attr("service.instance.id", &self.config.service_instance_ref),
                attr(
                    "deployment.environment.ref",
                    &self.config.deployment_environment_ref,
                ),
                attr("oyatie.cell.ref", &self.config.cell_id),
            ],
            component_refs: sorted_unique(all_component_refs()),
            non_claim_refs: sorted_unique(vec![
                "workflow-trigger-app:source-only-composition".to_owned(),
                "workflow-trigger-app:no-cloud-runtime-deployment".to_owned(),
                "workflow-trigger-app:no-scheduler-webhook-eventbus-io".to_owned(),
                "workflow-trigger-app:no-hyperscaler-claim".to_owned(),
            ]),
        }
    }

    pub fn probe(
        &self,
        kind: TriggerOrchestratorAppProbeKind,
    ) -> TriggerOrchestratorAppProbeReport {
        match kind {
            TriggerOrchestratorAppProbeKind::Startup => probe_report(
                kind,
                TriggerOrchestratorAppProbeStatus::Serving,
                TRIGGER_ORCHESTRATOR_APP_STARTUP_PROBE_PATH,
                vec![
                    TriggerOrchestratorAppComponent::RestApi,
                    TriggerOrchestratorAppComponent::SdkFacade,
                    TriggerOrchestratorAppComponent::Worker,
                ],
                vec!["workflow-trigger-app:startup-configured".to_owned()],
            ),
            TriggerOrchestratorAppProbeKind::Liveness => probe_report(
                kind,
                TriggerOrchestratorAppProbeStatus::Serving,
                TRIGGER_ORCHESTRATOR_APP_LIVENESS_PROBE_PATH,
                vec![
                    TriggerOrchestratorAppComponent::RestApi,
                    TriggerOrchestratorAppComponent::Worker,
                ],
                vec!["workflow-trigger-app:liveness-shallow".to_owned()],
            ),
            TriggerOrchestratorAppProbeKind::Readiness => {
                let status = if self.accepting_traffic {
                    TriggerOrchestratorAppProbeStatus::Serving
                } else {
                    TriggerOrchestratorAppProbeStatus::NotReady
                };
                probe_report(
                    kind,
                    status,
                    TRIGGER_ORCHESTRATOR_APP_READINESS_PROBE_PATH,
                    vec![
                        TriggerOrchestratorAppComponent::PostgresTriggerStoreRef,
                        TriggerOrchestratorAppComponent::ValkeyLeaseRef,
                        TriggerOrchestratorAppComponent::SchedulerCursorStore,
                        TriggerOrchestratorAppComponent::EventBusSubscriberRef,
                        TriggerOrchestratorAppComponent::WebhookIngressRef,
                        TriggerOrchestratorAppComponent::ExecutionRunStarter,
                        TriggerOrchestratorAppComponent::CedarPolicyBundle,
                        TriggerOrchestratorAppComponent::OpenBaoCredentialRef,
                        TriggerOrchestratorAppComponent::AuditChainRef,
                        TriggerOrchestratorAppComponent::OyatieCloudTenantWorkload,
                    ],
                    vec![if self.accepting_traffic {
                        "workflow-trigger-app:readiness-configured".to_owned()
                    } else {
                        "workflow-trigger-app:readiness-draining".to_owned()
                    }],
                )
            }
        }
    }

    pub fn mark_draining(&mut self, reason_ref: &str) -> Result<(), TriggerOrchestratorAppError> {
        if !is_safe_ref(reason_ref) {
            return Err(TriggerOrchestratorAppError::UnsafeMetadata {
                evidence_ref: "workflow-trigger-app:unsafe-drain-reason".to_owned(),
            });
        }
        self.accepting_traffic = false;
        Ok(())
    }

    pub fn mark_accepting_traffic(&mut self) {
        self.accepting_traffic = true;
    }

    pub fn plan_trigger_in_process(
        &self,
        context: TriggerOrchestratorSdkRequestContext,
        source: TriggerOrchestratorSdkSource,
        trigger: TriggerOrchestratorSdkTriggerDescriptor,
        metadata: TriggerOrchestratorSdkSourceMetadata,
    ) -> Result<TriggerOrchestratorSdkTriggerPlan, TriggerOrchestratorAppError> {
        self.sdk
            .plan_trigger(context, source, trigger, metadata)
            .map_err(app_error_from_sdk)
    }

    pub fn evaluate_trigger_in_process(
        &mut self,
        context: TriggerOrchestratorSdkRequestContext,
        source: TriggerOrchestratorSdkSource,
        trigger: TriggerOrchestratorSdkTriggerDescriptor,
        metadata: TriggerOrchestratorSdkSourceMetadata,
    ) -> Result<TriggerOrchestratorRestResponse, TriggerOrchestratorAppError> {
        let plan = self.plan_trigger_in_process(context, source, trigger, metadata)?;
        self.sdk
            .execute_in_process(&mut self.rest, plan)
            .map_err(|error| TriggerOrchestratorAppError::RestRejected {
                evidence_ref: error.primary_evidence_ref().to_owned(),
            })
    }

    pub fn run_worker_job_once(
        &mut self,
        job: TriggerOrchestratorWorkerJob,
    ) -> TriggerOrchestratorWorkerReceipt {
        self.worker.run_once(job)
    }

    pub fn plan_worker_resume(
        &self,
        throttle: TriggerOrchestratorWorkerResumeThrottle,
        candidates: Vec<TriggerOrchestratorWorkerResumeCandidate>,
    ) -> TriggerOrchestratorWorkerResumePlan {
        plan_resume_candidates(candidates, throttle)
    }

    pub fn rest_api_delegation_count(&self) -> usize {
        self.rest.api_delegation_count()
    }

    pub fn worker_api_apply_count(&self) -> usize {
        self.worker.api_apply_count()
    }

    pub fn worker_event_count(&self) -> usize {
        self.worker.events().len()
    }
}

fn app_error_from_sdk(error: TriggerOrchestratorSdkError) -> TriggerOrchestratorAppError {
    match error {
        TriggerOrchestratorSdkError::InvalidConfig { evidence_ref } => {
            TriggerOrchestratorAppError::SdkConfigRejected { evidence_ref }
        }
        TriggerOrchestratorSdkError::InvalidRequest { evidence_ref }
        | TriggerOrchestratorSdkError::MetadataMismatch { evidence_ref }
        | TriggerOrchestratorSdkError::UnsafeMetadata { evidence_ref } => {
            TriggerOrchestratorAppError::UnsafeMetadata { evidence_ref }
        }
        TriggerOrchestratorSdkError::RestRejected { evidence_ref } => {
            TriggerOrchestratorAppError::RestRejected { evidence_ref }
        }
    }
}

fn validate_config(
    config: &TriggerOrchestratorAppConfig,
) -> Result<(), TriggerOrchestratorAppError> {
    if config.tenant_id != config.sdk_config.tenant_id {
        return Err(TriggerOrchestratorAppError::InvalidConfig {
            evidence_ref: "workflow-trigger-app:tenant-sdk-drift".to_owned(),
        });
    }
    if config.cell_id != config.sdk_config.default_cell_id {
        return Err(TriggerOrchestratorAppError::InvalidConfig {
            evidence_ref: "workflow-trigger-app:cell-sdk-drift".to_owned(),
        });
    }
    if !is_safe_tenant(&config.tenant_id)
        || !is_safe_ref(&config.cell_id)
        || !is_safe_ref(&config.service_instance_ref)
        || !is_safe_ref(&config.deployment_environment_ref)
        || !is_safe_ref(&config.cloud_substrate_ref)
        || !is_safe_ref(&config.cedar_bundle_ref)
        || !is_safe_ref(&config.postgres_trigger_store_ref)
        || !is_safe_ref(&config.valkey_lease_ref)
        || !is_safe_ref(&config.scheduler_cursor_ref)
        || !is_safe_ref(&config.event_bus_subscriber_ref)
        || !is_safe_ref(&config.webhook_ingress_ref)
        || !is_safe_ref(&config.execution_run_starter_ref)
        || !is_safe_ref(&config.openbao_credential_ref)
        || !is_safe_ref(&config.audit_chain_ref)
    {
        return Err(TriggerOrchestratorAppError::InvalidConfig {
            evidence_ref: "workflow-trigger-app:invalid-config-metadata".to_owned(),
        });
    }
    if !config
        .cloud_substrate_ref
        .starts_with(TRIGGER_ORCHESTRATOR_APP_DOGFOOD_SUBSTRATE_REF_PREFIX)
    {
        return Err(TriggerOrchestratorAppError::InvalidConfig {
            evidence_ref: "workflow-trigger-app:cloud-substrate-ref-required".to_owned(),
        });
    }
    Ok(())
}

fn probe_report(
    kind: TriggerOrchestratorAppProbeKind,
    status: TriggerOrchestratorAppProbeStatus,
    path: &str,
    checked_components: Vec<TriggerOrchestratorAppComponent>,
    evidence_refs: Vec<String>,
) -> TriggerOrchestratorAppProbeReport {
    TriggerOrchestratorAppProbeReport {
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

fn attr(key: &str, value: &str) -> TriggerOrchestratorAppTelemetryAttribute {
    TriggerOrchestratorAppTelemetryAttribute {
        key: key.to_owned(),
        value: value.to_owned(),
    }
}

fn all_component_refs() -> Vec<String> {
    vec![
        TriggerOrchestratorAppComponent::RestApi.as_ref().to_owned(),
        TriggerOrchestratorAppComponent::SdkFacade
            .as_ref()
            .to_owned(),
        TriggerOrchestratorAppComponent::Worker.as_ref().to_owned(),
        TriggerOrchestratorAppComponent::TriggerRegistry
            .as_ref()
            .to_owned(),
        TriggerOrchestratorAppComponent::SchedulerCursorStore
            .as_ref()
            .to_owned(),
        TriggerOrchestratorAppComponent::WebhookVerifier
            .as_ref()
            .to_owned(),
        TriggerOrchestratorAppComponent::EventSubscriber
            .as_ref()
            .to_owned(),
        TriggerOrchestratorAppComponent::ExecutionRunStarter
            .as_ref()
            .to_owned(),
        TriggerOrchestratorAppComponent::CedarPolicyBundle
            .as_ref()
            .to_owned(),
        TriggerOrchestratorAppComponent::PostgresTriggerStoreRef
            .as_ref()
            .to_owned(),
        TriggerOrchestratorAppComponent::ValkeyLeaseRef
            .as_ref()
            .to_owned(),
        TriggerOrchestratorAppComponent::EventBusSubscriberRef
            .as_ref()
            .to_owned(),
        TriggerOrchestratorAppComponent::WebhookIngressRef
            .as_ref()
            .to_owned(),
        TriggerOrchestratorAppComponent::OpenBaoCredentialRef
            .as_ref()
            .to_owned(),
        TriggerOrchestratorAppComponent::AuditChainRef
            .as_ref()
            .to_owned(),
        TriggerOrchestratorAppComponent::OyatieCloudTenantWorkload
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

    fn sdk_config() -> TriggerOrchestratorSdkConfig {
        TriggerOrchestratorSdkConfig {
            tenant_id: "ten_trigger_app".to_owned(),
            principal_id: "principal:workflow-trigger-app".to_owned(),
            authorization_decision_id: "policy-decision:trigger-app-allow".to_owned(),
            authorization_evidence_ref: "policy-evidence:trigger-app-allow".to_owned(),
            policy_bundle_ref: "policy-bundle:trigger-app-v1".to_owned(),
            default_workflow_spec_id: "workflow:invoice-approval".to_owned(),
            default_version_sha: "sha:workflow:invoice:v1".to_owned(),
            default_cell_id: "cell:us-east-1a".to_owned(),
            authorization_surface_ref: "authz-surface:trigger-admission".to_owned(),
            trace_context_ref: "trace:trigger-app:root".to_owned(),
            oyatie_version: TRIGGER_ORCHESTRATOR_SDK_DECLARED_VERSION.to_owned(),
        }
    }

    fn app_config() -> TriggerOrchestratorAppConfig {
        TriggerOrchestratorAppConfig {
            tenant_id: "ten_trigger_app".to_owned(),
            cell_id: "cell:us-east-1a".to_owned(),
            service_instance_ref: "instance:workflow-trigger-app:pod-1".to_owned(),
            deployment_environment_ref: "env:dev:preview".to_owned(),
            cloud_substrate_ref: "oyatie-cloud:dogfood:dev-cell".to_owned(),
            cedar_bundle_ref: "cedar:bundle:workflow-trigger".to_owned(),
            postgres_trigger_store_ref: "postgres:trigger-store:workflow-trigger".to_owned(),
            valkey_lease_ref: "valkey:lease:workflow-trigger".to_owned(),
            scheduler_cursor_ref: "scheduler-cursor:workflow-trigger".to_owned(),
            event_bus_subscriber_ref: "event-bus:workflow-events:subscriber".to_owned(),
            webhook_ingress_ref: "webhook-ingress:workflow-trigger".to_owned(),
            execution_run_starter_ref: "execution-engine:start-run:workflow-trigger".to_owned(),
            openbao_credential_ref: "openbao:credential-ref:workflow-trigger".to_owned(),
            audit_chain_ref: "audit-chain:workflow-trigger".to_owned(),
            sdk_config: sdk_config(),
        }
    }

    fn new_app_error(config: TriggerOrchestratorAppConfig) -> TriggerOrchestratorAppError {
        match TriggerOrchestratorApp::new(config) {
            Ok(_) => panic!("expected trigger-orchestrator app config rejection"),
            Err(error) => error,
        }
    }

    fn context(idempotency_key: &str) -> TriggerOrchestratorSdkRequestContext {
        TriggerOrchestratorSdkRequestContext {
            request_id: format!("request:trigger-app:{idempotency_key}"),
            idempotency_key: idempotency_key.to_owned(),
            trace_context_ref: None,
            run_idempotency_key: format!("idem:trigger-run:{idempotency_key}"),
            correlation_ref: format!("corr:trigger-app:{idempotency_key}"),
        }
    }

    fn trigger_descriptor(trigger_id: &str) -> TriggerOrchestratorSdkTriggerDescriptor {
        TriggerOrchestratorSdkTriggerDescriptor {
            trigger_id: trigger_id.to_owned(),
            workflow_spec_id: None,
            version_sha: None,
            active_cell_id: None,
            trigger_lineage_ref: "lineage:trigger-app-parent".to_owned(),
            source_evidence_ref: "source-evidence:trigger-app".to_owned(),
            replay_epoch_ref: "replay-epoch:2026-05-25T000000Z".to_owned(),
            audit_chain_ref: "audit-chain:trigger-app".to_owned(),
            idempotency_scope_ref: "idem-scope:tenant-trigger".to_owned(),
            dry_run_reason_ref: None,
            replay_mode: false,
            dry_run: false,
            evidence_refs: vec!["evidence:trigger-app-unit-test".to_owned()],
        }
    }

    fn schedule_descriptor() -> TriggerOrchestratorSdkScheduleDescriptor {
        TriggerOrchestratorSdkScheduleDescriptor {
            cron_expr_ref: "cron:every-hour".to_owned(),
            timezone_ref: "tz:America-New_York".to_owned(),
            due_epoch_seconds: 1_750_000_000,
            observed_epoch_seconds: 1_750_000_008,
            catchup_window_seconds: 10,
            overlap_policy: "buffer-one".to_owned(),
            paused: false,
            pause_reason_ref: None,
            last_fired_epoch_seconds: Some(1_749_996_400),
            scheduler_evidence_ref: "scheduler:durable-clock-window".to_owned(),
        }
    }

    #[test]
    fn workload_plan_models_oyatie_cloud_dogfood_and_otel_service_identity() {
        let app = TriggerOrchestratorApp::new(app_config()).expect("valid app");
        let plan = app.workload_plan();
        assert_eq!(
            plan.service_namespace,
            TRIGGER_ORCHESTRATOR_APP_SERVICE_NAMESPACE
        );
        assert_eq!(plan.service_name, TRIGGER_ORCHESTRATOR_APP_SERVICE_NAME);
        assert!(plan.dogfood_tenant_workload);
        assert_eq!(
            plan.startup_probe_path,
            TRIGGER_ORCHESTRATOR_APP_STARTUP_PROBE_PATH
        );
        assert!(
            plan.telemetry_attributes
                .iter()
                .any(|attr| attr.key == "service.name"
                    && attr.value == TRIGGER_ORCHESTRATOR_APP_SERVICE_NAME)
        );
        assert!(
            plan.telemetry_attributes
                .iter()
                .any(|attr| attr.key == "service.instance.id")
        );
        assert!(
            plan.component_refs.contains(
                &TriggerOrchestratorAppComponent::OyatieCloudTenantWorkload
                    .as_ref()
                    .to_owned()
            )
        );
        assert!(
            plan.non_claim_refs
                .contains(&"workflow-trigger-app:no-hyperscaler-claim".to_owned())
        );
    }

    #[test]
    fn config_validation_rejects_missing_cloud_ref_tenant_drift_and_raw_secret_without_echo() {
        let missing_cloud = TriggerOrchestratorAppConfig {
            cloud_substrate_ref: "substrate:external-k8s".to_owned(),
            ..app_config()
        };
        let error = new_app_error(missing_cloud);
        assert_eq!(
            error.primary_evidence_ref(),
            "workflow-trigger-app:cloud-substrate-ref-required"
        );

        let tenant_drift = TriggerOrchestratorAppConfig {
            sdk_config: TriggerOrchestratorSdkConfig {
                tenant_id: "ten_other".to_owned(),
                ..sdk_config()
            },
            ..app_config()
        };
        let error = new_app_error(tenant_drift);
        assert_eq!(
            error.primary_evidence_ref(),
            "workflow-trigger-app:tenant-sdk-drift"
        );

        let raw_secret = TriggerOrchestratorAppConfig {
            openbao_credential_ref: "secret=super-secret-token".to_owned(),
            ..app_config()
        };
        let error = new_app_error(raw_secret);
        assert_eq!(
            error.primary_evidence_ref(),
            "workflow-trigger-app:invalid-config-metadata"
        );
        assert!(!format!("{error:?}").contains("super-secret-token"));
    }

    #[test]
    fn startup_liveness_and_readiness_are_distinct_and_readiness_tracks_draining() {
        let mut app = TriggerOrchestratorApp::new(app_config()).expect("valid app");
        let startup = app.probe(TriggerOrchestratorAppProbeKind::Startup);
        assert_eq!(startup.status, TriggerOrchestratorAppProbeStatus::Serving);
        assert!(
            startup.checked_components.contains(
                &TriggerOrchestratorAppComponent::SdkFacade
                    .as_ref()
                    .to_owned()
            )
        );

        let liveness = app.probe(TriggerOrchestratorAppProbeKind::Liveness);
        assert_eq!(liveness.status, TriggerOrchestratorAppProbeStatus::Serving);
        assert!(
            !liveness.checked_components.contains(
                &TriggerOrchestratorAppComponent::PostgresTriggerStoreRef
                    .as_ref()
                    .to_owned()
            )
        );

        let readiness = app.probe(TriggerOrchestratorAppProbeKind::Readiness);
        assert_eq!(readiness.status, TriggerOrchestratorAppProbeStatus::Serving);
        assert!(
            readiness.checked_components.contains(
                &TriggerOrchestratorAppComponent::PostgresTriggerStoreRef
                    .as_ref()
                    .to_owned()
            )
        );
        app.mark_draining("drain:trigger-app:rolling-update")
            .expect("safe drain");
        let readiness = app.probe(TriggerOrchestratorAppProbeKind::Readiness);
        assert_eq!(
            readiness.status,
            TriggerOrchestratorAppProbeStatus::NotReady
        );
        assert!(
            readiness
                .evidence_refs
                .contains(&"workflow-trigger-app:readiness-draining".to_owned())
        );
    }

    #[test]
    fn app_composes_sdk_rest_api_for_in_process_scheduler_preview() {
        let mut app = TriggerOrchestratorApp::new(app_config()).expect("valid app");
        let response = app
            .evaluate_trigger_in_process(
                context("idem:trigger-app:scheduler"),
                TriggerOrchestratorSdkSource::Scheduler,
                trigger_descriptor("trigger:daily-invoice"),
                TriggerOrchestratorSdkSourceMetadata::Scheduler(schedule_descriptor()),
            )
            .expect("trigger response");
        assert_eq!(response.status_code, 202);
        assert_eq!(app.rest_api_delegation_count(), 1);
        let TriggerOrchestratorRestResponseBody::Success(success) = response.body else {
            panic!("expected trigger success");
        };
        assert_eq!(success.trigger.usecase_status, "accepted");
        assert!(success.trigger.dispatch_required);
        assert!(!format!("{success:?}").contains("payload"));
    }

    #[test]
    fn app_preserves_rest_idempotent_replay_without_runtime_id_generation() {
        let mut app = TriggerOrchestratorApp::new(app_config()).expect("valid app");
        let plan = app
            .plan_trigger_in_process(
                context("idem:trigger-app:replay"),
                TriggerOrchestratorSdkSource::Scheduler,
                trigger_descriptor("trigger:daily-invoice"),
                TriggerOrchestratorSdkSourceMetadata::Scheduler(schedule_descriptor()),
            )
            .expect("plan");
        let first = app
            .sdk
            .execute_in_process(&mut app.rest, plan.clone())
            .expect("first");
        let second = app
            .sdk
            .execute_in_process(&mut app.rest, plan)
            .expect("second");
        assert_eq!(first, second);
        assert_eq!(app.rest_api_delegation_count(), 2);
    }

    #[test]
    fn app_delegates_worker_job_and_resume_planning_without_queue_or_kubernetes_io() {
        let mut app = TriggerOrchestratorApp::new(app_config()).expect("valid app");
        let plan = app
            .plan_trigger_in_process(
                context("idem:trigger-app:worker"),
                TriggerOrchestratorSdkSource::Scheduler,
                trigger_descriptor("trigger:worker-daily-invoice"),
                TriggerOrchestratorSdkSourceMetadata::Scheduler(schedule_descriptor()),
            )
            .expect("plan");
        let receipt = app.run_worker_job_once(TriggerOrchestratorWorkerJob {
            job_id: "job:trigger-app:worker:1".to_owned(),
            lease_id: "lease:trigger-app:worker:1".to_owned(),
            worker_ref: "worker:trigger-app:pod-1".to_owned(),
            attempt_id: "attempt:trigger-app:worker:1".to_owned(),
            attempt_number: 1,
            max_attempts: 3,
            now_epoch_seconds: 1_750_000_010,
            not_before_epoch_seconds: 1_750_000_000,
            lease_expires_epoch_seconds: 1_750_000_300,
            request: plan.rest_request.body,
        });
        assert_eq!(
            receipt.status,
            TriggerOrchestratorWorkerStatus::DispatchPlanned
        );
        assert_eq!(app.worker_api_apply_count(), 1);
        assert!(app.worker_event_count() >= 2);

        let resume = app.plan_worker_resume(
            TriggerOrchestratorWorkerResumeThrottle {
                max_resumes_per_tick: 1,
            },
            vec![
                TriggerOrchestratorWorkerResumeCandidate {
                    tenant_id: "ten_trigger_app".to_owned(),
                    trigger_id: "trigger:resume:2".to_owned(),
                    workflow_spec_id: "workflow:invoice-approval".to_owned(),
                    due_epoch_seconds: 20,
                    resume_priority: 10,
                    resume_evidence_ref: "resume:trigger-app:2".to_owned(),
                },
                TriggerOrchestratorWorkerResumeCandidate {
                    tenant_id: "ten_trigger_app".to_owned(),
                    trigger_id: "trigger:resume:1".to_owned(),
                    workflow_spec_id: "workflow:invoice-approval".to_owned(),
                    due_epoch_seconds: 10,
                    resume_priority: 1,
                    resume_evidence_ref: "resume:trigger-app:1".to_owned(),
                },
            ],
        );
        assert_eq!(resume.accepted.len(), 1);
        assert_eq!(resume.accepted[0].trigger_id, "trigger:resume:1");
        assert_eq!(resume.deferred.len(), 1);
    }

    #[test]
    fn unsafe_drain_reason_and_source_metadata_mismatch_are_denied_before_rest() {
        let mut app = TriggerOrchestratorApp::new(app_config()).expect("valid app");
        let error = app
            .mark_draining("raw prompt: drain for customer message")
            .expect_err("unsafe drain denied");
        assert_eq!(
            error.primary_evidence_ref(),
            "workflow-trigger-app:unsafe-drain-reason"
        );
        assert_eq!(
            app.probe(TriggerOrchestratorAppProbeKind::Readiness).status,
            TriggerOrchestratorAppProbeStatus::Serving
        );

        let error = app
            .evaluate_trigger_in_process(
                context("idem:trigger-app:mismatch"),
                TriggerOrchestratorSdkSource::Scheduler,
                trigger_descriptor("trigger:mismatch"),
                TriggerOrchestratorSdkSourceMetadata::MetadataOnly,
            )
            .expect_err("metadata mismatch denied");
        assert_eq!(
            error.primary_evidence_ref(),
            "workflow-trigger-sdk:source-metadata-mismatch"
        );
        assert_eq!(app.rest_api_delegation_count(), 0);
    }
}
