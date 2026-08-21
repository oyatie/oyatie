//! Tokio runtime seam for the bounded transactional outbox poller.
//!
//! This crate adds real asynchronous delay wiring around the deterministic
//! poller policy. It does not open a database pool, publish to a broker, call a
//! gRPC server, spawn a daemon, install a supervisor, or prove production
//! delivery guarantees.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::{future::Future, pin::Pin, time::Duration};

use shared_transactional_outbox_poller_app::{
    OutboxPollerConfig, OutboxPollerError, OutboxPollerRunReport, OutboxPollerRunnerError,
    OutboxPollerStopReason, OutboxPollerTickOutcome, OutboxPollerTickReport,
    planned_error_backoff_ms,
};
use shared_transactional_outbox_worker_app::OutboxWorkerCycleReport;

pub type BoxPollerFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait AsyncOutboxPollerCycleRunner {
    fn run_worker_cycle<'a>(
        &'a mut self,
    ) -> BoxPollerFuture<'a, Result<OutboxWorkerCycleReport, OutboxPollerRunnerError>>;
}

pub trait AsyncOutboxPollerSleeper {
    fn sleep_ms<'a>(&'a mut self, delay_ms: u64) -> BoxPollerFuture<'a, ()>;
}

pub trait AsyncOutboxShutdownSignal {
    fn should_stop<'a>(&'a mut self, completed_epochs: u16) -> BoxPollerFuture<'a, bool>;
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TokioOutboxPollerSleeper;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NeverStopOutboxShutdownSignal;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OutboxPollerServiceConfig {
    pub max_epochs: u16,                   // data_class: INTERNAL_ONLY
    pub epoch_pause_ms: u64,               // data_class: INTERNAL_ONLY
    pub poller_config: OutboxPollerConfig, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutboxPollerServiceStopReason {
    MaxEpochs,
    ShutdownRequested,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxPollerEpochReport {
    pub epoch: u16,                           // data_class: INTERNAL_ONLY
    pub poller_report: OutboxPollerRunReport, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxPollerServiceReport {
    pub stop_reason: OutboxPollerServiceStopReason, // data_class: INTERNAL_ONLY
    pub epochs: Vec<OutboxPollerEpochReport>,       // data_class: INTERNAL_ONLY
    pub total_ticks: usize,                         // data_class: INTERNAL_ONLY
    pub total_claimed_count: usize,                 // data_class: INTERNAL_ONLY
    pub total_published_count: usize,               // data_class: INTERNAL_ONLY
    pub total_dead_letter_count: usize,             // data_class: INTERNAL_ONLY
    pub total_error_count: usize,                   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutboxSupervisorLifecycleEventKind {
    Starting,
    Ready,
    ShutdownRequested,
    Stopped,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxSupervisorConfig {
    pub service_ref: String,                       // data_class: INTERNAL_ONLY
    pub worker_ref: String,                        // data_class: INTERNAL_ONLY
    pub readiness_timeout_ms: u64,                 // data_class: INTERNAL_ONLY
    pub shutdown_grace_ms: u64,                    // data_class: INTERNAL_ONLY
    pub service_config: OutboxPollerServiceConfig, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxSupervisorLifecycleEvent {
    pub sequence: u16,                                      // data_class: INTERNAL_ONLY
    pub kind: OutboxSupervisorLifecycleEventKind,           // data_class: INTERNAL_ONLY
    pub service_ref: String,                                // data_class: INTERNAL_ONLY
    pub worker_ref: String,                                 // data_class: INTERNAL_ONLY
    pub completed_epochs: u16,                              // data_class: INTERNAL_ONLY
    pub stop_reason: Option<OutboxPollerServiceStopReason>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxSupervisorLifecycleReport {
    pub service_ref: String,                         // data_class: INTERNAL_ONLY
    pub worker_ref: String,                          // data_class: INTERNAL_ONLY
    pub readiness_timeout_ms: u64,                   // data_class: INTERNAL_ONLY
    pub shutdown_grace_ms: u64,                      // data_class: INTERNAL_ONLY
    pub service_report: OutboxPollerServiceReport,   // data_class: INTERNAL_ONLY
    pub events: Vec<OutboxSupervisorLifecycleEvent>, // data_class: INTERNAL_ONLY
}

impl AsyncOutboxPollerSleeper for TokioOutboxPollerSleeper {
    fn sleep_ms<'a>(&'a mut self, delay_ms: u64) -> BoxPollerFuture<'a, ()> {
        Box::pin(async move {
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        })
    }
}

impl Default for OutboxPollerServiceConfig {
    fn default() -> Self {
        Self {
            max_epochs: 1,
            epoch_pause_ms: 1_000,
            poller_config: OutboxPollerConfig::default(),
        }
    }
}

impl OutboxPollerServiceConfig {
    pub fn validate(&self) -> Result<(), OutboxPollerError> {
        if self.max_epochs == 0 {
            return Err(OutboxPollerError::InvalidConfig {
                field: "max_epochs",
            });
        }
        if self.epoch_pause_ms == 0 {
            return Err(OutboxPollerError::InvalidConfig {
                field: "epoch_pause_ms",
            });
        }
        self.poller_config.validate()
    }
}

impl Default for OutboxSupervisorConfig {
    fn default() -> Self {
        Self {
            service_ref: "outbox-service".to_string(),
            worker_ref: "outbox-worker".to_string(),
            readiness_timeout_ms: 5_000,
            shutdown_grace_ms: 30_000,
            service_config: OutboxPollerServiceConfig::default(),
        }
    }
}

impl OutboxSupervisorConfig {
    pub fn validate(&self) -> Result<(), OutboxPollerError> {
        if self.service_ref.trim().is_empty() {
            return Err(OutboxPollerError::InvalidConfig {
                field: "service_ref",
            });
        }
        if self.worker_ref.trim().is_empty() {
            return Err(OutboxPollerError::InvalidConfig {
                field: "worker_ref",
            });
        }
        if self.readiness_timeout_ms == 0 {
            return Err(OutboxPollerError::InvalidConfig {
                field: "readiness_timeout_ms",
            });
        }
        if self.shutdown_grace_ms == 0 {
            return Err(OutboxPollerError::InvalidConfig {
                field: "shutdown_grace_ms",
            });
        }
        self.service_config.validate()
    }
}

impl AsyncOutboxShutdownSignal for NeverStopOutboxShutdownSignal {
    fn should_stop<'a>(&'a mut self, _completed_epochs: u16) -> BoxPollerFuture<'a, bool> {
        Box::pin(async { false })
    }
}

pub async fn run_tokio_bounded_outbox_poller(
    config: OutboxPollerConfig,
    runner: &mut dyn AsyncOutboxPollerCycleRunner,
    sleeper: &mut dyn AsyncOutboxPollerSleeper,
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
        let tick_report = match runner.run_worker_cycle().await {
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
                OutboxPollerTickReport {
                    tick,
                    outcome,
                    claimed_count: cycle.claimed_count,
                    published_count: cycle.published_count,
                    dead_letter_count: cycle.dead_letter_count,
                    consecutive_idle_ticks,
                    consecutive_error_ticks,
                    planned_delay_ms: config.base_backoff_ms,
                    error_ref: None,
                }
            }
            Err(error) => {
                consecutive_error_ticks = consecutive_error_ticks.saturating_add(1);
                consecutive_idle_ticks = 0;
                total_error_count += 1;
                OutboxPollerTickReport {
                    tick,
                    outcome: OutboxPollerTickOutcome::Error,
                    claimed_count: 0,
                    published_count: 0,
                    dead_letter_count: 0,
                    consecutive_idle_ticks,
                    consecutive_error_ticks,
                    planned_delay_ms: planned_error_backoff_ms(&config, consecutive_error_ticks),
                    error_ref: Some(error.error_ref),
                }
            }
        };

        let should_stop = if tick_report.outcome == OutboxPollerTickOutcome::Idle
            && tick_report.consecutive_idle_ticks >= config.idle_shutdown_after_ticks
        {
            stop_reason = OutboxPollerStopReason::IdleShutdown;
            true
        } else if tick_report.outcome == OutboxPollerTickOutcome::Error
            && tick_report.consecutive_error_ticks >= config.max_consecutive_errors
        {
            stop_reason = OutboxPollerStopReason::ErrorShutdown;
            true
        } else if tick == config.max_ticks {
            stop_reason = OutboxPollerStopReason::MaxTicks;
            true
        } else {
            false
        };
        let planned_delay_ms = tick_report.planned_delay_ms;
        ticks.push(tick_report);
        if should_stop {
            break;
        }
        sleeper.sleep_ms(planned_delay_ms).await;
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

pub async fn run_supervised_tokio_outbox_service(
    config: OutboxSupervisorConfig,
    runner: &mut dyn AsyncOutboxPollerCycleRunner,
    sleeper: &mut dyn AsyncOutboxPollerSleeper,
    shutdown_signal: &mut dyn AsyncOutboxShutdownSignal,
) -> Result<OutboxSupervisorLifecycleReport, OutboxPollerError> {
    config.validate()?;
    let mut events = vec![
        lifecycle_event(
            1,
            OutboxSupervisorLifecycleEventKind::Starting,
            &config,
            0,
            None,
        ),
        lifecycle_event(
            2,
            OutboxSupervisorLifecycleEventKind::Ready,
            &config,
            0,
            None,
        ),
    ];

    let service_report =
        run_tokio_outbox_service_loop(config.service_config, runner, sleeper, shutdown_signal)
            .await?;
    let completed_epochs = u16::try_from(service_report.epochs.len()).unwrap_or(u16::MAX);
    let mut next_sequence = 3;
    if service_report.stop_reason == OutboxPollerServiceStopReason::ShutdownRequested {
        events.push(lifecycle_event(
            next_sequence,
            OutboxSupervisorLifecycleEventKind::ShutdownRequested,
            &config,
            completed_epochs,
            Some(service_report.stop_reason),
        ));
        next_sequence += 1;
    }
    events.push(lifecycle_event(
        next_sequence,
        OutboxSupervisorLifecycleEventKind::Stopped,
        &config,
        completed_epochs,
        Some(service_report.stop_reason),
    ));

    Ok(OutboxSupervisorLifecycleReport {
        service_ref: config.service_ref,
        worker_ref: config.worker_ref,
        readiness_timeout_ms: config.readiness_timeout_ms,
        shutdown_grace_ms: config.shutdown_grace_ms,
        service_report,
        events,
    })
}

fn lifecycle_event(
    sequence: u16,
    kind: OutboxSupervisorLifecycleEventKind,
    config: &OutboxSupervisorConfig,
    completed_epochs: u16,
    stop_reason: Option<OutboxPollerServiceStopReason>,
) -> OutboxSupervisorLifecycleEvent {
    OutboxSupervisorLifecycleEvent {
        sequence,
        kind,
        service_ref: config.service_ref.clone(),
        worker_ref: config.worker_ref.clone(),
        completed_epochs,
        stop_reason,
    }
}

pub async fn run_tokio_outbox_service_loop(
    config: OutboxPollerServiceConfig,
    runner: &mut dyn AsyncOutboxPollerCycleRunner,
    sleeper: &mut dyn AsyncOutboxPollerSleeper,
    shutdown_signal: &mut dyn AsyncOutboxShutdownSignal,
) -> Result<OutboxPollerServiceReport, OutboxPollerError> {
    config.validate()?;
    let mut report = OutboxPollerServiceReport {
        stop_reason: OutboxPollerServiceStopReason::MaxEpochs,
        epochs: Vec::with_capacity(usize::from(config.max_epochs)),
        total_ticks: 0,
        total_claimed_count: 0,
        total_published_count: 0,
        total_dead_letter_count: 0,
        total_error_count: 0,
    };

    for epoch in 1..=config.max_epochs {
        if shutdown_signal.should_stop(epoch - 1).await {
            report.stop_reason = OutboxPollerServiceStopReason::ShutdownRequested;
            break;
        }

        let poller_report =
            run_tokio_bounded_outbox_poller(config.poller_config, runner, sleeper).await?;
        report.total_ticks += poller_report.ticks.len();
        report.total_claimed_count += poller_report.total_claimed_count;
        report.total_published_count += poller_report.total_published_count;
        report.total_dead_letter_count += poller_report.total_dead_letter_count;
        report.total_error_count += poller_report.total_error_count;
        report.epochs.push(OutboxPollerEpochReport {
            epoch,
            poller_report,
        });

        if epoch == config.max_epochs {
            report.stop_reason = OutboxPollerServiceStopReason::MaxEpochs;
            break;
        }
        if shutdown_signal.should_stop(epoch).await {
            report.stop_reason = OutboxPollerServiceStopReason::ShutdownRequested;
            break;
        }
        sleeper.sleep_ms(config.epoch_pause_ms).await;
    }

    Ok(report)
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
    struct AsyncScriptedRunner {
        steps: Vec<ScriptStep>,
        index: usize,
    }

    #[derive(Clone, Debug, Default, Eq, PartialEq)]
    struct RecordingSleeper {
        delays: Vec<u64>,
    }

    #[derive(Clone, Debug, Default, Eq, PartialEq)]
    struct ThresholdShutdown {
        stop_after_completed_epochs: Option<u16>,
        probes: Vec<u16>,
    }

    impl AsyncScriptedRunner {
        fn new(steps: Vec<ScriptStep>) -> Self {
            Self { steps, index: 0 }
        }
    }

    impl AsyncOutboxPollerCycleRunner for AsyncScriptedRunner {
        fn run_worker_cycle<'a>(
            &'a mut self,
        ) -> BoxPollerFuture<'a, Result<OutboxWorkerCycleReport, OutboxPollerRunnerError>> {
            Box::pin(async move {
                let step = self
                    .steps
                    .get(self.index)
                    .cloned()
                    .unwrap_or_else(|| ScriptStep::Cycle(cycle_report(0, 0, 0)));
                self.index += 1;
                match step {
                    ScriptStep::Cycle(report) => Ok(report),
                    ScriptStep::Error(error_ref) => Err(OutboxPollerRunnerError::new(error_ref)
                        .expect("scripted error refs are non-empty")),
                }
            })
        }
    }

    impl AsyncOutboxPollerSleeper for RecordingSleeper {
        fn sleep_ms<'a>(&'a mut self, delay_ms: u64) -> BoxPollerFuture<'a, ()> {
            Box::pin(async move {
                self.delays.push(delay_ms);
            })
        }
    }

    impl AsyncOutboxShutdownSignal for ThresholdShutdown {
        fn should_stop<'a>(&'a mut self, completed_epochs: u16) -> BoxPollerFuture<'a, bool> {
            Box::pin(async move {
                self.probes.push(completed_epochs);
                match self.stop_after_completed_epochs {
                    Some(threshold) => completed_epochs >= threshold,
                    None => false,
                }
            })
        }
    }

    fn config() -> OutboxPollerConfig {
        OutboxPollerConfig {
            max_ticks: 5,
            idle_shutdown_after_ticks: 2,
            max_consecutive_errors: 3,
            base_backoff_ms: 100,
            max_backoff_ms: 250,
        }
    }

    fn service_config(max_epochs: u16) -> OutboxPollerServiceConfig {
        OutboxPollerServiceConfig {
            max_epochs,
            epoch_pause_ms: 500,
            poller_config: OutboxPollerConfig {
                max_ticks: 1,
                ..config()
            },
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

    #[tokio::test]
    async fn tokio_sleeper_awaits_inside_runtime() {
        let mut sleeper = TokioOutboxPollerSleeper;
        sleeper.sleep_ms(1).await;
    }

    #[tokio::test]
    async fn runtime_sleeps_between_ticks_but_not_after_idle_stop() {
        let mut runner = AsyncScriptedRunner::new(vec![
            ScriptStep::Cycle(cycle_report(0, 0, 0)),
            ScriptStep::Cycle(cycle_report(0, 0, 0)),
            ScriptStep::Cycle(cycle_report(1, 1, 0)),
        ]);
        let mut sleeper = RecordingSleeper::default();

        let report = run_tokio_bounded_outbox_poller(config(), &mut runner, &mut sleeper)
            .await
            .unwrap();

        assert_eq!(report.stop_reason, OutboxPollerStopReason::IdleShutdown);
        assert_eq!(report.ticks.len(), 2);
        assert_eq!(sleeper.delays, vec![100]);
    }

    #[tokio::test]
    async fn runtime_uses_capped_error_backoff_between_nonterminal_errors() {
        let mut runner = AsyncScriptedRunner::new(vec![
            ScriptStep::Error("transient:one"),
            ScriptStep::Error("transient:two"),
            ScriptStep::Error("transient:three"),
        ]);
        let mut sleeper = RecordingSleeper::default();

        let report = run_tokio_bounded_outbox_poller(config(), &mut runner, &mut sleeper)
            .await
            .unwrap();

        assert_eq!(report.stop_reason, OutboxPollerStopReason::ErrorShutdown);
        assert_eq!(report.total_error_count, 3);
        assert_eq!(report.ticks[2].planned_delay_ms, 250);
        assert_eq!(sleeper.delays, vec![100, 200]);
    }

    #[tokio::test]
    async fn runtime_accumulates_work_counts_and_sleeps_until_max_ticks() {
        let mut runner = AsyncScriptedRunner::new(vec![
            ScriptStep::Cycle(cycle_report(2, 2, 0)),
            ScriptStep::Cycle(cycle_report(1, 0, 1)),
            ScriptStep::Cycle(cycle_report(2, 1, 1)),
        ]);
        let mut sleeper = RecordingSleeper::default();
        let cfg = OutboxPollerConfig {
            max_ticks: 3,
            ..config()
        };

        let report = run_tokio_bounded_outbox_poller(cfg, &mut runner, &mut sleeper)
            .await
            .unwrap();

        assert_eq!(report.stop_reason, OutboxPollerStopReason::MaxTicks);
        assert_eq!(report.total_claimed_count, 5);
        assert_eq!(report.total_published_count, 3);
        assert_eq!(report.total_dead_letter_count, 2);
        assert_eq!(sleeper.delays, vec![100, 100]);
    }

    #[test]
    fn service_config_rejects_zero_epochs_and_zero_pause() {
        let mut cfg = service_config(0);
        assert_eq!(
            cfg.validate(),
            Err(OutboxPollerError::InvalidConfig {
                field: "max_epochs"
            })
        );

        cfg = service_config(1);
        cfg.epoch_pause_ms = 0;
        assert_eq!(
            cfg.validate(),
            Err(OutboxPollerError::InvalidConfig {
                field: "epoch_pause_ms"
            })
        );
    }

    #[tokio::test]
    async fn service_loop_stops_before_first_epoch_when_shutdown_is_requested() {
        let mut runner = AsyncScriptedRunner::new(vec![ScriptStep::Cycle(cycle_report(2, 2, 0))]);
        let mut sleeper = RecordingSleeper::default();
        let mut shutdown = ThresholdShutdown {
            stop_after_completed_epochs: Some(0),
            probes: Vec::new(),
        };

        let report = run_tokio_outbox_service_loop(
            service_config(3),
            &mut runner,
            &mut sleeper,
            &mut shutdown,
        )
        .await
        .unwrap();

        assert_eq!(
            report.stop_reason,
            OutboxPollerServiceStopReason::ShutdownRequested
        );
        assert!(report.epochs.is_empty());
        assert_eq!(runner.index, 0);
        assert!(sleeper.delays.is_empty());
        assert_eq!(shutdown.probes, vec![0]);
    }

    #[tokio::test]
    async fn service_loop_runs_epochs_and_pauses_between_nonterminal_epochs() {
        let mut runner = AsyncScriptedRunner::new(vec![
            ScriptStep::Cycle(cycle_report(2, 1, 1)),
            ScriptStep::Cycle(cycle_report(3, 3, 0)),
        ]);
        let mut sleeper = RecordingSleeper::default();
        let mut shutdown = ThresholdShutdown::default();

        let report = run_tokio_outbox_service_loop(
            service_config(2),
            &mut runner,
            &mut sleeper,
            &mut shutdown,
        )
        .await
        .unwrap();

        assert_eq!(report.stop_reason, OutboxPollerServiceStopReason::MaxEpochs);
        assert_eq!(report.epochs.len(), 2);
        assert_eq!(report.total_ticks, 2);
        assert_eq!(report.total_claimed_count, 5);
        assert_eq!(report.total_published_count, 4);
        assert_eq!(report.total_dead_letter_count, 1);
        assert_eq!(sleeper.delays, vec![500]);
        assert_eq!(shutdown.probes, vec![0, 1, 1]);
    }

    #[tokio::test]
    async fn service_loop_honors_shutdown_after_completed_epoch_without_pause() {
        let mut runner = AsyncScriptedRunner::new(vec![
            ScriptStep::Cycle(cycle_report(1, 1, 0)),
            ScriptStep::Cycle(cycle_report(99, 99, 0)),
        ]);
        let mut sleeper = RecordingSleeper::default();
        let mut shutdown = ThresholdShutdown {
            stop_after_completed_epochs: Some(1),
            probes: Vec::new(),
        };

        let report = run_tokio_outbox_service_loop(
            service_config(3),
            &mut runner,
            &mut sleeper,
            &mut shutdown,
        )
        .await
        .unwrap();

        assert_eq!(
            report.stop_reason,
            OutboxPollerServiceStopReason::ShutdownRequested
        );
        assert_eq!(report.epochs.len(), 1);
        assert_eq!(report.total_claimed_count, 1);
        assert_eq!(runner.index, 1);
        assert!(sleeper.delays.is_empty());
        assert_eq!(shutdown.probes, vec![0, 1]);
    }

    fn supervisor_config(max_epochs: u16) -> OutboxSupervisorConfig {
        OutboxSupervisorConfig {
            service_ref: "outbox:messenger".into(),
            worker_ref: "worker:a".into(),
            readiness_timeout_ms: 2_000,
            shutdown_grace_ms: 10_000,
            service_config: service_config(max_epochs),
        }
    }

    #[test]
    fn supervisor_config_rejects_empty_refs_and_zero_timeouts() {
        let mut cfg = supervisor_config(1);
        cfg.service_ref = " ".into();
        assert_eq!(
            cfg.validate(),
            Err(OutboxPollerError::InvalidConfig {
                field: "service_ref"
            })
        );

        cfg = supervisor_config(1);
        cfg.worker_ref = " ".into();
        assert_eq!(
            cfg.validate(),
            Err(OutboxPollerError::InvalidConfig {
                field: "worker_ref"
            })
        );

        cfg = supervisor_config(1);
        cfg.readiness_timeout_ms = 0;
        assert_eq!(
            cfg.validate(),
            Err(OutboxPollerError::InvalidConfig {
                field: "readiness_timeout_ms"
            })
        );

        cfg = supervisor_config(1);
        cfg.shutdown_grace_ms = 0;
        assert_eq!(
            cfg.validate(),
            Err(OutboxPollerError::InvalidConfig {
                field: "shutdown_grace_ms"
            })
        );
    }

    #[tokio::test]
    async fn supervised_service_reports_start_ready_and_stopped_events() {
        let mut runner = AsyncScriptedRunner::new(vec![ScriptStep::Cycle(cycle_report(2, 2, 0))]);
        let mut sleeper = RecordingSleeper::default();
        let mut shutdown = ThresholdShutdown::default();

        let report = run_supervised_tokio_outbox_service(
            supervisor_config(1),
            &mut runner,
            &mut sleeper,
            &mut shutdown,
        )
        .await
        .unwrap();

        assert_eq!(report.service_ref, "outbox:messenger");
        assert_eq!(report.worker_ref, "worker:a");
        assert_eq!(
            report.service_report.stop_reason,
            OutboxPollerServiceStopReason::MaxEpochs
        );
        assert_eq!(
            report
                .events
                .iter()
                .map(|event| event.kind)
                .collect::<Vec<_>>(),
            vec![
                OutboxSupervisorLifecycleEventKind::Starting,
                OutboxSupervisorLifecycleEventKind::Ready,
                OutboxSupervisorLifecycleEventKind::Stopped,
            ]
        );
        assert_eq!(report.events[2].completed_epochs, 1);
        assert_eq!(
            report.events[2].stop_reason,
            Some(OutboxPollerServiceStopReason::MaxEpochs)
        );
    }

    #[tokio::test]
    async fn supervised_service_records_shutdown_requested_event() {
        let mut runner = AsyncScriptedRunner::new(vec![ScriptStep::Cycle(cycle_report(99, 99, 0))]);
        let mut sleeper = RecordingSleeper::default();
        let mut shutdown = ThresholdShutdown {
            stop_after_completed_epochs: Some(0),
            probes: Vec::new(),
        };

        let report = run_supervised_tokio_outbox_service(
            supervisor_config(3),
            &mut runner,
            &mut sleeper,
            &mut shutdown,
        )
        .await
        .unwrap();

        assert_eq!(
            report.service_report.stop_reason,
            OutboxPollerServiceStopReason::ShutdownRequested
        );
        assert_eq!(runner.index, 0);
        assert_eq!(
            report
                .events
                .iter()
                .map(|event| event.kind)
                .collect::<Vec<_>>(),
            vec![
                OutboxSupervisorLifecycleEventKind::Starting,
                OutboxSupervisorLifecycleEventKind::Ready,
                OutboxSupervisorLifecycleEventKind::ShutdownRequested,
                OutboxSupervisorLifecycleEventKind::Stopped,
            ]
        );
        assert_eq!(report.events[2].completed_epochs, 0);
        assert_eq!(
            report.events[3].stop_reason,
            Some(OutboxPollerServiceStopReason::ShutdownRequested)
        );
    }
}
