//! Cloud observability kernel (M-CC-M03-P03-IP-003 minimum viable kernel).
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
    pub signal: SignalKind,
    pub max_unique_attribute_combinations: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmissionPlan {
    pub plan_id: String,
    pub signal: SignalKind,
    pub estimated_combinations: u64,
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
}
