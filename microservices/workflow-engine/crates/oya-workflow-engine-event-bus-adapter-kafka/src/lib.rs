//! Workflow-engine event-bus Kafka adapter foundation.
//!
//! This crate provides source-level, plan-only Kafka command semantics for the
//! workflow event-bus adapter seam. It models future topic declarations,
//! producer configuration, producer records, consumer subscriptions, consumer
//! polls, offset observations, and dead-letter records using Kafka-shaped plans.
//! It never opens broker connections, creates topics, sends records, polls
//! consumers, coordinates consumer groups, commits offsets, materializes
//! payloads, signs events, deploys to Kubernetes/cloud, schedules tenant
//! workloads, or claims durable event-bus runtime behavior.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use oya_workflow_engine_event_bus_adapter::{
    WORKFLOW_EVENT_BUS_ADAPTER_SURFACE, WORKFLOW_EVENT_BUS_ASYNCAPI_CONTRACT_REF,
    WORKFLOW_EVENT_BUS_CLOUDEVENTS_SPECVERSION, WorkflowEventBusAdapterDeliveryEnvelope,
    WorkflowEventBusAdapterDeliveryReceipt, WorkflowEventBusAdapterPublishEnvelope,
    WorkflowEventBusAdapterPublishReceipt, WorkflowEventBusEventKind,
};

pub const KAFKA_EVENT_BUS_ADAPTER_SURFACE: &str = "workflow-engine.event-bus.adapter.kafka";
pub const KAFKA_EVENT_BUS_ADAPTER_MODE_REF: &str =
    "workflow-event-bus-adapter-kafka:plan-only-preview";
pub const KAFKA_EVENT_BUS_MAX_PARTITIONS: u32 = 4096;
pub const KAFKA_EVENT_BUS_MAX_REPLICATION_FACTOR: u16 = 9;
pub const KAFKA_EVENT_BUS_MAX_BATCH_SIZE: u32 = 1000;
pub const KAFKA_EVENT_BUS_DEFAULT_DELIVERY_TIMEOUT_MS: u64 = 120_000;
pub const KAFKA_EVENT_BUS_DEFAULT_LINGER_MS: u64 = 5;
pub const KAFKA_EVENT_BUS_MAX_LINGER_MS: u64 = 60_000;

