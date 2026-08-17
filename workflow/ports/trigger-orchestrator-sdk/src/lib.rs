//! Workflow-engine trigger-orchestrator SDK foundation.
//!
//! This crate provides a source-level, language-native SDK facade over the
//! trigger-orchestrator REST/API boundary for future generated SDK work. It
//! builds typed trigger evaluation plans for scheduler, Studio webhook, sibling
//! event-bus, manual UI, API command, workflow-spawn, and ontology-projection
//! sources; binds version, tenant, principal, authorization, trace,
//! idempotency, route, and source-specific metadata; exposes an in-process
//! preview execution seam for local tests; and rejects raw
//! prompt/output/payload/secret-shaped material before delegating to REST/API.
//! It performs no HTTP client work, DNS, sockets, serialization-framework work,
//! credential loading, random/UUID generation, wall-clock reads, automatic
//! retries, durable idempotency, queueing, signing, filesystem access,
//! Kubernetes calls, cloud deployment, or tenant workload scheduling.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use workflow_trigger_orchestrator_rest::{
    TRIGGER_ORCHESTRATOR_API_DECLARED_VERSION, TRIGGER_ORCHESTRATOR_API_METHOD,
    TRIGGER_ORCHESTRATOR_API_ROUTE, TRIGGER_ORCHESTRATOR_API_SURFACE,
    TRIGGER_ORCHESTRATOR_CLOUDEVENTS_SPECVERSION, TRIGGER_ORCHESTRATOR_REST_CONTRACT_REF,
    TRIGGER_ORCHESTRATOR_REST_METHOD, TRIGGER_ORCHESTRATOR_REST_ROUTE,
    TriggerOrchestratorApiAuthorization, TriggerOrchestratorApiBoundaryContext,
    TriggerOrchestratorApiEventDto, TriggerOrchestratorApiPrincipal, TriggerOrchestratorApiRequest,
    TriggerOrchestratorApiScheduleDto, TriggerOrchestratorApiTriggerBody,
    TriggerOrchestratorApiWebhookDto, TriggerOrchestratorRestError, TriggerOrchestratorRestMethod,
    TriggerOrchestratorRestRequest, TriggerOrchestratorRestResponse,
    TriggerOrchestratorRestService,
};

pub const TRIGGER_ORCHESTRATOR_SDK_SURFACE: &str = "workflow-engine.trigger-orchestrator.sdk";
pub const TRIGGER_ORCHESTRATOR_SDK_CONTRACT_REF: &str = TRIGGER_ORCHESTRATOR_REST_CONTRACT_REF;
pub const TRIGGER_ORCHESTRATOR_SDK_DECLARED_VERSION: &str =
    TRIGGER_ORCHESTRATOR_API_DECLARED_VERSION;
pub const TRIGGER_ORCHESTRATOR_SDK_AUTOMATIC_RETRIES_ENABLED: bool = false;
pub const TRIGGER_ORCHESTRATOR_SDK_RETRY_POLICY_REF: &str =
    "workflow-trigger-sdk:automatic-retry-disabled";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TriggerOrchestratorSdkSource {
    Scheduler,
    StudioWebhook,
    SiblingEventBus,
    ManualUi,
    ApiCommand,
    WorkflowSpawn,
    OntologyProjection,
}

