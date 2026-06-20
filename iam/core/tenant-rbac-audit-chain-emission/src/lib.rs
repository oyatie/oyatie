//! Tenant RBAC audit-chain emission contract foundation.
//!
//! This control-plane crate models the audit events Tenant RBAC runtimes
//! must emit before later cloud integration. It deliberately stays at the
//! contract/rehearsal layer: no write-ahead log, broker publish, Merkle sealer,
//! file adapter, network sink, or cloud audit-chain runtime is attached here.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use audit_emission_api::{AUDIT_EVENT_EMIT_SURFACE, AUDIT_EVENT_TOPIC};
use audit_emission_kernel::ChainCoordinate;

const PLAN_NAME: &str = "tenant-rbac-audit-chain-emission";
const SOURCE_PREFIX: &str = "https://audit.oyatie.com/tenant-rbac/";
const DATA_SCHEMA_PREFIX: &str = "https://schemas.oyatie.com/tenant-rbac/audit/";
const PACK: &str = "tenant-rbac";
const TENANT_PARTITION: &str = "tenant-scoped";
const PERIOD: &str = "calendar-month";
const SPEC_VERSION: &str = "1.0";
const DATA_CONTENT_TYPE: &str = "application/json";
const SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacAuditEventSchema {
    pub event_type: &'static str,               // data_class: PUBLIC
    pub source: &'static str,                   // data_class: PUBLIC
    pub subject_template: &'static str,         // data_class: INTERNAL_ONLY
    pub data_schema: &'static str,              // data_class: PUBLIC
    pub severity_text: &'static str,            // data_class: PUBLIC
    pub required_attributes: Vec<&'static str>, // data_class: PUBLIC
    pub required_extensions: Vec<&'static str>, // data_class: PUBLIC
    pub tenant_scoped: bool,                    // data_class: PUBLIC
    pub idempotent: bool,                       // data_class: PUBLIC
    pub payload_digest_only: bool,              // data_class: PUBLIC
    pub raw_payload_forbidden: bool,            // data_class: PUBLIC
    pub sensitive_context_forbidden: bool,      // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacAuditChainEmissionPlan {
    pub plan_name: &'static str,                        // data_class: PUBLIC
    pub producer_surface: &'static str,                 // data_class: PUBLIC
    pub outbox_topic: &'static str,                     // data_class: PUBLIC
    pub coordinate: ChainCoordinate,                    // data_class: INTERNAL_ONLY
    pub cloudevents_spec_version: &'static str,         // data_class: PUBLIC
    pub datacontenttype: &'static str,                  // data_class: PUBLIC
    pub source_prefix: &'static str,                    // data_class: PUBLIC
    pub data_schema_prefix: &'static str,               // data_class: PUBLIC
    pub required_context_attributes: Vec<&'static str>, // data_class: PUBLIC
    pub required_extension_attributes: Vec<&'static str>, // data_class: PUBLIC
    pub event_schemas: Vec<TenantRbacAuditEventSchema>, // data_class: PUBLIC
    pub cloud_events_json_required: bool,               // data_class: PUBLIC
    pub w3c_trace_context_required: bool,               // data_class: PUBLIC
    pub opentelemetry_log_mapping_required: bool,       // data_class: PUBLIC
    pub traceparent_required: bool,                     // data_class: PUBLIC
    pub tenant_partition_required: bool,                // data_class: PUBLIC
    pub idempotency_key_required: bool,                 // data_class: PUBLIC
    pub payload_digest_required: bool,                  // data_class: PUBLIC
    pub source_evidence_ref_required: bool,             // data_class: PUBLIC
    pub merkle_seal_required: bool,                     // data_class: PUBLIC
    pub write_ahead_log_required: bool,                 // data_class: PUBLIC
    pub broker_outbox_required: bool,                   // data_class: PUBLIC
    pub sensitive_context_forbidden: bool,              // data_class: PUBLIC
    pub raw_payload_storage_forbidden: bool,            // data_class: PUBLIC
    pub credential_material_forbidden: bool,            // data_class: PUBLIC
    pub runtime_emitter_attached: bool,                 // data_class: INTERNAL_ONLY
    pub write_ahead_log_runtime_attached: bool,         // data_class: INTERNAL_ONLY
    pub broker_publish_runtime_attached: bool,          // data_class: INTERNAL_ONLY
    pub merkle_sealer_runtime_attached: bool,           // data_class: INTERNAL_ONLY
    pub cloud_audit_sink_attached: bool,                // data_class: INTERNAL_ONLY
    pub runtime_audit_chain_emission_attached: bool,    // data_class: INTERNAL_ONLY
    pub schema_version: u32,                            // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantRbacAuditChainEmissionError {
    InvalidPlan,
    InvalidCoordinate,
    MissingRequiredAttribute(&'static str),
    MissingRequiredExtension(&'static str),
    InvalidEventSchema(&'static str),
    DuplicateEventType(String),
    SensitiveContextLeak(String),
    RuntimeAttachmentOverclaim,
}

pub fn tenant_rbac_audit_chain_emission_plan() -> TenantRbacAuditChainEmissionPlan {
    TenantRbacAuditChainEmissionPlan {
        plan_name: PLAN_NAME,
        producer_surface: AUDIT_EVENT_EMIT_SURFACE,
        outbox_topic: AUDIT_EVENT_TOPIC,
        coordinate: ChainCoordinate {
            pack: PACK.to_owned(),
            tenant_partition: TENANT_PARTITION.to_owned(),
            period: PERIOD.to_owned(),
        },
        cloudevents_spec_version: SPEC_VERSION,
        datacontenttype: DATA_CONTENT_TYPE,
        source_prefix: SOURCE_PREFIX,
        data_schema_prefix: DATA_SCHEMA_PREFIX,
        required_context_attributes: required_context_attributes(),
        required_extension_attributes: required_extension_attributes(),
        event_schemas: tenant_rbac_audit_event_schemas(),
        cloud_events_json_required: true,
        w3c_trace_context_required: true,
        opentelemetry_log_mapping_required: true,
        traceparent_required: true,
        tenant_partition_required: true,
        idempotency_key_required: true,
        payload_digest_required: true,
        source_evidence_ref_required: true,
        merkle_seal_required: true,
        write_ahead_log_required: true,
        broker_outbox_required: true,
        sensitive_context_forbidden: true,
        raw_payload_storage_forbidden: true,
        credential_material_forbidden: true,
        runtime_emitter_attached: false,
        write_ahead_log_runtime_attached: false,
        broker_publish_runtime_attached: false,
        merkle_sealer_runtime_attached: false,
        cloud_audit_sink_attached: false,
        runtime_audit_chain_emission_attached: false,
        schema_version: SCHEMA_VERSION,
    }
}

pub fn required_context_attributes() -> Vec<&'static str> {
    vec![
        "id",
        "source",
        "specversion",
        "type",
        "time",
        "subject",
        "datacontenttype",
        "dataschema",
    ]
}

pub fn required_extension_attributes() -> Vec<&'static str> {
    vec![
        "traceparent",
        "tenantid",
        "actorsubject",
        "idempotencykey",
        "payloaddigest",
        "evidenceref",
        "dataclass",
    ]
}

pub fn tenant_rbac_audit_event_schemas() -> Vec<TenantRbacAuditEventSchema> {
    vec![
        event_schema(
            "dev.oyatie.tenant_rbac.policy_admission_recorded.v1",
            "policy-admission",
            "policy/{tenantid}/{idempotencykey}",
            "info",
        ),
        event_schema(
            "dev.oyatie.tenant_rbac.group_close_rollup_planned.v1",
            "group-close-rollup",
            "group-close/{tenantid}/{period}",
            "info",
        ),
        event_schema(
            "dev.oyatie.tenant_rbac.cross_service_workflow_plan_queued.v1",
            "cross-service-workflow",
            "workflow/{tenantid}/{idempotencykey}",
            "info",
        ),
        event_schema(
            "dev.oyatie.tenant_rbac.incident_rollback_planned.v1",
            "incident-rollback",
            "incident/{tenantid}/{idempotencykey}",
            "warn",
        ),
        event_schema(
            "dev.oyatie.tenant_rbac.ops_command_refused.v1",
            "ops-command",
            "ops/{tenantid}/{idempotencykey}",
            "warn",
        ),
        event_schema(
            "dev.oyatie.tenant_rbac.listener_gateway_plan_reviewed.v1",
            "listener-gateway",
            "gateway/{tenantid}/{idempotencykey}",
            "info",
        ),
        event_schema(
            "dev.oyatie.tenant_rbac.identity_provider_plan_reviewed.v1",
            "identity-provider",
            "idp/{tenantid}/{idempotencykey}",
            "info",
        ),
        event_schema(
            "dev.oyatie.tenant_rbac.postgres_rls_plan_reviewed.v1",
            "postgres-rls",
            "storage/{tenantid}/{idempotencykey}",
            "info",
        ),
        event_schema(
            "dev.oyatie.tenant_rbac.cloud_readiness_gate_evaluated.v1",
            "cloud-readiness",
            "readiness/{tenantid}/{idempotencykey}",
            "info",
        ),
    ]
}

pub fn validate_tenant_rbac_audit_chain_emission_plan(
    plan: &TenantRbacAuditChainEmissionPlan,
) -> Result<(), TenantRbacAuditChainEmissionError> {
    if plan.plan_name != PLAN_NAME
        || plan.producer_surface != AUDIT_EVENT_EMIT_SURFACE
        || plan.outbox_topic != AUDIT_EVENT_TOPIC
        || plan.cloudevents_spec_version != SPEC_VERSION
        || plan.datacontenttype != DATA_CONTENT_TYPE
        || plan.source_prefix != SOURCE_PREFIX
        || plan.data_schema_prefix != DATA_SCHEMA_PREFIX
        || plan.schema_version != SCHEMA_VERSION
        || plan.event_schemas.len() < 9
        || !plan.cloud_events_json_required
        || !plan.w3c_trace_context_required
        || !plan.opentelemetry_log_mapping_required
        || !plan.traceparent_required
        || !plan.tenant_partition_required
        || !plan.idempotency_key_required
        || !plan.payload_digest_required
        || !plan.source_evidence_ref_required
        || !plan.merkle_seal_required
        || !plan.write_ahead_log_required
        || !plan.broker_outbox_required
        || !plan.sensitive_context_forbidden
        || !plan.raw_payload_storage_forbidden
        || !plan.credential_material_forbidden
    {
        return Err(TenantRbacAuditChainEmissionError::InvalidPlan);
    }
    validate_coordinate(&plan.coordinate)?;
    ensure_no_runtime_overclaim(plan)?;
    for attr in required_context_attributes() {
        if !plan.required_context_attributes.contains(&attr) {
            return Err(TenantRbacAuditChainEmissionError::MissingRequiredAttribute(
                attr,
            ));
        }
    }
    for ext in required_extension_attributes() {
        if !plan.required_extension_attributes.contains(&ext) {
            return Err(TenantRbacAuditChainEmissionError::MissingRequiredExtension(
                ext,
            ));
        }
    }

    let mut event_types = BTreeSet::new();
    for schema in &plan.event_schemas {
        validate_event_schema(schema)?;
        if !event_types.insert(schema.event_type.to_owned()) {
            return Err(TenantRbacAuditChainEmissionError::DuplicateEventType(
                schema.event_type.to_owned(),
            ));
        }
    }
    Ok(())
}

pub fn render_audit_chain_emission_checklist(
    plan: &TenantRbacAuditChainEmissionPlan,
) -> Result<String, TenantRbacAuditChainEmissionError> {
    validate_tenant_rbac_audit_chain_emission_plan(plan)?;
    let mut checklist = String::new();
    checklist.push_str("Tenant RBAC audit-chain emission contract\n");
    checklist.push_str(&format!(
        "CloudEvents specversion: {}\n",
        plan.cloudevents_spec_version
    ));
    checklist.push_str("Trace context: require W3C traceparent extension\n");
    checklist.push_str(&format!("Outbox topic: {}\n", plan.outbox_topic));
    checklist
        .push_str("Payload rule: digest-only audit payloads; raw sensitive payloads forbidden\n");
    checklist.push_str("Required event schemas:\n");
    for schema in &plan.event_schemas {
        checklist.push_str(&format!(
            "- {} ({})\n",
            schema.event_type, schema.severity_text
        ));
    }
    checklist.push_str("Nonclaims: no WAL runtime, broker publish, Merkle sealer, cloud sink, or runtime audit-chain emission attached.\n");
    Ok(checklist)
}

fn event_schema(
    event_type: &'static str,
    source_suffix: &'static str,
    subject_template: &'static str,
    severity_text: &'static str,
) -> TenantRbacAuditEventSchema {
    TenantRbacAuditEventSchema {
        event_type,
        source: source_for(source_suffix),
        subject_template,
        data_schema: schema_for(source_suffix),
        severity_text,
        required_attributes: required_context_attributes(),
        required_extensions: required_extension_attributes(),
        tenant_scoped: true,
        idempotent: true,
        payload_digest_only: true,
        raw_payload_forbidden: true,
        sensitive_context_forbidden: true,
    }
}

fn source_for(source_suffix: &str) -> &'static str {
    match source_suffix {
        "policy-admission" => "https://audit.oyatie.com/tenant-rbac/policy-admission",
        "group-close-rollup" => "https://audit.oyatie.com/tenant-rbac/group-close-rollup",
        "cross-service-workflow" => "https://audit.oyatie.com/tenant-rbac/cross-service-workflow",
        "incident-rollback" => "https://audit.oyatie.com/tenant-rbac/incident-rollback",
        "ops-command" => "https://audit.oyatie.com/tenant-rbac/ops-command",
        "listener-gateway" => "https://audit.oyatie.com/tenant-rbac/listener-gateway",
        "identity-provider" => "https://audit.oyatie.com/tenant-rbac/identity-provider",
        "postgres-rls" => "https://audit.oyatie.com/tenant-rbac/postgres-rls",
        "cloud-readiness" => "https://audit.oyatie.com/tenant-rbac/cloud-readiness",
        _ => "",
    }
}

