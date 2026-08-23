use iam_tenant_rbac_audit_chain_runtime_evidence::{
    AuditChainRuntimeEvidenceRequirementKind, TenantRbacAuditChainRuntimeEvidenceError,
    audit_chain_runtime_evidence_doc_urls, tenant_rbac_audit_chain_runtime_evidence_plan,
    validate_tenant_rbac_audit_chain_runtime_evidence_plan,
};

#[test]
fn audit_chain_runtime_evidence_plan_validates_controls_and_nonclaims() {
    let plan = tenant_rbac_audit_chain_runtime_evidence_plan().expect("plan builds");
    validate_tenant_rbac_audit_chain_runtime_evidence_plan(&plan).expect("plan validates");

    assert_eq!(
        plan.plan_name,
        "tenant-rbac-audit-chain-runtime-evidence-plan"
    );
    assert_eq!(plan.service_name, "tenant-rbac");
    assert_eq!(plan.substrate_name, "oyatie-cloud");
    assert_eq!(plan.tenant_namespace, "oyatie-fd001-tenant-rbac-dev");
    assert_eq!(plan.tenant_partition, "tenant-scoped");
    assert_eq!(
        plan.audit_chain_emission_plan_name,
        "tenant-rbac-audit-chain-emission"
    );
    assert_eq!(plan.outbox_topic, "oyatie.platform.audit");
    assert_eq!(plan.event_schema_count, 9);
    assert_eq!(plan.required_context_attribute_count, 8);
    assert_eq!(plan.required_extension_attribute_count, 7);
    assert_eq!(plan.requirements.len(), 15);
    assert!(plan.fd001_product_delivery_master_goal_preserved);
    assert!(plan.oyatie_cloud_substrate_proof_required);
    assert!(plan.official_docs_required);
    assert!(plan.cloudevents_envelope_evidence_required);
    assert!(plan.trace_context_evidence_required);
    assert!(plan.otel_log_record_mapping_required);
    assert!(plan.tenant_partition_evidence_required);
    assert!(plan.idempotency_dedupe_evidence_required);
    assert!(plan.payload_digest_match_required);
    assert!(plan.sensitive_payload_redaction_required);
    assert!(plan.wal_append_evidence_required);
    assert!(plan.outbox_publish_evidence_required);
    assert!(plan.broker_ack_evidence_required);
    assert!(plan.merkle_leaf_inclusion_required);
    assert!(plan.merkle_root_seal_required);
    assert!(plan.sink_ingestion_required);
    assert!(plan.replay_recovery_required);
    assert!(plan.failure_path_audit_required);
    assert!(plan.review_only_contract);
    assert!(!plan.runtime_emitter_attached);
    assert!(!plan.write_ahead_log_runtime_attached);
    assert!(!plan.broker_publish_runtime_attached);
    assert!(!plan.merkle_sealer_runtime_attached);
    assert!(!plan.cloud_audit_sink_attached);
    assert!(!plan.runtime_audit_chain_emission_attached);
    assert!(!plan.production_audit_emission_evidence_attached);
}

#[test]
fn audit_chain_runtime_evidence_plan_covers_required_requirement_kinds_and_docs() {
    let plan = tenant_rbac_audit_chain_runtime_evidence_plan().expect("plan builds");
    let kinds = plan
        .requirements
        .iter()
        .map(|requirement| requirement.requirement_kind)
        .collect::<std::collections::BTreeSet<_>>();

    for kind in [
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
    ] {
        assert!(kinds.contains(&kind), "missing {kind:?}");
    }

    let docs = audit_chain_runtime_evidence_doc_urls(&plan);
    assert!(docs.contains(&"https://cloudevents.io/"));
    assert!(docs.contains(&"https://www.w3.org/TR/trace-context/"));
    assert!(docs.contains(&"https://opentelemetry.io/docs/specs/otel/logs/data-model/"));
    assert!(docs.contains(&"https://opentelemetry.io/docs/specs/semconv/general/events/"));
    assert!(docs.contains(&"https://www.asyncapi.com/docs/reference/specification/v3.0.0"));
    assert!(docs.contains(&"https://www.rfc-editor.org/rfc/rfc8785"));
    assert!(docs.contains(&"https://www.rfc-editor.org/rfc/rfc9162"));
}

