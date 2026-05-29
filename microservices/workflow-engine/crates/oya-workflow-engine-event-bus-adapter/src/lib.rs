//! Workflow-engine event-bus generic adapter foundation.
//!
//! This crate provides a source-level in-memory adapter for preview event-bus
//! integration. It records CloudEvents/AsyncAPI-shaped publish envelopes,
//! metadata-only outbox/inbox plans, consumer offset observations, and explicit
//! offset-commit non-claims around the existing event-bus API boundary. It is
//! intentionally non-durable and performs no broker connection, topic creation,
//! network I/O, consumer-group coordination, durable outbox/inbox write,
//! serialization-framework work, database access, Valkey access, offset commit,
//! signing, Kubernetes call, cloud deployment, or tenant workload scheduling.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

pub use oya_workflow_engine_event_bus_api::{
    WORKFLOW_EVENT_BUS_API_DECLARED_VERSION, WORKFLOW_EVENT_BUS_API_DELIVERY_ROUTE,
    WORKFLOW_EVENT_BUS_API_METHOD, WORKFLOW_EVENT_BUS_API_PUBLISH_ROUTE,
    WORKFLOW_EVENT_BUS_API_SURFACE, WORKFLOW_EVENT_BUS_ASYNCAPI_CONTRACT_REF,
    WORKFLOW_EVENT_BUS_CLOUDEVENTS_SPECVERSION, WORKFLOW_EVENT_BUS_USECASE_SURFACE,
    WorkflowEventBusApi, WorkflowEventBusApiAuthorization, WorkflowEventBusApiBoundaryContext,
    WorkflowEventBusApiDeliveryBody, WorkflowEventBusApiDeliveryRequest,
    WorkflowEventBusApiEventDto, WorkflowEventBusApiPrincipal, WorkflowEventBusApiPublishBody,
    WorkflowEventBusApiPublishRequest, WorkflowEventBusApiResponseMetadata,
    WorkflowEventBusApiStatus, WorkflowEventBusApiSuccessResponse, WorkflowEventBusEventKind,
};

pub const WORKFLOW_EVENT_BUS_ADAPTER_SURFACE: &str = "workflow-engine.event-bus.adapter";
pub const WORKFLOW_EVENT_BUS_ADAPTER_MODE_REF: &str =
    "workflow-event-bus-adapter:in-memory-preview";

const NON_CLAIM_NO_BROKER_RUNTIME: &str = "workflow-event-bus-adapter:no-broker-runtime";
const NON_CLAIM_NO_TOPIC_RUNTIME: &str = "workflow-event-bus-adapter:no-topic-runtime";
const NON_CLAIM_NO_DURABLE_OUTBOX_RUNTIME: &str =
    "workflow-event-bus-adapter:no-durable-outbox-runtime";
const NON_CLAIM_NO_DURABLE_INBOX_RUNTIME: &str =
    "workflow-event-bus-adapter:no-durable-inbox-runtime";
const NON_CLAIM_NO_CONSUMER_GROUP_RUNTIME: &str =
    "workflow-event-bus-adapter:no-consumer-group-runtime";
const NON_CLAIM_NO_OFFSET_COMMIT_RUNTIME: &str =
    "workflow-event-bus-adapter:no-offset-commit-runtime";
