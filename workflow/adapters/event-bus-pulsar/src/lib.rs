//! Workflow-engine event-bus Apache Pulsar adapter preview foundation.
//!
//! This crate is a source-level, plan-only adapter around the generic event-bus
//! adapter seam. It models Pulsar tenant/namespace/topic, producer, message,
//! subscription, receive, ack, offset-observation, and dead-letter metadata
//! without opening a Pulsar connection or invoking the Pulsar Admin/client APIs.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use workflow_event_bus_adapter::{
    WORKFLOW_EVENT_BUS_ADAPTER_MODE_REF, WORKFLOW_EVENT_BUS_ADAPTER_SURFACE,
    WORKFLOW_EVENT_BUS_API_DECLARED_VERSION, WORKFLOW_EVENT_BUS_API_DELIVERY_ROUTE,
    WORKFLOW_EVENT_BUS_API_METHOD, WORKFLOW_EVENT_BUS_API_PUBLISH_ROUTE,
    WORKFLOW_EVENT_BUS_API_SURFACE, WORKFLOW_EVENT_BUS_ASYNCAPI_CONTRACT_REF,
    WORKFLOW_EVENT_BUS_CLOUDEVENTS_SPECVERSION, WorkflowEventBusAdapterDeliveryEnvelope,
    WorkflowEventBusAdapterPublishEnvelope, WorkflowEventBusAdapterReceiptStatus,
    WorkflowEventBusApi, WorkflowEventBusApiAuthorization, WorkflowEventBusApiBoundaryContext,
    WorkflowEventBusApiDeliveryBody, WorkflowEventBusApiDeliveryRequest,
    WorkflowEventBusApiPrincipal, WorkflowEventBusApiPublishBody,
    WorkflowEventBusApiPublishRequest, WorkflowEventBusApiStatus,
    WorkflowEventBusApiSuccessResponse, WorkflowEventBusEventKind, WorkflowEventBusMemoryAdapter,
};

pub const PULSAR_EVENT_BUS_ADAPTER_SURFACE: &str = "workflow-engine.event-bus.adapter.pulsar";
pub const PULSAR_EVENT_BUS_ADAPTER_MODE_REF: &str =
    "workflow-event-bus-adapter-pulsar:plan-only-preview";
pub const PULSAR_EVENT_BUS_MAX_PARTITIONS: u32 = 4096;
pub const PULSAR_EVENT_BUS_MAX_RECEIVE_BATCH_SIZE: u32 = 1000;
pub const PULSAR_EVENT_BUS_MAX_RECEIVER_QUEUE_SIZE: u32 = 50_000;
pub const PULSAR_EVENT_BUS_DEFAULT_ACK_TIMEOUT_MS: u64 = 30_000;
pub const PULSAR_EVENT_BUS_DEFAULT_NEGATIVE_ACK_REDELIVERY_DELAY_MS: u64 = 60_000;
pub const PULSAR_EVENT_BUS_MAX_RETENTION_TIME_MS: u64 = 31_536_000_000;