impl TriggerOrchestratorSdkSource {
    pub const fn source_wire(self) -> &'static str {
        match self {
            Self::Scheduler => "scheduler",
            Self::StudioWebhook => "studio-webhook",
            Self::SiblingEventBus => "sibling-event-bus",
            Self::ManualUi => "manual-ui",
            Self::ApiCommand => "api-command",
            Self::WorkflowSpawn => "workflow-spawn",
            Self::OntologyProjection => "ontology-projection",
        }
    }

    pub const fn trigger_kind_wire(self) -> &'static str {
        match self {
            Self::Scheduler => "cron",
            Self::StudioWebhook => "webhook",
            Self::SiblingEventBus => "event-bus",
            Self::ManualUi => "manual",
            Self::ApiCommand => "api",
            Self::WorkflowSpawn => "workflow-spawn",
            Self::OntologyProjection => "ontology",
        }
    }

    pub const fn operation_id(self) -> &'static str {
        match self {
            Self::Scheduler => "evaluateSchedulerTrigger",
            Self::StudioWebhook => "evaluateStudioWebhookTrigger",
            Self::SiblingEventBus => "evaluateSiblingEventBusTrigger",
            Self::ManualUi => "evaluateManualUiTrigger",
            Self::ApiCommand => "evaluateApiCommandTrigger",
            Self::WorkflowSpawn => "evaluateWorkflowSpawnTrigger",
            Self::OntologyProjection => "evaluateOntologyProjectionTrigger",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorSdkConfig {
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub principal_id: String,               // data_class: INTERNAL_ONLY
    pub authorization_decision_id: String,  // data_class: INTERNAL_ONLY
    pub authorization_evidence_ref: String, // data_class: INTERNAL_ONLY
    pub policy_bundle_ref: String,          // data_class: INTERNAL_ONLY
    pub default_workflow_spec_id: String,   // data_class: INTERNAL_ONLY
    pub default_version_sha: String,        // data_class: INTERNAL_ONLY
    pub default_cell_id: String,            // data_class: INTERNAL_ONLY
    pub authorization_surface_ref: String,  // data_class: INTERNAL_ONLY
    pub trace_context_ref: String,          // data_class: INTERNAL_ONLY
    pub oyatie_version: String,             // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorSdkRequestContext {
    pub request_id: String,                // data_class: INTERNAL_ONLY
    pub idempotency_key: String,           // data_class: INTERNAL_ONLY
    pub trace_context_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub run_idempotency_key: String,       // data_class: INTERNAL_ONLY
    pub correlation_ref: String,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorSdkTriggerDescriptor {
    pub trigger_id: String,                 // data_class: INTERNAL_ONLY
    pub workflow_spec_id: Option<String>,   // data_class: INTERNAL_ONLY
    pub version_sha: Option<String>,        // data_class: INTERNAL_ONLY
    pub active_cell_id: Option<String>,     // data_class: INTERNAL_ONLY
    pub trigger_lineage_ref: String,        // data_class: INTERNAL_ONLY
    pub source_evidence_ref: String,        // data_class: INTERNAL_ONLY
    pub replay_epoch_ref: String,           // data_class: INTERNAL_ONLY
    pub audit_chain_ref: String,            // data_class: INTERNAL_ONLY
    pub idempotency_scope_ref: String,      // data_class: INTERNAL_ONLY
    pub dry_run_reason_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub replay_mode: bool,                  // data_class: PUBLIC
    pub dry_run: bool,                      // data_class: PUBLIC
    pub evidence_refs: Vec<String>,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorSdkScheduleDescriptor {
    pub cron_expr_ref: String,                 // data_class: INTERNAL_ONLY
    pub timezone_ref: String,                  // data_class: INTERNAL_ONLY
    pub due_epoch_seconds: u64,                // data_class: INTERNAL_ONLY
    pub observed_epoch_seconds: u64,           // data_class: INTERNAL_ONLY
    pub catchup_window_seconds: u64,           // data_class: INTERNAL_ONLY
    pub overlap_policy: String,                // data_class: PUBLIC
    pub paused: bool,                          // data_class: PUBLIC
    pub pause_reason_ref: Option<String>,      // data_class: INTERNAL_ONLY
    pub last_fired_epoch_seconds: Option<u64>, // data_class: INTERNAL_ONLY
    pub scheduler_evidence_ref: String,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorSdkWebhookDescriptor {
    pub endpoint_ref: String,              // data_class: INTERNAL_ONLY
    pub signature_ref: String,             // data_class: INTERNAL_ONLY
    pub nonce_ref: String,                 // data_class: INTERNAL_ONLY
    pub hmac_key_ref: String,              // data_class: INTERNAL_ONLY
    pub received_epoch_seconds: u64,       // data_class: INTERNAL_ONLY
    pub expires_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
    pub webhook_auth_evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorSdkEventDescriptor {
    pub event_id: String,               // data_class: INTERNAL_ONLY
    pub source: String,                 // data_class: INTERNAL_ONLY
    pub event_type: String,             // data_class: INTERNAL_ONLY
    pub specversion: String,            // data_class: PUBLIC
    pub subject_ref: Option<String>,    // data_class: INTERNAL_ONLY
    pub event_time_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub correlation_id: String,         // data_class: INTERNAL_ONLY
    pub idempotency_key: String,        // data_class: INTERNAL_ONLY
    pub event_contract_ref: String,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TriggerOrchestratorSdkSourceMetadata {
    Scheduler(TriggerOrchestratorSdkScheduleDescriptor),
    StudioWebhook(TriggerOrchestratorSdkWebhookDescriptor),
    SiblingEventBus(TriggerOrchestratorSdkEventDescriptor),
    MetadataOnly,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorSdkRetryPolicy {
    pub automatic_retries_enabled: bool, // data_class: PUBLIC
    pub retry_policy_ref: String,        // data_class: INTERNAL_ONLY
}

impl Default for TriggerOrchestratorSdkRetryPolicy {
    fn default() -> Self {
        Self {
            automatic_retries_enabled: TRIGGER_ORCHESTRATOR_SDK_AUTOMATIC_RETRIES_ENABLED,
            retry_policy_ref: TRIGGER_ORCHESTRATOR_SDK_RETRY_POLICY_REF.to_owned(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorSdkTriggerPlan {
    pub source: TriggerOrchestratorSdkSource, // data_class: PUBLIC
    pub operation_id: String,                 // data_class: PUBLIC
    pub method: TriggerOrchestratorRestMethod, // data_class: PUBLIC
    pub path: String,                         // data_class: PUBLIC
    pub contract_ref: String,                 // data_class: INTERNAL_ONLY
    pub oyatie_version: String,               // data_class: PUBLIC
    pub retry_policy: TriggerOrchestratorSdkRetryPolicy, // data_class: INTERNAL_ONLY
    pub rest_request: TriggerOrchestratorRestRequest, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TriggerOrchestratorSdkError {
    InvalidConfig { evidence_ref: String },
    InvalidRequest { evidence_ref: String },
    MetadataMismatch { evidence_ref: String },
    RestRejected { evidence_ref: String },
    UnsafeMetadata { evidence_ref: String },
}

impl TriggerOrchestratorSdkError {
    pub fn primary_evidence_ref(&self) -> &str {
        match self {
            Self::InvalidConfig { evidence_ref }
            | Self::InvalidRequest { evidence_ref }
            | Self::MetadataMismatch { evidence_ref }
            | Self::RestRejected { evidence_ref }
            | Self::UnsafeMetadata { evidence_ref } => evidence_ref,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TriggerOrchestratorSdkClient {
    config: TriggerOrchestratorSdkConfig,
}

impl TriggerOrchestratorSdkClient {
    pub fn new(config: TriggerOrchestratorSdkConfig) -> Result<Self, TriggerOrchestratorSdkError> {
        validate_config(&config)?;
        Ok(Self { config })
    }

    pub fn plan_trigger(
        &self,
        context: TriggerOrchestratorSdkRequestContext,
        source: TriggerOrchestratorSdkSource,
        trigger: TriggerOrchestratorSdkTriggerDescriptor,
        metadata: TriggerOrchestratorSdkSourceMetadata,
    ) -> Result<TriggerOrchestratorSdkTriggerPlan, TriggerOrchestratorSdkError> {
        validate_context(&context)?;
        validate_trigger(&trigger)?;
        validate_metadata_for_source(source, &metadata)?;
        let api_request = self.api_request(context, source, trigger, metadata)?;
        Ok(TriggerOrchestratorSdkTriggerPlan {
            source,
            operation_id: source.operation_id().to_owned(),
            method: TRIGGER_ORCHESTRATOR_REST_METHOD,
            path: TRIGGER_ORCHESTRATOR_REST_ROUTE.to_owned(),
            contract_ref: TRIGGER_ORCHESTRATOR_SDK_CONTRACT_REF.to_owned(),
            oyatie_version: TRIGGER_ORCHESTRATOR_SDK_DECLARED_VERSION.to_owned(),
            retry_policy: TriggerOrchestratorSdkRetryPolicy::default(),
            rest_request: TriggerOrchestratorRestRequest {
                method: TRIGGER_ORCHESTRATOR_REST_METHOD,
                path: TRIGGER_ORCHESTRATOR_REST_ROUTE.to_owned(),
                request_id: api_request.boundary.request_id.clone(),
                body: api_request,
            },
            evidence_refs: sorted_unique(vec![
                TRIGGER_ORCHESTRATOR_SDK_SURFACE.to_owned(),
                source.operation_id().to_owned(),
                "workflow-trigger-sdk:plan-built".to_owned(),
            ]),
        })
    }

    pub fn execute_in_process(
        &self,
        rest: &mut TriggerOrchestratorRestService,
        plan: TriggerOrchestratorSdkTriggerPlan,
    ) -> Result<TriggerOrchestratorRestResponse, TriggerOrchestratorSdkError> {
        if plan.method != TRIGGER_ORCHESTRATOR_REST_METHOD
            || plan.path != TRIGGER_ORCHESTRATOR_REST_ROUTE
            || plan.rest_request.path != TRIGGER_ORCHESTRATOR_REST_ROUTE
            || plan.rest_request.method != TRIGGER_ORCHESTRATOR_REST_METHOD
        {
            return Err(TriggerOrchestratorSdkError::RestRejected {
                evidence_ref: "workflow-trigger-sdk:rest-route-drift".to_owned(),
            });
        }
        rest.handle(plan.rest_request)
            .map_err(|error| TriggerOrchestratorSdkError::RestRejected {
                evidence_ref: error.reason_ref,
            })
    }

    fn api_request(
        &self,
        context: TriggerOrchestratorSdkRequestContext,
        source: TriggerOrchestratorSdkSource,
        trigger: TriggerOrchestratorSdkTriggerDescriptor,
        metadata: TriggerOrchestratorSdkSourceMetadata,
    ) -> Result<TriggerOrchestratorApiRequest, TriggerOrchestratorSdkError> {
        let source_metadata = source_metadata(metadata);
        Ok(TriggerOrchestratorApiRequest {
            boundary: TriggerOrchestratorApiBoundaryContext {
                request_id: context.request_id,
                tenant_id: self.config.tenant_id.clone(),
                idempotency_key: context.idempotency_key,
                trace_context_ref: context
                    .trace_context_ref
                    .unwrap_or_else(|| self.config.trace_context_ref.clone()),
                oyatie_version: self.config.oyatie_version.clone(),
            },
            principal: TriggerOrchestratorApiPrincipal {
                tenant_id: self.config.tenant_id.clone(),
                principal_id: self.config.principal_id.clone(),
            },
            authorization: TriggerOrchestratorApiAuthorization {
                tenant_id: self.config.tenant_id.clone(),
                principal_id: self.config.principal_id.clone(),
                decision_id: self.config.authorization_decision_id.clone(),
                evidence_ref: self.config.authorization_evidence_ref.clone(),
                policy_bundle_ref: self.config.policy_bundle_ref.clone(),
                allowed_surfaces: vec![TRIGGER_ORCHESTRATOR_API_SURFACE.to_owned()],
            },
            method: TRIGGER_ORCHESTRATOR_API_METHOD.to_owned(),
            route: TRIGGER_ORCHESTRATOR_API_ROUTE.to_owned(),
            body: TriggerOrchestratorApiTriggerBody {
                source: source.source_wire().to_owned(),
                trigger_kind: source.trigger_kind_wire().to_owned(),
                trigger_id: trigger.trigger_id,
                workflow_spec_id: trigger
                    .workflow_spec_id
                    .unwrap_or_else(|| self.config.default_workflow_spec_id.clone()),
                version_sha: trigger
                    .version_sha
                    .unwrap_or_else(|| self.config.default_version_sha.clone()),
                active_cell_id: trigger
                    .active_cell_id
                    .unwrap_or_else(|| self.config.default_cell_id.clone()),
                trigger_lineage_ref: trigger.trigger_lineage_ref,
                run_idempotency_key: context.run_idempotency_key,
                authorization_surface_ref: self.config.authorization_surface_ref.clone(),
                source_evidence_ref: trigger.source_evidence_ref,
                scheduler_evidence_ref: source_metadata.scheduler_evidence_ref,
                webhook_auth_evidence_ref: source_metadata.webhook_auth_evidence_ref,
                event_contract_ref: source_metadata.event_contract_ref,
                replay_epoch_ref: trigger.replay_epoch_ref,
                audit_chain_ref: trigger.audit_chain_ref,
                correlation_ref: context.correlation_ref,
                idempotency_scope_ref: trigger.idempotency_scope_ref,
                dry_run_reason_ref: trigger.dry_run_reason_ref,
                replay_mode: trigger.replay_mode,
                dry_run: trigger.dry_run,
                schedule: source_metadata.schedule,
                webhook: source_metadata.webhook,
                event: source_metadata.event,
                evidence_refs: sorted_unique(
                    [
                        trigger.evidence_refs,
                        vec!["sdk-surface:trigger-orchestrator".to_owned()],
                    ]
                    .concat(),
                ),
            },
        })
    }
}

#[derive(Default)]
struct SourceMetadataParts {
    scheduler_evidence_ref: Option<String>,
    webhook_auth_evidence_ref: Option<String>,
    event_contract_ref: Option<String>,
    schedule: Option<TriggerOrchestratorApiScheduleDto>,
    webhook: Option<TriggerOrchestratorApiWebhookDto>,
    event: Option<TriggerOrchestratorApiEventDto>,
}

fn source_metadata(metadata: TriggerOrchestratorSdkSourceMetadata) -> SourceMetadataParts {
    match metadata {
        TriggerOrchestratorSdkSourceMetadata::Scheduler(schedule) => SourceMetadataParts {
            scheduler_evidence_ref: Some(schedule.scheduler_evidence_ref.clone()),
            schedule: Some(TriggerOrchestratorApiScheduleDto {
                cron_expr_ref: schedule.cron_expr_ref,
                timezone_ref: schedule.timezone_ref,
                due_epoch_seconds: schedule.due_epoch_seconds,
                observed_epoch_seconds: schedule.observed_epoch_seconds,
                catchup_window_seconds: schedule.catchup_window_seconds,
                overlap_policy: schedule.overlap_policy,
                paused: schedule.paused,
                pause_reason_ref: schedule.pause_reason_ref,
                last_fired_epoch_seconds: schedule.last_fired_epoch_seconds,
            }),
            ..SourceMetadataParts::default()
        },
        TriggerOrchestratorSdkSourceMetadata::StudioWebhook(webhook) => SourceMetadataParts {
            webhook_auth_evidence_ref: Some(webhook.webhook_auth_evidence_ref.clone()),
            webhook: Some(TriggerOrchestratorApiWebhookDto {
                endpoint_ref: webhook.endpoint_ref,
                signature_ref: webhook.signature_ref,
                nonce_ref: webhook.nonce_ref,
                hmac_key_ref: webhook.hmac_key_ref,
                received_epoch_seconds: webhook.received_epoch_seconds,
                expires_epoch_seconds: webhook.expires_epoch_seconds,
            }),
            ..SourceMetadataParts::default()
        },
        TriggerOrchestratorSdkSourceMetadata::SiblingEventBus(event) => SourceMetadataParts {
            event_contract_ref: Some(event.event_contract_ref.clone()),
            event: Some(TriggerOrchestratorApiEventDto {
                event_id: event.event_id,
                source: event.source,
                event_type: event.event_type,
                specversion: event.specversion,
                subject_ref: event.subject_ref,
                event_time_ref: event.event_time_ref,
                correlation_id: event.correlation_id,
                idempotency_key: event.idempotency_key,
            }),
            ..SourceMetadataParts::default()
        },
        TriggerOrchestratorSdkSourceMetadata::MetadataOnly => SourceMetadataParts::default(),
    }
}

fn validate_config(
    config: &TriggerOrchestratorSdkConfig,
) -> Result<(), TriggerOrchestratorSdkError> {
    if !is_safe_tenant(&config.tenant_id) {
        return Err(TriggerOrchestratorSdkError::InvalidConfig {
            evidence_ref: "workflow-trigger-sdk:tenant-invalid".to_owned(),
        });
    }
    let refs = [
        &config.principal_id,
        &config.authorization_decision_id,
        &config.authorization_evidence_ref,
        &config.policy_bundle_ref,
        &config.default_workflow_spec_id,
        &config.default_version_sha,
        &config.default_cell_id,
        &config.authorization_surface_ref,
        &config.trace_context_ref,
    ];
    if !refs.iter().all(|value| is_safe_ref(value)) {
        return Err(TriggerOrchestratorSdkError::UnsafeMetadata {
            evidence_ref: "workflow-trigger-sdk:config-ref-invalid".to_owned(),
        });
    }
    if config.oyatie_version != TRIGGER_ORCHESTRATOR_SDK_DECLARED_VERSION {
        return Err(TriggerOrchestratorSdkError::InvalidConfig {
            evidence_ref: "workflow-trigger-sdk:version-unsupported".to_owned(),
        });
    }
    Ok(())
}

fn validate_context(
    context: &TriggerOrchestratorSdkRequestContext,
) -> Result<(), TriggerOrchestratorSdkError> {
    if !is_safe_ref(&context.request_id)
        || !is_safe_ref(&context.idempotency_key)
        || !is_safe_ref(&context.run_idempotency_key)
        || !is_safe_ref(&context.correlation_ref)
        || !is_safe_optional_ref(context.trace_context_ref.as_deref())
    {
        return Err(TriggerOrchestratorSdkError::InvalidRequest {
            evidence_ref: "workflow-trigger-sdk:context-ref-invalid".to_owned(),
        });
    }
    Ok(())
}

fn validate_trigger(
    trigger: &TriggerOrchestratorSdkTriggerDescriptor,
) -> Result<(), TriggerOrchestratorSdkError> {
    if !is_safe_ref(&trigger.trigger_id)
        || !is_safe_optional_ref(trigger.workflow_spec_id.as_deref())
        || !is_safe_optional_ref(trigger.version_sha.as_deref())
        || !is_safe_optional_ref(trigger.active_cell_id.as_deref())
        || !is_safe_ref(&trigger.trigger_lineage_ref)
        || !is_safe_ref(&trigger.source_evidence_ref)
        || !is_safe_ref(&trigger.replay_epoch_ref)
        || !is_safe_ref(&trigger.audit_chain_ref)
        || !is_safe_ref(&trigger.idempotency_scope_ref)
        || !is_safe_optional_ref(trigger.dry_run_reason_ref.as_deref())
        || !trigger.evidence_refs.iter().all(|value| is_safe_ref(value))
    {
        return Err(TriggerOrchestratorSdkError::UnsafeMetadata {
            evidence_ref: "workflow-trigger-sdk:trigger-ref-invalid".to_owned(),
        });
    }
    Ok(())
}

fn validate_metadata_for_source(
    source: TriggerOrchestratorSdkSource,
    metadata: &TriggerOrchestratorSdkSourceMetadata,
) -> Result<(), TriggerOrchestratorSdkError> {
    match (source, metadata) {
        (
            TriggerOrchestratorSdkSource::Scheduler,
            TriggerOrchestratorSdkSourceMetadata::Scheduler(schedule),
        ) => validate_schedule(schedule),
        (
            TriggerOrchestratorSdkSource::StudioWebhook,
            TriggerOrchestratorSdkSourceMetadata::StudioWebhook(webhook),
        ) => validate_webhook(webhook),
        (
            TriggerOrchestratorSdkSource::SiblingEventBus,
            TriggerOrchestratorSdkSourceMetadata::SiblingEventBus(event),
        ) => validate_event(event),
        (
            TriggerOrchestratorSdkSource::ManualUi
            | TriggerOrchestratorSdkSource::ApiCommand
            | TriggerOrchestratorSdkSource::WorkflowSpawn
            | TriggerOrchestratorSdkSource::OntologyProjection,
            TriggerOrchestratorSdkSourceMetadata::MetadataOnly,
        ) => Ok(()),
        _ => Err(TriggerOrchestratorSdkError::MetadataMismatch {
            evidence_ref: "workflow-trigger-sdk:source-metadata-mismatch".to_owned(),
        }),
    }
}

fn validate_schedule(
    schedule: &TriggerOrchestratorSdkScheduleDescriptor,
) -> Result<(), TriggerOrchestratorSdkError> {
    if !is_safe_ref(&schedule.cron_expr_ref)
        || !is_safe_ref(&schedule.timezone_ref)
        || !is_safe_metadata(&schedule.overlap_policy)
        || !is_safe_optional_ref(schedule.pause_reason_ref.as_deref())
        || !is_safe_ref(&schedule.scheduler_evidence_ref)
    {
        return Err(TriggerOrchestratorSdkError::UnsafeMetadata {
            evidence_ref: "workflow-trigger-sdk:schedule-ref-invalid".to_owned(),
        });
    }
    Ok(())
}

fn validate_webhook(
    webhook: &TriggerOrchestratorSdkWebhookDescriptor,
) -> Result<(), TriggerOrchestratorSdkError> {
    if !is_safe_ref(&webhook.endpoint_ref)
        || !is_safe_ref(&webhook.signature_ref)
        || !is_safe_ref(&webhook.nonce_ref)
        || !is_safe_ref(&webhook.hmac_key_ref)
        || !is_safe_ref(&webhook.webhook_auth_evidence_ref)
    {
        return Err(TriggerOrchestratorSdkError::UnsafeMetadata {
            evidence_ref: "workflow-trigger-sdk:webhook-ref-invalid".to_owned(),
        });
    }
    Ok(())
}

fn validate_event(
    event: &TriggerOrchestratorSdkEventDescriptor,
) -> Result<(), TriggerOrchestratorSdkError> {
    if !is_safe_ref(&event.event_id)
        || !is_safe_ref(&event.source)
        || !is_safe_metadata(&event.event_type)
        || !is_safe_metadata(&event.specversion)
        || !is_safe_optional_ref(event.subject_ref.as_deref())
        || !is_safe_optional_ref(event.event_time_ref.as_deref())
        || !is_safe_ref(&event.correlation_id)
        || !is_safe_ref(&event.idempotency_key)
        || !is_safe_ref(&event.event_contract_ref)
    {
        return Err(TriggerOrchestratorSdkError::UnsafeMetadata {
            evidence_ref: "workflow-trigger-sdk:event-ref-invalid".to_owned(),
        });
    }
    Ok(())
}

fn is_safe_tenant(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && value == trimmed
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
        && !contains_unsafe_debug_material(value)
}

fn is_safe_ref(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && value == trimmed
        && value.contains(':')
        && !value.chars().any(char::is_whitespace)
        && !contains_unsafe_debug_material(value)
}

fn is_safe_optional_ref(value: Option<&str>) -> bool {
    value.is_none_or(is_safe_ref)
}

fn is_safe_metadata(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && value == trimmed
        && !value.chars().any(char::is_whitespace)
        && !contains_unsafe_debug_material(value)
}

fn contains_unsafe_debug_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("raw prompt")
        || lower.contains("raw output")
        || lower.contains("raw model")
        || lower.contains("payload")
        || lower.contains("write an email")
        || lower.contains("customer message")
        || lower.contains("model answer")
        || lower.contains("sk-")
        || lower.contains("bearer")
        || lower.contains("authorization:")
        || lower.contains("api_key=")
        || lower.contains("openai_api_key")
        || lower.contains("secret=")
        || lower.contains("private key")
        || lower.contains("-----begin")
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.retain(|value| !value.trim().is_empty() && !contains_unsafe_debug_material(value));
    values.sort();
    values.dedup();
    values
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use workflow_trigger_orchestrator_rest::{
        TriggerOrchestratorRestResponseBody, WorkflowTriggerOrchestratorApi,
    };

    #[test]
    fn sdk_constants_defaults_and_source_labels_are_contract_bound() {
        assert_eq!(
            TRIGGER_ORCHESTRATOR_SDK_CONTRACT_REF,
            TRIGGER_ORCHESTRATOR_REST_CONTRACT_REF
        );
        let retry_policy = TriggerOrchestratorSdkRetryPolicy::default();
        assert!(!retry_policy.automatic_retries_enabled);
        assert_eq!(
            retry_policy.retry_policy_ref,
            TRIGGER_ORCHESTRATOR_SDK_RETRY_POLICY_REF
        );
        let sources = [
            TriggerOrchestratorSdkSource::Scheduler,
            TriggerOrchestratorSdkSource::StudioWebhook,
            TriggerOrchestratorSdkSource::SiblingEventBus,
            TriggerOrchestratorSdkSource::ManualUi,
            TriggerOrchestratorSdkSource::ApiCommand,
            TriggerOrchestratorSdkSource::WorkflowSpawn,
            TriggerOrchestratorSdkSource::OntologyProjection,
        ];
        let source_labels: BTreeSet<_> =
            sources.iter().map(|source| source.source_wire()).collect();
        let kind_labels: BTreeSet<_> = sources
            .iter()
            .map(|source| source.trigger_kind_wire())
            .collect();
        assert_eq!(source_labels.len(), sources.len());
        assert_eq!(kind_labels.len(), sources.len());
    }

    #[test]
    fn scheduler_plan_binds_route_version_authorization_idempotency_and_metadata() {
        let client = valid_client();
        let plan = client
            .plan_trigger(
                context("idem:sdk:scheduler"),
                TriggerOrchestratorSdkSource::Scheduler,
                trigger_descriptor("trigger:daily-invoice"),
                TriggerOrchestratorSdkSourceMetadata::Scheduler(schedule_descriptor()),
            )
            .expect("scheduler plan");

        assert_eq!(plan.method, TriggerOrchestratorRestMethod::Post);
        assert_eq!(plan.path, TRIGGER_ORCHESTRATOR_REST_ROUTE);
        assert_eq!(plan.operation_id, "evaluateSchedulerTrigger");
        assert_eq!(
            plan.rest_request.body.boundary.oyatie_version,
            TRIGGER_ORCHESTRATOR_SDK_DECLARED_VERSION
        );
        assert_eq!(
            plan.rest_request.body.authorization.allowed_surfaces,
            vec![TRIGGER_ORCHESTRATOR_API_SURFACE.to_owned()]
        );
        assert_eq!(plan.rest_request.body.body.source, "scheduler");
        assert_eq!(plan.rest_request.body.body.trigger_kind, "cron");
        assert!(plan.rest_request.body.body.schedule.is_some());
        assert_eq!(
            plan.rest_request
                .body
                .body
                .scheduler_evidence_ref
                .as_deref(),
            Some("scheduler:durable-clock-window")
        );
        assert!(!plan.retry_policy.automatic_retries_enabled);
    }

    #[test]
    fn webhook_event_manual_workflow_and_ontology_plans_are_source_specific_without_payload() {
        let client = valid_client();
        let webhook = client
            .plan_trigger(
                context("idem:sdk:webhook"),
                TriggerOrchestratorSdkSource::StudioWebhook,
                trigger_descriptor("trigger:webhook-invoice"),
                TriggerOrchestratorSdkSourceMetadata::StudioWebhook(webhook_descriptor()),
            )
            .expect("webhook plan");
        assert_eq!(webhook.rest_request.body.body.source, "studio-webhook");
        assert!(webhook.rest_request.body.body.webhook.is_some());
        assert_eq!(
            webhook
                .rest_request
                .body
                .body
                .webhook_auth_evidence_ref
                .as_deref(),
            Some("webhook-auth:hmac-nonce-bound")
        );

        let event = client
            .plan_trigger(
                context("idem:sdk:event"),
                TriggerOrchestratorSdkSource::SiblingEventBus,
                trigger_descriptor("trigger:event-invoice"),
                TriggerOrchestratorSdkSourceMetadata::SiblingEventBus(event_descriptor()),
            )
            .expect("event plan");
        assert_eq!(event.rest_request.body.body.trigger_kind, "event-bus");
        assert!(event.rest_request.body.body.event.is_some());
        assert_eq!(
            event.rest_request.body.body.event_contract_ref.as_deref(),
            Some("event-contract:cloudevents-v1")
        );

        for source in [
            TriggerOrchestratorSdkSource::ManualUi,
            TriggerOrchestratorSdkSource::ApiCommand,
            TriggerOrchestratorSdkSource::WorkflowSpawn,
            TriggerOrchestratorSdkSource::OntologyProjection,
        ] {
            let plan = client
                .plan_trigger(
                    context(&format!("idem:sdk:{}", source.operation_id())),
                    source,
                    trigger_descriptor("trigger:metadata-only"),
                    TriggerOrchestratorSdkSourceMetadata::MetadataOnly,
                )
                .expect("metadata-only plan");
            assert_eq!(plan.rest_request.body.body.source, source.source_wire());
            assert!(plan.rest_request.body.body.schedule.is_none());
            assert!(plan.rest_request.body.body.webhook.is_none());
            assert!(plan.rest_request.body.body.event.is_none());
        }
    }

    #[test]
    fn invalid_config_or_request_denies_before_rest_without_raw_echo() {
        let mut config = valid_config();
        config.principal_id = "principal:raw prompt bearer sk-test payload".to_owned();
        let error = TriggerOrchestratorSdkClient::new(config).expect_err("invalid config");
        assert_eq!(
            error.primary_evidence_ref(),
            "workflow-trigger-sdk:config-ref-invalid"
        );
        assert!(!format!("{error:?}").contains("sk-test"));

        let client = valid_client();
        let mut ctx = context("idem:sdk:unsafe");
        ctx.correlation_ref = "corr:raw prompt bearer sk-test payload".to_owned();
        let request_error = client
            .plan_trigger(
                ctx,
                TriggerOrchestratorSdkSource::Scheduler,
                trigger_descriptor("trigger:daily-invoice"),
                TriggerOrchestratorSdkSourceMetadata::Scheduler(schedule_descriptor()),
            )
            .expect_err("invalid request");
        assert_eq!(
            request_error.primary_evidence_ref(),
            "workflow-trigger-sdk:context-ref-invalid"
        );
        assert!(!format!("{request_error:?}").contains("payload"));
    }

    #[test]
    fn source_metadata_mismatch_denies_before_rest() {
        let client = valid_client();
        let error = client
            .plan_trigger(
                context("idem:sdk:mismatch"),
                TriggerOrchestratorSdkSource::Scheduler,
                trigger_descriptor("trigger:daily-invoice"),
                TriggerOrchestratorSdkSourceMetadata::MetadataOnly,
            )
            .expect_err("mismatch");
        assert_eq!(
            error.primary_evidence_ref(),
            "workflow-trigger-sdk:source-metadata-mismatch"
        );
    }

    #[test]
    fn in_process_preview_execute_delegates_through_rest_api_without_http_client() {
        let client = valid_client();
        let plan = client
            .plan_trigger(
                context("idem:sdk:execute"),
                TriggerOrchestratorSdkSource::Scheduler,
                trigger_descriptor("trigger:daily-invoice"),
                TriggerOrchestratorSdkSourceMetadata::Scheduler(schedule_descriptor()),
            )
            .expect("plan");
        let mut rest =
            TriggerOrchestratorRestService::new(WorkflowTriggerOrchestratorApi::default());

        let response = client
            .execute_in_process(&mut rest, plan)
            .expect("response");

        assert_eq!(response.status_code, 202);
        assert_eq!(rest.api_delegation_count(), 1);
        let TriggerOrchestratorRestResponseBody::Success(success) = response.body else {
            panic!("expected success");
        };
        assert_eq!(success.trigger.usecase_status, "accepted");
        assert!(success.trigger.dispatch_required);
        assert!(!format!("{success:?}").contains("payload"));
    }

    #[test]
    fn idempotent_replay_keeps_stable_request_identity_without_second_side_effect_shape_change() {
        let client = valid_client();
        let plan = client
            .plan_trigger(
                context("idem:sdk:replay"),
                TriggerOrchestratorSdkSource::Scheduler,
                trigger_descriptor("trigger:daily-invoice"),
                TriggerOrchestratorSdkSourceMetadata::Scheduler(schedule_descriptor()),
            )
            .expect("plan");
        let mut rest = TriggerOrchestratorRestService::default();

        let first = client
            .execute_in_process(&mut rest, plan.clone())
            .expect("first");
        let second = client.execute_in_process(&mut rest, plan).expect("second");

        assert_eq!(first, second);
        assert_eq!(rest.api_delegation_count(), 2);
    }

    fn valid_client() -> TriggerOrchestratorSdkClient {
        TriggerOrchestratorSdkClient::new(valid_config()).expect("client")
    }

    fn valid_config() -> TriggerOrchestratorSdkConfig {
        TriggerOrchestratorSdkConfig {
            tenant_id: "ten_foundry".to_owned(),
            principal_id: "principal:workflow-operator".to_owned(),
            authorization_decision_id: "policy-decision:allow-trigger".to_owned(),
            authorization_evidence_ref: "policy-evidence:cedar-allow".to_owned(),
            policy_bundle_ref: "policy-bundle:trigger-v1".to_owned(),
            default_workflow_spec_id: "workflow:invoice-approval".to_owned(),
            default_version_sha: "sha:abc123".to_owned(),
            default_cell_id: "cell:use1-a".to_owned(),
            authorization_surface_ref: "authz-surface:trigger-admission".to_owned(),
            trace_context_ref: "trace:trigger-sdk".to_owned(),
            oyatie_version: TRIGGER_ORCHESTRATOR_SDK_DECLARED_VERSION.to_owned(),
        }
    }

    fn context(idempotency_key: &str) -> TriggerOrchestratorSdkRequestContext {
        TriggerOrchestratorSdkRequestContext {
            request_id: format!("request:trigger-sdk:{idempotency_key}"),
            idempotency_key: idempotency_key.to_owned(),
            trace_context_ref: None,
            run_idempotency_key: format!("idem:trigger-run:{idempotency_key}"),
            correlation_ref: format!("corr:trigger-sdk:{idempotency_key}"),
        }
    }

    fn trigger_descriptor(trigger_id: &str) -> TriggerOrchestratorSdkTriggerDescriptor {
        TriggerOrchestratorSdkTriggerDescriptor {
            trigger_id: trigger_id.to_owned(),
            workflow_spec_id: None,
            version_sha: None,
            active_cell_id: None,
            trigger_lineage_ref: "lineage:trigger-parent".to_owned(),
            source_evidence_ref: "source-evidence:trigger-admission".to_owned(),
            replay_epoch_ref: "replay-epoch:2026-05-25T000000Z".to_owned(),
            audit_chain_ref: "audit-chain:trigger-sdk".to_owned(),
            idempotency_scope_ref: "idem-scope:tenant-trigger".to_owned(),
            dry_run_reason_ref: None,
            replay_mode: false,
            dry_run: false,
            evidence_refs: vec!["evidence:sdk-unit-test".to_owned()],
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

    fn webhook_descriptor() -> TriggerOrchestratorSdkWebhookDescriptor {
        TriggerOrchestratorSdkWebhookDescriptor {
            endpoint_ref: "endpoint:webhook-invoice".to_owned(),
            signature_ref: "signature:webhook-headers".to_owned(),
            nonce_ref: "nonce:webhook-001".to_owned(),
            hmac_key_ref: "hmac-key:webhook-signing".to_owned(),
            received_epoch_seconds: 1_750_000_001,
            expires_epoch_seconds: 1_750_000_061,
            webhook_auth_evidence_ref: "webhook-auth:hmac-nonce-bound".to_owned(),
        }
    }

    fn event_descriptor() -> TriggerOrchestratorSdkEventDescriptor {
        TriggerOrchestratorSdkEventDescriptor {
            event_id: "event:invoice-approved-001".to_owned(),
            source: "https://events.oyatie.example/workflow".to_owned(),
            event_type: "com.oyatie.workflow.invoice_approved".to_owned(),
            specversion: TRIGGER_ORCHESTRATOR_CLOUDEVENTS_SPECVERSION.to_owned(),
            subject_ref: Some("subject:invoice-123".to_owned()),
            event_time_ref: Some("time:2026-05-25T00:00:00Z".to_owned()),
            correlation_id: "corr:invoice-123".to_owned(),
            idempotency_key: "idem:event-001".to_owned(),
            event_contract_ref: "event-contract:cloudevents-v1".to_owned(),
        }
    }
}
