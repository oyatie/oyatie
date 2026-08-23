//! Tenant RBAC Workflow runtime evidence contract.
//!
//! This review-only crate records the Workflow-engine, broker, durable-queue,
//! replay, trace, and audit evidence that must exist before FD-001 tenant
//! workloads can claim production Workflow execution on the future Oyatie Cloud
//! substrate. It validates official source refs, source-contract refs,
//! deterministic requirement coverage, and non-claim flags, but it does not
//! attach a Workflow engine, publish to a broker, attach a durable queue, call
//! downstream services, export runtime telemetry, emit audit-chain events, or claim
//! production Workflow evidence.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use iam_tenant_rbac_workflow_inmemory::{
    TenantRbacWorkflowQueueCapabilities, tenant_rbac_workflow_queue_capabilities,
};

const SCHEMA_VERSION: u32 = 1;
const MIN_REQUIREMENT_COUNT: usize = 14;
const PLAN_NAME: &str = "tenant-rbac-workflow-runtime-evidence-plan";
const SERVICE_NAME: &str = "tenant-rbac";
const SUBSTRATE_NAME: &str = "oyatie-cloud";
const TENANT_NAMESPACE: &str = "oyatie-fd001-tenant-rbac-dev";
const WORKFLOW_QUEUE_ADAPTER: &str = "in-memory-workflow-reference";
const SOURCE_PLAN_REF: &str = "crates/tenant-rbac-workflow-adapter-inmemory/src/lib.rs::tenant_rbac_workflow_queue_capabilities";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorkflowRuntimeEvidenceRequirementKind {
    WorkflowDefinitionVersionPinned,
    DeterministicGateSetObserved,
    DispatchIdempotencyObserved,
    ExecutionStateTransitionObserved,
    DurableQueueAckObserved,
    BrokerPublishConfirmed,
    BrokerDeliveryRetryObserved,
    DeadLetterRouteObserved,
    TenantPartitionObserved,
    PayloadDigestMatched,
    DownstreamServiceCallBoundaryObserved,
    OTelMessagingTraceCorrelated,
    WorkflowAuditEventRecorded,
    ReplayRecoveryObserved,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowRuntimeEvidenceRequirement {
    pub requirement_id: &'static str, // data_class: PUBLIC
    pub requirement_kind: WorkflowRuntimeEvidenceRequirementKind, // data_class: PUBLIC
    pub workload_scope: &'static str, // data_class: PUBLIC
    pub official_doc_url: &'static str, // data_class: PUBLIC
    pub expected_evidence_ref: &'static str, // data_class: INTERNAL_ONLY
    pub source_plan_ref: &'static str, // data_class: INTERNAL_ONLY
    pub tenant_namespace: &'static str, // data_class: INTERNAL_ONLY
    pub workflow_queue_adapter: &'static str, // data_class: INTERNAL_ONLY
    pub requires_workflow_definition_version: bool, // data_class: PUBLIC
    pub requires_deterministic_gate_evidence: bool, // data_class: PUBLIC
    pub requires_dispatch_idempotency: bool, // data_class: PUBLIC
    pub requires_execution_state_transition: bool, // data_class: PUBLIC
    pub requires_durable_queue_ack: bool, // data_class: PUBLIC
    pub requires_broker_publish_confirmation: bool, // data_class: PUBLIC
    pub requires_broker_retry_dlq: bool, // data_class: PUBLIC
    pub requires_tenant_partition: bool, // data_class: PUBLIC
    pub requires_payload_digest: bool, // data_class: PUBLIC
    pub requires_downstream_service_boundary: bool, // data_class: PUBLIC
    pub requires_otel_messaging_trace: bool, // data_class: PUBLIC
    pub requires_audit_event: bool,   // data_class: PUBLIC
    pub requires_replay_recovery: bool, // data_class: PUBLIC
    pub runtime_evidence_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,          // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacWorkflowRuntimeEvidencePlan {
    pub plan_name: &'static str,              // data_class: PUBLIC
    pub service_name: &'static str,           // data_class: PUBLIC
    pub substrate_name: &'static str,         // data_class: PUBLIC
    pub tenant_namespace: &'static str,       // data_class: INTERNAL_ONLY
    pub workflow_queue_adapter: &'static str, // data_class: PUBLIC
    pub requirements: Vec<WorkflowRuntimeEvidenceRequirement>, // data_class: INTERNAL_ONLY
    pub fd001_product_delivery_master_goal_preserved: bool, // data_class: PUBLIC
    pub oyatie_cloud_substrate_proof_required: bool, // data_class: PUBLIC
    pub official_docs_required: bool,         // data_class: PUBLIC
    pub workflow_definition_version_pin_required: bool, // data_class: PUBLIC
    pub deterministic_gate_evidence_required: bool, // data_class: PUBLIC
    pub dispatch_idempotency_required: bool,  // data_class: PUBLIC
    pub execution_state_transition_required: bool, // data_class: PUBLIC
    pub durable_queue_ack_required: bool,     // data_class: PUBLIC
    pub broker_publish_confirmation_required: bool, // data_class: PUBLIC
    pub broker_retry_or_dlq_required: bool,   // data_class: PUBLIC
    pub tenant_partition_required: bool,      // data_class: PUBLIC
    pub payload_digest_required: bool,        // data_class: PUBLIC
    pub downstream_service_boundary_required: bool, // data_class: PUBLIC
    pub otel_messaging_trace_required: bool,  // data_class: PUBLIC
    pub workflow_audit_event_required: bool,  // data_class: PUBLIC
    pub replay_recovery_required: bool,       // data_class: PUBLIC
    pub review_only_contract: bool,           // data_class: PUBLIC
    pub in_memory_execution_reference_attached: bool, // data_class: INTERNAL_ONLY
    pub workflow_engine_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub broker_publish_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub durable_queue_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub downstream_service_calls_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub cloud_workflow_runtime_attached: bool, // data_class: INTERNAL_ONLY
    pub runtime_otel_export_attached: bool,   // data_class: INTERNAL_ONLY
    pub runtime_audit_chain_emission_attached: bool, // data_class: INTERNAL_ONLY
    pub production_workflow_evidence_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                  // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantRbacWorkflowRuntimeEvidenceError {
    InvalidPlanName,
    InvalidServiceName,
    InvalidSubstrateName,
    InvalidTenantNamespace,
    InvalidWorkflowQueueAdapter,
    WorkflowQueueCapabilityDrift,
    MissingRequirements,
    DuplicateRequirement(String),
    MissingRequirementKind(WorkflowRuntimeEvidenceRequirementKind),
    InvalidRequirementId,
    InvalidWorkloadScope,
    InvalidOfficialDocUrl,
    InvalidExpectedEvidenceRef,
    InvalidSourcePlanRef,
    MissingRequiredControl(&'static str),
    RuntimeAttachmentOverclaim,
}

pub fn tenant_rbac_workflow_runtime_evidence_plan()
-> Result<TenantRbacWorkflowRuntimeEvidencePlan, TenantRbacWorkflowRuntimeEvidenceError> {
    let capabilities = tenant_rbac_workflow_queue_capabilities();
    validate_workflow_queue_capabilities(&capabilities)?;

    Ok(TenantRbacWorkflowRuntimeEvidencePlan {
        plan_name: PLAN_NAME,
        service_name: SERVICE_NAME,
        substrate_name: SUBSTRATE_NAME,
        tenant_namespace: TENANT_NAMESPACE,
        workflow_queue_adapter: WORKFLOW_QUEUE_ADAPTER,
        requirements: runtime_requirements(),
        fd001_product_delivery_master_goal_preserved: true,
        oyatie_cloud_substrate_proof_required: true,
        official_docs_required: true,
        workflow_definition_version_pin_required: true,
        deterministic_gate_evidence_required: true,
        dispatch_idempotency_required: true,
        execution_state_transition_required: true,
        durable_queue_ack_required: true,
        broker_publish_confirmation_required: true,
        broker_retry_or_dlq_required: true,
        tenant_partition_required: true,
        payload_digest_required: true,
        downstream_service_boundary_required: true,
        otel_messaging_trace_required: true,
        workflow_audit_event_required: true,
        replay_recovery_required: true,
        review_only_contract: true,
        in_memory_execution_reference_attached: capabilities.in_memory_execution_reference_attached,
        workflow_engine_runtime_attached: false,
        broker_publish_runtime_attached: false,
        durable_queue_runtime_attached: false,
        downstream_service_calls_runtime_attached: false,
        cloud_workflow_runtime_attached: false,
        runtime_otel_export_attached: false,
        runtime_audit_chain_emission_attached: false,
        production_workflow_evidence_attached: false,
        schema_version: SCHEMA_VERSION,
    })
}

pub fn validate_tenant_rbac_workflow_runtime_evidence_plan(
    plan: &TenantRbacWorkflowRuntimeEvidencePlan,
) -> Result<(), TenantRbacWorkflowRuntimeEvidenceError> {
    validate_slug(
        plan.plan_name,
        TenantRbacWorkflowRuntimeEvidenceError::InvalidPlanName,
    )?;
    if plan.service_name != SERVICE_NAME {
        return Err(TenantRbacWorkflowRuntimeEvidenceError::InvalidServiceName);
    }
    if plan.substrate_name != SUBSTRATE_NAME {
        return Err(TenantRbacWorkflowRuntimeEvidenceError::InvalidSubstrateName);
    }
    if plan.tenant_namespace != TENANT_NAMESPACE {
        return Err(TenantRbacWorkflowRuntimeEvidenceError::InvalidTenantNamespace);
    }
    if plan.workflow_queue_adapter != WORKFLOW_QUEUE_ADAPTER {
        return Err(TenantRbacWorkflowRuntimeEvidenceError::InvalidWorkflowQueueAdapter);
    }
    if plan.requirements.len() < MIN_REQUIREMENT_COUNT || plan.schema_version != SCHEMA_VERSION {
        return Err(TenantRbacWorkflowRuntimeEvidenceError::MissingRequirements);
    }
    validate_required_controls(plan)?;
    validate_nonclaims(plan)?;
    validate_runtime_requirements(plan)?;
    Ok(())
}

pub fn workflow_runtime_evidence_doc_urls(
    plan: &TenantRbacWorkflowRuntimeEvidencePlan,
) -> Vec<&'static str> {
    plan.requirements
        .iter()
        .map(|requirement| requirement.official_doc_url)
        .collect()
}

fn runtime_requirements() -> Vec<WorkflowRuntimeEvidenceRequirement> {
    vec![
        requirement(
            "workflow-definition-version-pinned",
            WorkflowRuntimeEvidenceRequirementKind::WorkflowDefinitionVersionPinned,
            "workflow-definition",
            "https://serverlessworkflow.io/",
            "evidence/workflow-runtime/tenant-rbac/workflow-definition-version.json",
        ),
        requirement(
            "deterministic-gate-set-observed",
            WorkflowRuntimeEvidenceRequirementKind::DeterministicGateSetObserved,
            "deterministic-gates",
            "https://github.com/serverlessworkflow/specification",
            "evidence/workflow-runtime/tenant-rbac/deterministic-gate-set.json",
        ),
        requirement(
            "dispatch-idempotency-observed",
            WorkflowRuntimeEvidenceRequirementKind::DispatchIdempotencyObserved,
            "workflow-dispatch",
            "https://cloudevents.io/",
            "evidence/workflow-runtime/tenant-rbac/dispatch-idempotency.json",
        ),
        requirement(
            "execution-state-transition-observed",
            WorkflowRuntimeEvidenceRequirementKind::ExecutionStateTransitionObserved,
            "workflow-execution",
            "https://serverlessworkflow.io/",
            "evidence/workflow-runtime/tenant-rbac/execution-state-transition.json",
        ),
        requirement(
            "durable-queue-ack-observed",
            WorkflowRuntimeEvidenceRequirementKind::DurableQueueAckObserved,
            "durable-queue",
            "https://www.asyncapi.com/docs/reference/specification/v3.0.0",
            "evidence/workflow-runtime/tenant-rbac/durable-queue-ack.json",
        ),
        requirement(
            "broker-publish-confirmed",
            WorkflowRuntimeEvidenceRequirementKind::BrokerPublishConfirmed,
            "broker-publish",
            "https://www.asyncapi.com/docs/reference/specification/v3.0.0",
            "evidence/workflow-runtime/tenant-rbac/broker-publish-confirmed.json",
        ),
        requirement(
            "broker-delivery-retry-observed",
            WorkflowRuntimeEvidenceRequirementKind::BrokerDeliveryRetryObserved,
            "broker-delivery",
            "https://serverlessworkflow.io/",
            "evidence/workflow-runtime/tenant-rbac/broker-delivery-retry.json",
        ),
        requirement(
            "dead-letter-route-observed",
            WorkflowRuntimeEvidenceRequirementKind::DeadLetterRouteObserved,
            "broker-delivery",
            "https://www.asyncapi.com/docs/reference/specification/v3.0.0",
            "evidence/workflow-runtime/tenant-rbac/dead-letter-route.json",
        ),
        requirement(
            "tenant-partition-observed",
            WorkflowRuntimeEvidenceRequirementKind::TenantPartitionObserved,
            "tenant-partition",
            "https://cloudevents.io/",
            "evidence/workflow-runtime/tenant-rbac/tenant-partition.json",
        ),
        requirement(
            "payload-digest-matched",
            WorkflowRuntimeEvidenceRequirementKind::PayloadDigestMatched,
            "payload-integrity",
            "https://cloudevents.io/",
            "evidence/workflow-runtime/tenant-rbac/payload-digest.json",
        ),
        requirement(
            "downstream-service-call-boundary-observed",
            WorkflowRuntimeEvidenceRequirementKind::DownstreamServiceCallBoundaryObserved,
            "downstream-service-boundary",
            "https://serverlessworkflow.io/",
            "evidence/workflow-runtime/tenant-rbac/downstream-service-boundary.json",
        ),
        requirement(
            "otel-messaging-trace-correlated",
            WorkflowRuntimeEvidenceRequirementKind::OTelMessagingTraceCorrelated,
            "messaging-trace",
            "https://opentelemetry.io/docs/specs/semconv/messaging/messaging-spans/",
            "evidence/workflow-runtime/tenant-rbac/otel-messaging-trace.json",
        ),
        requirement(
            "workflow-audit-event-recorded",
            WorkflowRuntimeEvidenceRequirementKind::WorkflowAuditEventRecorded,
            "audit-chain-contract",
            "https://cloudevents.io/",
            "evidence/workflow-runtime/tenant-rbac/workflow-audit-event.json",
        ),
        requirement(
            "replay-recovery-observed",
            WorkflowRuntimeEvidenceRequirementKind::ReplayRecoveryObserved,
            "workflow-replay",
            "https://serverlessworkflow.io/",
            "evidence/workflow-runtime/tenant-rbac/replay-recovery.json",
        ),
    ]
}

fn requirement(
    requirement_id: &'static str,
    requirement_kind: WorkflowRuntimeEvidenceRequirementKind,
    workload_scope: &'static str,
    official_doc_url: &'static str,
    expected_evidence_ref: &'static str,
) -> WorkflowRuntimeEvidenceRequirement {
    let requires_workflow_definition_version = matches!(
        requirement_kind,
        WorkflowRuntimeEvidenceRequirementKind::WorkflowDefinitionVersionPinned
            | WorkflowRuntimeEvidenceRequirementKind::ExecutionStateTransitionObserved
            | WorkflowRuntimeEvidenceRequirementKind::ReplayRecoveryObserved
    );
    let requires_deterministic_gate_evidence = matches!(
        requirement_kind,
        WorkflowRuntimeEvidenceRequirementKind::DeterministicGateSetObserved
            | WorkflowRuntimeEvidenceRequirementKind::ExecutionStateTransitionObserved
    );
    let requires_dispatch_idempotency = matches!(
        requirement_kind,
        WorkflowRuntimeEvidenceRequirementKind::DispatchIdempotencyObserved
            | WorkflowRuntimeEvidenceRequirementKind::BrokerPublishConfirmed
            | WorkflowRuntimeEvidenceRequirementKind::ReplayRecoveryObserved
    );
    let requires_execution_state_transition = matches!(
        requirement_kind,
        WorkflowRuntimeEvidenceRequirementKind::ExecutionStateTransitionObserved
            | WorkflowRuntimeEvidenceRequirementKind::ReplayRecoveryObserved
    );
    let requires_durable_queue_ack = matches!(
        requirement_kind,
        WorkflowRuntimeEvidenceRequirementKind::DurableQueueAckObserved
            | WorkflowRuntimeEvidenceRequirementKind::ReplayRecoveryObserved
    );
    let requires_broker_publish_confirmation = matches!(
        requirement_kind,
        WorkflowRuntimeEvidenceRequirementKind::BrokerPublishConfirmed
            | WorkflowRuntimeEvidenceRequirementKind::BrokerDeliveryRetryObserved
    );
    let requires_broker_retry_dlq = matches!(
        requirement_kind,
        WorkflowRuntimeEvidenceRequirementKind::BrokerDeliveryRetryObserved
            | WorkflowRuntimeEvidenceRequirementKind::DeadLetterRouteObserved
            | WorkflowRuntimeEvidenceRequirementKind::ReplayRecoveryObserved
    );
    let requires_tenant_partition = matches!(
        requirement_kind,
        WorkflowRuntimeEvidenceRequirementKind::TenantPartitionObserved
            | WorkflowRuntimeEvidenceRequirementKind::BrokerPublishConfirmed
            | WorkflowRuntimeEvidenceRequirementKind::WorkflowAuditEventRecorded
    );
    let requires_payload_digest = matches!(
        requirement_kind,
        WorkflowRuntimeEvidenceRequirementKind::PayloadDigestMatched
            | WorkflowRuntimeEvidenceRequirementKind::BrokerPublishConfirmed
            | WorkflowRuntimeEvidenceRequirementKind::WorkflowAuditEventRecorded
    );
    let requires_downstream_service_boundary = matches!(
        requirement_kind,
        WorkflowRuntimeEvidenceRequirementKind::DownstreamServiceCallBoundaryObserved
    );
    let requires_otel_messaging_trace = matches!(
        requirement_kind,
        WorkflowRuntimeEvidenceRequirementKind::OTelMessagingTraceCorrelated
            | WorkflowRuntimeEvidenceRequirementKind::BrokerPublishConfirmed
            | WorkflowRuntimeEvidenceRequirementKind::BrokerDeliveryRetryObserved
    );
    let requires_audit_event = matches!(
        requirement_kind,
        WorkflowRuntimeEvidenceRequirementKind::WorkflowAuditEventRecorded
            | WorkflowRuntimeEvidenceRequirementKind::ReplayRecoveryObserved
    );
    let requires_replay_recovery = matches!(
        requirement_kind,
        WorkflowRuntimeEvidenceRequirementKind::ReplayRecoveryObserved
    );

    WorkflowRuntimeEvidenceRequirement {
        requirement_id,
        requirement_kind,
        workload_scope,
        official_doc_url,
        expected_evidence_ref,
        source_plan_ref: SOURCE_PLAN_REF,
        tenant_namespace: TENANT_NAMESPACE,
        workflow_queue_adapter: WORKFLOW_QUEUE_ADAPTER,
        requires_workflow_definition_version,
        requires_deterministic_gate_evidence,
        requires_dispatch_idempotency,
        requires_execution_state_transition,
        requires_durable_queue_ack,
        requires_broker_publish_confirmation,
        requires_broker_retry_dlq,
        requires_tenant_partition,
        requires_payload_digest,
        requires_downstream_service_boundary,
        requires_otel_messaging_trace,
        requires_audit_event,
        requires_replay_recovery,
        runtime_evidence_attached: false,
        schema_version: SCHEMA_VERSION,
    }
}

fn validate_workflow_queue_capabilities(
    capabilities: &TenantRbacWorkflowQueueCapabilities,
) -> Result<(), TenantRbacWorkflowRuntimeEvidenceError> {
    if capabilities.adapter != WORKFLOW_QUEUE_ADAPTER
        || !capabilities.in_memory_execution_reference_attached
        || capabilities.workflow_engine_attached
        || capabilities.broker_publish_attached
        || capabilities.durable_queue_attached
        || capabilities.runtime_execution_attached
        || capabilities.downstream_service_calls_attached
        || capabilities.audit_chain_emission_attached
        || capabilities.schema_version != SCHEMA_VERSION
    {
        return Err(TenantRbacWorkflowRuntimeEvidenceError::WorkflowQueueCapabilityDrift);
    }
    Ok(())
}

fn validate_required_controls(
    plan: &TenantRbacWorkflowRuntimeEvidencePlan,
) -> Result<(), TenantRbacWorkflowRuntimeEvidenceError> {
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
            plan.workflow_definition_version_pin_required,
            "workflow_definition_version_pin_required",
        ),
        (
            plan.deterministic_gate_evidence_required,
            "deterministic_gate_evidence_required",
        ),
        (
            plan.dispatch_idempotency_required,
            "dispatch_idempotency_required",
        ),
        (
            plan.execution_state_transition_required,
            "execution_state_transition_required",
        ),
        (
            plan.durable_queue_ack_required,
            "durable_queue_ack_required",
        ),
        (
            plan.broker_publish_confirmation_required,
            "broker_publish_confirmation_required",
        ),
        (
            plan.broker_retry_or_dlq_required,
            "broker_retry_or_dlq_required",
        ),
        (plan.tenant_partition_required, "tenant_partition_required"),
        (plan.payload_digest_required, "payload_digest_required"),
        (
            plan.downstream_service_boundary_required,
            "downstream_service_boundary_required",
        ),
        (
            plan.otel_messaging_trace_required,
            "otel_messaging_trace_required",
        ),
        (
            plan.workflow_audit_event_required,
            "workflow_audit_event_required",
        ),
        (plan.replay_recovery_required, "replay_recovery_required"),
        (plan.review_only_contract, "review_only_contract"),
        (
            plan.in_memory_execution_reference_attached,
            "in_memory_execution_reference_attached",
        ),
    ] {
        require_control(control.0, control.1)?;
    }
    Ok(())
}

