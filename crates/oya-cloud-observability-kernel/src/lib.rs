//! Cloud observability kernel (M03-P03-IP-003 minimum viable kernel).
//!
//! Pure I/O-free model for OpenTelemetry signal kinds + cardinality
//! envelopes. Adapters do the wire-level OTLP work; the kernel only
//! enforces (a) signal-kind enumeration is allowlisted and (b) per-
//! signal attribute cardinality stays under a declared envelope.

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum SignalKind {
    Trace,
    Metric,
    Log,
    Profile,
}

impl SignalKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Trace => "trace",
            Self::Metric => "metric",
            Self::Log => "log",
            Self::Profile => "profile",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CardinalityEnvelope {
    // data_class: INTERNAL_ONLY
    pub signal: SignalKind, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub max_unique_attribute_combinations: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmissionPlan {
    // data_class: INTERNAL_ONLY
    pub plan_id: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub signal: SignalKind, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub estimated_combinations: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ObservabilityError {
    EmptyPlanId,
    NoEnvelopeForSignal { signal: SignalKind },
    EnvelopeExceeded { max: u64, estimated: u64 },
}

impl ObservabilityError {
    pub fn message(&self) -> String {
        match self {
            Self::EmptyPlanId => "plan id is empty".to_owned(),
            Self::NoEnvelopeForSignal { signal } => {
                format!("no cardinality envelope declared for {}", signal.name())
            }
            Self::EnvelopeExceeded { max, estimated } => {
                format!("cardinality envelope exceeded: max={max} estimated={estimated}")
            }
        }
    }
}

pub fn admit_plan(
    plan: &EmissionPlan,
    envelopes: &[CardinalityEnvelope],
) -> Result<(), ObservabilityError> {
    if plan.plan_id.is_empty() {
        return Err(ObservabilityError::EmptyPlanId);
    }
    let envelope = envelopes.iter().find(|e| e.signal == plan.signal).ok_or(
        ObservabilityError::NoEnvelopeForSignal {
            signal: plan.signal,
        },
    )?;
    if plan.estimated_combinations > envelope.max_unique_attribute_combinations {
        return Err(ObservabilityError::EnvelopeExceeded {
            max: envelope.max_unique_attribute_combinations,
            estimated: plan.estimated_combinations,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(signal: SignalKind, max: u64) -> CardinalityEnvelope {
        CardinalityEnvelope {
            signal,
            max_unique_attribute_combinations: max,
        }
    }
    fn plan(id: &str, signal: SignalKind, est: u64) -> EmissionPlan {
        EmissionPlan {
            plan_id: id.into(),
            signal,
            estimated_combinations: est,
        }
    }

    #[test]
    fn under_envelope_passes() {
        assert!(
            admit_plan(
                &plan("p1", SignalKind::Metric, 100),
                &[env(SignalKind::Metric, 1000)]
            )
            .is_ok()
        );
    }

    #[test]
    fn over_envelope_rejected() {
        assert!(matches!(
            admit_plan(
                &plan("p1", SignalKind::Metric, 2000),
                &[env(SignalKind::Metric, 1000)]
            ),
            Err(ObservabilityError::EnvelopeExceeded { .. })
        ));
    }

    #[test]
    fn at_envelope_boundary_passes() {
        assert!(
            admit_plan(
                &plan("p1", SignalKind::Metric, 1000),
                &[env(SignalKind::Metric, 1000)]
            )
            .is_ok()
        );
    }

    #[test]
    fn no_envelope_for_signal_rejected() {
        assert!(matches!(
            admit_plan(
                &plan("p1", SignalKind::Log, 10),
                &[env(SignalKind::Metric, 1000)]
            ),
            Err(ObservabilityError::NoEnvelopeForSignal { .. })
        ));
    }

    #[test]
    fn empty_plan_id_rejected() {
        assert!(matches!(
            admit_plan(
                &plan("", SignalKind::Metric, 10),
                &[env(SignalKind::Metric, 1000)]
            ),
            Err(ObservabilityError::EmptyPlanId)
        ));
    }

    #[test]
    fn signal_names_distinct() {
        use std::collections::HashSet;
        let s: HashSet<_> = [
            SignalKind::Trace,
            SignalKind::Metric,
            SignalKind::Log,
            SignalKind::Profile,
        ]
        .iter()
        .map(|k| k.name())
        .collect();
        assert_eq!(s.len(), 4);
    }

    // --- admit_budget tests (co-3) ---
    // These tests reference admit_budget and ObservabilityError::AggregateEnvelopeExceeded
    // which do not exist yet. They will fail to compile until co-1 and co-2 are implemented.

    /// Two plans for the same signal each pass individually but their sum exceeds the envelope.
    /// Expects AggregateEnvelopeExceeded.
    #[test]
    fn aggregate_over_envelope_rejected() {
        // envelope: Metric max=1000
        // plan A: 600 (under 1000 individually)
        // plan B: 600 (under 1000 individually)
        // sum: 1200 > 1000 → should be rejected
        let envelopes = [env(SignalKind::Metric, 1000)];
        let plans = [
            plan("p1", SignalKind::Metric, 600),
            plan("p2", SignalKind::Metric, 600),
        ];
        assert!(matches!(
            admit_budget(&plans, &envelopes),
            Err(ObservabilityError::AggregateEnvelopeExceeded {
                signal: SignalKind::Metric,
                max: 1000,
                aggregate: 1200,
            })
        ));
    }

    /// Two plans for the same signal whose sum equals the envelope exactly — must pass.
    #[test]
    fn aggregate_at_boundary_passes() {
        // envelope: Metric max=1000
        // plan A: 500, plan B: 500 → sum = 1000 = max → Ok
        let envelopes = [env(SignalKind::Metric, 1000)];
        let plans = [
            plan("p1", SignalKind::Metric, 500),
            plan("p2", SignalKind::Metric, 500),
        ];
        assert!(admit_budget(&plans, &envelopes).is_ok());
    }

    /// A plan for a signal that has no declared envelope returns NoEnvelopeForSignal.
    #[test]
    fn aggregate_no_envelope_for_signal_rejected() {
        // only a Metric envelope; plan uses Log
        let envelopes = [env(SignalKind::Metric, 1000)];
        let plans = [plan("p1", SignalKind::Log, 10)];
        assert!(matches!(
            admit_budget(&plans, &envelopes),
            Err(ObservabilityError::NoEnvelopeForSignal {
                signal: SignalKind::Log,
            })
        ));
    }

    /// Two plans with u64::MAX estimated_combinations must not panic (saturating_add);
    /// the saturated sum still exceeds any realistic envelope, so AggregateEnvelopeExceeded
    /// is returned.
    #[test]
    fn aggregate_saturating_add_no_panic() {
        // saturating_add(u64::MAX, u64::MAX) = u64::MAX (no overflow/panic)
        // u64::MAX > 1000 → AggregateEnvelopeExceeded
        let envelopes = [env(SignalKind::Trace, 1000)];
        let plans = [
            plan("p1", SignalKind::Trace, u64::MAX),
            plan("p2", SignalKind::Trace, u64::MAX),
        ];
        assert!(matches!(
            admit_budget(&plans, &envelopes),
            Err(ObservabilityError::AggregateEnvelopeExceeded { .. })
        ));
    }

    /// AggregateEnvelopeExceeded message is stable, low-cardinality, and data-class-safe.
    /// Contains signal name + the two integers; no plan IDs or attribute values.
    #[test]
    fn aggregate_envelope_exceeded_message_format() {
        let err = ObservabilityError::AggregateEnvelopeExceeded {
            signal: SignalKind::Metric,
            max: 1000,
            aggregate: 1200,
        };
        let msg = err.message();
        assert!(
            msg.contains("aggregate cardinality envelope exceeded"),
            "message must contain stable prefix: {msg}"
        );
        assert!(msg.contains("signal=metric"), "message must contain signal name: {msg}");
        assert!(msg.contains("max=1000"), "message must contain max value: {msg}");
        assert!(msg.contains("aggregate=1200"), "message must contain aggregate value: {msg}");
        // must NOT leak plan IDs or dynamic payload
        assert!(!msg.contains("p1"), "message must not contain plan id: {msg}");
    }
}
