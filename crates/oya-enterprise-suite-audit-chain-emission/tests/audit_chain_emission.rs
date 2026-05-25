use oya_enterprise_suite_audit_chain_emission::{
    EnterpriseAuditChainEmissionError, enterprise_suite_audit_chain_emission_plan,
    render_audit_chain_emission_checklist, validate_enterprise_suite_audit_chain_emission_plan,
};

#[test]
fn enterprise_audit_chain_emission_plan_covers_cloudevents_trace_context_and_digest_only_payloads()
{
    let plan = enterprise_suite_audit_chain_emission_plan();
    validate_enterprise_suite_audit_chain_emission_plan(&plan).expect("plan validates");

    assert_eq!(plan.cloudevents_spec_version, "1.0");
    assert_eq!(plan.datacontenttype, "application/json");
    assert_eq!(plan.outbox_topic, "oya.platform.audit");
    assert_eq!(plan.coordinate.pack, "enterprise-suite");
    assert_eq!(plan.coordinate.tenant_partition, "tenant-scoped");
    assert!(plan.cloud_events_json_required);
    assert!(plan.w3c_trace_context_required);
    assert!(plan.opentelemetry_log_mapping_required);
    assert!(plan.traceparent_required);
    assert!(plan.tenant_partition_required);
    assert!(plan.idempotency_key_required);
    assert!(plan.payload_digest_required);
    assert!(plan.source_evidence_ref_required);
    assert!(plan.merkle_seal_required);
    assert!(plan.write_ahead_log_required);
    assert!(plan.broker_outbox_required);
    assert!(plan.sensitive_context_forbidden);
    assert!(plan.raw_payload_storage_forbidden);
    assert!(plan.credential_material_forbidden);
    assert!(!plan.runtime_emitter_attached);
    assert!(!plan.write_ahead_log_runtime_attached);
    assert!(!plan.broker_publish_runtime_attached);
    assert!(!plan.merkle_sealer_runtime_attached);
    assert!(!plan.cloud_audit_sink_attached);
    assert!(!plan.runtime_audit_chain_emission_attached);
    assert_eq!(plan.event_schemas.len(), 9);

    for schema in &plan.event_schemas {
        assert!(
            schema
                .event_type
                .starts_with("dev.oyatie.enterprise_suite.")
        );
        assert!(
            schema
                .source
                .starts_with("https://audit.oyatie.dev/enterprise-suite/")
        );
        assert!(
            schema
                .data_schema
                .starts_with("https://schemas.oyatie.dev/enterprise-suite/audit/")
        );
        assert!(schema.required_attributes.contains(&"id"));
        assert!(schema.required_attributes.contains(&"source"));
        assert!(schema.required_attributes.contains(&"specversion"));
        assert!(schema.required_attributes.contains(&"type"));
        assert!(schema.required_extensions.contains(&"traceparent"));
        assert!(schema.required_extensions.contains(&"tenantid"));
        assert!(schema.required_extensions.contains(&"payloaddigest"));
        assert!(schema.tenant_scoped);
        assert!(schema.idempotent);
        assert!(schema.payload_digest_only);
        assert!(schema.raw_payload_forbidden);
        assert!(schema.sensitive_context_forbidden);
    }
}

#[test]
fn enterprise_audit_chain_emission_plan_renders_review_only_checklist() {
    let plan = enterprise_suite_audit_chain_emission_plan();
    let checklist = render_audit_chain_emission_checklist(&plan).expect("checklist renders");

    assert!(checklist.contains("CloudEvents specversion: 1.0"));
    assert!(checklist.contains("W3C traceparent"));
    assert!(checklist.contains("Outbox topic: oya.platform.audit"));
    assert!(checklist.contains("digest-only audit payloads"));
    assert!(checklist.contains("dev.oyatie.enterprise_suite.policy_admission_recorded.v1"));
    assert!(checklist.contains("dev.oyatie.enterprise_suite.cloud_readiness_gate_evaluated.v1"));
    assert!(checklist.contains("Nonclaims: no WAL runtime"));
    assert!(!checklist.to_ascii_lowercase().contains("client_secret"));
}

#[test]
fn enterprise_audit_chain_emission_plan_rejects_missing_required_metadata() {
    let mut plan = enterprise_suite_audit_chain_emission_plan();
    plan.required_context_attributes
        .retain(|attr| *attr != "source");
    assert_eq!(
        validate_enterprise_suite_audit_chain_emission_plan(&plan),
        Err(EnterpriseAuditChainEmissionError::MissingRequiredAttribute(
            "source"
        ))
    );

    let mut plan = enterprise_suite_audit_chain_emission_plan();
    plan.required_extension_attributes
        .retain(|attr| *attr != "traceparent");
    assert_eq!(
        validate_enterprise_suite_audit_chain_emission_plan(&plan),
        Err(EnterpriseAuditChainEmissionError::MissingRequiredExtension(
            "traceparent"
        ))
    );
}

#[test]
fn enterprise_audit_chain_emission_plan_rejects_duplicate_events_and_runtime_overclaims() {
    let mut plan = enterprise_suite_audit_chain_emission_plan();
    let duplicate = plan.event_schemas[0].clone();
    plan.event_schemas.push(duplicate);
    assert!(matches!(
        validate_enterprise_suite_audit_chain_emission_plan(&plan),
        Err(EnterpriseAuditChainEmissionError::DuplicateEventType(_))
    ));

    let mut plan = enterprise_suite_audit_chain_emission_plan();
    plan.runtime_audit_chain_emission_attached = true;
    assert_eq!(
        validate_enterprise_suite_audit_chain_emission_plan(&plan),
        Err(EnterpriseAuditChainEmissionError::RuntimeAttachmentOverclaim)
    );
}

#[test]
fn enterprise_audit_chain_emission_plan_rejects_bad_coordinate_or_sensitive_context() {
    let mut plan = enterprise_suite_audit_chain_emission_plan();
    plan.coordinate.pack = "enterprise-suite?secret=bad".to_owned();
    assert_eq!(
        validate_enterprise_suite_audit_chain_emission_plan(&plan),
        Err(EnterpriseAuditChainEmissionError::InvalidCoordinate)
    );

    let mut plan = enterprise_suite_audit_chain_emission_plan();
    plan.event_schemas[0].source = "https://audit.oyatie.dev/enterprise-suite/client_secret";
    assert!(matches!(
        validate_enterprise_suite_audit_chain_emission_plan(&plan),
        Err(EnterpriseAuditChainEmissionError::SensitiveContextLeak(_))
    ));
}
