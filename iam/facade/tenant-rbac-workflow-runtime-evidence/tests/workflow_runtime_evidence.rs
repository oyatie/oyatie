use iam_tenant_rbac_workflow_runtime_evidence::{
    TenantRbacWorkflowRuntimeEvidenceError, WorkflowRuntimeEvidenceRequirementKind,
    tenant_rbac_workflow_runtime_evidence_plan,
    validate_tenant_rbac_workflow_runtime_evidence_plan, workflow_runtime_evidence_doc_urls,
};

#[test]
fn workflow_runtime_evidence_plan_validates_controls_and_nonclaims() {
    let plan = tenant_rbac_workflow_runtime_evidence_plan().expect("plan builds");
    validate_tenant_rbac_workflow_runtime_evidence_plan(&plan).expect("plan validates");

    assert_eq!(plan.plan_name, "tenant-rbac-workflow-runtime-evidence-plan");
    assert_eq!(plan.service_name, "tenant-rbac");
    assert_eq!(plan.substrate_name, "oyatie-cloud");
    assert_eq!(plan.tenant_namespace, "oyatie-fd001-tenant-rbac-dev");
    assert_eq!(plan.workflow_queue_adapter, "in-memory-workflow-reference");
    assert_eq!(plan.requirements.len(), 14);
    assert!(plan.fd001_product_delivery_master_goal_preserved);
    assert!(plan.oyatie_cloud_substrate_proof_required);
    assert!(plan.official_docs_required);
    assert!(plan.workflow_definition_version_pin_required);
    assert!(plan.deterministic_gate_evidence_required);
    assert!(plan.dispatch_idempotency_required);
    assert!(plan.execution_state_transition_required);
    assert!(plan.durable_queue_ack_required);
    assert!(plan.broker_publish_confirmation_required);
    assert!(plan.broker_retry_or_dlq_required);
    assert!(plan.tenant_partition_required);
    assert!(plan.payload_digest_required);
    assert!(plan.downstream_service_boundary_required);
    assert!(plan.otel_messaging_trace_required);
    assert!(plan.workflow_audit_event_required);
    assert!(plan.replay_recovery_required);
    assert!(plan.review_only_contract);
    assert!(plan.in_memory_execution_reference_attached);
    assert!(!plan.workflow_engine_runtime_attached);
    assert!(!plan.broker_publish_runtime_attached);
    assert!(!plan.durable_queue_runtime_attached);
    assert!(!plan.downstream_service_calls_runtime_attached);
    assert!(!plan.cloud_workflow_runtime_attached);
    assert!(!plan.runtime_otel_export_attached);
    assert!(!plan.runtime_audit_chain_emission_attached);
    assert!(!plan.production_workflow_evidence_attached);
}

#[test]
fn workflow_runtime_evidence_plan_covers_required_requirement_kinds_and_docs() {
    let plan = tenant_rbac_workflow_runtime_evidence_plan().expect("plan builds");
    let kinds = plan
        .requirements
        .iter()
        .map(|requirement| requirement.requirement_kind)
        .collect::<std::collections::BTreeSet<_>>();

    for kind in [
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
    ] {
        assert!(kinds.contains(&kind), "missing {kind:?}");
    }

    let docs = workflow_runtime_evidence_doc_urls(&plan);
    assert!(docs.contains(&"https://serverlessworkflow.io/"));
    assert!(docs.contains(&"https://github.com/serverlessworkflow/specification"));
    assert!(docs.contains(&"https://www.asyncapi.com/docs/reference/specification/v3.0.0"));
    assert!(docs.contains(&"https://cloudevents.io/"));
    assert!(
        docs.contains(&"https://opentelemetry.io/docs/specs/semconv/messaging/messaging-spans/")
    );
}

