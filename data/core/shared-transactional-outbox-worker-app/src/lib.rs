//! SQLx transactional outbox worker-cycle seam.
//!
//! This crate composes one explicit claim→dispatch→state-mutation cycle over
//! the SQLx drain adapter and runtime-neutral dispatch app. It does not spawn a
//! daemon, schedule repeated polling, own a broker/gRPC client, or prove live
//! delivery guarantees.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use shared_transactional_outbox_adapter_sqlx::{
    OutboxClaimRequest, SqlxOutboxDrainError, SqlxTransactionalOutboxDrain,
};
use shared_transactional_outbox_dispatch_app::{
    OutboxDispatchAppError, OutboxDispatchBudget, OutboxTransportAck, OutboxTransportExecutor,
    plan_transport_from_outbox_event,
};
use shared_transactional_outbox_kernel::{
    BackboneOutboxTable, OutboxClaimBatch, OutboxDispatchState, OutboxMutationReport,
    PendingOutboxEvent,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OutboxWorkerError {
    Drain(SqlxOutboxDrainError),
    InvalidReportState {
        event_id: String, // data_class: INTERNAL_ONLY
        expected: OutboxDispatchState,
        actual: OutboxDispatchState,
    },
    MutationEventIdMismatch {
        expected: String, // data_class: INTERNAL_ONLY
        actual: String,   // data_class: INTERNAL_ONLY
    },
    MissingDeadLetterReason {
        event_id: String, // data_class: INTERNAL_ONLY
    },
    EventReportCountMismatch {
        claimed_count: usize,
        report_count: usize,
    },
    EventReportIdMismatch {
        expected: Vec<String>, // data_class: INTERNAL_ONLY
        actual: Vec<String>,   // data_class: INTERNAL_ONLY
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxWorkerCycleRequest {
    pub claim: OutboxClaimRequest,             // data_class: INTERNAL_ONLY
    pub dispatch_budget: OutboxDispatchBudget, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxWorkerEventReport {
    pub event_id: String,                          // data_class: INTERNAL_ONLY
    pub final_state: OutboxDispatchState,          // data_class: INTERNAL_ONLY
    pub transport_ack: Option<OutboxTransportAck>, // data_class: INTERNAL_ONLY
    pub mutation_report: OutboxMutationReport,     // data_class: INTERNAL_ONLY
    pub dead_letter_reason: Option<String>,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxWorkerCycleReport {
    pub table: BackboneOutboxTable, // data_class: INTERNAL_ONLY
    pub table_name: &'static str,   // data_class: INTERNAL_ONLY
    pub tenant_scope_ref: String,   // data_class: INTERNAL_ONLY
    pub worker_ref: String,         // data_class: INTERNAL_ONLY
    pub claimed_count: usize,       // data_class: INTERNAL_ONLY
    pub published_count: usize,     // data_class: INTERNAL_ONLY
    pub dead_letter_count: usize,   // data_class: INTERNAL_ONLY
    pub event_reports: Vec<OutboxWorkerEventReport>, // data_class: INTERNAL_ONLY
}

impl From<SqlxOutboxDrainError> for OutboxWorkerError {
    fn from(error: SqlxOutboxDrainError) -> Self {
        Self::Drain(error)
    }
}

impl OutboxWorkerCycleRequest {
    #[must_use]
    pub const fn new(claim: OutboxClaimRequest, dispatch_budget: OutboxDispatchBudget) -> Self {
        Self {
            claim,
            dispatch_budget,
        }
    }
}

impl OutboxWorkerEventReport {
    pub fn published(
        event_id: impl Into<String>,
        transport_ack: OutboxTransportAck,
        mutation_report: OutboxMutationReport,
    ) -> Result<Self, OutboxWorkerError> {
        let event_id = event_id.into();
        ensure_mutation_state(&event_id, &mutation_report, OutboxDispatchState::Published)?;
        Ok(Self {
            event_id,
            final_state: OutboxDispatchState::Published,
            transport_ack: Some(transport_ack),
            mutation_report,
            dead_letter_reason: None,
        })
    }

    pub fn dead_letter(
        event_id: impl Into<String>,
        reason: impl Into<String>,
        mutation_report: OutboxMutationReport,
    ) -> Result<Self, OutboxWorkerError> {
        let event_id = event_id.into();
        let reason = reason.into();
        ensure_mutation_state(&event_id, &mutation_report, OutboxDispatchState::DeadLetter)?;
        if reason.trim().is_empty() {
            return Err(OutboxWorkerError::MissingDeadLetterReason { event_id });
        }
        Ok(Self {
            event_id,
            final_state: OutboxDispatchState::DeadLetter,
            transport_ack: None,
            mutation_report,
            dead_letter_reason: Some(reason),
        })
    }
}

pub async fn run_sqlx_outbox_worker_cycle(
    drain: &SqlxTransactionalOutboxDrain,
    request: OutboxWorkerCycleRequest,
    executor: &mut dyn OutboxTransportExecutor,
) -> Result<OutboxWorkerCycleReport, OutboxWorkerError> {
    let batch = drain.claim_pending_batch(request.claim).await?;
    let mut event_reports = Vec::with_capacity(batch.events.len());
    for event in &batch.events {
        match dispatch_event(event, request.dispatch_budget, executor) {
            Ok(transport_ack) => {
                let mutation = drain
                    .mark_published(event.table, &event.tenant_scope_ref, &event.event_id)
                    .await?;
                event_reports.push(OutboxWorkerEventReport::published(
                    event.event_id.clone(),
                    transport_ack,
                    mutation,
                )?);
            }
            Err(error) => {
                let mutation = drain
                    .mark_dead_letter(event.table, &event.tenant_scope_ref, &event.event_id)
                    .await?;
                event_reports.push(OutboxWorkerEventReport::dead_letter(
                    event.event_id.clone(),
                    format!("{error:?}"),
                    mutation,
                )?);
            }
        }
    }
    cycle_report_from_parts(&batch, event_reports)
}

pub fn cycle_report_from_parts(
    batch: &OutboxClaimBatch,
    event_reports: Vec<OutboxWorkerEventReport>,
) -> Result<OutboxWorkerCycleReport, OutboxWorkerError> {
    if batch.claimed_count != event_reports.len() {
        return Err(OutboxWorkerError::EventReportCountMismatch {
            claimed_count: batch.claimed_count,
            report_count: event_reports.len(),
        });
    }
    let actual_event_ids: Vec<String> = event_reports
        .iter()
        .map(|report| report.event_id.clone())
        .collect();
    if batch.event_ids != actual_event_ids {
        return Err(OutboxWorkerError::EventReportIdMismatch {
            expected: batch.event_ids.clone(),
            actual: actual_event_ids,
        });
    }
    let published_count = event_reports
        .iter()
        .filter(|report| report.final_state == OutboxDispatchState::Published)
        .count();
    let dead_letter_count = event_reports
        .iter()
        .filter(|report| report.final_state == OutboxDispatchState::DeadLetter)
        .count();
    Ok(OutboxWorkerCycleReport {
        table: batch.table,
        table_name: batch.table_name,
        tenant_scope_ref: batch.tenant_scope_ref.clone(),
        worker_ref: batch.worker_ref.clone(),
        claimed_count: batch.claimed_count,
        published_count,
        dead_letter_count,
        event_reports,
    })
}

fn dispatch_event(
    event: &PendingOutboxEvent,
    dispatch_budget: OutboxDispatchBudget,
    executor: &mut dyn OutboxTransportExecutor,
) -> Result<OutboxTransportAck, OutboxDispatchAppError> {
    let plan = plan_transport_from_outbox_event(event, dispatch_budget)?;
    executor.execute_outbox_transport(&plan)
}

fn ensure_mutation_state(
    event_id: &str,
    mutation_report: &OutboxMutationReport,
    expected: OutboxDispatchState,
) -> Result<(), OutboxWorkerError> {
    if mutation_report.event_id != event_id {
        return Err(OutboxWorkerError::MutationEventIdMismatch {
            expected: event_id.to_string(),
            actual: mutation_report.event_id.clone(),
        });
    }
    if mutation_report.state == expected {
        Ok(())
    } else {
        Err(OutboxWorkerError::InvalidReportState {
            event_id: event_id.to_string(),
            expected,
            actual: mutation_report.state,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_transactional_outbox_dispatch_app::RecordingOutboxTransportExecutor;

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

    fn event_with_id(event_id: &str) -> PendingOutboxEvent {
        PendingOutboxEvent {
            event_id: event_id.into(),
            aggregate_id: format!("message:{event_id}"),
            ..event()
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

    fn mutation(state: OutboxDispatchState) -> OutboxMutationReport {
        OutboxMutationReport {
            table: BackboneOutboxTable::MessengerMessageStream,
            table_name: "messenger_message_stream.protocol_outbox_events",
            tenant_scope_ref: "tenant:t".into(),
            event_id: "event:e".into(),
            state,
            rows_affected: 1,
        }
    }

    #[test]
    fn published_event_report_requires_published_mutation_state() {
        let mut executor = RecordingOutboxTransportExecutor::new();
        let ack = dispatch_event(&event(), OutboxDispatchBudget::default(), &mut executor).unwrap();

        let report = OutboxWorkerEventReport::published(
            "event:e",
            ack.clone(),
            mutation(OutboxDispatchState::Published),
        )
        .unwrap();

        assert_eq!(report.final_state, OutboxDispatchState::Published);
        assert_eq!(report.transport_ack, Some(ack));
        assert_eq!(report.dead_letter_reason, None);
        assert_eq!(report.mutation_report.rows_affected, 1);

        assert_eq!(
            OutboxWorkerEventReport::published(
                "event:e",
                report.transport_ack.clone().unwrap(),
                mutation(OutboxDispatchState::DeadLetter),
            ),
            Err(OutboxWorkerError::InvalidReportState {
                event_id: "event:e".into(),
                expected: OutboxDispatchState::Published,
                actual: OutboxDispatchState::DeadLetter,
            })
        );
    }

    #[test]
    fn dead_letter_event_report_requires_reason_and_dead_letter_state() {
        let report = OutboxWorkerEventReport::dead_letter(
            "event:e",
            "MissingField { field: audit_correlation_id }",
            mutation(OutboxDispatchState::DeadLetter),
        )
        .unwrap();

        assert_eq!(report.final_state, OutboxDispatchState::DeadLetter);
        assert_eq!(report.transport_ack, None);
        assert!(report.dead_letter_reason.unwrap().contains("MissingField"));

        assert_eq!(
            OutboxWorkerEventReport::dead_letter(
                "event:e",
                " ",
                mutation(OutboxDispatchState::DeadLetter),
            ),
            Err(OutboxWorkerError::MissingDeadLetterReason {
                event_id: "event:e".into(),
            })
        );
    }

    #[test]
    fn cycle_report_counts_published_and_dead_letter_outcomes() {
        let mut executor = RecordingOutboxTransportExecutor::new();
        let ack = dispatch_event(&event(), OutboxDispatchBudget::default(), &mut executor).unwrap();
        let event_dead = event_with_id("event:dead");
        let reports = vec![
            OutboxWorkerEventReport::published(
                "event:e",
                ack,
                mutation(OutboxDispatchState::Published),
            )
            .unwrap(),
            OutboxWorkerEventReport::dead_letter(
                "event:dead",
                "transport invariant violation",
                OutboxMutationReport {
                    event_id: "event:dead".into(),
                    state: OutboxDispatchState::DeadLetter,
                    ..mutation(OutboxDispatchState::DeadLetter)
                },
            )
            .unwrap(),
        ];

        let cycle = cycle_report_from_parts(&batch_with_events(vec![event(), event_dead]), reports)
            .unwrap();

        assert_eq!(cycle.claimed_count, 2);
        assert_eq!(cycle.published_count, 1);
        assert_eq!(cycle.dead_letter_count, 1);
        assert_eq!(cycle.worker_ref, "worker:a");
        assert_eq!(cycle.tenant_scope_ref, "tenant:t");
    }

    #[test]
    fn cycle_report_rejects_report_count_and_id_drift() {
        assert_eq!(
            cycle_report_from_parts(&batch(), Vec::new()),
            Err(OutboxWorkerError::EventReportCountMismatch {
                claimed_count: 1,
                report_count: 0,
            })
        );

        let mut executor = RecordingOutboxTransportExecutor::new();
        let ack = dispatch_event(&event(), OutboxDispatchBudget::default(), &mut executor).unwrap();
        let report = OutboxWorkerEventReport::published(
            "event:other",
            ack,
            OutboxMutationReport {
                event_id: "event:other".into(),
                ..mutation(OutboxDispatchState::Published)
            },
        )
        .unwrap();

        assert_eq!(
            cycle_report_from_parts(&batch(), vec![report]),
            Err(OutboxWorkerError::EventReportIdMismatch {
                expected: vec!["event:e".into()],
                actual: vec!["event:other".into()],
            })
        );
    }

    #[test]
    fn dispatch_event_surfaces_dispatch_validation_errors_for_dead_letter_path() {
        let mut invalid = event();
        invalid.audit_correlation_id = " ".into();
        let mut executor = RecordingOutboxTransportExecutor::new();

        assert_eq!(
            dispatch_event(&invalid, OutboxDispatchBudget::default(), &mut executor),
            Err(OutboxDispatchAppError::MissingField {
                field: "audit_correlation_id",
            })
        );
        assert!(executor.is_empty());
    }
}
