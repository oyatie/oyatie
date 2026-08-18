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
    NoEnvelopeForSignal {
        signal: SignalKind,
    },
    EnvelopeExceeded {
        max: u64,
        estimated: u64,
    },
    // data_class: INTERNAL_ONLY — signal name + numeric thresholds only; no payload
    AggregateEnvelopeExceeded {
        signal: SignalKind,
        max: u64,
        aggregate: u64,
    },
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
            Self::AggregateEnvelopeExceeded {
                signal,
                max,
                aggregate,
            } => {
                format!(
                    "aggregate cardinality envelope exceeded: signal={} max={} aggregate={}",
                    signal.name(),
                    max,
                    aggregate,
                )
            }
        }
    }
}

/// Per-signal headroom report entry produced by [`budget_headroom`].
///
/// `remaining` is the number of attribute-combination slots still available before the
/// declared envelope is hit.  `over_budget` is `true` when the aggregate already exceeds
/// the envelope maximum.  Both fields use saturating arithmetic so neither can wrap.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalHeadroom {
    // data_class: INTERNAL_ONLY
    pub signal: SignalKind, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub max: u64, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub aggregate: u64, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub remaining: u64, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub over_budget: bool, // data_class: INTERNAL_ONLY
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

/// Aggregate multi-plan cardinality rollup admission.
///
/// For each plan the existing `EmptyPlanId` and `NoEnvelopeForSignal` guards are applied
/// first; the function returns the first such per-plan error encountered.  Plans that
/// pass per-plan guards are grouped by [`SignalKind`] and their `estimated_combinations`
/// values are summed per signal using [`u64::saturating_add`] (overflow-safe).  If any
/// per-signal aggregate exceeds `max_unique_attribute_combinations` from the matching
/// [`CardinalityEnvelope`], [`ObservabilityError::AggregateEnvelopeExceeded`] is returned.
///
/// Pure: no side effects, no I/O, no async.
pub fn admit_budget(
    plans: &[EmissionPlan],
    envelopes: &[CardinalityEnvelope],
) -> Result<(), ObservabilityError> {
    // Phase 1: per-plan guard pass (EmptyPlanId + NoEnvelopeForSignal).
    // We do not check single-plan EnvelopeExceeded here — that is admit_plan's concern.
    // We only need the two structural guards so that malformed plans are rejected early.
    for plan in plans {
        if plan.plan_id.is_empty() {
            return Err(ObservabilityError::EmptyPlanId);
        }
        if !envelopes.iter().any(|e| e.signal == plan.signal) {
            return Err(ObservabilityError::NoEnvelopeForSignal {
                signal: plan.signal,
            });
        }
    }

    // Phase 2: accumulate estimated_combinations per signal using saturating_add.
    // We use a small fixed-size array keyed by SignalKind ordinal to avoid heap allocation.
    // Order matches the SignalKind variants: Trace=0, Metric=1, Log=2, Profile=3.
    const N: usize = 4;
    let signal_index = |s: SignalKind| -> usize {
        match s {
            SignalKind::Trace => 0,
            SignalKind::Metric => 1,
            SignalKind::Log => 2,
            SignalKind::Profile => 3,
        }
    };
    let index_signal = |i: usize| -> SignalKind {
        match i {
            0 => SignalKind::Trace,
            1 => SignalKind::Metric,
            2 => SignalKind::Log,
            _ => SignalKind::Profile,
        }
    };

    let mut aggregates = [0u64; N];
    let mut seen = [false; N];
    for plan in plans {
        let idx = signal_index(plan.signal);
        aggregates[idx] = aggregates[idx].saturating_add(plan.estimated_combinations);
        seen[idx] = true;
    }

    // Phase 3: reject any signal whose aggregate exceeds its envelope.
    for i in 0..N {
        if !seen[i] {
            continue;
        }
        let sig = index_signal(i);
        // Safety: envelope presence was verified in phase 1 for every plan, so every
        // seen signal has a matching envelope.
        let envelope = envelopes.iter().find(|e| e.signal == sig).unwrap();
        if aggregates[i] > envelope.max_unique_attribute_combinations {
            return Err(ObservabilityError::AggregateEnvelopeExceeded {
                signal: sig,
                max: envelope.max_unique_attribute_combinations,
                aggregate: aggregates[i],
            });
        }
    }

    Ok(())
}

