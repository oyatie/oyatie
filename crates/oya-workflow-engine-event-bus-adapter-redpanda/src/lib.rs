//! Workflow-engine event-bus Redpanda adapter preview foundation.
//!
//! This crate is a source-level, plan-only adapter around the generic event-bus
//! adapter seam. It models future Redpanda Kafka-API topic, producer, consumer,
//! offset, Schema Registry, and dead-letter metadata without opening Redpanda
//! broker connections, invoking Admin/API/Schema Registry calls, producing,
//! consuming, committing offsets, deploying to cloud, or claiming runtime
//! maturity.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;

pub use oya_workflow_engine_event_bus_adapter::{
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

pub const REDPANDA_EVENT_BUS_ADAPTER_SURFACE: &str = "workflow-engine.event-bus.adapter.redpanda";
pub const REDPANDA_EVENT_BUS_ADAPTER_MODE_REF: &str =
    "workflow-event-bus-adapter-redpanda:plan-only-preview";
pub const REDPANDA_EVENT_BUS_MAX_PARTITIONS: u32 = 4096;
pub const REDPANDA_EVENT_BUS_MAX_REPLICATION_FACTOR: u16 = 9;
pub const REDPANDA_EVENT_BUS_MAX_BATCH_SIZE: u32 = 1000;
pub const REDPANDA_EVENT_BUS_DEFAULT_DELIVERY_TIMEOUT_MS: u64 = 120_000;
pub const REDPANDA_EVENT_BUS_DEFAULT_LINGER_MS: u64 = 5;
pub const REDPANDA_EVENT_BUS_MAX_LINGER_MS: u64 = 60_000;
pub const REDPANDA_EVENT_BUS_MAX_RETENTION_MS: u64 = 31_536_000_000;
pub const REDPANDA_EVENT_BUS_MAX_RETENTION_BYTES: i64 = 1_099_511_627_776;
pub const REDPANDA_EVENT_BUS_MAX_MESSAGE_BYTES: u32 = 104_857_600;

pub const REDPANDA_EVENT_BUS_ADAPTER_NON_CLAIMS: [&str; 11] = [
    "workflow-event-bus-adapter-redpanda:no-broker-connection",
    "workflow-event-bus-adapter-redpanda:no-admin-api-runtime",
    "workflow-event-bus-adapter-redpanda:no-topic-creation",
    "workflow-event-bus-adapter-redpanda:no-schema-registry-runtime",
    "workflow-event-bus-adapter-redpanda:no-producer-runtime",
    "workflow-event-bus-adapter-redpanda:no-consumer-runtime",
    "workflow-event-bus-adapter-redpanda:no-offset-commit-runtime",
    "workflow-event-bus-adapter-redpanda:no-payload-materialization",
    "workflow-event-bus-adapter-redpanda:no-cloud-runtime",
    "workflow-event-bus-adapter-redpanda:no-tenant-scheduler-runtime",
    "workflow-event-bus-adapter-redpanda:no-hyperscaler-claim",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RedpandaEventBusPlanKind {
    ClusterCompatibility,
    TopicDeclaration,
    ProducerConfig,
    ProducerRecord,
    ConsumerAssignment,
    ConsumerPoll,
    OffsetObservation,
    SchemaRegistrySubject,
    DeadLetterRecord,
}

impl RedpandaEventBusPlanKind {
    pub const fn operation(self) -> &'static str {
        match self {
            Self::ClusterCompatibility => "ClusterCompatibilityPlan",
            Self::TopicDeclaration => "RedpandaTopicDeclarationPlan",
            Self::ProducerConfig => "RedpandaProducerConfigPlan",
            Self::ProducerRecord => "RedpandaProducerRecordPlan",
            Self::ConsumerAssignment => "RedpandaConsumerAssignmentPlan",
            Self::ConsumerPoll => "RedpandaConsumerPollPlan",
            Self::OffsetObservation => "RedpandaOffsetObservationPlan",
            Self::SchemaRegistrySubject => "RedpandaSchemaRegistrySubjectPlan",
            Self::DeadLetterRecord => "RedpandaDeadLetterRecordPlan",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RedpandaEventBusConfigEntry {
    pub key: String,   // data_class: PUBLIC
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RedpandaEventBusHeaderPlan {
    pub key: String,       // data_class: PUBLIC
    pub value_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RedpandaEventBusCommandPlan {
    pub plan_kind: RedpandaEventBusPlanKind, // data_class: PUBLIC
    pub operation: &'static str,             // data_class: PUBLIC
    pub cluster_ref: Option<String>,         // data_class: INTERNAL_ONLY
    pub topic_ref: Option<String>,           // data_class: INTERNAL_ONLY
    pub key_ref: Option<String>,             // data_class: INTERNAL_ONLY
    pub payload_ref: Option<String>,         // data_class: INTERNAL_ONLY
    pub group_ref: Option<String>,           // data_class: INTERNAL_ONLY
    pub consumer_ref: Option<String>,        // data_class: INTERNAL_ONLY
    pub partition_ref: Option<String>,       // data_class: INTERNAL_ONLY
    pub offset_ref: Option<String>,          // data_class: INTERNAL_ONLY
    pub schema_subject_ref: Option<String>,  // data_class: INTERNAL_ONLY
    pub headers: Vec<RedpandaEventBusHeaderPlan>, // data_class: INTERNAL_ONLY
    pub configs: Vec<RedpandaEventBusConfigEntry>, // data_class: INTERNAL_ONLY
    pub batch_bound: Option<u32>,            // data_class: INTERNAL_ONLY
    pub register_runtime_planned: bool,      // data_class: INTERNAL_ONLY
    pub offset_commit_planned: bool,         // data_class: INTERNAL_ONLY
    pub executes_runtime: bool,              // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,          // data_class: INTERNAL_ONLY
    pub non_claim_refs: Vec<String>,         // data_class: INTERNAL_ONLY
}

impl RedpandaEventBusCommandPlan {
    fn new(plan_kind: RedpandaEventBusPlanKind, evidence_refs: Vec<String>) -> Self {
        Self {
            operation: plan_kind.operation(),
            plan_kind,
            cluster_ref: None,
            topic_ref: None,
            key_ref: None,
            payload_ref: None,
            group_ref: None,
            consumer_ref: None,
            partition_ref: None,
            offset_ref: None,
            schema_subject_ref: None,
            headers: Vec::new(),
            configs: Vec::new(),
            batch_bound: None,
            register_runtime_planned: false,
            offset_commit_planned: false,
            executes_runtime: false,
            evidence_refs: sorted_unique(evidence_refs),
            non_claim_refs: REDPANDA_EVENT_BUS_ADAPTER_NON_CLAIMS
                .iter()
                .map(|value| (*value).to_owned())
                .collect(),
        }
    }

    fn with_cluster_ref(mut self, cluster_ref: String) -> Self {
        self.cluster_ref = Some(cluster_ref);
        self
    }

    fn with_topic_ref(mut self, topic_ref: String) -> Self {
        self.topic_ref = Some(topic_ref);
        self
    }

    fn with_key_ref(mut self, key_ref: String) -> Self {
        self.key_ref = Some(key_ref);
        self
    }

    fn with_payload_ref(mut self, payload_ref: String) -> Self {
        self.payload_ref = Some(payload_ref);
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

    fn with_schema_subject_ref(mut self, schema_subject_ref: String) -> Self {
        self.schema_subject_ref = Some(schema_subject_ref);
        self
    }

    fn with_headers(mut self, headers: Vec<RedpandaEventBusHeaderPlan>) -> Self {
        self.headers = headers;
        self
    }

    fn with_configs(mut self, configs: Vec<RedpandaEventBusConfigEntry>) -> Self {
        self.configs = configs;
        self
    }

    fn with_batch_bound(mut self, batch_bound: u32) -> Self {
        self.batch_bound = Some(batch_bound);
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RedpandaEventBusPlanFailure {
    InvalidTenant,
    InvalidReference,
    InvalidTopicDescriptor,
    InvalidProducerDescriptor,
    InvalidConsumerDescriptor,
    InvalidSchemaRegistryDescriptor,
    InvalidOffsetDescriptor,
    InvalidEnvelope,
    UnsafeMetadata,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RedpandaEventBusTopicDescriptor {
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub cluster_ref: String,                // data_class: INTERNAL_ONLY
    pub topic_ref: String,                  // data_class: INTERNAL_ONLY
    pub partitions: u32,                    // data_class: INTERNAL_ONLY
    pub replication_factor: u16,            // data_class: INTERNAL_ONLY
    pub min_in_sync_replicas: u16,          // data_class: INTERNAL_ONLY
    pub cleanup_policy: String,             // data_class: INTERNAL_ONLY
    pub retention_ms: u64,                  // data_class: INTERNAL_ONLY
    pub retention_bytes: i64,               // data_class: INTERNAL_ONLY
    pub max_message_bytes: u32,             // data_class: INTERNAL_ONLY
    pub storage_mode: String,               // data_class: INTERNAL_ONLY
    pub schema_subject_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RedpandaEventBusProducerDescriptor {
    pub tenant_id: String,                          // data_class: INTERNAL_ONLY
    pub cluster_ref: String,                        // data_class: INTERNAL_ONLY
    pub topic_ref: String,                          // data_class: INTERNAL_ONLY
    pub client_id_ref: String,                      // data_class: INTERNAL_ONLY
    pub transactional_id_ref: Option<String>,       // data_class: INTERNAL_ONLY
    pub enable_idempotence: bool,                   // data_class: INTERNAL_ONLY
    pub acks: String,                               // data_class: INTERNAL_ONLY
    pub max_in_flight_requests_per_connection: u16, // data_class: INTERNAL_ONLY
    pub delivery_timeout_ms: u64,                   // data_class: INTERNAL_ONLY
    pub linger_ms: u64,                             // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,                 // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RedpandaEventBusConsumerDescriptor {
    pub tenant_id: String,            // data_class: INTERNAL_ONLY
    pub cluster_ref: String,          // data_class: INTERNAL_ONLY
    pub topic_ref: String,            // data_class: INTERNAL_ONLY
    pub group_ref: String,            // data_class: INTERNAL_ONLY
    pub consumer_ref: String,         // data_class: INTERNAL_ONLY
    pub assignment_strategy: String,  // data_class: INTERNAL_ONLY
    pub enable_auto_commit: bool,     // data_class: INTERNAL_ONLY
    pub isolation_level: String,      // data_class: INTERNAL_ONLY
    pub commit_runtime_planned: bool, // data_class: INTERNAL_ONLY
    pub heartbeat_interval_ms: u64,   // data_class: INTERNAL_ONLY
    pub session_timeout_ms: u64,      // data_class: INTERNAL_ONLY
    pub max_poll_records: u32,        // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RedpandaEventBusOffsetDescriptor {
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub topic_ref: String,                 // data_class: INTERNAL_ONLY
    pub group_ref: String,                 // data_class: INTERNAL_ONLY
    pub partition_ref: String,             // data_class: INTERNAL_ONLY
    pub offset_ref: String,                // data_class: INTERNAL_ONLY
    pub external_offset_store_ref: String, // data_class: INTERNAL_ONLY
    pub commit_runtime_planned: bool,      // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RedpandaEventBusSchemaRegistryDescriptor {
    pub tenant_id: String,              // data_class: INTERNAL_ONLY
    pub registry_context_ref: String,   // data_class: INTERNAL_ONLY
    pub subject_ref: String,            // data_class: INTERNAL_ONLY
    pub schema_format: String,          // data_class: INTERNAL_ONLY
    pub compatibility_level: String,    // data_class: INTERNAL_ONLY
    pub register_runtime_planned: bool, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RedpandaEventBusAdapter {
    generated_plans: Vec<RedpandaEventBusCommandPlan>, // data_class: INTERNAL_ONLY
}

impl RedpandaEventBusAdapter {
    pub fn adapter_surface(&self) -> &'static str {
        REDPANDA_EVENT_BUS_ADAPTER_SURFACE
    }

    pub fn adapter_mode_ref(&self) -> &'static str {
        REDPANDA_EVENT_BUS_ADAPTER_MODE_REF
    }

    pub fn cluster_compatibility_plan(
        evidence_refs: Vec<String>,
    ) -> Result<RedpandaEventBusCommandPlan, RedpandaEventBusPlanFailure> {
        validate_evidence(&evidence_refs)?;
        Ok(RedpandaEventBusCommandPlan::new(
            RedpandaEventBusPlanKind::ClusterCompatibility,
            evidence_refs,
        )
        .with_configs(vec![
            config("kafka_protocol_compatibility", "0.11-or-later"),
            config("recommended_client_policy", "latest-supported"),
            config("admin_api_runtime", "not-planned"),
            config("http_proxy_admin_crud", "not-used"),
            config("plan_only", "true"),
        ]))
    }

    pub fn topic_declaration_plan(
        descriptor: &RedpandaEventBusTopicDescriptor,
    ) -> Result<RedpandaEventBusCommandPlan, RedpandaEventBusPlanFailure> {
        validate_topic_descriptor(descriptor)?;
        let mut configs = vec![
            config("partitions", descriptor.partitions.to_string()),
            config(
                "replication.factor",
                descriptor.replication_factor.to_string(),
            ),
            config(
                "min.insync.replicas",
                descriptor.min_in_sync_replicas.to_string(),
            ),
            config("cleanup.policy", &descriptor.cleanup_policy),
            config("retention.ms", descriptor.retention_ms.to_string()),
            config("retention.bytes", descriptor.retention_bytes.to_string()),
            config(
                "max.message.bytes",
                descriptor.max_message_bytes.to_string(),
            ),
            config("redpanda.storage.mode", &descriptor.storage_mode),
            config("topic_creation_runtime", "not-planned"),
        ];
        if descriptor.storage_mode == "cloud" {
            configs.push(config("redpanda.cloud_topic_create_only", "true"));
        }
        if let Some(subject_ref) = &descriptor.schema_subject_ref {
            configs.push(config("schema.subject.ref", subject_ref));
        }
        Ok(RedpandaEventBusCommandPlan::new(
            RedpandaEventBusPlanKind::TopicDeclaration,
            descriptor.evidence_refs.clone(),
        )
        .with_cluster_ref(descriptor.cluster_ref.clone())
        .with_topic_ref(descriptor.topic_ref.clone())
        .with_configs(configs))
    }

    pub fn producer_config_plan(
        descriptor: &RedpandaEventBusProducerDescriptor,
    ) -> Result<RedpandaEventBusCommandPlan, RedpandaEventBusPlanFailure> {
        validate_producer_descriptor(descriptor)?;
        let mut configs = vec![
            config("client.id.ref", &descriptor.client_id_ref),
            config(
                "enable.idempotence",
                bool_wire(descriptor.enable_idempotence),
            ),
            config("acks", &descriptor.acks),
            config(
                "max.in.flight.requests.per.connection",
                descriptor.max_in_flight_requests_per_connection.to_string(),
            ),
            config(
                "delivery.timeout.ms",
                descriptor.delivery_timeout_ms.to_string(),
            ),
            config("linger.ms", descriptor.linger_ms.to_string()),
            config("producer_runtime", "not-planned"),
        ];
        if let Some(transactional_id_ref) = &descriptor.transactional_id_ref {
            configs.push(config("transactional.id.ref", transactional_id_ref));
            configs.push(config("transactions_api_runtime", "not-planned"));
        }
        Ok(RedpandaEventBusCommandPlan::new(
            RedpandaEventBusPlanKind::ProducerConfig,
            descriptor.evidence_refs.clone(),
        )
        .with_cluster_ref(descriptor.cluster_ref.clone())
        .with_topic_ref(descriptor.topic_ref.clone())
        .with_configs(configs))
    }

    pub fn producer_record_plan(
        envelope: &WorkflowEventBusAdapterPublishEnvelope,
        descriptor: &RedpandaEventBusProducerDescriptor,
    ) -> Result<RedpandaEventBusCommandPlan, RedpandaEventBusPlanFailure> {
        validate_publish_envelope(envelope)?;
        validate_producer_descriptor(descriptor)?;
        if envelope.tenant_id != descriptor.tenant_id {
            return Err(RedpandaEventBusPlanFailure::InvalidEnvelope);
        }
        Ok(RedpandaEventBusCommandPlan::new(
            RedpandaEventBusPlanKind::ProducerRecord,
            sorted_unique_nested(vec![
                envelope.evidence_refs.clone(),
                descriptor.evidence_refs.clone(),
            ]),
        )
        .with_cluster_ref(descriptor.cluster_ref.clone())
        .with_topic_ref(descriptor.topic_ref.clone())
        .with_key_ref(envelope.partition_key_ref.clone())
        .with_payload_ref(envelope.payload_ref.clone())
        .with_headers(vec![
            header("ce-specversion", &envelope.cloudevents_specversion),
            header("ce-id", &envelope.event_id),
            header("ce-type", &envelope.event_type),
            header("ce-source", &envelope.source_ref),
            header(
                "ce-subject-ref",
                envelope.subject_ref.as_deref().unwrap_or("none"),
            ),
            header("oyatie-idempotency-key-ref", &envelope.idempotency_key),
            header("oyatie-trace-context-ref", &envelope.trace_context_ref),
            header("oyatie-audit-chain-ref", &envelope.audit_chain_ref),
            header(
                "oyatie-asyncapi-channel-ref",
                envelope.asyncapi_channel_ref.as_deref().unwrap_or("none"),
            ),
        ])
        .with_configs(vec![
            config("record_runtime", "not-planned"),
            config("payload_materialization", "not-planned"),
            config("partition_key_routing", "metadata-only"),
        ]))
    }

    pub fn consumer_assignment_plan(
        descriptor: &RedpandaEventBusConsumerDescriptor,
    ) -> Result<RedpandaEventBusCommandPlan, RedpandaEventBusPlanFailure> {
        validate_consumer_descriptor(descriptor)?;
        Ok(RedpandaEventBusCommandPlan::new(
            RedpandaEventBusPlanKind::ConsumerAssignment,
            descriptor.evidence_refs.clone(),
        )
        .with_cluster_ref(descriptor.cluster_ref.clone())
        .with_topic_ref(descriptor.topic_ref.clone())
        .with_group_ref(descriptor.group_ref.clone())
        .with_consumer_ref(descriptor.consumer_ref.clone())
        .with_configs(vec![
            config("assignment_strategy", &descriptor.assignment_strategy),
            config(
                "enable.auto.commit",
                bool_wire(descriptor.enable_auto_commit),
            ),
            config("isolation.level", &descriptor.isolation_level),
            config(
                "heartbeat.interval.ms",
                descriptor.heartbeat_interval_ms.to_string(),
            ),
            config(
                "session.timeout.ms",
                descriptor.session_timeout_ms.to_string(),
            ),
            config(
                "commit_runtime_planned",
                bool_wire(descriptor.commit_runtime_planned),
            ),
            config("unique_consumer_group_recommended", "true"),
        ]))
    }

    pub fn consumer_poll_plan(
        envelope: &WorkflowEventBusAdapterDeliveryEnvelope,
        descriptor: &RedpandaEventBusConsumerDescriptor,
    ) -> Result<RedpandaEventBusCommandPlan, RedpandaEventBusPlanFailure> {
        validate_delivery_envelope(envelope)?;
        validate_consumer_descriptor(descriptor)?;
        if envelope.tenant_id != descriptor.tenant_id
            || envelope.consumer_ref != descriptor.consumer_ref
        {
            return Err(RedpandaEventBusPlanFailure::InvalidEnvelope);
        }
        Ok(RedpandaEventBusCommandPlan::new(
            RedpandaEventBusPlanKind::ConsumerPoll,
            sorted_unique_nested(vec![
                envelope.evidence_refs.clone(),
                descriptor.evidence_refs.clone(),
            ]),
        )
        .with_cluster_ref(descriptor.cluster_ref.clone())
        .with_topic_ref(descriptor.topic_ref.clone())
        .with_group_ref(descriptor.group_ref.clone())
        .with_consumer_ref(descriptor.consumer_ref.clone())
        .with_offset_ref(envelope.offset_ref.clone())
        .with_payload_ref(envelope.payload_ref.clone())
        .with_batch_bound(descriptor.max_poll_records)
        .with_configs(vec![
            config("poll_runtime", "not-planned"),
            config("enable.auto.commit", "false"),
            config("offset_commit_planned", "false"),
            config("payload_materialization", "not-planned"),
        ]))
    }

    pub fn offset_observation_plan(
        descriptor: &RedpandaEventBusOffsetDescriptor,
    ) -> Result<RedpandaEventBusCommandPlan, RedpandaEventBusPlanFailure> {
        validate_offset_descriptor(descriptor)?;
        Ok(RedpandaEventBusCommandPlan::new(
            RedpandaEventBusPlanKind::OffsetObservation,
            descriptor.evidence_refs.clone(),
        )
        .with_topic_ref(descriptor.topic_ref.clone())
        .with_group_ref(descriptor.group_ref.clone())
        .with_partition_ref(descriptor.partition_ref.clone())
        .with_offset_ref(descriptor.offset_ref.clone())
        .with_configs(vec![
            config("enable.auto.commit", "false"),
            config("offset_commit_planned", "false"),
            config(
                "external_offset_store_ref",
                &descriptor.external_offset_store_ref,
            ),
        ]))
    }

    pub fn schema_registry_subject_plan(
        descriptor: &RedpandaEventBusSchemaRegistryDescriptor,
    ) -> Result<RedpandaEventBusCommandPlan, RedpandaEventBusPlanFailure> {
        validate_schema_registry_descriptor(descriptor)?;
        Ok(RedpandaEventBusCommandPlan::new(
            RedpandaEventBusPlanKind::SchemaRegistrySubject,
            descriptor.evidence_refs.clone(),
        )
        .with_schema_subject_ref(descriptor.subject_ref.clone())
        .with_configs(vec![
            config("registry.context.ref", &descriptor.registry_context_ref),
            config("schema.format", &descriptor.schema_format),
            config("compatibility.level", &descriptor.compatibility_level),
            config(
                "register_runtime_planned",
                bool_wire(descriptor.register_runtime_planned),
            ),
            config("schema_registry_runtime", "not-planned"),
        ]))
    }

    pub fn dead_letter_record_plan(
        envelope: &WorkflowEventBusAdapterDeliveryEnvelope,
        descriptor: &RedpandaEventBusConsumerDescriptor,
        dead_letter_topic_ref: &str,
    ) -> Result<RedpandaEventBusCommandPlan, RedpandaEventBusPlanFailure> {
        validate_delivery_envelope(envelope)?;
        validate_consumer_descriptor(descriptor)?;
        if !is_safe_ref(dead_letter_topic_ref) || !dead_letter_topic_ref.starts_with("topic:") {
            return Err(RedpandaEventBusPlanFailure::InvalidReference);
        }
        Ok(RedpandaEventBusCommandPlan::new(
            RedpandaEventBusPlanKind::DeadLetterRecord,
            sorted_unique_nested(vec![
                envelope.evidence_refs.clone(),
                descriptor.evidence_refs.clone(),
            ]),
        )
        .with_cluster_ref(descriptor.cluster_ref.clone())
        .with_topic_ref(dead_letter_topic_ref.to_owned())
        .with_group_ref(descriptor.group_ref.clone())
        .with_consumer_ref(descriptor.consumer_ref.clone())
        .with_key_ref(envelope.idempotency_key.clone())
        .with_payload_ref(envelope.payload_ref.clone())
        .with_offset_ref(envelope.offset_ref.clone())
        .with_configs(vec![
            config("dead_letter_runtime", "not-planned"),
            config("offset_commit_planned", "false"),
            config("payload_materialization", "not-planned"),
        ]))
    }

    pub fn plan_topic_declaration(
        &mut self,
        descriptor: &RedpandaEventBusTopicDescriptor,
    ) -> Result<&RedpandaEventBusCommandPlan, RedpandaEventBusPlanFailure> {
        let plan = Self::topic_declaration_plan(descriptor)?;
        self.generated_plans.push(plan);
        Ok(self
            .generated_plans
            .last()
            .expect("just pushed redpanda topic plan"))
    }

    pub fn plan_producer_record(
        &mut self,
        envelope: &WorkflowEventBusAdapterPublishEnvelope,
        descriptor: &RedpandaEventBusProducerDescriptor,
    ) -> Result<&RedpandaEventBusCommandPlan, RedpandaEventBusPlanFailure> {
        let plan = Self::producer_record_plan(envelope, descriptor)?;
        self.generated_plans.push(plan);
        Ok(self
            .generated_plans
            .last()
            .expect("just pushed redpanda producer record plan"))
    }

    pub fn plan_consumer_assignment(
        &mut self,
        descriptor: &RedpandaEventBusConsumerDescriptor,
    ) -> Result<&RedpandaEventBusCommandPlan, RedpandaEventBusPlanFailure> {
        let plan = Self::consumer_assignment_plan(descriptor)?;
        self.generated_plans.push(plan);
        Ok(self
            .generated_plans
            .last()
            .expect("just pushed redpanda consumer assignment plan"))
    }

    pub fn generated_plans(&self) -> &[RedpandaEventBusCommandPlan] {
        &self.generated_plans
    }
}

fn config(key: &str, value: impl ToString) -> RedpandaEventBusConfigEntry {
    RedpandaEventBusConfigEntry {
        key: key.to_owned(),
        value: value.to_string(),
    }
}

fn header(key: &str, value_ref: &str) -> RedpandaEventBusHeaderPlan {
    RedpandaEventBusHeaderPlan {
        key: key.to_owned(),
        value_ref: value_ref.to_owned(),
    }
}

fn bool_wire(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn validate_topic_descriptor(
    descriptor: &RedpandaEventBusTopicDescriptor,
) -> Result<(), RedpandaEventBusPlanFailure> {
    validate_tenant(&descriptor.tenant_id)?;
    if !is_safe_ref(&descriptor.cluster_ref)
        || !is_topic_ref(&descriptor.topic_ref)
        || descriptor.partitions == 0
        || descriptor.partitions > REDPANDA_EVENT_BUS_MAX_PARTITIONS
        || !is_odd_replication_factor(descriptor.replication_factor)
        || descriptor.min_in_sync_replicas == 0
        || descriptor.min_in_sync_replicas > descriptor.replication_factor
        || !is_valid_cleanup_policy(&descriptor.cleanup_policy)
        || descriptor.retention_ms == 0
        || descriptor.retention_ms > REDPANDA_EVENT_BUS_MAX_RETENTION_MS
        || descriptor.retention_bytes < -1
        || descriptor.retention_bytes > REDPANDA_EVENT_BUS_MAX_RETENTION_BYTES
        || descriptor.max_message_bytes == 0
        || descriptor.max_message_bytes > REDPANDA_EVENT_BUS_MAX_MESSAGE_BYTES
        || !is_valid_storage_mode(&descriptor.storage_mode)
        || !is_safe_optional_ref(descriptor.schema_subject_ref.as_deref())
    {
        return Err(RedpandaEventBusPlanFailure::InvalidTopicDescriptor);
    }
    validate_evidence(&descriptor.evidence_refs)
}

fn validate_producer_descriptor(
    descriptor: &RedpandaEventBusProducerDescriptor,
) -> Result<(), RedpandaEventBusPlanFailure> {
    validate_tenant(&descriptor.tenant_id)?;
    if !is_safe_ref(&descriptor.cluster_ref)
        || !is_topic_ref(&descriptor.topic_ref)
        || !is_safe_ref(&descriptor.client_id_ref)
        || !is_safe_optional_ref(descriptor.transactional_id_ref.as_deref())
        || !descriptor.enable_idempotence
        || descriptor.acks != "all"
        || descriptor.max_in_flight_requests_per_connection == 0
        || descriptor.max_in_flight_requests_per_connection > 5
        || descriptor.delivery_timeout_ms <= descriptor.linger_ms
        || descriptor.linger_ms > REDPANDA_EVENT_BUS_MAX_LINGER_MS
    {
        return Err(RedpandaEventBusPlanFailure::InvalidProducerDescriptor);
    }
    validate_evidence(&descriptor.evidence_refs)
}

fn validate_consumer_descriptor(
    descriptor: &RedpandaEventBusConsumerDescriptor,
) -> Result<(), RedpandaEventBusPlanFailure> {
    validate_tenant(&descriptor.tenant_id)?;
    if !is_safe_ref(&descriptor.cluster_ref)
        || !is_topic_ref(&descriptor.topic_ref)
        || !is_safe_ref(&descriptor.group_ref)
        || !is_safe_ref(&descriptor.consumer_ref)
        || !is_valid_assignment_strategy(&descriptor.assignment_strategy)
        || descriptor.enable_auto_commit
        || descriptor.isolation_level != "read_committed"
        || descriptor.commit_runtime_planned
        || descriptor.heartbeat_interval_ms == 0
        || descriptor.session_timeout_ms <= descriptor.heartbeat_interval_ms
        || descriptor.max_poll_records == 0
        || descriptor.max_poll_records > REDPANDA_EVENT_BUS_MAX_BATCH_SIZE
    {
        return Err(RedpandaEventBusPlanFailure::InvalidConsumerDescriptor);
    }
    validate_evidence(&descriptor.evidence_refs)
}

fn validate_offset_descriptor(
    descriptor: &RedpandaEventBusOffsetDescriptor,
) -> Result<(), RedpandaEventBusPlanFailure> {
    validate_tenant(&descriptor.tenant_id)?;
    if !is_topic_ref(&descriptor.topic_ref)
        || !is_safe_ref(&descriptor.group_ref)
        || !is_safe_ref(&descriptor.partition_ref)
        || !is_safe_ref(&descriptor.offset_ref)
        || !is_safe_ref(&descriptor.external_offset_store_ref)
        || descriptor.commit_runtime_planned
    {
        return Err(RedpandaEventBusPlanFailure::InvalidOffsetDescriptor);
    }
    validate_evidence(&descriptor.evidence_refs)
}

fn validate_schema_registry_descriptor(
    descriptor: &RedpandaEventBusSchemaRegistryDescriptor,
) -> Result<(), RedpandaEventBusPlanFailure> {
    validate_tenant(&descriptor.tenant_id)?;
    if !is_safe_ref(&descriptor.registry_context_ref)
        || !is_safe_ref(&descriptor.subject_ref)
        || !is_valid_schema_format(&descriptor.schema_format)
        || !is_valid_compatibility_level(&descriptor.compatibility_level)
        || descriptor.register_runtime_planned
    {
        return Err(RedpandaEventBusPlanFailure::InvalidSchemaRegistryDescriptor);
    }
    validate_evidence(&descriptor.evidence_refs)
}

fn validate_publish_envelope(
    envelope: &WorkflowEventBusAdapterPublishEnvelope,
) -> Result<(), RedpandaEventBusPlanFailure> {
    validate_tenant(&envelope.tenant_id)?;
    if envelope.cloudevents_specversion != WORKFLOW_EVENT_BUS_CLOUDEVENTS_SPECVERSION
        || !is_safe_ref(&envelope.cell_id)
        || !is_safe_ref(&envelope.channel_address)
        || !is_safe_ref(&envelope.event_id)
        || !is_safe_ref(&envelope.event_type)
        || !is_safe_ref(&envelope.source_ref)
        || !is_safe_optional_ref(envelope.subject_ref.as_deref())
        || !is_safe_ref(&envelope.partition_key_ref)
        || !is_safe_ref(&envelope.payload_ref)
        || !is_safe_ref(&envelope.idempotency_key)
        || !is_safe_ref(&envelope.trace_context_ref)
        || !is_safe_ref(&envelope.audit_chain_ref)
        || !is_safe_optional_ref(envelope.asyncapi_channel_ref.as_deref())
    {
        return Err(RedpandaEventBusPlanFailure::InvalidEnvelope);
    }
    validate_evidence(&envelope.evidence_refs)
}

fn validate_delivery_envelope(
    envelope: &WorkflowEventBusAdapterDeliveryEnvelope,
) -> Result<(), RedpandaEventBusPlanFailure> {
    validate_tenant(&envelope.tenant_id)?;
    if !is_safe_ref(&envelope.cell_id)
        || !is_safe_ref(&envelope.channel_address)
        || !is_safe_ref(&envelope.event_id)
        || !is_safe_ref(&envelope.event_type)
        || !is_safe_ref(&envelope.consumer_ref)
        || !is_safe_ref(&envelope.offset_ref)
        || !is_safe_ref(&envelope.payload_ref)
        || !is_safe_ref(&envelope.idempotency_key)
        || !is_safe_optional_ref(envelope.replay_cursor_ref.as_deref())
        || !is_safe_ref(&envelope.trace_context_ref)
        || !is_safe_ref(&envelope.audit_chain_ref)
    {
        return Err(RedpandaEventBusPlanFailure::InvalidEnvelope);
    }
    validate_evidence(&envelope.evidence_refs)
}

fn validate_tenant(value: &str) -> Result<(), RedpandaEventBusPlanFailure> {
    if value.starts_with("ten_") && is_safe_ref(value) {
        Ok(())
    } else {
        Err(RedpandaEventBusPlanFailure::InvalidTenant)
    }
}

fn validate_evidence(values: &[String]) -> Result<(), RedpandaEventBusPlanFailure> {
    if !values.is_empty() && values.iter().all(|value| is_safe_ref(value)) {
        Ok(())
    } else if values
        .iter()
        .any(|value| contains_raw_secret_material(value))
    {
        Err(RedpandaEventBusPlanFailure::UnsafeMetadata)
    } else {
        Err(RedpandaEventBusPlanFailure::InvalidReference)
    }
}

fn is_topic_ref(value: &str) -> bool {
    value.starts_with("topic:") && is_safe_ref(value)
}

fn is_safe_optional_ref(value: Option<&str>) -> bool {
    value.map(is_safe_ref).unwrap_or(true)
}

fn is_valid_cleanup_policy(value: &str) -> bool {
    matches!(value, "delete" | "compact" | "compact,delete")
}

fn is_valid_storage_mode(value: &str) -> bool {
    matches!(value, "unset" | "local" | "tiered" | "cloud")
}

fn is_valid_assignment_strategy(value: &str) -> bool {
    matches!(value, "subscribe" | "assign")
}

fn is_valid_schema_format(value: &str) -> bool {
    matches!(value, "avro" | "protobuf" | "json-schema")
}

fn is_valid_compatibility_level(value: &str) -> bool {
    matches!(
        value,
        "BACKWARD"
            | "BACKWARD_TRANSITIVE"
            | "FORWARD"
            | "FORWARD_TRANSITIVE"
            | "FULL"
            | "FULL_TRANSITIVE"
            | "NONE"
    )
}

fn is_odd_replication_factor(value: u16) -> bool {
    value > 0 && value <= REDPANDA_EVENT_BUS_MAX_REPLICATION_FACTOR && value % 2 == 1
}

fn is_safe_ref(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 512
        && !value.chars().any(|c| c.is_control() || c.is_whitespace())
        && !contains_raw_secret_material(value)
}

fn contains_raw_secret_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "raw prompt",
        "raw output",
        "raw payload",
        "secret=",
        "api_key=",
        "apikey=",
        "authorization:",
        "bearer ",
        "provider-key",
        "private key",
        "password=",
        "credential=",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn sorted_unique(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn sorted_unique_nested(values: Vec<Vec<String>>) -> Vec<String> {
    sorted_unique(values.into_iter().flatten().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_and_non_claims_are_plan_only_redpanda_adapter() {
        let adapter = RedpandaEventBusAdapter::default();
        assert_eq!(
            adapter.adapter_surface(),
            "workflow-engine.event-bus.adapter.redpanda"
        );
        assert_eq!(
            adapter.adapter_mode_ref(),
            "workflow-event-bus-adapter-redpanda:plan-only-preview"
        );
        assert!(
            REDPANDA_EVENT_BUS_ADAPTER_NON_CLAIMS
                .contains(&"workflow-event-bus-adapter-redpanda:no-broker-connection")
        );
        assert!(
            REDPANDA_EVENT_BUS_ADAPTER_NON_CLAIMS
                .contains(&"workflow-event-bus-adapter-redpanda:no-schema-registry-runtime")
        );
        assert!(
            REDPANDA_EVENT_BUS_ADAPTER_NON_CLAIMS
                .contains(&"workflow-event-bus-adapter-redpanda:no-hyperscaler-claim")
        );
        let compatibility = RedpandaEventBusAdapter::cluster_compatibility_plan(vec![
            "evidence:event-bus-redpanda:compatibility".to_owned(),
        ])
        .unwrap();
        assert_eq!(compatibility.operation, "ClusterCompatibilityPlan");
        assert_eq!(
            config_value(&compatibility, "kafka_protocol_compatibility"),
            Some("0.11-or-later")
        );
        assert!(!compatibility.executes_runtime);
    }

    #[test]
    fn topic_plan_captures_redpanda_partition_replication_storage_and_schema_shape() {
        let plan = RedpandaEventBusAdapter::topic_declaration_plan(&topic_descriptor()).unwrap();
        assert_eq!(plan.operation, "RedpandaTopicDeclarationPlan");
        assert_eq!(
            plan.topic_ref.as_deref(),
            Some("topic:workflow-runs-events-v1")
        );
        assert_eq!(config_value(&plan, "partitions"), Some("12"));
        assert_eq!(config_value(&plan, "replication.factor"), Some("3"));
        assert_eq!(config_value(&plan, "min.insync.replicas"), Some("2"));
        assert_eq!(config_value(&plan, "redpanda.storage.mode"), Some("tiered"));
        assert_eq!(
            config_value(&plan, "schema.subject.ref"),
            Some("subject:redpanda:workflow-event-value")
        );
        assert_eq!(
            config_value(&plan, "topic_creation_runtime"),
            Some("not-planned")
        );
        assert!(!plan.executes_runtime);

        let mut invalid = topic_descriptor();
        invalid.replication_factor = 2;
        assert_eq!(
            RedpandaEventBusAdapter::topic_declaration_plan(&invalid),
            Err(RedpandaEventBusPlanFailure::InvalidTopicDescriptor)
        );
    }

    #[test]
    fn producer_config_requires_idempotence_acks_all_and_bounded_in_flight() {
        let plan = RedpandaEventBusAdapter::producer_config_plan(&producer_descriptor()).unwrap();
        assert_eq!(plan.operation, "RedpandaProducerConfigPlan");
        assert_eq!(config_value(&plan, "enable.idempotence"), Some("true"));
        assert_eq!(config_value(&plan, "acks"), Some("all"));
        assert_eq!(
            config_value(&plan, "max.in.flight.requests.per.connection"),
            Some("5")
        );
        assert_eq!(
            config_value(&plan, "transactional.id.ref"),
            Some("txn:redpanda:workflow-event-bus")
        );
        assert_eq!(config_value(&plan, "producer_runtime"), Some("not-planned"));

        let mut invalid = producer_descriptor();
        invalid.enable_idempotence = false;
        assert_eq!(
            RedpandaEventBusAdapter::producer_config_plan(&invalid),
            Err(RedpandaEventBusPlanFailure::InvalidProducerDescriptor)
        );
    }

    #[test]
    fn producer_record_plan_binds_cloudevents_asyncapi_idempotency_and_payload_refs_only() {
        let plan = RedpandaEventBusAdapter::producer_record_plan(
            &publish_envelope(),
            &producer_descriptor(),
        )
        .unwrap();
        assert_eq!(plan.operation, "RedpandaProducerRecordPlan");
        assert_eq!(
            plan.key_ref.as_deref(),
            Some("partition:tenant-workflow-run")
        );
        assert_eq!(
            plan.payload_ref.as_deref(),
            Some("body-ref:workflow-run-started")
        );
        assert_eq!(header_value(&plan, "ce-specversion"), Some("1.0"));
        assert_eq!(
            header_value(&plan, "oyatie-idempotency-key-ref"),
            Some("idem:event-bus-adapter:publish:1")
        );
        assert_eq!(config_value(&plan, "record_runtime"), Some("not-planned"));
        assert_eq!(
            config_value(&plan, "payload_materialization"),
            Some("not-planned")
        );
    }

    #[test]
    fn consumer_poll_and_offset_plans_keep_auto_commit_and_commit_runtime_disabled() {
        let assignment =
            RedpandaEventBusAdapter::consumer_assignment_plan(&consumer_descriptor()).unwrap();
        assert_eq!(assignment.operation, "RedpandaConsumerAssignmentPlan");
        assert_eq!(
            config_value(&assignment, "assignment_strategy"),
            Some("assign")
        );
        assert_eq!(
            config_value(&assignment, "enable.auto.commit"),
            Some("false")
        );
        assert_eq!(
            config_value(&assignment, "isolation.level"),
            Some("read_committed")
        );
        assert_eq!(
            config_value(&assignment, "unique_consumer_group_recommended"),
            Some("true")
        );

        let poll = RedpandaEventBusAdapter::consumer_poll_plan(
            &delivery_envelope("delivery-accepted"),
            &consumer_descriptor(),
        )
        .unwrap();
        assert_eq!(poll.batch_bound, Some(250));
        assert!(!poll.offset_commit_planned);
        assert_eq!(config_value(&poll, "poll_runtime"), Some("not-planned"));
        assert_eq!(config_value(&poll, "offset_commit_planned"), Some("false"));

        let offset =
            RedpandaEventBusAdapter::offset_observation_plan(&offset_descriptor()).unwrap();
        assert_eq!(offset.operation, "RedpandaOffsetObservationPlan");
        assert!(!offset.offset_commit_planned);
        assert_eq!(
            config_value(&offset, "external_offset_store_ref"),
            Some("store:workflow-event-bus-offsets")
        );
    }

    #[test]
    fn schema_registry_and_dead_letter_plans_are_metadata_only() {
        let schema =
            RedpandaEventBusAdapter::schema_registry_subject_plan(&schema_descriptor()).unwrap();
        assert_eq!(schema.operation, "RedpandaSchemaRegistrySubjectPlan");
        assert_eq!(
            schema.schema_subject_ref.as_deref(),
            Some("subject:redpanda:workflow-event-value")
        );
        assert_eq!(config_value(&schema, "schema.format"), Some("protobuf"));
        assert_eq!(
            config_value(&schema, "compatibility.level"),
            Some("BACKWARD")
        );
        assert_eq!(
            config_value(&schema, "register_runtime_planned"),
            Some("false")
        );
        assert!(!schema.register_runtime_planned);

        let dlq = RedpandaEventBusAdapter::dead_letter_record_plan(
            &delivery_envelope("delivery-denied"),
            &consumer_descriptor(),
            "topic:workflow-runs-events-dlq-v1",
        )
        .unwrap();
        assert_eq!(dlq.operation, "RedpandaDeadLetterRecordPlan");
        assert_eq!(
            dlq.topic_ref.as_deref(),
            Some("topic:workflow-runs-events-dlq-v1")
        );
        assert!(!dlq.offset_commit_planned);
        assert_eq!(
            config_value(&dlq, "dead_letter_runtime"),
            Some("not-planned")
        );
    }

    #[test]
    fn unsafe_raw_metadata_rejected_without_echoing_secret_material() {
        let mut envelope = publish_envelope();
        envelope.payload_ref =
            "raw payload Authorization: Bearer sk-test customer message".to_owned();
        let failure =
            RedpandaEventBusAdapter::producer_record_plan(&envelope, &producer_descriptor())
                .expect_err("raw payload must be rejected");
        assert!(matches!(
            failure,
            RedpandaEventBusPlanFailure::InvalidEnvelope
                | RedpandaEventBusPlanFailure::UnsafeMetadata
        ));
        let rendered = format!("{failure:?}");
        assert!(!rendered.contains("sk-test"));
        assert!(!rendered.contains("Authorization"));
        assert!(!rendered.contains("raw payload"));
    }

    #[test]
    fn api_generic_adapter_and_redpanda_plans_integrate_without_runtime_claims() {
        let mut api = WorkflowEventBusApi::default();
        let publish_success = api
            .publish_event(publish_request("idem:event-bus-redpanda:publish"))
            .unwrap();
        assert_eq!(publish_success.status, WorkflowEventBusApiStatus::Accepted);
        let delivery_success = api
            .evaluate_delivery(delivery_request("idem:event-bus-redpanda:delivery"))
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

        let mut adapter = RedpandaEventBusAdapter::default();
        adapter.plan_topic_declaration(&topic_descriptor()).unwrap();
        adapter
            .plan_producer_record(&publish_envelope(), &producer_descriptor())
            .unwrap();
        adapter
            .plan_consumer_assignment(&consumer_descriptor())
            .unwrap();
        assert_eq!(adapter.generated_plans().len(), 3);
        assert!(adapter.generated_plans().iter().all(|plan| {
            !plan.executes_runtime
                && plan.non_claim_refs.contains(
                    &"workflow-event-bus-adapter-redpanda:no-broker-connection".to_string(),
                )
                && plan
                    .non_claim_refs
                    .contains(&"workflow-event-bus-adapter-redpanda:no-cloud-runtime".to_string())
                && plan.non_claim_refs.contains(
                    &"workflow-event-bus-adapter-redpanda:no-hyperscaler-claim".to_string(),
                )
        }));
    }

    fn config_value<'a>(plan: &'a RedpandaEventBusCommandPlan, key: &str) -> Option<&'a str> {
        plan.configs
            .iter()
            .find(|entry| entry.key == key)
            .map(|entry| entry.value.as_str())
    }

    fn header_value<'a>(plan: &'a RedpandaEventBusCommandPlan, key: &str) -> Option<&'a str> {
        plan.headers
            .iter()
            .find(|entry| entry.key == key)
            .map(|entry| entry.value_ref.as_str())
    }

    fn topic_descriptor() -> RedpandaEventBusTopicDescriptor {
        RedpandaEventBusTopicDescriptor {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            cluster_ref: "cluster:redpanda:us-east-1:data-plane".to_owned(),
            topic_ref: "topic:workflow-runs-events-v1".to_owned(),
            partitions: 12,
            replication_factor: 3,
            min_in_sync_replicas: 2,
            cleanup_policy: "compact,delete".to_owned(),
            retention_ms: 604_800_000,
            retention_bytes: 1_073_741_824,
            max_message_bytes: 1_048_576,
            storage_mode: "tiered".to_owned(),
            schema_subject_ref: Some("subject:redpanda:workflow-event-value".to_owned()),
            evidence_refs: vec!["evidence:event-bus-redpanda:topic".to_owned()],
        }
    }

    fn producer_descriptor() -> RedpandaEventBusProducerDescriptor {
        RedpandaEventBusProducerDescriptor {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            cluster_ref: "cluster:redpanda:us-east-1:data-plane".to_owned(),
            topic_ref: "topic:workflow-runs-events-v1".to_owned(),
            client_id_ref: "client:redpanda:workflow-engine-event-bus".to_owned(),
            transactional_id_ref: Some("txn:redpanda:workflow-event-bus".to_owned()),
            enable_idempotence: true,
            acks: "all".to_owned(),
            max_in_flight_requests_per_connection: 5,
            delivery_timeout_ms: REDPANDA_EVENT_BUS_DEFAULT_DELIVERY_TIMEOUT_MS,
            linger_ms: REDPANDA_EVENT_BUS_DEFAULT_LINGER_MS,
            evidence_refs: vec!["evidence:event-bus-redpanda:producer".to_owned()],
        }
    }

    fn consumer_descriptor() -> RedpandaEventBusConsumerDescriptor {
        RedpandaEventBusConsumerDescriptor {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            cluster_ref: "cluster:redpanda:us-east-1:data-plane".to_owned(),
            topic_ref: "topic:workflow-runs-events-v1".to_owned(),
            group_ref: "group:redpanda:workflow-state-machine".to_owned(),
            consumer_ref: "consumer:workflow-state-machine".to_owned(),
            assignment_strategy: "assign".to_owned(),
            enable_auto_commit: false,
            isolation_level: "read_committed".to_owned(),
            commit_runtime_planned: false,
            heartbeat_interval_ms: 3000,
            session_timeout_ms: 30_000,
            max_poll_records: 250,
            evidence_refs: vec!["evidence:event-bus-redpanda:consumer".to_owned()],
        }
    }

    fn offset_descriptor() -> RedpandaEventBusOffsetDescriptor {
        RedpandaEventBusOffsetDescriptor {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            topic_ref: "topic:workflow-runs-events-v1".to_owned(),
            group_ref: "group:redpanda:workflow-state-machine".to_owned(),
            partition_ref: "partition:redpanda:workflow-runs:0".to_owned(),
            offset_ref: "offset:redpanda:workflow-runs:42".to_owned(),
            external_offset_store_ref: "store:workflow-event-bus-offsets".to_owned(),
            commit_runtime_planned: false,
            evidence_refs: vec!["evidence:event-bus-redpanda:offset".to_owned()],
        }
    }

    fn schema_descriptor() -> RedpandaEventBusSchemaRegistryDescriptor {
        RedpandaEventBusSchemaRegistryDescriptor {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            registry_context_ref: "schema-context:redpanda:workflow".to_owned(),
            subject_ref: "subject:redpanda:workflow-event-value".to_owned(),
            schema_format: "protobuf".to_owned(),
            compatibility_level: "BACKWARD".to_owned(),
            register_runtime_planned: false,
            evidence_refs: vec!["evidence:event-bus-redpanda:schema".to_owned()],
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
            evidence_refs: vec!["evidence:event-bus-redpanda:publish".to_owned()],
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
            offset_ref: "offset:redpanda:workflow-runs:42".to_owned(),
            payload_ref: "body-ref:workflow-state-transitioned".to_owned(),
            idempotency_key: "idem:event-bus-adapter:delivery:1".to_owned(),
            replay_cursor_ref: Some("cursor:event-bus-redpanda:state".to_owned()),
            trace_context_ref: "trace:event-bus-adapter".to_owned(),
            audit_chain_ref: "audit-chain:event-bus-adapter".to_owned(),
            evidence_refs: vec!["evidence:event-bus-redpanda:delivery".to_owned()],
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
                candidate_offset_ref: "offset:redpanda:workflow-runs:42".to_owned(),
                candidate_evidence_refs: vec!["evidence:event-bus-api:delivery".to_owned()],
            },
        }
    }

    fn boundary(idempotency_key: &str) -> WorkflowEventBusApiBoundaryContext {
        WorkflowEventBusApiBoundaryContext {
            request_id: format!("request:event-bus-redpanda:{idempotency_key}"),
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
