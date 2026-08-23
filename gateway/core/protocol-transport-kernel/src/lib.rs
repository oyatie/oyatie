//! Runtime-neutral transport planning for protocol-parity events.
//!
//! This kernel translates a validated protocol-parity event envelope into the
//! two transport plans the runtime layer must bind later: an AsyncAPI broker
//! publish plan and a proto/gRPC unary invocation descriptor. It also defines a
//! recording acknowledgement executor contract used by app/runtime tests before
//! a live broker or gRPC server exists. It performs no network I/O, protobuf
//! serialization, retry, or async runtime work.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use shared_protocol_parity_kernel::{ProtocolEventEnvelope, ProtocolParityError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProtocolTransportError {
    Protocol(ProtocolParityError),
    EmptyPayload,
    PayloadBudgetExceeded {
        actual_bytes: usize,
        budget_bytes: usize,
    },
    InvalidGrpcDeadline,
    ExecutionInvariantViolation {
        field: &'static str,
        broker_value: String,
        grpc_value: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProtocolTransportBudget {
    pub max_broker_payload_bytes: usize, // data_class: INTERNAL_ONLY
    pub grpc_deadline_ms: u64,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolPayload {
    pub encoding: &'static str,   // data_class: INTERNAL_ONLY
    pub bytes: Vec<u8>,           // data_class: INTERNAL_ONLY
    pub byte_len: usize,          // data_class: INTERNAL_ONLY
    pub max_payload_bytes: usize, // data_class: INTERNAL_ONLY
}

impl Default for ProtocolTransportBudget {
    fn default() -> Self {
        Self {
            max_broker_payload_bytes: 1 << 20,
            grpc_deadline_ms: 250,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrokerHeaders {
    pub tenant_scope_ref: String,        // data_class: INTERNAL_ONLY
    pub audit_correlation_id: String,    // data_class: INTERNAL_ONLY
    pub idempotency_key: Option<String>, // data_class: INTERNAL_ONLY
    pub policy_decision_ref: String,     // data_class: INTERNAL_ONLY
    pub schema_version: String,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrokerPublishPlan {
    pub operation_id: &'static str,     // data_class: INTERNAL_ONLY
    pub channel_address: &'static str,  // data_class: INTERNAL_ONLY
    pub message_name: &'static str,     // data_class: INTERNAL_ONLY
    pub event_kind: &'static str,       // data_class: INTERNAL_ONLY
    pub partition_key: String,          // data_class: INTERNAL_ONLY
    pub payload_encoding: &'static str, // data_class: INTERNAL_ONLY
    pub payload_bytes: usize,           // data_class: INTERNAL_ONLY
    pub max_payload_bytes: usize,       // data_class: INTERNAL_ONLY
    pub headers: BrokerHeaders,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GrpcUnaryPlan {
    pub package: &'static str,           // data_class: INTERNAL_ONLY
    pub service: &'static str,           // data_class: INTERNAL_ONLY
    pub rpc: &'static str,               // data_class: INTERNAL_ONLY
    pub fully_qualified_method: String,  // data_class: INTERNAL_ONLY
    pub tenant_scope_ref: String,        // data_class: INTERNAL_ONLY
    pub audit_correlation_id: String,    // data_class: INTERNAL_ONLY
    pub idempotency_key: Option<String>, // data_class: INTERNAL_ONLY
    pub policy_decision_ref: String,     // data_class: INTERNAL_ONLY
    pub deadline_ms: u64,                // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolTransportBundle {
    pub broker_publish: BrokerPublishPlan, // data_class: INTERNAL_ONLY
    pub grpc_unary: GrpcUnaryPlan,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrokerPublishAck {
    pub ack_ref: String,               // data_class: INTERNAL_ONLY
    pub operation_id: &'static str,    // data_class: INTERNAL_ONLY
    pub channel_address: &'static str, // data_class: INTERNAL_ONLY
    pub message_name: &'static str,    // data_class: INTERNAL_ONLY
    pub partition_key: String,         // data_class: INTERNAL_ONLY
    pub payload_bytes: usize,          // data_class: INTERNAL_ONLY
    pub audit_correlation_id: String,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GrpcUnaryAck {
    pub ack_ref: String,                // data_class: INTERNAL_ONLY
    pub fully_qualified_method: String, // data_class: INTERNAL_ONLY
    pub deadline_ms: u64,               // data_class: INTERNAL_ONLY
    pub audit_correlation_id: String,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolTransportExecutionReport {
    pub sequence: u64,                   // data_class: INTERNAL_ONLY
    pub broker_ack: BrokerPublishAck,    // data_class: INTERNAL_ONLY
    pub grpc_ack: GrpcUnaryAck,          // data_class: INTERNAL_ONLY
    pub tenant_scope_ref: String,        // data_class: INTERNAL_ONLY
    pub audit_correlation_id: String,    // data_class: INTERNAL_ONLY
    pub policy_decision_ref: String,     // data_class: INTERNAL_ONLY
    pub idempotency_key: Option<String>, // data_class: INTERNAL_ONLY
}

pub trait ProtocolTransportExecutor {
    fn execute_transport_bundle(
        &mut self,
        bundle: &ProtocolTransportBundle,
    ) -> Result<ProtocolTransportExecutionReport, ProtocolTransportError>;
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecordingProtocolTransportExecutor {
    reports: Vec<ProtocolTransportExecutionReport>,
}

impl RecordingProtocolTransportExecutor {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            reports: Vec::new(),
        }
    }

    #[must_use]
    pub fn reports(&self) -> &[ProtocolTransportExecutionReport] {
        &self.reports
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.reports.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.reports.is_empty()
    }
}

impl ProtocolTransportExecutor for RecordingProtocolTransportExecutor {
    fn execute_transport_bundle(
        &mut self,
        bundle: &ProtocolTransportBundle,
    ) -> Result<ProtocolTransportExecutionReport, ProtocolTransportError> {
        ensure_execution_invariants(bundle)?;
        let sequence = self.reports.len() as u64 + 1;
        let broker = &bundle.broker_publish;
        let grpc = &bundle.grpc_unary;
        let broker_ack = BrokerPublishAck {
            ack_ref: format!("broker:{}:{sequence}", broker.operation_id),
            operation_id: broker.operation_id,
            channel_address: broker.channel_address,
            message_name: broker.message_name,
            partition_key: broker.partition_key.clone(),
            payload_bytes: broker.payload_bytes,
            audit_correlation_id: broker.headers.audit_correlation_id.clone(),
        };
        let grpc_ack = GrpcUnaryAck {
            ack_ref: format!("grpc:{}:{sequence}", grpc.fully_qualified_method),
            fully_qualified_method: grpc.fully_qualified_method.clone(),
            deadline_ms: grpc.deadline_ms,
            audit_correlation_id: grpc.audit_correlation_id.clone(),
        };
        let report = ProtocolTransportExecutionReport {
            sequence,
            broker_ack,
            grpc_ack,
            tenant_scope_ref: broker.headers.tenant_scope_ref.clone(),
            audit_correlation_id: broker.headers.audit_correlation_id.clone(),
            policy_decision_ref: broker.headers.policy_decision_ref.clone(),
            idempotency_key: broker.headers.idempotency_key.clone(),
        };
        self.reports.push(report.clone());
        Ok(report)
    }
}

pub fn plan_transport_from_envelope(
    envelope: &ProtocolEventEnvelope,
    budget: ProtocolTransportBudget,
) -> Result<ProtocolTransportBundle, ProtocolTransportError> {
    let payload = encode_protocol_payload(envelope, budget)?;
    Ok(ProtocolTransportBundle {
        broker_publish: plan_broker_publish(envelope, payload.byte_len, budget)?,
        grpc_unary: plan_grpc_unary(envelope, budget)?,
    })
}

pub fn execute_transport_bundle(
    executor: &mut dyn ProtocolTransportExecutor,
    bundle: &ProtocolTransportBundle,
) -> Result<ProtocolTransportExecutionReport, ProtocolTransportError> {
    executor.execute_transport_bundle(bundle)
}

fn ensure_execution_invariants(
    bundle: &ProtocolTransportBundle,
) -> Result<(), ProtocolTransportError> {
    ensure_equal(
        "tenant_scope_ref",
        &bundle.broker_publish.headers.tenant_scope_ref,
        &bundle.grpc_unary.tenant_scope_ref,
    )?;
    ensure_equal(
        "audit_correlation_id",
        &bundle.broker_publish.headers.audit_correlation_id,
        &bundle.grpc_unary.audit_correlation_id,
    )?;
    ensure_equal(
        "policy_decision_ref",
        &bundle.broker_publish.headers.policy_decision_ref,
        &bundle.grpc_unary.policy_decision_ref,
    )?;
    if bundle.broker_publish.headers.idempotency_key != bundle.grpc_unary.idempotency_key {
        return Err(ProtocolTransportError::ExecutionInvariantViolation {
            field: "idempotency_key",
            broker_value: format!("{:?}", bundle.broker_publish.headers.idempotency_key),
            grpc_value: format!("{:?}", bundle.grpc_unary.idempotency_key),
        });
    }
    Ok(())
}

fn ensure_equal(
    field: &'static str,
    broker_value: &str,
    grpc_value: &str,
) -> Result<(), ProtocolTransportError> {
    if broker_value == grpc_value {
        Ok(())
    } else {
        Err(ProtocolTransportError::ExecutionInvariantViolation {
            field,
            broker_value: broker_value.to_string(),
            grpc_value: grpc_value.to_string(),
        })
    }
}

pub fn estimate_protocol_payload_bytes(
    envelope: &ProtocolEventEnvelope,
) -> Result<usize, ProtocolTransportError> {
    Ok(encode_protocol_payload(
        envelope,
        ProtocolTransportBudget {
            max_broker_payload_bytes: usize::MAX,
            grpc_deadline_ms: 1,
        },
    )?
    .byte_len)
}

pub fn encode_protocol_payload(
    envelope: &ProtocolEventEnvelope,
    budget: ProtocolTransportBudget,
) -> Result<ProtocolPayload, ProtocolTransportError> {
    envelope
        .validate()
        .map_err(ProtocolTransportError::Protocol)?;
    let payload_json = canonical_protocol_payload_json(envelope);
    let bytes = payload_json.into_bytes();
    let byte_len = bytes.len();
    if byte_len == 0 {
        Err(ProtocolTransportError::EmptyPayload)
    } else if byte_len > budget.max_broker_payload_bytes {
        Err(ProtocolTransportError::PayloadBudgetExceeded {
            actual_bytes: byte_len,
            budget_bytes: budget.max_broker_payload_bytes,
        })
    } else {
        Ok(ProtocolPayload {
            encoding: "proto-json-v1",
            bytes,
            byte_len,
            max_payload_bytes: budget.max_broker_payload_bytes,
        })
    }
}

fn canonical_protocol_payload_json(envelope: &ProtocolEventEnvelope) -> String {
    let mut out = String::new();
    let mut first = true;
    out.push('{');
    push_json_string_field(
        &mut out,
        &mut first,
        "schema_version",
        &envelope.schema_version,
    );
    push_json_string_field(
        &mut out,
        &mut first,
        "event_kind",
        envelope.binding.asyncapi_event_kind,
    );
    push_json_string_field(
        &mut out,
        &mut first,
        "message_name",
        envelope.binding.asyncapi_message_name,
    );
    push_json_string_field(
        &mut out,
        &mut first,
        "tenant_scope_ref",
        &envelope.tenant_scope_ref,
    );
    push_json_string_field(&mut out, &mut first, "aggregate_id", &envelope.aggregate_id);
    push_json_string_field(
        &mut out,
        &mut first,
        "audit_correlation_id",
        &envelope.audit_correlation_id,
    );
    push_json_optional_string_field(
        &mut out,
        &mut first,
        "idempotency_key",
        envelope.idempotency_key.as_deref(),
    );
    push_json_string_field(
        &mut out,
        &mut first,
        "policy_decision_ref",
        &envelope.policy_decision_ref,
    );
    out.push('}');
    out
}

fn push_json_string_field(out: &mut String, first: &mut bool, name: &str, value: &str) {
    push_json_field_prefix(out, first, name);
    push_json_string(out, value);
}

fn push_json_optional_string_field(
    out: &mut String,
    first: &mut bool,
    name: &str,
    value: Option<&str>,
) {
    push_json_field_prefix(out, first, name);
    if let Some(value) = value {
        push_json_string(out, value);
    } else {
        out.push_str("null");
    }
}

fn push_json_field_prefix(out: &mut String, first: &mut bool, name: &str) {
    if *first {
        *first = false;
    } else {
        out.push(',');
    }
    push_json_string(out, name);
    out.push(':');
}

fn push_json_string(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out.push('"');
}

pub fn plan_broker_publish(
    envelope: &ProtocolEventEnvelope,
    payload_bytes: usize,
    budget: ProtocolTransportBudget,
) -> Result<BrokerPublishPlan, ProtocolTransportError> {
    envelope
        .validate()
        .map_err(ProtocolTransportError::Protocol)?;
    if payload_bytes == 0 {
        return Err(ProtocolTransportError::EmptyPayload);
    }
    if payload_bytes > budget.max_broker_payload_bytes {
        return Err(ProtocolTransportError::PayloadBudgetExceeded {
            actual_bytes: payload_bytes,
            budget_bytes: budget.max_broker_payload_bytes,
        });
    }

    Ok(BrokerPublishPlan {
        operation_id: envelope.binding.asyncapi_operation_id,
        channel_address: envelope.binding.asyncapi_channel_address,
        message_name: envelope.binding.asyncapi_message_name,
        event_kind: envelope.binding.asyncapi_event_kind,
        partition_key: envelope.aggregate_id.clone(),
        payload_encoding: "proto-json-v1",
        payload_bytes,
        max_payload_bytes: budget.max_broker_payload_bytes,
        headers: BrokerHeaders {
            tenant_scope_ref: envelope.tenant_scope_ref.clone(),
            audit_correlation_id: envelope.audit_correlation_id.clone(),
            idempotency_key: envelope.idempotency_key.clone(),
            policy_decision_ref: envelope.policy_decision_ref.clone(),
            schema_version: envelope.schema_version.clone(),
        },
    })
}

pub fn plan_grpc_unary(
    envelope: &ProtocolEventEnvelope,
    budget: ProtocolTransportBudget,
) -> Result<GrpcUnaryPlan, ProtocolTransportError> {
    envelope
        .validate()
        .map_err(ProtocolTransportError::Protocol)?;
    if budget.grpc_deadline_ms == 0 {
        return Err(ProtocolTransportError::InvalidGrpcDeadline);
    }
    let package = envelope.binding.proto_package;
    let service = envelope.binding.proto_service;
    let rpc = envelope.binding.proto_rpc;

    Ok(GrpcUnaryPlan {
        package,
        service,
        rpc,
        fully_qualified_method: format!("/{package}.{service}/{rpc}"),
        tenant_scope_ref: envelope.tenant_scope_ref.clone(),
        audit_correlation_id: envelope.audit_correlation_id.clone(),
        idempotency_key: envelope.idempotency_key.clone(),
        policy_decision_ref: envelope.policy_decision_ref.clone(),
        deadline_ms: budget.grpc_deadline_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_protocol_parity_kernel::{
        ProtocolEventEnvelope, ProtocolParityBinding, ProtocolParityBindingSpec,
    };

    fn binding() -> ProtocolParityBinding {
        ProtocolParityBinding::new(ProtocolParityBindingSpec {
            rest_operation_id: "postMessage",
            asyncapi_operation_id: "emitMessagePosted",
            asyncapi_channel_address: "workflow-events/messenger.message.posted",
            asyncapi_message_name: "MessagePosted",
            asyncapi_event_kind: "oya.messenger.message.posted.v1",
            receipt_event_type: "messenger.message.sent",
            proto_package: "oya.messenger.v1",
            proto_service: "MessageStream",
            proto_rpc: "PostMessage",
        })
        .unwrap()
    }

    fn envelope() -> ProtocolEventEnvelope {
        ProtocolEventEnvelope::new(
            binding(),
            "1.0.0",
            "tenant:t",
            "message:m",
            "audit:a",
            Some("idem:i".into()),
            "policy:p",
        )
        .unwrap()
    }

    #[test]
    fn transport_bundle_maps_asyncapi_and_grpc_from_one_envelope() {
        let bundle = plan_transport_from_envelope(
            &envelope(),
            ProtocolTransportBudget {
                max_broker_payload_bytes: 4096,
                grpc_deadline_ms: 300,
            },
        )
        .unwrap();

        assert_eq!(bundle.broker_publish.operation_id, "emitMessagePosted");
        assert_eq!(bundle.broker_publish.partition_key, "message:m");
        assert_eq!(bundle.broker_publish.headers.tenant_scope_ref, "tenant:t");
        assert_eq!(bundle.grpc_unary.rpc, "PostMessage");
        assert_eq!(
            bundle.grpc_unary.fully_qualified_method,
            "/oya.messenger.v1.MessageStream/PostMessage"
        );
        assert_eq!(bundle.grpc_unary.deadline_ms, 300);
    }

    #[test]
    fn broker_publish_refuses_zero_payload_and_payload_budget_overrun() {
        assert_eq!(
            plan_broker_publish(&envelope(), 0, ProtocolTransportBudget::default()),
            Err(ProtocolTransportError::EmptyPayload)
        );
        assert_eq!(
            plan_broker_publish(
                &envelope(),
                128,
                ProtocolTransportBudget {
                    max_broker_payload_bytes: 12,
                    grpc_deadline_ms: 250,
                }
            ),
            Err(ProtocolTransportError::PayloadBudgetExceeded {
                actual_bytes: 128,
                budget_bytes: 12,
            })
        );
    }

    #[test]
    fn grpc_plan_requires_nonzero_deadline() {
        assert_eq!(
            plan_grpc_unary(
                &envelope(),
                ProtocolTransportBudget {
                    max_broker_payload_bytes: 4096,
                    grpc_deadline_ms: 0,
                }
            ),
            Err(ProtocolTransportError::InvalidGrpcDeadline)
        );
    }

    #[test]
    fn payload_codec_emits_canonical_proto_json_bytes() {
        let payload =
            encode_protocol_payload(&envelope(), ProtocolTransportBudget::default()).unwrap();
        let json = String::from_utf8(payload.bytes.clone()).unwrap();

        assert_eq!(payload.encoding, "proto-json-v1");
        assert_eq!(payload.byte_len, payload.bytes.len());
        assert!(json.starts_with("{\"schema_version\":\"1.0.0\""));
        assert!(json.contains("\"event_kind\":\"oya.messenger.message.posted.v1\""));
        assert!(json.contains("\"tenant_scope_ref\":\"tenant:t\""));
        assert!(json.contains("\"idempotency_key\":\"idem:i\""));
        assert_eq!(
            estimate_protocol_payload_bytes(&envelope()).unwrap(),
            payload.byte_len
        );
    }

    #[test]
    fn payload_codec_escapes_quotes_backslashes_and_control_chars() {
        let envelope = ProtocolEventEnvelope::new(
            binding(),
            "1.0.0",
            "tenant:\"quoted\"",
            "message:\\path",
            "audit:\nline",
            None,
            "policy:\tstep",
        )
        .unwrap();

        let payload =
            encode_protocol_payload(&envelope, ProtocolTransportBudget::default()).unwrap();
        let json = String::from_utf8(payload.bytes).unwrap();

        assert!(json.contains("tenant:\\\"quoted\\\""));
        assert!(json.contains("message:\\\\path"));
        assert!(json.contains("audit:\\nline"));
        assert!(json.contains("\"idempotency_key\":null"));
        assert!(json.contains("policy:\\tstep"));
    }

    #[test]
    fn payload_codec_enforces_broker_payload_budget() {
        let err = encode_protocol_payload(
            &envelope(),
            ProtocolTransportBudget {
                max_broker_payload_bytes: 8,
                grpc_deadline_ms: 250,
            },
        )
        .unwrap_err();

        assert!(matches!(
            err,
            ProtocolTransportError::PayloadBudgetExceeded {
                actual_bytes: _,
                budget_bytes: 8
            }
        ));
    }

    #[test]
    fn recording_executor_emits_broker_and_grpc_ack_report() {
        let bundle =
            plan_transport_from_envelope(&envelope(), ProtocolTransportBudget::default()).unwrap();
        let mut executor = RecordingProtocolTransportExecutor::new();

        let report = execute_transport_bundle(&mut executor, &bundle).unwrap();

        assert_eq!(report.sequence, 1);
        assert_eq!(report.tenant_scope_ref, "tenant:t");
        assert_eq!(report.audit_correlation_id, "audit:a");
        assert_eq!(report.policy_decision_ref, "policy:p");
        assert_eq!(report.idempotency_key.as_deref(), Some("idem:i"));
        assert_eq!(report.broker_ack.operation_id, "emitMessagePosted");
        assert_eq!(
            report.broker_ack.channel_address,
            "workflow-events/messenger.message.posted"
        );
        assert_eq!(report.broker_ack.message_name, "MessagePosted");
        assert_eq!(report.broker_ack.partition_key, "message:m");
        assert_eq!(
            report.broker_ack.payload_bytes,
            bundle.broker_publish.payload_bytes
        );
        assert_eq!(
            report.grpc_ack.fully_qualified_method,
            "/oya.messenger.v1.MessageStream/PostMessage"
        );
        assert_eq!(report.grpc_ack.deadline_ms, 250);
        assert_eq!(executor.len(), 1);
        assert_eq!(executor.reports()[0], report);

        let second = execute_transport_bundle(&mut executor, &bundle).unwrap();
        assert_eq!(second.sequence, 2);
        assert_eq!(second.broker_ack.ack_ref, "broker:emitMessagePosted:2");
    }

    #[test]
    fn recording_executor_detects_broker_grpc_scope_drift() {
        let mut bundle =
            plan_transport_from_envelope(&envelope(), ProtocolTransportBudget::default()).unwrap();
        bundle.grpc_unary.tenant_scope_ref = "tenant:other".into();
        let mut executor = RecordingProtocolTransportExecutor::new();

        assert_eq!(
            execute_transport_bundle(&mut executor, &bundle),
            Err(ProtocolTransportError::ExecutionInvariantViolation {
                field: "tenant_scope_ref",
                broker_value: "tenant:t".into(),
                grpc_value: "tenant:other".into()
            })
        );
        assert!(executor.is_empty());
    }

    #[test]
    fn recording_executor_detects_idempotency_drift() {
        let mut bundle =
            plan_transport_from_envelope(&envelope(), ProtocolTransportBudget::default()).unwrap();
        bundle.grpc_unary.idempotency_key = Some("idem:other".into());
        let mut executor = RecordingProtocolTransportExecutor::new();

        assert_eq!(
            execute_transport_bundle(&mut executor, &bundle),
            Err(ProtocolTransportError::ExecutionInvariantViolation {
                field: "idempotency_key",
                broker_value: "Some(\"idem:i\")".into(),
                grpc_value: "Some(\"idem:other\")".into()
            })
        );
    }
}