#[test]
fn audit_chain_runtime_evidence_plan_preserves_ref_boundaries_and_source_contract() {
    let plan = tenant_rbac_audit_chain_runtime_evidence_plan().expect("plan builds");

    assert!(plan.requirements.iter().all(|requirement| {
        requirement
            .expected_evidence_ref
            .starts_with("evidence/audit-chain-runtime/tenant-rbac/")
            && requirement
                .source_plan_ref
                .starts_with("iam/core/tenant-rbac-audit-chain-emission/")
            && requirement.tenant_namespace == "oyatie-fd001-tenant-rbac-dev"
            && requirement.tenant_partition == "tenant-scoped"
            && requirement.outbox_topic == "oyatie.platform.audit"
            && !requirement.runtime_evidence_attached
    }));
    assert!(plan.requirements.iter().any(|requirement| {
        requirement.requirement_kind
            == AuditChainRuntimeEvidenceRequirementKind::OutboxPublishConfirmed
            && requirement.requires_outbox_publish
            && requirement.requires_tenant_partition
    }));
    assert!(plan.requirements.iter().any(|requirement| {
        requirement.requirement_kind == AuditChainRuntimeEvidenceRequirementKind::MerkleRootSealed
            && requirement.requires_payload_digest
            && requirement.requires_merkle_leaf
            && requirement.requires_merkle_root_seal
    }));
    assert!(plan.requirements.iter().any(|requirement| {
        requirement.requirement_kind
            == AuditChainRuntimeEvidenceRequirementKind::FailurePathAuditRecorded
            && requirement.requires_trace_context
            && requirement.requires_otel_log_record
            && requirement.requires_sensitive_payload_redaction
            && requirement.requires_failure_path_audit
    }));
}

#[test]
fn audit_chain_runtime_evidence_plan_rejects_missing_duplicate_and_doc_drift() {
    let mut plan = tenant_rbac_audit_chain_runtime_evidence_plan().expect("plan builds");
    plan.requirements.truncate(4);
    assert_eq!(
        validate_tenant_rbac_audit_chain_runtime_evidence_plan(&plan),
        Err(TenantRbacAuditChainRuntimeEvidenceError::MissingRequirements)
    );

    let mut plan = tenant_rbac_audit_chain_runtime_evidence_plan().expect("plan builds");
    plan.requirements[1].requirement_id = plan.requirements[0].requirement_id;
    assert_eq!(
        validate_tenant_rbac_audit_chain_runtime_evidence_plan(&plan),
        Err(
            TenantRbacAuditChainRuntimeEvidenceError::DuplicateRequirement(
                "cloudevent-envelope-observed".to_owned()
            )
        )
    );

    let mut plan = tenant_rbac_audit_chain_runtime_evidence_plan().expect("plan builds");
    plan.requirements[0].official_doc_url = "https://example.com/audit";
    assert_eq!(
        validate_tenant_rbac_audit_chain_runtime_evidence_plan(&plan),
        Err(TenantRbacAuditChainRuntimeEvidenceError::InvalidOfficialDocUrl)
    );
}

#[test]
fn audit_chain_runtime_evidence_plan_rejects_unsafe_refs_missing_controls_and_overclaims() {
    let mut plan = tenant_rbac_audit_chain_runtime_evidence_plan().expect("plan builds");
    plan.requirements[0].expected_evidence_ref =
        "evidence/audit-chain-runtime/tenant-rbac/client_secret";
    assert_eq!(
        validate_tenant_rbac_audit_chain_runtime_evidence_plan(&plan),
        Err(TenantRbacAuditChainRuntimeEvidenceError::InvalidExpectedEvidenceRef)
    );

    let mut plan = tenant_rbac_audit_chain_runtime_evidence_plan().expect("plan builds");
    plan.merkle_root_seal_required = false;
    assert_eq!(
        validate_tenant_rbac_audit_chain_runtime_evidence_plan(&plan),
        Err(
            TenantRbacAuditChainRuntimeEvidenceError::MissingRequiredControl(
                "merkle_root_seal_required"
            )
        )
    );

    let mut plan = tenant_rbac_audit_chain_runtime_evidence_plan().expect("plan builds");
    plan.runtime_audit_chain_emission_attached = true;
    assert_eq!(
        validate_tenant_rbac_audit_chain_runtime_evidence_plan(&plan),
        Err(TenantRbacAuditChainRuntimeEvidenceError::RuntimeAttachmentOverclaim)
    );
}
