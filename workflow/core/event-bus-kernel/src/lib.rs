//! Workflow-engine event-bus kernel foundation.
//!
//! This crate provides deterministic, source-level event-bus planning for the
//! workflow runtime. It models CloudEvents-shaped workflow event envelopes,
//! AsyncAPI-shaped channel/message refs, tenant/cell/residency binding,
//! idempotent publish delivery keys, subscription delivery decisions, and
//! redaction-safe non-claim metadata for later Postgres/Valkey/NATS/Kafka and
//! Oyatie Cloud tenant workload integration. It performs no broker connection,
//! topic creation, network I/O, serialization-framework work, durable outbox or
//! inbox writes, consumer group coordination, offset commits, payload
//! materialization, signing, filesystem access, random/UUID generation,
//! wall-clock reads, Kubernetes calls, cloud deployment, or tenant workload
//! scheduling.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub const WORKFLOW_EVENT_BUS_KERNEL_SURFACE: &str = "workflow-engine.event-bus.kernel";
pub const WORKFLOW_EVENT_BUS_CLOUDEVENTS_SPECVERSION: &str = "1.0";
pub const WORKFLOW_EVENT_BUS_DEFAULT_CONTENT_TYPE: &str = "application/json";
pub const WORKFLOW_EVENT_BUS_ASYNCAPI_CONTRACT_REF: &str =
    "workflow/workflow-engine/contracts/asyncapi/workflow-events.yaml";
pub const WORKFLOW_EVENT_BUS_MAX_SUBSCRIPTION_BATCH_SIZE: u32 = 1000;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkflowEventBusChannel {
    WorkflowRuns,
    WorkflowSteps,
    WorkflowState,
    TriggerEvents,
    IntelligenceRequests,
    OntologyProjections,
}

