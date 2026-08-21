//! Workflow-engine event-bus API boundary foundation.
//!
//! This crate owns a source-level, framework-free API boundary for workflow
//! event-bus publish and delivery-evaluation commands that will later be served
//! by REST/gRPC adapters. It validates method, route, contract version, tenant,
//! principal, authorization, idempotency, policy/residency/audit refs,
//! CloudEvents-shaped event metadata, AsyncAPI-shaped channel metadata, and safe
//! request bodies before mapping DTOs into the event-bus usecase. It returns
//! stable OpenAPI/RFC-9457-shaped status and problem DTOs while preserving
//! in-memory idempotent API replay semantics. It performs no HTTP serving,
//! serialization-framework work, concrete storage, broker connection, topic
//! creation, network I/O, durable idempotency storage, durable outbox/inbox
//! writes, consumer group coordination, offset commits, signing, Kubernetes
//! calls, cloud deployment, or tenant workload scheduling.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

pub use workflow_event_bus_usecase::{
    WORKFLOW_EVENT_BUS_ASYNCAPI_CONTRACT_REF, WORKFLOW_EVENT_BUS_CLOUDEVENTS_SPECVERSION,
    WORKFLOW_EVENT_BUS_DEFAULT_CONTENT_TYPE, WORKFLOW_EVENT_BUS_DOMAIN_NON_CLAIM_REF,
    WORKFLOW_EVENT_BUS_DOMAIN_SURFACE, WORKFLOW_EVENT_BUS_KERNEL_SURFACE,
    WORKFLOW_EVENT_BUS_MAX_SUBSCRIPTION_BATCH_SIZE, WORKFLOW_EVENT_BUS_USECASE_CONTRACT_REF,
    WORKFLOW_EVENT_BUS_USECASE_SURFACE, WorkflowEventBusChannel, WorkflowEventBusCloudEvent,
    WorkflowEventBusContext, WorkflowEventBusDeliveryCandidate, WorkflowEventBusDeliveryDecision,
    WorkflowEventBusDeliveryStatus, WorkflowEventBusDomain, WorkflowEventBusDomainDeliveryReceipt,
    WorkflowEventBusDomainError, WorkflowEventBusDomainPolicyBinding,
    WorkflowEventBusDomainPublishIntent, WorkflowEventBusDomainPublishReceipt,
    WorkflowEventBusDomainStatus, WorkflowEventBusDomainSubscriptionIntent,
    WorkflowEventBusEventKind, WorkflowEventBusKernelError, WorkflowEventBusPublishPlan,
    WorkflowEventBusPublishRequest, WorkflowEventBusSubscription, WorkflowEventBusUsecase,
    WorkflowEventBusUsecaseDeliveryCommand, WorkflowEventBusUsecaseDeliveryReceipt,
    WorkflowEventBusUsecasePublishCommand, WorkflowEventBusUsecasePublishReceipt,
    WorkflowEventBusUsecaseStatus, evaluate_delivery, plan_publish,
};

pub const WORKFLOW_EVENT_BUS_API_SURFACE: &str = "workflow-engine.event-bus.command";
pub const WORKFLOW_EVENT_BUS_API_DECLARED_VERSION: &str = "2026-05-25";
pub const WORKFLOW_EVENT_BUS_API_CONTRACT_REF: &str =
    "workflow/workflow-engine/contracts/openapi/workflow-engine.yaml";
pub const WORKFLOW_EVENT_BUS_API_PUBLISH_ROUTE: &str = "/v/2026-05-25/event-bus/publish";
pub const WORKFLOW_EVENT_BUS_API_DELIVERY_ROUTE: &str = "/v/2026-05-25/event-bus/delivery/evaluate";
pub const WORKFLOW_EVENT_BUS_API_METHOD: &str = "POST";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkflowEventBusApiStatus {
    Accepted,
    BadRequest,
    Forbidden,
    Conflict,
    UnprocessableContent,
    ServiceUnavailable,
}

impl WorkflowEventBusApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Accepted => 202,
            Self::BadRequest => 400,
            Self::Forbidden => 403,
            Self::Conflict => 409,
            Self::UnprocessableContent => 422,
            Self::ServiceUnavailable => 503,
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::Accepted => "Accepted",
            Self::BadRequest => "Bad Request",
            Self::Forbidden => "Forbidden",
            Self::Conflict => "Conflict",
            Self::UnprocessableContent => "Unprocessable Content",
            Self::ServiceUnavailable => "Service Unavailable",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkflowEventBusApiErrorCode {
    AuthorizationDenied,
    AuthorizationEvidenceInvalid,
    AuthorizationPrincipalMismatch,
    AuthorizationTenantMismatch,
    ChannelMismatch,
    ContractVersionUnsupported,
    DeliveryDenied,
    DomainDenied,
    EventBusInvalid,
    IdempotencyKeyReused,
    MethodNotAllowed,
    RequestIdEmpty,
    RouteMismatch,
    TenantBindingMismatch,
    TenantHeaderEmpty,
    TraceContextInvalid,
    UnknownChannel,
    UnknownEventKind,
    UnsafeMetadata,
    UsecaseUnavailable,
}