pub const PULSAR_EVENT_BUS_ADAPTER_NON_CLAIMS: [&str; 10] = [
    "workflow-event-bus-adapter-pulsar:no-broker-connection",
    "workflow-event-bus-adapter-pulsar:no-admin-api-runtime",
    "workflow-event-bus-adapter-pulsar:no-namespace-or-topic-creation",
    "workflow-event-bus-adapter-pulsar:no-producer-runtime",
    "workflow-event-bus-adapter-pulsar:no-consumer-runtime",
    "workflow-event-bus-adapter-pulsar:no-receive-runtime",
    "workflow-event-bus-adapter-pulsar:no-ack-runtime",
    "workflow-event-bus-adapter-pulsar:no-message-id-commit-runtime",
    "workflow-event-bus-adapter-pulsar:no-cloud-runtime",
    "workflow-event-bus-adapter-pulsar:no-hyperscaler-claim",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PulsarEventBusPlanKind {
    NamespaceDeclaration,
    TopicDeclaration,
    ProducerConfig,
    ProducerMessage,
    ConsumerSubscription,
    ReceiveBatch,
    AckPlan,
    OffsetObservation,
    DeadLetterTopic,
}

impl PulsarEventBusPlanKind {
    pub const fn operation(self) -> &'static str {
        match self {
            Self::NamespaceDeclaration => "NamespaceDeclarationPlan",
            Self::TopicDeclaration => "TopicDeclarationPlan",
            Self::ProducerConfig => "ProducerConfigPlan",
            Self::ProducerMessage => "ProducerMessagePlan",
            Self::ConsumerSubscription => "ConsumerSubscriptionPlan",
            Self::ReceiveBatch => "ReceiveBatchPlan",
            Self::AckPlan => "AckPlan",
            Self::OffsetObservation => "OffsetObservationPlan",
            Self::DeadLetterTopic => "DeadLetterTopicPlan",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PulsarEventBusConfigEntry {
    pub key: String,   // data_class: PUBLIC
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PulsarEventBusPropertyPlan {
    pub key: String,       // data_class: PUBLIC
    pub value_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PulsarEventBusCommandPlan {
    pub plan_kind: PulsarEventBusPlanKind, // data_class: PUBLIC
    pub operation: &'static str,           // data_class: PUBLIC
    pub tenant_ref: Option<String>,        // data_class: INTERNAL_ONLY
    pub namespace_ref: Option<String>,     // data_class: INTERNAL_ONLY
    pub topic_uri: Option<String>,         // data_class: INTERNAL_ONLY
    pub producer_ref: Option<String>,      // data_class: INTERNAL_ONLY
    pub subscription_ref: Option<String>,  // data_class: INTERNAL_ONLY
    pub consumer_ref: Option<String>,      // data_class: INTERNAL_ONLY
    pub message_id_ref: Option<String>,    // data_class: INTERNAL_ONLY
    pub sequence_id_ref: Option<String>,   // data_class: INTERNAL_ONLY
    pub payload_ref: Option<String>,       // data_class: INTERNAL_ONLY
    pub properties: Vec<PulsarEventBusPropertyPlan>, // data_class: INTERNAL_ONLY
    pub configs: Vec<PulsarEventBusConfigEntry>, // data_class: INTERNAL_ONLY
    pub batch_bound: Option<u32>,          // data_class: INTERNAL_ONLY
    pub ack_runtime_planned: bool,         // data_class: INTERNAL_ONLY
    pub message_id_commit_planned: bool,   // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,        // data_class: INTERNAL_ONLY
    pub non_claim_refs: Vec<String>,       // data_class: INTERNAL_ONLY
}

impl PulsarEventBusCommandPlan {
    fn new(plan_kind: PulsarEventBusPlanKind, evidence_refs: Vec<String>) -> Self {
        Self {
            operation: plan_kind.operation(),
            plan_kind,
            tenant_ref: None,
            namespace_ref: None,
            topic_uri: None,
            producer_ref: None,
            subscription_ref: None,
            consumer_ref: None,
            message_id_ref: None,
            sequence_id_ref: None,
            payload_ref: None,
            properties: Vec::new(),
            configs: Vec::new(),
            batch_bound: None,
            ack_runtime_planned: false,
            message_id_commit_planned: false,
            evidence_refs: sorted_unique(evidence_refs),
            non_claim_refs: PULSAR_EVENT_BUS_ADAPTER_NON_CLAIMS
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
        }
    }

    fn with_tenant_ref(mut self, tenant_ref: String) -> Self {
        self.tenant_ref = Some(tenant_ref);
        self
    }

    fn with_namespace_ref(mut self, namespace_ref: String) -> Self {
        self.namespace_ref = Some(namespace_ref);
        self
    }

    fn with_topic_uri(mut self, topic_uri: String) -> Self {
        self.topic_uri = Some(topic_uri);
        self
    }

    fn with_producer_ref(mut self, producer_ref: String) -> Self {
        self.producer_ref = Some(producer_ref);
        self
    }

    fn with_subscription_ref(mut self, subscription_ref: String) -> Self {
        self.subscription_ref = Some(subscription_ref);
        self
    }

    fn with_consumer_ref(mut self, consumer_ref: String) -> Self {
        self.consumer_ref = Some(consumer_ref);
        self
    }

    fn with_message_id_ref(mut self, message_id_ref: String) -> Self {
        self.message_id_ref = Some(message_id_ref);
        self
    }

    fn with_sequence_id_ref(mut self, sequence_id_ref: String) -> Self {
        self.sequence_id_ref = Some(sequence_id_ref);
        self
    }

    fn with_payload_ref(mut self, payload_ref: String) -> Self {
        self.payload_ref = Some(payload_ref);
        self
    }

    fn with_properties(mut self, properties: Vec<PulsarEventBusPropertyPlan>) -> Self {
        self.properties = properties;
        self
    }

    fn with_configs(mut self, configs: Vec<PulsarEventBusConfigEntry>) -> Self {
        self.configs = configs;
        self
    }

    fn with_batch_bound(mut self, batch_bound: u32) -> Self {
        self.batch_bound = Some(batch_bound);
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PulsarEventBusPlanFailure {
    InvalidTenant,
    InvalidReference,
    InvalidNamespaceDescriptor,
    InvalidTopicDescriptor,
    InvalidProducerDescriptor,
    InvalidConsumerDescriptor,
    InvalidEnvelope,
    UnsafeMetadata,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PulsarEventBusNamespaceDescriptor {
    pub tenant_id: String,           // data_class: INTERNAL_ONLY
    pub pulsar_tenant_ref: String,   // data_class: INTERNAL_ONLY
    pub namespace_ref: String,       // data_class: INTERNAL_ONLY
    pub clusters_ref: String,        // data_class: INTERNAL_ONLY
    pub retention_time_ms: u64,      // data_class: INTERNAL_ONLY
    pub retention_size_bytes: u64,   // data_class: INTERNAL_ONLY
    pub message_ttl_seconds: u64,    // data_class: INTERNAL_ONLY
    pub deduplication_enabled: bool, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PulsarEventBusTopicDescriptor {
    pub tenant_id: String,           // data_class: INTERNAL_ONLY
    pub topic_uri: String,           // data_class: INTERNAL_ONLY
    pub partitions: u32,             // data_class: INTERNAL_ONLY
    pub deduplication_enabled: bool, // data_class: INTERNAL_ONLY
    pub schema_ref: Option<String>,  // data_class: INTERNAL_ONLY
    pub compaction_enabled: bool,    // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PulsarEventBusProducerDescriptor {
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub service_url_ref: String,         // data_class: INTERNAL_ONLY
    pub topic_uri: String,               // data_class: INTERNAL_ONLY
    pub producer_name_ref: String,       // data_class: INTERNAL_ONLY
    pub send_timeout_ms: u64,            // data_class: INTERNAL_ONLY
    pub batching_enabled: bool,          // data_class: INTERNAL_ONLY
    pub key_based_batching: bool,        // data_class: INTERNAL_ONLY
    pub max_pending_messages: u32,       // data_class: INTERNAL_ONLY
    pub compression_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PulsarEventBusConsumerDescriptor {
    pub tenant_id: String,                     // data_class: INTERNAL_ONLY
    pub topic_uri: String,                     // data_class: INTERNAL_ONLY
    pub subscription_name_ref: String,         // data_class: INTERNAL_ONLY
    pub subscription_type: String,             // data_class: PUBLIC
    pub subscription_mode: String,             // data_class: PUBLIC
    pub subscription_initial_position: String, // data_class: PUBLIC
    pub consumer_name_ref: String,             // data_class: INTERNAL_ONLY
    pub receiver_queue_size: u32,              // data_class: INTERNAL_ONLY
    pub ack_timeout_ms: u64,                   // data_class: INTERNAL_ONLY
    pub negative_ack_redelivery_delay_ms: u64, // data_class: INTERNAL_ONLY
    pub batch_index_ack_enabled: bool,         // data_class: INTERNAL_ONLY
    pub dead_letter_topic_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PulsarEventBusOffsetDescriptor {
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub topic_uri: String,               // data_class: INTERNAL_ONLY
    pub subscription_name_ref: String,   // data_class: INTERNAL_ONLY
    pub ledger_id_ref: String,           // data_class: INTERNAL_ONLY
    pub entry_id_ref: String,            // data_class: INTERNAL_ONLY
    pub partition_index_ref: String,     // data_class: INTERNAL_ONLY
    pub message_id_ref: String,          // data_class: INTERNAL_ONLY
    pub message_id_commit_planned: bool, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,      // data_class: INTERNAL_ONLY
}

#[derive(Debug, Default)]
pub struct PulsarEventBusAdapter {
    generated_plans: Vec<PulsarEventBusCommandPlan>,
}

impl PulsarEventBusAdapter {
    pub fn namespace_declaration_plan(
        descriptor: &PulsarEventBusNamespaceDescriptor,
    ) -> Result<PulsarEventBusCommandPlan, PulsarEventBusPlanFailure> {
        validate_namespace_descriptor(descriptor)?;
        Ok(PulsarEventBusCommandPlan::new(
            PulsarEventBusPlanKind::NamespaceDeclaration,
            descriptor.evidence_refs.clone(),
        )
        .with_tenant_ref(descriptor.pulsar_tenant_ref.clone())
        .with_namespace_ref(descriptor.namespace_ref.clone())
        .with_configs(vec![
            config("clusters_ref", &descriptor.clusters_ref),
            config(
                "retention_time_ms",
                &descriptor.retention_time_ms.to_string(),
            ),
            config(
                "retention_size_bytes",
                &descriptor.retention_size_bytes.to_string(),
            ),
            config(
                "message_ttl_seconds",
                &descriptor.message_ttl_seconds.to_string(),
            ),
            config(
                "deduplication_enabled",
                bool_wire(descriptor.deduplication_enabled),
            ),
            config("admin_api_runtime", "not-planned"),
        ]))
    }

    pub fn topic_declaration_plan(
        descriptor: &PulsarEventBusTopicDescriptor,
    ) -> Result<PulsarEventBusCommandPlan, PulsarEventBusPlanFailure> {
        validate_topic_descriptor(descriptor)?;
        Ok(PulsarEventBusCommandPlan::new(
            PulsarEventBusPlanKind::TopicDeclaration,
            descriptor.evidence_refs.clone(),
        )
        .with_topic_uri(descriptor.topic_uri.clone())
        .with_configs(vec![
            config("partitions", &descriptor.partitions.to_string()),
            config(
                "deduplication_enabled",
                bool_wire(descriptor.deduplication_enabled),
            ),
            config(
                "schema_ref",
                descriptor.schema_ref.as_deref().unwrap_or("schema:none"),
            ),
            config(
                "compaction_enabled",
                bool_wire(descriptor.compaction_enabled),
            ),
            config("topic_creation_runtime", "not-planned"),
        ]))
    }

    pub fn producer_config_plan(
        descriptor: &PulsarEventBusProducerDescriptor,
    ) -> Result<PulsarEventBusCommandPlan, PulsarEventBusPlanFailure> {
        validate_producer_descriptor(descriptor)?;
        Ok(PulsarEventBusCommandPlan::new(
            PulsarEventBusPlanKind::ProducerConfig,
            descriptor.evidence_refs.clone(),
        )
        .with_topic_uri(descriptor.topic_uri.clone())
        .with_producer_ref(descriptor.producer_name_ref.clone())
        .with_configs(vec![
            config("service_url_ref", &descriptor.service_url_ref),
            config("producer_name_ref", &descriptor.producer_name_ref),
            config("send_timeout_ms", &descriptor.send_timeout_ms.to_string()),
            config("batching_enabled", bool_wire(descriptor.batching_enabled)),
            config(
                "key_based_batching",
                bool_wire(descriptor.key_based_batching),
            ),
            config(
                "max_pending_messages",
                &descriptor.max_pending_messages.to_string(),
            ),
            config("producer_runtime", "not-planned"),
        ]))
    }

    pub fn publish_message_plan(
        envelope: &WorkflowEventBusAdapterPublishEnvelope,
        descriptor: &PulsarEventBusProducerDescriptor,
    ) -> Result<PulsarEventBusCommandPlan, PulsarEventBusPlanFailure> {
        validate_publish_envelope(envelope)?;
        validate_producer_descriptor(descriptor)?;
        if envelope.tenant_id != descriptor.tenant_id {
            return Err(PulsarEventBusPlanFailure::InvalidEnvelope);
        }
        Ok(PulsarEventBusCommandPlan::new(
            PulsarEventBusPlanKind::ProducerMessage,
            sorted_unique_join(vec![
                envelope.evidence_refs.clone(),
                descriptor.evidence_refs.clone(),
            ]),
        )
        .with_topic_uri(descriptor.topic_uri.clone())
        .with_producer_ref(descriptor.producer_name_ref.clone())
        .with_sequence_id_ref(envelope.idempotency_key.clone())
        .with_payload_ref(envelope.payload_ref.clone())
        .with_properties(publish_properties(envelope)))
    }

    pub fn consumer_subscription_plan(
        descriptor: &PulsarEventBusConsumerDescriptor,
    ) -> Result<PulsarEventBusCommandPlan, PulsarEventBusPlanFailure> {
        validate_consumer_descriptor(descriptor)?;
        Ok(PulsarEventBusCommandPlan::new(
            PulsarEventBusPlanKind::ConsumerSubscription,
            descriptor.evidence_refs.clone(),
        )
        .with_topic_uri(descriptor.topic_uri.clone())
        .with_subscription_ref(descriptor.subscription_name_ref.clone())
        .with_consumer_ref(descriptor.consumer_name_ref.clone())
        .with_configs(vec![
            config("subscription_type", &descriptor.subscription_type),
            config("subscription_mode", &descriptor.subscription_mode),
            config(
                "subscription_initial_position",
                &descriptor.subscription_initial_position,
            ),
            config(
                "receiver_queue_size",
                &descriptor.receiver_queue_size.to_string(),
            ),
            config("ack_timeout_ms", &descriptor.ack_timeout_ms.to_string()),
            config(
                "negative_ack_redelivery_delay_ms",
                &descriptor.negative_ack_redelivery_delay_ms.to_string(),
            ),
            config(
                "batch_index_ack_enabled",
                bool_wire(descriptor.batch_index_ack_enabled),
            ),
            config("consumer_runtime", "not-planned"),
        ]))
    }

    pub fn receive_batch_plan(
        descriptor: &PulsarEventBusConsumerDescriptor,
        batch_bound: u32,
    ) -> Result<PulsarEventBusCommandPlan, PulsarEventBusPlanFailure> {
        validate_consumer_descriptor(descriptor)?;
        if batch_bound == 0 || batch_bound > PULSAR_EVENT_BUS_MAX_RECEIVE_BATCH_SIZE {
            return Err(PulsarEventBusPlanFailure::InvalidConsumerDescriptor);
        }
        Ok(PulsarEventBusCommandPlan::new(
            PulsarEventBusPlanKind::ReceiveBatch,
            descriptor.evidence_refs.clone(),
        )
        .with_topic_uri(descriptor.topic_uri.clone())
        .with_subscription_ref(descriptor.subscription_name_ref.clone())
        .with_consumer_ref(descriptor.consumer_name_ref.clone())
        .with_batch_bound(batch_bound)
        .with_configs(vec![
            config("receive_batch", &batch_bound.to_string()),
            config("receive_runtime", "not-planned"),
        ]))
    }

    pub fn ack_plan(
        delivery: &WorkflowEventBusAdapterDeliveryEnvelope,
        descriptor: &PulsarEventBusConsumerDescriptor,
    ) -> Result<PulsarEventBusCommandPlan, PulsarEventBusPlanFailure> {
        validate_delivery_envelope(delivery)?;
        validate_consumer_descriptor(descriptor)?;
        if delivery.tenant_id != descriptor.tenant_id {
            return Err(PulsarEventBusPlanFailure::InvalidEnvelope);
        }
        Ok(PulsarEventBusCommandPlan::new(
            PulsarEventBusPlanKind::AckPlan,
            sorted_unique_join(vec![
                delivery.evidence_refs.clone(),
                descriptor.evidence_refs.clone(),
            ]),
        )
        .with_topic_uri(descriptor.topic_uri.clone())
        .with_subscription_ref(descriptor.subscription_name_ref.clone())
        .with_consumer_ref(descriptor.consumer_name_ref.clone())
        .with_message_id_ref(delivery.offset_ref.clone())
        .with_payload_ref(delivery.payload_ref.clone())
        .with_configs(vec![
            config("acknowledgement", "individual"),
            config("ack_runtime", "not-planned"),
            config("message_id_commit_planned", "false"),
        ]))
    }

    pub fn offset_observation_plan(
        descriptor: &PulsarEventBusOffsetDescriptor,
    ) -> Result<PulsarEventBusCommandPlan, PulsarEventBusPlanFailure> {
        validate_offset_descriptor(descriptor)?;
        Ok(PulsarEventBusCommandPlan::new(
            PulsarEventBusPlanKind::OffsetObservation,
            descriptor.evidence_refs.clone(),
        )
        .with_topic_uri(descriptor.topic_uri.clone())
        .with_subscription_ref(descriptor.subscription_name_ref.clone())
        .with_message_id_ref(descriptor.message_id_ref.clone())
        .with_configs(vec![
            config("ledger_id_ref", &descriptor.ledger_id_ref),
            config("entry_id_ref", &descriptor.entry_id_ref),
            config("partition_index_ref", &descriptor.partition_index_ref),
            config("message_id_commit_planned", "false"),
        ]))
    }

    pub fn dead_letter_topic_plan(
        delivery: &WorkflowEventBusAdapterDeliveryEnvelope,
        descriptor: &PulsarEventBusConsumerDescriptor,
    ) -> Result<PulsarEventBusCommandPlan, PulsarEventBusPlanFailure> {
        validate_delivery_envelope(delivery)?;
        validate_consumer_descriptor(descriptor)?;
        if delivery.tenant_id != descriptor.tenant_id {
            return Err(PulsarEventBusPlanFailure::InvalidEnvelope);
        }
        let dead_letter_topic = descriptor
            .dead_letter_topic_ref
            .clone()
            .unwrap_or_else(|| dead_letter_topic_uri(&descriptor.topic_uri));
        Ok(PulsarEventBusCommandPlan::new(
            PulsarEventBusPlanKind::DeadLetterTopic,
            sorted_unique_join(vec![
                delivery.evidence_refs.clone(),
                descriptor.evidence_refs.clone(),
            ]),
        )
        .with_topic_uri(dead_letter_topic)
        .with_subscription_ref(descriptor.subscription_name_ref.clone())
        .with_message_id_ref(delivery.event_id.clone())
        .with_payload_ref(delivery.payload_ref.clone())
        .with_configs(vec![
            config("dead_letter_runtime", "not-planned"),
            config("message_id_commit_planned", "false"),
        ]))
    }

    pub fn plan_topic_declaration(
        &mut self,
        descriptor: &PulsarEventBusTopicDescriptor,
    ) -> Result<&PulsarEventBusCommandPlan, PulsarEventBusPlanFailure> {
        let plan = Self::topic_declaration_plan(descriptor)?;
        self.generated_plans.push(plan);
        self.generated_plans
            .last()
            .ok_or(PulsarEventBusPlanFailure::InvalidTopicDescriptor)
    }

    pub fn plan_publish_message(
        &mut self,
        envelope: &WorkflowEventBusAdapterPublishEnvelope,
        descriptor: &PulsarEventBusProducerDescriptor,
    ) -> Result<&PulsarEventBusCommandPlan, PulsarEventBusPlanFailure> {
        let plan = Self::publish_message_plan(envelope, descriptor)?;
        self.generated_plans.push(plan);
        self.generated_plans
            .last()
            .ok_or(PulsarEventBusPlanFailure::InvalidEnvelope)
    }

    pub fn plan_consumer_subscription(
        &mut self,
        descriptor: &PulsarEventBusConsumerDescriptor,
    ) -> Result<&PulsarEventBusCommandPlan, PulsarEventBusPlanFailure> {
        let plan = Self::consumer_subscription_plan(descriptor)?;
        self.generated_plans.push(plan);
        self.generated_plans
            .last()
            .ok_or(PulsarEventBusPlanFailure::InvalidConsumerDescriptor)
    }

    pub fn generated_plans(&self) -> &[PulsarEventBusCommandPlan] {
        &self.generated_plans
    }
}

fn config(key: &str, value: &str) -> PulsarEventBusConfigEntry {
    PulsarEventBusConfigEntry {
        key: key.to_string(),
        value: value.to_string(),
    }
}

fn property(key: &str, value_ref: &str) -> PulsarEventBusPropertyPlan {
    PulsarEventBusPropertyPlan {
        key: key.to_string(),
        value_ref: value_ref.to_string(),
    }
}

fn bool_wire(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn publish_properties(
    envelope: &WorkflowEventBusAdapterPublishEnvelope,
) -> Vec<PulsarEventBusPropertyPlan> {
    vec![
        property("ce-specversion", &envelope.cloudevents_specversion),
        property("ce-id", &envelope.event_id),
        property("ce-type", &envelope.event_type),
        property("ce-source", &envelope.source_ref),
        property(
            "Oya-AsyncAPI-Channel-Ref",
            envelope
                .asyncapi_channel_ref
                .as_deref()
                .unwrap_or("missing"),
        ),
        property("Oya-Trace-Context-Ref", &envelope.trace_context_ref),
        property("Oya-Audit-Chain-Ref", &envelope.audit_chain_ref),
        property("Oya-Partition-Key-Ref", &envelope.partition_key_ref),
    ]
}

fn validate_namespace_descriptor(
    descriptor: &PulsarEventBusNamespaceDescriptor,
) -> Result<(), PulsarEventBusPlanFailure> {
    if !is_safe_tenant(&descriptor.tenant_id) {
        return Err(PulsarEventBusPlanFailure::InvalidTenant);
    }
    if !is_safe_pulsar_name(&descriptor.pulsar_tenant_ref)
        || !is_safe_namespace(&descriptor.namespace_ref)
        || !is_safe_ref(&descriptor.clusters_ref)
        || descriptor.retention_time_ms == 0
        || descriptor.retention_time_ms > PULSAR_EVENT_BUS_MAX_RETENTION_TIME_MS
        || descriptor.retention_size_bytes == 0
        || descriptor.message_ttl_seconds == 0
    {
        return Err(PulsarEventBusPlanFailure::InvalidNamespaceDescriptor);
    }
    validate_evidence(&descriptor.evidence_refs)
}

fn validate_topic_descriptor(
    descriptor: &PulsarEventBusTopicDescriptor,
) -> Result<(), PulsarEventBusPlanFailure> {
    if !is_safe_tenant(&descriptor.tenant_id) {
        return Err(PulsarEventBusPlanFailure::InvalidTenant);
    }
    if !is_safe_topic_uri(&descriptor.topic_uri)
        || descriptor.partitions == 0
        || descriptor.partitions > PULSAR_EVENT_BUS_MAX_PARTITIONS
        || !is_safe_optional_ref(descriptor.schema_ref.as_deref())
    {
        return Err(PulsarEventBusPlanFailure::InvalidTopicDescriptor);
    }
    validate_evidence(&descriptor.evidence_refs)
}

fn validate_producer_descriptor(
    descriptor: &PulsarEventBusProducerDescriptor,
) -> Result<(), PulsarEventBusPlanFailure> {
    if !is_safe_tenant(&descriptor.tenant_id) {
        return Err(PulsarEventBusPlanFailure::InvalidTenant);
    }
    if !is_safe_connection_ref(&descriptor.service_url_ref)
        || !is_safe_topic_uri(&descriptor.topic_uri)
        || !is_safe_pulsar_name(&descriptor.producer_name_ref)
        || descriptor.send_timeout_ms != 0
        || descriptor.max_pending_messages == 0
        || descriptor.max_pending_messages > PULSAR_EVENT_BUS_MAX_RECEIVER_QUEUE_SIZE
        || !is_safe_optional_ref(descriptor.compression_ref.as_deref())
    {
        return Err(PulsarEventBusPlanFailure::InvalidProducerDescriptor);
    }
    validate_evidence(&descriptor.evidence_refs)
}

fn validate_consumer_descriptor(
    descriptor: &PulsarEventBusConsumerDescriptor,
) -> Result<(), PulsarEventBusPlanFailure> {
    if !is_safe_tenant(&descriptor.tenant_id) {
        return Err(PulsarEventBusPlanFailure::InvalidTenant);
    }
    if !is_safe_topic_uri(&descriptor.topic_uri)
        || !is_safe_pulsar_name(&descriptor.subscription_name_ref)
        || !is_valid_subscription_type(&descriptor.subscription_type)
        || !is_valid_subscription_mode(&descriptor.subscription_mode)
        || !is_valid_initial_position(&descriptor.subscription_initial_position)
        || !is_safe_pulsar_name(&descriptor.consumer_name_ref)
        || descriptor.receiver_queue_size == 0
        || descriptor.receiver_queue_size > PULSAR_EVENT_BUS_MAX_RECEIVER_QUEUE_SIZE
        || descriptor.ack_timeout_ms == 0
        || descriptor.negative_ack_redelivery_delay_ms == 0
        || !is_safe_optional_topic_uri(descriptor.dead_letter_topic_ref.as_deref())
    {
        return Err(PulsarEventBusPlanFailure::InvalidConsumerDescriptor);
    }
    validate_evidence(&descriptor.evidence_refs)
}

fn validate_offset_descriptor(
    descriptor: &PulsarEventBusOffsetDescriptor,
) -> Result<(), PulsarEventBusPlanFailure> {
    if !is_safe_tenant(&descriptor.tenant_id) {
        return Err(PulsarEventBusPlanFailure::InvalidTenant);
    }
    if !is_safe_topic_uri(&descriptor.topic_uri)
        || !is_safe_pulsar_name(&descriptor.subscription_name_ref)
        || !is_safe_ref(&descriptor.ledger_id_ref)
        || !is_safe_ref(&descriptor.entry_id_ref)
        || !is_safe_ref(&descriptor.partition_index_ref)
        || !is_safe_ref(&descriptor.message_id_ref)
        || descriptor.message_id_commit_planned
    {
        return Err(PulsarEventBusPlanFailure::InvalidReference);
    }
    validate_evidence(&descriptor.evidence_refs)
}

fn validate_publish_envelope(
    envelope: &WorkflowEventBusAdapterPublishEnvelope,
) -> Result<(), PulsarEventBusPlanFailure> {
    if !is_safe_tenant(&envelope.tenant_id)
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
        || !is_safe_asyncapi_channel_ref(envelope.asyncapi_channel_ref.as_deref())
        || envelope.cloudevents_specversion != WORKFLOW_EVENT_BUS_CLOUDEVENTS_SPECVERSION
    {
        return Err(PulsarEventBusPlanFailure::InvalidEnvelope);
    }
    validate_evidence(&envelope.evidence_refs)
}

fn validate_delivery_envelope(
    envelope: &WorkflowEventBusAdapterDeliveryEnvelope,
) -> Result<(), PulsarEventBusPlanFailure> {
    if !is_safe_tenant(&envelope.tenant_id)
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
    {
        return Err(PulsarEventBusPlanFailure::InvalidEnvelope);
    }
    validate_evidence(&envelope.evidence_refs)
}

fn validate_evidence(values: &[String]) -> Result<(), PulsarEventBusPlanFailure> {
    if values.is_empty() || !values.iter().all(|value| is_safe_ref(value)) {
        return Err(PulsarEventBusPlanFailure::UnsafeMetadata);
    }
    Ok(())
}

fn is_valid_subscription_type(value: &str) -> bool {
    matches!(value, "exclusive" | "shared" | "failover" | "key_shared")
}

fn is_valid_subscription_mode(value: &str) -> bool {
    matches!(value, "durable" | "non_durable")
}

fn is_valid_initial_position(value: &str) -> bool {
    matches!(value, "earliest" | "latest")
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

fn is_safe_pulsar_name(value: &str) -> bool {
    !value.trim().is_empty()
        && !value.contains("..")
        && !value.chars().any(|ch| matches!(ch, '/' | '\\' | '*' | '>'))
        && is_safe_metadata(value)
}

fn is_safe_namespace(value: &str) -> bool {
    let parts: Vec<&str> = value.split('/').collect();
    parts.len() == 2 && parts.iter().all(|part| is_safe_pulsar_name(part))
}

fn is_safe_topic_uri(value: &str) -> bool {
    (value.starts_with("persistent://") || value.starts_with("non-persistent://"))
        && value.split('/').count() >= 5
        && is_safe_metadata(value)
}

fn is_safe_optional_topic_uri(value: Option<&str>) -> bool {
    value.is_none_or(is_safe_topic_uri)
}

fn is_safe_connection_ref(value: &str) -> bool {
    (value.starts_with("pulsar://")
        || value.starts_with("pulsar+ssl://")
        || value.starts_with("service-url:"))
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

fn dead_letter_topic_uri(topic_uri: &str) -> String {
    format!("{topic_uri}-DLQ")
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
            PULSAR_EVENT_BUS_ADAPTER_SURFACE,
            "workflow-engine.event-bus.adapter.pulsar"
        );
        assert_eq!(
            PULSAR_EVENT_BUS_ADAPTER_MODE_REF,
            "workflow-event-bus-adapter-pulsar:plan-only-preview"
        );
        assert_eq!(PULSAR_EVENT_BUS_MAX_PARTITIONS, 4096);
        assert_eq!(PULSAR_EVENT_BUS_DEFAULT_ACK_TIMEOUT_MS, 30_000);
        assert!(
            PULSAR_EVENT_BUS_ADAPTER_NON_CLAIMS
                .contains(&"workflow-event-bus-adapter-pulsar:no-broker-connection")
        );
        assert!(
            PULSAR_EVENT_BUS_ADAPTER_NON_CLAIMS
                .contains(&"workflow-event-bus-adapter-pulsar:no-admin-api-runtime")
        );
        assert!(
            PULSAR_EVENT_BUS_ADAPTER_NON_CLAIMS
                .contains(&"workflow-event-bus-adapter-pulsar:no-ack-runtime")
        );
        assert!(
            PULSAR_EVENT_BUS_ADAPTER_NON_CLAIMS
                .contains(&"workflow-event-bus-adapter-pulsar:no-hyperscaler-claim")
        );
    }

    #[test]
    fn namespace_and_topic_plans_capture_multitenant_partitioned_deduplicated_shape() {
        let namespace =
            PulsarEventBusAdapter::namespace_declaration_plan(&namespace_descriptor()).unwrap();
        assert_eq!(namespace.operation, "NamespaceDeclarationPlan");
        assert_eq!(namespace.tenant_ref.as_deref(), Some("oyatie-workflow"));
        assert_eq!(
            namespace.namespace_ref.as_deref(),
            Some("oyatie-workflow/workflow-engine")
        );
        assert_eq!(
            config_value(&namespace, "deduplication_enabled"),
            Some("true")
        );
        assert_eq!(
            config_value(&namespace, "admin_api_runtime"),
            Some("not-planned")
        );

        let topic = PulsarEventBusAdapter::topic_declaration_plan(&topic_descriptor()).unwrap();
        assert_eq!(topic.operation, "TopicDeclarationPlan");
        assert_eq!(topic.topic_uri.as_deref(), Some(topic_uri()));
        assert_eq!(config_value(&topic, "partitions"), Some("12"));
        assert_eq!(config_value(&topic, "deduplication_enabled"), Some("true"));
        assert_eq!(
            config_value(&topic, "topic_creation_runtime"),
            Some("not-planned")
        );

        let mut invalid = topic_descriptor();
        invalid.partitions = PULSAR_EVENT_BUS_MAX_PARTITIONS + 1;
        assert_eq!(
            PulsarEventBusAdapter::topic_declaration_plan(&invalid),
            Err(PulsarEventBusPlanFailure::InvalidTopicDescriptor)
        );
    }

    #[test]
    fn producer_config_requires_named_dedup_safe_producer_and_no_send_timeout() {
        let producer = producer_descriptor();
        let plan = PulsarEventBusAdapter::producer_config_plan(&producer).unwrap();
        assert_eq!(plan.operation, "ProducerConfigPlan");
        assert_eq!(
            plan.producer_ref.as_deref(),
            Some("workflow-engine-event-bus-producer")
        );
        assert_eq!(config_value(&plan, "send_timeout_ms"), Some("0"));
        assert_eq!(config_value(&plan, "batching_enabled"), Some("true"));
        assert_eq!(config_value(&plan, "key_based_batching"), Some("true"));
        assert_eq!(config_value(&plan, "producer_runtime"), Some("not-planned"));

        let mut invalid = producer;
        invalid.send_timeout_ms = 30_000;
        assert_eq!(
            PulsarEventBusAdapter::producer_config_plan(&invalid),
            Err(PulsarEventBusPlanFailure::InvalidProducerDescriptor)
        );
    }

    #[test]
    fn publish_message_plan_binds_sequence_id_cloudevents_asyncapi_and_payload_ref_only() {
        let envelope = publish_envelope();
        let plan =
            PulsarEventBusAdapter::publish_message_plan(&envelope, &producer_descriptor()).unwrap();
        assert_eq!(plan.operation, "ProducerMessagePlan");
        assert_eq!(plan.topic_uri.as_deref(), Some(topic_uri()));
        assert_eq!(
            plan.producer_ref.as_deref(),
            Some("workflow-engine-event-bus-producer")
        );
        assert_eq!(
            plan.sequence_id_ref.as_deref(),
            Some("idem:event-bus-adapter:publish:1")
        );
        assert_eq!(
            plan.payload_ref.as_deref(),
            Some("body-ref:workflow-run-started")
        );
        assert_eq!(
            property_value(&plan, "ce-specversion"),
            Some(WORKFLOW_EVENT_BUS_CLOUDEVENTS_SPECVERSION)
        );
        assert_eq!(
            property_value(&plan, "ce-type"),
            Some(WorkflowEventBusEventKind::WorkflowRunStarted.event_type())
        );
        assert_eq!(
            property_value(&plan, "Oya-AsyncAPI-Channel-Ref"),
            Some(
                "workflow/workflow-engine/contracts/asyncapi/workflow-events.yaml#/channels/workflow_runs_events_v1"
            )
        );
        assert!(
            plan.properties
                .iter()
                .all(|property| !property.value_ref.contains("payload="))
        );
    }

    #[test]
    fn consumer_receive_and_ack_plans_are_subscription_aware_bounded_and_no_ack_runtime() {
        let consumer = consumer_descriptor();
        let subscription = PulsarEventBusAdapter::consumer_subscription_plan(&consumer).unwrap();
        assert_eq!(subscription.operation, "ConsumerSubscriptionPlan");
        assert_eq!(
            subscription.subscription_ref.as_deref(),
            Some("workflow-engine-event-bus-sub")
        );
        assert_eq!(
            config_value(&subscription, "subscription_type"),
            Some("key_shared")
        );
        assert_eq!(
            config_value(&subscription, "subscription_mode"),
            Some("durable")
        );
        assert_eq!(
            config_value(&subscription, "batch_index_ack_enabled"),
            Some("true")
        );
        assert_eq!(
            config_value(&subscription, "consumer_runtime"),
            Some("not-planned")
        );

        let receive = PulsarEventBusAdapter::receive_batch_plan(&consumer, 100).unwrap();
        assert_eq!(receive.operation, "ReceiveBatchPlan");
        assert_eq!(receive.batch_bound, Some(100));
        assert_eq!(
            config_value(&receive, "receive_runtime"),
            Some("not-planned")
        );
        assert_eq!(
            PulsarEventBusAdapter::receive_batch_plan(
                &consumer,
                PULSAR_EVENT_BUS_MAX_RECEIVE_BATCH_SIZE + 1
            ),
            Err(PulsarEventBusPlanFailure::InvalidConsumerDescriptor)
        );

        let ack =
            PulsarEventBusAdapter::ack_plan(&delivery_envelope("delivered"), &consumer).unwrap();
        assert_eq!(ack.operation, "AckPlan");
        assert!(!ack.ack_runtime_planned);
        assert!(!ack.message_id_commit_planned);
        assert_eq!(config_value(&ack, "acknowledgement"), Some("individual"));
        assert_eq!(config_value(&ack, "ack_runtime"), Some("not-planned"));
    }

    #[test]
    fn offset_observation_and_dead_letter_plans_do_not_commit_message_ids() {
        let offset = PulsarEventBusAdapter::offset_observation_plan(&offset_descriptor()).unwrap();
        assert_eq!(offset.operation, "OffsetObservationPlan");
        assert_eq!(
            offset.message_id_ref.as_deref(),
            Some("message-id:pulsar:ledger-1:entry-42:partition-0")
        );
        assert!(!offset.message_id_commit_planned);
        assert_eq!(
            config_value(&offset, "message_id_commit_planned"),
            Some("false")
        );

        let mut invalid = offset_descriptor();
        invalid.message_id_commit_planned = true;
        assert_eq!(
            PulsarEventBusAdapter::offset_observation_plan(&invalid),
            Err(PulsarEventBusPlanFailure::InvalidReference)
        );

        let dlq = PulsarEventBusAdapter::dead_letter_topic_plan(
            &delivery_envelope("max-deliver"),
            &consumer_descriptor(),
        )
        .unwrap();
        assert_eq!(dlq.operation, "DeadLetterTopicPlan");
        assert!(!dlq.message_id_commit_planned);
        assert_eq!(
            config_value(&dlq, "dead_letter_runtime"),
            Some("not-planned")
        );
        assert_eq!(
            dlq.topic_uri.as_deref(),
            Some("persistent://oyatie-workflow/workflow-engine/workflow-events-DLQ")
        );
    }

    #[test]
    fn unsafe_raw_metadata_is_rejected_before_plan_without_echo() {
        let mut envelope = publish_envelope();
        envelope.payload_ref =
            "raw payload Authorization: Bearer sk-test customer message".to_owned();
        let err = PulsarEventBusAdapter::publish_message_plan(&envelope, &producer_descriptor())
            .unwrap_err();
        assert_eq!(err, PulsarEventBusPlanFailure::InvalidEnvelope);
        let rendered = format!("{err:?}");
        assert!(!rendered.contains("raw payload"));
        assert!(!rendered.contains("Authorization"));
        assert!(!rendered.contains("sk-test"));
    }

    #[test]
    fn api_generic_adapter_and_pulsar_plans_integrate_without_runtime_claims() {
        let mut api = WorkflowEventBusApi::default();
        let publish_success = api
            .publish_event(publish_request("idem:event-bus-pulsar:publish"))
            .unwrap();
        assert_eq!(publish_success.status, WorkflowEventBusApiStatus::Accepted);
        let delivery_success = api
            .evaluate_delivery(delivery_request("idem:event-bus-pulsar:delivery"))
            .unwrap();
        assert_eq!(delivery_success.status, WorkflowEventBusApiStatus::Accepted);

        let mut memory_adapter = WorkflowEventBusMemoryAdapter::default();
        let publish_receipt = memory_adapter
            .record_publish_from_api_success(
                &publish_success,
                publish_envelope_from_api(&publish_success),
            )
            .unwrap();
        let delivery_receipt = memory_adapter
            .record_delivery_from_api_success(
                &delivery_success,
                delivery_envelope_from_api(&delivery_success),
            )
            .unwrap();
        assert_eq!(
            publish_receipt.status,
            WorkflowEventBusAdapterReceiptStatus::Recorded
        );
        assert!(!delivery_receipt.offset_commit_planned);

        let mut adapter = PulsarEventBusAdapter::default();
        adapter.plan_topic_declaration(&topic_descriptor()).unwrap();
        adapter
            .plan_publish_message(&publish_envelope(), &producer_descriptor())
            .unwrap();
        adapter
            .plan_consumer_subscription(&consumer_descriptor())
            .unwrap();
        assert_eq!(adapter.generated_plans().len(), 3);
        assert!(adapter.generated_plans().iter().all(|plan| {
            plan.non_claim_refs
                .contains(&"workflow-event-bus-adapter-pulsar:no-broker-connection".to_string())
                && plan
                    .non_claim_refs
                    .contains(&"workflow-event-bus-adapter-pulsar:no-cloud-runtime".to_string())
                && plan
                    .non_claim_refs
                    .contains(&"workflow-event-bus-adapter-pulsar:no-hyperscaler-claim".to_string())
        }));
    }

    fn config_value<'a>(plan: &'a PulsarEventBusCommandPlan, key: &str) -> Option<&'a str> {
        plan.configs
            .iter()
            .find(|entry| entry.key == key)
            .map(|entry| entry.value.as_str())
    }

    fn property_value<'a>(plan: &'a PulsarEventBusCommandPlan, key: &str) -> Option<&'a str> {
        plan.properties
            .iter()
            .find(|entry| entry.key == key)
            .map(|entry| entry.value_ref.as_str())
    }

    fn topic_uri() -> &'static str {
        "persistent://oyatie-workflow/workflow-engine/workflow-events"
    }

    fn namespace_descriptor() -> PulsarEventBusNamespaceDescriptor {
        PulsarEventBusNamespaceDescriptor {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            pulsar_tenant_ref: "oyatie-workflow".to_owned(),
            namespace_ref: "oyatie-workflow/workflow-engine".to_owned(),
            clusters_ref: "clusters:pulsar:us-east-1".to_owned(),
            retention_time_ms: 604_800_000,
            retention_size_bytes: 1_073_741_824,
            message_ttl_seconds: 86_400,
            deduplication_enabled: true,
            evidence_refs: vec!["evidence:event-bus-pulsar:namespace".to_owned()],
        }
    }

    fn topic_descriptor() -> PulsarEventBusTopicDescriptor {
        PulsarEventBusTopicDescriptor {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            topic_uri: topic_uri().to_owned(),
            partitions: 12,
            deduplication_enabled: true,
            schema_ref: Some("schema:pulsar:workflow-event".to_owned()),
            compaction_enabled: false,
            evidence_refs: vec!["evidence:event-bus-pulsar:topic".to_owned()],
        }
    }

    fn producer_descriptor() -> PulsarEventBusProducerDescriptor {
        PulsarEventBusProducerDescriptor {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            service_url_ref: "pulsar+ssl://broker.pulsar.internal:6651".to_owned(),
            topic_uri: topic_uri().to_owned(),
            producer_name_ref: "workflow-engine-event-bus-producer".to_owned(),
            send_timeout_ms: 0,
            batching_enabled: true,
            key_based_batching: true,
            max_pending_messages: 1000,
            compression_ref: Some("compression:lz4".to_owned()),
            evidence_refs: vec!["evidence:event-bus-pulsar:producer".to_owned()],
        }
    }

    fn consumer_descriptor() -> PulsarEventBusConsumerDescriptor {
        PulsarEventBusConsumerDescriptor {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            topic_uri: topic_uri().to_owned(),
            subscription_name_ref: "workflow-engine-event-bus-sub".to_owned(),
            subscription_type: "key_shared".to_owned(),
            subscription_mode: "durable".to_owned(),
            subscription_initial_position: "earliest".to_owned(),
            consumer_name_ref: "workflow-state-machine-consumer".to_owned(),
            receiver_queue_size: 1000,
            ack_timeout_ms: PULSAR_EVENT_BUS_DEFAULT_ACK_TIMEOUT_MS,
            negative_ack_redelivery_delay_ms:
                PULSAR_EVENT_BUS_DEFAULT_NEGATIVE_ACK_REDELIVERY_DELAY_MS,
            batch_index_ack_enabled: true,
            dead_letter_topic_ref: Some(
                "persistent://oyatie-workflow/workflow-engine/workflow-events-DLQ".to_owned(),
            ),
            evidence_refs: vec!["evidence:event-bus-pulsar:consumer".to_owned()],
        }
    }

    fn offset_descriptor() -> PulsarEventBusOffsetDescriptor {
        PulsarEventBusOffsetDescriptor {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            topic_uri: topic_uri().to_owned(),
            subscription_name_ref: "workflow-engine-event-bus-sub".to_owned(),
            ledger_id_ref: "ledger:pulsar:1".to_owned(),
            entry_id_ref: "entry:pulsar:42".to_owned(),
            partition_index_ref: "partition:pulsar:0".to_owned(),
            message_id_ref: "message-id:pulsar:ledger-1:entry-42:partition-0".to_owned(),
            message_id_commit_planned: false,
            evidence_refs: vec!["evidence:event-bus-pulsar:offset".to_owned()],
        }
    }

    fn publish_envelope() -> WorkflowEventBusAdapterPublishEnvelope {
        WorkflowEventBusAdapterPublishEnvelope {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            cell_id: "cell:us-east-1a".to_owned(),
            channel_address: "workflow.runs.events.v1".to_owned(),
            event_id: "event:workflow-run-started:001".to_owned(),
            event_type: WorkflowEventBusEventKind::WorkflowRunStarted
                .event_type()
                .to_owned(),
            source_ref: "urn:oyatie:workflow-engine:execution".to_owned(),
            subject_ref: Some("subject:workflow-run:001".to_owned()),
            partition_key_ref: "partition:tenant-workflow-run".to_owned(),
            payload_ref: "body-ref:workflow-run-started".to_owned(),
            idempotency_key: "idem:event-bus-adapter:publish:1".to_owned(),
            trace_context_ref: "trace:event-bus-adapter".to_owned(),
            audit_chain_ref: "audit-chain:event-bus-adapter".to_owned(),
            asyncapi_channel_ref: Some(format!(
                "{}#/channels/workflow_runs_events_v1",
                WORKFLOW_EVENT_BUS_ASYNCAPI_CONTRACT_REF
            )),
            cloudevents_specversion: WORKFLOW_EVENT_BUS_CLOUDEVENTS_SPECVERSION.to_owned(),
            evidence_refs: vec!["evidence:event-bus-pulsar:publish".to_owned()],
        }
    }

    fn delivery_envelope(delivery_status: &str) -> WorkflowEventBusAdapterDeliveryEnvelope {
        let event_kind = if delivery_status == "delivery-denied" {
            WorkflowEventBusEventKind::WorkflowRunStarted
        } else {
            WorkflowEventBusEventKind::WorkflowStateTransitioned
        };
        WorkflowEventBusAdapterDeliveryEnvelope {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            cell_id: "cell:us-east-1a".to_owned(),
            channel_address: event_kind.channel().address().to_owned(),
            event_id: "event:workflow-state:001".to_owned(),
            event_type: event_kind.event_type().to_owned(),
            consumer_ref: "consumer:workflow-state-machine".to_owned(),
            offset_ref: "message-id:pulsar:ledger-1:entry-42:partition-0".to_owned(),
            payload_ref: "body-ref:workflow-state-transitioned".to_owned(),
            idempotency_key: "idem:event-bus-adapter:delivery:1".to_owned(),
            replay_cursor_ref: Some("cursor:event-bus-pulsar:state".to_owned()),
            trace_context_ref: "trace:event-bus-adapter".to_owned(),
            audit_chain_ref: "audit-chain:event-bus-adapter".to_owned(),
            evidence_refs: vec!["evidence:event-bus-pulsar:delivery".to_owned()],
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
            request_id: format!("request:event-bus-pulsar:{idempotency_key}"),
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
