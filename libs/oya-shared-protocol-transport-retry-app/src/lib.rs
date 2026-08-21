//! Bounded retry seam for protocol transport execution.
//!
//! This app crate executes an already-planned broker/gRPC transport bundle
//! through an injected executor with explicit retryable/permanent attempt
//! classification. It performs no broker I/O, no gRPC network calls, no async
//! sleeping, no process supervision, and proves no production delivery SLOs.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use shared_protocol_transport_kernel::{
    ProtocolTransportBundle, ProtocolTransportError, ProtocolTransportExecutionReport,
    ProtocolTransportExecutor,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProtocolTransportRetryError {
    InvalidPolicy { field: &'static str },
    InvalidAttemptError { field: &'static str },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProtocolTransportRetryPolicy {
    pub max_attempts: u16,    // data_class: INTERNAL_ONLY
    pub base_backoff_ms: u64, // data_class: INTERNAL_ONLY
    pub max_backoff_ms: u64,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProtocolTransportAttemptError {
    Retryable { error_ref: String }, // data_class: INTERNAL_ONLY
    Permanent { error_ref: String }, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtocolTransportAttemptOutcome {
    Succeeded,
    RetryableError,
    PermanentError,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtocolTransportRetryStopReason {
    Succeeded,
    PermanentError,
    AttemptsExhausted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolTransportAttemptReport {
    pub attempt: u16,                             // data_class: INTERNAL_ONLY
    pub outcome: ProtocolTransportAttemptOutcome, // data_class: INTERNAL_ONLY
    pub planned_backoff_ms: Option<u64>,          // data_class: INTERNAL_ONLY
    pub error_ref: Option<String>,                // data_class: INTERNAL_ONLY
    pub ack_sequence: Option<u64>,                // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolTransportRetryRunReport {
    pub stop_reason: ProtocolTransportRetryStopReason, // data_class: INTERNAL_ONLY
    pub attempts: Vec<ProtocolTransportAttemptReport>, // data_class: INTERNAL_ONLY
    pub final_transport_report: Option<ProtocolTransportExecutionReport>, // data_class: INTERNAL_ONLY
}

pub trait ProtocolTransportRetryExecutor {
    fn execute_attempt(
        &mut self,
        bundle: &ProtocolTransportBundle,
        attempt: u16,
    ) -> Result<ProtocolTransportExecutionReport, ProtocolTransportAttemptError>;
}

pub struct KernelProtocolTransportRetryExecutor<'a> {
    inner: &'a mut dyn ProtocolTransportExecutor,
}

impl Default for ProtocolTransportRetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_backoff_ms: 100,
            max_backoff_ms: 1_000,
        }
    }
}

impl ProtocolTransportRetryPolicy {
    pub fn validate(&self) -> Result<(), ProtocolTransportRetryError> {
        if self.max_attempts == 0 {
            return Err(ProtocolTransportRetryError::InvalidPolicy {
                field: "max_attempts",
            });
        }
        if self.base_backoff_ms == 0 {
            return Err(ProtocolTransportRetryError::InvalidPolicy {
                field: "base_backoff_ms",
            });
        }
        if self.max_backoff_ms < self.base_backoff_ms {
            return Err(ProtocolTransportRetryError::InvalidPolicy {
                field: "max_backoff_ms",
            });
        }
        Ok(())
    }
}

impl ProtocolTransportAttemptError {
    pub fn retryable(error_ref: impl Into<String>) -> Result<Self, ProtocolTransportRetryError> {
        let error_ref = normalize_error_ref(error_ref.into())?;
        Ok(Self::Retryable { error_ref })
    }

    pub fn permanent(error_ref: impl Into<String>) -> Result<Self, ProtocolTransportRetryError> {
        let error_ref = normalize_error_ref(error_ref.into())?;
        Ok(Self::Permanent { error_ref })
    }

    #[must_use]
    pub fn error_ref(&self) -> &str {
        match self {
            Self::Retryable { error_ref } | Self::Permanent { error_ref } => error_ref,
        }
    }
}

impl<'a> KernelProtocolTransportRetryExecutor<'a> {
    #[must_use]
    pub fn new(inner: &'a mut dyn ProtocolTransportExecutor) -> Self {
        Self { inner }
    }
}

impl ProtocolTransportRetryExecutor for KernelProtocolTransportRetryExecutor<'_> {
    fn execute_attempt(
        &mut self,
        bundle: &ProtocolTransportBundle,
        _attempt: u16,
    ) -> Result<ProtocolTransportExecutionReport, ProtocolTransportAttemptError> {
        self.inner
            .execute_transport_bundle(bundle)
            .map_err(permanent_transport_error)
    }
}

pub fn execute_protocol_transport_with_retry(
    policy: ProtocolTransportRetryPolicy,
    bundle: &ProtocolTransportBundle,
    executor: &mut dyn ProtocolTransportRetryExecutor,
) -> Result<ProtocolTransportRetryRunReport, ProtocolTransportRetryError> {
    policy.validate()?;
    let mut attempts = Vec::with_capacity(usize::from(policy.max_attempts));

    for attempt in 1..=policy.max_attempts {
        match executor.execute_attempt(bundle, attempt) {
            Ok(report) => {
                attempts.push(ProtocolTransportAttemptReport {
                    attempt,
                    outcome: ProtocolTransportAttemptOutcome::Succeeded,
                    planned_backoff_ms: None,
                    error_ref: None,
                    ack_sequence: Some(report.sequence),
                });
                return Ok(ProtocolTransportRetryRunReport {
                    stop_reason: ProtocolTransportRetryStopReason::Succeeded,
                    attempts,
                    final_transport_report: Some(report),
                });
            }
            Err(ProtocolTransportAttemptError::Permanent { error_ref }) => {
                attempts.push(ProtocolTransportAttemptReport {
                    attempt,
                    outcome: ProtocolTransportAttemptOutcome::PermanentError,
                    planned_backoff_ms: None,
                    error_ref: Some(error_ref),
                    ack_sequence: None,
                });
                return Ok(ProtocolTransportRetryRunReport {
                    stop_reason: ProtocolTransportRetryStopReason::PermanentError,
                    attempts,
                    final_transport_report: None,
                });
            }
            Err(ProtocolTransportAttemptError::Retryable { error_ref }) => {
                let planned_backoff_ms = if attempt == policy.max_attempts {
                    None
                } else {
                    Some(planned_retry_backoff_ms(&policy, attempt))
                };
                attempts.push(ProtocolTransportAttemptReport {
                    attempt,
                    outcome: ProtocolTransportAttemptOutcome::RetryableError,
                    planned_backoff_ms,
                    error_ref: Some(error_ref),
                    ack_sequence: None,
                });
                if attempt == policy.max_attempts {
                    return Ok(ProtocolTransportRetryRunReport {
                        stop_reason: ProtocolTransportRetryStopReason::AttemptsExhausted,
                        attempts,
                        final_transport_report: None,
                    });
                }
            }
        }
    }

    Ok(ProtocolTransportRetryRunReport {
        stop_reason: ProtocolTransportRetryStopReason::AttemptsExhausted,
        attempts,
        final_transport_report: None,
    })
}

#[must_use]
pub fn planned_retry_backoff_ms(policy: &ProtocolTransportRetryPolicy, failed_attempt: u16) -> u64 {
    let exponent = u32::from(failed_attempt.saturating_sub(1));
    let multiplier = 1_u64.checked_shl(exponent).unwrap_or(u64::MAX);
    policy
        .base_backoff_ms
        .saturating_mul(multiplier)
        .min(policy.max_backoff_ms)
}

fn normalize_error_ref(error_ref: String) -> Result<String, ProtocolTransportRetryError> {
    let trimmed = error_ref.trim();
    if trimmed.is_empty() {
        return Err(ProtocolTransportRetryError::InvalidAttemptError { field: "error_ref" });
    }
    Ok(trimmed.to_string())
}

fn permanent_transport_error(error: ProtocolTransportError) -> ProtocolTransportAttemptError {
    ProtocolTransportAttemptError::Permanent {
        error_ref: format!("transport:{error:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_protocol_parity_kernel::{
        ProtocolEventEnvelope, ProtocolParityBinding, ProtocolParityBindingSpec,
    };
    use shared_protocol_transport_kernel::{
        RecordingProtocolTransportExecutor, execute_transport_bundle, plan_transport_from_envelope,
    };

    #[derive(Clone, Debug)]
    enum ScriptStep {
        Retryable(&'static str),
        Permanent(&'static str),
        Success,
    }

    #[derive(Clone, Debug)]
    struct ScriptedRetryExecutor {
        steps: Vec<ScriptStep>,
        index: usize,
    }

    impl ScriptedRetryExecutor {
        fn new(steps: Vec<ScriptStep>) -> Self {
            Self { steps, index: 0 }
        }
    }

    impl ProtocolTransportRetryExecutor for ScriptedRetryExecutor {
        fn execute_attempt(
            &mut self,
            bundle: &ProtocolTransportBundle,
            attempt: u16,
        ) -> Result<ProtocolTransportExecutionReport, ProtocolTransportAttemptError> {
            let step = self
                .steps
                .get(self.index)
                .cloned()
                .unwrap_or(ScriptStep::Success);
            self.index += 1;
            match step {
                ScriptStep::Retryable(error_ref) => {
                    Err(ProtocolTransportAttemptError::retryable(error_ref)
                        .expect("scripted retryable refs are non-empty"))
                }
                ScriptStep::Permanent(error_ref) => {
                    Err(ProtocolTransportAttemptError::permanent(error_ref)
                        .expect("scripted permanent refs are non-empty"))
                }
                ScriptStep::Success => Ok(ProtocolTransportExecutionReport {
                    sequence: u64::from(attempt),
                    broker_ack: shared_protocol_transport_kernel::BrokerPublishAck {
                        ack_ref: format!("broker:{}:{attempt}", bundle.broker_publish.operation_id),
                        operation_id: bundle.broker_publish.operation_id,
                        channel_address: bundle.broker_publish.channel_address,
                        message_name: bundle.broker_publish.message_name,
                        partition_key: bundle.broker_publish.partition_key.clone(),
                        payload_bytes: bundle.broker_publish.payload_bytes,
                        audit_correlation_id: bundle
                            .broker_publish
                            .headers
                            .audit_correlation_id
                            .clone(),
                    },
                    grpc_ack: shared_protocol_transport_kernel::GrpcUnaryAck {
                        ack_ref: format!(
                            "grpc:{}:{attempt}",
                            bundle.grpc_unary.fully_qualified_method
                        ),
                        fully_qualified_method: bundle.grpc_unary.fully_qualified_method.clone(),
                        deadline_ms: bundle.grpc_unary.deadline_ms,
                        audit_correlation_id: bundle.grpc_unary.audit_correlation_id.clone(),
                    },
                    tenant_scope_ref: bundle.broker_publish.headers.tenant_scope_ref.clone(),
                    audit_correlation_id: bundle
                        .broker_publish
                        .headers
                        .audit_correlation_id
                        .clone(),
                    policy_decision_ref: bundle.broker_publish.headers.policy_decision_ref.clone(),
                    idempotency_key: bundle.broker_publish.headers.idempotency_key.clone(),
                }),
            }
        }
    }

    fn policy() -> ProtocolTransportRetryPolicy {
        ProtocolTransportRetryPolicy {
            max_attempts: 3,
            base_backoff_ms: 100,
            max_backoff_ms: 250,
        }
    }

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

    fn bundle() -> ProtocolTransportBundle {
        let envelope = ProtocolEventEnvelope::new(
            binding(),
            "1.0.0",
            "tenant:t",
            "message:m",
            "audit:a",
            Some("idem:i".into()),
            "policy:p",
        )
        .unwrap();
        plan_transport_from_envelope(&envelope, Default::default()).unwrap()
    }

    #[test]
    fn policy_rejects_zero_attempts_zero_backoff_and_inverted_cap() {
        let mut cfg = policy();
        cfg.max_attempts = 0;
        assert_eq!(
            cfg.validate(),
            Err(ProtocolTransportRetryError::InvalidPolicy {
                field: "max_attempts"
            })
        );

        cfg = policy();
        cfg.base_backoff_ms = 0;
        assert_eq!(
            cfg.validate(),
            Err(ProtocolTransportRetryError::InvalidPolicy {
                field: "base_backoff_ms"
            })
        );

        cfg = policy();
        cfg.max_backoff_ms = 99;
        assert_eq!(
            cfg.validate(),
            Err(ProtocolTransportRetryError::InvalidPolicy {
                field: "max_backoff_ms"
            })
        );
    }

    #[test]
    fn retryable_attempts_backoff_then_success() {
        let mut executor = ScriptedRetryExecutor::new(vec![
            ScriptStep::Retryable("broker:timeout"),
            ScriptStep::Retryable("grpc:unavailable"),
            ScriptStep::Success,
        ]);

        let report =
            execute_protocol_transport_with_retry(policy(), &bundle(), &mut executor).unwrap();

        assert_eq!(
            report.stop_reason,
            ProtocolTransportRetryStopReason::Succeeded
        );
        assert_eq!(report.attempts.len(), 3);
        assert_eq!(report.attempts[0].planned_backoff_ms, Some(100));
        assert_eq!(report.attempts[1].planned_backoff_ms, Some(200));
        assert_eq!(report.attempts[2].ack_sequence, Some(3));
        assert!(report.final_transport_report.is_some());
    }

    #[test]
    fn retryable_errors_exhaust_attempts_without_terminal_backoff() {
        let mut executor = ScriptedRetryExecutor::new(vec![
            ScriptStep::Retryable("broker:timeout"),
            ScriptStep::Retryable("broker:timeout"),
            ScriptStep::Retryable("broker:timeout"),
        ]);

        let report =
            execute_protocol_transport_with_retry(policy(), &bundle(), &mut executor).unwrap();

        assert_eq!(
            report.stop_reason,
            ProtocolTransportRetryStopReason::AttemptsExhausted
        );
        assert_eq!(report.attempts.len(), 3);
        assert_eq!(report.attempts[0].planned_backoff_ms, Some(100));
        assert_eq!(report.attempts[1].planned_backoff_ms, Some(200));
        assert_eq!(report.attempts[2].planned_backoff_ms, None);
        assert!(report.final_transport_report.is_none());
    }

    #[test]
    fn permanent_error_stops_without_retry() {
        let mut executor = ScriptedRetryExecutor::new(vec![
            ScriptStep::Permanent("payload-budget-exceeded"),
            ScriptStep::Success,
        ]);

        let report =
            execute_protocol_transport_with_retry(policy(), &bundle(), &mut executor).unwrap();

        assert_eq!(
            report.stop_reason,
            ProtocolTransportRetryStopReason::PermanentError
        );
        assert_eq!(report.attempts.len(), 1);
        assert_eq!(report.attempts[0].planned_backoff_ms, None);
        assert_eq!(executor.index, 1);
    }

    #[test]
    fn kernel_executor_adapter_preserves_recording_executor_success() {
        let bundle = bundle();
        let mut recording = RecordingProtocolTransportExecutor::new();
        let mut adapter = KernelProtocolTransportRetryExecutor::new(&mut recording);

        let report =
            execute_protocol_transport_with_retry(policy(), &bundle, &mut adapter).unwrap();

        assert_eq!(
            report.stop_reason,
            ProtocolTransportRetryStopReason::Succeeded
        );
        assert_eq!(report.attempts[0].ack_sequence, Some(1));
        assert_eq!(recording.len(), 1);
    }

    #[test]
    fn kernel_executor_adapter_maps_invariant_errors_to_permanent() {
        let mut drifted = bundle();
        drifted.grpc_unary.tenant_scope_ref = "tenant:other".into();
        let mut recording = RecordingProtocolTransportExecutor::new();
        let mut adapter = KernelProtocolTransportRetryExecutor::new(&mut recording);

        let report =
            execute_protocol_transport_with_retry(policy(), &drifted, &mut adapter).unwrap();

        assert_eq!(
            report.stop_reason,
            ProtocolTransportRetryStopReason::PermanentError
        );
        assert_eq!(report.attempts.len(), 1);
        assert!(
            report.attempts[0]
                .error_ref
                .as_deref()
                .unwrap()
                .contains("tenant_scope_ref")
        );
        assert!(recording.is_empty());
    }

    #[test]
    fn direct_kernel_recording_executor_still_executes_without_retry_wrapper() {
        let bundle = bundle();
        let mut recording = RecordingProtocolTransportExecutor::new();
        let report = execute_transport_bundle(&mut recording, &bundle).unwrap();
        assert_eq!(report.sequence, 1);
    }
}