impl WorkflowEventBusApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AuthorizationDenied => "WORKFLOW_EVENT_BUS_AUTHORIZATION_DENIED",
            Self::AuthorizationEvidenceInvalid => {
                "WORKFLOW_EVENT_BUS_AUTHORIZATION_EVIDENCE_INVALID"
            }
            Self::AuthorizationPrincipalMismatch => {
                "WORKFLOW_EVENT_BUS_AUTHORIZATION_PRINCIPAL_MISMATCH"
            }
            Self::AuthorizationTenantMismatch => "WORKFLOW_EVENT_BUS_AUTHORIZATION_TENANT_MISMATCH",
            Self::ChannelMismatch => "WORKFLOW_EVENT_BUS_CHANNEL_MISMATCH",
            Self::ContractVersionUnsupported => "WORKFLOW_EVENT_BUS_CONTRACT_VERSION_UNSUPPORTED",
            Self::DeliveryDenied => "WORKFLOW_EVENT_BUS_DELIVERY_DENIED",
            Self::DomainDenied => "WORKFLOW_EVENT_BUS_DOMAIN_DENIED",
            Self::EventBusInvalid => "WORKFLOW_EVENT_BUS_INVALID",
            Self::IdempotencyKeyReused => "WORKFLOW_EVENT_BUS_IDEMPOTENCY_KEY_REUSED",
            Self::MethodNotAllowed => "WORKFLOW_EVENT_BUS_METHOD_NOT_ALLOWED",
            Self::RequestIdEmpty => "WORKFLOW_EVENT_BUS_REQUEST_ID_EMPTY",
            Self::RouteMismatch => "WORKFLOW_EVENT_BUS_ROUTE_MISMATCH",
            Self::TenantBindingMismatch => "WORKFLOW_EVENT_BUS_TENANT_BINDING_MISMATCH",
            Self::TenantHeaderEmpty => "WORKFLOW_EVENT_BUS_TENANT_HEADER_EMPTY",
            Self::TraceContextInvalid => "WORKFLOW_EVENT_BUS_TRACE_CONTEXT_INVALID",
            Self::UnknownChannel => "WORKFLOW_EVENT_BUS_UNKNOWN_CHANNEL",
            Self::UnknownEventKind => "WORKFLOW_EVENT_BUS_UNKNOWN_EVENT_KIND",
            Self::UnsafeMetadata => "WORKFLOW_EVENT_BUS_UNSAFE_METADATA",
            Self::UsecaseUnavailable => "WORKFLOW_EVENT_BUS_USECASE_UNAVAILABLE",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusApiBoundaryContext {
    pub request_id: String,        // data_class: INTERNAL_ONLY
    pub tenant_id: String,         // data_class: INTERNAL_ONLY
    pub idempotency_key: String,   // data_class: INTERNAL_ONLY
    pub trace_context_ref: String, // data_class: INTERNAL_ONLY
    pub oyatie_version: String,    // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusApiAuthorization {
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub principal_id: String,             // data_class: INTERNAL_ONLY
    pub decision_id: String,              // data_class: INTERNAL_ONLY
    pub evidence_ref: String,             // data_class: INTERNAL_ONLY
    pub policy_bundle_ref: String,        // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>,    // data_class: INTERNAL_ONLY
    pub allowed_channels: Vec<String>,    // data_class: INTERNAL_ONLY
    pub allowed_event_types: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusApiPublishRequest {
    pub boundary: WorkflowEventBusApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: WorkflowEventBusApiPrincipal,      // data_class: INTERNAL_ONLY
    pub authorization: WorkflowEventBusApiAuthorization, // data_class: INTERNAL_ONLY
    pub method: String,                               // data_class: PUBLIC
    pub route: String,                                // data_class: PUBLIC
    pub body: WorkflowEventBusApiPublishBody,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusApiPublishBody {
    pub cell_id: String,                 // data_class: INTERNAL_ONLY
    pub residency_ref: String,           // data_class: INTERNAL_ONLY
    pub audit_chain_ref: String,         // data_class: INTERNAL_ONLY
    pub event_kind: String,              // data_class: PUBLIC
    pub producer_ref: String,            // data_class: INTERNAL_ONLY
    pub event_id: String,                // data_class: INTERNAL_ONLY
    pub source_ref: String,              // data_class: INTERNAL_ONLY
    pub subject_ref: Option<String>,     // data_class: INTERNAL_ONLY
    pub time_ref: Option<String>,        // data_class: INTERNAL_ONLY
    pub dataschema_ref: Option<String>,  // data_class: INTERNAL_ONLY
    pub partition_key_ref: String,       // data_class: INTERNAL_ONLY
    pub publish_idempotency_key: String, // data_class: INTERNAL_ONLY
    pub causation_ref: String,           // data_class: INTERNAL_ONLY
    pub correlation_ref: String,         // data_class: INTERNAL_ONLY
    pub payload_ref: String,             // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusApiDeliveryRequest {
    pub boundary: WorkflowEventBusApiBoundaryContext, // data_class: INTERNAL_ONLY
    pub principal: WorkflowEventBusApiPrincipal,      // data_class: INTERNAL_ONLY
    pub authorization: WorkflowEventBusApiAuthorization, // data_class: INTERNAL_ONLY
    pub method: String,                               // data_class: PUBLIC
    pub route: String,                                // data_class: PUBLIC
    pub body: WorkflowEventBusApiDeliveryBody,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusApiDeliveryBody {
    pub cell_id: String,                                 // data_class: INTERNAL_ONLY
    pub residency_ref: String,                           // data_class: INTERNAL_ONLY
    pub audit_chain_ref: String,                         // data_class: INTERNAL_ONLY
    pub subscription_channel: String,                    // data_class: PUBLIC
    pub consumer_ref: String,                            // data_class: INTERNAL_ONLY
    pub subscription_event_types: Vec<String>,           // data_class: INTERNAL_ONLY
    pub replay_cursor_ref: Option<String>,               // data_class: INTERNAL_ONLY
    pub max_batch_size: u32,                             // data_class: INTERNAL_ONLY
    pub subscription_authorization_evidence_ref: String, // data_class: INTERNAL_ONLY
    pub candidate_channel: String,                       // data_class: PUBLIC
    pub candidate_event_id: String,                      // data_class: INTERNAL_ONLY
    pub candidate_event_type: String,                    // data_class: PUBLIC
    pub candidate_idempotency_key: String,               // data_class: INTERNAL_ONLY
    pub candidate_payload_ref: String,                   // data_class: INTERNAL_ONLY
    pub candidate_offset_ref: String,                    // data_class: INTERNAL_ONLY
    pub candidate_evidence_refs: Vec<String>,            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusApiResponseMetadata {
    pub request_id: String,        // data_class: INTERNAL_ONLY
    pub tenant_id: String,         // data_class: INTERNAL_ONLY
    pub idempotency_key: String,   // data_class: INTERNAL_ONLY
    pub trace_context_ref: String, // data_class: INTERNAL_ONLY
    pub surface: String,           // data_class: INTERNAL_ONLY
    pub contract_ref: String,      // data_class: INTERNAL_ONLY
    pub oyatie_version: String,    // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusApiEventDto {
    pub operation: String,                    // data_class: PUBLIC
    pub usecase_status: String,               // data_class: PUBLIC
    pub domain_status: Option<String>,        // data_class: PUBLIC
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub cell_id: String,                      // data_class: INTERNAL_ONLY
    pub event_type: String,                   // data_class: PUBLIC
    pub channel_address: Option<String>,      // data_class: PUBLIC
    pub delivery_key: Option<String>,         // data_class: INTERNAL_ONLY
    pub consumer_ref: Option<String>,         // data_class: INTERNAL_ONLY
    pub offset_ref: Option<String>,           // data_class: INTERNAL_ONLY
    pub asyncapi_channel_ref: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusApiSuccessResponse {
    pub status: WorkflowEventBusApiStatus,  // data_class: PUBLIC
    pub route: String,                      // data_class: PUBLIC
    pub event: WorkflowEventBusApiEventDto, // data_class: INTERNAL_ONLY
    pub metadata: WorkflowEventBusApiResponseMetadata, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,         // data_class: INTERNAL_ONLY
    pub non_claim_refs: Vec<String>,        // data_class: INTERNAL_ONLY
}

impl WorkflowEventBusApiSuccessResponse {
    pub fn http_status_code(&self) -> u16 {
        self.status.code()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusApiProblemDetails {
    pub type_ref: String,           // data_class: PUBLIC
    pub status: u16,                // data_class: PUBLIC
    pub code: String,               // data_class: PUBLIC
    pub title: String,              // data_class: PUBLIC
    pub detail_ref: String,         // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkflowEventBusApiError {
    Boundary {
        code: WorkflowEventBusApiErrorCode,
        status: WorkflowEventBusApiStatus,
        evidence_ref: String,
    },
    IdempotencyConflict,
    UsecaseDenied {
        status: WorkflowEventBusApiStatus,
        code: WorkflowEventBusApiErrorCode,
        evidence_refs: Vec<String>,
    },
}

impl WorkflowEventBusApiError {
    pub fn status(&self) -> WorkflowEventBusApiStatus {
        match self {
            Self::Boundary { status, .. } | Self::UsecaseDenied { status, .. } => *status,
            Self::IdempotencyConflict => WorkflowEventBusApiStatus::Conflict,
        }
    }

    pub fn status_code(&self) -> u16 {
        self.status().code()
    }

    pub fn code(&self) -> WorkflowEventBusApiErrorCode {
        match self {
            Self::Boundary { code, .. } | Self::UsecaseDenied { code, .. } => *code,
            Self::IdempotencyConflict => WorkflowEventBusApiErrorCode::IdempotencyKeyReused,
        }
    }

    pub fn problem(&self) -> WorkflowEventBusApiProblemDetails {
        let status = self.status();
        let code = self.code();
        let evidence_refs = match self {
            Self::Boundary { evidence_ref, .. } => vec![evidence_ref.clone()],
            Self::UsecaseDenied { evidence_refs, .. } => sorted_unique(evidence_refs.clone()),
            Self::IdempotencyConflict => {
                vec!["workflow-event-bus-api:idempotency-conflict".to_owned()]
            }
        };
        WorkflowEventBusApiProblemDetails {
            type_ref: format!("problem:workflow-event-bus:{}", code.as_str()),
            status: status.code(),
            code: code.as_str().to_owned(),
            title: status.title().to_owned(),
            detail_ref: format!("detail:workflow-event-bus:{}", code.as_str()),
            evidence_refs,
        }
    }
}

#[derive(Default)]
pub struct WorkflowEventBusApi {
    usecase: WorkflowEventBusUsecase,
    publish_responses_by_idempotency:
        BTreeMap<String, (String, WorkflowEventBusApiSuccessResponse)>,
    delivery_responses_by_idempotency:
        BTreeMap<String, (String, WorkflowEventBusApiSuccessResponse)>,
}

impl WorkflowEventBusApi {
    pub fn publish_event(
        &mut self,
        request: WorkflowEventBusApiPublishRequest,
    ) -> Result<WorkflowEventBusApiSuccessResponse, WorkflowEventBusApiError> {
        validate_publish_boundary(&request)?;
        validate_publish_body_metadata(&request)?;
        let event_kind = parse_event_kind(&request.body.event_kind)?;
        let allowed_channels = parse_channels(&request.authorization.allowed_channels)?;
        let fingerprint = format!("{request:?}");
        let cache_key = idempotency_cache_key(&request.boundary);
        if let Some((cached_fingerprint, response)) =
            self.publish_responses_by_idempotency.get(&cache_key)
        {
            if cached_fingerprint == &fingerprint {
                return Ok(response.clone());
            }
            return Err(WorkflowEventBusApiError::IdempotencyConflict);
        }

        let command = WorkflowEventBusUsecasePublishCommand {
            request_id: request.boundary.request_id.clone(),
            idempotency_key: request.boundary.idempotency_key.clone(),
            trace_ref: request.boundary.trace_context_ref.clone(),
            binding: policy_binding_from_publish(&request, allowed_channels),
            intent: publish_intent_from_body(&request.body, event_kind),
        };
        let receipt = self.usecase.publish(command);
        let response = map_publish_receipt(&request, receipt)?;
        self.publish_responses_by_idempotency
            .insert(cache_key, (fingerprint, response.clone()));
        Ok(response)
    }

    pub fn evaluate_delivery(
        &mut self,
        request: WorkflowEventBusApiDeliveryRequest,
    ) -> Result<WorkflowEventBusApiSuccessResponse, WorkflowEventBusApiError> {
        validate_delivery_boundary(&request)?;
        validate_delivery_body_metadata(&request)?;
        let subscription_channel = parse_channel(&request.body.subscription_channel)?;
        let candidate_channel = parse_channel(&request.body.candidate_channel)?;
        let allowed_channels = parse_channels(&request.authorization.allowed_channels)?;
        let fingerprint = format!("{request:?}");
        let cache_key = idempotency_cache_key(&request.boundary);
        if let Some((cached_fingerprint, response)) =
            self.delivery_responses_by_idempotency.get(&cache_key)
        {
            if cached_fingerprint == &fingerprint {
                return Ok(response.clone());
            }
            return Err(WorkflowEventBusApiError::IdempotencyConflict);
        }

        let command = WorkflowEventBusUsecaseDeliveryCommand {
            request_id: request.boundary.request_id.clone(),
            idempotency_key: request.boundary.idempotency_key.clone(),
            trace_ref: request.boundary.trace_context_ref.clone(),
            binding: policy_binding_from_delivery(&request, allowed_channels),
            subscription_intent: WorkflowEventBusDomainSubscriptionIntent {
                consumer_ref: request.body.consumer_ref.clone(),
                channel: subscription_channel,
                allowed_event_types: request.body.subscription_event_types.clone(),
                replay_cursor_ref: request.body.replay_cursor_ref.clone(),
                max_batch_size: request.body.max_batch_size,
                authorization_evidence_ref: request
                    .body
                    .subscription_authorization_evidence_ref
                    .clone(),
            },
            candidate: WorkflowEventBusDeliveryCandidate {
                tenant_id: request.boundary.tenant_id.clone(),
                cell_id: request.body.cell_id.clone(),
                channel: candidate_channel,
                event_id: request.body.candidate_event_id.clone(),
                event_type: request.body.candidate_event_type.clone(),
                idempotency_key: request.body.candidate_idempotency_key.clone(),
                payload_ref: request.body.candidate_payload_ref.clone(),
                offset_ref: request.body.candidate_offset_ref.clone(),
                evidence_refs: request.body.candidate_evidence_refs.clone(),
            },
        };
        let receipt = self.usecase.evaluate_delivery(command);
        let response = map_delivery_receipt(&request, receipt)?;
        self.delivery_responses_by_idempotency
            .insert(cache_key, (fingerprint, response.clone()));
        Ok(response)
    }
}

fn policy_binding_from_publish(
    request: &WorkflowEventBusApiPublishRequest,
    allowed_channels: Vec<WorkflowEventBusChannel>,
) -> WorkflowEventBusDomainPolicyBinding {
    WorkflowEventBusDomainPolicyBinding {
        tenant_id: request.boundary.tenant_id.clone(),
        cell_id: request.body.cell_id.clone(),
        principal_id: request.principal.principal_id.clone(),
        authorization_decision_id: request.authorization.decision_id.clone(),
        authorization_evidence_ref: request.authorization.evidence_ref.clone(),
        policy_bundle_ref: request.authorization.policy_bundle_ref.clone(),
        residency_ref: request.body.residency_ref.clone(),
        trace_context_ref: request.boundary.trace_context_ref.clone(),
        audit_chain_ref: request.body.audit_chain_ref.clone(),
        allowed_channels,
        allowed_event_types: request.authorization.allowed_event_types.clone(),
    }
}

fn policy_binding_from_delivery(
    request: &WorkflowEventBusApiDeliveryRequest,
    allowed_channels: Vec<WorkflowEventBusChannel>,
) -> WorkflowEventBusDomainPolicyBinding {
    WorkflowEventBusDomainPolicyBinding {
        tenant_id: request.boundary.tenant_id.clone(),
        cell_id: request.body.cell_id.clone(),
        principal_id: request.principal.principal_id.clone(),
        authorization_decision_id: request.authorization.decision_id.clone(),
        authorization_evidence_ref: request.authorization.evidence_ref.clone(),
        policy_bundle_ref: request.authorization.policy_bundle_ref.clone(),
        residency_ref: request.body.residency_ref.clone(),
        trace_context_ref: request.boundary.trace_context_ref.clone(),
        audit_chain_ref: request.body.audit_chain_ref.clone(),
        allowed_channels,
        allowed_event_types: request.authorization.allowed_event_types.clone(),
    }
}

fn publish_intent_from_body(
    body: &WorkflowEventBusApiPublishBody,
    event_kind: WorkflowEventBusEventKind,
) -> WorkflowEventBusDomainPublishIntent {
    WorkflowEventBusDomainPublishIntent {
        producer_ref: body.producer_ref.clone(),
        event_kind,
        event_id: body.event_id.clone(),
        source_ref: body.source_ref.clone(),
        subject_ref: body.subject_ref.clone(),
        time_ref: body.time_ref.clone(),
        dataschema_ref: body.dataschema_ref.clone(),
        partition_key_ref: body.partition_key_ref.clone(),
        idempotency_key: body.publish_idempotency_key.clone(),
        causation_ref: body.causation_ref.clone(),
        correlation_ref: body.correlation_ref.clone(),
        payload_ref: body.payload_ref.clone(),
        evidence_refs: sorted_unique(
            [
                body.evidence_refs.clone(),
                vec![format!("api-surface:{WORKFLOW_EVENT_BUS_API_SURFACE}")],
            ]
            .concat(),
        ),
    }
}

fn map_publish_receipt(
    request: &WorkflowEventBusApiPublishRequest,
    receipt: WorkflowEventBusUsecasePublishReceipt,
) -> Result<WorkflowEventBusApiSuccessResponse, WorkflowEventBusApiError> {
    match receipt.status {
        WorkflowEventBusUsecaseStatus::Published => {
            let asyncapi_ref = receipt
                .domain_receipt
                .as_ref()
                .and_then(|domain| domain.publish_plan.as_ref())
                .map(|plan| plan.asyncapi_channel_ref.clone());
            let mut evidence_refs = receipt.evidence_refs.clone();
            evidence_refs.push(WORKFLOW_EVENT_BUS_API_SURFACE.to_owned());
            Ok(WorkflowEventBusApiSuccessResponse {
                status: WorkflowEventBusApiStatus::Accepted,
                route: WORKFLOW_EVENT_BUS_API_PUBLISH_ROUTE.to_owned(),
                event: WorkflowEventBusApiEventDto {
                    operation: "publish".to_owned(),
                    usecase_status: receipt.status.as_wire().to_owned(),
                    domain_status: receipt
                        .domain_status
                        .map(domain_status_label)
                        .map(str::to_owned),
                    tenant_id: receipt.tenant_id.clone(),
                    cell_id: receipt.cell_id.clone(),
                    event_type: receipt.event_type.clone(),
                    channel_address: receipt.channel_address.clone(),
                    delivery_key: receipt.delivery_key.clone(),
                    consumer_ref: None,
                    offset_ref: None,
                    asyncapi_channel_ref: asyncapi_ref,
                },
                metadata: response_metadata(&request.boundary),
                evidence_refs: sorted_unique(evidence_refs),
                non_claim_refs: sorted_unique(receipt.non_claim_refs),
            })
        }
        WorkflowEventBusUsecaseStatus::DomainDenied => {
            Err(WorkflowEventBusApiError::UsecaseDenied {
                status: WorkflowEventBusApiStatus::Forbidden,
                code: WorkflowEventBusApiErrorCode::DomainDenied,
                evidence_refs: sorted_unique(receipt.evidence_refs),
            })
        }
        WorkflowEventBusUsecaseStatus::InvalidInput => {
            Err(WorkflowEventBusApiError::UsecaseDenied {
                status: WorkflowEventBusApiStatus::BadRequest,
                code: WorkflowEventBusApiErrorCode::EventBusInvalid,
                evidence_refs: sorted_unique(receipt.evidence_refs),
            })
        }
        WorkflowEventBusUsecaseStatus::IdempotencyConflict => {
            Err(WorkflowEventBusApiError::IdempotencyConflict)
        }
        WorkflowEventBusUsecaseStatus::DeliveryAccepted
        | WorkflowEventBusUsecaseStatus::DeliveryDenied => {
            Err(WorkflowEventBusApiError::UsecaseDenied {
                status: WorkflowEventBusApiStatus::ServiceUnavailable,
                code: WorkflowEventBusApiErrorCode::UsecaseUnavailable,
                evidence_refs: vec!["workflow-event-bus-api:unexpected-publish-status".to_owned()],
            })
        }
    }
}

fn map_delivery_receipt(
    request: &WorkflowEventBusApiDeliveryRequest,
    receipt: WorkflowEventBusUsecaseDeliveryReceipt,
) -> Result<WorkflowEventBusApiSuccessResponse, WorkflowEventBusApiError> {
    match receipt.status {
        WorkflowEventBusUsecaseStatus::DeliveryAccepted
        | WorkflowEventBusUsecaseStatus::DeliveryDenied => {
            let mut evidence_refs = receipt.evidence_refs.clone();
            evidence_refs.push(WORKFLOW_EVENT_BUS_API_SURFACE.to_owned());
            Ok(WorkflowEventBusApiSuccessResponse {
                status: WorkflowEventBusApiStatus::Accepted,
                route: WORKFLOW_EVENT_BUS_API_DELIVERY_ROUTE.to_owned(),
                event: WorkflowEventBusApiEventDto {
                    operation: "delivery-evaluate".to_owned(),
                    usecase_status: receipt.status.as_wire().to_owned(),
                    domain_status: receipt
                        .domain_status
                        .map(domain_status_label)
                        .map(str::to_owned),
                    tenant_id: receipt.tenant_id.clone(),
                    cell_id: receipt.cell_id.clone(),
                    event_type: receipt.event_type.clone(),
                    channel_address: Some(receipt.channel_address.clone()),
                    delivery_key: None,
                    consumer_ref: Some(receipt.consumer_ref.clone()),
                    offset_ref: Some(receipt.offset_ref.clone()),
                    asyncapi_channel_ref: Some(channel_asyncapi_ref(&receipt.channel_address)),
                },
                metadata: response_metadata(&request.boundary),
                evidence_refs: sorted_unique(evidence_refs),
                non_claim_refs: sorted_unique(receipt.non_claim_refs),
            })
        }
        WorkflowEventBusUsecaseStatus::DomainDenied => {
            Err(WorkflowEventBusApiError::UsecaseDenied {
                status: WorkflowEventBusApiStatus::Forbidden,
                code: WorkflowEventBusApiErrorCode::DomainDenied,
                evidence_refs: sorted_unique(receipt.evidence_refs),
            })
        }
        WorkflowEventBusUsecaseStatus::InvalidInput => {
            Err(WorkflowEventBusApiError::UsecaseDenied {
                status: WorkflowEventBusApiStatus::BadRequest,
                code: WorkflowEventBusApiErrorCode::EventBusInvalid,
                evidence_refs: sorted_unique(receipt.evidence_refs),
            })
        }
        WorkflowEventBusUsecaseStatus::IdempotencyConflict => {
            Err(WorkflowEventBusApiError::IdempotencyConflict)
        }
        WorkflowEventBusUsecaseStatus::Published => Err(WorkflowEventBusApiError::UsecaseDenied {
            status: WorkflowEventBusApiStatus::ServiceUnavailable,
            code: WorkflowEventBusApiErrorCode::UsecaseUnavailable,
            evidence_refs: vec!["workflow-event-bus-api:unexpected-delivery-status".to_owned()],
        }),
    }
}

fn validate_publish_boundary(
    request: &WorkflowEventBusApiPublishRequest,
) -> Result<(), WorkflowEventBusApiError> {
    validate_common_boundary(
        &request.boundary,
        &request.principal,
        &request.authorization,
        &request.method,
        &request.route,
        WORKFLOW_EVENT_BUS_API_PUBLISH_ROUTE,
    )
}

fn validate_delivery_boundary(
    request: &WorkflowEventBusApiDeliveryRequest,
) -> Result<(), WorkflowEventBusApiError> {
    validate_common_boundary(
        &request.boundary,
        &request.principal,
        &request.authorization,
        &request.method,
        &request.route,
        WORKFLOW_EVENT_BUS_API_DELIVERY_ROUTE,
    )
}

fn validate_common_boundary(
    boundary: &WorkflowEventBusApiBoundaryContext,
    principal: &WorkflowEventBusApiPrincipal,
    authorization: &WorkflowEventBusApiAuthorization,
    method: &str,
    route: &str,
    expected_route: &str,
) -> Result<(), WorkflowEventBusApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(boundary_error(
            WorkflowEventBusApiErrorCode::RequestIdEmpty,
            WorkflowEventBusApiStatus::BadRequest,
            "workflow-event-bus-api:request-id-required",
        ));
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(boundary_error(
            WorkflowEventBusApiErrorCode::TenantHeaderEmpty,
            WorkflowEventBusApiStatus::BadRequest,
            "workflow-event-bus-api:tenant-required",
        ));
    }
    if boundary.oyatie_version != WORKFLOW_EVENT_BUS_API_DECLARED_VERSION {
        return Err(boundary_error(
            WorkflowEventBusApiErrorCode::ContractVersionUnsupported,
            WorkflowEventBusApiStatus::BadRequest,
            "workflow-event-bus-api:unsupported-version",
        ));
    }
    if method != WORKFLOW_EVENT_BUS_API_METHOD {
        return Err(boundary_error(
            WorkflowEventBusApiErrorCode::MethodNotAllowed,
            WorkflowEventBusApiStatus::BadRequest,
            "workflow-event-bus-api:method-not-allowed",
        ));
    }
    if route != expected_route {
        return Err(boundary_error(
            WorkflowEventBusApiErrorCode::RouteMismatch,
            WorkflowEventBusApiStatus::BadRequest,
            "workflow-event-bus-api:route-mismatch",
        ));
    }
    if !is_safe_ref(&boundary.request_id)
        || !is_safe_tenant(&boundary.tenant_id)
        || !is_safe_ref(&boundary.idempotency_key)
    {
        return Err(boundary_error(
            WorkflowEventBusApiErrorCode::UnsafeMetadata,
            WorkflowEventBusApiStatus::BadRequest,
            "workflow-event-bus-api:unsafe-boundary-metadata",
        ));
    }
    if !is_safe_ref(&boundary.trace_context_ref) {
        return Err(boundary_error(
            WorkflowEventBusApiErrorCode::TraceContextInvalid,
            WorkflowEventBusApiStatus::BadRequest,
            "workflow-event-bus-api:trace-context-invalid",
        ));
    }
    if principal.tenant_id != boundary.tenant_id {
        return Err(boundary_error(
            WorkflowEventBusApiErrorCode::TenantBindingMismatch,
            WorkflowEventBusApiStatus::Forbidden,
            "workflow-event-bus-api:principal-tenant-mismatch",
        ));
    }
    if authorization.tenant_id != boundary.tenant_id {
        return Err(boundary_error(
            WorkflowEventBusApiErrorCode::AuthorizationTenantMismatch,
            WorkflowEventBusApiStatus::Forbidden,
            "workflow-event-bus-api:auth-tenant-mismatch",
        ));
    }
    if authorization.principal_id != principal.principal_id {
        return Err(boundary_error(
            WorkflowEventBusApiErrorCode::AuthorizationPrincipalMismatch,
            WorkflowEventBusApiStatus::Forbidden,
            "workflow-event-bus-api:auth-principal-mismatch",
        ));
    }
    if !is_safe_ref(&principal.principal_id)
        || !is_safe_ref(&authorization.decision_id)
        || !is_safe_ref(&authorization.evidence_ref)
        || !is_safe_ref(&authorization.policy_bundle_ref)
        || !authorization
            .allowed_surfaces
            .iter()
            .all(|value| is_safe_metadata(value))
        || !authorization
            .allowed_channels
            .iter()
            .all(|value| is_safe_metadata(value))
        || !authorization
            .allowed_event_types
            .iter()
            .all(|value| is_safe_metadata(value))
        || authorization.allowed_channels.is_empty()
        || authorization.allowed_event_types.is_empty()
    {
        return Err(boundary_error(
            WorkflowEventBusApiErrorCode::AuthorizationEvidenceInvalid,
            WorkflowEventBusApiStatus::Forbidden,
            "workflow-event-bus-api:auth-evidence-invalid",
        ));
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|surface| surface == WORKFLOW_EVENT_BUS_API_SURFACE)
    {
        return Err(boundary_error(
            WorkflowEventBusApiErrorCode::AuthorizationDenied,
            WorkflowEventBusApiStatus::Forbidden,
            "workflow-event-bus-api:surface-denied",
        ));
    }
    Ok(())
}

fn validate_publish_body_metadata(
    request: &WorkflowEventBusApiPublishRequest,
) -> Result<(), WorkflowEventBusApiError> {
    let body = &request.body;
    let invalid = !is_safe_ref(&body.cell_id)
        || !is_safe_ref(&body.residency_ref)
        || !is_safe_ref(&body.audit_chain_ref)
        || !is_safe_metadata(&body.event_kind)
        || !is_safe_ref(&body.producer_ref)
        || !is_safe_ref(&body.event_id)
        || !is_safe_ref(&body.source_ref)
        || !is_safe_optional_ref(body.subject_ref.as_deref())
        || !is_safe_optional_ref(body.time_ref.as_deref())
        || !is_safe_optional_ref(body.dataschema_ref.as_deref())
        || !is_safe_ref(&body.partition_key_ref)
        || !is_safe_ref(&body.publish_idempotency_key)
        || !is_safe_ref(&body.causation_ref)
        || !is_safe_ref(&body.correlation_ref)
        || !is_safe_ref(&body.payload_ref)
        || !body.evidence_refs.iter().all(|value| is_safe_ref(value));
    if invalid {
        return Err(boundary_error(
            WorkflowEventBusApiErrorCode::UnsafeMetadata,
            WorkflowEventBusApiStatus::BadRequest,
            "workflow-event-bus-api:unsafe-publish-body-metadata",
        ));
    }
    Ok(())
}

fn validate_delivery_body_metadata(
    request: &WorkflowEventBusApiDeliveryRequest,
) -> Result<(), WorkflowEventBusApiError> {
    let body = &request.body;
    let invalid = !is_safe_ref(&body.cell_id)
        || !is_safe_ref(&body.residency_ref)
        || !is_safe_ref(&body.audit_chain_ref)
        || !is_safe_metadata(&body.subscription_channel)
        || !is_safe_ref(&body.consumer_ref)
        || body.subscription_event_types.is_empty()
        || !body
            .subscription_event_types
            .iter()
            .all(|value| is_safe_metadata(value))
        || !is_safe_optional_ref(body.replay_cursor_ref.as_deref())
        || !is_safe_ref(&body.subscription_authorization_evidence_ref)
        || !is_safe_metadata(&body.candidate_channel)
        || !is_safe_ref(&body.candidate_event_id)
        || !is_safe_metadata(&body.candidate_event_type)
        || !is_safe_ref(&body.candidate_idempotency_key)
        || !is_safe_ref(&body.candidate_payload_ref)
        || !is_safe_ref(&body.candidate_offset_ref)
        || !body
            .candidate_evidence_refs
            .iter()
            .all(|value| is_safe_ref(value))
        || body.max_batch_size == 0
        || body.max_batch_size > WORKFLOW_EVENT_BUS_MAX_SUBSCRIPTION_BATCH_SIZE;
    if invalid {
        return Err(boundary_error(
            WorkflowEventBusApiErrorCode::UnsafeMetadata,
            WorkflowEventBusApiStatus::BadRequest,
            "workflow-event-bus-api:unsafe-delivery-body-metadata",
        ));
    }
    Ok(())
}

fn parse_event_kind(value: &str) -> Result<WorkflowEventBusEventKind, WorkflowEventBusApiError> {
    match value {
        "workflow-run-started" | "com.oyatie.workflow.run.started.v1" => {
            Ok(WorkflowEventBusEventKind::WorkflowRunStarted)
        }
        "workflow-step-dispatched" | "com.oyatie.workflow.step.dispatched.v1" => {
            Ok(WorkflowEventBusEventKind::WorkflowStepDispatched)
        }
        "workflow-state-transitioned" | "com.oyatie.workflow.state.transitioned.v1" => {
            Ok(WorkflowEventBusEventKind::WorkflowStateTransitioned)
        }
        "trigger-evaluated" | "com.oyatie.workflow.trigger.evaluated.v1" => {
            Ok(WorkflowEventBusEventKind::TriggerEvaluated)
        }
        "intelligence-draft-requested" | "com.oyatie.workflow.intelligence.draft_requested.v1" => {
            Ok(WorkflowEventBusEventKind::IntelligenceDraftRequested)
        }
        "ontology-projection-updated" | "com.oyatie.workflow.ontology.projection_updated.v1" => {
            Ok(WorkflowEventBusEventKind::OntologyProjectionUpdated)
        }
        _ => Err(boundary_error(
            WorkflowEventBusApiErrorCode::UnknownEventKind,
            WorkflowEventBusApiStatus::BadRequest,
            "workflow-event-bus-api:unknown-event-kind",
        )),
    }
}

fn parse_channel(value: &str) -> Result<WorkflowEventBusChannel, WorkflowEventBusApiError> {
    match value {
        "workflow-runs" | "workflow.runs.events.v1" => Ok(WorkflowEventBusChannel::WorkflowRuns),
        "workflow-steps" | "workflow.steps.events.v1" => Ok(WorkflowEventBusChannel::WorkflowSteps),
        "workflow-state" | "workflow.state.events.v1" => Ok(WorkflowEventBusChannel::WorkflowState),
        "trigger-events" | "workflow.triggers.events.v1" => {
            Ok(WorkflowEventBusChannel::TriggerEvents)
        }
        "intelligence-requests" | "workflow.intelligence.requests.v1" => {
            Ok(WorkflowEventBusChannel::IntelligenceRequests)
        }
        "ontology-projections" | "workflow.ontology.projections.v1" => {
            Ok(WorkflowEventBusChannel::OntologyProjections)
        }
        _ => Err(boundary_error(
            WorkflowEventBusApiErrorCode::UnknownChannel,
            WorkflowEventBusApiStatus::BadRequest,
            "workflow-event-bus-api:unknown-channel",
        )),
    }
}

fn parse_channels(
    values: &[String],
) -> Result<Vec<WorkflowEventBusChannel>, WorkflowEventBusApiError> {
    values.iter().map(|value| parse_channel(value)).collect()
}

fn response_metadata(
    boundary: &WorkflowEventBusApiBoundaryContext,
) -> WorkflowEventBusApiResponseMetadata {
    WorkflowEventBusApiResponseMetadata {
        request_id: boundary.request_id.clone(),
        tenant_id: boundary.tenant_id.clone(),
        idempotency_key: boundary.idempotency_key.clone(),
        trace_context_ref: boundary.trace_context_ref.clone(),
        surface: WORKFLOW_EVENT_BUS_API_SURFACE.to_owned(),
        contract_ref: WORKFLOW_EVENT_BUS_API_CONTRACT_REF.to_owned(),
        oyatie_version: WORKFLOW_EVENT_BUS_API_DECLARED_VERSION.to_owned(),
    }
}

fn idempotency_cache_key(boundary: &WorkflowEventBusApiBoundaryContext) -> String {
    format!("{}|{}", boundary.tenant_id, boundary.idempotency_key)
}

fn boundary_error(
    code: WorkflowEventBusApiErrorCode,
    status: WorkflowEventBusApiStatus,
    evidence_ref: &str,
) -> WorkflowEventBusApiError {
    WorkflowEventBusApiError::Boundary {
        code,
        status,
        evidence_ref: evidence_ref.to_owned(),
    }
}

fn domain_status_label(status: WorkflowEventBusDomainStatus) -> &'static str {
    match status {
        WorkflowEventBusDomainStatus::Accepted => "accepted",
        WorkflowEventBusDomainStatus::Denied => "denied",
    }
}

fn channel_asyncapi_ref(channel_address: &str) -> String {
    format!(
        "{}#/channels/{}",
        WORKFLOW_EVENT_BUS_ASYNCAPI_CONTRACT_REF,
        channel_address.replace('.', "_")
    )
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
    use std::collections::BTreeSet;

    #[test]
    fn api_status_and_error_codes_are_stable_and_unique() {
        let statuses = [
            WorkflowEventBusApiStatus::Accepted,
            WorkflowEventBusApiStatus::BadRequest,
            WorkflowEventBusApiStatus::Forbidden,
            WorkflowEventBusApiStatus::Conflict,
            WorkflowEventBusApiStatus::UnprocessableContent,
            WorkflowEventBusApiStatus::ServiceUnavailable,
        ];
        assert_eq!(statuses[0].code(), 202);
        let status_codes: BTreeSet<u16> = statuses.iter().map(|status| status.code()).collect();
        assert_eq!(status_codes.len(), statuses.len());

        let codes = [
            WorkflowEventBusApiErrorCode::AuthorizationDenied,
            WorkflowEventBusApiErrorCode::AuthorizationEvidenceInvalid,
            WorkflowEventBusApiErrorCode::AuthorizationPrincipalMismatch,
            WorkflowEventBusApiErrorCode::AuthorizationTenantMismatch,
            WorkflowEventBusApiErrorCode::ChannelMismatch,
            WorkflowEventBusApiErrorCode::ContractVersionUnsupported,
            WorkflowEventBusApiErrorCode::DeliveryDenied,
            WorkflowEventBusApiErrorCode::DomainDenied,
            WorkflowEventBusApiErrorCode::EventBusInvalid,
            WorkflowEventBusApiErrorCode::IdempotencyKeyReused,
            WorkflowEventBusApiErrorCode::MethodNotAllowed,
            WorkflowEventBusApiErrorCode::RequestIdEmpty,
            WorkflowEventBusApiErrorCode::RouteMismatch,
            WorkflowEventBusApiErrorCode::TenantBindingMismatch,
            WorkflowEventBusApiErrorCode::TenantHeaderEmpty,
            WorkflowEventBusApiErrorCode::TraceContextInvalid,
            WorkflowEventBusApiErrorCode::UnknownChannel,
            WorkflowEventBusApiErrorCode::UnknownEventKind,
            WorkflowEventBusApiErrorCode::UnsafeMetadata,
            WorkflowEventBusApiErrorCode::UsecaseUnavailable,
        ];
        let wire: Vec<&str> = codes.iter().map(|code| code.as_str()).collect();
        let unique: BTreeSet<&str> = wire.iter().copied().collect();
        assert_eq!(wire.len(), unique.len());
    }

    #[test]
    fn publish_event_accepts_authorized_request_and_returns_openapi_shaped_metadata() {
        let mut api = WorkflowEventBusApi::default();
        let response = api
            .publish_event(publish_request("idem:event-bus-api:publish:1"))
            .expect("published");

        assert_eq!(response.status, WorkflowEventBusApiStatus::Accepted);
        assert_eq!(response.http_status_code(), 202);
        assert_eq!(response.route, WORKFLOW_EVENT_BUS_API_PUBLISH_ROUTE);
        assert_eq!(response.event.operation, "publish");
        assert_eq!(response.event.usecase_status, "published");
        assert_eq!(response.event.domain_status.as_deref(), Some("accepted"));
        assert_eq!(
            response.event.event_type,
            "com.oyatie.workflow.run.started.v1"
        );
        assert_eq!(
            response.event.channel_address.as_deref(),
            Some("workflow.runs.events.v1")
        );
        assert!(
            response
                .event
                .delivery_key
                .as_deref()
                .unwrap()
                .contains("workflow.runs.events.v1")
        );
        assert_eq!(response.metadata.surface, WORKFLOW_EVENT_BUS_API_SURFACE);
        assert!(
            response
                .evidence_refs
                .contains(&WORKFLOW_EVENT_BUS_API_SURFACE.to_owned())
        );
        assert!(
            response
                .non_claim_refs
                .contains(&"workflow-event-bus-usecase:no-broker-runtime".to_owned())
        );
        assert!(
            response
                .non_claim_refs
                .contains(&"workflow-event-bus-usecase:no-hyperscaler-claim".to_owned())
        );
    }

    #[test]
    fn publish_replay_returns_cached_response_and_drift_conflicts_without_redelegating() {
        let mut api = WorkflowEventBusApi::default();
        let first_request = publish_request("idem:event-bus-api:publish-replay");
        let first = api.publish_event(first_request.clone()).unwrap();
        let second = api.publish_event(first_request.clone()).unwrap();
        assert_eq!(first, second);

        let mut drifted = first_request;
        drifted.body.event_id = "event:workflow-run-started:drift".to_owned();
        let error = api.publish_event(drifted).expect_err("conflict");
        assert_eq!(error.status(), WorkflowEventBusApiStatus::Conflict);
        assert_eq!(
            error.code(),
            WorkflowEventBusApiErrorCode::IdempotencyKeyReused
        );
    }

    #[test]
    fn boundary_route_version_and_authorization_fail_before_usecase() {
        let mut api = WorkflowEventBusApi::default();
        let mut version = publish_request("idem:event-bus-api:bad-version");
        version.boundary.oyatie_version = "2020-01-01".to_owned();
        assert_eq!(
            api.publish_event(version).unwrap_err().code(),
            WorkflowEventBusApiErrorCode::ContractVersionUnsupported
        );

        let mut route = publish_request("idem:event-bus-api:bad-route");
        route.route = "/wrong".to_owned();
        assert_eq!(
            api.publish_event(route).unwrap_err().code(),
            WorkflowEventBusApiErrorCode::RouteMismatch
        );

        let mut denied = publish_request("idem:event-bus-api:surface-denied");
        denied.authorization.allowed_surfaces.clear();
        assert_eq!(
            api.publish_event(denied).unwrap_err().code(),
            WorkflowEventBusApiErrorCode::AuthorizationDenied
        );
    }

    #[test]
    fn delivery_evaluate_accepts_and_delivery_denied_remains_success_without_offset_commit_claim() {
        let mut api = WorkflowEventBusApi::default();
        let accepted = api
            .evaluate_delivery(delivery_request("idem:event-bus-api:delivery:1"))
            .expect("accepted delivery");
        assert_eq!(accepted.status, WorkflowEventBusApiStatus::Accepted);
        assert_eq!(accepted.event.operation, "delivery-evaluate");
        assert_eq!(accepted.event.usecase_status, "delivery-accepted");
        assert_eq!(
            accepted.event.channel_address.as_deref(),
            Some("workflow.state.events.v1")
        );
        assert_eq!(
            accepted.event.consumer_ref.as_deref(),
            Some("consumer:workflow-state-machine")
        );

        let mut denied_request = delivery_request("idem:event-bus-api:delivery-denied");
        denied_request.body.candidate_channel = "workflow-runs".to_owned();
        denied_request.body.candidate_event_type = WorkflowEventBusEventKind::WorkflowRunStarted
            .event_type()
            .to_owned();
        let denied = api
            .evaluate_delivery(denied_request)
            .expect("denied delivery response");
        assert_eq!(denied.status, WorkflowEventBusApiStatus::Accepted);
        assert_eq!(denied.event.usecase_status, "delivery-denied");
        assert!(
            denied
                .evidence_refs
                .contains(&"workflow-event-bus-kernel:channel-not-subscribed".to_owned())
        );
        assert!(
            denied
                .non_claim_refs
                .contains(&"workflow-event-bus-usecase:no-offset-commit-runtime".to_owned())
        );
    }

    #[test]
    fn domain_denial_maps_to_forbidden_problem_without_raw_echo() {
        let mut api = WorkflowEventBusApi::default();
        let mut request = publish_request("idem:event-bus-api:domain-denied");
        request.authorization.allowed_event_types = vec![
            WorkflowEventBusEventKind::WorkflowStepDispatched
                .event_type()
                .to_owned(),
        ];

        let error = api.publish_event(request).unwrap_err();
        assert_eq!(error.status(), WorkflowEventBusApiStatus::Forbidden);
        assert_eq!(error.code(), WorkflowEventBusApiErrorCode::DomainDenied);
        let problem = error.problem();
        assert_eq!(problem.status, 403);
        assert!(
            problem
                .evidence_refs
                .contains(&"workflow-event-bus-domain:publish-not-authorized".to_owned())
        );
        assert!(!format!("{problem:?}").contains("raw prompt"));
    }

    #[test]
    fn unsafe_body_metadata_is_rejected_without_echoing_payload_or_secret() {
        let mut api = WorkflowEventBusApi::default();
        let mut request = publish_request("idem:event-bus-api:unsafe");
        request.body.payload_ref =
            "raw payload Authorization: Bearer sk-test customer message".to_owned();

        let error = api.publish_event(request).unwrap_err();
        assert_eq!(error.status(), WorkflowEventBusApiStatus::BadRequest);
        assert_eq!(error.code(), WorkflowEventBusApiErrorCode::UnsafeMetadata);
        let rendered = format!("{:?}", error.problem());
        assert!(!rendered.contains("raw payload"));
        assert!(!rendered.contains("Authorization"));
        assert!(!rendered.contains("sk-test"));
        assert!(!rendered.contains("customer message"));
    }

    fn publish_request(idempotency_key: &str) -> WorkflowEventBusApiPublishRequest {
        WorkflowEventBusApiPublishRequest {
            boundary: boundary(idempotency_key),
            principal: principal(),
            authorization: authorization(),
            method: WORKFLOW_EVENT_BUS_API_METHOD.to_owned(),
            route: WORKFLOW_EVENT_BUS_API_PUBLISH_ROUTE.to_owned(),
            body: WorkflowEventBusApiPublishBody {
                cell_id: "cell:us-east-1a".to_owned(),
                residency_ref: "residency:us:data-plane".to_owned(),
                audit_chain_ref: "audit-chain:event-bus-api".to_owned(),
                event_kind: "workflow-run-started".to_owned(),
                producer_ref: "producer:workflow-engine:execution".to_owned(),
                event_id: "event:workflow-run-started:001".to_owned(),
                source_ref: "urn:oyatie:workflow-engine:execution".to_owned(),
                subject_ref: Some("subject:workflow-run:001".to_owned()),
                time_ref: Some("time:2026-05-25T00:00:00Z".to_owned()),
                dataschema_ref: Some("schema:workflow-event-run-started".to_owned()),
                partition_key_ref: "partition:tenant-workflow-run".to_owned(),
                publish_idempotency_key: "idem:event-bus-domain:publish:001".to_owned(),
                causation_ref: "cause:execution-engine:start-run".to_owned(),
                correlation_ref: "corr:workflow-run:001".to_owned(),
                payload_ref: "body-ref:workflow-run-started".to_owned(),
                evidence_refs: vec!["evidence:event-bus-api:publish".to_owned()],
            },
        }
    }

    fn delivery_request(idempotency_key: &str) -> WorkflowEventBusApiDeliveryRequest {
        WorkflowEventBusApiDeliveryRequest {
            boundary: boundary(idempotency_key),
            principal: principal(),
            authorization: authorization(),
            method: WORKFLOW_EVENT_BUS_API_METHOD.to_owned(),
            route: WORKFLOW_EVENT_BUS_API_DELIVERY_ROUTE.to_owned(),
            body: WorkflowEventBusApiDeliveryBody {
                cell_id: "cell:us-east-1a".to_owned(),
                residency_ref: "residency:us:data-plane".to_owned(),
                audit_chain_ref: "audit-chain:event-bus-api".to_owned(),
                subscription_channel: "workflow-state".to_owned(),
                consumer_ref: "consumer:workflow-state-machine".to_owned(),
                subscription_event_types: vec![
                    WorkflowEventBusEventKind::WorkflowStateTransitioned
                        .event_type()
                        .to_owned(),
                ],
                replay_cursor_ref: Some("cursor:event-bus-api:state".to_owned()),
                max_batch_size: 100,
                subscription_authorization_evidence_ref: "authz:event-bus-api:consume".to_owned(),
                candidate_channel: "workflow-state".to_owned(),
                candidate_event_id: "event:workflow-state:001".to_owned(),
                candidate_event_type: WorkflowEventBusEventKind::WorkflowStateTransitioned
                    .event_type()
                    .to_owned(),
                candidate_idempotency_key: "idem:event-bus-domain:delivery:001".to_owned(),
                candidate_payload_ref: "body-ref:workflow-state-transitioned".to_owned(),
                candidate_offset_ref: "offset:partition-0:42".to_owned(),
                candidate_evidence_refs: vec!["evidence:event-bus-api:delivery".to_owned()],
            },
        }
    }

    fn boundary(idempotency_key: &str) -> WorkflowEventBusApiBoundaryContext {
        WorkflowEventBusApiBoundaryContext {
            request_id: format!("request:event-bus-api:{idempotency_key}"),
            tenant_id: "ten_workflow_event_bus".to_owned(),
            idempotency_key: idempotency_key.to_owned(),
            trace_context_ref: "trace:event-bus-api".to_owned(),
            oyatie_version: WORKFLOW_EVENT_BUS_API_DECLARED_VERSION.to_owned(),
        }
    }

    fn principal() -> WorkflowEventBusApiPrincipal {
        WorkflowEventBusApiPrincipal {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            principal_id: "principal:workflow-operator".to_owned(),
        }
    }

    fn authorization() -> WorkflowEventBusApiAuthorization {
        WorkflowEventBusApiAuthorization {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            principal_id: "principal:workflow-operator".to_owned(),
            decision_id: "policy-decision:event-bus-allow".to_owned(),
            evidence_ref: "policy-evidence:event-bus-allow".to_owned(),
            policy_bundle_ref: "policy-bundle:event-bus-v1".to_owned(),
            allowed_surfaces: vec![WORKFLOW_EVENT_BUS_API_SURFACE.to_owned()],
            allowed_channels: vec![
                "workflow-runs".to_owned(),
                "workflow-state".to_owned(),
                "trigger-events".to_owned(),
                "intelligence-requests".to_owned(),
                "ontology-projections".to_owned(),
            ],
            allowed_event_types: vec![
                WorkflowEventBusEventKind::WorkflowRunStarted
                    .event_type()
                    .to_owned(),
                WorkflowEventBusEventKind::WorkflowStateTransitioned
                    .event_type()
                    .to_owned(),
                WorkflowEventBusEventKind::TriggerEvaluated
                    .event_type()
                    .to_owned(),
                WorkflowEventBusEventKind::IntelligenceDraftRequested
                    .event_type()
                    .to_owned(),
                WorkflowEventBusEventKind::OntologyProjectionUpdated
                    .event_type()
                    .to_owned(),
            ],
        }
    }
}
