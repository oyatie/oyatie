//! Tenant RBAC audit-chain runtime evidence contract.
//!
//! This review-only crate records the runtime audit-chain evidence that must
//! exist before FD-001 tenant workloads can claim production audit emission on
//! the future Oyatie Cloud substrate. It validates official event/log/trace,
//! canonicalization, broker, and Merkle evidence requirements against the
//! existing Tenant RBAC audit-chain emission plan. It does not attach a
//! runtime emitter, write-ahead log runtime, broker publisher, Merkle sealer,
//! cloud audit sink, production evidence, or runtime audit-chain emission.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use iam_tenant_rbac_audit_chain_emission::{
    TenantRbacAuditChainEmissionError, TenantRbacAuditChainEmissionPlan,
    tenant_rbac_audit_chain_emission_plan, validate_tenant_rbac_audit_chain_emission_plan,
};

const SCHEMA_VERSION: u32 = 1;
const MIN_REQUIREMENT_COUNT: usize = 15;
const PLAN_NAME: &str = "tenant-rbac-audit-chain-runtime-evidence-plan";
const SERVICE_NAME: &str = "tenant-rbac";
const SUBSTRATE_NAME: &str = "oyatie-cloud";
const TENANT_NAMESPACE: &str = "oyatie-fd001-tenant-rbac-dev";
const TENANT_PARTITION: &str = "tenant-scoped";
const OUTBOX_TOPIC: &str = "oyatie.platform.audit";
const SOURCE_PLAN_REF: &str =
    "iam/core/tenant-rbac-audit-chain-emission/src/lib.rs::tenant_rbac_audit_chain_emission_plan";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AuditChainRuntimeEvidenceRequirementKind {
    CloudEventEnvelopeObserved,
    TraceContextPropagated,
    OTelLogRecordMapped,
    TenantPartitionObserved,
    IdempotencyDeduplicated,
    PayloadDigestVerified,
    SensitivePayloadRedacted,
    WalAppendAcknowledged,
    OutboxPublishConfirmed,
    BrokerAckObserved,
    MerkleLeafIncluded,
    MerkleRootSealed,
    SinkIngestionObserved,
    ReplayRecoveryObserved,
    FailurePathAuditRecorded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditChainRuntimeEvidenceRequirement {
    pub requirement_id: &'static str, // data_class: PUBLIC
    pub requirement_kind: AuditChainRuntimeEvidenceRequirementKind, // data_class: PUBLIC
    pub event_scope: &'static str,    // data_class: PUBLIC
    pub official_doc_url: &'static str, // data_class: PUBLIC
    pub expected_evidence_ref: &'static str, // data_class: INTERNAL_ONLY
    pub source_plan_ref: &'static str, // data_class: INTERNAL_ONLY
    pub tenant_namespace: &'static str, // data_class: INTERNAL_ONLY
    pub tenant_partition: &'static str, // data_class: INTERNAL_ONLY
    pub outbox_topic: &'static str,   // data_class: INTERNAL_ONLY
    pub requires_cloudevent_envelope: bool, // data_class: PUBLIC
    pub requires_trace_context: bool, // data_class: PUBLIC
    pub requires_otel_log_record: bool, // data_class: PUBLIC
    pub requires_tenant_partition: bool, // data_class: PUBLIC
    pub requires_idempotency_dedupe: bool, // data_class: PUBLIC
    pub requires_payload_digest: bool, // data_class: PUBLIC
    pub requires_sensitive_payload_redaction: bool, // data_class: PUBLIC
    pub requires_wal_append: bool,    // data_class: PUBLIC
    pub requires_outbox_publish: bool, // data_class: PUBLIC
    pub requires_broker_ack: bool,    // data_class: PUBLIC
    pub requires_merkle_leaf: bool,   // data_class: PUBLIC
    pub requires_merkle_root_seal: bool, // data_class: PUBLIC
    pub requires_sink_ingestion: bool, // data_class: PUBLIC
    pub requires_replay_recovery: bool, // data_class: PUBLIC
    pub requires_failure_path_audit: bool, // data_class: PUBLIC
    pub runtime_evidence_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,          // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacAuditChainRuntimeEvidencePlan {
    pub plan_name: &'static str,                      // data_class: PUBLIC
    pub service_name: &'static str,                   // data_class: PUBLIC
    pub substrate_name: &'static str,                 // data_class: PUBLIC
    pub tenant_namespace: &'static str,               // data_class: INTERNAL_ONLY
    pub tenant_partition: &'static str,               // data_class: INTERNAL_ONLY
    pub audit_chain_emission_plan_name: &'static str, // data_class: INTERNAL_ONLY
    pub outbox_topic: &'static str,                   // data_class: PUBLIC
    pub event_schema_count: usize,                    // data_class: PUBLIC
    pub required_context_attribute_count: usize,      // data_class: PUBLIC
    pub required_extension_attribute_count: usize,    // data_class: PUBLIC
    pub requirements: Vec<AuditChainRuntimeEvidenceRequirement>, // data_class: INTERNAL_ONLY
    pub fd001_product_delivery_master_goal_preserved: bool, // data_class: PUBLIC
    pub oyatie_cloud_substrate_proof_required: bool,  // data_class: PUBLIC
    pub official_docs_required: bool,                 // data_class: PUBLIC
    pub cloudevents_envelope_evidence_required: bool, // data_class: PUBLIC
    pub trace_context_evidence_required: bool,        // data_class: PUBLIC
    pub otel_log_record_mapping_required: bool,       // data_class: PUBLIC
    pub tenant_partition_evidence_required: bool,     // data_class: PUBLIC
    pub idempotency_dedupe_evidence_required: bool,   // data_class: PUBLIC
    pub payload_digest_match_required: bool,          // data_class: PUBLIC
    pub sensitive_payload_redaction_required: bool,   // data_class: PUBLIC
    pub wal_append_evidence_required: bool,           // data_class: PUBLIC
    pub outbox_publish_evidence_required: bool,       // data_class: PUBLIC
    pub broker_ack_evidence_required: bool,           // data_class: PUBLIC
    pub merkle_leaf_inclusion_required: bool,         // data_class: PUBLIC
    pub merkle_root_seal_required: bool,              // data_class: PUBLIC
    pub sink_ingestion_required: bool,                // data_class: PUBLIC
    pub replay_recovery_required: bool,               // data_class: PUBLIC
    pub failure_path_audit_required: bool,            // data_class: PUBLIC
    pub review_only_contract: bool,                   // data_class: PUBLIC
    pub runtime_emitter_attached: bool,               // data_class: INTERNAL_ONLY
    pub write_ahead_log_runtime_attached: bool,       // data_class: INTERNAL_ONLY
    pub broker_publish_runtime_attached: bool,        // data_class: INTERNAL_ONLY
    pub merkle_sealer_runtime_attached: bool,         // data_class: INTERNAL_ONLY
    pub cloud_audit_sink_attached: bool,              // data_class: INTERNAL_ONLY
    pub runtime_audit_chain_emission_attached: bool,  // data_class: INTERNAL_ONLY
    pub production_audit_emission_evidence_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                          // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantRbacAuditChainRuntimeEvidenceError {
    AuditChainEmission(TenantRbacAuditChainEmissionError),
    InvalidPlanName,
    InvalidServiceName,
    InvalidSubstrateName,
    InvalidTenantNamespace,
    InvalidTenantPartition,
    InvalidAuditChainEmissionPlanName,
    InvalidOutboxTopic,
    MissingRequirements,
    DuplicateRequirement(String),
    MissingRequirementKind(AuditChainRuntimeEvidenceRequirementKind),
    InvalidRequirementId,
    InvalidEventScope,
    InvalidOfficialDocUrl,
    InvalidExpectedEvidenceRef,
    InvalidSourcePlanRef,
    MissingRequiredControl(&'static str),
    RuntimeAttachmentOverclaim,
}

pub fn tenant_rbac_audit_chain_runtime_evidence_plan()
-> Result<TenantRbacAuditChainRuntimeEvidencePlan, TenantRbacAuditChainRuntimeEvidenceError> {
    let emission_plan = tenant_rbac_audit_chain_emission_plan();
    validate_tenant_rbac_audit_chain_emission_plan(&emission_plan)
        .map_err(TenantRbacAuditChainRuntimeEvidenceError::AuditChainEmission)?;

    Ok(TenantRbacAuditChainRuntimeEvidencePlan {
        plan_name: PLAN_NAME,
        service_name: SERVICE_NAME,
        substrate_name: SUBSTRATE_NAME,
        tenant_namespace: TENANT_NAMESPACE,
        tenant_partition: TENANT_PARTITION,
        audit_chain_emission_plan_name: emission_plan.plan_name,
        outbox_topic: emission_plan.outbox_topic,
        event_schema_count: emission_plan.event_schemas.len(),
        required_context_attribute_count: emission_plan.required_context_attributes.len(),
        required_extension_attribute_count: emission_plan.required_extension_attributes.len(),
        requirements: runtime_requirements(&emission_plan),
        fd001_product_delivery_master_goal_preserved: true,
        oyatie_cloud_substrate_proof_required: true,
        official_docs_required: true,
        cloudevents_envelope_evidence_required: true,
        trace_context_evidence_required: true,
        otel_log_record_mapping_required: true,
        tenant_partition_evidence_required: true,
        idempotency_dedupe_evidence_required: true,
        payload_digest_match_required: true,
        sensitive_payload_redaction_required: true,
        wal_append_evidence_required: true,
        outbox_publish_evidence_required: true,
        broker_ack_evidence_required: true,
        merkle_leaf_inclusion_required: true,
        merkle_root_seal_required: true,
        sink_ingestion_required: true,
        replay_recovery_required: true,
        failure_path_audit_required: true,
        review_only_contract: true,
        runtime_emitter_attached: false,
        write_ahead_log_runtime_attached: false,
        broker_publish_runtime_attached: false,
        merkle_sealer_runtime_attached: false,
        cloud_audit_sink_attached: false,
        runtime_audit_chain_emission_attached: false,
        production_audit_emission_evidence_attached: false,
        schema_version: SCHEMA_VERSION,
    })
}

pub fn validate_tenant_rbac_audit_chain_runtime_evidence_plan(
    plan: &TenantRbacAuditChainRuntimeEvidencePlan,
) -> Result<(), TenantRbacAuditChainRuntimeEvidenceError> {
    validate_slug(
        plan.plan_name,
        TenantRbacAuditChainRuntimeEvidenceError::InvalidPlanName,
    )?;
    if plan.service_name != SERVICE_NAME {
        return Err(TenantRbacAuditChainRuntimeEvidenceError::InvalidServiceName);
    }
    if plan.substrate_name != SUBSTRATE_NAME {
        return Err(TenantRbacAuditChainRuntimeEvidenceError::InvalidSubstrateName);
    }
    if plan.tenant_namespace != TENANT_NAMESPACE {
        return Err(TenantRbacAuditChainRuntimeEvidenceError::InvalidTenantNamespace);
    }
    if plan.tenant_partition != TENANT_PARTITION {
        return Err(TenantRbacAuditChainRuntimeEvidenceError::InvalidTenantPartition);
    }
    if plan.audit_chain_emission_plan_name != "tenant-rbac-audit-chain-emission" {
        return Err(TenantRbacAuditChainRuntimeEvidenceError::InvalidAuditChainEmissionPlanName);
    }
    if plan.outbox_topic != OUTBOX_TOPIC || has_unsafe_text(plan.outbox_topic) {
        return Err(TenantRbacAuditChainRuntimeEvidenceError::InvalidOutboxTopic);
    }
    if plan.event_schema_count < 9
        || plan.required_context_attribute_count < 8
        || plan.required_extension_attribute_count < 7
        || plan.requirements.len() < MIN_REQUIREMENT_COUNT
        || plan.schema_version != SCHEMA_VERSION
    {
        return Err(TenantRbacAuditChainRuntimeEvidenceError::MissingRequirements);
    }
    validate_required_controls(plan)?;
    validate_nonclaims(plan)?;
    validate_runtime_requirements(plan)?;
    Ok(())
}

pub fn audit_chain_runtime_evidence_doc_urls(
    plan: &TenantRbacAuditChainRuntimeEvidencePlan,
) -> Vec<&'static str> {
    plan.requirements
        .iter()
        .map(|requirement| requirement.official_doc_url)
        .collect()
}

fn runtime_requirements(
    emission_plan: &TenantRbacAuditChainEmissionPlan,
) -> Vec<AuditChainRuntimeEvidenceRequirement> {
    vec![
        requirement(
            "cloudevent-envelope-observed",
            AuditChainRuntimeEvidenceRequirementKind::CloudEventEnvelopeObserved,
            "audit-event-envelope",
            "https://cloudevents.io/",
            "evidence/audit-chain-runtime/tenant-rbac/cloudevent-envelope.json",
            emission_plan,
        ),
        requirement(
            "trace-context-propagated",
            AuditChainRuntimeEvidenceRequirementKind::TraceContextPropagated,
            "distributed-trace",
            "https://www.w3.org/TR/trace-context/",
            "evidence/audit-chain-runtime/tenant-rbac/trace-context.json",
            emission_plan,
        ),
        requirement(
            "otel-log-record-mapped",
            AuditChainRuntimeEvidenceRequirementKind::OTelLogRecordMapped,
            "otel-log-record",
            "https://opentelemetry.io/docs/specs/otel/logs/data-model/",
            "evidence/audit-chain-runtime/tenant-rbac/otel-log-record.json",
            emission_plan,
        ),
        requirement(
            "tenant-partition-observed",
            AuditChainRuntimeEvidenceRequirementKind::TenantPartitionObserved,
            "tenant-partition",
            "https://cloudevents.io/",
            "evidence/audit-chain-runtime/tenant-rbac/tenant-partition.json",
            emission_plan,
        ),
        requirement(
            "idempotency-deduplicated",
            AuditChainRuntimeEvidenceRequirementKind::IdempotencyDeduplicated,
            "dedupe-store",
            "https://www.asyncapi.com/docs/reference/specification/v3.0.0",
            "evidence/audit-chain-runtime/tenant-rbac/idempotency-dedupe.json",
            emission_plan,
        ),
        requirement(
            "payload-digest-verified",
            AuditChainRuntimeEvidenceRequirementKind::PayloadDigestVerified,
            "payload-integrity",
            "https://www.rfc-editor.org/rfc/rfc8785",
            "evidence/audit-chain-runtime/tenant-rbac/payload-digest.json",
            emission_plan,
        ),
        requirement(
            "sensitive-payload-redacted",
            AuditChainRuntimeEvidenceRequirementKind::SensitivePayloadRedacted,
            "payload-redaction",
            "https://opentelemetry.io/docs/specs/otel/logs/data-model/",
            "evidence/audit-chain-runtime/tenant-rbac/sensitive-payload-redaction.json",
            emission_plan,
        ),
        requirement(
            "wal-append-acknowledged",
            AuditChainRuntimeEvidenceRequirementKind::WalAppendAcknowledged,
            "write-ahead-log",
            "https://cloudevents.io/",
            "evidence/audit-chain-runtime/tenant-rbac/wal-append.json",
            emission_plan,
        ),
        requirement(
            "outbox-publish-confirmed",
            AuditChainRuntimeEvidenceRequirementKind::OutboxPublishConfirmed,
            "broker-outbox",
            "https://www.asyncapi.com/docs/reference/specification/v3.0.0",
            "evidence/audit-chain-runtime/tenant-rbac/outbox-publish.json",
            emission_plan,
        ),
        requirement(
            "broker-ack-observed",
            AuditChainRuntimeEvidenceRequirementKind::BrokerAckObserved,
            "broker-delivery",
            "https://www.asyncapi.com/docs/reference/specification/v3.0.0",
            "evidence/audit-chain-runtime/tenant-rbac/broker-ack.json",
            emission_plan,
        ),
        requirement(
            "merkle-leaf-included",
            AuditChainRuntimeEvidenceRequirementKind::MerkleLeafIncluded,
            "merkle-leaf",
            "https://www.rfc-editor.org/rfc/rfc9162",
            "evidence/audit-chain-runtime/tenant-rbac/merkle-leaf.json",
            emission_plan,
        ),
        requirement(
            "merkle-root-sealed",
            AuditChainRuntimeEvidenceRequirementKind::MerkleRootSealed,
            "merkle-root",
            "https://www.rfc-editor.org/rfc/rfc9162",
            "evidence/audit-chain-runtime/tenant-rbac/merkle-root-seal.json",
            emission_plan,
        ),
        requirement(
            "sink-ingestion-observed",
            AuditChainRuntimeEvidenceRequirementKind::SinkIngestionObserved,
            "cloud-audit-sink",
            "https://opentelemetry.io/docs/specs/semconv/general/events/",
            "evidence/audit-chain-runtime/tenant-rbac/sink-ingestion.json",
            emission_plan,
        ),
        requirement(
            "replay-recovery-observed",
            AuditChainRuntimeEvidenceRequirementKind::ReplayRecoveryObserved,
            "replay-recovery",
            "https://www.asyncapi.com/docs/reference/specification/v3.0.0",
            "evidence/audit-chain-runtime/tenant-rbac/replay-recovery.json",
            emission_plan,
        ),
        requirement(
            "failure-path-audit-recorded",
            AuditChainRuntimeEvidenceRequirementKind::FailurePathAuditRecorded,
            "failure-path",
            "https://opentelemetry.io/docs/specs/semconv/general/events/",
            "evidence/audit-chain-runtime/tenant-rbac/failure-path-audit.json",
            emission_plan,
        ),
    ]
}

fn requirement(
    requirement_id: &'static str,
    requirement_kind: AuditChainRuntimeEvidenceRequirementKind,
    event_scope: &'static str,
    official_doc_url: &'static str,
    expected_evidence_ref: &'static str,
    emission_plan: &TenantRbacAuditChainEmissionPlan,
) -> AuditChainRuntimeEvidenceRequirement {
    let requires_cloudevent_envelope = matches!(
        requirement_kind,
        AuditChainRuntimeEvidenceRequirementKind::CloudEventEnvelopeObserved
            | AuditChainRuntimeEvidenceRequirementKind::TenantPartitionObserved
            | AuditChainRuntimeEvidenceRequirementKind::WalAppendAcknowledged
    );
    let requires_trace_context = matches!(
        requirement_kind,
        AuditChainRuntimeEvidenceRequirementKind::TraceContextPropagated
            | AuditChainRuntimeEvidenceRequirementKind::OTelLogRecordMapped
            | AuditChainRuntimeEvidenceRequirementKind::SinkIngestionObserved
            | AuditChainRuntimeEvidenceRequirementKind::FailurePathAuditRecorded
    );
    let requires_otel_log_record = matches!(
        requirement_kind,
        AuditChainRuntimeEvidenceRequirementKind::OTelLogRecordMapped
            | AuditChainRuntimeEvidenceRequirementKind::SinkIngestionObserved
            | AuditChainRuntimeEvidenceRequirementKind::FailurePathAuditRecorded
    );
    let requires_tenant_partition = matches!(
        requirement_kind,
        AuditChainRuntimeEvidenceRequirementKind::TenantPartitionObserved
            | AuditChainRuntimeEvidenceRequirementKind::OutboxPublishConfirmed
            | AuditChainRuntimeEvidenceRequirementKind::SinkIngestionObserved
    );
    let requires_idempotency_dedupe = matches!(
        requirement_kind,
        AuditChainRuntimeEvidenceRequirementKind::IdempotencyDeduplicated
            | AuditChainRuntimeEvidenceRequirementKind::ReplayRecoveryObserved
    );
    let requires_payload_digest = matches!(
        requirement_kind,
        AuditChainRuntimeEvidenceRequirementKind::PayloadDigestVerified
            | AuditChainRuntimeEvidenceRequirementKind::MerkleLeafIncluded
            | AuditChainRuntimeEvidenceRequirementKind::MerkleRootSealed
    );
    let requires_sensitive_payload_redaction = matches!(
        requirement_kind,
        AuditChainRuntimeEvidenceRequirementKind::SensitivePayloadRedacted
            | AuditChainRuntimeEvidenceRequirementKind::FailurePathAuditRecorded
    );
    let requires_wal_append = matches!(
        requirement_kind,
        AuditChainRuntimeEvidenceRequirementKind::WalAppendAcknowledged
            | AuditChainRuntimeEvidenceRequirementKind::ReplayRecoveryObserved
    );
    let requires_outbox_publish = matches!(
        requirement_kind,
        AuditChainRuntimeEvidenceRequirementKind::OutboxPublishConfirmed
            | AuditChainRuntimeEvidenceRequirementKind::BrokerAckObserved
    );
    let requires_broker_ack = matches!(
        requirement_kind,
        AuditChainRuntimeEvidenceRequirementKind::BrokerAckObserved
            | AuditChainRuntimeEvidenceRequirementKind::ReplayRecoveryObserved
    );
    let requires_merkle_leaf = matches!(
        requirement_kind,
        AuditChainRuntimeEvidenceRequirementKind::MerkleLeafIncluded
            | AuditChainRuntimeEvidenceRequirementKind::MerkleRootSealed
    );
    let requires_merkle_root_seal = matches!(
        requirement_kind,
        AuditChainRuntimeEvidenceRequirementKind::MerkleRootSealed
    );
    let requires_sink_ingestion = matches!(
        requirement_kind,
        AuditChainRuntimeEvidenceRequirementKind::SinkIngestionObserved
    );
    let requires_replay_recovery = matches!(
        requirement_kind,
        AuditChainRuntimeEvidenceRequirementKind::ReplayRecoveryObserved
    );
    let requires_failure_path_audit = matches!(
        requirement_kind,
        AuditChainRuntimeEvidenceRequirementKind::FailurePathAuditRecorded
    );

    AuditChainRuntimeEvidenceRequirement {
        requirement_id,
        requirement_kind,
        event_scope,
        official_doc_url,
        expected_evidence_ref,
        source_plan_ref: SOURCE_PLAN_REF,
        tenant_namespace: TENANT_NAMESPACE,
        tenant_partition: TENANT_PARTITION,
        outbox_topic: emission_plan.outbox_topic,
        requires_cloudevent_envelope,
        requires_trace_context,
        requires_otel_log_record,
        requires_tenant_partition,
        requires_idempotency_dedupe,
        requires_payload_digest,
        requires_sensitive_payload_redaction,
        requires_wal_append,
        requires_outbox_publish,
        requires_broker_ack,
        requires_merkle_leaf,
        requires_merkle_root_seal,
        requires_sink_ingestion,
        requires_replay_recovery,
        requires_failure_path_audit,
        runtime_evidence_attached: false,
        schema_version: SCHEMA_VERSION,
    }
}

fn validate_required_controls(
    plan: &TenantRbacAuditChainRuntimeEvidencePlan,
) -> Result<(), TenantRbacAuditChainRuntimeEvidenceError> {
    for control in [
        (
            plan.fd001_product_delivery_master_goal_preserved,
            "fd001_product_delivery_master_goal_preserved",
        ),
        (
            plan.oyatie_cloud_substrate_proof_required,
            "oyatie_cloud_substrate_proof_required",
        ),
        (plan.official_docs_required, "official_docs_required"),
        (
            plan.cloudevents_envelope_evidence_required,
            "cloudevents_envelope_evidence_required",
        ),
        (
            plan.trace_context_evidence_required,
            "trace_context_evidence_required",
        ),
        (
            plan.otel_log_record_mapping_required,
            "otel_log_record_mapping_required",
        ),
        (
            plan.tenant_partition_evidence_required,
            "tenant_partition_evidence_required",
        ),
        (
            plan.idempotency_dedupe_evidence_required,
            "idempotency_dedupe_evidence_required",
        ),
        (
            plan.payload_digest_match_required,
            "payload_digest_match_required",
        ),
        (
            plan.sensitive_payload_redaction_required,
            "sensitive_payload_redaction_required",
        ),
        (
            plan.wal_append_evidence_required,
            "wal_append_evidence_required",
        ),
        (
            plan.outbox_publish_evidence_required,
            "outbox_publish_evidence_required",
        ),
        (
            plan.broker_ack_evidence_required,
            "broker_ack_evidence_required",
        ),
        (
            plan.merkle_leaf_inclusion_required,
            "merkle_leaf_inclusion_required",
        ),
        (plan.merkle_root_seal_required, "merkle_root_seal_required"),
        (plan.sink_ingestion_required, "sink_ingestion_required"),
        (plan.replay_recovery_required, "replay_recovery_required"),
        (
            plan.failure_path_audit_required,
            "failure_path_audit_required",
        ),
        (plan.review_only_contract, "review_only_contract"),
    ] {
        require_control(control.0, control.1)?;
    }
    Ok(())
}

fn validate_nonclaims(
    plan: &TenantRbacAuditChainRuntimeEvidencePlan,
) -> Result<(), TenantRbacAuditChainRuntimeEvidenceError> {
    if plan.runtime_emitter_attached
        || plan.write_ahead_log_runtime_attached
        || plan.broker_publish_runtime_attached
        || plan.merkle_sealer_runtime_attached
        || plan.cloud_audit_sink_attached
        || plan.runtime_audit_chain_emission_attached
        || plan.production_audit_emission_evidence_attached
    {
        return Err(TenantRbacAuditChainRuntimeEvidenceError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn validate_runtime_requirements(
    plan: &TenantRbacAuditChainRuntimeEvidencePlan,
) -> Result<(), TenantRbacAuditChainRuntimeEvidenceError> {
    let mut seen_requirements = BTreeSet::new();
    let mut seen_kinds = BTreeSet::new();
    for requirement in &plan.requirements {
        validate_requirement(requirement)?;
        if !seen_requirements.insert(requirement.requirement_id) {
            return Err(
                TenantRbacAuditChainRuntimeEvidenceError::DuplicateRequirement(
                    requirement.requirement_id.to_owned(),
                ),
            );
        }
        seen_kinds.insert(requirement.requirement_kind);
    }
    for kind in required_requirement_kinds() {
        if !seen_kinds.contains(&kind) {
            return Err(TenantRbacAuditChainRuntimeEvidenceError::MissingRequirementKind(kind));
        }
    }
    Ok(())
}

fn validate_requirement(
    requirement: &AuditChainRuntimeEvidenceRequirement,
) -> Result<(), TenantRbacAuditChainRuntimeEvidenceError> {
    validate_slug(
        requirement.requirement_id,
        TenantRbacAuditChainRuntimeEvidenceError::InvalidRequirementId,
    )?;
    validate_event_scope(requirement.event_scope)?;
    validate_doc_url(requirement.official_doc_url)?;
    validate_prefixed_ref(
        requirement.expected_evidence_ref,
        "evidence/audit-chain-runtime/tenant-rbac/",
        TenantRbacAuditChainRuntimeEvidenceError::InvalidExpectedEvidenceRef,
    )?;
    validate_prefixed_ref(
        requirement.source_plan_ref,
        "iam/core/tenant-rbac-audit-chain-emission/",
        TenantRbacAuditChainRuntimeEvidenceError::InvalidSourcePlanRef,
    )?;
    if requirement.tenant_namespace != TENANT_NAMESPACE {
        return Err(TenantRbacAuditChainRuntimeEvidenceError::InvalidTenantNamespace);
    }
    if requirement.tenant_partition != TENANT_PARTITION {
        return Err(TenantRbacAuditChainRuntimeEvidenceError::InvalidTenantPartition);
    }
    if requirement.outbox_topic != OUTBOX_TOPIC {
        return Err(TenantRbacAuditChainRuntimeEvidenceError::InvalidOutboxTopic);
    }
    require_kind_controls(requirement)?;
    if requirement.runtime_evidence_attached {
        return Err(TenantRbacAuditChainRuntimeEvidenceError::RuntimeAttachmentOverclaim);
    }
    if requirement.schema_version != SCHEMA_VERSION {
        return Err(TenantRbacAuditChainRuntimeEvidenceError::MissingRequirements);
    }
    Ok(())
}

fn require_kind_controls(
    requirement: &AuditChainRuntimeEvidenceRequirement,
) -> Result<(), TenantRbacAuditChainRuntimeEvidenceError> {
    for control in required_controls_for_requirement(requirement) {
        require_control(control.0, control.1)?;
    }
    Ok(())
}

fn required_controls_for_requirement(
    requirement: &AuditChainRuntimeEvidenceRequirement,
) -> Vec<(bool, &'static str)> {
    let mut controls = Vec::new();
    if matches!(
        requirement.requirement_kind,
        AuditChainRuntimeEvidenceRequirementKind::CloudEventEnvelopeObserved
            | AuditChainRuntimeEvidenceRequirementKind::TenantPartitionObserved
            | AuditChainRuntimeEvidenceRequirementKind::WalAppendAcknowledged
    ) {
        controls.push((
            requirement.requires_cloudevent_envelope,
            "requirement_requires_cloudevent_envelope",
        ));
    }
    if matches!(
        requirement.requirement_kind,
        AuditChainRuntimeEvidenceRequirementKind::TraceContextPropagated
            | AuditChainRuntimeEvidenceRequirementKind::OTelLogRecordMapped
            | AuditChainRuntimeEvidenceRequirementKind::SinkIngestionObserved
            | AuditChainRuntimeEvidenceRequirementKind::FailurePathAuditRecorded
    ) {
        controls.push((
            requirement.requires_trace_context,
            "requirement_requires_trace_context",
        ));
    }
    if matches!(
        requirement.requirement_kind,
        AuditChainRuntimeEvidenceRequirementKind::OTelLogRecordMapped
            | AuditChainRuntimeEvidenceRequirementKind::SinkIngestionObserved
            | AuditChainRuntimeEvidenceRequirementKind::FailurePathAuditRecorded
    ) {
        controls.push((
            requirement.requires_otel_log_record,
            "requirement_requires_otel_log_record",
        ));
    }
    if matches!(
        requirement.requirement_kind,
        AuditChainRuntimeEvidenceRequirementKind::TenantPartitionObserved
            | AuditChainRuntimeEvidenceRequirementKind::OutboxPublishConfirmed
            | AuditChainRuntimeEvidenceRequirementKind::SinkIngestionObserved
    ) {
        controls.push((
            requirement.requires_tenant_partition,
            "requirement_requires_tenant_partition",
        ));
    }
    if matches!(
        requirement.requirement_kind,
        AuditChainRuntimeEvidenceRequirementKind::IdempotencyDeduplicated
            | AuditChainRuntimeEvidenceRequirementKind::ReplayRecoveryObserved
    ) {
        controls.push((
            requirement.requires_idempotency_dedupe,
            "requirement_requires_idempotency_dedupe",
        ));
    }
    if matches!(
        requirement.requirement_kind,
        AuditChainRuntimeEvidenceRequirementKind::PayloadDigestVerified
            | AuditChainRuntimeEvidenceRequirementKind::MerkleLeafIncluded
            | AuditChainRuntimeEvidenceRequirementKind::MerkleRootSealed
    ) {
        controls.push((
            requirement.requires_payload_digest,
            "requirement_requires_payload_digest",
        ));
    }
    if matches!(
        requirement.requirement_kind,
        AuditChainRuntimeEvidenceRequirementKind::SensitivePayloadRedacted
            | AuditChainRuntimeEvidenceRequirementKind::FailurePathAuditRecorded
    ) {
        controls.push((
            requirement.requires_sensitive_payload_redaction,
            "requirement_requires_sensitive_payload_redaction",
        ));
    }
    if matches!(
        requirement.requirement_kind,
        AuditChainRuntimeEvidenceRequirementKind::WalAppendAcknowledged
            | AuditChainRuntimeEvidenceRequirementKind::ReplayRecoveryObserved
    ) {
        controls.push((
            requirement.requires_wal_append,
            "requirement_requires_wal_append",
        ));
    }
    if matches!(
        requirement.requirement_kind,
        AuditChainRuntimeEvidenceRequirementKind::OutboxPublishConfirmed
            | AuditChainRuntimeEvidenceRequirementKind::BrokerAckObserved
    ) {
        controls.push((
            requirement.requires_outbox_publish,
            "requirement_requires_outbox_publish",
        ));
    }
    if matches!(
        requirement.requirement_kind,
        AuditChainRuntimeEvidenceRequirementKind::BrokerAckObserved
            | AuditChainRuntimeEvidenceRequirementKind::ReplayRecoveryObserved
    ) {
        controls.push((
            requirement.requires_broker_ack,
            "requirement_requires_broker_ack",
        ));
    }
    if matches!(
        requirement.requirement_kind,
        AuditChainRuntimeEvidenceRequirementKind::MerkleLeafIncluded
            | AuditChainRuntimeEvidenceRequirementKind::MerkleRootSealed
    ) {
        controls.push((
            requirement.requires_merkle_leaf,
            "requirement_requires_merkle_leaf",
        ));
    }
    if matches!(
        requirement.requirement_kind,
        AuditChainRuntimeEvidenceRequirementKind::MerkleRootSealed
    ) {
        controls.push((
            requirement.requires_merkle_root_seal,
            "requirement_requires_merkle_root_seal",
        ));
    }
    if matches!(
        requirement.requirement_kind,
        AuditChainRuntimeEvidenceRequirementKind::SinkIngestionObserved
    ) {
        controls.push((
            requirement.requires_sink_ingestion,
            "requirement_requires_sink_ingestion",
        ));
    }
    if matches!(
        requirement.requirement_kind,
        AuditChainRuntimeEvidenceRequirementKind::ReplayRecoveryObserved
    ) {
        controls.push((
            requirement.requires_replay_recovery,
            "requirement_requires_replay_recovery",
        ));
    }
    if matches!(
        requirement.requirement_kind,
        AuditChainRuntimeEvidenceRequirementKind::FailurePathAuditRecorded
    ) {
        controls.push((
            requirement.requires_failure_path_audit,
            "requirement_requires_failure_path_audit",
        ));
    }
    controls
}

fn required_requirement_kinds() -> [AuditChainRuntimeEvidenceRequirementKind; 15] {
    [
        AuditChainRuntimeEvidenceRequirementKind::CloudEventEnvelopeObserved,
        AuditChainRuntimeEvidenceRequirementKind::TraceContextPropagated,
        AuditChainRuntimeEvidenceRequirementKind::OTelLogRecordMapped,
        AuditChainRuntimeEvidenceRequirementKind::TenantPartitionObserved,
        AuditChainRuntimeEvidenceRequirementKind::IdempotencyDeduplicated,
        AuditChainRuntimeEvidenceRequirementKind::PayloadDigestVerified,
        AuditChainRuntimeEvidenceRequirementKind::SensitivePayloadRedacted,
        AuditChainRuntimeEvidenceRequirementKind::WalAppendAcknowledged,
        AuditChainRuntimeEvidenceRequirementKind::OutboxPublishConfirmed,
        AuditChainRuntimeEvidenceRequirementKind::BrokerAckObserved,
        AuditChainRuntimeEvidenceRequirementKind::MerkleLeafIncluded,
        AuditChainRuntimeEvidenceRequirementKind::MerkleRootSealed,
        AuditChainRuntimeEvidenceRequirementKind::SinkIngestionObserved,
        AuditChainRuntimeEvidenceRequirementKind::ReplayRecoveryObserved,
        AuditChainRuntimeEvidenceRequirementKind::FailurePathAuditRecorded,
    ]
}

fn validate_slug(
    value: &str,
    error: TenantRbacAuditChainRuntimeEvidenceError,
) -> Result<(), TenantRbacAuditChainRuntimeEvidenceError> {
    if value.is_empty()
        || has_unsafe_text(value)
        || has_path_traversal(value)
        || value
            .chars()
            .any(|ch| !(ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-'))
    {
        return Err(error);
    }
    Ok(())
}

fn validate_event_scope(value: &str) -> Result<(), TenantRbacAuditChainRuntimeEvidenceError> {
    if value.is_empty() || has_unsafe_text(value) || has_path_traversal(value) {
        return Err(TenantRbacAuditChainRuntimeEvidenceError::InvalidEventScope);
    }
    Ok(())
}

fn validate_doc_url(url: &str) -> Result<(), TenantRbacAuditChainRuntimeEvidenceError> {
    let allowed = [
        "https://cloudevents.io/",
        "https://www.w3.org/TR/trace-context/",
        "https://opentelemetry.io/docs/specs/otel/logs/data-model/",
        "https://opentelemetry.io/docs/specs/semconv/general/events/",
        "https://www.asyncapi.com/docs/reference/specification/v3.0.0",
        "https://www.rfc-editor.org/rfc/rfc8785",
        "https://www.rfc-editor.org/rfc/rfc9162",
    ];
    if !allowed.contains(&url) {
        return Err(TenantRbacAuditChainRuntimeEvidenceError::InvalidOfficialDocUrl);
    }
    Ok(())
}

fn validate_prefixed_ref(
    value: &str,
    prefix: &str,
    error: TenantRbacAuditChainRuntimeEvidenceError,
) -> Result<(), TenantRbacAuditChainRuntimeEvidenceError> {
    if !value.starts_with(prefix) || has_unsafe_text(value) || has_path_traversal(value) {
        return Err(error);
    }
    Ok(())
}

fn require_control(
    enabled: bool,
    field: &'static str,
) -> Result<(), TenantRbacAuditChainRuntimeEvidenceError> {
    if enabled {
        Ok(())
    } else {
        Err(TenantRbacAuditChainRuntimeEvidenceError::MissingRequiredControl(field))
    }
}

fn has_path_traversal(value: &str) -> bool {
    value.contains("..") || value.contains('~') || value.starts_with('/')
}

fn has_unsafe_text(value: &str) -> bool {
    value.contains("secret")
        || value.contains("password")
        || value.contains("credential")
        || value.contains("private-key")
        || value.contains("client_secret")
        || value.contains("bearer ")
}