/// Per-signal cardinality headroom report — a non-throwing companion to [`admit_budget`].
///
/// Applies the same Phase-1 structural guards (EmptyPlanId, NoEnvelopeForSignal) and the
/// same Phase-2 saturating accumulation as `admit_budget`.  Instead of returning an error
/// on aggregate overage, Phase 3 computes `remaining = max.saturating_sub(aggregate)` and
/// `over_budget = aggregate > max` for each seen signal and returns the results as a
/// `Vec<SignalHeadroom>` in deterministic [`SignalKind`] ordinal order.
///
/// Pure: no side effects, no I/O, no async.
pub fn budget_headroom(
    plans: &[EmissionPlan],
    envelopes: &[CardinalityEnvelope],
) -> Result<Vec<SignalHeadroom>, ObservabilityError> {
    // Phase 1: per-plan structural guards (EmptyPlanId + NoEnvelopeForSignal).
    for plan in plans {
        if plan.plan_id.is_empty() {
            return Err(ObservabilityError::EmptyPlanId);
        }
        if !envelopes.iter().any(|e| e.signal == plan.signal) {
            return Err(ObservabilityError::NoEnvelopeForSignal {
                signal: plan.signal,
            });
        }
    }

    // Phase 2: accumulate estimated_combinations per signal using saturating_add.
    // Fixed-size array keyed by SignalKind ordinal: Trace=0, Metric=1, Log=2, Profile=3.
    const N: usize = 4;
    let signal_index = |s: SignalKind| -> usize {
        match s {
            SignalKind::Trace => 0,
            SignalKind::Metric => 1,
            SignalKind::Log => 2,
            SignalKind::Profile => 3,
        }
    };
    let index_signal = |i: usize| -> SignalKind {
        match i {
            0 => SignalKind::Trace,
            1 => SignalKind::Metric,
            2 => SignalKind::Log,
            _ => SignalKind::Profile,
        }
    };

    let mut aggregates = [0u64; N];
    let mut seen = [false; N];
    for plan in plans {
        let idx = signal_index(plan.signal);
        aggregates[idx] = aggregates[idx].saturating_add(plan.estimated_combinations);
        seen[idx] = true;
    }

    // Phase 3: build headroom report for each seen signal in ordinal order.
    let mut report = Vec::new();
    for i in 0..N {
        if !seen[i] {
            continue;
        }
        let sig = index_signal(i);
        // Safety: envelope presence was verified in phase 1 for every plan, so every
        // seen signal has a matching envelope.
        let envelope = envelopes.iter().find(|e| e.signal == sig).unwrap();
        let max = envelope.max_unique_attribute_combinations;
        let aggregate = aggregates[i];
        let remaining = max.saturating_sub(aggregate);
        let over_budget = aggregate > max;
        report.push(SignalHeadroom {
            signal: sig,
            max,
            aggregate,
            remaining,
            over_budget,
        });
    }

    Ok(report)
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
        assert!(
            msg.contains("signal=metric"),
            "message must contain signal name: {msg}"
        );
        assert!(
            msg.contains("max=1000"),
            "message must contain max value: {msg}"
        );
        assert!(
            msg.contains("aggregate=1200"),
            "message must contain aggregate value: {msg}"
        );
        // must NOT leak plan IDs or dynamic payload
        assert!(
            !msg.contains("p1"),
            "message must not contain plan id: {msg}"
        );
    }

    // --- budget_headroom tests ---

    /// Under-budget: aggregate < max → remaining = max - aggregate, over_budget = false.
    #[test]
    fn headroom_under_budget() {
        let envelopes = [env(SignalKind::Metric, 1000)];
        let plans = [plan("p1", SignalKind::Metric, 300)];
        let report = budget_headroom(&plans, &envelopes).unwrap();
        assert_eq!(report.len(), 1);
        assert_eq!(
            report[0],
            SignalHeadroom {
                signal: SignalKind::Metric,
                max: 1000,
                aggregate: 300,
                remaining: 700,
                over_budget: false,
            }
        );
    }

    /// At boundary: aggregate == max → remaining = 0, over_budget = false.
    #[test]
    fn headroom_at_boundary() {
        let envelopes = [env(SignalKind::Log, 500)];
        let plans = [plan("p1", SignalKind::Log, 500)];
        let report = budget_headroom(&plans, &envelopes).unwrap();
        assert_eq!(report.len(), 1);
        assert_eq!(
            report[0],
            SignalHeadroom {
                signal: SignalKind::Log,
                max: 500,
                aggregate: 500,
                remaining: 0,
                over_budget: false,
            }
        );
    }

    /// Over budget: aggregate > max → remaining = 0, over_budget = true.
    #[test]
    fn headroom_over_budget() {
        let envelopes = [env(SignalKind::Trace, 200)];
        let plans = [
            plan("p1", SignalKind::Trace, 150),
            plan("p2", SignalKind::Trace, 100),
        ];
        let report = budget_headroom(&plans, &envelopes).unwrap();
        assert_eq!(report.len(), 1);
        assert_eq!(
            report[0],
            SignalHeadroom {
                signal: SignalKind::Trace,
                max: 200,
                aggregate: 250,
                remaining: 0,
                over_budget: true,
            }
        );
    }

    /// Saturating arithmetic: two plans each with u64::MAX must not panic.
    /// aggregate saturates to u64::MAX; over_budget = true (u64::MAX > any reasonable max).
    #[test]
    fn headroom_saturating_no_panic() {
        let envelopes = [env(SignalKind::Profile, 1000)];
        let plans = [
            plan("p1", SignalKind::Profile, u64::MAX),
            plan("p2", SignalKind::Profile, u64::MAX),
        ];
        let report = budget_headroom(&plans, &envelopes).unwrap();
        assert_eq!(report.len(), 1);
        assert_eq!(report[0].aggregate, u64::MAX); // saturated
        assert_eq!(report[0].remaining, 0);
        assert!(report[0].over_budget);
    }

    /// Multi-signal: Trace + Metric plans produce two entries in deterministic ordinal order.
    #[test]
    fn headroom_multi_signal_deterministic_order() {
        let envelopes = [env(SignalKind::Metric, 800), env(SignalKind::Trace, 500)];
        let plans = [
            plan("p1", SignalKind::Metric, 300),
            plan("p2", SignalKind::Trace, 100),
        ];
        let report = budget_headroom(&plans, &envelopes).unwrap();
        assert_eq!(report.len(), 2);
        // Trace ordinal=0 comes before Metric ordinal=1
        assert_eq!(report[0].signal, SignalKind::Trace);
        assert_eq!(report[0].aggregate, 100);
        assert_eq!(report[0].remaining, 400);
        assert!(!report[0].over_budget);
        assert_eq!(report[1].signal, SignalKind::Metric);
        assert_eq!(report[1].aggregate, 300);
        assert_eq!(report[1].remaining, 500);
        assert!(!report[1].over_budget);
    }

    /// Structural guard: EmptyPlanId is rejected before any accumulation.
    #[test]
    fn headroom_empty_plan_id_rejected() {
        let envelopes = [env(SignalKind::Metric, 1000)];
        let plans = [plan("", SignalKind::Metric, 100)];
        assert!(matches!(
            budget_headroom(&plans, &envelopes),
            Err(ObservabilityError::EmptyPlanId)
        ));
    }

    /// Structural guard: NoEnvelopeForSignal is rejected before any accumulation.
    #[test]
    fn headroom_no_envelope_for_signal_rejected() {
        let envelopes = [env(SignalKind::Metric, 1000)];
        let plans = [plan("p1", SignalKind::Trace, 100)];
        assert!(matches!(
            budget_headroom(&plans, &envelopes),
            Err(ObservabilityError::NoEnvelopeForSignal {
                signal: SignalKind::Trace,
            })
        ));
    }
}
