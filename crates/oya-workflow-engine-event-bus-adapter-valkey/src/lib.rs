//! Workflow-engine event-bus Valkey adapter foundation.
//!
//! This crate provides source-level, plan-only Valkey command semantics for the
//! workflow event-bus adapter seam. It models future lease, idempotency,
//! stream publish/read/ack, offset-observation, and rate-limit command plans
//! using Valkey command shapes such as `SET NX PX`, `EVALSHA`, `XADD`,
//! `XREADGROUP`, `XACK`, `INCR`, and `PEXPIRE`. It never opens Valkey
//! connections, executes commands or scripts, coordinates consumer groups,
//! commits offsets, materializes payloads, signs events, deploys to
//! Kubernetes/cloud, schedules tenant workloads, or claims durable event-bus
//! runtime behavior.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use oya_workflow_engine_event_bus_adapter::{
    WORKFLOW_EVENT_BUS_ADAPTER_SURFACE, WORKFLOW_EVENT_BUS_ASYNCAPI_CONTRACT_REF,
    WORKFLOW_EVENT_BUS_CLOUDEVENTS_SPECVERSION, WorkflowEventBusAdapterDeliveryEnvelope,
    WorkflowEventBusAdapterDeliveryReceipt, WorkflowEventBusAdapterPublishEnvelope,
    WorkflowEventBusAdapterPublishReceipt, WorkflowEventBusEventKind,
};

pub const VALKEY_EVENT_BUS_ADAPTER_SURFACE: &str = "workflow-engine.event-bus.adapter.valkey";
pub const VALKEY_EVENT_BUS_ADAPTER_MODE_REF: &str =
    "workflow-event-bus-adapter-valkey:plan-only-preview";
pub const VALKEY_EVENT_BUS_MAX_TTL_MS: u64 = 86_400_000;
pub const VALKEY_EVENT_BUS_MAX_STREAM_BATCH_SIZE: u32 = 1000;
pub const VALKEY_EVENT_BUS_MAX_STREAM_BLOCK_MS: u64 = 30_000;
pub const VALKEY_EVENT_BUS_DEFAULT_STREAM_MAXLEN: u64 = 10_000;

pub const VALKEY_EVENT_BUS_RENEW_IF_OWNER_SCRIPT_SHA_REF: &str =
    "script-sha:workflow-event-bus-valkey:renew-if-owner";
pub const VALKEY_EVENT_BUS_RELEASE_IF_OWNER_SCRIPT_SHA_REF: &str =
    "script-sha:workflow-event-bus-valkey:release-if-owner";