impl WorkflowEventBusChannel {
    pub const fn address(self) -> &'static str {
        match self {
            Self::WorkflowRuns => "workflow.runs.events.v1",
            Self::WorkflowSteps => "workflow.steps.events.v1",
            Self::WorkflowState => "workflow.state.events.v1",
            Self::TriggerEvents => "workflow.triggers.events.v1",
            Self::IntelligenceRequests => "workflow.intelligence.requests.v1",
            Self::OntologyProjections => "workflow.ontology.projections.v1",
        }
    }

    pub fn asyncapi_ref(self) -> String {
        format!(
            "{}#/channels/{}",
            WORKFLOW_EVENT_BUS_ASYNCAPI_CONTRACT_REF,
            self.address().replace('.', "_")
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkflowEventBusEventKind {
    WorkflowRunStarted,
    WorkflowStepDispatched,
    WorkflowStateTransitioned,
    TriggerEvaluated,
    IntelligenceDraftRequested,
    OntologyProjectionUpdated,
}

impl WorkflowEventBusEventKind {
    pub const fn event_type(self) -> &'static str {
        match self {
            Self::WorkflowRunStarted => "com.oyatie.workflow.run.started.v1",
            Self::WorkflowStepDispatched => "com.oyatie.workflow.step.dispatched.v1",
            Self::WorkflowStateTransitioned => "com.oyatie.workflow.state.transitioned.v1",
            Self::TriggerEvaluated => "com.oyatie.workflow.trigger.evaluated.v1",
            Self::IntelligenceDraftRequested => {
                "com.oyatie.workflow.intelligence.draft_requested.v1"
            }
            Self::OntologyProjectionUpdated => "com.oyatie.workflow.ontology.projection_updated.v1",
        }
    }

    pub const fn channel(self) -> WorkflowEventBusChannel {
        match self {
            Self::WorkflowRunStarted => WorkflowEventBusChannel::WorkflowRuns,
            Self::WorkflowStepDispatched => WorkflowEventBusChannel::WorkflowSteps,
            Self::WorkflowStateTransitioned => WorkflowEventBusChannel::WorkflowState,
            Self::TriggerEvaluated => WorkflowEventBusChannel::TriggerEvents,
            Self::IntelligenceDraftRequested => WorkflowEventBusChannel::IntelligenceRequests,
            Self::OntologyProjectionUpdated => WorkflowEventBusChannel::OntologyProjections,
        }
    }

    pub fn asyncapi_message_ref(self) -> String {
        format!(
            "{}#/channels/{}/messages/{}",
            WORKFLOW_EVENT_BUS_ASYNCAPI_CONTRACT_REF,
            self.channel().address().replace('.', "_"),
            self.event_type().replace('.', "_")
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusContext {
    pub tenant_id: String,           // data_class: INTERNAL_ONLY
    pub cell_id: String,             // data_class: INTERNAL_ONLY
    pub producer_ref: String,        // data_class: INTERNAL_ONLY
    pub trace_context_ref: String,   // data_class: INTERNAL_ONLY
    pub policy_decision_ref: String, // data_class: INTERNAL_ONLY
    pub residency_ref: String,       // data_class: INTERNAL_ONLY
    pub audit_chain_ref: String,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusCloudEvent {
    pub specversion: String,            // data_class: PUBLIC
    pub id: String,                     // data_class: INTERNAL_ONLY
    pub source: String,                 // data_class: INTERNAL_ONLY
    pub event_type: String,             // data_class: PUBLIC
    pub subject_ref: Option<String>,    // data_class: INTERNAL_ONLY
    pub time_ref: Option<String>,       // data_class: INTERNAL_ONLY
    pub datacontenttype: String,        // data_class: PUBLIC
    pub dataschema_ref: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusPublishRequest {
    pub context: WorkflowEventBusContext, // data_class: INTERNAL_ONLY
    pub channel: WorkflowEventBusChannel, // data_class: PUBLIC
    pub event: WorkflowEventBusCloudEvent, // data_class: INTERNAL_ONLY
    pub partition_key_ref: String,        // data_class: INTERNAL_ONLY
    pub idempotency_key: String,          // data_class: INTERNAL_ONLY
    pub causation_ref: String,            // data_class: INTERNAL_ONLY
    pub correlation_ref: String,          // data_class: INTERNAL_ONLY
    pub payload_ref: String,              // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusPublishPlan {
    pub channel: WorkflowEventBusChannel, // data_class: PUBLIC
    pub channel_address: String,          // data_class: PUBLIC
    pub event_type: String,               // data_class: PUBLIC
    pub event_id: String,                 // data_class: INTERNAL_ONLY
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub cell_id: String,                  // data_class: INTERNAL_ONLY
    pub partition_key_ref: String,        // data_class: INTERNAL_ONLY
    pub idempotency_key: String,          // data_class: INTERNAL_ONLY
    pub delivery_key: String,             // data_class: INTERNAL_ONLY
    pub asyncapi_channel_ref: String,     // data_class: INTERNAL_ONLY
    pub asyncapi_message_ref: String,     // data_class: INTERNAL_ONLY
    pub payload_ref: String,              // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,       // data_class: INTERNAL_ONLY
    pub non_claim_refs: Vec<String>,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusSubscription {
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub cell_id: String,                    // data_class: INTERNAL_ONLY
    pub consumer_ref: String,               // data_class: INTERNAL_ONLY
    pub channel: WorkflowEventBusChannel,   // data_class: PUBLIC
    pub allowed_event_types: Vec<String>,   // data_class: INTERNAL_ONLY
    pub replay_cursor_ref: Option<String>,  // data_class: INTERNAL_ONLY
    pub max_batch_size: u32,                // data_class: INTERNAL_ONLY
    pub authorization_evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusDeliveryCandidate {
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub cell_id: String,                  // data_class: INTERNAL_ONLY
    pub channel: WorkflowEventBusChannel, // data_class: PUBLIC
    pub event_id: String,                 // data_class: INTERNAL_ONLY
    pub event_type: String,               // data_class: PUBLIC
    pub idempotency_key: String,          // data_class: INTERNAL_ONLY
    pub payload_ref: String,              // data_class: INTERNAL_ONLY
    pub offset_ref: String,               // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkflowEventBusDeliveryStatus {
    Accepted,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusDeliveryDecision {
    pub status: WorkflowEventBusDeliveryStatus, // data_class: PUBLIC
    pub consumer_ref: String,                   // data_class: INTERNAL_ONLY
    pub event_id: String,                       // data_class: INTERNAL_ONLY
    pub event_type: String,                     // data_class: PUBLIC
    pub channel_address: String,                // data_class: PUBLIC
    pub offset_ref: String,                     // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,             // data_class: INTERNAL_ONLY
    pub non_claim_refs: Vec<String>,            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkflowEventBusKernelError {
    InvalidContext { evidence_ref: String },
    InvalidCloudEvent { evidence_ref: String },
    InvalidPublishRequest { evidence_ref: String },
    InvalidSubscription { evidence_ref: String },
    InvalidDeliveryCandidate { evidence_ref: String },
    ChannelEventMismatch { evidence_ref: String },
    TenantOrCellMismatch { evidence_ref: String },
    UnsupportedEventType { evidence_ref: String },
    UnsafeMetadata { evidence_ref: String },
}

impl WorkflowEventBusKernelError {
    pub fn primary_evidence_ref(&self) -> &str {
        match self {
            Self::InvalidContext { evidence_ref }
            | Self::InvalidCloudEvent { evidence_ref }
            | Self::InvalidPublishRequest { evidence_ref }
            | Self::InvalidSubscription { evidence_ref }
            | Self::InvalidDeliveryCandidate { evidence_ref }
            | Self::ChannelEventMismatch { evidence_ref }
            | Self::TenantOrCellMismatch { evidence_ref }
            | Self::UnsupportedEventType { evidence_ref }
            | Self::UnsafeMetadata { evidence_ref } => evidence_ref,
        }
    }
}

pub fn plan_publish(
    request: WorkflowEventBusPublishRequest,
) -> Result<WorkflowEventBusPublishPlan, WorkflowEventBusKernelError> {
    validate_context(&request.context)?;
    validate_cloud_event(&request.event)?;
    validate_publish_request_refs(&request)?;
    let kind = event_kind_from_type(&request.event.event_type)?;
    if kind.channel() != request.channel {
        return Err(WorkflowEventBusKernelError::ChannelEventMismatch {
            evidence_ref: "workflow-event-bus-kernel:channel-event-mismatch".to_owned(),
        });
    }

    Ok(WorkflowEventBusPublishPlan {
        channel: request.channel,
        channel_address: request.channel.address().to_owned(),
        event_type: request.event.event_type.clone(),
        event_id: request.event.id.clone(),
        tenant_id: request.context.tenant_id.clone(),
        cell_id: request.context.cell_id.clone(),
        partition_key_ref: request.partition_key_ref.clone(),
        idempotency_key: request.idempotency_key.clone(),
        delivery_key: delivery_key(
            &request.context.tenant_id,
            request.channel,
            &request.event.id,
            &request.idempotency_key,
        ),
        asyncapi_channel_ref: request.channel.asyncapi_ref(),
        asyncapi_message_ref: kind.asyncapi_message_ref(),
        payload_ref: request.payload_ref.clone(),
        evidence_refs: sorted_unique(
            [
                request.evidence_refs,
                vec![
                    WORKFLOW_EVENT_BUS_KERNEL_SURFACE.to_owned(),
                    request.context.policy_decision_ref,
                    request.context.audit_chain_ref,
                    request.causation_ref,
                    request.correlation_ref,
                    "workflow-event-bus-kernel:publish-plan-built".to_owned(),
                ],
            ]
            .concat(),
        ),
        non_claim_refs: source_level_non_claim_refs(),
    })
}

pub fn evaluate_delivery(
    subscription: WorkflowEventBusSubscription,
    candidate: WorkflowEventBusDeliveryCandidate,
) -> Result<WorkflowEventBusDeliveryDecision, WorkflowEventBusKernelError> {
    validate_subscription(&subscription)?;
    validate_delivery_candidate(&candidate)?;
    if subscription.tenant_id != candidate.tenant_id || subscription.cell_id != candidate.cell_id {
        return Err(WorkflowEventBusKernelError::TenantOrCellMismatch {
            evidence_ref: "workflow-event-bus-kernel:tenant-cell-delivery-mismatch".to_owned(),
        });
    }
    if subscription.channel != candidate.channel {
        return Ok(delivery_decision(
            WorkflowEventBusDeliveryStatus::Rejected,
            &subscription,
            &candidate,
            vec!["workflow-event-bus-kernel:channel-not-subscribed".to_owned()],
        ));
    }
    if !subscription
        .allowed_event_types
        .iter()
        .any(|event_type| event_type == &candidate.event_type)
    {
        return Ok(delivery_decision(
            WorkflowEventBusDeliveryStatus::Rejected,
            &subscription,
            &candidate,
            vec!["workflow-event-bus-kernel:event-type-not-allowed".to_owned()],
        ));
    }
    event_kind_from_type(&candidate.event_type)?;
    Ok(delivery_decision(
        WorkflowEventBusDeliveryStatus::Accepted,
        &subscription,
        &candidate,
        vec!["workflow-event-bus-kernel:delivery-accepted".to_owned()],
    ))
}

fn delivery_decision(
    status: WorkflowEventBusDeliveryStatus,
    subscription: &WorkflowEventBusSubscription,
    candidate: &WorkflowEventBusDeliveryCandidate,
    evidence_refs: Vec<String>,
) -> WorkflowEventBusDeliveryDecision {
    WorkflowEventBusDeliveryDecision {
        status,
        consumer_ref: subscription.consumer_ref.clone(),
        event_id: candidate.event_id.clone(),
        event_type: candidate.event_type.clone(),
        channel_address: candidate.channel.address().to_owned(),
        offset_ref: candidate.offset_ref.clone(),
        evidence_refs: sorted_unique([candidate.evidence_refs.clone(), evidence_refs].concat()),
        non_claim_refs: source_level_non_claim_refs(),
    }
}

fn validate_context(context: &WorkflowEventBusContext) -> Result<(), WorkflowEventBusKernelError> {
    if !is_safe_tenant(&context.tenant_id) {
        return Err(WorkflowEventBusKernelError::InvalidContext {
            evidence_ref: "workflow-event-bus-kernel:tenant-invalid".to_owned(),
        });
    }
    for value in [
        &context.cell_id,
        &context.producer_ref,
        &context.trace_context_ref,
        &context.policy_decision_ref,
        &context.residency_ref,
        &context.audit_chain_ref,
    ] {
        if !is_safe_ref(value) {
            return Err(WorkflowEventBusKernelError::InvalidContext {
                evidence_ref: "workflow-event-bus-kernel:context-ref-invalid".to_owned(),
            });
        }
    }
    Ok(())
}

fn validate_cloud_event(
    event: &WorkflowEventBusCloudEvent,
) -> Result<(), WorkflowEventBusKernelError> {
    if event.specversion != WORKFLOW_EVENT_BUS_CLOUDEVENTS_SPECVERSION {
        return Err(WorkflowEventBusKernelError::InvalidCloudEvent {
            evidence_ref: "workflow-event-bus-kernel:cloudevents-version-unsupported".to_owned(),
        });
    }
    if event.datacontenttype != WORKFLOW_EVENT_BUS_DEFAULT_CONTENT_TYPE {
        return Err(WorkflowEventBusKernelError::InvalidCloudEvent {
            evidence_ref: "workflow-event-bus-kernel:content-type-unsupported".to_owned(),
        });
    }
    if !is_safe_ref(&event.id)
        || !is_safe_ref(&event.source)
        || !is_safe_metadata(&event.event_type)
        || !is_safe_optional_ref(event.subject_ref.as_deref())
        || !is_safe_optional_ref(event.time_ref.as_deref())
        || !is_safe_optional_ref(event.dataschema_ref.as_deref())
    {
        return Err(WorkflowEventBusKernelError::InvalidCloudEvent {
            evidence_ref: "workflow-event-bus-kernel:cloudevent-ref-invalid".to_owned(),
        });
    }
    event_kind_from_type(&event.event_type)?;
    Ok(())
}

fn validate_publish_request_refs(
    request: &WorkflowEventBusPublishRequest,
) -> Result<(), WorkflowEventBusKernelError> {
    for value in [
        &request.partition_key_ref,
        &request.idempotency_key,
        &request.causation_ref,
        &request.correlation_ref,
        &request.payload_ref,
    ] {
        if !is_safe_ref(value) {
            return Err(WorkflowEventBusKernelError::InvalidPublishRequest {
                evidence_ref: "workflow-event-bus-kernel:publish-ref-invalid".to_owned(),
            });
        }
    }
    if request
        .evidence_refs
        .iter()
        .any(|value| !is_safe_ref(value))
    {
        return Err(WorkflowEventBusKernelError::UnsafeMetadata {
            evidence_ref: "workflow-event-bus-kernel:evidence-ref-invalid".to_owned(),
        });
    }
    Ok(())
}

fn validate_subscription(
    subscription: &WorkflowEventBusSubscription,
) -> Result<(), WorkflowEventBusKernelError> {
    if !is_safe_tenant(&subscription.tenant_id)
        || !is_safe_ref(&subscription.cell_id)
        || !is_safe_ref(&subscription.consumer_ref)
        || !is_safe_optional_ref(subscription.replay_cursor_ref.as_deref())
        || !is_safe_ref(&subscription.authorization_evidence_ref)
    {
        return Err(WorkflowEventBusKernelError::InvalidSubscription {
            evidence_ref: "workflow-event-bus-kernel:subscription-ref-invalid".to_owned(),
        });
    }
    if subscription.max_batch_size == 0
        || subscription.max_batch_size > WORKFLOW_EVENT_BUS_MAX_SUBSCRIPTION_BATCH_SIZE
    {
        return Err(WorkflowEventBusKernelError::InvalidSubscription {
            evidence_ref: "workflow-event-bus-kernel:subscription-batch-invalid".to_owned(),
        });
    }
    if subscription.allowed_event_types.is_empty()
        || subscription
            .allowed_event_types
            .iter()
            .any(|event_type| event_kind_from_type(event_type).is_err())
    {
        return Err(WorkflowEventBusKernelError::UnsupportedEventType {
            evidence_ref: "workflow-event-bus-kernel:subscription-event-type-unsupported"
                .to_owned(),
        });
    }
    Ok(())
}

fn validate_delivery_candidate(
    candidate: &WorkflowEventBusDeliveryCandidate,
) -> Result<(), WorkflowEventBusKernelError> {
    if !is_safe_tenant(&candidate.tenant_id)
        || !is_safe_ref(&candidate.cell_id)
        || !is_safe_ref(&candidate.event_id)
        || !is_safe_ref(&candidate.idempotency_key)
        || !is_safe_ref(&candidate.payload_ref)
        || !is_safe_ref(&candidate.offset_ref)
    {
        return Err(WorkflowEventBusKernelError::InvalidDeliveryCandidate {
            evidence_ref: "workflow-event-bus-kernel:delivery-candidate-ref-invalid".to_owned(),
        });
    }
    if candidate
        .evidence_refs
        .iter()
        .any(|value| !is_safe_ref(value))
    {
        return Err(WorkflowEventBusKernelError::UnsafeMetadata {
            evidence_ref: "workflow-event-bus-kernel:delivery-evidence-ref-invalid".to_owned(),
        });
    }
    event_kind_from_type(&candidate.event_type)?;
    Ok(())
}

fn event_kind_from_type(
    event_type: &str,
) -> Result<WorkflowEventBusEventKind, WorkflowEventBusKernelError> {
    match event_type {
        value if value == WorkflowEventBusEventKind::WorkflowRunStarted.event_type() => {
            Ok(WorkflowEventBusEventKind::WorkflowRunStarted)
        }
        value if value == WorkflowEventBusEventKind::WorkflowStepDispatched.event_type() => {
            Ok(WorkflowEventBusEventKind::WorkflowStepDispatched)
        }
        value if value == WorkflowEventBusEventKind::WorkflowStateTransitioned.event_type() => {
            Ok(WorkflowEventBusEventKind::WorkflowStateTransitioned)
        }
        value if value == WorkflowEventBusEventKind::TriggerEvaluated.event_type() => {
            Ok(WorkflowEventBusEventKind::TriggerEvaluated)
        }
        value if value == WorkflowEventBusEventKind::IntelligenceDraftRequested.event_type() => {
            Ok(WorkflowEventBusEventKind::IntelligenceDraftRequested)
        }
        value if value == WorkflowEventBusEventKind::OntologyProjectionUpdated.event_type() => {
            Ok(WorkflowEventBusEventKind::OntologyProjectionUpdated)
        }
        _ => Err(WorkflowEventBusKernelError::UnsupportedEventType {
            evidence_ref: "workflow-event-bus-kernel:event-type-unsupported".to_owned(),
        }),
    }
}

fn delivery_key(
    tenant_id: &str,
    channel: WorkflowEventBusChannel,
    event_id: &str,
    idempotency_key: &str,
) -> String {
    format!(
        "delivery-key:{}:{}:{}:{}",
        tenant_id,
        channel.address(),
        event_id,
        idempotency_key
    )
}

fn source_level_non_claim_refs() -> Vec<String> {
    sorted_unique(vec![
        "workflow-event-bus-kernel:source-only-plan".to_owned(),
        "workflow-event-bus-kernel:no-broker-connection".to_owned(),
        "workflow-event-bus-kernel:no-durable-outbox-inbox".to_owned(),
        "workflow-event-bus-kernel:no-cloud-runtime-deployment".to_owned(),
        "workflow-event-bus-kernel:no-hyperscaler-claim".to_owned(),
    ])
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
    values.retain(|value| !value.trim().is_empty() && !contains_raw_secret_material(value));
    values.sort();
    values.dedup();
    values
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn context() -> WorkflowEventBusContext {
        WorkflowEventBusContext {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            cell_id: "cell:us-east-1a".to_owned(),
            producer_ref: "producer:workflow-engine:execution".to_owned(),
            trace_context_ref: "trace:workflow-event-bus:root".to_owned(),
            policy_decision_ref: "policy-decision:event-publish-allowed".to_owned(),
            residency_ref: "residency:us:data-plane".to_owned(),
            audit_chain_ref: "audit-chain:workflow-event-bus".to_owned(),
        }
    }

    fn cloud_event(kind: WorkflowEventBusEventKind) -> WorkflowEventBusCloudEvent {
        WorkflowEventBusCloudEvent {
            specversion: WORKFLOW_EVENT_BUS_CLOUDEVENTS_SPECVERSION.to_owned(),
            id: "event:workflow-run-started:001".to_owned(),
            source: "urn:oyatie:workflow-engine:execution".to_owned(),
            event_type: kind.event_type().to_owned(),
            subject_ref: Some("subject:workflow-run:001".to_owned()),
            time_ref: Some("time:2026-05-25T00:00:00Z".to_owned()),
            datacontenttype: WORKFLOW_EVENT_BUS_DEFAULT_CONTENT_TYPE.to_owned(),
            dataschema_ref: Some("schema:workflow-event-run-started".to_owned()),
        }
    }

    fn publish_request(kind: WorkflowEventBusEventKind) -> WorkflowEventBusPublishRequest {
        WorkflowEventBusPublishRequest {
            context: context(),
            channel: kind.channel(),
            event: cloud_event(kind),
            partition_key_ref: "partition:tenant-workflow-run".to_owned(),
            idempotency_key: "idem:workflow-event-bus:publish:001".to_owned(),
            causation_ref: "cause:execution-engine:start-run".to_owned(),
            correlation_ref: "corr:workflow-run:001".to_owned(),
            payload_ref: "body-ref:workflow-run-started".to_owned(),
            evidence_refs: vec![
                "evidence:workflow-event-bus:publish".to_owned(),
                "evidence:workflow-event-bus:publish".to_owned(),
            ],
        }
    }

    fn subscription() -> WorkflowEventBusSubscription {
        WorkflowEventBusSubscription {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            cell_id: "cell:us-east-1a".to_owned(),
            consumer_ref: "consumer:workflow-state-machine".to_owned(),
            channel: WorkflowEventBusChannel::WorkflowState,
            allowed_event_types: vec![
                WorkflowEventBusEventKind::WorkflowStateTransitioned
                    .event_type()
                    .to_owned(),
            ],
            replay_cursor_ref: Some("cursor:workflow-state:001".to_owned()),
            max_batch_size: 100,
            authorization_evidence_ref: "authz:event-bus:consume".to_owned(),
        }
    }

    fn candidate(kind: WorkflowEventBusEventKind) -> WorkflowEventBusDeliveryCandidate {
        WorkflowEventBusDeliveryCandidate {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            cell_id: "cell:us-east-1a".to_owned(),
            channel: kind.channel(),
            event_id: "event:workflow-state:001".to_owned(),
            event_type: kind.event_type().to_owned(),
            idempotency_key: "idem:workflow-event-bus:delivery:001".to_owned(),
            payload_ref: "body-ref:workflow-state-transitioned".to_owned(),
            offset_ref: "offset:partition-0:42".to_owned(),
            evidence_refs: vec!["evidence:workflow-event-bus:delivery".to_owned()],
        }
    }

    #[test]
    fn event_kind_labels_channels_and_asyncapi_refs_are_stable_and_unique() {
        let kinds = [
            WorkflowEventBusEventKind::WorkflowRunStarted,
            WorkflowEventBusEventKind::WorkflowStepDispatched,
            WorkflowEventBusEventKind::WorkflowStateTransitioned,
            WorkflowEventBusEventKind::TriggerEvaluated,
            WorkflowEventBusEventKind::IntelligenceDraftRequested,
            WorkflowEventBusEventKind::OntologyProjectionUpdated,
        ];
        let event_types: BTreeSet<_> = kinds.iter().map(|kind| kind.event_type()).collect();
        let message_refs: BTreeSet<_> = kinds
            .iter()
            .map(|kind| kind.asyncapi_message_ref())
            .collect();
        assert_eq!(event_types.len(), kinds.len());
        assert_eq!(message_refs.len(), kinds.len());
        assert!(
            WorkflowEventBusEventKind::TriggerEvaluated
                .asyncapi_message_ref()
                .contains("workflow_triggers_events_v1")
        );
    }

    #[test]
    fn publish_plan_binds_cloudevents_asyncapi_tenant_cell_and_idempotency_metadata() {
        let plan = plan_publish(publish_request(
            WorkflowEventBusEventKind::WorkflowRunStarted,
        ))
        .expect("publish plan");
        assert_eq!(plan.channel, WorkflowEventBusChannel::WorkflowRuns);
        assert_eq!(plan.channel_address, "workflow.runs.events.v1");
        assert_eq!(
            plan.event_type,
            WorkflowEventBusEventKind::WorkflowRunStarted.event_type()
        );
        assert_eq!(plan.tenant_id, "ten_workflow_event_bus");
        assert!(plan.delivery_key.contains("workflow.runs.events.v1"));
        assert!(plan.asyncapi_channel_ref.contains("#/channels/"));
        assert!(plan.asyncapi_message_ref.contains("/messages/"));
        assert_eq!(
            plan.evidence_refs
                .iter()
                .filter(|value| *value == "evidence:workflow-event-bus:publish")
                .count(),
            1
        );
        assert!(
            plan.non_claim_refs
                .contains(&"workflow-event-bus-kernel:no-broker-connection".to_owned())
        );
    }

    #[test]
    fn invalid_cloudevents_channel_mismatch_and_raw_material_are_denied_without_echo() {
        let mut unsupported_version =
            publish_request(WorkflowEventBusEventKind::WorkflowRunStarted);
        unsupported_version.event.specversion = "0.3".to_owned();
        let error = plan_publish(unsupported_version).expect_err("bad cloudevents version");
        assert_eq!(
            error.primary_evidence_ref(),
            "workflow-event-bus-kernel:cloudevents-version-unsupported"
        );

        let mismatch = WorkflowEventBusPublishRequest {
            channel: WorkflowEventBusChannel::WorkflowSteps,
            ..publish_request(WorkflowEventBusEventKind::WorkflowRunStarted)
        };
        let error = plan_publish(mismatch).expect_err("channel mismatch");
        assert_eq!(
            error.primary_evidence_ref(),
            "workflow-event-bus-kernel:channel-event-mismatch"
        );

        let raw_secret = WorkflowEventBusPublishRequest {
            payload_ref: "raw payload bearer sk-test customer message".to_owned(),
            ..publish_request(WorkflowEventBusEventKind::WorkflowRunStarted)
        };
        let error = plan_publish(raw_secret).expect_err("raw payload denied");
        assert_eq!(
            error.primary_evidence_ref(),
            "workflow-event-bus-kernel:publish-ref-invalid"
        );
        assert!(!format!("{error:?}").contains("sk-test"));
    }

    #[test]
    fn subscription_accepts_allowed_delivery_and_rejects_unsubscribed_event_type() {
        let accepted = evaluate_delivery(
            subscription(),
            candidate(WorkflowEventBusEventKind::WorkflowStateTransitioned),
        )
        .expect("accepted delivery");
        assert_eq!(accepted.status, WorkflowEventBusDeliveryStatus::Accepted);
        assert_eq!(accepted.channel_address, "workflow.state.events.v1");
        assert!(
            accepted
                .evidence_refs
                .contains(&"workflow-event-bus-kernel:delivery-accepted".to_owned())
        );

        let rejected = evaluate_delivery(
            subscription(),
            candidate(WorkflowEventBusEventKind::WorkflowRunStarted),
        )
        .expect("rejected delivery decision");
        assert_eq!(rejected.status, WorkflowEventBusDeliveryStatus::Rejected);
        assert!(
            rejected
                .evidence_refs
                .contains(&"workflow-event-bus-kernel:channel-not-subscribed".to_owned())
        );
    }

    #[test]
    fn subscription_rejects_tenant_cell_drift_and_invalid_batch_bounds_before_delivery() {
        let tenant_drift = WorkflowEventBusDeliveryCandidate {
            tenant_id: "ten_other".to_owned(),
            ..candidate(WorkflowEventBusEventKind::WorkflowStateTransitioned)
        };
        let error = evaluate_delivery(subscription(), tenant_drift).expect_err("tenant drift");
        assert_eq!(
            error.primary_evidence_ref(),
            "workflow-event-bus-kernel:tenant-cell-delivery-mismatch"
        );

        let invalid_subscription = WorkflowEventBusSubscription {
            max_batch_size: WORKFLOW_EVENT_BUS_MAX_SUBSCRIPTION_BATCH_SIZE + 1,
            ..subscription()
        };
        let error = evaluate_delivery(
            invalid_subscription,
            candidate(WorkflowEventBusEventKind::WorkflowStateTransitioned),
        )
        .expect_err("invalid batch");
        assert_eq!(
            error.primary_evidence_ref(),
            "workflow-event-bus-kernel:subscription-batch-invalid"
        );
    }

    #[test]
    fn deterministic_delivery_key_is_stable_and_source_level_non_claims_are_preserved() {
        let left = plan_publish(publish_request(
            WorkflowEventBusEventKind::WorkflowRunStarted,
        ))
        .expect("left plan");
        let mut right_request = publish_request(WorkflowEventBusEventKind::WorkflowRunStarted);
        right_request.evidence_refs.reverse();
        let right = plan_publish(right_request).expect("right plan");
        assert_eq!(left.delivery_key, right.delivery_key);
        assert_eq!(left.evidence_refs, right.evidence_refs);
        assert!(
            right
                .non_claim_refs
                .contains(&"workflow-event-bus-kernel:no-hyperscaler-claim".to_owned())
        );
    }
}
