//! Runtime metrics for a guarded PDP.

use crate::*;

#[derive(Debug)]
struct PdpRuntimeMetricsInner {
    authorize_total: u64,           // data_class: INTERNAL_ONLY
    allow_total: u64,               // data_class: INTERNAL_ONLY
    deny_total: u64,                // data_class: INTERNAL_ONLY
    error_total: u64,               // data_class: INTERNAL_ONLY
    timeout_total: u64,             // data_class: INTERNAL_ONLY
    panic_total: u64,               // data_class: INTERNAL_ONLY
    circuit_open_total: u64,        // data_class: INTERNAL_ONLY
    latency_ms: VecDeque<u64>,      // data_class: INTERNAL_ONLY
    metrics_window: usize,          // data_class: INTERNAL_ONLY
    circuit_state: PdpCircuitState, // data_class: INTERNAL_ONLY
}

impl PdpRuntimeMetricsInner {
    fn new(metrics_window: usize) -> Self {
        Self {
            authorize_total: 0,
            allow_total: 0,
            deny_total: 0,
            error_total: 0,
            timeout_total: 0,
            panic_total: 0,
            circuit_open_total: 0,
            latency_ms: VecDeque::new(),
            metrics_window,
            circuit_state: PdpCircuitState::Closed,
        }
    }

    fn push_latency(&mut self, elapsed: Duration) {
        self.latency_ms.push_back(duration_millis_u64(elapsed));
        while self.latency_ms.len() > self.metrics_window {
            self.latency_ms.pop_front();
        }
    }

    pub(crate) fn snapshot(&self) -> PdpRuntimeMetricsSnapshot {
        PdpRuntimeMetricsSnapshot {
            authorize_total: self.authorize_total,
            allow_total: self.allow_total,
            deny_total: self.deny_total,
            error_total: self.error_total,
            timeout_total: self.timeout_total,
            panic_total: self.panic_total,
            circuit_open_total: self.circuit_open_total,
            p99_latency_ms: p99_latency_ms(&self.latency_ms),
            circuit_state: self.circuit_state,
        }
    }
}

/// In-process PDP runtime counters/gauges. The kernel keeps this dependency-free;
/// adapters may scrape [`PdpRuntimeMetricsSnapshot::prometheus_text`] or map
/// [`PdpRuntimeMetricsSnapshot::trace_fields`] into their tracing substrate.
#[derive(Clone, Debug)]
pub struct PdpRuntimeMetrics {
    inner: Arc<Mutex<PdpRuntimeMetricsInner>>, // data_class: INTERNAL_ONLY
}

impl PdpRuntimeMetrics {
    #[must_use]
    pub fn new(metrics_window: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(PdpRuntimeMetricsInner::new(
                metrics_window.max(1),
            ))),
        }
    }

    pub(crate) fn record_success(&self, elapsed: Duration, decision: Decision) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.authorize_total += 1;
            match decision {
                Decision::Allow => inner.allow_total += 1,
                Decision::Deny => inner.deny_total += 1,
            }
            inner.circuit_state = PdpCircuitState::Closed;
            inner.push_latency(elapsed);
        }
    }

    pub(crate) fn record_error(
        &self,
        elapsed: Duration,
        err: &PdpError,
        circuit_state: PdpCircuitState,
    ) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.authorize_total += 1;
            inner.deny_total += 1;
            inner.error_total += 1;
            match err {
                PdpError::RuntimeTimeout { .. } => inner.timeout_total += 1,
                PdpError::RuntimePanic { .. } => inner.panic_total += 1,
                PdpError::CircuitOpen { .. } => inner.circuit_open_total += 1,
                _ => {}
            }
            inner.circuit_state = circuit_state;
            inner.push_latency(elapsed);
        }
    }

    pub(crate) fn record_circuit_open(&self, err: &PdpError) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.authorize_total += 1;
            inner.deny_total += 1;
            inner.error_total += 1;
            if matches!(err, PdpError::CircuitOpen { .. }) {
                inner.circuit_open_total += 1;
            }
            inner.circuit_state = PdpCircuitState::Open;
        }
    }

    #[must_use]
    pub fn snapshot(&self) -> PdpRuntimeMetricsSnapshot {
        match self.inner.lock() {
            Ok(inner) => inner.snapshot(),
            Err(poisoned) => poisoned.into_inner().snapshot(),
        }
    }
}