pub const VALKEY_EVENT_BUS_ADAPTER_NON_CLAIMS: [&str; 9] = [
    "workflow-event-bus-adapter-valkey:no-valkey-connection",
    "workflow-event-bus-adapter-valkey:no-command-execution",
    "workflow-event-bus-adapter-valkey:no-script-execution",
    "workflow-event-bus-adapter-valkey:no-stream-runtime",
    "workflow-event-bus-adapter-valkey:no-consumer-group-runtime",
    "workflow-event-bus-adapter-valkey:no-offset-commit-runtime",
    "workflow-event-bus-adapter-valkey:no-payload-materialization",
    "workflow-event-bus-adapter-valkey:no-cloud-runtime",
    "workflow-event-bus-adapter-valkey:no-hyperscaler-claim",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ValkeyEventBusCommandKind {
    SetLeaseNxPx,
    RenewLeaseIfOwnerScript,
    ReleaseLeaseIfOwnerScript,
    RecordIdempotencySetNxPx,
    StreamAddPlan,
    StreamReadGroupPlan,
    StreamAckPlan,
    OffsetObservationSet,
    RateLimitIncrementExpire,
}

impl ValkeyEventBusCommandKind {
    pub const fn command_name(self) -> &'static str {
        match self {
            Self::SetLeaseNxPx | Self::RecordIdempotencySetNxPx | Self::OffsetObservationSet => {
                "SET"
            }
            Self::RenewLeaseIfOwnerScript | Self::ReleaseLeaseIfOwnerScript => "EVALSHA",
            Self::StreamAddPlan => "XADD",
            Self::StreamReadGroupPlan => "XREADGROUP",
            Self::StreamAckPlan => "XACK",
            Self::RateLimitIncrementExpire => "INCR+PEXPIRE",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValkeyEventBusCommandPlan {
    pub command_kind: ValkeyEventBusCommandKind, // data_class: INTERNAL_ONLY
    pub command: String,                         // data_class: INTERNAL_ONLY
    pub key: String,                             // data_class: INTERNAL_ONLY
    pub args: Vec<String>,                       // data_class: INTERNAL_ONLY
    pub ttl_ms: Option<u64>,                     // data_class: INTERNAL_ONLY
    pub script_sha_ref: Option<String>,          // data_class: INTERNAL_ONLY
    pub stream_ref: Option<String>,              // data_class: INTERNAL_ONLY
    pub consumer_group_ref: Option<String>,      // data_class: INTERNAL_ONLY
    pub idempotency_key: Option<String>,         // data_class: INTERNAL_ONLY
    pub offset_commit_planned: Option<bool>,     // data_class: INTERNAL_ONLY
    pub executes_runtime: bool,                  // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,              // data_class: INTERNAL_ONLY
    pub non_claim_refs: Vec<String>,             // data_class: INTERNAL_ONLY
}

impl ValkeyEventBusCommandPlan {
    fn with_ttl_ms(mut self, ttl_ms: u64) -> Self {
        self.ttl_ms = Some(ttl_ms);
        self
    }

    fn with_script_sha_ref(mut self, script_sha_ref: String) -> Self {
        self.script_sha_ref = Some(script_sha_ref);
        self
    }

    fn with_stream_ref(mut self, stream_ref: String) -> Self {
        self.stream_ref = Some(stream_ref);
        self
    }

    fn with_consumer_group_ref(mut self, consumer_group_ref: String) -> Self {
        self.consumer_group_ref = Some(consumer_group_ref);
        self
    }

    fn with_idempotency_key(mut self, idempotency_key: String) -> Self {
        self.idempotency_key = Some(idempotency_key);
        self
    }

    fn with_offset_commit_planned(mut self, offset_commit_planned: bool) -> Self {
        self.offset_commit_planned = Some(offset_commit_planned);
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValkeyEventBusPlanFailure {
    InvalidBatchSize,
    InvalidLimit,
    InvalidTtl,
    PlanOnly { evidence_ref: String },
    UnsafeMetadata,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValkeyEventBusLeaseDescriptor {
    pub tenant_id: String,          // data_class: INTERNAL_ONLY
    pub cell_id: String,            // data_class: INTERNAL_ONLY
    pub lease_key_ref: String,      // data_class: INTERNAL_ONLY
    pub owner_ref: String,          // data_class: INTERNAL_ONLY
    pub token_ref: String,          // data_class: INTERNAL_ONLY
    pub ttl_ms: u64,                // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValkeyEventBusStreamDescriptor {
    pub tenant_id: String,          // data_class: INTERNAL_ONLY
    pub stream_key_ref: String,     // data_class: INTERNAL_ONLY
    pub group_ref: String,          // data_class: INTERNAL_ONLY
    pub consumer_ref: String,       // data_class: INTERNAL_ONLY
    pub batch_size: u32,            // data_class: INTERNAL_ONLY
    pub block_ms: Option<u64>,      // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValkeyEventBusIdempotencyDescriptor {
    pub tenant_id: String,          // data_class: INTERNAL_ONLY
    pub idempotency_key: String,    // data_class: INTERNAL_ONLY
    pub receipt_ref: String,        // data_class: INTERNAL_ONLY
    pub ttl_ms: u64,                // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValkeyEventBusOffsetDescriptor {
    pub tenant_id: String,          // data_class: INTERNAL_ONLY
    pub consumer_ref: String,       // data_class: INTERNAL_ONLY
    pub channel_address: String,    // data_class: PUBLIC
    pub offset_key_ref: String,     // data_class: INTERNAL_ONLY
    pub offset_ref: String,         // data_class: INTERNAL_ONLY
    pub ttl_ms: u64,                // data_class: INTERNAL_ONLY
    pub commit_planned: bool,       // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValkeyEventBusRateLimitDescriptor {
    pub tenant_id: String,          // data_class: INTERNAL_ONLY
    pub bucket_key_ref: String,     // data_class: INTERNAL_ONLY
    pub subject_ref: String,        // data_class: INTERNAL_ONLY
    pub limit: u32,                 // data_class: INTERNAL_ONLY
    pub window_ms: u64,             // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Default)]
pub struct ValkeyEventBusAdapter {
    generated_plans: Vec<ValkeyEventBusCommandPlan>,
}

impl ValkeyEventBusAdapter {
    pub fn lease_acquire_plan(
        descriptor: &ValkeyEventBusLeaseDescriptor,
    ) -> Result<ValkeyEventBusCommandPlan, ValkeyEventBusPlanFailure> {
        validate_lease_descriptor(descriptor)?;
        Ok(command_plan(
            ValkeyEventBusCommandKind::SetLeaseNxPx,
            descriptor.lease_key_ref.clone(),
            vec![
                descriptor.token_ref.clone(),
                "NX".to_owned(),
                "PX".to_owned(),
                descriptor.ttl_ms.to_string(),
                descriptor.owner_ref.clone(),
            ],
            descriptor.evidence_refs.clone(),
        )
        .with_ttl_ms(descriptor.ttl_ms))
    }

    pub fn lease_renew_if_owner_plan(
        descriptor: &ValkeyEventBusLeaseDescriptor,
    ) -> Result<ValkeyEventBusCommandPlan, ValkeyEventBusPlanFailure> {
        validate_lease_descriptor(descriptor)?;
        Ok(command_plan(
            ValkeyEventBusCommandKind::RenewLeaseIfOwnerScript,
            descriptor.lease_key_ref.clone(),
            vec![
                VALKEY_EVENT_BUS_RENEW_IF_OWNER_SCRIPT_SHA_REF.to_owned(),
                "1".to_owned(),
                descriptor.lease_key_ref.clone(),
                descriptor.token_ref.clone(),
                descriptor.ttl_ms.to_string(),
            ],
            descriptor.evidence_refs.clone(),
        )
        .with_ttl_ms(descriptor.ttl_ms)
        .with_script_sha_ref(VALKEY_EVENT_BUS_RENEW_IF_OWNER_SCRIPT_SHA_REF.to_owned()))
    }

    pub fn lease_release_if_owner_plan(
        descriptor: &ValkeyEventBusLeaseDescriptor,
    ) -> Result<ValkeyEventBusCommandPlan, ValkeyEventBusPlanFailure> {
        validate_lease_descriptor(descriptor)?;
        Ok(command_plan(
            ValkeyEventBusCommandKind::ReleaseLeaseIfOwnerScript,
            descriptor.lease_key_ref.clone(),
            vec![
                VALKEY_EVENT_BUS_RELEASE_IF_OWNER_SCRIPT_SHA_REF.to_owned(),
                "1".to_owned(),
                descriptor.lease_key_ref.clone(),
                descriptor.token_ref.clone(),
            ],
            descriptor.evidence_refs.clone(),
        )
        .with_script_sha_ref(VALKEY_EVENT_BUS_RELEASE_IF_OWNER_SCRIPT_SHA_REF.to_owned()))
    }

    pub fn idempotency_record_plan(
        descriptor: &ValkeyEventBusIdempotencyDescriptor,
    ) -> Result<ValkeyEventBusCommandPlan, ValkeyEventBusPlanFailure> {
        validate_idempotency_descriptor(descriptor)?;
        let key = idempotency_key_ref(&descriptor.tenant_id, &descriptor.idempotency_key);
        Ok(command_plan(
            ValkeyEventBusCommandKind::RecordIdempotencySetNxPx,
            key,
            vec![
                descriptor.receipt_ref.clone(),
                "NX".to_owned(),
                "PX".to_owned(),
                descriptor.ttl_ms.to_string(),
            ],
            descriptor.evidence_refs.clone(),
        )
        .with_ttl_ms(descriptor.ttl_ms)
        .with_idempotency_key(descriptor.idempotency_key.clone()))
    }

    pub fn publish_stream_add_plan(
        envelope: &WorkflowEventBusAdapterPublishEnvelope,
    ) -> Result<ValkeyEventBusCommandPlan, ValkeyEventBusPlanFailure> {
        validate_publish_envelope(envelope)?;
        let stream_key = stream_key_ref(&envelope.tenant_id, &envelope.channel_address);
        Ok(command_plan(
            ValkeyEventBusCommandKind::StreamAddPlan,
            stream_key.clone(),
            vec![
                "MAXLEN".to_owned(),
                "~".to_owned(),
                VALKEY_EVENT_BUS_DEFAULT_STREAM_MAXLEN.to_string(),
                "*".to_owned(),
                "cloudevents_specversion".to_owned(),
                envelope.cloudevents_specversion.clone(),
                "event_id_ref".to_owned(),
                envelope.event_id.clone(),
                "event_type".to_owned(),
                envelope.event_type.clone(),
                "source_ref".to_owned(),
                envelope.source_ref.clone(),
                "partition_key_ref".to_owned(),
                envelope.partition_key_ref.clone(),
                "payload_ref".to_owned(),
                envelope.payload_ref.clone(),
                "trace_context_ref".to_owned(),
                envelope.trace_context_ref.clone(),
                "audit_chain_ref".to_owned(),
                envelope.audit_chain_ref.clone(),
                "idempotency_key".to_owned(),
                envelope.idempotency_key.clone(),
            ],
            envelope.evidence_refs.clone(),
        )
        .with_stream_ref(stream_key)
        .with_idempotency_key(envelope.idempotency_key.clone()))
    }

    pub fn delivery_stream_read_group_plan(
        descriptor: &ValkeyEventBusStreamDescriptor,
    ) -> Result<ValkeyEventBusCommandPlan, ValkeyEventBusPlanFailure> {
        validate_stream_descriptor(descriptor)?;
        let mut args = vec![
            "GROUP".to_owned(),
            descriptor.group_ref.clone(),
            descriptor.consumer_ref.clone(),
            "COUNT".to_owned(),
            descriptor.batch_size.to_string(),
        ];
        if let Some(block_ms) = descriptor.block_ms {
            args.push("BLOCK".to_owned());
            args.push(block_ms.to_string());
        }
        args.extend([
            "STREAMS".to_owned(),
            descriptor.stream_key_ref.clone(),
            ">".to_owned(),
        ]);
        Ok(command_plan(
            ValkeyEventBusCommandKind::StreamReadGroupPlan,
            descriptor.stream_key_ref.clone(),
            args,
            descriptor.evidence_refs.clone(),
        )
        .with_stream_ref(descriptor.stream_key_ref.clone())
        .with_consumer_group_ref(descriptor.group_ref.clone())
        .with_offset_commit_planned(false))
    }

    pub fn delivery_stream_ack_plan(
        descriptor: &ValkeyEventBusStreamDescriptor,
        message_id_ref: &str,
    ) -> Result<ValkeyEventBusCommandPlan, ValkeyEventBusPlanFailure> {
        validate_stream_descriptor(descriptor)?;
        if !is_safe_ref(message_id_ref) {
            return Err(ValkeyEventBusPlanFailure::UnsafeMetadata);
        }
        Ok(command_plan(
            ValkeyEventBusCommandKind::StreamAckPlan,
            descriptor.stream_key_ref.clone(),
            vec![descriptor.group_ref.clone(), message_id_ref.to_owned()],
            descriptor.evidence_refs.clone(),
        )
        .with_stream_ref(descriptor.stream_key_ref.clone())
        .with_consumer_group_ref(descriptor.group_ref.clone())
        .with_offset_commit_planned(false))
    }

    pub fn offset_observation_plan(
        descriptor: &ValkeyEventBusOffsetDescriptor,
    ) -> Result<ValkeyEventBusCommandPlan, ValkeyEventBusPlanFailure> {
        validate_offset_descriptor(descriptor)?;
        Ok(command_plan(
            ValkeyEventBusCommandKind::OffsetObservationSet,
            descriptor.offset_key_ref.clone(),
            vec![
                descriptor.offset_ref.clone(),
                "PX".to_owned(),
                descriptor.ttl_ms.to_string(),
            ],
            descriptor.evidence_refs.clone(),
        )
        .with_ttl_ms(descriptor.ttl_ms)
        .with_offset_commit_planned(false))
    }

    pub fn rate_limit_increment_expire_plan(
        descriptor: &ValkeyEventBusRateLimitDescriptor,
    ) -> Result<ValkeyEventBusCommandPlan, ValkeyEventBusPlanFailure> {
        validate_rate_limit_descriptor(descriptor)?;
        Ok(command_plan(
            ValkeyEventBusCommandKind::RateLimitIncrementExpire,
            descriptor.bucket_key_ref.clone(),
            vec![
                "INCR".to_owned(),
                descriptor.bucket_key_ref.clone(),
                "LIMIT".to_owned(),
                descriptor.limit.to_string(),
                "PEXPIRE".to_owned(),
                descriptor.bucket_key_ref.clone(),
                descriptor.window_ms.to_string(),
                descriptor.subject_ref.clone(),
            ],
            descriptor.evidence_refs.clone(),
        )
        .with_ttl_ms(descriptor.window_ms))
    }

    pub fn plan_lease_acquire(
        &mut self,
        descriptor: &ValkeyEventBusLeaseDescriptor,
    ) -> Result<(), ValkeyEventBusPlanFailure> {
        let plan = Self::lease_acquire_plan(descriptor)?;
        self.generated_plans.push(plan);
        Err(ValkeyEventBusPlanFailure::PlanOnly {
            evidence_ref: "workflow-event-bus-valkey-adapter:plan-only-lease-acquire".to_owned(),
        })
    }

    pub fn plan_publish_stream_add(
        &mut self,
        envelope: &WorkflowEventBusAdapterPublishEnvelope,
    ) -> Result<(), ValkeyEventBusPlanFailure> {
        let plan = Self::publish_stream_add_plan(envelope)?;
        self.generated_plans.push(plan);
        Err(ValkeyEventBusPlanFailure::PlanOnly {
            evidence_ref: "workflow-event-bus-valkey-adapter:plan-only-stream-add".to_owned(),
        })
    }

    pub fn plan_delivery_read_and_ack(
        &mut self,
        descriptor: &ValkeyEventBusStreamDescriptor,
        message_id_ref: &str,
    ) -> Result<(), ValkeyEventBusPlanFailure> {
        let read = Self::delivery_stream_read_group_plan(descriptor)?;
        let ack = Self::delivery_stream_ack_plan(descriptor, message_id_ref)?;
        self.generated_plans.push(read);
        self.generated_plans.push(ack);
        Err(ValkeyEventBusPlanFailure::PlanOnly {
            evidence_ref: "workflow-event-bus-valkey-adapter:plan-only-stream-read-ack".to_owned(),
        })
    }

    pub fn generated_plans(&self) -> &[ValkeyEventBusCommandPlan] {
        &self.generated_plans
    }
}

fn command_plan(
    command_kind: ValkeyEventBusCommandKind,
    key: String,
    args: Vec<String>,
    evidence_refs: Vec<String>,
) -> ValkeyEventBusCommandPlan {
    ValkeyEventBusCommandPlan {
        command_kind,
        command: command_kind.command_name().to_owned(),
        key,
        args,
        ttl_ms: None,
        script_sha_ref: None,
        stream_ref: None,
        consumer_group_ref: None,
        idempotency_key: None,
        offset_commit_planned: None,
        executes_runtime: false,
        evidence_refs: sorted_unique(evidence_refs),
        non_claim_refs: VALKEY_EVENT_BUS_ADAPTER_NON_CLAIMS
            .iter()
            .map(|value| (*value).to_owned())
            .collect(),
    }
}

fn validate_lease_descriptor(
    descriptor: &ValkeyEventBusLeaseDescriptor,
) -> Result<(), ValkeyEventBusPlanFailure> {
    validate_ttl_ms(descriptor.ttl_ms)?;
    if is_safe_tenant(&descriptor.tenant_id)
        && is_safe_ref(&descriptor.cell_id)
        && is_safe_ref(&descriptor.lease_key_ref)
        && is_safe_ref(&descriptor.owner_ref)
        && is_safe_ref(&descriptor.token_ref)
        && descriptor
            .evidence_refs
            .iter()
            .all(|value| is_safe_ref(value))
    {
        Ok(())
    } else {
        Err(ValkeyEventBusPlanFailure::UnsafeMetadata)
    }
}

fn validate_stream_descriptor(
    descriptor: &ValkeyEventBusStreamDescriptor,
) -> Result<(), ValkeyEventBusPlanFailure> {
    if descriptor.batch_size == 0 || descriptor.batch_size > VALKEY_EVENT_BUS_MAX_STREAM_BATCH_SIZE
    {
        return Err(ValkeyEventBusPlanFailure::InvalidBatchSize);
    }
    if descriptor
        .block_ms
        .is_some_and(|value| value > VALKEY_EVENT_BUS_MAX_STREAM_BLOCK_MS)
    {
        return Err(ValkeyEventBusPlanFailure::InvalidTtl);
    }
    if is_safe_tenant(&descriptor.tenant_id)
        && is_safe_ref(&descriptor.stream_key_ref)
        && is_safe_ref(&descriptor.group_ref)
        && is_safe_ref(&descriptor.consumer_ref)
        && descriptor
            .evidence_refs
            .iter()
            .all(|value| is_safe_ref(value))
    {
        Ok(())
    } else {
        Err(ValkeyEventBusPlanFailure::UnsafeMetadata)
    }
}

fn validate_idempotency_descriptor(
    descriptor: &ValkeyEventBusIdempotencyDescriptor,
) -> Result<(), ValkeyEventBusPlanFailure> {
    validate_ttl_ms(descriptor.ttl_ms)?;
    if is_safe_tenant(&descriptor.tenant_id)
        && is_safe_ref(&descriptor.idempotency_key)
        && is_safe_ref(&descriptor.receipt_ref)
        && descriptor
            .evidence_refs
            .iter()
            .all(|value| is_safe_ref(value))
    {
        Ok(())
    } else {
        Err(ValkeyEventBusPlanFailure::UnsafeMetadata)
    }
}

fn validate_offset_descriptor(
    descriptor: &ValkeyEventBusOffsetDescriptor,
) -> Result<(), ValkeyEventBusPlanFailure> {
    validate_ttl_ms(descriptor.ttl_ms)?;
    if is_safe_tenant(&descriptor.tenant_id)
        && is_safe_ref(&descriptor.consumer_ref)
        && is_safe_metadata(&descriptor.channel_address)
        && is_safe_ref(&descriptor.offset_key_ref)
        && is_safe_ref(&descriptor.offset_ref)
        && !descriptor.commit_planned
        && descriptor
            .evidence_refs
            .iter()
            .all(|value| is_safe_ref(value))
    {
        Ok(())
    } else {
        Err(ValkeyEventBusPlanFailure::UnsafeMetadata)
    }
}

fn validate_rate_limit_descriptor(
    descriptor: &ValkeyEventBusRateLimitDescriptor,
) -> Result<(), ValkeyEventBusPlanFailure> {
    validate_ttl_ms(descriptor.window_ms)?;
    if descriptor.limit == 0 {
        return Err(ValkeyEventBusPlanFailure::InvalidLimit);
    }
    if is_safe_tenant(&descriptor.tenant_id)
        && is_safe_ref(&descriptor.bucket_key_ref)
        && is_safe_ref(&descriptor.subject_ref)
        && descriptor
            .evidence_refs
            .iter()
            .all(|value| is_safe_ref(value))
    {
        Ok(())
    } else {
        Err(ValkeyEventBusPlanFailure::UnsafeMetadata)
    }
}

fn validate_publish_envelope(
    envelope: &WorkflowEventBusAdapterPublishEnvelope,
) -> Result<(), ValkeyEventBusPlanFailure> {
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
        Err(ValkeyEventBusPlanFailure::UnsafeMetadata)
    }
}

fn validate_ttl_ms(ttl_ms: u64) -> Result<(), ValkeyEventBusPlanFailure> {
    if ttl_ms == 0 || ttl_ms > VALKEY_EVENT_BUS_MAX_TTL_MS {
        Err(ValkeyEventBusPlanFailure::InvalidTtl)
    } else {
        Ok(())
    }
}

fn stream_key_ref(tenant_id: &str, channel_address: &str) -> String {
    format!("valkey-key:event-bus:stream:{tenant_id}:{channel_address}")
}

fn idempotency_key_ref(tenant_id: &str, idempotency_key: &str) -> String {
    format!("valkey-key:event-bus:idempotency:{tenant_id}:{idempotency_key}")
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
    fn constants_commands_and_non_claims_are_plan_only() {
        assert_eq!(
            VALKEY_EVENT_BUS_ADAPTER_SURFACE,
            "workflow-engine.event-bus.adapter.valkey"
        );
        assert_eq!(
            ValkeyEventBusCommandKind::SetLeaseNxPx.command_name(),
            "SET"
        );
        assert_eq!(
            ValkeyEventBusCommandKind::RenewLeaseIfOwnerScript.command_name(),
            "EVALSHA"
        );
        assert_eq!(
            ValkeyEventBusCommandKind::StreamAddPlan.command_name(),
            "XADD"
        );
        assert_eq!(
            ValkeyEventBusCommandKind::StreamReadGroupPlan.command_name(),
            "XREADGROUP"
        );
        assert_eq!(
            ValkeyEventBusCommandKind::StreamAckPlan.command_name(),
            "XACK"
        );
        assert!(
            VALKEY_EVENT_BUS_ADAPTER_NON_CLAIMS
                .contains(&"workflow-event-bus-adapter-valkey:no-command-execution")
        );
        assert!(
            VALKEY_EVENT_BUS_ADAPTER_NON_CLAIMS
                .contains(&"workflow-event-bus-adapter-valkey:no-hyperscaler-claim")
        );
    }

    #[test]
    fn lease_plans_use_set_nx_px_and_owner_guarded_evalsha() {
        let lease = lease_descriptor();
        let acquire = ValkeyEventBusAdapter::lease_acquire_plan(&lease).unwrap();
        let renew = ValkeyEventBusAdapter::lease_renew_if_owner_plan(&lease).unwrap();
        let release = ValkeyEventBusAdapter::lease_release_if_owner_plan(&lease).unwrap();

        assert_eq!(acquire.command, "SET");
        assert_eq!(
            acquire.command_kind,
            ValkeyEventBusCommandKind::SetLeaseNxPx
        );
        assert!(acquire.args.contains(&"NX".to_owned()));
        assert!(acquire.args.contains(&"PX".to_owned()));
        assert_eq!(acquire.ttl_ms, Some(30_000));
        assert!(!acquire.executes_runtime);
        assert_eq!(renew.command, "EVALSHA");
        assert_eq!(
            renew.script_sha_ref.as_deref(),
            Some(VALKEY_EVENT_BUS_RENEW_IF_OWNER_SCRIPT_SHA_REF)
        );
        assert_eq!(renew.args[1], "1");
        assert_eq!(renew.args[2], lease.lease_key_ref);
        assert_eq!(
            release.script_sha_ref.as_deref(),
            Some(VALKEY_EVENT_BUS_RELEASE_IF_OWNER_SCRIPT_SHA_REF)
        );

        let mut invalid = lease_descriptor();
        invalid.ttl_ms = 0;
        assert_eq!(
            ValkeyEventBusAdapter::lease_acquire_plan(&invalid).unwrap_err(),
            ValkeyEventBusPlanFailure::InvalidTtl
        );

        let mut adapter = ValkeyEventBusAdapter::default();
        assert!(matches!(
            adapter.plan_lease_acquire(&lease_descriptor()),
            Err(ValkeyEventBusPlanFailure::PlanOnly { .. })
        ));
        assert_eq!(adapter.generated_plans().len(), 1);
    }

    #[test]
    fn idempotency_record_plan_uses_set_nx_px_receipt_refs() {
        let plan =
            ValkeyEventBusAdapter::idempotency_record_plan(&idempotency_descriptor()).unwrap();

        assert_eq!(plan.command, "SET");
        assert_eq!(
            plan.command_kind,
            ValkeyEventBusCommandKind::RecordIdempotencySetNxPx
        );
        assert!(
            plan.key
                .contains("valkey-key:event-bus:idempotency:ten_workflow_event_bus")
        );
        assert_eq!(plan.args[0], "receipt:event-bus-valkey:publish:001");
        assert!(plan.args.contains(&"NX".to_owned()));
        assert!(plan.args.contains(&"PX".to_owned()));
        assert_eq!(plan.ttl_ms, Some(60_000));
        assert!(!plan.executes_runtime);

        let mut invalid = idempotency_descriptor();
        invalid.receipt_ref = "raw output Authorization: Bearer sk-test".to_owned();
        let err = ValkeyEventBusAdapter::idempotency_record_plan(&invalid).unwrap_err();
        assert_eq!(err, ValkeyEventBusPlanFailure::UnsafeMetadata);
        let rendered = format!("{err:?}");
        assert!(!rendered.contains("Authorization"));
        assert!(!rendered.contains("sk-test"));
    }

    #[test]
    fn publish_stream_xadd_plan_uses_cloudevents_asyncapi_and_payload_refs_only() {
        let envelope = publish_envelope();
        let plan = ValkeyEventBusAdapter::publish_stream_add_plan(&envelope).unwrap();

        assert_eq!(plan.command, "XADD");
        assert_eq!(plan.command_kind, ValkeyEventBusCommandKind::StreamAddPlan);
        assert!(plan.args.contains(&"MAXLEN".to_owned()));
        assert!(plan.args.contains(&"~".to_owned()));
        assert!(plan.args.contains(&"cloudevents_specversion".to_owned()));
        assert!(
            plan.args
                .contains(&WORKFLOW_EVENT_BUS_CLOUDEVENTS_SPECVERSION.to_owned())
        );
        assert!(plan.args.contains(&"payload_ref".to_owned()));
        assert!(
            plan.args
                .contains(&"body-ref:workflow-run-started".to_owned())
        );
        assert_eq!(
            plan.stream_ref.as_deref(),
            Some("valkey-key:event-bus:stream:ten_workflow_event_bus:workflow.runs.events.v1")
        );
        assert!(!plan.executes_runtime);

        let mut adapter = ValkeyEventBusAdapter::default();
        assert!(matches!(
            adapter.plan_publish_stream_add(&envelope),
            Err(ValkeyEventBusPlanFailure::PlanOnly { .. })
        ));
        assert_eq!(adapter.generated_plans().len(), 1);
    }

    #[test]
    fn delivery_read_group_and_ack_plans_are_guarded_and_do_not_commit_offsets() {
        let stream = stream_descriptor();
        let read = ValkeyEventBusAdapter::delivery_stream_read_group_plan(&stream).unwrap();
        let ack =
            ValkeyEventBusAdapter::delivery_stream_ack_plan(&stream, "msgid:1700000000-0").unwrap();

        assert_eq!(read.command, "XREADGROUP");
        assert!(read.args.starts_with(&[
            "GROUP".to_owned(),
            "group:workflow-state-machine".to_owned(),
            "consumer:workflow-state-machine:worker-1".to_owned(),
        ]));
        assert!(read.args.contains(&"COUNT".to_owned()));
        assert!(read.args.contains(&"BLOCK".to_owned()));
        assert_eq!(read.offset_commit_planned, Some(false));
        assert_eq!(ack.command, "XACK");
        assert_eq!(
            ack.args,
            vec!["group:workflow-state-machine", "msgid:1700000000-0"]
        );
        assert_eq!(ack.offset_commit_planned, Some(false));
        assert!(!ack.executes_runtime);

        let mut invalid = stream_descriptor();
        invalid.batch_size = 0;
        assert_eq!(
            ValkeyEventBusAdapter::delivery_stream_read_group_plan(&invalid).unwrap_err(),
            ValkeyEventBusPlanFailure::InvalidBatchSize
        );

        let mut adapter = ValkeyEventBusAdapter::default();
        assert!(matches!(
            adapter.plan_delivery_read_and_ack(&stream_descriptor(), "msgid:1700000000-0"),
            Err(ValkeyEventBusPlanFailure::PlanOnly { .. })
        ));
        assert_eq!(adapter.generated_plans().len(), 2);
    }

    #[test]
    fn offset_and_rate_limit_plans_are_non_executing_and_bounded() {
        let offset = ValkeyEventBusAdapter::offset_observation_plan(&offset_descriptor()).unwrap();
        assert_eq!(offset.command, "SET");
        assert_eq!(
            offset.command_kind,
            ValkeyEventBusCommandKind::OffsetObservationSet
        );
        assert_eq!(offset.offset_commit_planned, Some(false));
        assert_eq!(offset.args, vec!["offset:partition-0:42", "PX", "60000"]);
        assert!(!offset.executes_runtime);

        let rate =
            ValkeyEventBusAdapter::rate_limit_increment_expire_plan(&rate_limit_descriptor())
                .unwrap();
        assert_eq!(rate.command, "INCR+PEXPIRE");
        assert!(rate.args.contains(&"INCR".to_owned()));
        assert!(rate.args.contains(&"PEXPIRE".to_owned()));
        assert_eq!(rate.ttl_ms, Some(1_000));
        assert!(!rate.executes_runtime);

        let mut invalid = offset_descriptor();
        invalid.commit_planned = true;
        assert_eq!(
            ValkeyEventBusAdapter::offset_observation_plan(&invalid).unwrap_err(),
            ValkeyEventBusPlanFailure::UnsafeMetadata
        );
    }

    #[test]
    fn unsafe_raw_metadata_is_rejected_without_echo_before_command_plan() {
        let mut envelope = publish_envelope();
        envelope.payload_ref =
            "raw payload Authorization: Bearer sk-test customer message".to_owned();
        let err = ValkeyEventBusAdapter::publish_stream_add_plan(&envelope).unwrap_err();

        assert_eq!(err, ValkeyEventBusPlanFailure::UnsafeMetadata);
        let rendered = format!("{err:?}");
        assert!(!rendered.contains("raw payload"));
        assert!(!rendered.contains("Authorization"));
        assert!(!rendered.contains("sk-test"));
    }

    #[test]
    fn api_generic_adapter_and_valkey_plans_integrate_without_runtime_claims() {
        let mut api = WorkflowEventBusApi::default();
        let publish_success = api
            .publish_event(publish_request("idem:event-bus-valkey:publish"))
            .unwrap();
        let delivery_success = api
            .evaluate_delivery(delivery_request("idem:event-bus-valkey:delivery"))
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

        let stream_plan = ValkeyEventBusAdapter::publish_stream_add_plan(
            &publish_envelope_from_api(&publish_success),
        )
        .unwrap();
        let read_plan =
            ValkeyEventBusAdapter::delivery_stream_read_group_plan(&stream_descriptor()).unwrap();
        let offset_plan =
            ValkeyEventBusAdapter::offset_observation_plan(&offset_descriptor()).unwrap();

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
        assert_eq!(stream_plan.command, "XADD");
        assert_eq!(read_plan.offset_commit_planned, Some(false));
        assert_eq!(offset_plan.offset_commit_planned, Some(false));
        assert!(
            stream_plan
                .non_claim_refs
                .iter()
                .any(|value| value.contains("no-valkey-connection"))
        );
    }

    fn lease_descriptor() -> ValkeyEventBusLeaseDescriptor {
        ValkeyEventBusLeaseDescriptor {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            cell_id: "cell:us-east-1a".to_owned(),
            lease_key_ref: "valkey-key:event-bus:lease:workflow-worker:001".to_owned(),
            owner_ref: "owner:event-bus-worker:001".to_owned(),
            token_ref: "lease-token-ref:event-bus-worker:001".to_owned(),
            ttl_ms: 30_000,
            evidence_refs: vec!["evidence:event-bus-valkey:lease".to_owned()],
        }
    }

    fn stream_descriptor() -> ValkeyEventBusStreamDescriptor {
        ValkeyEventBusStreamDescriptor {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            stream_key_ref:
                "valkey-key:event-bus:stream:ten_workflow_event_bus:workflow.state.events.v1"
                    .to_owned(),
            group_ref: "group:workflow-state-machine".to_owned(),
            consumer_ref: "consumer:workflow-state-machine:worker-1".to_owned(),
            batch_size: 25,
            block_ms: Some(2_000),
            evidence_refs: vec!["evidence:event-bus-valkey:stream".to_owned()],
        }
    }

    fn idempotency_descriptor() -> ValkeyEventBusIdempotencyDescriptor {
        ValkeyEventBusIdempotencyDescriptor {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            idempotency_key: "idem:event-bus-valkey:publish:001".to_owned(),
            receipt_ref: "receipt:event-bus-valkey:publish:001".to_owned(),
            ttl_ms: 60_000,
            evidence_refs: vec!["evidence:event-bus-valkey:idempotency".to_owned()],
        }
    }

    fn offset_descriptor() -> ValkeyEventBusOffsetDescriptor {
        ValkeyEventBusOffsetDescriptor {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            consumer_ref: "consumer:workflow-state-machine".to_owned(),
            channel_address: "workflow.state.events.v1".to_owned(),
            offset_key_ref: "valkey-key:event-bus:offset:workflow-state-machine".to_owned(),
            offset_ref: "offset:partition-0:42".to_owned(),
            ttl_ms: 60_000,
            commit_planned: false,
            evidence_refs: vec!["evidence:event-bus-valkey:offset".to_owned()],
        }
    }

    fn rate_limit_descriptor() -> ValkeyEventBusRateLimitDescriptor {
        ValkeyEventBusRateLimitDescriptor {
            tenant_id: "ten_workflow_event_bus".to_owned(),
            bucket_key_ref: "valkey-key:event-bus:rate-limit:publish:tenant".to_owned(),
            subject_ref: "subject:tenant:workflow-event-bus".to_owned(),
            limit: 100,
            window_ms: 1_000,
            evidence_refs: vec!["evidence:event-bus-valkey:rate-limit".to_owned()],
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
            evidence_refs: vec!["evidence:event-bus-valkey:publish".to_owned()],
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
        WorkflowEventBusAdapterDeliveryEnvelope {
            tenant_id: success.event.tenant_id.clone(),
            cell_id: success.event.cell_id.clone(),
            channel_address: success.event.channel_address.clone().unwrap(),
            event_id: "event:workflow-state:001".to_owned(),
            event_type: success.event.event_type.clone(),
            consumer_ref: success.event.consumer_ref.clone().unwrap(),
            offset_ref: success.event.offset_ref.clone().unwrap(),
            payload_ref: "body-ref:workflow-state-transitioned".to_owned(),
            idempotency_key: success.metadata.idempotency_key.clone(),
            replay_cursor_ref: Some("cursor:event-bus-valkey:state".to_owned()),
            trace_context_ref: success.metadata.trace_context_ref.clone(),
            audit_chain_ref: "audit-chain:event-bus-adapter".to_owned(),
            evidence_refs: vec!["evidence:event-bus-valkey:delivery".to_owned()],
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
            request_id: format!("request:event-bus-valkey:{idempotency_key}"),
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