fn validate_nonclaims(
    plan: &TenantRbacWorkflowRuntimeEvidencePlan,
) -> Result<(), TenantRbacWorkflowRuntimeEvidenceError> {
    if plan.workflow_engine_runtime_attached
        || plan.broker_publish_runtime_attached
        || plan.durable_queue_runtime_attached
        || plan.downstream_service_calls_runtime_attached
        || plan.cloud_workflow_runtime_attached
        || plan.runtime_otel_export_attached
        || plan.runtime_audit_chain_emission_attached
        || plan.production_workflow_evidence_attached
    {
        return Err(TenantRbacWorkflowRuntimeEvidenceError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn validate_runtime_requirements(
    plan: &TenantRbacWorkflowRuntimeEvidencePlan,
) -> Result<(), TenantRbacWorkflowRuntimeEvidenceError> {
    let mut seen_requirements = BTreeSet::new();
    let mut seen_kinds = BTreeSet::new();
    for requirement in &plan.requirements {
        validate_requirement(requirement)?;
        if !seen_requirements.insert(requirement.requirement_id) {
            return Err(
                TenantRbacWorkflowRuntimeEvidenceError::DuplicateRequirement(
                    requirement.requirement_id.to_owned(),
                ),
            );
        }
        seen_kinds.insert(requirement.requirement_kind);
    }
    for kind in required_requirement_kinds() {
        if !seen_kinds.contains(&kind) {
            return Err(TenantRbacWorkflowRuntimeEvidenceError::MissingRequirementKind(kind));
        }
    }
    Ok(())
}

fn validate_requirement(
    requirement: &WorkflowRuntimeEvidenceRequirement,
) -> Result<(), TenantRbacWorkflowRuntimeEvidenceError> {
    validate_slug(
        requirement.requirement_id,
        TenantRbacWorkflowRuntimeEvidenceError::InvalidRequirementId,
    )?;
    validate_workload_scope(requirement.workload_scope)?;
    validate_doc_url(requirement.official_doc_url)?;
    validate_prefixed_ref(
        requirement.expected_evidence_ref,
        "evidence/workflow-runtime/tenant-rbac/",
        TenantRbacWorkflowRuntimeEvidenceError::InvalidExpectedEvidenceRef,
    )?;
    validate_prefixed_ref(
        requirement.source_plan_ref,
        "crates/tenant-rbac-workflow-adapter-inmemory/",
        TenantRbacWorkflowRuntimeEvidenceError::InvalidSourcePlanRef,
    )?;
    if requirement.tenant_namespace != TENANT_NAMESPACE {
        return Err(TenantRbacWorkflowRuntimeEvidenceError::InvalidTenantNamespace);
    }
    if requirement.workflow_queue_adapter != WORKFLOW_QUEUE_ADAPTER {
        return Err(TenantRbacWorkflowRuntimeEvidenceError::InvalidWorkflowQueueAdapter);
    }
    if matches!(
        requirement.requirement_kind,
        WorkflowRuntimeEvidenceRequirementKind::WorkflowDefinitionVersionPinned
            | WorkflowRuntimeEvidenceRequirementKind::ExecutionStateTransitionObserved
            | WorkflowRuntimeEvidenceRequirementKind::ReplayRecoveryObserved
    ) {
        require_control(
            requirement.requires_workflow_definition_version,
            "requirement_requires_workflow_definition_version",
        )?;
    }
    if matches!(
        requirement.requirement_kind,
        WorkflowRuntimeEvidenceRequirementKind::DeterministicGateSetObserved
            | WorkflowRuntimeEvidenceRequirementKind::ExecutionStateTransitionObserved
    ) {
        require_control(
            requirement.requires_deterministic_gate_evidence,
            "requirement_requires_deterministic_gate_evidence",
        )?;
    }
    if matches!(
        requirement.requirement_kind,
        WorkflowRuntimeEvidenceRequirementKind::DispatchIdempotencyObserved
            | WorkflowRuntimeEvidenceRequirementKind::BrokerPublishConfirmed
            | WorkflowRuntimeEvidenceRequirementKind::ReplayRecoveryObserved
    ) {
        require_control(
            requirement.requires_dispatch_idempotency,
            "requirement_requires_dispatch_idempotency",
        )?;
    }
    if matches!(
        requirement.requirement_kind,
        WorkflowRuntimeEvidenceRequirementKind::ExecutionStateTransitionObserved
            | WorkflowRuntimeEvidenceRequirementKind::ReplayRecoveryObserved
    ) {
        require_control(
            requirement.requires_execution_state_transition,
            "requirement_requires_execution_state_transition",
        )?;
    }
    if matches!(
        requirement.requirement_kind,
        WorkflowRuntimeEvidenceRequirementKind::DurableQueueAckObserved
            | WorkflowRuntimeEvidenceRequirementKind::ReplayRecoveryObserved
    ) {
        require_control(
            requirement.requires_durable_queue_ack,
            "requirement_requires_durable_queue_ack",
        )?;
    }
    if matches!(
        requirement.requirement_kind,
        WorkflowRuntimeEvidenceRequirementKind::BrokerPublishConfirmed
            | WorkflowRuntimeEvidenceRequirementKind::BrokerDeliveryRetryObserved
    ) {
        require_control(
            requirement.requires_broker_publish_confirmation,
            "requirement_requires_broker_publish_confirmation",
        )?;
    }
    if matches!(
        requirement.requirement_kind,
        WorkflowRuntimeEvidenceRequirementKind::BrokerDeliveryRetryObserved
            | WorkflowRuntimeEvidenceRequirementKind::DeadLetterRouteObserved
            | WorkflowRuntimeEvidenceRequirementKind::ReplayRecoveryObserved
    ) {
        require_control(
            requirement.requires_broker_retry_dlq,
            "requirement_requires_broker_retry_dlq",
        )?;
    }
    if matches!(
        requirement.requirement_kind,
        WorkflowRuntimeEvidenceRequirementKind::TenantPartitionObserved
            | WorkflowRuntimeEvidenceRequirementKind::BrokerPublishConfirmed
            | WorkflowRuntimeEvidenceRequirementKind::WorkflowAuditEventRecorded
    ) {
        require_control(
            requirement.requires_tenant_partition,
            "requirement_requires_tenant_partition",
        )?;
    }
    if matches!(
        requirement.requirement_kind,
        WorkflowRuntimeEvidenceRequirementKind::PayloadDigestMatched
            | WorkflowRuntimeEvidenceRequirementKind::BrokerPublishConfirmed
            | WorkflowRuntimeEvidenceRequirementKind::WorkflowAuditEventRecorded
    ) {
        require_control(
            requirement.requires_payload_digest,
            "requirement_requires_payload_digest",
        )?;
    }
    if matches!(
        requirement.requirement_kind,
        WorkflowRuntimeEvidenceRequirementKind::DownstreamServiceCallBoundaryObserved
    ) {
        require_control(
            requirement.requires_downstream_service_boundary,
            "requirement_requires_downstream_service_boundary",
        )?;
    }
    if matches!(
        requirement.requirement_kind,
        WorkflowRuntimeEvidenceRequirementKind::OTelMessagingTraceCorrelated
            | WorkflowRuntimeEvidenceRequirementKind::BrokerPublishConfirmed
            | WorkflowRuntimeEvidenceRequirementKind::BrokerDeliveryRetryObserved
    ) {
        require_control(
            requirement.requires_otel_messaging_trace,
            "requirement_requires_otel_messaging_trace",
        )?;
    }
    if matches!(
        requirement.requirement_kind,
        WorkflowRuntimeEvidenceRequirementKind::WorkflowAuditEventRecorded
            | WorkflowRuntimeEvidenceRequirementKind::ReplayRecoveryObserved
    ) {
        require_control(
            requirement.requires_audit_event,
            "requirement_requires_audit_event",
        )?;
    }
    if matches!(
        requirement.requirement_kind,
        WorkflowRuntimeEvidenceRequirementKind::ReplayRecoveryObserved
    ) {
        require_control(
            requirement.requires_replay_recovery,
            "requirement_requires_replay_recovery",
        )?;
    }
    if requirement.runtime_evidence_attached {
        return Err(TenantRbacWorkflowRuntimeEvidenceError::RuntimeAttachmentOverclaim);
    }
    if requirement.schema_version != SCHEMA_VERSION {
        return Err(TenantRbacWorkflowRuntimeEvidenceError::MissingRequirements);
    }
    Ok(())
}

fn required_requirement_kinds() -> [WorkflowRuntimeEvidenceRequirementKind; 14] {
    [
        WorkflowRuntimeEvidenceRequirementKind::WorkflowDefinitionVersionPinned,
        WorkflowRuntimeEvidenceRequirementKind::DeterministicGateSetObserved,
        WorkflowRuntimeEvidenceRequirementKind::DispatchIdempotencyObserved,
        WorkflowRuntimeEvidenceRequirementKind::ExecutionStateTransitionObserved,
        WorkflowRuntimeEvidenceRequirementKind::DurableQueueAckObserved,
        WorkflowRuntimeEvidenceRequirementKind::BrokerPublishConfirmed,
        WorkflowRuntimeEvidenceRequirementKind::BrokerDeliveryRetryObserved,
        WorkflowRuntimeEvidenceRequirementKind::DeadLetterRouteObserved,
        WorkflowRuntimeEvidenceRequirementKind::TenantPartitionObserved,
        WorkflowRuntimeEvidenceRequirementKind::PayloadDigestMatched,
        WorkflowRuntimeEvidenceRequirementKind::DownstreamServiceCallBoundaryObserved,
        WorkflowRuntimeEvidenceRequirementKind::OTelMessagingTraceCorrelated,
        WorkflowRuntimeEvidenceRequirementKind::WorkflowAuditEventRecorded,
        WorkflowRuntimeEvidenceRequirementKind::ReplayRecoveryObserved,
    ]
}

fn validate_slug(
    value: &str,
    error: TenantRbacWorkflowRuntimeEvidenceError,
) -> Result<(), TenantRbacWorkflowRuntimeEvidenceError> {
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

fn validate_workload_scope(value: &str) -> Result<(), TenantRbacWorkflowRuntimeEvidenceError> {
    if value.is_empty() || has_unsafe_text(value) || has_path_traversal(value) {
        return Err(TenantRbacWorkflowRuntimeEvidenceError::InvalidWorkloadScope);
    }
    Ok(())
}

fn validate_doc_url(url: &str) -> Result<(), TenantRbacWorkflowRuntimeEvidenceError> {
    let allowed = [
        "https://serverlessworkflow.io/",
        "https://github.com/serverlessworkflow/specification",
        "https://www.asyncapi.com/docs/reference/specification/v3.0.0",
        "https://cloudevents.io/",
        "https://opentelemetry.io/docs/specs/semconv/messaging/messaging-spans/",
    ];
    if !allowed.contains(&url) {
        return Err(TenantRbacWorkflowRuntimeEvidenceError::InvalidOfficialDocUrl);
    }
    Ok(())
}

fn validate_prefixed_ref(
    value: &str,
    prefix: &str,
    error: TenantRbacWorkflowRuntimeEvidenceError,
) -> Result<(), TenantRbacWorkflowRuntimeEvidenceError> {
    if !value.starts_with(prefix) || has_unsafe_text(value) || has_path_traversal(value) {
        return Err(error);
    }
    Ok(())
}

fn require_control(
    enabled: bool,
    field: &'static str,
) -> Result<(), TenantRbacWorkflowRuntimeEvidenceError> {
    if enabled {
        Ok(())
    } else {
        Err(TenantRbacWorkflowRuntimeEvidenceError::MissingRequiredControl(field))
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
}
