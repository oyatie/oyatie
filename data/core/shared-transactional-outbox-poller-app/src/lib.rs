//! Bounded transactional outbox poller policy seam.
//!
//! This crate owns deterministic scheduler policy for repeated outbox worker
//! cycles: bounded ticks, idle shutdown, consecutive-error shutdown, and capped
//! error backoff. It intentionally performs no sleeping, no process spawning, no
//! live broker/gRPC I/O, and no database I/O; runtime binaries wire those later.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use shared_transactional_outbox_worker_app::OutboxWorkerCycleReport;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OutboxPollerError {
    InvalidConfig { field: &'static str },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxPollerRunnerError {
    pub error_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OutboxPollerConfig {
    pub max_ticks: u16,                 // data_class: INTERNAL_ONLY
    pub idle_shutdown_after_ticks: u16, // data_class: INTERNAL_ONLY
    pub max_consecutive_errors: u16,    // data_class: INTERNAL_ONLY
    pub base_backoff_ms: u64,           // data_class: INTERNAL_ONLY
    pub max_backoff_ms: u64,            // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutboxPollerTickOutcome {
    Work,
    Idle,
    Error,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutboxPollerStopReason {
    MaxTicks,
    IdleShutdown,
    ErrorShutdown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxPollerTickReport {
    pub tick: u16,                        // data_class: INTERNAL_ONLY
    pub outcome: OutboxPollerTickOutcome, // data_class: INTERNAL_ONLY
    pub claimed_count: usize,             // data_class: INTERNAL_ONLY
    pub published_count: usize,           // data_class: INTERNAL_ONLY
    pub dead_letter_count: usize,         // data_class: INTERNAL_ONLY
    pub consecutive_idle_ticks: u16,      // data_class: INTERNAL_ONLY
    pub consecutive_error_ticks: u16,     // data_class: INTERNAL_ONLY
    pub planned_delay_ms: u64,            // data_class: INTERNAL_ONLY
    pub error_ref: Option<String>,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxPollerRunReport {
    pub stop_reason: OutboxPollerStopReason, // data_class: INTERNAL_ONLY
    pub ticks: Vec<OutboxPollerTickReport>,  // data_class: INTERNAL_ONLY
    pub total_claimed_count: usize,          // data_class: INTERNAL_ONLY
    pub total_published_count: usize,        // data_class: INTERNAL_ONLY
    pub total_dead_letter_count: usize,      // data_class: INTERNAL_ONLY
    pub total_error_count: usize,            // data_class: INTERNAL_ONLY
}

pub trait OutboxPollerCycleRunner {
    fn run_worker_cycle(&mut self) -> Result<OutboxWorkerCycleReport, OutboxPollerRunnerError>;
}

impl Default for OutboxPollerConfig {
    fn default() -> Self {
        Self {
            max_ticks: 64,
            idle_shutdown_after_ticks: 8,
            max_consecutive_errors: 4,
            base_backoff_ms: 250,
            max_backoff_ms: 5_000,
        }
    }
}

impl OutboxPollerRunnerError {
    pub fn new(error_ref: impl Into<String>) -> Result<Self, OutboxPollerError> {
        let error_ref = error_ref.into();
        if error_ref.trim().is_empty() {
            return Err(OutboxPollerError::InvalidConfig { field: "error_ref" });
        }
        Ok(Self { error_ref })
    }
}

impl OutboxPollerConfig {
    pub fn validate(&self) -> Result<(), OutboxPollerError> {
        if self.max_ticks == 0 {
            return Err(OutboxPollerError::InvalidConfig { field: "max_ticks" });
        }
        if self.idle_shutdown_after_ticks == 0 {
            return Err(OutboxPollerError::InvalidConfig {
                field: "idle_shutdown_after_ticks",
            });
        }
        if self.max_consecutive_errors == 0 {
            return Err(OutboxPollerError::InvalidConfig {
                field: "max_consecutive_errors",
            });
        }
        if self.base_backoff_ms == 0 {
            return Err(OutboxPollerError::InvalidConfig {
                field: "base_backoff_ms",
            });
        }
        if self.max_backoff_ms < self.base_backoff_ms {
            return Err(OutboxPollerError::InvalidConfig {
                field: "max_backoff_ms",
            });
        }
        Ok(())
    }
}

pub fn run_bounded_outbox_poller(
    config: OutboxPollerConfig,
    runner: &mut dyn OutboxPollerCycleRunner,
) -> Result<OutboxPollerRunReport, OutboxPollerError> {
    config.validate()?;
    let mut ticks = Vec::with_capacity(usize::from(config.max_ticks));
    let mut consecutive_idle_ticks = 0_u16;
    let mut consecutive_error_ticks = 0_u16;
    let mut total_claimed_count = 0_usize;
    let mut total_published_count = 0_usize;
    let mut total_dead_letter_count = 0_usize;
    let mut total_error_count = 0_usize;
    let mut stop_reason = OutboxPollerStopReason::MaxTicks;

    for tick in 1..=config.max_ticks {
        match runner.run_worker_cycle() {
            Ok(cycle) => {
                consecutive_error_ticks = 0;
                total_claimed_count += cycle.claimed_count;
                total_published_count += cycle.published_count;
                total_dead_letter_count += cycle.dead_letter_count;
                let outcome = if cycle.claimed_count == 0 {
                    consecutive_idle_ticks = consecutive_idle_ticks.saturating_add(1);
                    OutboxPollerTickOutcome::Idle
                } else {
                    consecutive_idle_ticks = 0;
                    OutboxPollerTickOutcome::Work
                };
                ticks.push(OutboxPollerTickReport {
                    tick,
                    outcome,
                    claimed_count: cycle.claimed_count,
                    published_count: cycle.published_count,
                    dead_letter_count: cycle.dead_letter_count,
                    consecutive_idle_ticks,
                    consecutive_error_ticks,
                    planned_delay_ms: config.base_backoff_ms,
                    error_ref: None,
                });
                if consecutive_idle_ticks >= config.idle_shutdown_after_ticks {
                    stop_reason = OutboxPollerStopReason::IdleShutdown;
                    break;
                }
            }
            Err(error) => {
                consecutive_error_ticks = consecutive_error_ticks.saturating_add(1);
                consecutive_idle_ticks = 0;
                total_error_count += 1;
                ticks.push(OutboxPollerTickReport {
                    tick,
                    outcome: OutboxPollerTickOutcome::Error,
                    claimed_count: 0,
                    published_count: 0,
                    dead_letter_count: 0,
                    consecutive_idle_ticks,
                    consecutive_error_ticks,
                    planned_delay_ms: planned_error_backoff_ms(&config, consecutive_error_ticks),
                    error_ref: Some(error.error_ref),
                });
                if consecutive_error_ticks >= config.max_consecutive_errors {
                    stop_reason = OutboxPollerStopReason::ErrorShutdown;
                    break;
                }
            }
        }
    }

    Ok(OutboxPollerRunReport {
        stop_reason,
        ticks,
        total_claimed_count,
        total_published_count,
        total_dead_letter_count,
        total_error_count,
    })
}

#[must_use]
pub fn planned_error_backoff_ms(config: &OutboxPollerConfig, consecutive_errors: u16) -> u64 {
    let exponent = consecutive_errors.saturating_sub(1).min(16);
    let multiplier = 1_u64.checked_shl(u32::from(exponent)).unwrap_or(u64::MAX);
    config
        .base_backoff_ms
        .saturating_mul(multiplier)
        .min(config.max_backoff_ms)
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared_transactional_outbox_kernel::BackboneOutboxTable;

    #[derive(Clone, Debug)]
    enum ScriptStep {
        Cycle(OutboxWorkerCycleReport),
        Error(&'static str),
    }

    #[derive(Clone, Debug)]
    struct ScriptedRunner {
        steps: Vec<ScriptStep>,
        index: usize,
    }

    impl ScriptedRunner {
        fn new(steps: Vec<ScriptStep>) -> Self {
            Self { steps, index: 0 }
        }
    }

    impl OutboxPollerCycleRunner for ScriptedRunner {
        fn run_worker_cycle(&mut self) -> Result<OutboxWorkerCycleReport, OutboxPollerRunnerError> {
            let step = self
                .steps
                .get(self.index)
                .cloned()
                .unwrap_or_else(|| ScriptStep::Cycle(cycle_report(0, 0, 0)));
            self.index += 1;
            match step {
                ScriptStep::Cycle(report) => Ok(report),
                ScriptStep::Error(error_ref) => Err(OutboxPollerRunnerError::new(error_ref)
                    .expect("scripted runner error refs are non-empty")),
            }
        }
    }

    fn config() -> OutboxPollerConfig {
        OutboxPollerConfig {
            max_ticks: 5,
            idle_shutdown_after_ticks: 3,
            max_consecutive_errors: 3,
            base_backoff_ms: 100,
            max_backoff_ms: 250,
        }
    }

    fn cycle_report(
        claimed_count: usize,
        published_count: usize,
        dead_letter_count: usize,
    ) -> OutboxWorkerCycleReport {
        OutboxWorkerCycleReport {
            table: BackboneOutboxTable::MessengerMessageStream,
            table_name: "messenger_message_stream.protocol_outbox_events",
            tenant_scope_ref: "tenant:t".into(),
            worker_ref: "worker:a".into(),
            claimed_count,
            published_count,
            dead_letter_count,
            event_reports: Vec::new(),
        }
    }

    #[test]
    fn config_rejects_zero_values_and_inverted_backoff() {
        assert_eq!(
            OutboxPollerConfig {
                max_ticks: 0,
                ..config()
            }
            .validate(),
            Err(OutboxPollerError::InvalidConfig { field: "max_ticks" })
        );
        assert_eq!(
            OutboxPollerConfig {
                base_backoff_ms: 500,
                max_backoff_ms: 250,
                ..config()
            }
            .validate(),
            Err(OutboxPollerError::InvalidConfig {
                field: "max_backoff_ms",
            })
        );
        assert_eq!(
            OutboxPollerRunnerError::new(" "),
            Err(OutboxPollerError::InvalidConfig { field: "error_ref" })
        );
    }

    #[test]
    fn idle_shutdown_stops_after_configured_consecutive_idle_ticks() {
        let mut runner = ScriptedRunner::new(vec![
            ScriptStep::Cycle(cycle_report(0, 0, 0)),
            ScriptStep::Cycle(cycle_report(0, 0, 0)),
            ScriptStep::Cycle(cycle_report(0, 0, 0)),
            ScriptStep::Cycle(cycle_report(1, 1, 0)),
        ]);

        let report = run_bounded_outbox_poller(config(), &mut runner).unwrap();

        assert_eq!(report.stop_reason, OutboxPollerStopReason::IdleShutdown);
        assert_eq!(report.ticks.len(), 3);
        assert_eq!(report.ticks[2].consecutive_idle_ticks, 3);
        assert_eq!(report.total_claimed_count, 0);
    }

    #[test]
    fn work_resets_idle_and_error_streaks_until_max_ticks() {
        let mut runner = ScriptedRunner::new(vec![
            ScriptStep::Cycle(cycle_report(0, 0, 0)),
            ScriptStep::Error("transient:timeout"),
            ScriptStep::Cycle(cycle_report(2, 1, 1)),
            ScriptStep::Cycle(cycle_report(0, 0, 0)),
            ScriptStep::Cycle(cycle_report(1, 1, 0)),
        ]);

        let report = run_bounded_outbox_poller(config(), &mut runner).unwrap();

        assert_eq!(report.stop_reason, OutboxPollerStopReason::MaxTicks);
        assert_eq!(report.ticks.len(), 5);
        assert_eq!(report.ticks[2].outcome, OutboxPollerTickOutcome::Work);
        assert_eq!(report.ticks[2].consecutive_error_ticks, 0);
        assert_eq!(report.ticks[2].consecutive_idle_ticks, 0);
        assert_eq!(report.total_claimed_count, 3);
        assert_eq!(report.total_published_count, 2);
        assert_eq!(report.total_dead_letter_count, 1);
        assert_eq!(report.total_error_count, 1);
    }

    #[test]
    fn consecutive_errors_use_capped_backoff_and_error_shutdown() {
        let mut runner = ScriptedRunner::new(vec![
            ScriptStep::Error("transient:one"),
            ScriptStep::Error("transient:two"),
            ScriptStep::Error("transient:three"),
            ScriptStep::Cycle(cycle_report(1, 1, 0)),
        ]);

        let report = run_bounded_outbox_poller(config(), &mut runner).unwrap();

        assert_eq!(report.stop_reason, OutboxPollerStopReason::ErrorShutdown);
        assert_eq!(report.ticks.len(), 3);
        assert_eq!(report.ticks[0].planned_delay_ms, 100);
        assert_eq!(report.ticks[1].planned_delay_ms, 200);
        assert_eq!(report.ticks[2].planned_delay_ms, 250);
        assert_eq!(report.total_error_count, 3);
        assert_eq!(
            report.ticks[2].error_ref.as_deref(),
            Some("transient:three")
        );
    }

    #[test]
    fn explicit_backoff_helper_caps_without_overflow() {
        let cfg = config();

        assert_eq!(planned_error_backoff_ms(&cfg, 1), 100);
        assert_eq!(planned_error_backoff_ms(&cfg, 2), 200);
        assert_eq!(planned_error_backoff_ms(&cfg, 3), 250);
        assert_eq!(planned_error_backoff_ms(&cfg, u16::MAX), 250);
    }
}
