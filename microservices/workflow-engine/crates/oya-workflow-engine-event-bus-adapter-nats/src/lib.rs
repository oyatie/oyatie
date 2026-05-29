//! Workflow-engine event-bus NATS JetStream adapter preview foundation.
//!
//! This crate is a source-level, plan-only adapter around the generic
//! event-bus adapter seam. It models JetStream stream declarations, durable
//! pull-consumer configuration, CloudEvents/AsyncAPI publish-message metadata,
//! bounded pull/ack plans, offset observations, and dead-letter plans without
//! opening a NATS connection or executing any JetStream API request.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use oya_workflow_engine_event_bus_adapter::{
    WORKFLOW_EVENT_BUS_ADAPTER_MODE_REF, WORKFLOW_EVENT_BUS_ADAPTER_SURFACE,
    WORKFLOW_EVENT_BUS_API_DECLARED_VERSION, WORKFLOW_EVENT_BUS_API_DELIVERY_ROUTE,
    WORKFLOW_EVENT_BUS_API_METHOD, WORKFLOW_EVENT_BUS_API_PUBLISH_ROUTE,
    WORKFLOW_EVENT_BUS_API_SURFACE, WORKFLOW_EVENT_BUS_ASYNCAPI_CONTRACT_REF,
    WORKFLOW_EVENT_BUS_CLOUDEVENTS_SPECVERSION, WORKFLOW_EVENT_BUS_USECASE_SURFACE,
    WorkflowEventBusAdapterDeliveryEnvelope, WorkflowEventBusAdapterPublishEnvelope,
    WorkflowEventBusAdapterReceiptStatus, WorkflowEventBusApi, WorkflowEventBusApiAuthorization,
    WorkflowEventBusApiBoundaryContext, WorkflowEventBusApiDeliveryBody,
    WorkflowEventBusApiDeliveryRequest, WorkflowEventBusApiEventDto, WorkflowEventBusApiPrincipal,
    WorkflowEventBusApiPublishBody, WorkflowEventBusApiPublishRequest,
    WorkflowEventBusApiResponseMetadata, WorkflowEventBusApiStatus,
    WorkflowEventBusApiSuccessResponse, WorkflowEventBusEventKind, WorkflowEventBusMemoryAdapter,
};

pub const NATS_EVENT_BUS_ADAPTER_SURFACE: &str = "workflow-engine.event-bus.adapter.nats";
pub const NATS_EVENT_BUS_ADAPTER_MODE_REF: &str =
    "workflow-event-bus-adapter-nats:plan-only-preview";
pub const NATS_EVENT_BUS_MAX_REPLICAS: u8 = 5;
pub const NATS_EVENT_BUS_MAX_PULL_BATCH_SIZE: u32 = 1000;
pub const NATS_EVENT_BUS_MAX_ACK_PENDING: u32 = 50_000;
pub const NATS_EVENT_BUS_DEFAULT_ACK_WAIT_MS: u64 = 30_000;
pub const NATS_EVENT_BUS_DEFAULT_DUPLICATE_WINDOW_MS: u64 = 120_000;
pub const NATS_EVENT_BUS_MAX_DUPLICATE_WINDOW_MS: u64 = 86_400_000;

