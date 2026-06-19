//! Workflow-engine event-bus SDK foundation.
//!
//! This crate provides a source-level, language-native SDK facade over the
//! workflow event-bus REST/API boundary for future generated SDK work. It builds
//! typed publish and delivery-evaluation plans, binds version, tenant,
//! principal, authorization, trace, idempotency, route, CloudEvents-shaped event
//! refs, AsyncAPI-shaped channel refs, subscription/candidate metadata, and
//! safe evidence refs; exposes an in-process preview execution seam for local
//! tests; and rejects raw prompt/output/payload/secret-shaped material before
//! delegating to REST/API/usecase code. It performs no generated SDK
//! publication, HTTP client work, DNS, sockets, serialization-framework work,
//! credential loading, random/UUID generation, wall-clock reads, automatic
//! retries, durable idempotency, queueing, signing, filesystem access,
//! Kubernetes calls, cloud deployment, or tenant workload scheduling.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use workflow_event_bus_rest::{
    WORKFLOW_EVENT_BUS_API_CONTRACT_REF, WORKFLOW_EVENT_BUS_API_DECLARED_VERSION,
    WORKFLOW_EVENT_BUS_API_DELIVERY_ROUTE, WORKFLOW_EVENT_BUS_API_METHOD,
    WORKFLOW_EVENT_BUS_API_PUBLISH_ROUTE, WORKFLOW_EVENT_BUS_API_SURFACE,
    WORKFLOW_EVENT_BUS_ASYNCAPI_CONTRACT_REF, WORKFLOW_EVENT_BUS_CLOUDEVENTS_SPECVERSION,
    WORKFLOW_EVENT_BUS_REST_CONTRACT_REF, WORKFLOW_EVENT_BUS_REST_DELIVERY_ROUTE,
    WORKFLOW_EVENT_BUS_REST_METHOD, WORKFLOW_EVENT_BUS_REST_PROBLEM_CONTENT_TYPE,
    WORKFLOW_EVENT_BUS_REST_PUBLISH_ROUTE, WORKFLOW_EVENT_BUS_REST_SUCCESS_CONTENT_TYPE,
    WorkflowEventBusApi, WorkflowEventBusApiAuthorization, WorkflowEventBusApiBoundaryContext,
    WorkflowEventBusApiDeliveryBody, WorkflowEventBusApiDeliveryRequest,
    WorkflowEventBusApiErrorCode, WorkflowEventBusApiEventDto, WorkflowEventBusApiPrincipal,
    WorkflowEventBusApiProblemDetails, WorkflowEventBusApiPublishBody,
    WorkflowEventBusApiPublishRequest, WorkflowEventBusApiResponseMetadata,
    WorkflowEventBusApiStatus, WorkflowEventBusApiSuccessResponse, WorkflowEventBusChannel,
    WorkflowEventBusDeliveryDecision, WorkflowEventBusDeliveryStatus, WorkflowEventBusEventKind,
    WorkflowEventBusRestBodyKind, WorkflowEventBusRestError, WorkflowEventBusRestMethod,
    WorkflowEventBusRestOperation, WorkflowEventBusRestRequest, WorkflowEventBusRestRequestBody,
    WorkflowEventBusRestResponse, WorkflowEventBusRestResponseBody, WorkflowEventBusRestService,
};

pub const WORKFLOW_EVENT_BUS_SDK_SURFACE: &str = "workflow-engine.event-bus.sdk";
pub const WORKFLOW_EVENT_BUS_SDK_CONTRACT_REF: &str = WORKFLOW_EVENT_BUS_REST_CONTRACT_REF;
pub const WORKFLOW_EVENT_BUS_SDK_DECLARED_VERSION: &str = WORKFLOW_EVENT_BUS_API_DECLARED_VERSION;
pub const WORKFLOW_EVENT_BUS_SDK_AUTOMATIC_RETRIES_ENABLED: bool = false;
pub const WORKFLOW_EVENT_BUS_SDK_RETRY_POLICY_REF: &str =
    "workflow-event-bus-sdk:automatic-retry-disabled";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkflowEventBusSdkOperation {
    Publish,
    EvaluateDelivery,
}

