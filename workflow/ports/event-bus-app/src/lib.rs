//! Workflow-engine event-bus app foundation.
//!
//! This crate provides a source-level app composition root for the preview
//! event-bus SDK, REST/API, and worker seams. It validates metadata refs for
//! later broker clusters, topic catalogs, durable outbox/inbox stores, Valkey
//! leases, consumer groups, offset stores, Cedar bundles, OpenBao credentials,
//! audit-chain emission, and Oyatie Cloud dogfood workload binding; exposes
//! Kubernetes-shaped startup/liveness/readiness probe decisions and
//! OpenTelemetry-shaped service resource attributes; and supports in-process
//! preview publish and delivery-evaluation execution for tests. It performs no
//! binary startup, environment loading, HTTP serving, socket/DNS I/O, broker
//! connection, topic creation, database connection, Valkey lease coordination,
//! consumer group coordination, offset commits, payload serialization, Cedar
//! evaluation, OpenBao secret materialization, audit-chain sealing, filesystem
//! access, random/UUID generation, wall-clock reads, Kubernetes API calls,
//! container orchestration, cloud deployment, or tenant workload scheduling.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use workflow_event_bus_sdk::{
    WORKFLOW_EVENT_BUS_API_DECLARED_VERSION, WORKFLOW_EVENT_BUS_API_DELIVERY_ROUTE,
    WORKFLOW_EVENT_BUS_API_METHOD, WORKFLOW_EVENT_BUS_API_PUBLISH_ROUTE,
    WORKFLOW_EVENT_BUS_API_SURFACE, WORKFLOW_EVENT_BUS_REST_DELIVERY_ROUTE,
    WORKFLOW_EVENT_BUS_REST_METHOD, WORKFLOW_EVENT_BUS_REST_PUBLISH_ROUTE,
    WORKFLOW_EVENT_BUS_SDK_DECLARED_VERSION, WorkflowEventBusApi, WorkflowEventBusApiStatus,
    WorkflowEventBusEventKind, WorkflowEventBusRestResponse, WorkflowEventBusRestResponseBody,
    WorkflowEventBusRestService, WorkflowEventBusSdk, WorkflowEventBusSdkCommandPlan,
    WorkflowEventBusSdkConfig, WorkflowEventBusSdkDeliveryDescriptor, WorkflowEventBusSdkError,
    WorkflowEventBusSdkPublishDescriptor, WorkflowEventBusSdkRequestContext,
};
pub use workflow_event_bus_worker::{
    WorkflowEventBusWorker, WorkflowEventBusWorkerJob, WorkflowEventBusWorkerJobBody,
    WorkflowEventBusWorkerReceipt, WorkflowEventBusWorkerResumeCandidate,
    WorkflowEventBusWorkerResumePlan, WorkflowEventBusWorkerResumeThrottle,
    WorkflowEventBusWorkerStatus, plan_resume_candidates,
};

pub const WORKFLOW_EVENT_BUS_APP_SURFACE: &str = "workflow-engine.event-bus.app";
pub const WORKFLOW_EVENT_BUS_APP_SERVICE_NAMESPACE: &str = "oyatie.workflow-engine";
pub const WORKFLOW_EVENT_BUS_APP_SERVICE_NAME: &str = "workflow-engine-event-bus";
pub const WORKFLOW_EVENT_BUS_APP_STARTUP_PROBE_PATH: &str = "/healthz/startup";
pub const WORKFLOW_EVENT_BUS_APP_LIVENESS_PROBE_PATH: &str = "/healthz/live";
pub const WORKFLOW_EVENT_BUS_APP_READINESS_PROBE_PATH: &str = "/healthz/ready";
pub const WORKFLOW_EVENT_BUS_APP_DOGFOOD_SUBSTRATE_REF_PREFIX: &str = "oyatie-cloud:";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkflowEventBusAppComponent {
    RestApi,
    SdkFacade,
    Worker,
    EventPublisher,
    DeliveryEvaluator,
    BrokerClusterRef,
    TopicCatalogRef,
    DurableOutboxRef,
    DurableInboxRef,
    ValkeyLeaseRef,
    ConsumerGroupRef,
    OffsetStoreRef,
    CedarPolicyBundle,
    OpenBaoCredentialRef,
    AuditChainRef,
    OyatieCloudTenantWorkload,
}

