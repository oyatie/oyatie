//! Runtime-neutral transactional outbox dispatch app seam.
//!
//! This crate maps already-claimed outbox rows into dynamic broker/gRPC
//! transport plans and records dispatch acknowledgements through an injectable
//! executor trait. It performs no broker I/O, no gRPC network calls, no SQL
//! mutations, no background polling, and proves no delivery SLOs.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use shared_transactional_outbox_kernel::{
    BackboneOutboxTable, OutboxClaimBatch, OutboxDispatchState, PendingOutboxEvent,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OutboxDispatchAppError {
    MissingField {
        field: &'static str,
    },
    InvalidBudget {
        field: &'static str,
    },
    PayloadBudgetExceeded {
        actual_bytes: usize,
        budget_bytes: usize,
    },
    EmptyPayload,
    EventTableMismatch {
        expected: BackboneOutboxTable,
        actual: BackboneOutboxTable,
    },
    EventTenantMismatch {
        expected: String, // data_class: INTERNAL_ONLY
        actual: String,   // data_class: INTERNAL_ONLY
    },
    EventServiceMismatch {
        expected: &'static str,
        actual: String, // data_class: INTERNAL_ONLY
    },
    EventIdMismatch {
        expected: Vec<String>, // data_class: INTERNAL_ONLY
        actual: Vec<String>,   // data_class: INTERNAL_ONLY
    },
    TransportInvariantViolation {
        field: &'static str,
        broker_value: String, // data_class: INTERNAL_ONLY
        grpc_value: String,   // data_class: INTERNAL_ONLY
    },
    TransportExecutionFailed {
        transport: &'static str,
        error_ref: String, // data_class: INTERNAL_ONLY
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OutboxDispatchBudget {
    pub max_payload_bytes: usize, // data_class: INTERNAL_ONLY
    pub grpc_deadline_ms: u64,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxBrokerHeaders {
    pub tenant_scope_ref: String,        // data_class: INTERNAL_ONLY
    pub audit_correlation_id: String,    // data_class: INTERNAL_ONLY
    pub idempotency_key: Option<String>, // data_class: INTERNAL_ONLY
    pub policy_decision_ref: String,     // data_class: INTERNAL_ONLY
    pub schema_version: String,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxBrokerPublishPlan {
    pub operation_id: String,           // data_class: INTERNAL_ONLY
    pub channel_address: String,        // data_class: INTERNAL_ONLY
    pub message_name: String,           // data_class: INTERNAL_ONLY
    pub event_kind: String,             // data_class: INTERNAL_ONLY
    pub partition_key: String,          // data_class: INTERNAL_ONLY
    pub payload_encoding: &'static str, // data_class: INTERNAL_ONLY
    pub payload_bytes: usize,           // data_class: INTERNAL_ONLY
    pub max_payload_bytes: usize,       // data_class: INTERNAL_ONLY
    pub headers: OutboxBrokerHeaders,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxGrpcUnaryPlan {
    pub package: String,                 // data_class: INTERNAL_ONLY
    pub service: String,                 // data_class: INTERNAL_ONLY
    pub rpc: String,                     // data_class: INTERNAL_ONLY
    pub fully_qualified_method: String,  // data_class: INTERNAL_ONLY
    pub tenant_scope_ref: String,        // data_class: INTERNAL_ONLY
    pub audit_correlation_id: String,    // data_class: INTERNAL_ONLY
    pub idempotency_key: Option<String>, // data_class: INTERNAL_ONLY
    pub policy_decision_ref: String,     // data_class: INTERNAL_ONLY
    pub deadline_ms: u64,                // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxTransportPlan {
    pub table: BackboneOutboxTable, // data_class: INTERNAL_ONLY
    pub table_name: &'static str,   // data_class: INTERNAL_ONLY
    pub service_id: String,         // data_class: INTERNAL_ONLY
    pub event_id: String,           // data_class: INTERNAL_ONLY
    pub broker_publish: OutboxBrokerPublishPlan, // data_class: INTERNAL_ONLY
    pub grpc_unary: OutboxGrpcUnaryPlan, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxTransportAck {
    pub sequence: u64,                   // data_class: INTERNAL_ONLY
    pub event_id: String,                // data_class: INTERNAL_ONLY
    pub broker_ack_ref: String,          // data_class: INTERNAL_ONLY
    pub grpc_ack_ref: String,            // data_class: INTERNAL_ONLY
    pub tenant_scope_ref: String,        // data_class: INTERNAL_ONLY
    pub audit_correlation_id: String,    // data_class: INTERNAL_ONLY
    pub policy_decision_ref: String,     // data_class: INTERNAL_ONLY
    pub idempotency_key: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxEventDispatchReport {
    pub event_id: String,                       // data_class: INTERNAL_ONLY
    pub recommended_state: OutboxDispatchState, // data_class: INTERNAL_ONLY
    pub transport_ack: OutboxTransportAck,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxDispatchBatchReport {
    pub table: BackboneOutboxTable,       // data_class: INTERNAL_ONLY
    pub table_name: &'static str,         // data_class: INTERNAL_ONLY
    pub tenant_scope_ref: String,         // data_class: INTERNAL_ONLY
    pub worker_ref: String,               // data_class: INTERNAL_ONLY
    pub attempted_count: usize,           // data_class: INTERNAL_ONLY
    pub published_event_ids: Vec<String>, // data_class: INTERNAL_ONLY
    pub reports: Vec<OutboxEventDispatchReport>, // data_class: INTERNAL_ONLY
}

pub trait OutboxTransportExecutor {
    fn execute_outbox_transport(
        &mut self,
        plan: &OutboxTransportPlan,
    ) -> Result<OutboxTransportAck, OutboxDispatchAppError>;
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecordingOutboxTransportExecutor {
    reports: Vec<OutboxTransportAck>,
}

impl Default for OutboxDispatchBudget {
    fn default() -> Self {
        Self {
            max_payload_bytes: 1 << 20,
            grpc_deadline_ms: 250,
        }
    }
}

impl OutboxDispatchBudget {
    pub fn validate(&self) -> Result<(), OutboxDispatchAppError> {
        if self.max_payload_bytes == 0 {
            return Err(OutboxDispatchAppError::InvalidBudget {
                field: "max_payload_bytes",
            });
        }
        if self.grpc_deadline_ms == 0 {
            return Err(OutboxDispatchAppError::InvalidBudget {
                field: "grpc_deadline_ms",
            });
        }
        Ok(())
    }
}

impl RecordingOutboxTransportExecutor {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            reports: Vec::new(),
        }
    }

    #[must_use]
    pub fn reports(&self) -> &[OutboxTransportAck] {
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

impl OutboxTransportExecutor for RecordingOutboxTransportExecutor {
    fn execute_outbox_transport(
        &mut self,
        plan: &OutboxTransportPlan,
    ) -> Result<OutboxTransportAck, OutboxDispatchAppError> {
        ensure_transport_invariants(plan)?;
        let sequence = self.reports.len() as u64 + 1;
        let ack = OutboxTransportAck {
            sequence,
            event_id: plan.event_id.clone(),
            broker_ack_ref: format!("broker:{}:{sequence}", plan.broker_publish.operation_id),
            grpc_ack_ref: format!("grpc:{}:{sequence}", plan.grpc_unary.fully_qualified_method),
            tenant_scope_ref: plan.broker_publish.headers.tenant_scope_ref.clone(),
            audit_correlation_id: plan.broker_publish.headers.audit_correlation_id.clone(),
            policy_decision_ref: plan.broker_publish.headers.policy_decision_ref.clone(),
            idempotency_key: plan.broker_publish.headers.idempotency_key.clone(),
        };
        self.reports.push(ack.clone());
        Ok(ack)
    }
}

pub fn dispatch_claim_batch(
    batch: &OutboxClaimBatch,
    executor: &mut dyn OutboxTransportExecutor,
    budget: OutboxDispatchBudget,
) -> Result<OutboxDispatchBatchReport, OutboxDispatchAppError> {
    validate_claim_batch(batch)?;
    let mut reports = Vec::with_capacity(batch.events.len());
    for event in &batch.events {
        let plan = plan_transport_from_outbox_event(event, budget)?;
        let transport_ack = executor.execute_outbox_transport(&plan)?;
        reports.push(OutboxEventDispatchReport {
            event_id: event.event_id.clone(),
            recommended_state: OutboxDispatchState::Published,
            transport_ack,
        });
    }
    let published_event_ids = reports
        .iter()
        .map(|report| report.event_id.clone())
        .collect();
    Ok(OutboxDispatchBatchReport {
        table: batch.table,
        table_name: batch.table_name,
        tenant_scope_ref: batch.tenant_scope_ref.clone(),
        worker_ref: batch.worker_ref.clone(),
        attempted_count: batch.events.len(),
        published_event_ids,
        reports,
    })
}

pub fn plan_transport_from_outbox_event(
    event: &PendingOutboxEvent,
    budget: OutboxDispatchBudget,
) -> Result<OutboxTransportPlan, OutboxDispatchAppError> {
    budget.validate()?;
    validate_event_identity(event)?;
    let payload_bytes = estimate_outbox_payload_bytes(event)?;
    if payload_bytes > budget.max_payload_bytes {
        return Err(OutboxDispatchAppError::PayloadBudgetExceeded {
            actual_bytes: payload_bytes,
            budget_bytes: budget.max_payload_bytes,
        });
    }
    let broker_publish = OutboxBrokerPublishPlan {
        operation_id: event.asyncapi_operation_id.clone(),
        channel_address: event.asyncapi_channel_address.clone(),
        message_name: event.asyncapi_message_name.clone(),
        event_kind: event.event_kind.clone(),
        partition_key: event.aggregate_id.clone(),
        payload_encoding: "outbox-metadata-proto-json-v1",
        payload_bytes,
        max_payload_bytes: budget.max_payload_bytes,
        headers: OutboxBrokerHeaders {
            tenant_scope_ref: event.tenant_scope_ref.clone(),
            audit_correlation_id: event.audit_correlation_id.clone(),
            idempotency_key: event.idempotency_key.clone(),
            policy_decision_ref: event.policy_decision_ref.clone(),
            schema_version: event.schema_version.clone(),
        },
    };
    let grpc_unary = OutboxGrpcUnaryPlan {
        package: event.proto_package.clone(),
        service: event.proto_service.clone(),
        rpc: event.proto_rpc.clone(),
        fully_qualified_method: format!(
            "/{}.{}/{}",
            event.proto_package, event.proto_service, event.proto_rpc
        ),
        tenant_scope_ref: event.tenant_scope_ref.clone(),
        audit_correlation_id: event.audit_correlation_id.clone(),
        idempotency_key: event.idempotency_key.clone(),
        policy_decision_ref: event.policy_decision_ref.clone(),
        deadline_ms: budget.grpc_deadline_ms,
    };
    let plan = OutboxTransportPlan {
        table: event.table,
        table_name: event.table.table_name(),
        service_id: event.service_id.clone(),
        event_id: event.event_id.clone(),
        broker_publish,
        grpc_unary,
    };
    ensure_transport_invariants(&plan)?;
    Ok(plan)
}

pub fn validate_claim_batch(batch: &OutboxClaimBatch) -> Result<(), OutboxDispatchAppError> {
    require_non_empty("worker_ref", &batch.worker_ref)?;
    require_non_empty("tenant_scope_ref", &batch.tenant_scope_ref)?;
    let actual_event_ids: Vec<String> = batch
        .events
        .iter()
        .map(|event| event.event_id.clone())
        .collect();
    if batch.event_ids != actual_event_ids {
        return Err(OutboxDispatchAppError::EventIdMismatch {
            expected: batch.event_ids.clone(),
            actual: actual_event_ids,
        });
    }
    for event in &batch.events {
        if event.table != batch.table {
            return Err(OutboxDispatchAppError::EventTableMismatch {
                expected: batch.table,
                actual: event.table,
            });
        }
        if event.tenant_scope_ref != batch.tenant_scope_ref {
            return Err(OutboxDispatchAppError::EventTenantMismatch {
                expected: batch.tenant_scope_ref.clone(),
                actual: event.tenant_scope_ref.clone(),
            });
        }
        validate_event_identity(event)?;
    }
    Ok(())
}

pub fn estimate_outbox_payload_bytes(
    event: &PendingOutboxEvent,
) -> Result<usize, OutboxDispatchAppError> {
    validate_event_identity(event)?;
    let idempotency_len = event.idempotency_key.as_deref().map_or(0, str::len);
    let bytes = event.event_kind.len()
        + event.aggregate_id.len()
        + event.asyncapi_operation_id.len()
        + event.asyncapi_channel_address.len()
        + event.asyncapi_message_name.len()
        + event.proto_package.len()
        + event.proto_service.len()
        + event.proto_rpc.len()
        + event.schema_version.len()
        + event.tenant_scope_ref.len()
        + event.audit_correlation_id.len()
        + event.policy_decision_ref.len()
        + idempotency_len;
    if bytes == 0 {
        Err(OutboxDispatchAppError::EmptyPayload)
    } else {
        Ok(bytes)
    }
}

fn validate_event_identity(event: &PendingOutboxEvent) -> Result<(), OutboxDispatchAppError> {
    require_non_empty("service_id", &event.service_id)?;
    require_non_empty("tenant_scope_ref", &event.tenant_scope_ref)?;
    require_non_empty("event_id", &event.event_id)?;
    require_non_empty("event_kind", &event.event_kind)?;
    require_non_empty("aggregate_id", &event.aggregate_id)?;
    require_non_empty("asyncapi_operation_id", &event.asyncapi_operation_id)?;
    require_non_empty("asyncapi_channel_address", &event.asyncapi_channel_address)?;
    require_non_empty("asyncapi_message_name", &event.asyncapi_message_name)?;
    require_non_empty("proto_package", &event.proto_package)?;
    require_non_empty("proto_service", &event.proto_service)?;
    require_non_empty("proto_rpc", &event.proto_rpc)?;
    require_non_empty("schema_version", &event.schema_version)?;
    require_non_empty("policy_decision_ref", &event.policy_decision_ref)?;
    require_non_empty("audit_correlation_id", &event.audit_correlation_id)?;
    if let Some(idempotency_key) = &event.idempotency_key {
        require_non_empty("idempotency_key", idempotency_key)?;
    }
    if event.service_id != event.table.service_id() {
        return Err(OutboxDispatchAppError::EventServiceMismatch {
            expected: event.table.service_id(),
            actual: event.service_id.clone(),
        });
    }
    Ok(())
}

fn ensure_transport_invariants(plan: &OutboxTransportPlan) -> Result<(), OutboxDispatchAppError> {
    ensure_equal(
        "tenant_scope_ref",
        &plan.broker_publish.headers.tenant_scope_ref,
        &plan.grpc_unary.tenant_scope_ref,
    )?;
    ensure_equal(
        "audit_correlation_id",
        &plan.broker_publish.headers.audit_correlation_id,
        &plan.grpc_unary.audit_correlation_id,
    )?;
    ensure_equal(
        "policy_decision_ref",
        &plan.broker_publish.headers.policy_decision_ref,
        &plan.grpc_unary.policy_decision_ref,
    )?;
    if plan.broker_publish.headers.idempotency_key != plan.grpc_unary.idempotency_key {
        return Err(OutboxDispatchAppError::TransportInvariantViolation {
            field: "idempotency_key",
            broker_value: format!("{:?}", plan.broker_publish.headers.idempotency_key),
            grpc_value: format!("{:?}", plan.grpc_unary.idempotency_key),
        });
    }
    Ok(())
}

fn ensure_equal(
    field: &'static str,
    broker_value: &str,
    grpc_value: &str,
) -> Result<(), OutboxDispatchAppError> {
    if broker_value == grpc_value {
        Ok(())
    } else {
        Err(OutboxDispatchAppError::TransportInvariantViolation {
            field,
            broker_value: broker_value.to_string(),
            grpc_value: grpc_value.to_string(),
        })
    }
}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), OutboxDispatchAppError> {
    if value.trim().is_empty() {
        Err(OutboxDispatchAppError::MissingField { field })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event() -> PendingOutboxEvent {
        PendingOutboxEvent {
            table: BackboneOutboxTable::MessengerMessageStream,
            service_id: "messenger-message-stream".into(),
            tenant_scope_ref: "tenant:t".into(),
            event_id: "event:e".into(),
            event_kind: "oya.messenger.message.posted.v1".into(),
            aggregate_id: "message:m".into(),
            asyncapi_operation_id: "emitMessagePosted".into(),
            asyncapi_channel_address: "workflow-events/messenger.message.posted".into(),
            asyncapi_message_name: "MessagePosted".into(),
            proto_package: "oya.messenger.v1".into(),
            proto_service: "MessageStream".into(),
            proto_rpc: "PostMessage".into(),
            schema_version: "1.0.0".into(),
            idempotency_key: Some("idem:i".into()),
            policy_decision_ref: "policy:p".into(),
            audit_correlation_id: "audit:a".into(),
            attempt_count: 1,
        }
    }

    fn batch() -> OutboxClaimBatch {
        batch_with_events(vec![event()])
    }

    fn batch_with_events(events: Vec<PendingOutboxEvent>) -> OutboxClaimBatch {
        let event_ids = events.iter().map(|event| event.event_id.clone()).collect();
        OutboxClaimBatch {
            table: BackboneOutboxTable::MessengerMessageStream,
            table_name: "messenger_message_stream.protocol_outbox_events",
            worker_ref: "worker:a".into(),
            tenant_scope_ref: "tenant:t".into(),
            claimed_count: events.len(),
            event_ids,
            events,
        }
    }

    #[test]
    fn plans_dynamic_transport_from_outbox_metadata() {
        let plan =
            plan_transport_from_outbox_event(&event(), OutboxDispatchBudget::default()).unwrap();

        assert_eq!(
            plan.table_name,
            "messenger_message_stream.protocol_outbox_events"
        );
        assert_eq!(plan.event_id, "event:e");
        assert_eq!(plan.broker_publish.operation_id, "emitMessagePosted");
        assert_eq!(plan.broker_publish.partition_key, "message:m");
        assert_eq!(plan.broker_publish.headers.tenant_scope_ref, "tenant:t");
        assert!(plan.broker_publish.payload_bytes > 0);
        assert_eq!(plan.grpc_unary.rpc, "PostMessage");
        assert_eq!(
            plan.grpc_unary.fully_qualified_method,
            "/oya.messenger.v1.MessageStream/PostMessage"
        );
        assert_eq!(plan.grpc_unary.deadline_ms, 250);
    }

    #[test]
    fn dispatch_claim_batch_records_ack_and_recommends_published() {
        let mut executor = RecordingOutboxTransportExecutor::new();

        let report =
            dispatch_claim_batch(&batch(), &mut executor, OutboxDispatchBudget::default()).unwrap();

        assert_eq!(report.attempted_count, 1);
        assert_eq!(report.published_event_ids, vec!["event:e"]);
        assert_eq!(report.worker_ref, "worker:a");
        assert_eq!(
            report.reports[0].recommended_state,
            OutboxDispatchState::Published
        );
        assert_eq!(report.reports[0].transport_ack.sequence, 1);
        assert_eq!(
            report.reports[0].transport_ack.broker_ack_ref,
            "broker:emitMessagePosted:1"
        );
        assert_eq!(
            report.reports[0].transport_ack.grpc_ack_ref,
            "grpc:/oya.messenger.v1.MessageStream/PostMessage:1"
        );
        assert_eq!(executor.len(), 1);
        assert_eq!(executor.reports()[0], report.reports[0].transport_ack);
    }

    #[test]
    fn batch_validation_rejects_table_tenant_and_event_id_drift() {
        let mut wrong_table = event();
        wrong_table.table = BackboneOutboxTable::MailMailboxStore;
        assert_eq!(
            validate_claim_batch(&batch_with_events(vec![wrong_table])),
            Err(OutboxDispatchAppError::EventTableMismatch {
                expected: BackboneOutboxTable::MessengerMessageStream,
                actual: BackboneOutboxTable::MailMailboxStore,
            })
        );

        let mut wrong_tenant = event();
        wrong_tenant.tenant_scope_ref = "tenant:other".into();
        assert_eq!(
            validate_claim_batch(&batch_with_events(vec![wrong_tenant])),
            Err(OutboxDispatchAppError::EventTenantMismatch {
                expected: "tenant:t".into(),
                actual: "tenant:other".into(),
            })
        );

        let mut mismatched = batch();
        mismatched.event_ids = vec!["event:other".into()];
        assert_eq!(
            validate_claim_batch(&mismatched),
            Err(OutboxDispatchAppError::EventIdMismatch {
                expected: vec!["event:other".into()],
                actual: vec!["event:e".into()],
            })
        );
    }

    #[test]
    fn event_validation_rejects_service_drift_and_missing_fields() {
        let mut wrong_service = event();
        wrong_service.service_id = "mail-mailbox-store".into();
        assert_eq!(
            plan_transport_from_outbox_event(&wrong_service, OutboxDispatchBudget::default()),
            Err(OutboxDispatchAppError::EventServiceMismatch {
                expected: "messenger-message-stream",
                actual: "mail-mailbox-store".into(),
            })
        );

        let mut missing_audit = event();
        missing_audit.audit_correlation_id = " ".into();
        assert_eq!(
            plan_transport_from_outbox_event(&missing_audit, OutboxDispatchBudget::default()),
            Err(OutboxDispatchAppError::MissingField {
                field: "audit_correlation_id",
            })
        );
    }

    #[test]
    fn budget_validation_rejects_zero_deadline_and_payload_overrun() {
        assert_eq!(
            plan_transport_from_outbox_event(
                &event(),
                OutboxDispatchBudget {
                    max_payload_bytes: 1024,
                    grpc_deadline_ms: 0,
                }
            ),
            Err(OutboxDispatchAppError::InvalidBudget {
                field: "grpc_deadline_ms",
            })
        );
        assert_eq!(
            plan_transport_from_outbox_event(
                &event(),
                OutboxDispatchBudget {
                    max_payload_bytes: 4,
                    grpc_deadline_ms: 250,
                }
            ),
            Err(OutboxDispatchAppError::PayloadBudgetExceeded {
                actual_bytes: estimate_outbox_payload_bytes(&event()).unwrap(),
                budget_bytes: 4,
            })
        );
    }

    #[test]
    fn recording_executor_detects_transport_invariant_drift() {
        let mut plan =
            plan_transport_from_outbox_event(&event(), OutboxDispatchBudget::default()).unwrap();
        plan.grpc_unary.tenant_scope_ref = "tenant:other".into();
        let mut executor = RecordingOutboxTransportExecutor::new();

        assert_eq!(
            executor.execute_outbox_transport(&plan),
            Err(OutboxDispatchAppError::TransportInvariantViolation {
                field: "tenant_scope_ref",
                broker_value: "tenant:t".into(),
                grpc_value: "tenant:other".into(),
            })
        );
        assert!(executor.is_empty());
    }
}