fn schema_for(source_suffix: &str) -> &'static str {
    match source_suffix {
        "policy-admission" => {
            "https://schemas.oyatie.com/tenant-rbac/audit/policy-admission.v1.json"
        }
        "group-close-rollup" => {
            "https://schemas.oyatie.com/tenant-rbac/audit/group-close-rollup.v1.json"
        }
        "cross-service-workflow" => {
            "https://schemas.oyatie.com/tenant-rbac/audit/cross-service-workflow.v1.json"
        }
        "incident-rollback" => {
            "https://schemas.oyatie.com/tenant-rbac/audit/incident-rollback.v1.json"
        }
        "ops-command" => "https://schemas.oyatie.com/tenant-rbac/audit/ops-command.v1.json",
        "listener-gateway" => {
            "https://schemas.oyatie.com/tenant-rbac/audit/listener-gateway.v1.json"
        }
        "identity-provider" => {
            "https://schemas.oyatie.com/tenant-rbac/audit/identity-provider.v1.json"
        }
        "postgres-rls" => "https://schemas.oyatie.com/tenant-rbac/audit/postgres-rls.v1.json",
        "cloud-readiness" => "https://schemas.oyatie.com/tenant-rbac/audit/cloud-readiness.v1.json",
        _ => "",
    }
}