impl WorkflowEventBusAppComponent {
    pub const fn as_ref(self) -> &'static str {
        match self {
            Self::RestApi => "component:event-bus:rest-api",
            Self::SdkFacade => "component:event-bus:sdk-facade",
            Self::Worker => "component:event-bus:worker",
            Self::EventPublisher => "component:event-bus:event-publisher",
            Self::DeliveryEvaluator => "component:event-bus:delivery-evaluator",
            Self::BrokerClusterRef => "component:event-bus:broker-cluster-ref",
            Self::TopicCatalogRef => "component:event-bus:topic-catalog-ref",
            Self::DurableOutboxRef => "component:event-bus:durable-outbox-ref",
            Self::DurableInboxRef => "component:event-bus:durable-inbox-ref",
            Self::ValkeyLeaseRef => "component:event-bus:valkey-lease-ref",
            Self::ConsumerGroupRef => "component:event-bus:consumer-group-ref",
            Self::OffsetStoreRef => "component:event-bus:offset-store-ref",
            Self::CedarPolicyBundle => "component:event-bus:cedar-policy-bundle",
            Self::OpenBaoCredentialRef => "component:event-bus:openbao-credential-ref",
            Self::AuditChainRef => "component:event-bus:audit-chain-ref",
            Self::OyatieCloudTenantWorkload => "component:event-bus:oyatie-cloud-tenant-workload",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkflowEventBusAppProbeKind {
    Startup,
    Liveness,
    Readiness,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkflowEventBusAppProbeStatus {
    Serving,
    NotReady,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusAppProbeReport {
    pub kind: WorkflowEventBusAppProbeKind, // data_class: PUBLIC
    pub status: WorkflowEventBusAppProbeStatus, // data_class: PUBLIC
    pub path: String,                       // data_class: PUBLIC
    pub checked_components: Vec<String>,    // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusAppTelemetryAttribute {
    pub key: String,   // data_class: PUBLIC
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusAppWorkloadPlan {
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
    pub telemetry_attributes: Vec<WorkflowEventBusAppTelemetryAttribute>, // data_class: INTERNAL_ONLY
    pub component_refs: Vec<String>, // data_class: INTERNAL_ONLY
    pub non_claim_refs: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusAppConfig {
    pub tenant_id: String,                     // data_class: INTERNAL_ONLY
    pub cell_id: String,                       // data_class: INTERNAL_ONLY
    pub service_instance_ref: String,          // data_class: INTERNAL_ONLY
    pub deployment_environment_ref: String,    // data_class: INTERNAL_ONLY
    pub cloud_substrate_ref: String,           // data_class: INTERNAL_ONLY
    pub cedar_bundle_ref: String,              // data_class: INTERNAL_ONLY
    pub broker_cluster_ref: String,            // data_class: INTERNAL_ONLY
    pub topic_catalog_ref: String,             // data_class: INTERNAL_ONLY
    pub postgres_outbox_ref: String,           // data_class: INTERNAL_ONLY
    pub postgres_inbox_ref: String,            // data_class: INTERNAL_ONLY
    pub valkey_lease_ref: String,              // data_class: INTERNAL_ONLY
    pub consumer_group_ref: String,            // data_class: INTERNAL_ONLY
    pub offset_store_ref: String,              // data_class: INTERNAL_ONLY
    pub openbao_credential_ref: String,        // data_class: INTERNAL_ONLY
    pub audit_chain_ref: String,               // data_class: INTERNAL_ONLY
    pub sdk_config: WorkflowEventBusSdkConfig, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkflowEventBusAppError {
    InvalidConfig { evidence_ref: String },
    SdkConfigRejected { evidence_ref: String },
    UnsafeMetadata { evidence_ref: String },
    RestRejected { evidence_ref: String },
}

impl WorkflowEventBusAppError {
    pub fn primary_evidence_ref(&self) -> &str {
        match self {
            Self::InvalidConfig { evidence_ref }
            | Self::SdkConfigRejected { evidence_ref }
            | Self::UnsafeMetadata { evidence_ref }
            | Self::RestRejected { evidence_ref } => evidence_ref,
        }
    }
}

pub struct WorkflowEventBusApp {
    config: WorkflowEventBusAppConfig,
    sdk: WorkflowEventBusSdk,
    rest: WorkflowEventBusRestService,
    worker: WorkflowEventBusWorker,
    accepting_traffic: bool,
}

impl WorkflowEventBusApp {
    pub fn new(config: WorkflowEventBusAppConfig) -> Result<Self, WorkflowEventBusAppError> {
        validate_config(&config)?;
        let sdk = WorkflowEventBusSdk::new(config.sdk_config.clone()).map_err(|error| {
            WorkflowEventBusAppError::SdkConfigRejected {
                evidence_ref: error.primary_evidence_ref().to_owned(),
            }
        })?;
        Ok(Self {
            config,
            sdk,
            rest: WorkflowEventBusRestService::default(),
            worker: WorkflowEventBusWorker::new(WorkflowEventBusApi::default()),
            accepting_traffic: true,
        })
    }

    pub fn workload_plan(&self) -> WorkflowEventBusAppWorkloadPlan {
        WorkflowEventBusAppWorkloadPlan {
            service_namespace: WORKFLOW_EVENT_BUS_APP_SERVICE_NAMESPACE.to_owned(),
            service_name: WORKFLOW_EVENT_BUS_APP_SERVICE_NAME.to_owned(),
            service_instance_ref: self.config.service_instance_ref.clone(),
            tenant_id: self.config.tenant_id.clone(),
            cell_id: self.config.cell_id.clone(),
            cloud_substrate_ref: self.config.cloud_substrate_ref.clone(),
            dogfood_tenant_workload: self
                .config
                .cloud_substrate_ref
                .starts_with(WORKFLOW_EVENT_BUS_APP_DOGFOOD_SUBSTRATE_REF_PREFIX),
            startup_probe_path: WORKFLOW_EVENT_BUS_APP_STARTUP_PROBE_PATH.to_owned(),
            liveness_probe_path: WORKFLOW_EVENT_BUS_APP_LIVENESS_PROBE_PATH.to_owned(),
            readiness_probe_path: WORKFLOW_EVENT_BUS_APP_READINESS_PROBE_PATH.to_owned(),
            telemetry_attributes: vec![
                attr(
                    "service.namespace",
                    WORKFLOW_EVENT_BUS_APP_SERVICE_NAMESPACE,
                ),
                attr("service.name", WORKFLOW_EVENT_BUS_APP_SERVICE_NAME),
                attr("service.instance.id", &self.config.service_instance_ref),
                attr(
                    "deployment.environment.ref",
                    &self.config.deployment_environment_ref,
                ),
                attr("k8s.namespace.name", &self.config.tenant_id),
                attr("oyatie.cell.ref", &self.config.cell_id),
            ],
            component_refs: sorted_unique(all_component_refs()),
            non_claim_refs: sorted_unique(vec![
                "workflow-event-bus-app:source-only-composition".to_owned(),
                "workflow-event-bus-app:no-cloud-runtime-deployment".to_owned(),
                "workflow-event-bus-app:no-broker-topic-runtime".to_owned(),
                "workflow-event-bus-app:no-consumer-group-or-offset-runtime".to_owned(),
                "workflow-event-bus-app:no-hyperscaler-claim".to_owned(),
            ]),
        }
    }

    pub fn probe(&self, kind: WorkflowEventBusAppProbeKind) -> WorkflowEventBusAppProbeReport {
        match kind {
            WorkflowEventBusAppProbeKind::Startup => probe_report(
                kind,
                WorkflowEventBusAppProbeStatus::Serving,
                WORKFLOW_EVENT_BUS_APP_STARTUP_PROBE_PATH,
                vec![
                    WorkflowEventBusAppComponent::RestApi,
                    WorkflowEventBusAppComponent::SdkFacade,
                    WorkflowEventBusAppComponent::Worker,
                ],
                vec!["workflow-event-bus-app:startup-configured".to_owned()],
            ),
            WorkflowEventBusAppProbeKind::Liveness => probe_report(
                kind,
                WorkflowEventBusAppProbeStatus::Serving,
                WORKFLOW_EVENT_BUS_APP_LIVENESS_PROBE_PATH,
                vec![
                    WorkflowEventBusAppComponent::RestApi,
                    WorkflowEventBusAppComponent::Worker,
                ],
                vec!["workflow-event-bus-app:liveness-shallow".to_owned()],
            ),
            WorkflowEventBusAppProbeKind::Readiness => {
                let status = if self.accepting_traffic {
                    WorkflowEventBusAppProbeStatus::Serving
                } else {
                    WorkflowEventBusAppProbeStatus::NotReady
                };
                probe_report(
                    kind,
                    status,
                    WORKFLOW_EVENT_BUS_APP_READINESS_PROBE_PATH,
                    vec![
                        WorkflowEventBusAppComponent::BrokerClusterRef,
                        WorkflowEventBusAppComponent::TopicCatalogRef,
                        WorkflowEventBusAppComponent::DurableOutboxRef,
                        WorkflowEventBusAppComponent::DurableInboxRef,
                        WorkflowEventBusAppComponent::ValkeyLeaseRef,
                        WorkflowEventBusAppComponent::ConsumerGroupRef,
                        WorkflowEventBusAppComponent::OffsetStoreRef,
                        WorkflowEventBusAppComponent::CedarPolicyBundle,
                        WorkflowEventBusAppComponent::OpenBaoCredentialRef,
                        WorkflowEventBusAppComponent::AuditChainRef,
                        WorkflowEventBusAppComponent::OyatieCloudTenantWorkload,
                    ],
                    vec![if self.accepting_traffic {
                        "workflow-event-bus-app:readiness-configured".to_owned()
                    } else {
                        "workflow-event-bus-app:readiness-draining".to_owned()
                    }],
                )
            }
        }
    }

    pub fn mark_draining(&mut self, reason_ref: &str) -> Result<(), WorkflowEventBusAppError> {
        if !is_safe_ref(reason_ref) {
            return Err(WorkflowEventBusAppError::UnsafeMetadata {
                evidence_ref: "workflow-event-bus-app:unsafe-drain-reason".to_owned(),
            });
        }
        self.accepting_traffic = false;
        Ok(())
    }

    pub fn mark_accepting_traffic(&mut self) {
        self.accepting_traffic = true;
    }

    pub fn plan_publish_in_process(
        &self,
        context: WorkflowEventBusSdkRequestContext,
        descriptor: WorkflowEventBusSdkPublishDescriptor,
    ) -> Result<WorkflowEventBusSdkCommandPlan, WorkflowEventBusAppError> {
        self.sdk
            .plan_publish(context, descriptor)
            .map_err(app_error_from_sdk)
    }

    pub fn publish_in_process(
        &mut self,
        context: WorkflowEventBusSdkRequestContext,
        descriptor: WorkflowEventBusSdkPublishDescriptor,
    ) -> Result<WorkflowEventBusRestResponse, WorkflowEventBusAppError> {
        let plan = self.plan_publish_in_process(context, descriptor)?;
        self.sdk
            .execute_in_process(plan, &mut self.rest)
            .map_err(|error| WorkflowEventBusAppError::RestRejected {
                evidence_ref: error.primary_evidence_ref().to_owned(),
            })
    }

    pub fn plan_delivery_in_process(
        &self,
        context: WorkflowEventBusSdkRequestContext,
        descriptor: WorkflowEventBusSdkDeliveryDescriptor,
    ) -> Result<WorkflowEventBusSdkCommandPlan, WorkflowEventBusAppError> {
        self.sdk
            .plan_delivery(context, descriptor)
            .map_err(app_error_from_sdk)
    }

    pub fn evaluate_delivery_in_process(
        &mut self,
        context: WorkflowEventBusSdkRequestContext,
        descriptor: WorkflowEventBusSdkDeliveryDescriptor,
    ) -> Result<WorkflowEventBusRestResponse, WorkflowEventBusAppError> {
        let plan = self.plan_delivery_in_process(context, descriptor)?;
        self.sdk
            .execute_in_process(plan, &mut self.rest)
            .map_err(|error| WorkflowEventBusAppError::RestRejected {
                evidence_ref: error.primary_evidence_ref().to_owned(),
            })
    }

    pub fn run_worker_job_once(
        &mut self,
        job: WorkflowEventBusWorkerJob,
    ) -> WorkflowEventBusWorkerReceipt {
        self.worker.run_once(job)
    }

    pub fn plan_worker_resume(
        &self,
        throttle: WorkflowEventBusWorkerResumeThrottle,
        candidates: Vec<WorkflowEventBusWorkerResumeCandidate>,
    ) -> WorkflowEventBusWorkerResumePlan {
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

fn app_error_from_sdk(error: WorkflowEventBusSdkError) -> WorkflowEventBusAppError {
    match error {
        WorkflowEventBusSdkError::InvalidConfig { evidence_ref } => {
            WorkflowEventBusAppError::SdkConfigRejected { evidence_ref }
        }
        WorkflowEventBusSdkError::InvalidRequest { evidence_ref }
        | WorkflowEventBusSdkError::MetadataMismatch { evidence_ref }
        | WorkflowEventBusSdkError::UnsafeMetadata { evidence_ref } => {
            WorkflowEventBusAppError::UnsafeMetadata { evidence_ref }
        }
        WorkflowEventBusSdkError::RestRejected { evidence_ref } => {
            WorkflowEventBusAppError::RestRejected { evidence_ref }
        }
    }
}

fn validate_config(config: &WorkflowEventBusAppConfig) -> Result<(), WorkflowEventBusAppError> {
    if config.tenant_id != config.sdk_config.tenant_id {
        return Err(WorkflowEventBusAppError::InvalidConfig {
            evidence_ref: "workflow-event-bus-app:tenant-sdk-drift".to_owned(),
        });
    }
    if config.cell_id != config.sdk_config.default_cell_id {
        return Err(WorkflowEventBusAppError::InvalidConfig {
            evidence_ref: "workflow-event-bus-app:cell-sdk-drift".to_owned(),
        });
    }
    if config.audit_chain_ref != config.sdk_config.default_audit_chain_ref {
        return Err(WorkflowEventBusAppError::InvalidConfig {
            evidence_ref: "workflow-event-bus-app:audit-chain-sdk-drift".to_owned(),
        });
    }
    if !is_safe_tenant(&config.tenant_id)
        || !is_safe_ref(&config.cell_id)
        || !is_safe_ref(&config.service_instance_ref)
        || !is_safe_ref(&config.deployment_environment_ref)
        || !is_safe_ref(&config.cloud_substrate_ref)
        || !is_safe_ref(&config.cedar_bundle_ref)
        || !is_safe_ref(&config.broker_cluster_ref)
        || !is_safe_ref(&config.topic_catalog_ref)
        || !is_safe_ref(&config.postgres_outbox_ref)
        || !is_safe_ref(&config.postgres_inbox_ref)
        || !is_safe_ref(&config.valkey_lease_ref)
        || !is_safe_ref(&config.consumer_group_ref)
        || !is_safe_ref(&config.offset_store_ref)
        || !is_safe_ref(&config.openbao_credential_ref)
        || !is_safe_ref(&config.audit_chain_ref)
    {
        return Err(WorkflowEventBusAppError::InvalidConfig {
            evidence_ref: "workflow-event-bus-app:invalid-config-metadata".to_owned(),
        });
    }
    if !config
        .cloud_substrate_ref
        .starts_with(WORKFLOW_EVENT_BUS_APP_DOGFOOD_SUBSTRATE_REF_PREFIX)
    {
        return Err(WorkflowEventBusAppError::InvalidConfig {
            evidence_ref: "workflow-event-bus-app:cloud-substrate-ref-required".to_owned(),
        });
    }
    Ok(())
}

fn probe_report(
    kind: WorkflowEventBusAppProbeKind,
    status: WorkflowEventBusAppProbeStatus,
    path: &str,
    checked_components: Vec<WorkflowEventBusAppComponent>,
    evidence_refs: Vec<String>,
) -> WorkflowEventBusAppProbeReport {
    WorkflowEventBusAppProbeReport {
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

fn attr(key: &str, value: &str) -> WorkflowEventBusAppTelemetryAttribute {
    WorkflowEventBusAppTelemetryAttribute {
        key: key.to_owned(),
        value: value.to_owned(),
    }
}

fn all_component_refs() -> Vec<String> {
    vec![
        WorkflowEventBusAppComponent::RestApi.as_ref().to_owned(),
        WorkflowEventBusAppComponent::SdkFacade.as_ref().to_owned(),
        WorkflowEventBusAppComponent::Worker.as_ref().to_owned(),
        WorkflowEventBusAppComponent::EventPublisher
            .as_ref()
            .to_owned(),
        WorkflowEventBusAppComponent::DeliveryEvaluator
            .as_ref()
            .to_owned(),
        WorkflowEventBusAppComponent::BrokerClusterRef
            .as_ref()
            .to_owned(),
        WorkflowEventBusAppComponent::TopicCatalogRef
            .as_ref()
            .to_owned(),
        WorkflowEventBusAppComponent::DurableOutboxRef
            .as_ref()
            .to_owned(),
        WorkflowEventBusAppComponent::DurableInboxRef
            .as_ref()
            .to_owned(),
        WorkflowEventBusAppComponent::ValkeyLeaseRef
            .as_ref()
            .to_owned(),
        WorkflowEventBusAppComponent::ConsumerGroupRef
            .as_ref()
            .to_owned(),
        WorkflowEventBusAppComponent::OffsetStoreRef
            .as_ref()
            .to_owned(),
        WorkflowEventBusAppComponent::CedarPolicyBundle
            .as_ref()
            .to_owned(),
        WorkflowEventBusAppComponent::OpenBaoCredentialRef
            .as_ref()
            .to_owned(),
        WorkflowEventBusAppComponent::AuditChainRef
            .as_ref()
            .to_owned(),
        WorkflowEventBusAppComponent::OyatieCloudTenantWorkload
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
        || lower.contains("raw payload")
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

    #[test]
    fn workload_plan_models_oyatie_cloud_dogfood_and_otel_service_identity() {
        let app = WorkflowEventBusApp::new(app_config()).expect("valid app");
        let plan = app.workload_plan();
        assert_eq!(
            plan.service_namespace,
            WORKFLOW_EVENT_BUS_APP_SERVICE_NAMESPACE
        );
        assert_eq!(plan.service_name, WORKFLOW_EVENT_BUS_APP_SERVICE_NAME);
        assert!(plan.dogfood_tenant_workload);
        assert_eq!(
            plan.startup_probe_path,
            WORKFLOW_EVENT_BUS_APP_STARTUP_PROBE_PATH
        );
        assert!(
            plan.telemetry_attributes
                .iter()
                .any(|attr| attr.key == "service.name"
                    && attr.value == WORKFLOW_EVENT_BUS_APP_SERVICE_NAME)
        );
        assert!(
            plan.telemetry_attributes
                .iter()
                .any(|attr| attr.key == "service.instance.id")
        );
        assert!(
            plan.telemetry_attributes
                .iter()
                .any(|attr| attr.key == "k8s.namespace.name")
        );
        assert!(
            plan.component_refs.contains(
                &WorkflowEventBusAppComponent::OyatieCloudTenantWorkload
                    .as_ref()
                    .to_owned()
            )
        );
        assert!(
            plan.non_claim_refs
                .contains(&"workflow-event-bus-app:no-hyperscaler-claim".to_owned())
        );
    }

    #[test]
    fn config_validation_rejects_missing_cloud_ref_sdk_drift_and_raw_secret_without_echo() {
        let missing_cloud = WorkflowEventBusAppConfig {
            cloud_substrate_ref: "substrate:external-k8s".to_owned(),
            ..app_config()
        };
        let error = new_app_error(missing_cloud);
        assert_eq!(
            error.primary_evidence_ref(),
            "workflow-event-bus-app:cloud-substrate-ref-required"
        );

        let tenant_drift = WorkflowEventBusAppConfig {
            sdk_config: WorkflowEventBusSdkConfig {
                tenant_id: "ten_other".to_owned(),
                ..sdk_config()
            },
            ..app_config()
        };
        let error = new_app_error(tenant_drift);
        assert_eq!(
            error.primary_evidence_ref(),
            "workflow-event-bus-app:tenant-sdk-drift"
        );

        let audit_drift = WorkflowEventBusAppConfig {
            sdk_config: WorkflowEventBusSdkConfig {
                default_audit_chain_ref: "audit-chain:event-bus-sdk-drift".to_owned(),
                ..sdk_config()
            },
            ..app_config()
        };
        let error = new_app_error(audit_drift);
        assert_eq!(
            error.primary_evidence_ref(),
            "workflow-event-bus-app:audit-chain-sdk-drift"
        );

        let raw_secret = WorkflowEventBusAppConfig {
            openbao_credential_ref: "secret=super-secret-token".to_owned(),
            ..app_config()
        };
        let error = new_app_error(raw_secret);
        assert_eq!(
            error.primary_evidence_ref(),
            "workflow-event-bus-app:invalid-config-metadata"
        );
        assert!(!format!("{error:?}").contains("super-secret-token"));
    }

    #[test]
    fn startup_liveness_and_readiness_are_distinct_and_readiness_tracks_draining() {
        let mut app = WorkflowEventBusApp::new(app_config()).expect("valid app");
        let startup = app.probe(WorkflowEventBusAppProbeKind::Startup);
        assert_eq!(startup.status, WorkflowEventBusAppProbeStatus::Serving);
        assert!(
            startup
                .checked_components
                .contains(&WorkflowEventBusAppComponent::SdkFacade.as_ref().to_owned())
        );

        let liveness = app.probe(WorkflowEventBusAppProbeKind::Liveness);
        assert_eq!(liveness.status, WorkflowEventBusAppProbeStatus::Serving);
        assert!(
            !liveness.checked_components.contains(
                &WorkflowEventBusAppComponent::BrokerClusterRef
                    .as_ref()
                    .to_owned()
            )
        );

        let readiness = app.probe(WorkflowEventBusAppProbeKind::Readiness);
        assert_eq!(readiness.status, WorkflowEventBusAppProbeStatus::Serving);
        assert!(
            readiness.checked_components.contains(
                &WorkflowEventBusAppComponent::BrokerClusterRef
                    .as_ref()
                    .to_owned()
            )
        );
        app.mark_draining("drain:event-bus-app:rolling-update")
            .expect("safe drain");
        let readiness = app.probe(WorkflowEventBusAppProbeKind::Readiness);
        assert_eq!(readiness.status, WorkflowEventBusAppProbeStatus::NotReady);
        assert!(
            readiness
                .evidence_refs
                .contains(&"workflow-event-bus-app:readiness-draining".to_owned())
        );
    }

    #[test]
    fn app_composes_sdk_rest_api_for_in_process_publish_preview() {
        let mut app = WorkflowEventBusApp::new(app_config()).expect("valid app");
        let response = app
            .publish_in_process(context("idem:event-bus-app:publish"), publish_descriptor())
            .expect("publish response");
        assert_eq!(response.status_code, 202);
        assert_eq!(app.rest_api_delegation_count(), 1);
        let WorkflowEventBusRestResponseBody::Success(success) = response.body else {
            unreachable!("expected event-bus success");
        };
        assert_eq!(success.event.operation, "publish");
        assert_eq!(success.metadata.surface, WORKFLOW_EVENT_BUS_API_SURFACE);
        assert!(!format!("{success:?}").contains("raw payload"));
    }

    #[test]
    fn app_evaluates_delivery_and_preserves_no_offset_commit_non_claim() {
        let mut app = WorkflowEventBusApp::new(app_config()).expect("valid app");
        let response = app
            .evaluate_delivery_in_process(
                context("idem:event-bus-app:delivery"),
                delivery_descriptor(),
            )
            .expect("delivery response");
        assert_eq!(response.status_code, 202);
        let WorkflowEventBusRestResponseBody::Success(success) = response.body else {
            unreachable!("expected delivery success");
        };
        assert_eq!(success.event.operation, "delivery-evaluate");
        assert_eq!(success.event.usecase_status, "delivery-accepted");
        assert!(
            success
                .non_claim_refs
                .contains(&"workflow-event-bus-usecase:no-offset-commit-runtime".to_owned())
        );
    }

    #[test]
    fn app_preserves_rest_idempotent_replay_without_runtime_id_generation() {
        let mut app = WorkflowEventBusApp::new(app_config()).expect("valid app");
        let plan = app
            .plan_publish_in_process(context("idem:event-bus-app:replay"), publish_descriptor())
            .expect("plan");
        let first = app
            .sdk
            .execute_in_process(plan.clone(), &mut app.rest)
            .expect("first");
        let second = app
            .sdk
            .execute_in_process(plan, &mut app.rest)
            .expect("second");
        assert_eq!(first, second);
        assert_eq!(app.rest_api_delegation_count(), 2);
    }

    #[test]
    fn app_delegates_worker_job_and_resume_planning_without_queue_or_kubernetes_io() {
        let mut app = WorkflowEventBusApp::new(app_config()).expect("valid app");
        let plan = app
            .plan_publish_in_process(context("idem:event-bus-app:worker"), publish_descriptor())
            .expect("plan");
        let body = worker_body_from_plan(plan);
        let receipt = app.run_worker_job_once(WorkflowEventBusWorkerJob {
            job_id: "job:event-bus-app:worker:1".to_owned(),
            lease_id: "lease:event-bus-app:worker:1".to_owned(),
            worker_ref: "worker:event-bus-app:pod-1".to_owned(),
            attempt_id: "attempt:event-bus-app:worker:1".to_owned(),
            attempt_number: 1,
            max_attempts: 3,
            now_epoch_seconds: 1_750_000_010,
            not_before_epoch_seconds: 1_750_000_000,
            lease_expires_epoch_seconds: 1_750_000_300,
            body,
        });
        assert_eq!(receipt.status, WorkflowEventBusWorkerStatus::Published);
        assert_eq!(app.worker_api_apply_count(), 1);
        assert!(app.worker_event_count() >= 2);

        let resume = app.plan_worker_resume(
            WorkflowEventBusWorkerResumeThrottle {
                max_resumes_per_tick: 1,
            },
            vec![
                resume_candidate("event:resume:2", 20, 10),
                resume_candidate("event:resume:1", 10, 1),
            ],
        );
        assert_eq!(resume.accepted.len(), 1);
        assert_eq!(resume.accepted[0].event_id, "event:resume:1");
        assert_eq!(resume.deferred.len(), 1);
    }

    #[test]
    fn unsafe_drain_reason_and_sdk_metadata_mismatch_are_denied_before_rest() {
        let mut app = WorkflowEventBusApp::new(app_config()).expect("valid app");
        let error = app
            .mark_draining("raw prompt: drain for customer message")
            .expect_err("unsafe drain denied");
        assert_eq!(
            error.primary_evidence_ref(),
            "workflow-event-bus-app:unsafe-drain-reason"
        );
        assert_eq!(
            app.probe(WorkflowEventBusAppProbeKind::Readiness).status,
            WorkflowEventBusAppProbeStatus::Serving
        );

        let mut descriptor = publish_descriptor();
        descriptor.event_kind = WorkflowEventBusEventKind::OntologyProjectionUpdated
            .event_type()
            .to_owned();
        let error = app
            .publish_in_process(context("idem:event-bus-app:mismatch"), descriptor)
            .expect_err("metadata mismatch denied");
        assert_eq!(
            error.primary_evidence_ref(),
            "workflow-event-bus-sdk:publish-event-type-not-allowed"
        );
        assert_eq!(app.rest_api_delegation_count(), 0);
    }

    fn new_app_error(config: WorkflowEventBusAppConfig) -> WorkflowEventBusAppError {
        match WorkflowEventBusApp::new(config) {
            Ok(_) => unreachable!("expected invalid app config"),
            Err(error) => error,
        }
    }

    fn app_config() -> WorkflowEventBusAppConfig {
        WorkflowEventBusAppConfig {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            cell_id: "cell:us-east-1a".to_owned(),
            service_instance_ref: "pod:event-bus-app:001".to_owned(),
            deployment_environment_ref: "env:dev".to_owned(),
            cloud_substrate_ref: "oyatie-cloud:dogfood:workflow-engine:event-bus".to_owned(),
            cedar_bundle_ref: "cedar:event-bus:bundle:v1".to_owned(),
            broker_cluster_ref: "broker:event-bus:cluster:preview".to_owned(),
            topic_catalog_ref: "topic-catalog:event-bus:preview".to_owned(),
            postgres_outbox_ref: "postgres:event-bus:outbox".to_owned(),
            postgres_inbox_ref: "postgres:event-bus:inbox".to_owned(),
            valkey_lease_ref: "valkey:event-bus:leases".to_owned(),
            consumer_group_ref: "consumer-group:event-bus:workflow-engine".to_owned(),
            offset_store_ref: "offset-store:event-bus:workflow-engine".to_owned(),
            openbao_credential_ref: "openbao:event-bus:credentials".to_owned(),
            audit_chain_ref: "audit-chain:event-bus-app".to_owned(),
            sdk_config: sdk_config(),
        }
    }

    fn sdk_config() -> WorkflowEventBusSdkConfig {
        WorkflowEventBusSdkConfig {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            principal_id: "principal:workflow-operator".to_owned(),
            authorization_decision_id: "policy-decision:event-bus-allow".to_owned(),
            authorization_evidence_ref: "policy-evidence:event-bus-allow".to_owned(),
            policy_bundle_ref: "policy-bundle:event-bus-v1".to_owned(),
            default_cell_id: "cell:us-east-1a".to_owned(),
            default_residency_ref: "residency:us:data-plane".to_owned(),
            default_audit_chain_ref: "audit-chain:event-bus-app".to_owned(),
            trace_context_ref: "trace:event-bus-app".to_owned(),
            allowed_channels: vec!["workflow-runs".to_owned(), "workflow-state".to_owned()],
            allowed_event_types: vec![
                WorkflowEventBusEventKind::WorkflowRunStarted
                    .event_type()
                    .to_owned(),
                WorkflowEventBusEventKind::WorkflowStateTransitioned
                    .event_type()
                    .to_owned(),
            ],
            oyatie_version: WORKFLOW_EVENT_BUS_API_DECLARED_VERSION.to_owned(),
        }
    }

    fn context(idempotency_key: &str) -> WorkflowEventBusSdkRequestContext {
        WorkflowEventBusSdkRequestContext {
            request_id: format!("request:event-bus-app:{idempotency_key}"),
            idempotency_key: idempotency_key.to_owned(),
            trace_context_ref: None,
            correlation_ref: "corr:event-bus-app:request".to_owned(),
        }
    }

    fn publish_descriptor() -> WorkflowEventBusSdkPublishDescriptor {
        WorkflowEventBusSdkPublishDescriptor {
            event_kind: WorkflowEventBusEventKind::WorkflowRunStarted
                .event_type()
                .to_owned(),
            event_id: "event:workflow-run-started:001".to_owned(),
            producer_ref: "producer:workflow-engine:execution".to_owned(),
            source_ref: "urn:oyatie:workflow-engine:execution".to_owned(),
            subject_ref: Some("subject:workflow-run:001".to_owned()),
            time_ref: Some("time:2026-05-25T00:00:00Z".to_owned()),
            dataschema_ref: Some("schema:workflow-event-run-started".to_owned()),
            partition_key_ref: "partition:tenant-workflow-run".to_owned(),
            publish_idempotency_key: "idem:event-bus-domain:publish:001".to_owned(),
            causation_ref: "cause:execution-engine:start-run".to_owned(),
            correlation_ref: None,
            payload_ref: "body-ref:workflow-run-started".to_owned(),
            cell_id: None,
            residency_ref: None,
            audit_chain_ref: None,
            evidence_refs: vec!["evidence:event-bus-app:publish".to_owned()],
        }
    }

    fn delivery_descriptor() -> WorkflowEventBusSdkDeliveryDescriptor {
        WorkflowEventBusSdkDeliveryDescriptor {
            subscription_channel: "workflow-state".to_owned(),
            consumer_ref: "consumer:workflow-state-machine".to_owned(),
            subscription_event_types: vec![
                WorkflowEventBusEventKind::WorkflowStateTransitioned
                    .event_type()
                    .to_owned(),
            ],
            replay_cursor_ref: Some("cursor:event-bus-app:state".to_owned()),
            max_batch_size: 100,
            subscription_authorization_evidence_ref: "authz:event-bus-app:consume".to_owned(),
            candidate_channel: "workflow-state".to_owned(),
            candidate_event_id: "event:workflow-state:001".to_owned(),
            candidate_event_type: WorkflowEventBusEventKind::WorkflowStateTransitioned
                .event_type()
                .to_owned(),
            candidate_idempotency_key: "idem:event-bus-domain:delivery:001".to_owned(),
            candidate_payload_ref: "body-ref:workflow-state-transitioned".to_owned(),
            candidate_offset_ref: "offset:partition-0:42".to_owned(),
            cell_id: None,
            residency_ref: None,
            audit_chain_ref: None,
            candidate_evidence_refs: vec!["evidence:event-bus-app:delivery".to_owned()],
        }
    }

    fn worker_body_from_plan(
        plan: WorkflowEventBusSdkCommandPlan,
    ) -> WorkflowEventBusWorkerJobBody {
        match plan.rest_request.body {
            workflow_event_bus_sdk::WorkflowEventBusRestRequestBody::Publish(request) => {
                WorkflowEventBusWorkerJobBody::Publish(request)
            }
            workflow_event_bus_sdk::WorkflowEventBusRestRequestBody::Delivery(request) => {
                WorkflowEventBusWorkerJobBody::Delivery(request)
            }
        }
    }

    fn resume_candidate(
        event_id: &str,
        due_epoch_seconds: u64,
        resume_priority: u32,
    ) -> WorkflowEventBusWorkerResumeCandidate {
        WorkflowEventBusWorkerResumeCandidate {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            channel_address: "workflow-runs".to_owned(),
            event_id: event_id.to_owned(),
            event_type: WorkflowEventBusEventKind::WorkflowRunStarted
                .event_type()
                .to_owned(),
            due_epoch_seconds,
            resume_priority,
            resume_evidence_ref: format!("resume:event-bus-app:{event_id}"),
        }
    }
}
