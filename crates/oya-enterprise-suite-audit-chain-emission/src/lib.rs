//! Enterprise Suite audit-chain emission contract foundation.
//!
//! This control-plane crate models the audit events Enterprise Suite runtimes
//! must emit before later cloud integration. It deliberately stays at the
//! contract/rehearsal layer: no write-ahead log, broker publish, Merkle sealer,
//! file adapter, network sink, or cloud audit-chain runtime is attached here.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use oya_audit_chain_emission_api::{AUDIT_EVENT_EMIT_SURFACE, AUDIT_EVENT_TOPIC};
use oya_audit_chain_emission_kernel::ChainCoordinate;

const PLAN_NAME: &str = "enterprise-suite-audit-chain-emission";
const SOURCE_PREFIX: &str = "https://audit.oyatie.dev/enterprise-suite/";
const DATA_SCHEMA_PREFIX: &str = "https://schemas.oyatie.dev/enterprise-suite/audit/";
const PACK: &str = "enterprise-suite";
const TENANT_PARTITION: &str = "tenant-scoped";
const PERIOD: &str = "calendar-month";
const SPEC_VERSION: &str = "1.0";
const DATA_CONTENT_TYPE: &str = "application/json";
const SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnterpriseAuditEventSchema {
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
pub struct EnterpriseAuditChainEmissionPlan {
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
    pub event_schemas: Vec<EnterpriseAuditEventSchema>, // data_class: PUBLIC
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
pub enum EnterpriseAuditChainEmissionError {
    InvalidPlan,
    InvalidCoordinate,
    MissingRequiredAttribute(&'static str),
    MissingRequiredExtension(&'static str),
    InvalidEventSchema(&'static str),
    DuplicateEventType(String),
    SensitiveContextLeak(String),
    RuntimeAttachmentOverclaim,
}

pub fn enterprise_suite_audit_chain_emission_plan() -> EnterpriseAuditChainEmissionPlan {
    EnterpriseAuditChainEmissionPlan {
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
        event_schemas: enterprise_audit_event_schemas(),
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

pub fn enterprise_audit_event_schemas() -> Vec<EnterpriseAuditEventSchema> {
    vec![
        event_schema(
            "dev.oyatie.enterprise_suite.policy_admission_recorded.v1",
            "policy-admission",
            "policy/{tenantid}/{idempotencykey}",
            "info",
        ),
        event_schema(
            "dev.oyatie.enterprise_suite.group_close_rollup_planned.v1",
            "group-close-rollup",
            "group-close/{tenantid}/{period}",
            "info",
        ),
        event_schema(
            "dev.oyatie.enterprise_suite.cross_product_workflow_plan_queued.v1",
            "cross-product-workflow",
            "workflow/{tenantid}/{idempotencykey}",
            "info",
        ),
        event_schema(
            "dev.oyatie.enterprise_suite.incident_rollback_planned.v1",
            "incident-rollback",
            "incident/{tenantid}/{idempotencykey}",
            "warn",
        ),
        event_schema(
            "dev.oyatie.enterprise_suite.ops_command_refused.v1",
            "ops-command",
            "ops/{tenantid}/{idempotencykey}",
            "warn",
        ),
        event_schema(
            "dev.oyatie.enterprise_suite.listener_gateway_plan_reviewed.v1",
            "listener-gateway",
            "gateway/{tenantid}/{idempotencykey}",
            "info",
        ),
        event_schema(
            "dev.oyatie.enterprise_suite.identity_provider_plan_reviewed.v1",
            "identity-provider",
            "idp/{tenantid}/{idempotencykey}",
            "info",
        ),
        event_schema(
            "dev.oyatie.enterprise_suite.postgres_rls_plan_reviewed.v1",
            "postgres-rls",
            "storage/{tenantid}/{idempotencykey}",
            "info",
        ),
        event_schema(
            "dev.oyatie.enterprise_suite.cloud_readiness_gate_evaluated.v1",
            "cloud-readiness",
            "readiness/{tenantid}/{idempotencykey}",
            "info",
        ),
    ]
}

pub fn validate_enterprise_suite_audit_chain_emission_plan(
    plan: &EnterpriseAuditChainEmissionPlan,
) -> Result<(), EnterpriseAuditChainEmissionError> {
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
        return Err(EnterpriseAuditChainEmissionError::InvalidPlan);
    }
    validate_coordinate(&plan.coordinate)?;
    ensure_no_runtime_overclaim(plan)?;
    for attr in required_context_attributes() {
        if !plan.required_context_attributes.contains(&attr) {
            return Err(EnterpriseAuditChainEmissionError::MissingRequiredAttribute(
                attr,
            ));
        }
    }
    for ext in required_extension_attributes() {
        if !plan.required_extension_attributes.contains(&ext) {
            return Err(EnterpriseAuditChainEmissionError::MissingRequiredExtension(
                ext,
            ));
        }
    }

    let mut event_types = BTreeSet::new();
    for schema in &plan.event_schemas {
        validate_event_schema(schema)?;
        if !event_types.insert(schema.event_type.to_owned()) {
            return Err(EnterpriseAuditChainEmissionError::DuplicateEventType(
                schema.event_type.to_owned(),
            ));
        }
    }
    Ok(())
}

pub fn render_audit_chain_emission_checklist(
    plan: &EnterpriseAuditChainEmissionPlan,
) -> Result<String, EnterpriseAuditChainEmissionError> {
    validate_enterprise_suite_audit_chain_emission_plan(plan)?;
    let mut checklist = String::new();
    checklist.push_str("Enterprise Suite audit-chain emission contract\n");
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
) -> EnterpriseAuditEventSchema {
    EnterpriseAuditEventSchema {
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
        "policy-admission" => "https://audit.oyatie.dev/enterprise-suite/policy-admission",
        "group-close-rollup" => "https://audit.oyatie.dev/enterprise-suite/group-close-rollup",
        "cross-product-workflow" => {
            "https://audit.oyatie.dev/enterprise-suite/cross-product-workflow"
        }
        "incident-rollback" => "https://audit.oyatie.dev/enterprise-suite/incident-rollback",
        "ops-command" => "https://audit.oyatie.dev/enterprise-suite/ops-command",
        "listener-gateway" => "https://audit.oyatie.dev/enterprise-suite/listener-gateway",
        "identity-provider" => "https://audit.oyatie.dev/enterprise-suite/identity-provider",
        "postgres-rls" => "https://audit.oyatie.dev/enterprise-suite/postgres-rls",
        "cloud-readiness" => "https://audit.oyatie.dev/enterprise-suite/cloud-readiness",
        _ => "",
    }
}

fn schema_for(source_suffix: &str) -> &'static str {
    match source_suffix {
        "policy-admission" => {
            "https://schemas.oyatie.dev/enterprise-suite/audit/policy-admission.v1.json"
        }
        "group-close-rollup" => {
            "https://schemas.oyatie.dev/enterprise-suite/audit/group-close-rollup.v1.json"
        }
        "cross-product-workflow" => {
            "https://schemas.oyatie.dev/enterprise-suite/audit/cross-product-workflow.v1.json"
        }
        "incident-rollback" => {
            "https://schemas.oyatie.dev/enterprise-suite/audit/incident-rollback.v1.json"
        }
        "ops-command" => "https://schemas.oyatie.dev/enterprise-suite/audit/ops-command.v1.json",
        "listener-gateway" => {
            "https://schemas.oyatie.dev/enterprise-suite/audit/listener-gateway.v1.json"
        }
        "identity-provider" => {
            "https://schemas.oyatie.dev/enterprise-suite/audit/identity-provider.v1.json"
        }
        "postgres-rls" => "https://schemas.oyatie.dev/enterprise-suite/audit/postgres-rls.v1.json",
        "cloud-readiness" => {
            "https://schemas.oyatie.dev/enterprise-suite/audit/cloud-readiness.v1.json"
        }
        _ => "",
    }
}

fn validate_coordinate(
    coordinate: &ChainCoordinate,
) -> Result<(), EnterpriseAuditChainEmissionError> {
    if coordinate.pack != PACK
        || coordinate.tenant_partition != TENANT_PARTITION
        || coordinate.period != PERIOD
        || contains_credential_material(&coordinate.pack)
        || contains_credential_material(&coordinate.tenant_partition)
        || contains_credential_material(&coordinate.period)
    {
        return Err(EnterpriseAuditChainEmissionError::InvalidCoordinate);
    }
    Ok(())
}

fn ensure_no_runtime_overclaim(
    plan: &EnterpriseAuditChainEmissionPlan,
) -> Result<(), EnterpriseAuditChainEmissionError> {
    if plan.runtime_emitter_attached
        || plan.write_ahead_log_runtime_attached
        || plan.broker_publish_runtime_attached
        || plan.merkle_sealer_runtime_attached
        || plan.cloud_audit_sink_attached
        || plan.runtime_audit_chain_emission_attached
    {
        return Err(EnterpriseAuditChainEmissionError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

fn validate_event_schema(
    schema: &EnterpriseAuditEventSchema,
) -> Result<(), EnterpriseAuditChainEmissionError> {
    if !schema
        .event_type
        .starts_with("dev.oyatie.enterprise_suite.")
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
        return Err(EnterpriseAuditChainEmissionError::InvalidEventSchema(
            schema.event_type,
        ));
    }
    for attr in required_context_attributes() {
        if !schema.required_attributes.contains(&attr) {
            return Err(EnterpriseAuditChainEmissionError::MissingRequiredAttribute(
                attr,
            ));
        }
    }
    for ext in required_extension_attributes() {
        if !schema.required_extensions.contains(&ext) {
            return Err(EnterpriseAuditChainEmissionError::MissingRequiredExtension(
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
            return Err(EnterpriseAuditChainEmissionError::SensitiveContextLeak(
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