fn validate_coordinate(
    coordinate: &ChainCoordinate,
) -> Result<(), TenantRbacAuditChainEmissionError> {
    if coordinate.pack != PACK
        || coordinate.tenant_partition != TENANT_PARTITION
        || coordinate.period != PERIOD
        || contains_credential_material(&coordinate.pack)
        || contains_credential_material(&coordinate.tenant_partition)
        || contains_credential_material(&coordinate.period)
    {
        return Err(TenantRbacAuditChainEmissionError::InvalidCoordinate);
    }
    Ok(())
}

fn ensure_no_runtime_overclaim(
    plan: &TenantRbacAuditChainEmissionPlan,
) -> Result<(), TenantRbacAuditChainEmissionError> {
    if plan.runtime_emitter_attached
        || plan.write_ahead_log_runtime_attached
        || plan.broker_publish_runtime_attached
        || plan.merkle_sealer_runtime_attached
        || plan.cloud_audit_sink_attached
        || plan.runtime_audit_chain_emission_attached
    {
        return Err(TenantRbacAuditChainEmissionError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn validate_event_schema(
    schema: &TenantRbacAuditEventSchema,
) -> Result<(), TenantRbacAuditChainEmissionError> {
    if !schema.event_type.starts_with("dev.oyatie.tenant_rbac.")
        || !schema.event_type.ends_with(".v1")
        || !schema.source.starts_with(SOURCE_PREFIX)
        || !schema.data_schema.starts_with(DATA_SCHEMA_PREFIX)
        || schema.subject_template.is_empty()
        || !matches!(schema.severity_text, "info" | "warn" | "error")
        || !schema.tenant_scoped
        || !schema.idempotent
        || !schema.payload_digest_only
        || !schema.raw_payload_forbidden
        || !schema.sensitive_context_forbidden
    {
        return Err(TenantRbacAuditChainEmissionError::InvalidEventSchema(
            schema.event_type,
        ));
    }
    for attr in required_context_attributes() {
        if !schema.required_attributes.contains(&attr) {
            return Err(TenantRbacAuditChainEmissionError::MissingRequiredAttribute(
                attr,
            ));
        }
    }
    for ext in required_extension_attributes() {
        if !schema.required_extensions.contains(&ext) {
            return Err(TenantRbacAuditChainEmissionError::MissingRequiredExtension(
                ext,
            ));
        }
    }
    for value in [
        schema.event_type,
        schema.source,
        schema.subject_template,
        schema.data_schema,
    ] {
        if contains_credential_material(value) {
            return Err(TenantRbacAuditChainEmissionError::SensitiveContextLeak(
                value.to_owned(),
            ));
        }
    }
    Ok(())
}

fn contains_credential_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "password",
        "client_secret",
        "secret=",
        "private_key",
        "bearer ",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}