const NON_CLAIM_NO_CLOUD_RUNTIME: &str = "workflow-event-bus-adapter:no-cloud-runtime";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkflowEventBusAdapterMode {
    InMemoryPreview,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkflowEventBusAdapterActionKind {
    PublishEnvelopeRecorded,
    OutboxInsertPlanned,
    BrokerPublishPlanned,
    InboxDeliveryRecorded,
    ConsumerOffsetObserved,
    OffsetCommitPlannedFalse,
    IdempotencyReplay,
    IdempotencyConflict,
    UnsafeMetadataRejected,
}

impl WorkflowEventBusAdapterActionKind {
    pub const fn as_wire(self) -> &'static str {
        match self {
            Self::PublishEnvelopeRecorded => "publish-envelope-recorded",
            Self::OutboxInsertPlanned => "outbox-insert-planned",
            Self::BrokerPublishPlanned => "broker-publish-planned",
            Self::InboxDeliveryRecorded => "inbox-delivery-recorded",
            Self::ConsumerOffsetObserved => "consumer-offset-observed",
            Self::OffsetCommitPlannedFalse => "offset-commit-planned-false",
            Self::IdempotencyReplay => "idempotency-replay",
            Self::IdempotencyConflict => "idempotency-conflict",
            Self::UnsafeMetadataRejected => "unsafe-metadata-rejected",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkflowEventBusAdapterReceiptStatus {
    Recorded,
    Replay,
}

impl WorkflowEventBusAdapterReceiptStatus {
    pub const fn as_wire(self) -> &'static str {
        match self {
            Self::Recorded => "recorded",
            Self::Replay => "replay",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkflowEventBusAdapterErrorCode {
    ApiMismatch,
    IdempotencyConflict,
    UnsafeMetadata,
}

impl WorkflowEventBusAdapterErrorCode {
    pub const fn as_wire(self) -> &'static str {
        match self {
            Self::ApiMismatch => "WORKFLOW_EVENT_BUS_ADAPTER_API_MISMATCH",
            Self::IdempotencyConflict => "WORKFLOW_EVENT_BUS_ADAPTER_IDEMPOTENCY_CONFLICT",
            Self::UnsafeMetadata => "WORKFLOW_EVENT_BUS_ADAPTER_UNSAFE_METADATA",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusAdapterError {
    pub code: WorkflowEventBusAdapterErrorCode, // data_class: PUBLIC
    pub evidence_ref: String,                   // data_class: INTERNAL_ONLY
}

impl WorkflowEventBusAdapterError {
    pub fn code(&self) -> WorkflowEventBusAdapterErrorCode {
        self.code
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusAdapterAction {
    pub kind: WorkflowEventBusAdapterActionKind, // data_class: INTERNAL_ONLY
    pub tenant_id: String,                       // data_class: INTERNAL_ONLY
    pub cell_id: String,                         // data_class: INTERNAL_ONLY
    pub channel_address: String,                 // data_class: PUBLIC
    pub event_id: Option<String>,                // data_class: INTERNAL_ONLY
    pub event_type: Option<String>,              // data_class: PUBLIC
    pub idempotency_key: String,                 // data_class: INTERNAL_ONLY
    pub consumer_ref: Option<String>,            // data_class: INTERNAL_ONLY
    pub offset_ref: Option<String>,              // data_class: INTERNAL_ONLY
    pub evidence_ref: String,                    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusAdapterPublishEnvelope {
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub cell_id: String,                      // data_class: INTERNAL_ONLY
    pub channel_address: String,              // data_class: PUBLIC
    pub event_id: String,                     // data_class: INTERNAL_ONLY
    pub event_type: String,                   // data_class: PUBLIC
    pub source_ref: String,                   // data_class: INTERNAL_ONLY
    pub subject_ref: Option<String>,          // data_class: INTERNAL_ONLY
    pub partition_key_ref: String,            // data_class: INTERNAL_ONLY
    pub payload_ref: String,                  // data_class: INTERNAL_ONLY
    pub idempotency_key: String,              // data_class: INTERNAL_ONLY
    pub trace_context_ref: String,            // data_class: INTERNAL_ONLY
    pub audit_chain_ref: String,              // data_class: INTERNAL_ONLY
    pub asyncapi_channel_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub cloudevents_specversion: String,      // data_class: PUBLIC
    pub evidence_refs: Vec<String>,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusAdapterDeliveryEnvelope {
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub cell_id: String,                   // data_class: INTERNAL_ONLY
    pub channel_address: String,           // data_class: PUBLIC
    pub event_id: String,                  // data_class: INTERNAL_ONLY
    pub event_type: String,                // data_class: PUBLIC
    pub consumer_ref: String,              // data_class: INTERNAL_ONLY
    pub offset_ref: String,                // data_class: INTERNAL_ONLY
    pub payload_ref: String,               // data_class: INTERNAL_ONLY
    pub idempotency_key: String,           // data_class: INTERNAL_ONLY
    pub replay_cursor_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub trace_context_ref: String,         // data_class: INTERNAL_ONLY
    pub audit_chain_ref: String,           // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusAdapterPublishReceipt {
    pub status: WorkflowEventBusAdapterReceiptStatus, // data_class: PUBLIC
    pub delivery_key: String,                         // data_class: INTERNAL_ONLY
    pub channel_address: String,                      // data_class: PUBLIC
    pub event_type: String,                           // data_class: PUBLIC
    pub actions: Vec<WorkflowEventBusAdapterAction>,  // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,                   // data_class: INTERNAL_ONLY
    pub non_claim_refs: Vec<String>,                  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowEventBusAdapterDeliveryReceipt {
    pub status: WorkflowEventBusAdapterReceiptStatus, // data_class: PUBLIC
    pub delivery_status: String,                      // data_class: PUBLIC
    pub channel_address: String,                      // data_class: PUBLIC
    pub event_type: String,                           // data_class: PUBLIC
    pub consumer_ref: String,                         // data_class: INTERNAL_ONLY
    pub offset_ref: String,                           // data_class: INTERNAL_ONLY
    pub offset_commit_planned: bool,                  // data_class: INTERNAL_ONLY
    pub actions: Vec<WorkflowEventBusAdapterAction>,  // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,                   // data_class: INTERNAL_ONLY
    pub non_claim_refs: Vec<String>,                  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct AdapterKey {
    tenant_id: String,
    idempotency_key: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StoredPublish {
    fingerprint: String,
    receipt: WorkflowEventBusAdapterPublishReceipt,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StoredDelivery {
    fingerprint: String,
    receipt: WorkflowEventBusAdapterDeliveryReceipt,
}

#[derive(Default, Debug)]
pub struct WorkflowEventBusMemoryAdapter {
    publish_by_key: BTreeMap<AdapterKey, StoredPublish>,
    delivery_by_key: BTreeMap<AdapterKey, StoredDelivery>,
    recorded_actions: Vec<WorkflowEventBusAdapterAction>,
}

pub type InMemoryWorkflowEventBusAdapter = WorkflowEventBusMemoryAdapter;

impl WorkflowEventBusMemoryAdapter {
    pub fn adapter_mode(&self) -> WorkflowEventBusAdapterMode {
        WorkflowEventBusAdapterMode::InMemoryPreview
    }

    pub fn publish_count(&self) -> usize {
        self.publish_by_key.len()
    }

    pub fn delivery_count(&self) -> usize {
        self.delivery_by_key.len()
    }

    pub fn recorded_actions(&self) -> &[WorkflowEventBusAdapterAction] {
        &self.recorded_actions
    }

    pub fn record_publish_from_api_success(
        &mut self,
        success: &WorkflowEventBusApiSuccessResponse,
        envelope: WorkflowEventBusAdapterPublishEnvelope,
    ) -> Result<WorkflowEventBusAdapterPublishReceipt, WorkflowEventBusAdapterError> {
        validate_publish_envelope(&envelope)?;
        validate_publish_success(success, &envelope)?;

        let key = adapter_key(&envelope.tenant_id, &envelope.idempotency_key);
        let fingerprint = publish_fingerprint(success, &envelope);
        if let Some(stored) = self.publish_by_key.get(&key) {
            if stored.fingerprint == fingerprint {
                let replay_action = publish_action(
                    WorkflowEventBusAdapterActionKind::IdempotencyReplay,
                    &envelope,
                    "workflow-event-bus-adapter:publish-idempotency-replay",
                );
                self.recorded_actions.push(replay_action.clone());
                let mut replay = stored.receipt.clone();
                replay.status = WorkflowEventBusAdapterReceiptStatus::Replay;
                replay.actions = vec![replay_action];
                replay.evidence_refs = sorted_unique(vec![
                    replay.evidence_refs,
                    vec!["workflow-event-bus-adapter:publish-replay".to_owned()],
                ]);
                return Ok(replay);
            }
            let conflict_action = publish_action(
                WorkflowEventBusAdapterActionKind::IdempotencyConflict,
                &envelope,
                "workflow-event-bus-adapter:publish-idempotency-conflict",
            );
            self.recorded_actions.push(conflict_action);
            return Err(adapter_error(
                WorkflowEventBusAdapterErrorCode::IdempotencyConflict,
                "workflow-event-bus-adapter:publish-idempotency-conflict",
            ));
        }

        let actions = vec![
            publish_action(
                WorkflowEventBusAdapterActionKind::PublishEnvelopeRecorded,
                &envelope,
                "workflow-event-bus-adapter:publish-envelope-recorded",
            ),
            publish_action(
                WorkflowEventBusAdapterActionKind::OutboxInsertPlanned,
                &envelope,
                "workflow-event-bus-adapter:outbox-insert-planned",
            ),
            publish_action(
                WorkflowEventBusAdapterActionKind::BrokerPublishPlanned,
                &envelope,
                "workflow-event-bus-adapter:broker-publish-planned",
            ),
        ];
        self.recorded_actions.extend(actions.clone());
        let receipt = WorkflowEventBusAdapterPublishReceipt {
            status: WorkflowEventBusAdapterReceiptStatus::Recorded,
            delivery_key: success
                .event
                .delivery_key
                .clone()
                .unwrap_or_else(|| delivery_key_from_envelope(&envelope)),
            channel_address: envelope.channel_address.clone(),
            event_type: envelope.event_type.clone(),
            actions,
            evidence_refs: sorted_unique(vec![
                success.evidence_refs.clone(),
                envelope.evidence_refs.clone(),
                vec![WORKFLOW_EVENT_BUS_ADAPTER_SURFACE.to_owned()],
            ]),
            non_claim_refs: sorted_unique(vec![
                success.non_claim_refs.clone(),
                publish_non_claim_refs(),
            ]),
        };
        self.publish_by_key.insert(
            key,
            StoredPublish {
                fingerprint,
                receipt: receipt.clone(),
            },
        );
        Ok(receipt)
    }

    pub fn record_delivery_from_api_success(
        &mut self,
        success: &WorkflowEventBusApiSuccessResponse,
        envelope: WorkflowEventBusAdapterDeliveryEnvelope,
    ) -> Result<WorkflowEventBusAdapterDeliveryReceipt, WorkflowEventBusAdapterError> {
        validate_delivery_envelope(&envelope)?;
        validate_delivery_success(success, &envelope)?;

        let key = adapter_key(&envelope.tenant_id, &envelope.idempotency_key);
        let fingerprint = delivery_fingerprint(success, &envelope);
        if let Some(stored) = self.delivery_by_key.get(&key) {
            if stored.fingerprint == fingerprint {
                let replay_action = delivery_action(
                    WorkflowEventBusAdapterActionKind::IdempotencyReplay,
                    &envelope,
                    "workflow-event-bus-adapter:delivery-idempotency-replay",
                );
                self.recorded_actions.push(replay_action.clone());
                let mut replay = stored.receipt.clone();
                replay.status = WorkflowEventBusAdapterReceiptStatus::Replay;
                replay.actions = vec![replay_action];
                replay.evidence_refs = sorted_unique(vec![
                    replay.evidence_refs,
                    vec!["workflow-event-bus-adapter:delivery-replay".to_owned()],
                ]);
                return Ok(replay);
            }
            let conflict_action = delivery_action(
                WorkflowEventBusAdapterActionKind::IdempotencyConflict,
                &envelope,
                "workflow-event-bus-adapter:delivery-idempotency-conflict",
            );
            self.recorded_actions.push(conflict_action);
            return Err(adapter_error(
                WorkflowEventBusAdapterErrorCode::IdempotencyConflict,
                "workflow-event-bus-adapter:delivery-idempotency-conflict",
            ));
        }

        let actions = vec![
            delivery_action(
                WorkflowEventBusAdapterActionKind::InboxDeliveryRecorded,
                &envelope,
                "workflow-event-bus-adapter:inbox-delivery-recorded",
            ),
            delivery_action(
                WorkflowEventBusAdapterActionKind::ConsumerOffsetObserved,
                &envelope,
                "workflow-event-bus-adapter:consumer-offset-observed",
            ),
            delivery_action(
                WorkflowEventBusAdapterActionKind::OffsetCommitPlannedFalse,
                &envelope,
                "workflow-event-bus-adapter:offset-commit-planned-false",
            ),
        ];
        self.recorded_actions.extend(actions.clone());
        let receipt = WorkflowEventBusAdapterDeliveryReceipt {
            status: WorkflowEventBusAdapterReceiptStatus::Recorded,
            delivery_status: success.event.usecase_status.clone(),
            channel_address: envelope.channel_address.clone(),
            event_type: envelope.event_type.clone(),
            consumer_ref: envelope.consumer_ref.clone(),
            offset_ref: envelope.offset_ref.clone(),
            offset_commit_planned: false,
            actions,
            evidence_refs: sorted_unique(vec![
                success.evidence_refs.clone(),
                envelope.evidence_refs.clone(),
                vec![WORKFLOW_EVENT_BUS_ADAPTER_SURFACE.to_owned()],
            ]),
            non_claim_refs: sorted_unique(vec![
                success.non_claim_refs.clone(),
                delivery_non_claim_refs(),
            ]),
        };
        self.delivery_by_key.insert(
            key,
            StoredDelivery {
                fingerprint,
                receipt: receipt.clone(),
            },
        );
        Ok(receipt)
    }
}

fn adapter_key(tenant_id: &str, idempotency_key: &str) -> AdapterKey {
    AdapterKey {
        tenant_id: tenant_id.to_owned(),
        idempotency_key: idempotency_key.to_owned(),
    }
}

fn publish_action(
    kind: WorkflowEventBusAdapterActionKind,
    envelope: &WorkflowEventBusAdapterPublishEnvelope,
    evidence_ref: &str,
) -> WorkflowEventBusAdapterAction {
    WorkflowEventBusAdapterAction {
        kind,
        tenant_id: envelope.tenant_id.clone(),
        cell_id: envelope.cell_id.clone(),
        channel_address: envelope.channel_address.clone(),
        event_id: Some(envelope.event_id.clone()),
        event_type: Some(envelope.event_type.clone()),
        idempotency_key: envelope.idempotency_key.clone(),
        consumer_ref: None,
        offset_ref: None,
        evidence_ref: evidence_ref.to_owned(),
    }
}

fn delivery_action(
    kind: WorkflowEventBusAdapterActionKind,
    envelope: &WorkflowEventBusAdapterDeliveryEnvelope,
    evidence_ref: &str,
) -> WorkflowEventBusAdapterAction {
    WorkflowEventBusAdapterAction {
        kind,
        tenant_id: envelope.tenant_id.clone(),
        cell_id: envelope.cell_id.clone(),
        channel_address: envelope.channel_address.clone(),
        event_id: Some(envelope.event_id.clone()),
        event_type: Some(envelope.event_type.clone()),
        idempotency_key: envelope.idempotency_key.clone(),
        consumer_ref: Some(envelope.consumer_ref.clone()),
        offset_ref: Some(envelope.offset_ref.clone()),
        evidence_ref: evidence_ref.to_owned(),
    }
}

fn validate_publish_success(
    success: &WorkflowEventBusApiSuccessResponse,
    envelope: &WorkflowEventBusAdapterPublishEnvelope,
) -> Result<(), WorkflowEventBusAdapterError> {
    let api_channel = success.event.channel_address.as_deref();
    let asyncapi_ref = success.event.asyncapi_channel_ref.as_deref();
    let envelope_asyncapi = envelope.asyncapi_channel_ref.as_deref();
    let valid = success.status == WorkflowEventBusApiStatus::Accepted
        && success.route == WORKFLOW_EVENT_BUS_API_PUBLISH_ROUTE
        && success.event.operation == "publish"
        && success.event.usecase_status == "published"
        && success.event.tenant_id == envelope.tenant_id
        && success.event.cell_id == envelope.cell_id
        && success.event.event_type == envelope.event_type
        && api_channel == Some(envelope.channel_address.as_str())
        && success.metadata.tenant_id == envelope.tenant_id
        && success.metadata.idempotency_key == envelope.idempotency_key
        && success.metadata.trace_context_ref == envelope.trace_context_ref
        && success.metadata.surface == WORKFLOW_EVENT_BUS_API_SURFACE
        && asyncapi_ref == envelope_asyncapi;
    if valid {
        Ok(())
    } else {
        Err(adapter_error(
            WorkflowEventBusAdapterErrorCode::ApiMismatch,
            "workflow-event-bus-adapter:publish-api-envelope-mismatch",
        ))
    }
}

fn validate_delivery_success(
    success: &WorkflowEventBusApiSuccessResponse,
    envelope: &WorkflowEventBusAdapterDeliveryEnvelope,
) -> Result<(), WorkflowEventBusAdapterError> {
    let api_channel = success.event.channel_address.as_deref();
    let consumer_ref = success.event.consumer_ref.as_deref();
    let offset_ref = success.event.offset_ref.as_deref();
    let valid = success.status == WorkflowEventBusApiStatus::Accepted
        && success.route == WORKFLOW_EVENT_BUS_API_DELIVERY_ROUTE
        && success.event.operation == "delivery-evaluate"
        && matches!(
            success.event.usecase_status.as_str(),
            "delivery-accepted" | "delivery-denied"
        )
        && success.event.tenant_id == envelope.tenant_id
        && success.event.cell_id == envelope.cell_id
        && success.event.event_type == envelope.event_type
        && api_channel == Some(envelope.channel_address.as_str())
        && consumer_ref == Some(envelope.consumer_ref.as_str())
        && offset_ref == Some(envelope.offset_ref.as_str())
        && success.metadata.tenant_id == envelope.tenant_id
        && success.metadata.idempotency_key == envelope.idempotency_key
        && success.metadata.trace_context_ref == envelope.trace_context_ref
        && success.metadata.surface == WORKFLOW_EVENT_BUS_API_SURFACE;
    if valid {
        Ok(())
    } else {
        Err(adapter_error(
            WorkflowEventBusAdapterErrorCode::ApiMismatch,
            "workflow-event-bus-adapter:delivery-api-envelope-mismatch",
        ))
    }
}

fn validate_publish_envelope(
    envelope: &WorkflowEventBusAdapterPublishEnvelope,
) -> Result<(), WorkflowEventBusAdapterError> {
    let asyncapi_safe = envelope
        .asyncapi_channel_ref
        .as_deref()
        .is_some_and(|value| {
            is_safe_metadata(value)
                && value.starts_with(WORKFLOW_EVENT_BUS_ASYNCAPI_CONTRACT_REF)
                && value.contains("#/channels/")
        });
    let invalid = !is_safe_tenant(&envelope.tenant_id)
        || !is_safe_ref(&envelope.cell_id)
        || !is_safe_metadata(&envelope.channel_address)
        || !is_safe_ref(&envelope.event_id)
        || !is_safe_metadata(&envelope.event_type)
        || !is_safe_ref(&envelope.source_ref)
        || !is_safe_optional_ref(envelope.subject_ref.as_deref())
        || !is_safe_ref(&envelope.partition_key_ref)
        || !is_safe_ref(&envelope.payload_ref)
        || !is_safe_ref(&envelope.idempotency_key)
        || !is_safe_ref(&envelope.trace_context_ref)
        || !is_safe_ref(&envelope.audit_chain_ref)
        || !asyncapi_safe
        || envelope.cloudevents_specversion != WORKFLOW_EVENT_BUS_CLOUDEVENTS_SPECVERSION
        || !envelope
            .evidence_refs
            .iter()
            .all(|value| is_safe_ref(value));
    if invalid {
        Err(adapter_error(
            WorkflowEventBusAdapterErrorCode::UnsafeMetadata,
            "workflow-event-bus-adapter:unsafe-publish-envelope-metadata",
        ))
    } else {
        Ok(())
    }
}

fn validate_delivery_envelope(
    envelope: &WorkflowEventBusAdapterDeliveryEnvelope,
) -> Result<(), WorkflowEventBusAdapterError> {
    let invalid = !is_safe_tenant(&envelope.tenant_id)
        || !is_safe_ref(&envelope.cell_id)
        || !is_safe_metadata(&envelope.channel_address)
        || !is_safe_ref(&envelope.event_id)
        || !is_safe_metadata(&envelope.event_type)
        || !is_safe_ref(&envelope.consumer_ref)
        || !is_safe_ref(&envelope.offset_ref)
        || !is_safe_ref(&envelope.payload_ref)
        || !is_safe_ref(&envelope.idempotency_key)
        || !is_safe_optional_ref(envelope.replay_cursor_ref.as_deref())
        || !is_safe_ref(&envelope.trace_context_ref)
        || !is_safe_ref(&envelope.audit_chain_ref)
        || !envelope
            .evidence_refs
            .iter()
            .all(|value| is_safe_ref(value));
    if invalid {
        Err(adapter_error(
            WorkflowEventBusAdapterErrorCode::UnsafeMetadata,
            "workflow-event-bus-adapter:unsafe-delivery-envelope-metadata",
        ))
    } else {
        Ok(())
    }
}

fn publish_fingerprint(
    success: &WorkflowEventBusApiSuccessResponse,
    envelope: &WorkflowEventBusAdapterPublishEnvelope,
) -> String {
    format!(
        "publish|api={}|tenant={}|cell={}|channel={}|event_type={}|event_id={}|source={}|subject={:?}|partition={}|payload={}|idem={}|trace={}|audit={}|asyncapi={:?}|specversion={}|evidence={:?}",
        success.metadata.request_id,
        envelope.tenant_id,
        envelope.cell_id,
        envelope.channel_address,
        envelope.event_type,
        envelope.event_id,
        envelope.source_ref,
        envelope.subject_ref,
        envelope.partition_key_ref,
        envelope.payload_ref,
        envelope.idempotency_key,
        envelope.trace_context_ref,
        envelope.audit_chain_ref,
        envelope.asyncapi_channel_ref,
        envelope.cloudevents_specversion,
        sorted_unique(vec![
            success.evidence_refs.clone(),
            envelope.evidence_refs.clone()
        ]),
    )
}

fn delivery_fingerprint(
    success: &WorkflowEventBusApiSuccessResponse,
    envelope: &WorkflowEventBusAdapterDeliveryEnvelope,
) -> String {
    format!(
        "delivery|api={}|tenant={}|cell={}|channel={}|event_type={}|event_id={}|consumer={}|offset={}|payload={}|idem={}|replay={:?}|trace={}|audit={}|status={}|evidence={:?}",
        success.metadata.request_id,
        envelope.tenant_id,
        envelope.cell_id,
        envelope.channel_address,
        envelope.event_type,
        envelope.event_id,
        envelope.consumer_ref,
        envelope.offset_ref,
        envelope.payload_ref,
        envelope.idempotency_key,
        envelope.replay_cursor_ref,
        envelope.trace_context_ref,
        envelope.audit_chain_ref,
        success.event.usecase_status,
        sorted_unique(vec![
            success.evidence_refs.clone(),
            envelope.evidence_refs.clone()
        ]),
    )
}

fn delivery_key_from_envelope(envelope: &WorkflowEventBusAdapterPublishEnvelope) -> String {
    format!(
        "delivery-key:{}:{}:{}",
        envelope.channel_address, envelope.event_type, envelope.event_id
    )
}

fn publish_non_claim_refs() -> Vec<String> {
    vec![
        NON_CLAIM_NO_BROKER_RUNTIME.to_owned(),
        NON_CLAIM_NO_TOPIC_RUNTIME.to_owned(),
        NON_CLAIM_NO_DURABLE_OUTBOX_RUNTIME.to_owned(),
        NON_CLAIM_NO_CLOUD_RUNTIME.to_owned(),
    ]
}

fn delivery_non_claim_refs() -> Vec<String> {
    vec![
        NON_CLAIM_NO_BROKER_RUNTIME.to_owned(),
        NON_CLAIM_NO_DURABLE_INBOX_RUNTIME.to_owned(),
        NON_CLAIM_NO_CONSUMER_GROUP_RUNTIME.to_owned(),
        NON_CLAIM_NO_OFFSET_COMMIT_RUNTIME.to_owned(),
        NON_CLAIM_NO_CLOUD_RUNTIME.to_owned(),
    ]
}

fn adapter_error(
    code: WorkflowEventBusAdapterErrorCode,
    evidence_ref: &str,
) -> WorkflowEventBusAdapterError {
    WorkflowEventBusAdapterError {
        code,
        evidence_ref: evidence_ref.to_owned(),
    }
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

fn sorted_unique(chunks: Vec<Vec<String>>) -> Vec<String> {
    let mut values: Vec<String> = chunks.into_iter().flatten().collect();
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
    fn adapter_mode_actions_and_non_claims_are_stable_metadata_only() {
        let adapter = WorkflowEventBusMemoryAdapter::default();
        assert_eq!(
            adapter.adapter_mode(),
            WorkflowEventBusAdapterMode::InMemoryPreview
        );
        assert_eq!(
            WORKFLOW_EVENT_BUS_ADAPTER_MODE_REF,
            "workflow-event-bus-adapter:in-memory-preview"
        );

        let actions = [
            WorkflowEventBusAdapterActionKind::PublishEnvelopeRecorded,
            WorkflowEventBusAdapterActionKind::OutboxInsertPlanned,
            WorkflowEventBusAdapterActionKind::BrokerPublishPlanned,
            WorkflowEventBusAdapterActionKind::InboxDeliveryRecorded,
            WorkflowEventBusAdapterActionKind::ConsumerOffsetObserved,
            WorkflowEventBusAdapterActionKind::OffsetCommitPlannedFalse,
            WorkflowEventBusAdapterActionKind::IdempotencyReplay,
            WorkflowEventBusAdapterActionKind::IdempotencyConflict,
            WorkflowEventBusAdapterActionKind::UnsafeMetadataRejected,
        ];
        let wire: Vec<&str> = actions.iter().map(|action| action.as_wire()).collect();
        let unique: BTreeSet<&str> = wire.iter().copied().collect();
        assert_eq!(wire.len(), unique.len());
        assert!(publish_non_claim_refs().contains(&NON_CLAIM_NO_BROKER_RUNTIME.to_owned()));
        assert!(delivery_non_claim_refs().contains(&NON_CLAIM_NO_OFFSET_COMMIT_RUNTIME.to_owned()));
    }

    #[test]
    fn publish_success_records_cloudevents_asyncapi_outbox_and_broker_plan_metadata() {
        let mut api = WorkflowEventBusApi::default();
        let success = api
            .publish_event(publish_request("idem:event-bus-adapter:publish:1"))
            .expect("api publish accepted");
        let mut adapter = WorkflowEventBusMemoryAdapter::default();
        let receipt = adapter
            .record_publish_from_api_success(&success, publish_envelope(&success))
            .expect("adapter recorded publish");

        assert_eq!(receipt.status.as_wire(), "recorded");
        assert_eq!(adapter.publish_count(), 1);
        assert_eq!(receipt.channel_address, "workflow.runs.events.v1");
        assert_eq!(
            receipt.event_type,
            WorkflowEventBusEventKind::WorkflowRunStarted.event_type()
        );
        assert_eq!(receipt.actions.len(), 3);
        assert!(receipt.actions.iter().any(|action| {
            action.kind == WorkflowEventBusAdapterActionKind::OutboxInsertPlanned
        }));
        assert!(receipt.actions.iter().any(|action| {
            action.kind == WorkflowEventBusAdapterActionKind::BrokerPublishPlanned
        }));
        assert!(
            receipt
                .evidence_refs
                .contains(&WORKFLOW_EVENT_BUS_ADAPTER_SURFACE.to_owned())
        );
        assert!(
            receipt
                .non_claim_refs
                .contains(&NON_CLAIM_NO_DURABLE_OUTBOX_RUNTIME.to_owned())
        );
        assert!(
            receipt
                .non_claim_refs
                .contains(&NON_CLAIM_NO_BROKER_RUNTIME.to_owned())
        );
    }

    #[test]
    fn publish_replay_records_replay_action_and_same_key_drift_conflicts_without_replace() {
        let mut api = WorkflowEventBusApi::default();
        let success = api
            .publish_event(publish_request("idem:event-bus-adapter:publish-replay"))
            .expect("api publish accepted");
        let mut adapter = WorkflowEventBusMemoryAdapter::default();
        let envelope = publish_envelope(&success);
        let first = adapter
            .record_publish_from_api_success(&success, envelope.clone())
            .expect("first record");
        let replay = adapter
            .record_publish_from_api_success(&success, envelope.clone())
            .expect("replay record");
        assert_eq!(first.delivery_key, replay.delivery_key);
        assert_eq!(replay.status, WorkflowEventBusAdapterReceiptStatus::Replay);
        assert_eq!(adapter.publish_count(), 1);

        let mut drifted = envelope;
        drifted.partition_key_ref = "partition:tenant-workflow-run-drift".to_owned();
        let error = adapter
            .record_publish_from_api_success(&success, drifted)
            .expect_err("conflict");
        assert_eq!(
            error.code(),
            WorkflowEventBusAdapterErrorCode::IdempotencyConflict
        );
        assert_eq!(adapter.publish_count(), 1);
    }

    #[test]
    fn delivery_success_records_inbox_offset_observation_and_no_commit_plan() {
        let mut api = WorkflowEventBusApi::default();
        let success = api
            .evaluate_delivery(delivery_request("idem:event-bus-adapter:delivery:1"))
            .expect("api delivery accepted");
        let mut adapter = WorkflowEventBusMemoryAdapter::default();
        let receipt = adapter
            .record_delivery_from_api_success(&success, delivery_envelope(&success))
            .expect("adapter recorded delivery");

        assert_eq!(adapter.delivery_count(), 1);
        assert_eq!(receipt.delivery_status, "delivery-accepted");
        assert!(!receipt.offset_commit_planned);
        assert!(receipt.actions.iter().any(|action| {
            action.kind == WorkflowEventBusAdapterActionKind::InboxDeliveryRecorded
        }));
        assert!(receipt.actions.iter().any(|action| {
            action.kind == WorkflowEventBusAdapterActionKind::ConsumerOffsetObserved
        }));
        assert!(receipt.actions.iter().any(|action| {
            action.kind == WorkflowEventBusAdapterActionKind::OffsetCommitPlannedFalse
        }));
        assert!(
            receipt
                .non_claim_refs
                .contains(&NON_CLAIM_NO_OFFSET_COMMIT_RUNTIME.to_owned())
        );
    }

    #[test]
    fn delivery_denied_success_still_records_metadata_without_offset_commit() {
        let mut api = WorkflowEventBusApi::default();
        let mut request = delivery_request("idem:event-bus-adapter:delivery-denied");
        request.body.candidate_channel = "workflow-runs".to_owned();
        request.body.candidate_event_type = WorkflowEventBusEventKind::WorkflowRunStarted
            .event_type()
            .to_owned();
        let success = api
            .evaluate_delivery(request)
            .expect("delivery denied as success DTO");
        let mut adapter = WorkflowEventBusMemoryAdapter::default();
        let receipt = adapter
            .record_delivery_from_api_success(&success, delivery_envelope(&success))
            .expect("adapter recorded denied delivery");

        assert_eq!(receipt.delivery_status, "delivery-denied");
        assert!(!receipt.offset_commit_planned);
        assert_eq!(receipt.offset_ref, "offset:partition-0:42");
        assert!(
            receipt
                .evidence_refs
                .contains(&"workflow-event-bus-kernel:channel-not-subscribed".to_owned())
        );
    }

    #[test]
    fn unsafe_raw_metadata_is_rejected_without_echo_and_without_action() {
        let mut api = WorkflowEventBusApi::default();
        let success = api
            .publish_event(publish_request("idem:event-bus-adapter:unsafe"))
            .expect("api publish accepted");
        let mut envelope = publish_envelope(&success);
        envelope.payload_ref =
            "raw payload Authorization: Bearer sk-test customer message".to_owned();
        let mut adapter = WorkflowEventBusMemoryAdapter::default();
        let error = adapter
            .record_publish_from_api_success(&success, envelope)
            .expect_err("unsafe envelope");

        assert_eq!(
            error.code(),
            WorkflowEventBusAdapterErrorCode::UnsafeMetadata
        );
        let rendered = format!("{error:?}");
        assert!(!rendered.contains("raw payload"));
        assert!(!rendered.contains("Authorization"));
        assert!(!rendered.contains("sk-test"));
        assert_eq!(adapter.recorded_actions().len(), 0);
    }

    #[test]
    fn api_envelope_mismatch_is_rejected_before_recording() {
        let mut api = WorkflowEventBusApi::default();
        let success = api
            .publish_event(publish_request("idem:event-bus-adapter:mismatch"))
            .expect("api publish accepted");
        let mut envelope = publish_envelope(&success);
        envelope.channel_address = "workflow.state.events.v1".to_owned();
        let mut adapter = WorkflowEventBusMemoryAdapter::default();
        let error = adapter
            .record_publish_from_api_success(&success, envelope)
            .expect_err("mismatch");

        assert_eq!(error.code(), WorkflowEventBusAdapterErrorCode::ApiMismatch);
        assert_eq!(adapter.publish_count(), 0);
        assert_eq!(adapter.recorded_actions().len(), 0);
    }

    #[test]
    fn api_publish_and_delivery_integration_preserves_idempotency_and_no_offset_commit() {
        let mut api = WorkflowEventBusApi::default();
        let publish_success = api
            .publish_event(publish_request(
                "idem:event-bus-adapter:integration:publish",
            ))
            .expect("publish accepted");
        let delivery_success = api
            .evaluate_delivery(delivery_request(
                "idem:event-bus-adapter:integration:delivery",
            ))
            .expect("delivery accepted");
        let mut adapter = WorkflowEventBusMemoryAdapter::default();
        let publish_receipt = adapter
            .record_publish_from_api_success(&publish_success, publish_envelope(&publish_success))
            .expect("publish recorded");
        let delivery_receipt = adapter
            .record_delivery_from_api_success(
                &delivery_success,
                delivery_envelope(&delivery_success),
            )
            .expect("delivery recorded");

        assert!(
            publish_receipt
                .delivery_key
                .contains("workflow.runs.events.v1")
        );
        assert_eq!(
            delivery_receipt.consumer_ref,
            "consumer:workflow-state-machine"
        );
        assert!(!delivery_receipt.offset_commit_planned);
        assert_eq!(adapter.recorded_actions().len(), 6);
    }

    fn publish_envelope(
        success: &WorkflowEventBusApiSuccessResponse,
    ) -> WorkflowEventBusAdapterPublishEnvelope {
        WorkflowEventBusAdapterPublishEnvelope {
            tenant_id: success.event.tenant_id.clone(),
            cell_id: success.event.cell_id.clone(),
            channel_address: success.event.channel_address.clone().unwrap(),
            event_id: "event:workflow-run-started:001".to_owned(),
            event_type: success.event.event_type.clone(),
            source_ref: "urn:oyatie:workflow-engine:execution".to_owned(),
            subject_ref: Some("subject:workflow-run:001".to_owned()),
            partition_key_ref: "partition:tenant-workflow-run".to_owned(),
            payload_ref: "body-ref:workflow-run-started".to_owned(),
            idempotency_key: success.metadata.idempotency_key.clone(),
            trace_context_ref: success.metadata.trace_context_ref.clone(),
            audit_chain_ref: "audit-chain:event-bus-adapter".to_owned(),
            asyncapi_channel_ref: success.event.asyncapi_channel_ref.clone(),
            cloudevents_specversion: WORKFLOW_EVENT_BUS_CLOUDEVENTS_SPECVERSION.to_owned(),
            evidence_refs: vec!["evidence:event-bus-adapter:publish".to_owned()],
        }
    }

    fn delivery_envelope(
        success: &WorkflowEventBusApiSuccessResponse,
    ) -> WorkflowEventBusAdapterDeliveryEnvelope {
        WorkflowEventBusAdapterDeliveryEnvelope {
            tenant_id: success.event.tenant_id.clone(),
            cell_id: success.event.cell_id.clone(),
            channel_address: success.event.channel_address.clone().unwrap(),
            event_id: match success.event.usecase_status.as_str() {
                "delivery-accepted" => "event:workflow-state:001".to_owned(),
                "delivery-denied" => "event:workflow-state:001".to_owned(),
                _ => unreachable!("delivery status covered by API helper"),
            },
            event_type: success.event.event_type.clone(),
            consumer_ref: success.event.consumer_ref.clone().unwrap(),
            offset_ref: success.event.offset_ref.clone().unwrap(),
            payload_ref: "body-ref:workflow-state-transitioned".to_owned(),
            idempotency_key: success.metadata.idempotency_key.clone(),
            replay_cursor_ref: Some("cursor:event-bus-adapter:state".to_owned()),
            trace_context_ref: success.metadata.trace_context_ref.clone(),
            audit_chain_ref: "audit-chain:event-bus-adapter".to_owned(),
            evidence_refs: vec!["evidence:event-bus-adapter:delivery".to_owned()],
        }
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
            request_id: format!("request:event-bus-adapter:{idempotency_key}"),
            tenant_id: "ten_workflow_event_bus".to_owned(),
            idempotency_key: idempotency_key.to_owned(),
            trace_context_ref: "trace:event-bus-adapter".to_owned(),
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