/// Stable scrape/trace view of PDP runtime behavior. Error counters are also
/// deny counters because every wrapped timeout/fault/panic/circuit-open path is
/// fail-closed by contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdpRuntimeMetricsSnapshot {
    pub authorize_total: u64,           // data_class: INTERNAL_ONLY
    pub allow_total: u64,               // data_class: INTERNAL_ONLY
    pub deny_total: u64,                // data_class: INTERNAL_ONLY
    pub error_total: u64,               // data_class: INTERNAL_ONLY
    pub timeout_total: u64,             // data_class: INTERNAL_ONLY
    pub panic_total: u64,               // data_class: INTERNAL_ONLY
    pub circuit_open_total: u64,        // data_class: INTERNAL_ONLY
    pub p99_latency_ms: u64,            // data_class: INTERNAL_ONLY
    pub circuit_state: PdpCircuitState, // data_class: INTERNAL_ONLY
}

impl PdpRuntimeMetricsSnapshot {
    #[must_use]
    pub fn prometheus_text(&self) -> String {
        let closed_value = if self.circuit_state == PdpCircuitState::Closed {
            1
        } else {
            0
        };
        let open_value = if self.circuit_state == PdpCircuitState::Open {
            1
        } else {
            0
        };
        format!(
            "# HELP pdp_authorize_latency_p99_ms PDP authorize p99 latency over the in-process runtime window.\n\
             # TYPE pdp_authorize_latency_p99_ms gauge\n\
             pdp_authorize_latency_p99_ms {}\n\
             # HELP pdp_runtime_circuit_state PDP runtime circuit-breaker state; exactly one state is 1.\n\
             # TYPE pdp_runtime_circuit_state gauge\n\
             pdp_runtime_circuit_state{{state=\"closed\"}} {}\n\
             pdp_runtime_circuit_state{{state=\"open\"}} {}\n\
             # TYPE pdp_authorize_total counter\n\
             pdp_authorize_total {}\n\
             # TYPE pdp_authorize_error_total counter\n\
             pdp_authorize_error_total {}\n",
            self.p99_latency_ms, closed_value, open_value, self.authorize_total, self.error_total
        )
    }

    #[must_use]
    pub fn trace_fields(&self) -> BTreeMap<String, String> {
        BTreeMap::from([
            (
                "pdp.runtime.latency_p99_ms".to_owned(),
                self.p99_latency_ms.to_string(),
            ),
            (
                "pdp.runtime.circuit_state".to_owned(),
                self.circuit_state.as_label().to_owned(),
            ),
            (
                "pdp.runtime.authorize_total".to_owned(),
                self.authorize_total.to_string(),
            ),
            (
                "pdp.runtime.error_total".to_owned(),
                self.error_total.to_string(),
            ),
        ])
    }
}

pub(crate) fn duration_millis_u64(duration: Duration) -> u64 {
    let millis = duration.as_millis();
    if millis > u128::from(u64::MAX) {
        u64::MAX
    } else {
        millis as u64
    }
}

pub(crate) fn p99_latency_ms(samples: &VecDeque<u64>) -> u64 {
    if samples.is_empty() {
        return 0;
    }
    let mut sorted: Vec<u64> = samples.iter().copied().collect();
    sorted.sort_unstable();
    let index = ((sorted.len() * 99).div_ceil(100)).saturating_sub(1);
    sorted[index]
}