impl WorkflowEventBusSdkOperation {
    pub const fn operation_id(self) -> &'static str {
        match self {
            Self::Publish => "publishEventBusEvent",
            Self::EvaluateDelivery => "evaluateEventBusDelivery",
        }
    }

    pub const fn rest_operation(self) -> WorkflowEventBusRestOperation {
        match self {
            Self::Publish => WorkflowEventBusRestOperation::Publish,
            Self::EvaluateDelivery => WorkflowEventBusRestOperation::EvaluateDelivery,
        }
    }

    pub const fn body_kind(self) -> WorkflowEventBusRestBodyKind {
        match self {
            Self::Publish => WorkflowEventBusRestBodyKind::Publish,
            Self::EvaluateDelivery => WorkflowEventBusRestBodyKind::Delivery,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusSdkConfig {
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub principal_id: String,               // data_class: INTERNAL_ONLY
    pub authorization_decision_id: String,  // data_class: INTERNAL_ONLY
    pub authorization_evidence_ref: String, // data_class: INTERNAL_ONLY
    pub policy_bundle_ref: String,          // data_class: INTERNAL_ONLY
    pub default_cell_id: String,            // data_class: INTERNAL_ONLY
    pub default_residency_ref: String,      // data_class: INTERNAL_ONLY
    pub default_audit_chain_ref: String,    // data_class: INTERNAL_ONLY
    pub trace_context_ref: String,          // data_class: INTERNAL_ONLY
    pub allowed_channels: Vec<String>,      // data_class: INTERNAL_ONLY
    pub allowed_event_types: Vec<String>,   // data_class: INTERNAL_ONLY
    pub oyatie_version: String,             // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusSdkRequestContext {
    pub request_id: String,                // data_class: INTERNAL_ONLY
    pub idempotency_key: String,           // data_class: INTERNAL_ONLY
    pub trace_context_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub correlation_ref: String,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusSdkPublishDescriptor {
    pub event_kind: String,              // data_class: PUBLIC
    pub event_id: String,                // data_class: INTERNAL_ONLY
    pub producer_ref: String,            // data_class: INTERNAL_ONLY
    pub source_ref: String,              // data_class: INTERNAL_ONLY
    pub subject_ref: Option<String>,     // data_class: INTERNAL_ONLY
    pub time_ref: Option<String>,        // data_class: INTERNAL_ONLY
    pub dataschema_ref: Option<String>,  // data_class: INTERNAL_ONLY
    pub partition_key_ref: String,       // data_class: INTERNAL_ONLY
    pub publish_idempotency_key: String, // data_class: INTERNAL_ONLY
    pub causation_ref: String,           // data_class: INTERNAL_ONLY
    pub correlation_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub payload_ref: String,             // data_class: INTERNAL_ONLY
    pub cell_id: Option<String>,         // data_class: INTERNAL_ONLY
    pub residency_ref: Option<String>,   // data_class: INTERNAL_ONLY
    pub audit_chain_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusSdkDeliveryDescriptor {
    pub subscription_channel: String,          // data_class: PUBLIC
    pub consumer_ref: String,                  // data_class: INTERNAL_ONLY
    pub subscription_event_types: Vec<String>, // data_class: INTERNAL_ONLY
    pub replay_cursor_ref: Option<String>,     // data_class: INTERNAL_ONLY
    pub max_batch_size: u32,                   // data_class: INTERNAL_ONLY
    pub subscription_authorization_evidence_ref: String, // data_class: INTERNAL_ONLY
    pub candidate_channel: String,             // data_class: PUBLIC
    pub candidate_event_id: String,            // data_class: INTERNAL_ONLY
    pub candidate_event_type: String,          // data_class: PUBLIC
    pub candidate_idempotency_key: String,     // data_class: INTERNAL_ONLY
    pub candidate_payload_ref: String,         // data_class: INTERNAL_ONLY
    pub candidate_offset_ref: String,          // data_class: INTERNAL_ONLY
    pub cell_id: Option<String>,               // data_class: INTERNAL_ONLY
    pub residency_ref: Option<String>,         // data_class: INTERNAL_ONLY
    pub audit_chain_ref: Option<String>,       // data_class: INTERNAL_ONLY
    pub candidate_evidence_refs: Vec<String>,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusSdkRetryPolicy {
    pub automatic_retries_enabled: bool, // data_class: PUBLIC
    pub retry_policy_ref: String,        // data_class: INTERNAL_ONLY
}

impl Default for WorkflowEventBusSdkRetryPolicy {
    fn default() -> Self {
        Self {
            automatic_retries_enabled: WORKFLOW_EVENT_BUS_SDK_AUTOMATIC_RETRIES_ENABLED,
            retry_policy_ref: WORKFLOW_EVENT_BUS_SDK_RETRY_POLICY_REF.to_owned(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusSdkCommandPlan {
    pub operation: WorkflowEventBusSdkOperation, // data_class: PUBLIC
    pub operation_id: String,                    // data_class: PUBLIC
    pub method: WorkflowEventBusRestMethod,      // data_class: PUBLIC
    pub path: String,                            // data_class: PUBLIC
    pub body_kind: WorkflowEventBusRestBodyKind, // data_class: PUBLIC
    pub contract_ref: String,                    // data_class: INTERNAL_ONLY
    pub api_version: String,                     // data_class: PUBLIC
    pub retry_policy: WorkflowEventBusSdkRetryPolicy, // data_class: INTERNAL_ONLY
    pub rest_request: WorkflowEventBusRestRequest, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,              // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkflowEventBusSdkError {
    InvalidConfig { evidence_ref: String },
    InvalidRequest { evidence_ref: String },
    MetadataMismatch { evidence_ref: String },
    RestRejected { evidence_ref: String },
    UnsafeMetadata { evidence_ref: String },
}

impl WorkflowEventBusSdkError {
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
pub struct WorkflowEventBusSdk {
    config: WorkflowEventBusSdkConfig,
}

impl WorkflowEventBusSdk {
    pub fn new(config: WorkflowEventBusSdkConfig) -> Result<Self, WorkflowEventBusSdkError> {
        validate_config(&config)?;
        Ok(Self { config })
    }

    pub fn plan_publish(
        &self,
        context: WorkflowEventBusSdkRequestContext,
        descriptor: WorkflowEventBusSdkPublishDescriptor,
    ) -> Result<WorkflowEventBusSdkCommandPlan, WorkflowEventBusSdkError> {
        validate_context(&context)?;
        validate_publish_descriptor(&self.config, &descriptor)?;
        let api_request = self.publish_api_request(context, descriptor)?;
        Ok(WorkflowEventBusSdkCommandPlan {
            operation: WorkflowEventBusSdkOperation::Publish,
            operation_id: WorkflowEventBusSdkOperation::Publish
                .operation_id()
                .to_owned(),
            method: WORKFLOW_EVENT_BUS_REST_METHOD,
            path: WORKFLOW_EVENT_BUS_REST_PUBLISH_ROUTE.to_owned(),
            body_kind: WorkflowEventBusRestBodyKind::Publish,
            contract_ref: WORKFLOW_EVENT_BUS_SDK_CONTRACT_REF.to_owned(),
            api_version: WORKFLOW_EVENT_BUS_SDK_DECLARED_VERSION.to_owned(),
            retry_policy: WorkflowEventBusSdkRetryPolicy::default(),
            rest_request: WorkflowEventBusRestRequest {
                method: WORKFLOW_EVENT_BUS_REST_METHOD,
                path: WORKFLOW_EVENT_BUS_REST_PUBLISH_ROUTE.to_owned(),
                request_id: api_request.boundary.request_id.clone(),
                body: WorkflowEventBusRestRequestBody::Publish(Box::new(api_request)),
            },
            evidence_refs: sorted_unique(vec![
                WORKFLOW_EVENT_BUS_SDK_SURFACE.to_owned(),
                WORKFLOW_EVENT_BUS_API_SURFACE.to_owned(),
                WORKFLOW_EVENT_BUS_SDK_CONTRACT_REF.to_owned(),
                "workflow-event-bus-sdk:publish-plan".to_owned(),
            ]),
        })
    }

    pub fn plan_delivery(
        &self,
        context: WorkflowEventBusSdkRequestContext,
        descriptor: WorkflowEventBusSdkDeliveryDescriptor,
    ) -> Result<WorkflowEventBusSdkCommandPlan, WorkflowEventBusSdkError> {
        validate_context(&context)?;
        validate_delivery_descriptor(&self.config, &descriptor)?;
        let api_request = self.delivery_api_request(context, descriptor)?;
        Ok(WorkflowEventBusSdkCommandPlan {
            operation: WorkflowEventBusSdkOperation::EvaluateDelivery,
            operation_id: WorkflowEventBusSdkOperation::EvaluateDelivery
                .operation_id()
                .to_owned(),
            method: WORKFLOW_EVENT_BUS_REST_METHOD,
            path: WORKFLOW_EVENT_BUS_REST_DELIVERY_ROUTE.to_owned(),
            body_kind: WorkflowEventBusRestBodyKind::Delivery,
            contract_ref: WORKFLOW_EVENT_BUS_SDK_CONTRACT_REF.to_owned(),
            api_version: WORKFLOW_EVENT_BUS_SDK_DECLARED_VERSION.to_owned(),
            retry_policy: WorkflowEventBusSdkRetryPolicy::default(),
            rest_request: WorkflowEventBusRestRequest {
                method: WORKFLOW_EVENT_BUS_REST_METHOD,
                path: WORKFLOW_EVENT_BUS_REST_DELIVERY_ROUTE.to_owned(),
                request_id: api_request.boundary.request_id.clone(),
                body: WorkflowEventBusRestRequestBody::Delivery(Box::new(api_request)),
            },
            evidence_refs: sorted_unique(vec![
                WORKFLOW_EVENT_BUS_SDK_SURFACE.to_owned(),
                WORKFLOW_EVENT_BUS_API_SURFACE.to_owned(),
                WORKFLOW_EVENT_BUS_SDK_CONTRACT_REF.to_owned(),
                "workflow-event-bus-sdk:delivery-plan".to_owned(),
                "workflow-event-bus-sdk:no-offset-commit-claim".to_owned(),
            ]),
        })
    }

    pub fn execute_in_process(
        &self,
        plan: WorkflowEventBusSdkCommandPlan,
        rest: &mut WorkflowEventBusRestService,
    ) -> Result<WorkflowEventBusRestResponse, WorkflowEventBusSdkError> {
        validate_plan(&plan)?;
        rest.handle(plan.rest_request)
            .map_err(|_| WorkflowEventBusSdkError::RestRejected {
                evidence_ref: "workflow-event-bus-sdk:rest-rejected".to_owned(),
            })
    }

    fn publish_api_request(
        &self,
        context: WorkflowEventBusSdkRequestContext,
        descriptor: WorkflowEventBusSdkPublishDescriptor,
    ) -> Result<WorkflowEventBusApiPublishRequest, WorkflowEventBusSdkError> {
        Ok(WorkflowEventBusApiPublishRequest {
            boundary: self.boundary(&context),
            principal: self.principal(),
            authorization: self.authorization(),
            method: WORKFLOW_EVENT_BUS_API_METHOD.to_owned(),
            route: WORKFLOW_EVENT_BUS_API_PUBLISH_ROUTE.to_owned(),
            body: WorkflowEventBusApiPublishBody {
                cell_id: descriptor
                    .cell_id
                    .unwrap_or_else(|| self.config.default_cell_id.clone()),
                residency_ref: descriptor
                    .residency_ref
                    .unwrap_or_else(|| self.config.default_residency_ref.clone()),
                audit_chain_ref: descriptor
                    .audit_chain_ref
                    .unwrap_or_else(|| self.config.default_audit_chain_ref.clone()),
                event_kind: descriptor.event_kind,
                producer_ref: descriptor.producer_ref,
                event_id: descriptor.event_id,
                source_ref: descriptor.source_ref,
                subject_ref: descriptor.subject_ref,
                time_ref: descriptor.time_ref,
                dataschema_ref: descriptor.dataschema_ref,
                partition_key_ref: descriptor.partition_key_ref,
                publish_idempotency_key: descriptor.publish_idempotency_key,
                causation_ref: descriptor.causation_ref,
                correlation_ref: descriptor
                    .correlation_ref
                    .unwrap_or(context.correlation_ref),
                payload_ref: descriptor.payload_ref,
                evidence_refs: sorted_unique(
                    [
                        descriptor.evidence_refs,
                        vec!["workflow-event-bus-sdk:publish-descriptor".to_owned()],
                    ]
                    .concat(),
                ),
            },
        })
    }

    fn delivery_api_request(
        &self,
        context: WorkflowEventBusSdkRequestContext,
        descriptor: WorkflowEventBusSdkDeliveryDescriptor,
    ) -> Result<WorkflowEventBusApiDeliveryRequest, WorkflowEventBusSdkError> {
        Ok(WorkflowEventBusApiDeliveryRequest {
            boundary: self.boundary(&context),
            principal: self.principal(),
            authorization: self.authorization(),
            method: WORKFLOW_EVENT_BUS_API_METHOD.to_owned(),
            route: WORKFLOW_EVENT_BUS_API_DELIVERY_ROUTE.to_owned(),
            body: WorkflowEventBusApiDeliveryBody {
                cell_id: descriptor
                    .cell_id
                    .unwrap_or_else(|| self.config.default_cell_id.clone()),
                residency_ref: descriptor
                    .residency_ref
                    .unwrap_or_else(|| self.config.default_residency_ref.clone()),
                audit_chain_ref: descriptor
                    .audit_chain_ref
                    .unwrap_or_else(|| self.config.default_audit_chain_ref.clone()),
                subscription_channel: descriptor.subscription_channel,
                consumer_ref: descriptor.consumer_ref,
                subscription_event_types: descriptor.subscription_event_types,
                replay_cursor_ref: descriptor.replay_cursor_ref,
                max_batch_size: descriptor.max_batch_size,
                subscription_authorization_evidence_ref: descriptor
                    .subscription_authorization_evidence_ref,
                candidate_channel: descriptor.candidate_channel,
                candidate_event_id: descriptor.candidate_event_id,
                candidate_event_type: descriptor.candidate_event_type,
                candidate_idempotency_key: descriptor.candidate_idempotency_key,
                candidate_payload_ref: descriptor.candidate_payload_ref,
                candidate_offset_ref: descriptor.candidate_offset_ref,
                candidate_evidence_refs: sorted_unique(
                    [
                        descriptor.candidate_evidence_refs,
                        vec!["workflow-event-bus-sdk:delivery-descriptor".to_owned()],
                    ]
                    .concat(),
                ),
            },
        })
    }

    fn boundary(
        &self,
        context: &WorkflowEventBusSdkRequestContext,
    ) -> WorkflowEventBusApiBoundaryContext {
        WorkflowEventBusApiBoundaryContext {
            request_id: context.request_id.clone(),
            tenant_id: self.config.tenant_id.clone(),
            idempotency_key: context.idempotency_key.clone(),
            trace_context_ref: context
                .trace_context_ref
                .clone()
                .unwrap_or_else(|| self.config.trace_context_ref.clone()),
            oyatie_version: self.config.oyatie_version.clone(),
        }
    }

    fn principal(&self) -> WorkflowEventBusApiPrincipal {
        WorkflowEventBusApiPrincipal {
            tenant_id: self.config.tenant_id.clone(),
            principal_id: self.config.principal_id.clone(),
        }
    }

    fn authorization(&self) -> WorkflowEventBusApiAuthorization {
        WorkflowEventBusApiAuthorization {
            tenant_id: self.config.tenant_id.clone(),
            principal_id: self.config.principal_id.clone(),
            decision_id: self.config.authorization_decision_id.clone(),
            evidence_ref: self.config.authorization_evidence_ref.clone(),
            policy_bundle_ref: self.config.policy_bundle_ref.clone(),
            allowed_surfaces: vec![WORKFLOW_EVENT_BUS_API_SURFACE.to_owned()],
            allowed_channels: self.config.allowed_channels.clone(),
            allowed_event_types: self.config.allowed_event_types.clone(),
        }
    }
}

fn validate_config(config: &WorkflowEventBusSdkConfig) -> Result<(), WorkflowEventBusSdkError> {
    if !is_safe_tenant(&config.tenant_id)
        || !is_safe_ref(&config.principal_id)
        || !is_safe_ref(&config.authorization_decision_id)
        || !is_safe_ref(&config.authorization_evidence_ref)
        || !is_safe_ref(&config.policy_bundle_ref)
        || !is_safe_ref(&config.default_cell_id)
        || !is_safe_ref(&config.default_residency_ref)
        || !is_safe_ref(&config.default_audit_chain_ref)
        || !is_safe_ref(&config.trace_context_ref)
        || config.allowed_channels.is_empty()
        || config.allowed_event_types.is_empty()
        || !config
            .allowed_channels
            .iter()
            .all(|value| is_safe_metadata(value))
        || !config
            .allowed_event_types
            .iter()
            .all(|value| is_safe_metadata(value))
    {
        return Err(WorkflowEventBusSdkError::InvalidConfig {
            evidence_ref: "workflow-event-bus-sdk:invalid-config".to_owned(),
        });
    }
    if config.oyatie_version != WORKFLOW_EVENT_BUS_SDK_DECLARED_VERSION {
        return Err(WorkflowEventBusSdkError::InvalidConfig {
            evidence_ref: "workflow-event-bus-sdk:version-mismatch".to_owned(),
        });
    }
    Ok(())
}

fn validate_context(
    context: &WorkflowEventBusSdkRequestContext,
) -> Result<(), WorkflowEventBusSdkError> {
    if !is_safe_ref(&context.request_id)
        || !is_safe_ref(&context.idempotency_key)
        || !is_safe_optional_ref(context.trace_context_ref.as_deref())
        || !is_safe_ref(&context.correlation_ref)
    {
        return Err(WorkflowEventBusSdkError::InvalidRequest {
            evidence_ref: "workflow-event-bus-sdk:invalid-context".to_owned(),
        });
    }
    Ok(())
}

fn validate_publish_descriptor(
    config: &WorkflowEventBusSdkConfig,
    descriptor: &WorkflowEventBusSdkPublishDescriptor,
) -> Result<(), WorkflowEventBusSdkError> {
    let required_refs = [
        &descriptor.event_id,
        &descriptor.producer_ref,
        &descriptor.source_ref,
        &descriptor.partition_key_ref,
        &descriptor.publish_idempotency_key,
        &descriptor.causation_ref,
        &descriptor.payload_ref,
    ];
    if !is_safe_metadata(&descriptor.event_kind)
        || !required_refs.iter().all(|value| is_safe_ref(value))
        || !is_safe_optional_ref(descriptor.subject_ref.as_deref())
        || !is_safe_optional_ref(descriptor.time_ref.as_deref())
        || !is_safe_optional_ref(descriptor.dataschema_ref.as_deref())
        || !is_safe_optional_ref(descriptor.correlation_ref.as_deref())
        || !is_safe_optional_ref(descriptor.cell_id.as_deref())
        || !is_safe_optional_ref(descriptor.residency_ref.as_deref())
        || !is_safe_optional_ref(descriptor.audit_chain_ref.as_deref())
        || !descriptor
            .evidence_refs
            .iter()
            .all(|value| is_safe_ref(value))
    {
        return Err(WorkflowEventBusSdkError::UnsafeMetadata {
            evidence_ref: "workflow-event-bus-sdk:unsafe-publish-descriptor".to_owned(),
        });
    }
    if !config.allowed_event_types.contains(&descriptor.event_kind) {
        return Err(WorkflowEventBusSdkError::MetadataMismatch {
            evidence_ref: "workflow-event-bus-sdk:publish-event-type-not-allowed".to_owned(),
        });
    }
    Ok(())
}

fn validate_delivery_descriptor(
    config: &WorkflowEventBusSdkConfig,
    descriptor: &WorkflowEventBusSdkDeliveryDescriptor,
) -> Result<(), WorkflowEventBusSdkError> {
    let required_refs = [
        &descriptor.consumer_ref,
        &descriptor.subscription_authorization_evidence_ref,
        &descriptor.candidate_event_id,
        &descriptor.candidate_idempotency_key,
        &descriptor.candidate_payload_ref,
        &descriptor.candidate_offset_ref,
    ];
    if !is_safe_metadata(&descriptor.subscription_channel)
        || !is_safe_metadata(&descriptor.candidate_channel)
        || !is_safe_metadata(&descriptor.candidate_event_type)
        || !required_refs.iter().all(|value| is_safe_ref(value))
        || descriptor.subscription_event_types.is_empty()
        || !descriptor
            .subscription_event_types
            .iter()
            .all(|value| is_safe_metadata(value))
        || !is_safe_optional_ref(descriptor.replay_cursor_ref.as_deref())
        || !is_safe_optional_ref(descriptor.cell_id.as_deref())
        || !is_safe_optional_ref(descriptor.residency_ref.as_deref())
        || !is_safe_optional_ref(descriptor.audit_chain_ref.as_deref())
        || !descriptor
            .candidate_evidence_refs
            .iter()
            .all(|value| is_safe_ref(value))
    {
        return Err(WorkflowEventBusSdkError::UnsafeMetadata {
            evidence_ref: "workflow-event-bus-sdk:unsafe-delivery-descriptor".to_owned(),
        });
    }
    if descriptor.max_batch_size == 0 {
        return Err(WorkflowEventBusSdkError::InvalidRequest {
            evidence_ref: "workflow-event-bus-sdk:delivery-batch-size-required".to_owned(),
        });
    }
    if !config
        .allowed_channels
        .contains(&descriptor.subscription_channel)
        || !config
            .allowed_channels
            .contains(&descriptor.candidate_channel)
        || !config
            .allowed_event_types
            .contains(&descriptor.candidate_event_type)
        || !descriptor
            .subscription_event_types
            .contains(&descriptor.candidate_event_type)
    {
        return Err(WorkflowEventBusSdkError::MetadataMismatch {
            evidence_ref: "workflow-event-bus-sdk:delivery-metadata-mismatch".to_owned(),
        });
    }
    Ok(())
}

fn validate_plan(plan: &WorkflowEventBusSdkCommandPlan) -> Result<(), WorkflowEventBusSdkError> {
    if plan.method != WORKFLOW_EVENT_BUS_REST_METHOD
        || plan.contract_ref != WORKFLOW_EVENT_BUS_SDK_CONTRACT_REF
        || plan.api_version != WORKFLOW_EVENT_BUS_SDK_DECLARED_VERSION
        || plan.retry_policy.automatic_retries_enabled
    {
        return Err(WorkflowEventBusSdkError::InvalidRequest {
            evidence_ref: "workflow-event-bus-sdk:invalid-plan".to_owned(),
        });
    }
    if plan.path != plan.operation.rest_operation().route()
        || plan.body_kind != plan.operation.body_kind()
        || plan.rest_request.method != WORKFLOW_EVENT_BUS_REST_METHOD
        || plan.rest_request.path != plan.path
        || !is_safe_ref(&plan.rest_request.request_id)
        || !plan
            .evidence_refs
            .iter()
            .all(|value| is_safe_metadata(value))
    {
        return Err(WorkflowEventBusSdkError::InvalidRequest {
            evidence_ref: "workflow-event-bus-sdk:plan-route-mismatch".to_owned(),
        });
    }
    Ok(())
}

fn is_safe_tenant(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.starts_with("ten_") && value == trimmed && is_safe_metadata(value)
}

fn is_safe_ref(value: &str) -> bool {
    is_safe_metadata(value) && value.contains(':')
}

fn is_safe_optional_ref(value: Option<&str>) -> bool {
    value.is_none_or(is_safe_ref)
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
    values.retain(|value| {
        !value.trim().is_empty()
            && !contains_raw_secret_material(value)
            && !contains_raw_content_material(value)
    });
    values.sort();
    values.dedup();
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sdk_constants_defaults_and_operation_labels_are_contract_bound() {
        assert_eq!(
            WORKFLOW_EVENT_BUS_SDK_SURFACE,
            "workflow-engine.event-bus.sdk"
        );
        assert_eq!(
            WORKFLOW_EVENT_BUS_SDK_DECLARED_VERSION,
            WORKFLOW_EVENT_BUS_API_DECLARED_VERSION
        );
        assert_eq!(
            WORKFLOW_EVENT_BUS_SDK_CONTRACT_REF,
            WORKFLOW_EVENT_BUS_REST_CONTRACT_REF
        );
        let automatic_retries_enabled = WORKFLOW_EVENT_BUS_SDK_AUTOMATIC_RETRIES_ENABLED;
        assert!(!automatic_retries_enabled);
        assert_eq!(
            WorkflowEventBusSdkOperation::Publish.operation_id(),
            "publishEventBusEvent"
        );
        assert_eq!(
            WorkflowEventBusSdkOperation::EvaluateDelivery.rest_operation(),
            WorkflowEventBusRestOperation::EvaluateDelivery
        );
        let retry = WorkflowEventBusSdkRetryPolicy::default();
        assert!(!retry.automatic_retries_enabled);
        assert_eq!(
            retry.retry_policy_ref,
            WORKFLOW_EVENT_BUS_SDK_RETRY_POLICY_REF
        );
    }

    #[test]
    fn publish_plan_binds_route_version_authorization_idempotency_and_metadata() {
        let sdk = valid_sdk();
        let plan = sdk
            .plan_publish(
                context("idem:event-bus-sdk:publish:1"),
                publish_descriptor(),
            )
            .expect("publish plan");

        assert_eq!(plan.operation, WorkflowEventBusSdkOperation::Publish);
        assert_eq!(plan.method, WorkflowEventBusRestMethod::Post);
        assert_eq!(plan.path, WORKFLOW_EVENT_BUS_REST_PUBLISH_ROUTE);
        assert_eq!(plan.api_version, WORKFLOW_EVENT_BUS_API_DECLARED_VERSION);
        assert!(!plan.retry_policy.automatic_retries_enabled);
        assert!(
            plan.evidence_refs
                .contains(&WORKFLOW_EVENT_BUS_SDK_SURFACE.to_owned())
        );
        let WorkflowEventBusRestRequestBody::Publish(api_request) = &plan.rest_request.body else {
            unreachable!("expected publish body");
        };
        assert_eq!(api_request.method, WORKFLOW_EVENT_BUS_API_METHOD);
        assert_eq!(api_request.route, WORKFLOW_EVENT_BUS_API_PUBLISH_ROUTE);
        assert_eq!(api_request.boundary.tenant_id, "ten_workflow_event_bus");
        assert_eq!(
            api_request.boundary.idempotency_key,
            "idem:event-bus-sdk:publish:1"
        );
        assert_eq!(
            api_request.authorization.allowed_surfaces,
            vec![WORKFLOW_EVENT_BUS_API_SURFACE]
        );
        assert_eq!(api_request.body.cell_id, "cell:us-east-1a");
        assert_eq!(
            api_request.body.correlation_ref,
            "corr:event-bus-sdk:request"
        );
        assert_eq!(
            api_request.body.payload_ref,
            "body-ref:workflow-run-started"
        );
    }

    #[test]
    fn delivery_plan_binds_subscription_candidate_metadata_without_offset_commit_claim() {
        let sdk = valid_sdk();
        let plan = sdk
            .plan_delivery(
                context("idem:event-bus-sdk:delivery:1"),
                delivery_descriptor(),
            )
            .expect("delivery plan");

        assert_eq!(
            plan.operation,
            WorkflowEventBusSdkOperation::EvaluateDelivery
        );
        assert_eq!(plan.path, WORKFLOW_EVENT_BUS_REST_DELIVERY_ROUTE);
        assert_eq!(plan.body_kind, WorkflowEventBusRestBodyKind::Delivery);
        assert!(
            plan.evidence_refs
                .contains(&"workflow-event-bus-sdk:no-offset-commit-claim".to_owned())
        );
        let WorkflowEventBusRestRequestBody::Delivery(api_request) = &plan.rest_request.body else {
            unreachable!("expected delivery body");
        };
        assert_eq!(api_request.body.subscription_channel, "workflow-state");
        assert_eq!(api_request.body.candidate_channel, "workflow-state");
        assert_eq!(
            api_request.body.candidate_offset_ref,
            "offset:partition-0:42"
        );
        assert_eq!(
            api_request.body.consumer_ref,
            "consumer:workflow-state-machine"
        );
        assert_eq!(api_request.body.cell_id, "cell:us-east-1a");
    }

    #[test]
    fn invalid_config_or_request_denies_before_rest_without_raw_echo() {
        let mut invalid_config = valid_config();
        invalid_config.authorization_evidence_ref =
            "raw prompt Authorization: Bearer sk-test customer message".to_owned();
        let config_error = WorkflowEventBusSdk::new(invalid_config).expect_err("invalid config");
        assert_eq!(
            config_error.primary_evidence_ref(),
            "workflow-event-bus-sdk:invalid-config"
        );
        let rendered_config = format!("{config_error:?}");
        assert!(!rendered_config.contains("raw prompt"));
        assert!(!rendered_config.contains("sk-test"));
        assert!(!rendered_config.contains("customer message"));

        let sdk = valid_sdk();
        let mut invalid_context = context("idem:event-bus-sdk:invalid-request");
        invalid_context.request_id = "request:raw payload bearer sk-test".to_owned();
        let request_error = sdk
            .plan_publish(invalid_context, publish_descriptor())
            .expect_err("invalid context");
        assert_eq!(
            request_error.primary_evidence_ref(),
            "workflow-event-bus-sdk:invalid-context"
        );
        let rendered_request = format!("{request_error:?}");
        assert!(!rendered_request.contains("raw payload"));
        assert!(!rendered_request.contains("sk-test"));
    }

    #[test]
    fn metadata_mismatch_and_unsafe_descriptor_deny_before_rest_without_raw_echo() {
        let sdk = valid_sdk();
        let mut mismatch = publish_descriptor();
        mismatch.event_kind = WorkflowEventBusEventKind::OntologyProjectionUpdated
            .event_type()
            .to_owned();
        let mismatch_error = sdk
            .plan_publish(context("idem:event-bus-sdk:mismatch"), mismatch)
            .expect_err("publish event type denied by config");
        assert_eq!(
            mismatch_error.primary_evidence_ref(),
            "workflow-event-bus-sdk:publish-event-type-not-allowed"
        );

        let mut unsafe_descriptor = publish_descriptor();
        unsafe_descriptor.payload_ref =
            "raw payload Authorization: Bearer sk-test customer message".to_owned();
        let unsafe_error = sdk
            .plan_publish(
                context("idem:event-bus-sdk:unsafe-descriptor"),
                unsafe_descriptor,
            )
            .expect_err("unsafe descriptor");
        assert_eq!(
            unsafe_error.primary_evidence_ref(),
            "workflow-event-bus-sdk:unsafe-publish-descriptor"
        );
        let rendered = format!("{unsafe_error:?}");
        assert!(!rendered.contains("raw payload"));
        assert!(!rendered.contains("Authorization"));
        assert!(!rendered.contains("sk-test"));
        assert!(!rendered.contains("customer message"));
    }

    #[test]
    fn in_process_preview_execute_delegates_through_rest_api_without_http_client() {
        let sdk = valid_sdk();
        let plan = sdk
            .plan_publish(context("idem:event-bus-sdk:execute"), publish_descriptor())
            .expect("publish plan");
        let mut rest = WorkflowEventBusRestService::default();

        let response = sdk
            .execute_in_process(plan, &mut rest)
            .expect("in-process response");

        assert_eq!(response.status_code, 202);
        assert_eq!(
            response.content_type,
            WORKFLOW_EVENT_BUS_REST_SUCCESS_CONTENT_TYPE
        );
        let WorkflowEventBusRestResponseBody::Success(success) = response.body else {
            unreachable!("expected success body");
        };
        assert_eq!(success.route, WORKFLOW_EVENT_BUS_API_PUBLISH_ROUTE);
        assert_eq!(success.metadata.surface, WORKFLOW_EVENT_BUS_API_SURFACE);
        assert!(
            success
                .non_claim_refs
                .contains(&"workflow-event-bus-usecase:no-broker-runtime".to_owned())
        );
        assert_eq!(rest.api_delegation_count(), 1);
    }

    #[test]
    fn idempotent_replay_and_conflict_are_preserved_through_sdk_rest_api() {
        let sdk = valid_sdk();
        let plan = sdk
            .plan_publish(context("idem:event-bus-sdk:replay"), publish_descriptor())
            .expect("first plan");
        let mut rest = WorkflowEventBusRestService::default();
        let first = sdk
            .execute_in_process(plan.clone(), &mut rest)
            .expect("first");
        let second = sdk.execute_in_process(plan, &mut rest).expect("second");
        assert_eq!(first, second);
        assert_eq!(rest.api_delegation_count(), 2);

        let mut drifted_descriptor = publish_descriptor();
        drifted_descriptor.event_id = "event:workflow-run-started:drift".to_owned();
        let drifted = sdk
            .plan_publish(context("idem:event-bus-sdk:replay"), drifted_descriptor)
            .expect("drifted plan");
        let conflict = sdk
            .execute_in_process(drifted, &mut rest)
            .expect("conflict as problem response");
        assert_eq!(conflict.status_code, 409);
        assert!(
            format!("{conflict:?}")
                .contains(WorkflowEventBusApiErrorCode::IdempotencyKeyReused.as_str())
        );
    }

    #[test]
    fn automatic_retries_remain_disabled_for_state_changing_sdk_plans() {
        let sdk = valid_sdk();
        let publish = sdk
            .plan_publish(
                context("idem:event-bus-sdk:retry-publish"),
                publish_descriptor(),
            )
            .expect("publish plan");
        let delivery = sdk
            .plan_delivery(
                context("idem:event-bus-sdk:retry-delivery"),
                delivery_descriptor(),
            )
            .expect("delivery plan");

        for plan in [publish, delivery] {
            assert!(!plan.retry_policy.automatic_retries_enabled);
            assert_eq!(
                plan.retry_policy.retry_policy_ref,
                WORKFLOW_EVENT_BUS_SDK_RETRY_POLICY_REF
            );
            assert_eq!(plan.method, WORKFLOW_EVENT_BUS_REST_METHOD);
            assert!(plan.contract_ref.contains("workflow-engine.yaml"));
        }
    }

    fn valid_sdk() -> WorkflowEventBusSdk {
        WorkflowEventBusSdk::new(valid_config()).expect("valid sdk")
    }

    fn valid_config() -> WorkflowEventBusSdkConfig {
        WorkflowEventBusSdkConfig {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            principal_id: "principal:workflow-operator".to_owned(),
            authorization_decision_id: "policy-decision:event-bus-allow".to_owned(),
            authorization_evidence_ref: "policy-evidence:event-bus-allow".to_owned(),
            policy_bundle_ref: "policy-bundle:event-bus-v1".to_owned(),
            default_cell_id: "cell:us-east-1a".to_owned(),
            default_residency_ref: "residency:us:data-plane".to_owned(),
            default_audit_chain_ref: "audit-chain:event-bus-sdk".to_owned(),
            trace_context_ref: "trace:event-bus-sdk".to_owned(),
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
            request_id: format!("request:event-bus-sdk:{idempotency_key}"),
            idempotency_key: idempotency_key.to_owned(),
            trace_context_ref: None,
            correlation_ref: "corr:event-bus-sdk:request".to_owned(),
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
            evidence_refs: vec!["evidence:event-bus-sdk:publish".to_owned()],
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
            replay_cursor_ref: Some("cursor:event-bus-sdk:state".to_owned()),
            max_batch_size: 100,
            subscription_authorization_evidence_ref: "authz:event-bus-sdk:consume".to_owned(),
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
            candidate_evidence_refs: vec!["evidence:event-bus-sdk:delivery".to_owned()],
        }
    }
}
