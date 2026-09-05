mod metrics;

use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use shared_platform_contracts_kernel::pdp::{AuthorizationRequest, PolicyVersion};

use crate::{EntitySlice, PdpError, PdpOutcome, PolicyDecisionPoint};

pub use metrics::{PdpCircuitState, PdpRuntimeMetrics, PdpRuntimeMetricsSnapshot};

/// Runtime wrapper configuration: an elapsed-time budget, a runtime-fault
/// streak that opens the fail-closed circuit, and a bounded cooldown after
/// which the guard closes again until the next runtime fault re-opens it.
///
/// The budget is deliberately not described as a hard cancellation deadline.
/// [`PdpRuntimeGuard`] invokes the wrapped synchronous PDP on the caller's
/// thread, catches unwind panics, and returns a fail-closed timeout only after
/// the inner call has completed. That narrower semantics avoids unbounded
/// timeout workers and forbids late side effects after the denial is returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PdpRuntimeConfig {
    pub deadline: Duration,               // data_class: INTERNAL_ONLY
    pub circuit_open_after_failures: u32, // data_class: INTERNAL_ONLY
    pub metrics_window: usize,            // data_class: INTERNAL_ONLY
    pub circuit_open_cooldown: Duration,  // data_class: INTERNAL_ONLY
}

impl PdpRuntimeConfig {
    const DEFAULT_METRICS_WINDOW: usize = 128;
    const DEFAULT_CIRCUIT_OPEN_COOLDOWN: Duration = Duration::from_secs(30);

    #[must_use]
    pub fn new(deadline: Duration, circuit_open_after_failures: u32) -> Self {
        Self {
            deadline,
            circuit_open_after_failures: circuit_open_after_failures.max(1),
            metrics_window: Self::DEFAULT_METRICS_WINDOW,
            circuit_open_cooldown: Self::DEFAULT_CIRCUIT_OPEN_COOLDOWN,
        }
    }

    #[must_use]
    pub fn with_metrics_window(mut self, metrics_window: usize) -> Self {
        self.metrics_window = metrics_window.max(1);
        self
    }

    #[must_use]
    pub fn with_circuit_open_cooldown(mut self, cooldown: Duration) -> Self {
        self.circuit_open_cooldown = cooldown;
        self
    }

    fn circuit_threshold(self) -> u32 {
        self.circuit_open_after_failures.max(1)
    }
}

/// Fail-closed PDP runtime wrapper with bounded/no-late-side-effect semantics.
///
/// This guard intentionally does not spawn timeout workers. The synchronous PDP
/// executes on the caller's thread, so a wedged PDP can still occupy that caller
/// until it returns; composition roots that need preemptive cancellation must
/// inject a PDP implementation with its own cooperative cancellation boundary.
/// The kernel guard's contract is narrower and auditable: elapsed-budget
/// violations fail closed after completion, panics are caught, runtime-fault
/// streaks open a deny-only circuit with a bounded cooldown probe, and no worker
/// continues after a denial.
#[derive(Clone)]
pub struct PdpRuntimeGuard {
    inner: Arc<dyn PolicyDecisionPoint>,  // data_class: INTERNAL_ONLY
    config: PdpRuntimeConfig,             // data_class: INTERNAL_ONLY
    metrics: PdpRuntimeMetrics,           // data_class: INTERNAL_ONLY
    consecutive_failures: Arc<AtomicU32>, // data_class: INTERNAL_ONLY
    circuit_opened_at: Arc<Mutex<Option<Instant>>>, // data_class: INTERNAL_ONLY
}

impl PdpRuntimeGuard {
    #[must_use]
    pub fn new(inner: Arc<dyn PolicyDecisionPoint>, config: PdpRuntimeConfig) -> Self {
        Self {
            inner,
            config,
            metrics: PdpRuntimeMetrics::new(config.metrics_window),
            consecutive_failures: Arc::new(AtomicU32::new(0)),
            circuit_opened_at: Arc::new(Mutex::new(None)),
        }
    }

    #[must_use]
    pub fn metrics(&self) -> PdpRuntimeMetrics {
        self.metrics.clone()
    }

    fn failure_count(&self) -> u32 {
        self.consecutive_failures.load(Ordering::SeqCst)
    }

    fn circuit_state(&self) -> PdpCircuitState {
        if self.failure_count() < self.config.circuit_threshold() {
            return PdpCircuitState::Closed;
        }
        match self.circuit_opened_at.lock() {
            Ok(opened_at) => match *opened_at {
                Some(opened_at) if opened_at.elapsed() < self.config.circuit_open_cooldown => {
                    PdpCircuitState::Open
                }
                Some(_) | None => PdpCircuitState::Closed,
            },
            Err(_) => PdpCircuitState::Open,
        }
    }

    fn mark_runtime_failure(&self) -> PdpCircuitState {
        let failures = self.consecutive_failures.fetch_add(1, Ordering::SeqCst) + 1;
        if failures >= self.config.circuit_threshold() {
            if let Ok(mut opened_at) = self.circuit_opened_at.lock() {
                *opened_at = Some(Instant::now());
            }
            PdpCircuitState::Open
        } else {
            PdpCircuitState::Closed
        }
    }

    fn reset_runtime_failures(&self) {
        self.consecutive_failures.store(0, Ordering::SeqCst);
        if let Ok(mut opened_at) = self.circuit_opened_at.lock() {
            *opened_at = None;
        }
    }
}

impl PolicyDecisionPoint for PdpRuntimeGuard {
    fn authorize(
        &self,
        request: &AuthorizationRequest,
        entities: &EntitySlice,
    ) -> Result<PdpOutcome, PdpError> {
        if self.circuit_state() == PdpCircuitState::Open {
            let err = PdpError::CircuitOpen {
                consecutive_failures: self.failure_count(),
            };
            self.metrics.record_circuit_open(&err);
            return Err(err);
        }

        let start = Instant::now();
        let result =
            panic::catch_unwind(AssertUnwindSafe(|| self.inner.authorize(request, entities)));
        let elapsed = start.elapsed();

        match result {
            Ok(Ok(outcome)) => {
                if elapsed > self.config.deadline {
                    let err = PdpError::RuntimeTimeout {
                        deadline_ms: duration_millis_u64(self.config.deadline),
                    };
                    let state = self.mark_runtime_failure();
                    self.metrics.record_error(elapsed, &err, state);
                    Err(err)
                } else {
                    self.reset_runtime_failures();
                    self.metrics
                        .record_success(elapsed, outcome.response.decision);
                    Ok(outcome)
                }
            }
            Ok(Err(err)) => {
                let err = if elapsed > self.config.deadline {
                    PdpError::RuntimeTimeout {
                        deadline_ms: duration_millis_u64(self.config.deadline),
                    }
                } else {
                    err
                };
                let state = if err.is_runtime_fault() {
                    self.mark_runtime_failure()
                } else {
                    self.circuit_state()
                };
                self.metrics.record_error(elapsed, &err, state);
                Err(err)
            }
            Err(payload) => {
                let err = PdpError::RuntimePanic {
                    detail: panic_payload_detail(payload.as_ref()),
                };
                let state = self.mark_runtime_failure();
                self.metrics.record_error(elapsed, &err, state);
                Err(err)
            }
        }
    }

    fn loaded_policy_version(&self) -> PolicyVersion {
        self.inner.loaded_policy_version()
    }
}

fn duration_millis_u64(duration: Duration) -> u64 {
    let millis = duration.as_millis();
    if millis > u128::from(u64::MAX) {
        u64::MAX
    } else {
        millis as u64
    }
}

fn panic_payload_detail(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic payload".to_owned()
    }
}
