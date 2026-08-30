//! Guard circuit state and the runtime budget applied around a PDP.

use crate::*;

/// Guard circuit state exposed in PDP runtime metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdpCircuitState {
    Closed,
    Open,
}

impl PdpCircuitState {
    #[must_use]
    pub fn as_label(self) -> &'static str {
        match self {
            Self::Closed => "closed",
            Self::Open => "open",
        }
    }
}

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

    pub(crate) fn circuit_threshold(self) -> u32 {
        self.circuit_open_after_failures.max(1)
    }
}