pub const NATS_EVENT_BUS_ADAPTER_NON_CLAIMS: [&str; 10] = [
    "workflow-event-bus-adapter-nats:no-broker-connection",
    "workflow-event-bus-adapter-nats:no-jetstream-api-runtime",
    "workflow-event-bus-adapter-nats:no-stream-creation-runtime",
    "workflow-event-bus-adapter-nats:no-consumer-creation-runtime",
    "workflow-event-bus-adapter-nats:no-publish-runtime",
    "workflow-event-bus-adapter-nats:no-pull-consumer-runtime",
    "workflow-event-bus-adapter-nats:no-ack-runtime",
    "workflow-event-bus-adapter-nats:no-offset-commit-runtime",
    "workflow-event-bus-adapter-nats:no-cloud-runtime",
    "workflow-event-bus-adapter-nats:no-hyperscaler-claim",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum NatsEventBusPlanKind {
    StreamDeclaration,
    ConsumerConfig,
    PublishMessage,
    PullRequest,
    AckPlan,
    OffsetObservation,
    DeadLetterMessage,
}

impl NatsEventBusPlanKind {
    pub const fn operation(self) -> &'static str {
        match self {
            Self::StreamDeclaration => "STREAM_DECLARATION",
            Self::ConsumerConfig => "CONSUMER_CONFIG",
            Self::PublishMessage => "PUBLISH_MESSAGE",
            Self::PullRequest => "PULL_REQUEST",
            Self::AckPlan => "ACK_PLAN",
            Self::OffsetObservation => "OFFSET_OBSERVATION",
            Self::DeadLetterMessage => "DEAD_LETTER_MESSAGE",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct NatsEventBusConfigEntry {
    pub key: String,   // data_class: PUBLIC
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct NatsEventBusHeaderPlan {
    pub name: String,      // data_class: PUBLIC
    pub value_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NatsEventBusCommandPlan {
    pub plan_kind: NatsEventBusPlanKind,       // data_class: PUBLIC
    pub operation: &'static str,               // data_class: PUBLIC
    pub stream_ref: Option<String>,            // data_class: INTERNAL_ONLY
    pub subject_ref: Option<String>,           // data_class: INTERNAL_ONLY
    pub durable_consumer_ref: Option<String>,  // data_class: INTERNAL_ONLY
    pub message_id_ref: Option<String>,        // data_class: INTERNAL_ONLY
    pub payload_ref: Option<String>,           // data_class: INTERNAL_ONLY
    pub sequence_ref: Option<String>,          // data_class: INTERNAL_ONLY
    pub headers: Vec<NatsEventBusHeaderPlan>,  // data_class: INTERNAL_ONLY
    pub configs: Vec<NatsEventBusConfigEntry>, // data_class: INTERNAL_ONLY
    pub batch_bound: Option<u32>,              // data_class: INTERNAL_ONLY
    pub ack_runtime_planned: bool,             // data_class: INTERNAL_ONLY
    pub offset_commit_planned: bool,           // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,            // data_class: INTERNAL_ONLY
    pub non_claim_refs: Vec<String>,           // data_class: INTERNAL_ONLY
}

impl NatsEventBusCommandPlan {
    fn new(plan_kind: NatsEventBusPlanKind, evidence_refs: Vec<String>) -> Self {
        Self {
            operation: plan_kind.operation(),
            plan_kind,
            stream_ref: None,
            subject_ref: None,
            durable_consumer_ref: None,
            message_id_ref: None,
            payload_ref: None,
            sequence_ref: None,
            headers: Vec::new(),
            configs: Vec::new(),
            batch_bound: None,
            ack_runtime_planned: false,
            offset_commit_planned: false,
            evidence_refs: sorted_unique(evidence_refs),
            non_claim_refs: NATS_EVENT_BUS_ADAPTER_NON_CLAIMS
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
        }
    }

    fn with_stream_ref(mut self, stream_ref: String) -> Self {
        self.stream_ref = Some(stream_ref);
        self
    }

    fn with_subject_ref(mut self, subject_ref: String) -> Self {
        self.subject_ref = Some(subject_ref);
        self
    }

    fn with_durable_consumer_ref(mut self, durable_consumer_ref: String) -> Self {
        self.durable_consumer_ref = Some(durable_consumer_ref);
        self
    }

    fn with_message_id_ref(mut self, message_id_ref: String) -> Self {
        self.message_id_ref = Some(message_id_ref);
        self
    }

    fn with_payload_ref(mut self, payload_ref: String) -> Self {
        self.payload_ref = Some(payload_ref);
        self
    }

    fn with_sequence_ref(mut self, sequence_ref: String) -> Self {
        self.sequence_ref = Some(sequence_ref);
        self
    }

    fn with_headers(mut self, headers: Vec<NatsEventBusHeaderPlan>) -> Self {
        self.headers = headers;
        self
    }

    fn with_configs(mut self, configs: Vec<NatsEventBusConfigEntry>) -> Self {
        self.configs = configs;
        self
    }

    fn with_batch_bound(mut self, batch_bound: u32) -> Self {
        self.batch_bound = Some(batch_bound);
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum NatsEventBusPlanFailure {
    InvalidTenant,
    InvalidReference,
    InvalidStreamDescriptor,
    InvalidConsumerDescriptor,
    InvalidEnvelope,
    UnsafeMetadata,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NatsEventBusStreamDescriptor {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub stream_name_ref: String,       // data_class: INTERNAL_ONLY
    pub subject_ref: String,           // data_class: INTERNAL_ONLY
    pub storage: String,               // data_class: PUBLIC
    pub retention: String,             // data_class: PUBLIC
    pub discard: String,               // data_class: PUBLIC
    pub replicas: u8,                  // data_class: INTERNAL_ONLY
    pub max_age_ms: u64,               // data_class: INTERNAL_ONLY
    pub duplicate_window_ms: u64,      // data_class: INTERNAL_ONLY
    pub placement_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NatsEventBusConsumerDescriptor {
    pub tenant_id: String,            // data_class: INTERNAL_ONLY
    pub stream_name_ref: String,      // data_class: INTERNAL_ONLY
    pub durable_consumer_ref: String, // data_class: INTERNAL_ONLY
    pub filter_subject_ref: String,   // data_class: INTERNAL_ONLY
    pub ack_policy: String,           // data_class: PUBLIC
    pub deliver_policy: String,       // data_class: PUBLIC
    pub replay_policy: String,        // data_class: PUBLIC
    pub max_ack_pending: u32,         // data_class: INTERNAL_ONLY
    pub max_deliver: u32,             // data_class: INTERNAL_ONLY
    pub ack_wait_ms: u64,             // data_class: INTERNAL_ONLY
    pub replicas: u8,                 // data_class: INTERNAL_ONLY
    pub metadata_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NatsEventBusOffsetDescriptor {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub stream_name_ref: String,       // data_class: INTERNAL_ONLY
    pub durable_consumer_ref: String,  // data_class: INTERNAL_ONLY
    pub stream_sequence_ref: String,   // data_class: INTERNAL_ONLY
    pub consumer_sequence_ref: String, // data_class: INTERNAL_ONLY
    pub offset_commit_planned: bool,   // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,    // data_class: INTERNAL_ONLY
}

#[derive(Debug, Default)]
pub struct NatsEventBusAdapter {
    generated_plans: Vec<NatsEventBusCommandPlan>,
}

impl NatsEventBusAdapter {
    pub fn stream_declaration_plan(
        descriptor: &NatsEventBusStreamDescriptor,
    ) -> Result<NatsEventBusCommandPlan, NatsEventBusPlanFailure> {
        validate_stream_descriptor(descriptor)?;
        Ok(NatsEventBusCommandPlan::new(
            NatsEventBusPlanKind::StreamDeclaration,
            descriptor.evidence_refs.clone(),
        )
        .with_stream_ref(descriptor.stream_name_ref.clone())
        .with_subject_ref(descriptor.subject_ref.clone())
        .with_configs(vec![
            config("storage", &descriptor.storage),
            config("retention", &descriptor.retention),
            config("discard", &descriptor.discard),
            config("replicas", &descriptor.replicas.to_string()),
            config("max_age_ms", &descriptor.max_age_ms.to_string()),
            config(
                "duplicate_window_ms",
                &descriptor.duplicate_window_ms.to_string(),
            ),
            config("stream_creation_runtime", "not-planned"),
            config("jetstream_api_runtime", "not-planned"),
        ]))
    }

    pub fn consumer_config_plan(
        descriptor: &NatsEventBusConsumerDescriptor,
    ) -> Result<NatsEventBusCommandPlan, NatsEventBusPlanFailure> {
        validate_consumer_descriptor(descriptor)?;
        Ok(NatsEventBusCommandPlan::new(
            NatsEventBusPlanKind::ConsumerConfig,
            descriptor.evidence_refs.clone(),
        )
        .with_stream_ref(descriptor.stream_name_ref.clone())
        .with_durable_consumer_ref(descriptor.durable_consumer_ref.clone())
        .with_subject_ref(descriptor.filter_subject_ref.clone())
        .with_configs(vec![
            config("dispatch", "pull"),
            config("durable", &descriptor.durable_consumer_ref),
            config("ack_policy", &descriptor.ack_policy),
            config("deliver_policy", &descriptor.deliver_policy),
            config("replay_policy", &descriptor.replay_policy),
            config("max_ack_pending", &descriptor.max_ack_pending.to_string()),
            config("max_deliver", &descriptor.max_deliver.to_string()),
            config("ack_wait_ms", &descriptor.ack_wait_ms.to_string()),
            config("replicas", &descriptor.replicas.to_string()),
            config("consumer_creation_runtime", "not-planned"),
        ]))
    }

    pub fn publish_message_plan(
        envelope: &WorkflowEventBusAdapterPublishEnvelope,
        descriptor: &NatsEventBusStreamDescriptor,
    ) -> Result<NatsEventBusCommandPlan, NatsEventBusPlanFailure> {
        validate_publish_envelope(envelope)?;
        validate_stream_descriptor(descriptor)?;
        if envelope.tenant_id != descriptor.tenant_id {
            return Err(NatsEventBusPlanFailure::InvalidEnvelope);
        }
        Ok(NatsEventBusCommandPlan::new(
            NatsEventBusPlanKind::PublishMessage,
            sorted_unique_join(vec![
                envelope.evidence_refs.clone(),
                descriptor.evidence_refs.clone(),
            ]),
        )
        .with_stream_ref(descriptor.stream_name_ref.clone())
        .with_subject_ref(descriptor.subject_ref.clone())
        .with_message_id_ref(envelope.idempotency_key.clone())
        .with_payload_ref(envelope.payload_ref.clone())
        .with_headers(publish_headers(envelope, descriptor)))
    }

    pub fn pull_request_plan(
        descriptor: &NatsEventBusConsumerDescriptor,
        batch_bound: u32,
    ) -> Result<NatsEventBusCommandPlan, NatsEventBusPlanFailure> {
        validate_consumer_descriptor(descriptor)?;
        if batch_bound == 0 || batch_bound > NATS_EVENT_BUS_MAX_PULL_BATCH_SIZE {
            return Err(NatsEventBusPlanFailure::InvalidConsumerDescriptor);
        }
        Ok(NatsEventBusCommandPlan::new(
            NatsEventBusPlanKind::PullRequest,
            descriptor.evidence_refs.clone(),
        )
        .with_stream_ref(descriptor.stream_name_ref.clone())
        .with_durable_consumer_ref(descriptor.durable_consumer_ref.clone())
        .with_subject_ref(descriptor.filter_subject_ref.clone())
        .with_batch_bound(batch_bound)
        .with_configs(vec![
            config("pull_batch", &batch_bound.to_string()),
            config("ack_policy", &descriptor.ack_policy),
            config("pull_runtime", "not-planned"),
        ]))
    }

    pub fn ack_plan(
        delivery: &WorkflowEventBusAdapterDeliveryEnvelope,
        descriptor: &NatsEventBusConsumerDescriptor,
    ) -> Result<NatsEventBusCommandPlan, NatsEventBusPlanFailure> {
        validate_delivery_envelope(delivery)?;
        validate_consumer_descriptor(descriptor)?;
        if delivery.tenant_id != descriptor.tenant_id {
            return Err(NatsEventBusPlanFailure::InvalidEnvelope);
        }
        Ok(NatsEventBusCommandPlan::new(
            NatsEventBusPlanKind::AckPlan,
            sorted_unique_join(vec![
                delivery.evidence_refs.clone(),
                descriptor.evidence_refs.clone(),
            ]),
        )
        .with_stream_ref(descriptor.stream_name_ref.clone())
        .with_subject_ref(delivery.channel_address.clone())
        .with_durable_consumer_ref(descriptor.durable_consumer_ref.clone())
        .with_message_id_ref(delivery.event_id.clone())
        .with_payload_ref(delivery.payload_ref.clone())
        .with_configs(vec![
            config("ack_policy", &descriptor.ack_policy),
            config("ack_runtime", "not-planned"),
            config("offset_commit_planned", "false"),
        ]))
    }

    pub fn offset_observation_plan(
        descriptor: &NatsEventBusOffsetDescriptor,
    ) -> Result<NatsEventBusCommandPlan, NatsEventBusPlanFailure> {
        validate_offset_descriptor(descriptor)?;
        Ok(NatsEventBusCommandPlan::new(
            NatsEventBusPlanKind::OffsetObservation,
            descriptor.evidence_refs.clone(),
        )
        .with_stream_ref(descriptor.stream_name_ref.clone())
        .with_durable_consumer_ref(descriptor.durable_consumer_ref.clone())
        .with_sequence_ref(descriptor.stream_sequence_ref.clone())
        .with_configs(vec![
            config("stream_sequence_ref", &descriptor.stream_sequence_ref),
            config("consumer_sequence_ref", &descriptor.consumer_sequence_ref),
            config("offset_commit_planned", "false"),
        ]))
    }

    pub fn dead_letter_message_plan(
        delivery: &WorkflowEventBusAdapterDeliveryEnvelope,
        descriptor: &NatsEventBusStreamDescriptor,
    ) -> Result<NatsEventBusCommandPlan, NatsEventBusPlanFailure> {
        validate_delivery_envelope(delivery)?;
        validate_stream_descriptor(descriptor)?;
        if delivery.tenant_id != descriptor.tenant_id {
            return Err(NatsEventBusPlanFailure::InvalidEnvelope);
        }
        Ok(NatsEventBusCommandPlan::new(
            NatsEventBusPlanKind::DeadLetterMessage,
            sorted_unique_join(vec![
                delivery.evidence_refs.clone(),
                descriptor.evidence_refs.clone(),
            ]),
        )
        .with_stream_ref(dead_letter_stream_ref(
            &delivery.tenant_id,
            &delivery.channel_address,
        ))
        .with_subject_ref(dead_letter_subject_ref(
            &delivery.tenant_id,
            &delivery.channel_address,
        ))
        .with_message_id_ref(delivery.event_id.clone())
        .with_payload_ref(delivery.payload_ref.clone())
        .with_configs(vec![
            config("dead_letter_runtime", "not-planned"),
            config("offset_commit_planned", "false"),
        ]))
    }

    pub fn plan_stream_declaration(
        &mut self,
        descriptor: &NatsEventBusStreamDescriptor,
    ) -> Result<&NatsEventBusCommandPlan, NatsEventBusPlanFailure> {
        let plan = Self::stream_declaration_plan(descriptor)?;
        self.generated_plans.push(plan);
        self.generated_plans
            .last()
            .ok_or(NatsEventBusPlanFailure::InvalidStreamDescriptor)
    }

    pub fn plan_publish_message(
        &mut self,
        envelope: &WorkflowEventBusAdapterPublishEnvelope,
        descriptor: &NatsEventBusStreamDescriptor,
    ) -> Result<&NatsEventBusCommandPlan, NatsEventBusPlanFailure> {
        let plan = Self::publish_message_plan(envelope, descriptor)?;
        self.generated_plans.push(plan);
        self.generated_plans
            .last()
            .ok_or(NatsEventBusPlanFailure::InvalidEnvelope)
    }

    pub fn plan_consumer_config(
        &mut self,
        descriptor: &NatsEventBusConsumerDescriptor,
    ) -> Result<&NatsEventBusCommandPlan, NatsEventBusPlanFailure> {
        let plan = Self::consumer_config_plan(descriptor)?;
        self.generated_plans.push(plan);
        self.generated_plans
            .last()
            .ok_or(NatsEventBusPlanFailure::InvalidConsumerDescriptor)
    }

    pub fn generated_plans(&self) -> &[NatsEventBusCommandPlan] {
        &self.generated_plans
    }
}

fn config(key: &str, value: &str) -> NatsEventBusConfigEntry {
    NatsEventBusConfigEntry {
        key: key.to_string(),
        value: value.to_string(),
    }
}

fn header(name: &str, value_ref: &str) -> NatsEventBusHeaderPlan {
    NatsEventBusHeaderPlan {
        name: name.to_string(),
        value_ref: value_ref.to_string(),
    }
}

fn publish_headers(
    envelope: &WorkflowEventBusAdapterPublishEnvelope,
    descriptor: &NatsEventBusStreamDescriptor,
) -> Vec<NatsEventBusHeaderPlan> {
    vec![
        header("Nats-Msg-Id", &envelope.idempotency_key),
        header("Nats-Expected-Stream", &descriptor.stream_name_ref),
        header("ce-specversion", &envelope.cloudevents_specversion),
        header("ce-id", &envelope.event_id),
        header("ce-type", &envelope.event_type),
        header("ce-source", &envelope.source_ref),
        header(
            "Oya-AsyncAPI-Channel-Ref",
            envelope
                .asyncapi_channel_ref
                .as_deref()
                .unwrap_or("missing"),
        ),
        header("Oya-Trace-Context-Ref", &envelope.trace_context_ref),
        header("Oya-Audit-Chain-Ref", &envelope.audit_chain_ref),
    ]
}

fn validate_stream_descriptor(
    descriptor: &NatsEventBusStreamDescriptor,
) -> Result<(), NatsEventBusPlanFailure> {
    if !is_safe_tenant(&descriptor.tenant_id) {
        return Err(NatsEventBusPlanFailure::InvalidTenant);
    }
    if !is_safe_nats_name(&descriptor.stream_name_ref)
        || !is_safe_nats_subject(&descriptor.subject_ref)
        || !is_safe_optional_ref(descriptor.placement_ref.as_deref())
        || descriptor.replicas == 0
        || descriptor.replicas > NATS_EVENT_BUS_MAX_REPLICAS
        || descriptor.max_age_ms == 0
        || descriptor.duplicate_window_ms == 0
        || descriptor.duplicate_window_ms > NATS_EVENT_BUS_MAX_DUPLICATE_WINDOW_MS
        || !is_valid_storage(&descriptor.storage)
        || !is_valid_retention(&descriptor.retention)
        || !is_valid_discard(&descriptor.discard)
    {
        return Err(NatsEventBusPlanFailure::InvalidStreamDescriptor);
    }
    validate_evidence(&descriptor.evidence_refs)
}

fn validate_consumer_descriptor(
    descriptor: &NatsEventBusConsumerDescriptor,
) -> Result<(), NatsEventBusPlanFailure> {
    if !is_safe_tenant(&descriptor.tenant_id) {
        return Err(NatsEventBusPlanFailure::InvalidTenant);
    }
    if !is_safe_nats_name(&descriptor.stream_name_ref)
        || !is_safe_nats_name(&descriptor.durable_consumer_ref)
        || !is_safe_nats_subject(&descriptor.filter_subject_ref)
        || !is_safe_optional_ref(descriptor.metadata_ref.as_deref())
        || !is_valid_ack_policy(&descriptor.ack_policy)
        || !is_valid_deliver_policy(&descriptor.deliver_policy)
        || !is_valid_replay_policy(&descriptor.replay_policy)
        || descriptor.max_ack_pending == 0
        || descriptor.max_ack_pending > NATS_EVENT_BUS_MAX_ACK_PENDING
        || descriptor.max_deliver == 0
        || descriptor.ack_wait_ms == 0
        || descriptor.replicas > NATS_EVENT_BUS_MAX_REPLICAS
    {
        return Err(NatsEventBusPlanFailure::InvalidConsumerDescriptor);
    }
    validate_evidence(&descriptor.evidence_refs)
}

fn validate_offset_descriptor(
    descriptor: &NatsEventBusOffsetDescriptor,
) -> Result<(), NatsEventBusPlanFailure> {
    if !is_safe_tenant(&descriptor.tenant_id) {
        return Err(NatsEventBusPlanFailure::InvalidTenant);
    }
    if !is_safe_nats_name(&descriptor.stream_name_ref)
        || !is_safe_nats_name(&descriptor.durable_consumer_ref)
        || !is_safe_ref(&descriptor.stream_sequence_ref)
        || !is_safe_ref(&descriptor.consumer_sequence_ref)
        || descriptor.offset_commit_planned
    {
        return Err(NatsEventBusPlanFailure::InvalidReference);
    }
    validate_evidence(&descriptor.evidence_refs)
}

fn validate_publish_envelope(
    envelope: &WorkflowEventBusAdapterPublishEnvelope,
) -> Result<(), NatsEventBusPlanFailure> {
    if !is_safe_tenant(&envelope.tenant_id)
        || !is_safe_ref(&envelope.cell_id)
        || !is_safe_nats_subject(&envelope.channel_address)
        || !is_safe_ref(&envelope.event_id)
        || !is_safe_metadata(&envelope.event_type)
        || !is_safe_ref(&envelope.source_ref)
        || !is_safe_optional_ref(envelope.subject_ref.as_deref())
        || !is_safe_ref(&envelope.partition_key_ref)
        || !is_safe_ref(&envelope.payload_ref)
        || !is_safe_ref(&envelope.idempotency_key)
        || !is_safe_ref(&envelope.trace_context_ref)
        || !is_safe_ref(&envelope.audit_chain_ref)
        || !is_safe_asyncapi_channel_ref(envelope.asyncapi_channel_ref.as_deref())
        || envelope.cloudevents_specversion != WORKFLOW_EVENT_BUS_CLOUDEVENTS_SPECVERSION
    {
        return Err(NatsEventBusPlanFailure::InvalidEnvelope);
    }
    validate_evidence(&envelope.evidence_refs)
}

fn validate_delivery_envelope(
    envelope: &WorkflowEventBusAdapterDeliveryEnvelope,
) -> Result<(), NatsEventBusPlanFailure> {
    if !is_safe_tenant(&envelope.tenant_id)
        || !is_safe_ref(&envelope.cell_id)
        || !is_safe_nats_subject(&envelope.channel_address)
        || !is_safe_ref(&envelope.event_id)
        || !is_safe_metadata(&envelope.event_type)
        || !is_safe_ref(&envelope.consumer_ref)
        || !is_safe_ref(&envelope.offset_ref)
        || !is_safe_ref(&envelope.payload_ref)
        || !is_safe_ref(&envelope.idempotency_key)
        || !is_safe_optional_ref(envelope.replay_cursor_ref.as_deref())
        || !is_safe_ref(&envelope.trace_context_ref)
        || !is_safe_ref(&envelope.audit_chain_ref)
    {
        return Err(NatsEventBusPlanFailure::InvalidEnvelope);
    }
    validate_evidence(&envelope.evidence_refs)
}

fn validate_evidence(values: &[String]) -> Result<(), NatsEventBusPlanFailure> {
    if values.is_empty() || !values.iter().all(|value| is_safe_ref(value)) {
        return Err(NatsEventBusPlanFailure::UnsafeMetadata);
    }
    Ok(())
}

fn is_valid_storage(value: &str) -> bool {
    matches!(value, "file" | "memory")
}

fn is_valid_retention(value: &str) -> bool {
    matches!(value, "limits" | "interest" | "workqueue")
}

fn is_valid_discard(value: &str) -> bool {
    matches!(value, "old" | "new")
}

fn is_valid_ack_policy(value: &str) -> bool {
    matches!(value, "explicit" | "none" | "all")
}

fn is_valid_deliver_policy(value: &str) -> bool {
    matches!(
        value,
        "all" | "last" | "new" | "by_start_sequence" | "by_start_time" | "last_per_subject"
    )
}

fn is_valid_replay_policy(value: &str) -> bool {
    matches!(value, "instant" | "original")
}

fn is_safe_tenant(value: &str) -> bool {
    value.starts_with("ten_") && is_safe_metadata(value)
}

fn is_safe_ref(value: &str) -> bool {
    !value.trim().is_empty() && value.contains(':') && is_safe_metadata(value)
}

fn is_safe_optional_ref(value: Option<&str>) -> bool {
    value.is_none_or(is_safe_ref)
}

fn is_safe_asyncapi_channel_ref(value: Option<&str>) -> bool {
    value.is_some_and(|channel| {
        channel.starts_with(WORKFLOW_EVENT_BUS_ASYNCAPI_CONTRACT_REF)
            && channel.contains("#/channels/")
            && is_safe_metadata(channel)
    })
}

fn is_safe_nats_name(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| byte.is_ascii_graphic())
        && !value
            .chars()
            .any(|ch| matches!(ch, '.' | '*' | '>' | '/' | '\\'))
        && is_safe_metadata(value)
}

fn is_safe_nats_subject(value: &str) -> bool {
    !value.trim().is_empty()
        && !value.chars().any(char::is_whitespace)
        && !value.contains("..")
        && !value.starts_with('.')
        && !value.ends_with('.')
        && is_safe_metadata(value)
}

fn is_safe_metadata(value: &str) -> bool {
    !contains_raw_secret_material(value)
        && !contains_raw_content_material(value)
        && !value.chars().any(char::is_control)
        && !value.chars().any(char::is_whitespace)
}

fn contains_raw_secret_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "secret=",
        "token=",
        "password=",
        "credential=",
        "private_key",
        "bearer ",
        "api_key=",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn contains_raw_content_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "raw prompt",
        "raw_prompt",
        "prompt=",
        "raw output",
        "raw_output",
        "output=",
        "raw payload",
        "raw_payload",
        "payload=",
        "document=",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn dead_letter_stream_ref(tenant_id: &str, subject_ref: &str) -> String {
    format!("NATS_DLQ_{}__{}", tenant_id, subject_ref.replace('.', "_"))
}

fn dead_letter_subject_ref(tenant_id: &str, subject_ref: &str) -> String {
    format!("oya.{}.dead_letter.{}", tenant_id, subject_ref)
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn sorted_unique_join(value_sets: Vec<Vec<String>>) -> Vec<String> {
    let mut values = Vec::new();
    for mut set in value_sets {
        values.append(&mut set);
    }
    sorted_unique(values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_configs_and_non_claims_are_plan_only() {
        assert_eq!(
            NATS_EVENT_BUS_ADAPTER_SURFACE,
            "workflow-engine.event-bus.adapter.nats"
        );
        assert_eq!(
            NATS_EVENT_BUS_ADAPTER_MODE_REF,
            "workflow-event-bus-adapter-nats:plan-only-preview"
        );
        assert_eq!(NATS_EVENT_BUS_MAX_REPLICAS, 5);
        assert_eq!(NATS_EVENT_BUS_DEFAULT_ACK_WAIT_MS, 30_000);
        assert_eq!(NATS_EVENT_BUS_DEFAULT_DUPLICATE_WINDOW_MS, 120_000);
        assert!(
            NATS_EVENT_BUS_ADAPTER_NON_CLAIMS
                .contains(&"workflow-event-bus-adapter-nats:no-broker-connection")
        );
        assert!(
            NATS_EVENT_BUS_ADAPTER_NON_CLAIMS
                .contains(&"workflow-event-bus-adapter-nats:no-jetstream-api-runtime")
        );
        assert!(
            NATS_EVENT_BUS_ADAPTER_NON_CLAIMS
                .contains(&"workflow-event-bus-adapter-nats:no-ack-runtime")
        );
        assert!(
            NATS_EVENT_BUS_ADAPTER_NON_CLAIMS
                .contains(&"workflow-event-bus-adapter-nats:no-hyperscaler-claim")
        );
    }

    #[test]
    fn stream_declaration_plan_uses_replicas_retention_storage_and_duplicate_window_guards() {
        let descriptor = stream_descriptor();
        let plan = NatsEventBusAdapter::stream_declaration_plan(&descriptor).unwrap();
        assert_eq!(plan.operation, "STREAM_DECLARATION");
        assert_eq!(plan.stream_ref.as_deref(), Some("WF_EVENTS"));
        assert_eq!(
            plan.subject_ref.as_deref(),
            Some("oya.tenants.workflow.events.>")
        );
        assert_eq!(config_value(&plan, "storage"), Some("file"));
        assert_eq!(config_value(&plan, "retention"), Some("limits"));
        assert_eq!(config_value(&plan, "discard"), Some("old"));
        assert_eq!(config_value(&plan, "replicas"), Some("3"));
        assert_eq!(config_value(&plan, "duplicate_window_ms"), Some("120000"));
        assert_eq!(
            config_value(&plan, "stream_creation_runtime"),
            Some("not-planned")
        );

        let mut invalid = descriptor.clone();
        invalid.replicas = 6;
        assert_eq!(
            NatsEventBusAdapter::stream_declaration_plan(&invalid),
            Err(NatsEventBusPlanFailure::InvalidStreamDescriptor)
        );

        invalid = descriptor;
        invalid.retention = "forever".to_string();
        assert_eq!(
            NatsEventBusAdapter::stream_declaration_plan(&invalid),
            Err(NatsEventBusPlanFailure::InvalidStreamDescriptor)
        );
    }

    #[test]
    fn consumer_config_plan_is_durable_pull_ack_explicit_and_flow_controlled() {
        let descriptor = consumer_descriptor();
        let plan = NatsEventBusAdapter::consumer_config_plan(&descriptor).unwrap();
        assert_eq!(plan.operation, "CONSUMER_CONFIG");
        assert_eq!(plan.stream_ref.as_deref(), Some("WF_EVENTS"));
        assert_eq!(
            plan.durable_consumer_ref.as_deref(),
            Some("WF_ENGINE_DISPATCH")
        );
        assert_eq!(config_value(&plan, "dispatch"), Some("pull"));
        assert_eq!(config_value(&plan, "ack_policy"), Some("explicit"));
        assert_eq!(config_value(&plan, "deliver_policy"), Some("new"));
        assert_eq!(config_value(&plan, "replay_policy"), Some("instant"));
        assert_eq!(config_value(&plan, "max_ack_pending"), Some("1000"));
        assert_eq!(
            config_value(&plan, "consumer_creation_runtime"),
            Some("not-planned")
        );

        let mut invalid = descriptor;
        invalid.max_ack_pending = NATS_EVENT_BUS_MAX_ACK_PENDING + 1;
        assert_eq!(
            NatsEventBusAdapter::consumer_config_plan(&invalid),
            Err(NatsEventBusPlanFailure::InvalidConsumerDescriptor)
        );
    }

    #[test]
    fn publish_message_plan_binds_nats_dedup_cloudevents_asyncapi_and_payload_ref_only() {
        let envelope = publish_envelope();
        let descriptor = stream_descriptor();
        let plan = NatsEventBusAdapter::publish_message_plan(&envelope, &descriptor).unwrap();
        assert_eq!(plan.operation, "PUBLISH_MESSAGE");
        assert_eq!(plan.stream_ref.as_deref(), Some("WF_EVENTS"));
        assert_eq!(
            plan.subject_ref.as_deref(),
            Some("oya.tenants.workflow.events.>")
        );
        assert_eq!(
            plan.message_id_ref.as_deref(),
            Some("idem:workflow-event-001")
        );
        assert_eq!(
            plan.payload_ref.as_deref(),
            Some("payload:workflow-event-001")
        );
        assert_eq!(
            header_value(&plan, "Nats-Msg-Id"),
            Some("idem:workflow-event-001")
        );
        assert_eq!(
            header_value(&plan, "Nats-Expected-Stream"),
            Some("WF_EVENTS")
        );
        assert_eq!(
            header_value(&plan, "ce-specversion"),
            Some(WORKFLOW_EVENT_BUS_CLOUDEVENTS_SPECVERSION)
        );
        assert_eq!(header_value(&plan, "ce-type"), Some("workflow.run.started"));
        assert_eq!(
            header_value(&plan, "Oya-AsyncAPI-Channel-Ref"),
            Some(
                "microservices/workflow-engine/contracts/asyncapi/workflow-events.yaml#/channels/workflow_events"
            )
        );
        assert!(
            plan.headers
                .iter()
                .all(|header| !header.value_ref.contains("payload="))
        );
        assert!(!plan.non_claim_refs.is_empty());

        let mut wrong_tenant = envelope;
        wrong_tenant.tenant_id = "ten_other".to_string();
        assert_eq!(
            NatsEventBusAdapter::publish_message_plan(&wrong_tenant, &descriptor),
            Err(NatsEventBusPlanFailure::InvalidEnvelope)
        );
    }

    #[test]
    fn pull_and_ack_plans_are_bounded_and_do_not_execute_ack_runtime_or_offset_commit() {
        let consumer = consumer_descriptor();
        let pull_plan = NatsEventBusAdapter::pull_request_plan(&consumer, 100).unwrap();
        assert_eq!(pull_plan.operation, "PULL_REQUEST");
        assert_eq!(pull_plan.batch_bound, Some(100));
        assert_eq!(
            config_value(&pull_plan, "pull_runtime"),
            Some("not-planned")
        );
        assert_eq!(
            NatsEventBusAdapter::pull_request_plan(
                &consumer,
                NATS_EVENT_BUS_MAX_PULL_BATCH_SIZE + 1
            ),
            Err(NatsEventBusPlanFailure::InvalidConsumerDescriptor)
        );

        let delivery = delivery_envelope("delivered");
        let ack_plan = NatsEventBusAdapter::ack_plan(&delivery, &consumer).unwrap();
        assert_eq!(ack_plan.operation, "ACK_PLAN");
        assert!(!ack_plan.ack_runtime_planned);
        assert!(!ack_plan.offset_commit_planned);
        assert_eq!(config_value(&ack_plan, "ack_runtime"), Some("not-planned"));
        assert_eq!(
            config_value(&ack_plan, "offset_commit_planned"),
            Some("false")
        );
    }

    #[test]
    fn offset_observation_and_dead_letter_plans_do_not_commit_offsets() {
        let offset = offset_descriptor();
        let plan = NatsEventBusAdapter::offset_observation_plan(&offset).unwrap();
        assert_eq!(plan.operation, "OFFSET_OBSERVATION");
        assert_eq!(plan.sequence_ref.as_deref(), Some("stream-seq:00000042"));
        assert!(!plan.offset_commit_planned);
        assert_eq!(config_value(&plan, "offset_commit_planned"), Some("false"));

        let mut invalid = offset;
        invalid.offset_commit_planned = true;
        assert_eq!(
            NatsEventBusAdapter::offset_observation_plan(&invalid),
            Err(NatsEventBusPlanFailure::InvalidReference)
        );

        let dlq = NatsEventBusAdapter::dead_letter_message_plan(
            &delivery_envelope("max-deliver"),
            &stream_descriptor(),
        )
        .unwrap();
        assert_eq!(dlq.operation, "DEAD_LETTER_MESSAGE");
        assert!(!dlq.offset_commit_planned);
        assert_eq!(
            config_value(&dlq, "dead_letter_runtime"),
            Some("not-planned")
        );
        assert!(
            dlq.stream_ref
                .as_deref()
                .unwrap()
                .starts_with("NATS_DLQ_ten_workflow__")
        );
    }

    #[test]
    fn unsafe_raw_metadata_is_rejected_before_plan_without_echo() {
        let mut descriptor = stream_descriptor();
        descriptor.subject_ref = "oya.tenants.workflow.events.payload=raw".to_string();
        let err = NatsEventBusAdapter::stream_declaration_plan(&descriptor).unwrap_err();
        assert_eq!(err, NatsEventBusPlanFailure::InvalidStreamDescriptor);
        assert!(!format!("{err:?}").contains("payload=raw"));

        let mut envelope = publish_envelope();
        envelope.payload_ref = "payload=raw secret=abc".to_string();
        let err =
            NatsEventBusAdapter::publish_message_plan(&envelope, &stream_descriptor()).unwrap_err();
        assert_eq!(err, NatsEventBusPlanFailure::InvalidEnvelope);
        assert!(!format!("{err:?}").contains("secret=abc"));
    }

    #[test]
    fn api_generic_adapter_and_nats_plans_integrate_without_runtime_claims() {
        let mut api = WorkflowEventBusApi::default();
        let publish = api
            .publish_event(publish_request("idem:event-bus-nats:publish"))
            .unwrap();
        assert_eq!(publish.status, WorkflowEventBusApiStatus::Accepted);
        let delivery = api
            .evaluate_delivery(delivery_request("idem:event-bus-nats:delivery"))
            .unwrap();
        assert_eq!(delivery.status, WorkflowEventBusApiStatus::Accepted);

        let mut memory = WorkflowEventBusMemoryAdapter::default();
        let publish_receipt = memory
            .record_publish_from_api_success(&publish, publish_envelope_from_api(&publish))
            .unwrap();
        assert_eq!(
            publish_receipt.status,
            WorkflowEventBusAdapterReceiptStatus::Recorded
        );
        let delivery_receipt = memory
            .record_delivery_from_api_success(&delivery, delivery_envelope_from_api(&delivery))
            .unwrap();
        assert!(!delivery_receipt.offset_commit_planned);

        let mut adapter = NatsEventBusAdapter::default();
        adapter
            .plan_stream_declaration(&stream_descriptor())
            .unwrap();
        adapter
            .plan_publish_message(&publish_envelope(), &stream_descriptor())
            .unwrap();
        adapter
            .plan_consumer_config(&consumer_descriptor())
            .unwrap();
        assert_eq!(adapter.generated_plans().len(), 3);
        assert!(adapter.generated_plans().iter().all(|plan| {
            plan.non_claim_refs
                .contains(&"workflow-event-bus-adapter-nats:no-broker-connection".to_string())
                && plan
                    .non_claim_refs
                    .contains(&"workflow-event-bus-adapter-nats:no-cloud-runtime".to_string())
                && plan
                    .non_claim_refs
                    .contains(&"workflow-event-bus-adapter-nats:no-hyperscaler-claim".to_string())
        }));
    }

    fn config_value<'a>(plan: &'a NatsEventBusCommandPlan, key: &str) -> Option<&'a str> {
        plan.configs
            .iter()
            .find(|entry| entry.key == key)
            .map(|entry| entry.value.as_str())
    }

    fn header_value<'a>(plan: &'a NatsEventBusCommandPlan, name: &str) -> Option<&'a str> {
        plan.headers
            .iter()
            .find(|header| header.name == name)
            .map(|header| header.value_ref.as_str())
    }

    fn stream_descriptor() -> NatsEventBusStreamDescriptor {
        NatsEventBusStreamDescriptor {
            tenant_id: "ten_workflow".to_string(),
            stream_name_ref: "WF_EVENTS".to_string(),
            subject_ref: "oya.tenants.workflow.events.>".to_string(),
            storage: "file".to_string(),
            retention: "limits".to_string(),
            discard: "old".to_string(),
            replicas: 3,
            max_age_ms: 86_400_000,
            duplicate_window_ms: NATS_EVENT_BUS_DEFAULT_DUPLICATE_WINDOW_MS,
            placement_ref: Some("placement:nats-cell-a".to_string()),
            evidence_refs: vec!["evidence:nats-stream-declaration".to_string()],
        }
    }

    fn consumer_descriptor() -> NatsEventBusConsumerDescriptor {
        NatsEventBusConsumerDescriptor {
            tenant_id: "ten_workflow".to_string(),
            stream_name_ref: "WF_EVENTS".to_string(),
            durable_consumer_ref: "WF_ENGINE_DISPATCH".to_string(),
            filter_subject_ref: "oya.tenants.workflow.events.run".to_string(),
            ack_policy: "explicit".to_string(),
            deliver_policy: "new".to_string(),
            replay_policy: "instant".to_string(),
            max_ack_pending: 1000,
            max_deliver: 5,
            ack_wait_ms: NATS_EVENT_BUS_DEFAULT_ACK_WAIT_MS,
            replicas: 3,
            metadata_ref: Some("metadata:nats-consumer".to_string()),
            evidence_refs: vec!["evidence:nats-consumer-config".to_string()],
        }
    }

    fn offset_descriptor() -> NatsEventBusOffsetDescriptor {
        NatsEventBusOffsetDescriptor {
            tenant_id: "ten_workflow".to_string(),
            stream_name_ref: "WF_EVENTS".to_string(),
            durable_consumer_ref: "WF_ENGINE_DISPATCH".to_string(),
            stream_sequence_ref: "stream-seq:00000042".to_string(),
            consumer_sequence_ref: "consumer-seq:00000007".to_string(),
            offset_commit_planned: false,
            evidence_refs: vec!["evidence:nats-offset-observation".to_string()],
        }
    }

    fn publish_envelope() -> WorkflowEventBusAdapterPublishEnvelope {
        WorkflowEventBusAdapterPublishEnvelope {
            tenant_id: "ten_workflow".to_string(),
            cell_id: "cell:use1-a".to_string(),
            channel_address: "oya.tenants.workflow.events.run".to_string(),
            event_id: "event:workflow-run-started-001".to_string(),
            event_type: "workflow.run.started".to_string(),
            source_ref: "workflow-engine:event-bus".to_string(),
            subject_ref: Some("workflow-run:run-001".to_string()),
            partition_key_ref: "partition:workflow-run-001".to_string(),
            payload_ref: "payload:workflow-event-001".to_string(),
            idempotency_key: "idem:workflow-event-001".to_string(),
            trace_context_ref: "trace:workflow-event-001".to_string(),
            audit_chain_ref: "audit:workflow-event-001".to_string(),
            asyncapi_channel_ref: Some(format!(
                "{}#/channels/workflow_events",
                WORKFLOW_EVENT_BUS_ASYNCAPI_CONTRACT_REF
            )),
            cloudevents_specversion: WORKFLOW_EVENT_BUS_CLOUDEVENTS_SPECVERSION.to_string(),
            evidence_refs: vec!["evidence:nats-publish".to_string()],
        }
    }

    fn delivery_envelope(delivery_status: &str) -> WorkflowEventBusAdapterDeliveryEnvelope {
        WorkflowEventBusAdapterDeliveryEnvelope {
            tenant_id: "ten_workflow".to_string(),
            cell_id: "cell:use1-a".to_string(),
            channel_address: "oya.tenants.workflow.events.run".to_string(),
            event_id: "event:workflow-run-started-001".to_string(),
            event_type: "workflow.run.started".to_string(),
            consumer_ref: "consumer:nats-workflow".to_string(),
            offset_ref: "offset:nats-seq-42".to_string(),
            payload_ref: "payload:workflow-event-001".to_string(),
            idempotency_key: format!("idem:nats-delivery-{delivery_status}"),
            replay_cursor_ref: Some("cursor:nats-seq-42".to_string()),
            trace_context_ref: "trace:nats-delivery".to_string(),
            audit_chain_ref: "audit:nats-delivery".to_string(),
            evidence_refs: vec![format!("evidence:nats-delivery-{delivery_status}")],
        }
    }

    fn publish_envelope_from_api(
        success: &WorkflowEventBusApiSuccessResponse,
    ) -> WorkflowEventBusAdapterPublishEnvelope {
        let mut envelope = publish_envelope();
        envelope.idempotency_key = success.metadata.idempotency_key.clone();
        envelope.trace_context_ref = success.metadata.trace_context_ref.clone();
        envelope.tenant_id = success.event.tenant_id.clone();
        envelope.cell_id = success.event.cell_id.clone();
        envelope.channel_address = success.event.channel_address.clone().unwrap();
        envelope.event_type = success.event.event_type.clone();
        envelope.asyncapi_channel_ref = success.event.asyncapi_channel_ref.clone();
        envelope
    }

    fn delivery_envelope_from_api(
        success: &WorkflowEventBusApiSuccessResponse,
    ) -> WorkflowEventBusAdapterDeliveryEnvelope {
        let mut envelope = delivery_envelope(&success.event.usecase_status);
        envelope.idempotency_key = success.metadata.idempotency_key.clone();
        envelope.trace_context_ref = success.metadata.trace_context_ref.clone();
        envelope.tenant_id = success.event.tenant_id.clone();
        envelope.cell_id = success.event.cell_id.clone();
        envelope.channel_address = success.event.channel_address.clone().unwrap();
        envelope.event_type = success.event.event_type.clone();
        envelope.consumer_ref = success.event.consumer_ref.clone().unwrap();
        envelope.offset_ref = success.event.offset_ref.clone().unwrap();
        envelope
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
            request_id: format!("request:event-bus-nats:{idempotency_key}"),
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