#[test]
fn workflow_runtime_evidence_plan_preserves_ref_boundaries_and_inmemory_source_contract() {
    let plan = tenant_rbac_workflow_runtime_evidence_plan().expect("plan builds");

    assert!(plan.requirements.iter().all(|requirement| {
        requirement
            .expected_evidence_ref
            .starts_with("evidence/workflow-runtime/tenant-rbac/")
            && requirement
                .source_plan_ref
                .starts_with("crates/tenant-rbac-workflow-adapter-inmemory/")
            && requirement.tenant_namespace == "oyatie-fd001-tenant-rbac-dev"
            && requirement.workflow_queue_adapter == "in-memory-workflow-reference"
            && !requirement.runtime_evidence_attached
    }));
    assert!(plan.requirements.iter().any(|requirement| {
        requirement.requirement_kind
            == WorkflowRuntimeEvidenceRequirementKind::BrokerPublishConfirmed
            && requirement.requires_broker_publish_confirmation
            && requirement.requires_dispatch_idempotency
            && requirement.requires_otel_messaging_trace
    }));
    assert!(plan.requirements.iter().any(|requirement| {
        requirement.requirement_kind
            == WorkflowRuntimeEvidenceRequirementKind::ReplayRecoveryObserved
            && requirement.requires_replay_recovery
            && requirement.requires_execution_state_transition
            && requirement.requires_durable_queue_ack
    }));
    assert!(plan.requirements.iter().any(|requirement| {
        requirement.requirement_kind
            == WorkflowRuntimeEvidenceRequirementKind::WorkflowAuditEventRecorded
            && requirement.requires_audit_event
            && requirement.requires_tenant_partition
            && requirement.requires_payload_digest
    }));
}

#[test]
fn workflow_runtime_evidence_plan_rejects_missing_duplicate_and_doc_drift() {
    let mut plan = tenant_rbac_workflow_runtime_evidence_plan().expect("plan builds");
    plan.requirements.truncate(3);
    assert_eq!(
        validate_tenant_rbac_workflow_runtime_evidence_plan(&plan),
        Err(TenantRbacWorkflowRuntimeEvidenceError::MissingRequirements)
    );

    let mut plan = tenant_rbac_workflow_runtime_evidence_plan().expect("plan builds");
    plan.requirements[1].requirement_id = plan.requirements[0].requirement_id;
    assert_eq!(
        validate_tenant_rbac_workflow_runtime_evidence_plan(&plan),
        Err(
            TenantRbacWorkflowRuntimeEvidenceError::DuplicateRequirement(
                "workflow-definition-version-pinned".to_owned()
            )
        )
    );

    let mut plan = tenant_rbac_workflow_runtime_evidence_plan().expect("plan builds");
    plan.requirements[0].official_doc_url = "https://example.com/workflow";
    assert_eq!(
        validate_tenant_rbac_workflow_runtime_evidence_plan(&plan),
        Err(TenantRbacWorkflowRuntimeEvidenceError::InvalidOfficialDocUrl)
    );
}

#[test]
fn workflow_runtime_evidence_plan_rejects_unsafe_refs_missing_controls_and_overclaims() {
    let mut plan = tenant_rbac_workflow_runtime_evidence_plan().expect("plan builds");
    plan.requirements[0].expected_evidence_ref =
        "evidence/workflow-runtime/tenant-rbac/password-material";
    assert_eq!(
        validate_tenant_rbac_workflow_runtime_evidence_plan(&plan),
        Err(TenantRbacWorkflowRuntimeEvidenceError::InvalidExpectedEvidenceRef)
    );

    let mut plan = tenant_rbac_workflow_runtime_evidence_plan().expect("plan builds");
    plan.broker_publish_confirmation_required = false;
    assert_eq!(
        validate_tenant_rbac_workflow_runtime_evidence_plan(&plan),
        Err(
            TenantRbacWorkflowRuntimeEvidenceError::MissingRequiredControl(
                "broker_publish_confirmation_required"
            )
        )
    );

    let mut plan = tenant_rbac_workflow_runtime_evidence_plan().expect("plan builds");
    plan.workflow_engine_runtime_attached = true;
    assert_eq!(
        validate_tenant_rbac_workflow_runtime_evidence_plan(&plan),
        Err(TenantRbacWorkflowRuntimeEvidenceError::RuntimeAttachmentOverclaim)
    );
}