pub const KAFKA_EVENT_BUS_ADAPTER_NON_CLAIMS: [&str; 9] = [
    "workflow-event-bus-adapter-kafka:no-broker-connection",
    "workflow-event-bus-adapter-kafka:no-admin-client-runtime",
    "workflow-event-bus-adapter-kafka:no-topic-creation",
    "workflow-event-bus-adapter-kafka:no-producer-runtime",
    "workflow-event-bus-adapter-kafka:no-consumer-group-runtime",
    "workflow-event-bus-adapter-kafka:no-offset-commit-runtime",
    "workflow-event-bus-adapter-kafka:no-payload-materialization",
    "workflow-event-bus-adapter-kafka:no-cloud-runtime",
    "workflow-event-bus-adapter-kafka:no-hyperscaler-claim",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum KafkaEventBusPlanKind {
    TopicDeclaration,
    ProducerConfig,
    ProducerRecord,
    ConsumerSubscription,
    ConsumerPoll,
    OffsetObservation,
    DeadLetterRecord,
}

impl KafkaEventBusPlanKind {
    pub const fn operation(self) -> &'static str {
        match self {
            Self::TopicDeclaration => "CreateTopicsPlan",
            Self::ProducerConfig => "ProducerConfigPlan",
            Self::ProducerRecord => "ProducerRecordPlan",
            Self::ConsumerSubscription => "ConsumerSubscriptionPlan",
            Self::ConsumerPoll => "ConsumerPollPlan",
            Self::OffsetObservation => "OffsetObservationPlan",
            Self::DeadLetterRecord => "DeadLetterRecordPlan",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KafkaEventBusConfigEntry {
    pub key: String,   // data_class: INTERNAL_ONLY
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KafkaEventBusHeaderPlan {
    pub key: String,       // data_class: PUBLIC
    pub value_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KafkaEventBusCommandPlan {
    pub plan_kind: KafkaEventBusPlanKind, // data_class: INTERNAL_ONLY
    pub operation: String,                // data_class: INTERNAL_ONLY
    pub topic_ref: Option<String>,        // data_class: INTERNAL_ONLY
    pub key_ref: Option<String>,          // data_class: INTERNAL_ONLY
    pub value_ref: Option<String>,        // data_class: INTERNAL_ONLY
    pub group_ref: Option<String>,        // data_class: INTERNAL_ONLY
    pub consumer_ref: Option<String>,     // data_class: INTERNAL_ONLY
    pub partition_ref: Option<String>,    // data_class: INTERNAL_ONLY
    pub offset_ref: Option<String>,       // data_class: INTERNAL_ONLY
    pub headers: Vec<KafkaEventBusHeaderPlan>, // data_class: INTERNAL_ONLY
    pub configs: Vec<KafkaEventBusConfigEntry>, // data_class: INTERNAL_ONLY
    pub batch_bound: Option<u32>,         // data_class: INTERNAL_ONLY
    pub offset_commit_planned: Option<bool>, // data_class: INTERNAL_ONLY
    pub auto_commit_enabled: Option<bool>, // data_class: INTERNAL_ONLY
    pub executes_runtime: bool,           // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,       // data_class: INTERNAL_ONLY
    pub non_claim_refs: Vec<String>,      // data_class: INTERNAL_ONLY
}

impl KafkaEventBusCommandPlan {
    fn new(plan_kind: KafkaEventBusPlanKind, evidence_refs: Vec<String>) -> Self {
        Self {
            plan_kind,
            operation: plan_kind.operation().to_owned(),
            topic_ref: None,
            key_ref: None,
            value_ref: None,
            group_ref: None,
            consumer_ref: None,
            partition_ref: None,
            offset_ref: None,
            headers: Vec::new(),
            configs: Vec::new(),
            batch_bound: None,
            offset_commit_planned: None,
            auto_commit_enabled: None,
            executes_runtime: false,
            evidence_refs: sorted_unique(evidence_refs),
            non_claim_refs: KAFKA_EVENT_BUS_ADAPTER_NON_CLAIMS
                .iter()
                .map(|value| (*value).to_owned())
                .collect(),
        }
    }

    fn with_topic_ref(mut self, topic_ref: String) -> Self {
        self.topic_ref = Some(topic_ref);
        self
    }

    fn with_key_ref(mut self, key_ref: String) -> Self {
        self.key_ref = Some(key_ref);
        self
    }

    fn with_value_ref(mut self, value_ref: String) -> Self {
        self.value_ref = Some(value_ref);
        self
    }

    fn with_group_ref(mut self, group_ref: String) -> Self {
        self.group_ref = Some(group_ref);
        self
    }

    fn with_consumer_ref(mut self, consumer_ref: String) -> Self {
        self.consumer_ref = Some(consumer_ref);
        self
    }

    fn with_partition_ref(mut self, partition_ref: String) -> Self {
        self.partition_ref = Some(partition_ref);
        self
    }

    fn with_offset_ref(mut self, offset_ref: String) -> Self {
        self.offset_ref = Some(offset_ref);
        self
    }

    fn with_headers(mut self, headers: Vec<KafkaEventBusHeaderPlan>) -> Self {
        self.headers = headers;
        self
    }

    fn with_configs(mut self, configs: Vec<KafkaEventBusConfigEntry>) -> Self {
        self.configs = configs;
        self
    }

    fn with_batch_bound(mut self, batch_bound: u32) -> Self {
        self.batch_bound = Some(batch_bound);
        self
    }

    fn with_offset_commit_planned(mut self, offset_commit_planned: bool) -> Self {
        self.offset_commit_planned = Some(offset_commit_planned);
        self
    }

    fn with_auto_commit_enabled(mut self, auto_commit_enabled: bool) -> Self {
        self.auto_commit_enabled = Some(auto_commit_enabled);
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KafkaEventBusPlanFailure {
    InvalidBatchSize,
    InvalidProducerConfig,
    InvalidReplication,
    InvalidTopicConfig,
    PlanOnly { evidence_ref: String },
    UnsafeMetadata,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KafkaEventBusTopicDescriptor {
    pub tenant_id: String,          // data_class: INTERNAL_ONLY
    pub topic_ref: String,          // data_class: INTERNAL_ONLY
    pub partitions: u32,            // data_class: INTERNAL_ONLY
    pub replication_factor: u16,    // data_class: INTERNAL_ONLY
    pub min_in_sync_replicas: u16,  // data_class: INTERNAL_ONLY
    pub cleanup_policy: String,     // data_class: INTERNAL_ONLY
    pub retention_ms: u64,          // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KafkaEventBusProducerDescriptor {
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub bootstrap_cluster_ref: String,        // data_class: INTERNAL_ONLY
    pub client_id_ref: String,                // data_class: INTERNAL_ONLY
    pub transactional_id_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub delivery_timeout_ms: u64,             // data_class: INTERNAL_ONLY
    pub linger_ms: u64,                       // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KafkaEventBusConsumerDescriptor {
    pub tenant_id: String,          // data_class: INTERNAL_ONLY
    pub group_ref: String,          // data_class: INTERNAL_ONLY
    pub consumer_ref: String,       // data_class: INTERNAL_ONLY
    pub topic_refs: Vec<String>,    // data_class: INTERNAL_ONLY
    pub max_batch_size: u32,        // data_class: INTERNAL_ONLY
    pub isolation_level: String,    // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KafkaEventBusOffsetDescriptor {
    pub tenant_id: String,          // data_class: INTERNAL_ONLY
    pub group_ref: String,          // data_class: INTERNAL_ONLY
    pub topic_ref: String,          // data_class: INTERNAL_ONLY
    pub partition_ref: String,      // data_class: INTERNAL_ONLY
    pub offset_ref: String,         // data_class: INTERNAL_ONLY
    pub commit_planned: bool,       // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Default)]
pub struct KafkaEventBusAdapter {
    generated_plans: Vec<KafkaEventBusCommandPlan>,
}

impl KafkaEventBusAdapter {
    pub fn topic_declaration_plan(
        descriptor: &KafkaEventBusTopicDescriptor,
    ) -> Result<KafkaEventBusCommandPlan, KafkaEventBusPlanFailure> {
        validate_topic_descriptor(descriptor)?;
        Ok(KafkaEventBusCommandPlan::new(
            KafkaEventBusPlanKind::TopicDeclaration,
            descriptor.evidence_refs.clone(),
        )
        .with_topic_ref(descriptor.topic_ref.clone())
        .with_configs(vec![
            config("partitions", &descriptor.partitions.to_string()),
            config(
                "replication.factor",
                &descriptor.replication_factor.to_string(),
            ),
            config(
                "min.insync.replicas",
                &descriptor.min_in_sync_replicas.to_string(),
            ),
            config("cleanup.policy", &descriptor.cleanup_policy),
            config("retention.ms", &descriptor.retention_ms.to_string()),
        ]))
    }

    pub fn producer_config_plan(
        descriptor: &KafkaEventBusProducerDescriptor,
    ) -> Result<KafkaEventBusCommandPlan, KafkaEventBusPlanFailure> {
        validate_producer_descriptor(descriptor)?;
        let mut configs = vec![
            config("bootstrap.servers.ref", &descriptor.bootstrap_cluster_ref),
            config("client.id.ref", &descriptor.client_id_ref),
            config("acks", "all"),
            config("enable.idempotence", "true"),
            config("max.in.flight.requests.per.connection", "5"),
            config("retries", "2147483647"),
            config(
                "delivery.timeout.ms",
                &descriptor.delivery_timeout_ms.to_string(),
            ),
            config("linger.ms", &descriptor.linger_ms.to_string()),
        ];
        if let Some(transactional_id_ref) = &descriptor.transactional_id_ref {
            configs.push(config("transactional.id.ref", transactional_id_ref));
        }
        Ok(KafkaEventBusCommandPlan::new(
            KafkaEventBusPlanKind::ProducerConfig,
            descriptor.evidence_refs.clone(),
        )
        .with_configs(configs))
    }

    pub fn publish_record_plan(
        envelope: &WorkflowEventBusAdapterPublishEnvelope,
        producer: &KafkaEventBusProducerDescriptor,
    ) -> Result<KafkaEventBusCommandPlan, KafkaEventBusPlanFailure> {
        validate_publish_envelope(envelope)?;
        validate_producer_descriptor(producer)?;
        if envelope.tenant_id != producer.tenant_id {
            return Err(KafkaEventBusPlanFailure::UnsafeMetadata);
        }
        let topic_ref = topic_ref_from_channel(&envelope.tenant_id, &envelope.channel_address);
        Ok(KafkaEventBusCommandPlan::new(
            KafkaEventBusPlanKind::ProducerRecord,
            sorted_unique_join(vec![
                envelope.evidence_refs.clone(),
                producer.evidence_refs.clone(),
            ]),
        )
        .with_topic_ref(topic_ref)
        .with_key_ref(envelope.partition_key_ref.clone())
        .with_value_ref(envelope.payload_ref.clone())
        .with_headers(publish_headers(envelope)))
    }

    pub fn consumer_subscription_plan(
        descriptor: &KafkaEventBusConsumerDescriptor,
    ) -> Result<KafkaEventBusCommandPlan, KafkaEventBusPlanFailure> {
        validate_consumer_descriptor(descriptor)?;
        Ok(KafkaEventBusCommandPlan::new(
            KafkaEventBusPlanKind::ConsumerSubscription,
            descriptor.evidence_refs.clone(),
        )
        .with_group_ref(descriptor.group_ref.clone())
        .with_consumer_ref(descriptor.consumer_ref.clone())
        .with_batch_bound(descriptor.max_batch_size)
        .with_offset_commit_planned(false)
        .with_auto_commit_enabled(false)
        .with_configs(vec![
            config("group.id.ref", &descriptor.group_ref),
            config("client.id.ref", &descriptor.consumer_ref),
            config("topic.refs", &descriptor.topic_refs.join("|")),
            config("enable.auto.commit", "false"),
            config("isolation.level", &descriptor.isolation_level),
            config("max.poll.records", &descriptor.max_batch_size.to_string()),
        ]))
    }

    pub fn consumer_poll_plan(
        descriptor: &KafkaEventBusConsumerDescriptor,
    ) -> Result<KafkaEventBusCommandPlan, KafkaEventBusPlanFailure> {
        validate_consumer_descriptor(descriptor)?;
        Ok(KafkaEventBusCommandPlan::new(
            KafkaEventBusPlanKind::ConsumerPoll,
            descriptor.evidence_refs.clone(),
        )
        .with_group_ref(descriptor.group_ref.clone())
        .with_consumer_ref(descriptor.consumer_ref.clone())
        .with_batch_bound(descriptor.max_batch_size)
        .with_offset_commit_planned(false)
        .with_auto_commit_enabled(false))
    }

    pub fn offset_observation_plan(
        descriptor: &KafkaEventBusOffsetDescriptor,
    ) -> Result<KafkaEventBusCommandPlan, KafkaEventBusPlanFailure> {
        validate_offset_descriptor(descriptor)?;
        Ok(KafkaEventBusCommandPlan::new(
            KafkaEventBusPlanKind::OffsetObservation,
            descriptor.evidence_refs.clone(),
        )
        .with_topic_ref(descriptor.topic_ref.clone())
        .with_group_ref(descriptor.group_ref.clone())
        .with_partition_ref(descriptor.partition_ref.clone())
        .with_offset_ref(descriptor.offset_ref.clone())
        .with_offset_commit_planned(false)
        .with_auto_commit_enabled(false))
    }

    pub fn dead_letter_record_plan(
        envelope: &WorkflowEventBusAdapterDeliveryEnvelope,
        producer: &KafkaEventBusProducerDescriptor,
        reason_ref: &str,
    ) -> Result<KafkaEventBusCommandPlan, KafkaEventBusPlanFailure> {
        validate_delivery_envelope(envelope)?;
        validate_producer_descriptor(producer)?;
        if envelope.tenant_id != producer.tenant_id || !is_safe_ref(reason_ref) {
            return Err(KafkaEventBusPlanFailure::UnsafeMetadata);
        }
        let topic_ref = dead_letter_topic_ref(&envelope.tenant_id, &envelope.channel_address);
        Ok(KafkaEventBusCommandPlan::new(
            KafkaEventBusPlanKind::DeadLetterRecord,
            sorted_unique_join(vec![
                envelope.evidence_refs.clone(),
                producer.evidence_refs.clone(),
            ]),
        )
        .with_topic_ref(topic_ref)
        .with_key_ref(envelope.idempotency_key.clone())
        .with_value_ref(envelope.payload_ref.clone())
        .with_headers(vec![
            header("event_id_ref", &envelope.event_id),
            header("event_type", &envelope.event_type),
            header("consumer_ref", &envelope.consumer_ref),
            header("offset_ref", &envelope.offset_ref),
            header("trace_context_ref", &envelope.trace_context_ref),
            header("audit_chain_ref", &envelope.audit_chain_ref),
            header("dead_letter_reason_ref", reason_ref),
        ]))
    }

    pub fn plan_topic_declaration(
        &mut self,
        descriptor: &KafkaEventBusTopicDescriptor,
    ) -> Result<(), KafkaEventBusPlanFailure> {
        let plan = Self::topic_declaration_plan(descriptor)?;
        self.generated_plans.push(plan);
        Err(KafkaEventBusPlanFailure::PlanOnly {
            evidence_ref: "workflow-event-bus-kafka-adapter:plan-only-topic-declaration".to_owned(),
        })
    }

    pub fn plan_publish_record(
        &mut self,
        envelope: &WorkflowEventBusAdapterPublishEnvelope,
        producer: &KafkaEventBusProducerDescriptor,
    ) -> Result<(), KafkaEventBusPlanFailure> {
        let config_plan = Self::producer_config_plan(producer)?;
        let record_plan = Self::publish_record_plan(envelope, producer)?;
        self.generated_plans.push(config_plan);
        self.generated_plans.push(record_plan);
        Err(KafkaEventBusPlanFailure::PlanOnly {
            evidence_ref: "workflow-event-bus-kafka-adapter:plan-only-publish-record".to_owned(),
        })
    }

    pub fn plan_consumer_subscription(
        &mut self,
        descriptor: &KafkaEventBusConsumerDescriptor,
    ) -> Result<(), KafkaEventBusPlanFailure> {
        let subscription = Self::consumer_subscription_plan(descriptor)?;
        let poll = Self::consumer_poll_plan(descriptor)?;
        self.generated_plans.push(subscription);
        self.generated_plans.push(poll);
        Err(KafkaEventBusPlanFailure::PlanOnly {
            evidence_ref: "workflow-event-bus-kafka-adapter:plan-only-consumer-subscription"
                .to_owned(),
        })
    }

    pub fn generated_plans(&self) -> &[KafkaEventBusCommandPlan] {
        &self.generated_plans
    }
}

fn config(key: &str, value: &str) -> KafkaEventBusConfigEntry {
    KafkaEventBusConfigEntry {
        key: key.to_owned(),
        value: value.to_owned(),
    }
}

fn header(key: &str, value_ref: &str) -> KafkaEventBusHeaderPlan {
    KafkaEventBusHeaderPlan {
        key: key.to_owned(),
        value_ref: value_ref.to_owned(),
    }
}

fn publish_headers(
    envelope: &WorkflowEventBusAdapterPublishEnvelope,
) -> Vec<KafkaEventBusHeaderPlan> {
    let asyncapi_ref = envelope.asyncapi_channel_ref.clone().unwrap_or_default();
    vec![
        header("ce_specversion", &envelope.cloudevents_specversion),
        header("ce_id_ref", &envelope.event_id),
        header("ce_type", &envelope.event_type),
        header("ce_source_ref", &envelope.source_ref),
        header("oyatie_asyncapi_channel_ref", &asyncapi_ref),
        header("tenant_id", &envelope.tenant_id),
        header("cell_id", &envelope.cell_id),
        header("trace_context_ref", &envelope.trace_context_ref),
        header("audit_chain_ref", &envelope.audit_chain_ref),
        header("idempotency_key", &envelope.idempotency_key),
    ]
}

fn validate_topic_descriptor(
    descriptor: &KafkaEventBusTopicDescriptor,
) -> Result<(), KafkaEventBusPlanFailure> {
    if !is_safe_tenant(&descriptor.tenant_id)
        || !is_safe_ref(&descriptor.topic_ref)
        || !descriptor
            .evidence_refs
            .iter()
            .all(|value| is_safe_ref(value))
    {
        return Err(KafkaEventBusPlanFailure::UnsafeMetadata);
    }
    if descriptor.partitions == 0 || descriptor.partitions > KAFKA_EVENT_BUS_MAX_PARTITIONS {
        return Err(KafkaEventBusPlanFailure::InvalidTopicConfig);
    }
    if descriptor.replication_factor == 0
        || descriptor.replication_factor > KAFKA_EVENT_BUS_MAX_REPLICATION_FACTOR
        || descriptor.min_in_sync_replicas == 0
        || descriptor.min_in_sync_replicas > descriptor.replication_factor
    {
        return Err(KafkaEventBusPlanFailure::InvalidReplication);
    }
    if descriptor.retention_ms == 0 || !is_valid_cleanup_policy(&descriptor.cleanup_policy) {
        return Err(KafkaEventBusPlanFailure::InvalidTopicConfig);
    }
    Ok(())
}

fn validate_producer_descriptor(
    descriptor: &KafkaEventBusProducerDescriptor,
) -> Result<(), KafkaEventBusPlanFailure> {
    if !is_safe_tenant(&descriptor.tenant_id)
        || !is_safe_ref(&descriptor.bootstrap_cluster_ref)
        || !is_safe_ref(&descriptor.client_id_ref)
        || !is_safe_optional_ref(descriptor.transactional_id_ref.as_deref())
        || !descriptor
            .evidence_refs
            .iter()
            .all(|value| is_safe_ref(value))
    {
        return Err(KafkaEventBusPlanFailure::UnsafeMetadata);
    }
    if descriptor.delivery_timeout_ms == 0 || descriptor.linger_ms > KAFKA_EVENT_BUS_MAX_LINGER_MS {
        return Err(KafkaEventBusPlanFailure::InvalidProducerConfig);
    }
    Ok(())
}

fn validate_consumer_descriptor(
    descriptor: &KafkaEventBusConsumerDescriptor,
) -> Result<(), KafkaEventBusPlanFailure> {
    if descriptor.max_batch_size == 0 || descriptor.max_batch_size > KAFKA_EVENT_BUS_MAX_BATCH_SIZE
    {
        return Err(KafkaEventBusPlanFailure::InvalidBatchSize);
    }
    if !is_valid_isolation_level(&descriptor.isolation_level) {
        return Err(KafkaEventBusPlanFailure::InvalidTopicConfig);
    }
    if is_safe_tenant(&descriptor.tenant_id)
        && is_safe_ref(&descriptor.group_ref)
        && is_safe_ref(&descriptor.consumer_ref)
        && !descriptor.topic_refs.is_empty()
        && descriptor.topic_refs.iter().all(|value| is_safe_ref(value))
        && descriptor
            .evidence_refs
            .iter()
            .all(|value| is_safe_ref(value))
    {
        Ok(())
    } else {
        Err(KafkaEventBusPlanFailure::UnsafeMetadata)
    }
}

fn validate_offset_descriptor(
    descriptor: &KafkaEventBusOffsetDescriptor,
) -> Result<(), KafkaEventBusPlanFailure> {
    if is_safe_tenant(&descriptor.tenant_id)
        && is_safe_ref(&descriptor.group_ref)
        && is_safe_ref(&descriptor.topic_ref)
        && is_safe_ref(&descriptor.partition_ref)
        && is_safe_ref(&descriptor.offset_ref)
        && !descriptor.commit_planned
        && descriptor
            .evidence_refs
            .iter()
            .all(|value| is_safe_ref(value))
    {
        Ok(())
    } else {
        Err(KafkaEventBusPlanFailure::UnsafeMetadata)
    }
}

fn validate_publish_envelope(
    envelope: &WorkflowEventBusAdapterPublishEnvelope,
) -> Result<(), KafkaEventBusPlanFailure> {
    let asyncapi_safe = envelope
        .asyncapi_channel_ref
        .as_deref()
        .is_some_and(|value| {
            is_safe_metadata(value)
                && value.starts_with(WORKFLOW_EVENT_BUS_ASYNCAPI_CONTRACT_REF)
                && value.contains("#/channels/")
        });
    let valid = is_safe_tenant(&envelope.tenant_id)
        && is_safe_ref(&envelope.cell_id)
        && is_safe_metadata(&envelope.channel_address)
        && is_safe_ref(&envelope.event_id)
        && is_safe_metadata(&envelope.event_type)
        && is_safe_ref(&envelope.source_ref)
        && is_safe_optional_ref(envelope.subject_ref.as_deref())
        && is_safe_ref(&envelope.partition_key_ref)
        && is_safe_ref(&envelope.payload_ref)
        && is_safe_ref(&envelope.idempotency_key)
        && is_safe_ref(&envelope.trace_context_ref)
        && is_safe_ref(&envelope.audit_chain_ref)
        && asyncapi_safe
        && envelope.cloudevents_specversion == WORKFLOW_EVENT_BUS_CLOUDEVENTS_SPECVERSION
        && envelope
            .evidence_refs
            .iter()
            .all(|value| is_safe_ref(value));
    if valid {
        Ok(())
    } else {
        Err(KafkaEventBusPlanFailure::UnsafeMetadata)
    }
}

fn validate_delivery_envelope(
    envelope: &WorkflowEventBusAdapterDeliveryEnvelope,
) -> Result<(), KafkaEventBusPlanFailure> {
    let valid = is_safe_tenant(&envelope.tenant_id)
        && is_safe_ref(&envelope.cell_id)
        && is_safe_metadata(&envelope.channel_address)
        && is_safe_ref(&envelope.event_id)
        && is_safe_metadata(&envelope.event_type)
        && is_safe_ref(&envelope.consumer_ref)
        && is_safe_ref(&envelope.offset_ref)
        && is_safe_ref(&envelope.payload_ref)
        && is_safe_ref(&envelope.idempotency_key)
        && is_safe_optional_ref(envelope.replay_cursor_ref.as_deref())
        && is_safe_ref(&envelope.trace_context_ref)
        && is_safe_ref(&envelope.audit_chain_ref)
        && envelope
            .evidence_refs
            .iter()
            .all(|value| is_safe_ref(value));
    if valid {
        Ok(())
    } else {
        Err(KafkaEventBusPlanFailure::UnsafeMetadata)
    }
}

fn is_valid_cleanup_policy(value: &str) -> bool {
    matches!(value, "delete" | "compact" | "delete,compact")
}

fn is_valid_isolation_level(value: &str) -> bool {
    matches!(value, "read_committed" | "read_uncommitted")
}

fn topic_ref_from_channel(tenant_id: &str, channel_address: &str) -> String {
    format!("topic:kafka:{tenant_id}:{channel_address}")
}

fn dead_letter_topic_ref(tenant_id: &str, channel_address: &str) -> String {
    format!("topic:kafka:{tenant_id}:dead-letter:{channel_address}")
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

fn sorted_unique_join(value_sets: Vec<Vec<String>>) -> Vec<String> {
    let mut values = Vec::new();
    for set in value_sets {
        values.extend(set);
    }
    sorted_unique(values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_workflow_engine_event_bus_adapter::{
        WORKFLOW_EVENT_BUS_API_DECLARED_VERSION, WORKFLOW_EVENT_BUS_API_DELIVERY_ROUTE,
        WORKFLOW_EVENT_BUS_API_METHOD, WORKFLOW_EVENT_BUS_API_PUBLISH_ROUTE,
        WORKFLOW_EVENT_BUS_API_SURFACE, WorkflowEventBusApi, WorkflowEventBusApiAuthorization,
        WorkflowEventBusApiBoundaryContext, WorkflowEventBusApiDeliveryBody,
        WorkflowEventBusApiDeliveryRequest, WorkflowEventBusApiPrincipal,
        WorkflowEventBusApiPublishBody, WorkflowEventBusApiPublishRequest,
        WorkflowEventBusApiSuccessResponse, WorkflowEventBusMemoryAdapter,
    };

    #[test]
    fn constants_configs_and_non_claims_are_plan_only() {
        assert_eq!(
            KAFKA_EVENT_BUS_ADAPTER_SURFACE,
            "workflow-engine.event-bus.adapter.kafka"
        );
        assert_eq!(
            KafkaEventBusPlanKind::TopicDeclaration.operation(),
            "CreateTopicsPlan"
        );
        assert_eq!(
            KafkaEventBusPlanKind::ProducerRecord.operation(),
            "ProducerRecordPlan"
        );
        assert_eq!(
            KafkaEventBusPlanKind::ConsumerSubscription.operation(),
            "ConsumerSubscriptionPlan"
        );
        assert!(
            KAFKA_EVENT_BUS_ADAPTER_NON_CLAIMS
                .contains(&"workflow-event-bus-adapter-kafka:no-broker-connection")
        );
        assert!(
            KAFKA_EVENT_BUS_ADAPTER_NON_CLAIMS
                .contains(&"workflow-event-bus-adapter-kafka:no-offset-commit-runtime")
        );
        assert_eq!(KAFKA_EVENT_BUS_DEFAULT_DELIVERY_TIMEOUT_MS, 120_000);
        assert_eq!(KAFKA_EVENT_BUS_DEFAULT_LINGER_MS, 5);
    }

    #[test]
    fn topic_declaration_plan_uses_min_isr_retention_and_cleanup_guards() {
        let topic = topic_descriptor();
        let plan = KafkaEventBusAdapter::topic_declaration_plan(&topic).unwrap();

        assert_eq!(plan.operation, "CreateTopicsPlan");
        assert_eq!(
            plan.topic_ref.as_deref(),
            Some("topic:kafka:ten_workflow_event_bus:workflow.runs.events.v1")
        );
        assert!(config_value(&plan, "partitions").is_some_and(|value| value == "12"));
        assert!(config_value(&plan, "replication.factor").is_some_and(|value| value == "3"));
        assert!(config_value(&plan, "min.insync.replicas").is_some_and(|value| value == "2"));
        assert!(config_value(&plan, "cleanup.policy").is_some_and(|value| value == "delete"));
        assert!(!plan.executes_runtime);

        let mut invalid = topic_descriptor();
        invalid.min_in_sync_replicas = 4;
        assert_eq!(
            KafkaEventBusAdapter::topic_declaration_plan(&invalid).unwrap_err(),
            KafkaEventBusPlanFailure::InvalidReplication
        );

        let mut adapter = KafkaEventBusAdapter::default();
        assert!(matches!(
            adapter.plan_topic_declaration(&topic_descriptor()),
            Err(KafkaEventBusPlanFailure::PlanOnly { .. })
        ));
        assert_eq!(adapter.generated_plans().len(), 1);
    }

    #[test]
    fn producer_config_plan_sets_idempotent_durable_defaults() {
        let producer = producer_descriptor();
        let plan = KafkaEventBusAdapter::producer_config_plan(&producer).unwrap();

        assert_eq!(plan.operation, "ProducerConfigPlan");
        assert!(config_value(&plan, "acks").is_some_and(|value| value == "all"));
        assert!(config_value(&plan, "enable.idempotence").is_some_and(|value| value == "true"));
        assert!(
            config_value(&plan, "max.in.flight.requests.per.connection")
                .is_some_and(|value| value == "5")
        );
        assert!(config_value(&plan, "retries").is_some_and(|value| value == "2147483647"));
        assert!(config_value(&plan, "delivery.timeout.ms").is_some_and(|value| value == "120000"));
        assert!(!plan.executes_runtime);

        let mut invalid = producer_descriptor();
        invalid.linger_ms = KAFKA_EVENT_BUS_MAX_LINGER_MS + 1;
        assert_eq!(
            KafkaEventBusAdapter::producer_config_plan(&invalid).unwrap_err(),
            KafkaEventBusPlanFailure::InvalidProducerConfig
        );
    }

    #[test]
    fn publish_record_plan_binds_cloudevents_asyncapi_headers_and_payload_ref_only() {
        let envelope = publish_envelope();
        let plan =
            KafkaEventBusAdapter::publish_record_plan(&envelope, &producer_descriptor()).unwrap();

        assert_eq!(plan.operation, "ProducerRecordPlan");
        assert_eq!(
            plan.topic_ref.as_deref(),
            Some("topic:kafka:ten_workflow_event_bus:workflow.runs.events.v1")
        );
        assert_eq!(
            plan.key_ref.as_deref(),
            Some("partition:tenant-workflow-run")
        );
        assert_eq!(
            plan.value_ref.as_deref(),
            Some("body-ref:workflow-run-started")
        );
        assert!(
            header_value(&plan, "ce_specversion")
                .is_some_and(|value| value == WORKFLOW_EVENT_BUS_CLOUDEVENTS_SPECVERSION)
        );
        assert!(
            header_value(&plan, "oyatie_asyncapi_channel_ref")
                .is_some_and(|value| value.contains("#/channels/workflow_runs_events_v1"))
        );
        assert!(
            header_value(&plan, "idempotency_key")
                .is_some_and(|value| value == "idem:event-bus-adapter:publish:1")
        );
        assert!(!plan.executes_runtime);

        let mut adapter = KafkaEventBusAdapter::default();
        assert!(matches!(
            adapter.plan_publish_record(&envelope, &producer_descriptor()),
            Err(KafkaEventBusPlanFailure::PlanOnly { .. })
        ));
        assert_eq!(adapter.generated_plans().len(), 2);
    }

    #[test]
    fn consumer_subscription_and_poll_disable_auto_commit_and_bound_batches() {
        let consumer = consumer_descriptor();
        let subscription = KafkaEventBusAdapter::consumer_subscription_plan(&consumer).unwrap();
        let poll = KafkaEventBusAdapter::consumer_poll_plan(&consumer).unwrap();

        assert_eq!(subscription.operation, "ConsumerSubscriptionPlan");
        assert_eq!(
            subscription.group_ref.as_deref(),
            Some("group:workflow-state-machine")
        );
        assert_eq!(subscription.auto_commit_enabled, Some(false));
        assert_eq!(subscription.offset_commit_planned, Some(false));
        assert_eq!(subscription.batch_bound, Some(100));
        assert!(
            config_value(&subscription, "enable.auto.commit").is_some_and(|value| value == "false")
        );
        assert!(
            config_value(&subscription, "isolation.level")
                .is_some_and(|value| value == "read_committed")
        );
        assert_eq!(poll.offset_commit_planned, Some(false));
        assert_eq!(poll.auto_commit_enabled, Some(false));

        let mut invalid = consumer_descriptor();
        invalid.max_batch_size = 0;
        assert_eq!(
            KafkaEventBusAdapter::consumer_subscription_plan(&invalid).unwrap_err(),
            KafkaEventBusPlanFailure::InvalidBatchSize
        );

        let mut adapter = KafkaEventBusAdapter::default();
        assert!(matches!(
            adapter.plan_consumer_subscription(&consumer_descriptor()),
            Err(KafkaEventBusPlanFailure::PlanOnly { .. })
        ));
        assert_eq!(adapter.generated_plans().len(), 2);
    }

    #[test]
    fn offset_observation_and_dead_letter_plans_do_not_commit_offsets() {
        let offset = KafkaEventBusAdapter::offset_observation_plan(&offset_descriptor()).unwrap();
        assert_eq!(offset.operation, "OffsetObservationPlan");
        assert_eq!(offset.offset_commit_planned, Some(false));
        assert_eq!(offset.auto_commit_enabled, Some(false));
        assert_eq!(
            offset.topic_ref.as_deref(),
            Some("topic:kafka:ten_workflow_event_bus:workflow.state.events.v1")
        );
        assert!(!offset.executes_runtime);

        let dead_letter = KafkaEventBusAdapter::dead_letter_record_plan(
            &delivery_envelope("delivery-denied"),
            &producer_descriptor(),
            "reason:event-bus:kafka:delivery-denied",
        )
        .unwrap();
        assert_eq!(dead_letter.operation, "DeadLetterRecordPlan");
        assert!(
            dead_letter
                .topic_ref
                .as_deref()
                .is_some_and(|value| value.contains("dead-letter"))
        );
        assert_eq!(
            dead_letter.value_ref.as_deref(),
            Some("body-ref:workflow-state-transitioned")
        );
        assert!(!dead_letter.executes_runtime);

        let mut invalid = offset_descriptor();
        invalid.commit_planned = true;
        assert_eq!(
            KafkaEventBusAdapter::offset_observation_plan(&invalid).unwrap_err(),
            KafkaEventBusPlanFailure::UnsafeMetadata
        );
    }

    #[test]
    fn unsafe_raw_metadata_is_rejected_before_plan_without_echo() {
        let mut envelope = publish_envelope();
        envelope.payload_ref =
            "raw payload Authorization: Bearer sk-test customer message".to_owned();
        let err = KafkaEventBusAdapter::publish_record_plan(&envelope, &producer_descriptor())
            .unwrap_err();

        assert_eq!(err, KafkaEventBusPlanFailure::UnsafeMetadata);
        let rendered = format!("{err:?}");
        assert!(!rendered.contains("raw payload"));
        assert!(!rendered.contains("Authorization"));
        assert!(!rendered.contains("sk-test"));
    }

    #[test]
    fn api_generic_adapter_and_kafka_plans_integrate_without_runtime_claims() {
        let mut api = WorkflowEventBusApi::default();
        let publish_success = api
            .publish_event(publish_request("idem:event-bus-kafka:publish"))
            .unwrap();
        let delivery_success = api
            .evaluate_delivery(delivery_request("idem:event-bus-kafka:delivery"))
            .unwrap();
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

        let record = KafkaEventBusAdapter::publish_record_plan(
            &publish_envelope_from_api(&publish_success),
            &producer_descriptor(),
        )
        .unwrap();
        let subscription =
            KafkaEventBusAdapter::consumer_subscription_plan(&consumer_descriptor()).unwrap();
        let offset = KafkaEventBusAdapter::offset_observation_plan(&offset_descriptor()).unwrap();

        assert!(
            publish_receipt
                .non_claim_refs
                .iter()
                .any(|value| value.contains("no-broker"))
        );
        assert!(
            delivery_receipt
                .non_claim_refs
                .iter()
                .any(|value| value.contains("no-offset-commit"))
        );
        assert_eq!(record.operation, "ProducerRecordPlan");
        assert_eq!(subscription.auto_commit_enabled, Some(false));
        assert_eq!(offset.offset_commit_planned, Some(false));
        assert!(
            record
                .non_claim_refs
                .iter()
                .any(|value| value.contains("no-producer-runtime"))
        );
    }

    fn config_value<'a>(plan: &'a KafkaEventBusCommandPlan, key: &str) -> Option<&'a str> {
        plan.configs
            .iter()
            .find(|entry| entry.key == key)
            .map(|entry| entry.value.as_str())
    }

    fn header_value<'a>(plan: &'a KafkaEventBusCommandPlan, key: &str) -> Option<&'a str> {
        plan.headers
            .iter()
            .find(|entry| entry.key == key)
            .map(|entry| entry.value_ref.as_str())
    }

    fn topic_descriptor() -> KafkaEventBusTopicDescriptor {
        KafkaEventBusTopicDescriptor {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            topic_ref: "topic:kafka:ten_workflow_event_bus:workflow.runs.events.v1".to_owned(),
            partitions: 12,
            replication_factor: 3,
            min_in_sync_replicas: 2,
            cleanup_policy: "delete".to_owned(),
            retention_ms: 604_800_000,
            evidence_refs: vec!["evidence:event-bus-kafka:topic".to_owned()],
        }
    }

    fn producer_descriptor() -> KafkaEventBusProducerDescriptor {
        KafkaEventBusProducerDescriptor {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            bootstrap_cluster_ref: "broker:kafka:event-bus:cluster:preview".to_owned(),
            client_id_ref: "producer:event-bus:workflow-engine".to_owned(),
            transactional_id_ref: Some("transactional:event-bus:workflow-engine".to_owned()),
            delivery_timeout_ms: KAFKA_EVENT_BUS_DEFAULT_DELIVERY_TIMEOUT_MS,
            linger_ms: KAFKA_EVENT_BUS_DEFAULT_LINGER_MS,
            evidence_refs: vec!["evidence:event-bus-kafka:producer".to_owned()],
        }
    }

    fn consumer_descriptor() -> KafkaEventBusConsumerDescriptor {
        KafkaEventBusConsumerDescriptor {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            group_ref: "group:workflow-state-machine".to_owned(),
            consumer_ref: "consumer:workflow-state-machine:worker-1".to_owned(),
            topic_refs: vec![
                "topic:kafka:ten_workflow_event_bus:workflow.state.events.v1".to_owned(),
            ],
            max_batch_size: 100,
            isolation_level: "read_committed".to_owned(),
            evidence_refs: vec!["evidence:event-bus-kafka:consumer".to_owned()],
        }
    }

    fn offset_descriptor() -> KafkaEventBusOffsetDescriptor {
        KafkaEventBusOffsetDescriptor {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            group_ref: "group:workflow-state-machine".to_owned(),
            topic_ref: "topic:kafka:ten_workflow_event_bus:workflow.state.events.v1".to_owned(),
            partition_ref: "partition:kafka:workflow-state:0".to_owned(),
            offset_ref: "offset:kafka:workflow-state:42".to_owned(),
            commit_planned: false,
            evidence_refs: vec!["evidence:event-bus-kafka:offset".to_owned()],
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
            evidence_refs: vec!["evidence:event-bus-kafka:publish".to_owned()],
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
            offset_ref: "offset:kafka:workflow-state:42".to_owned(),
            payload_ref: "body-ref:workflow-state-transitioned".to_owned(),
            idempotency_key: "idem:event-bus-adapter:delivery:1".to_owned(),
            replay_cursor_ref: Some("cursor:event-bus-kafka:state".to_owned()),
            trace_context_ref: "trace:event-bus-adapter".to_owned(),
            audit_chain_ref: "audit-chain:event-bus-adapter".to_owned(),
            evidence_refs: vec!["evidence:event-bus-kafka:delivery".to_owned()],
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
            request_id: format!("request:event-bus-kafka:{idempotency_key}"),
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
