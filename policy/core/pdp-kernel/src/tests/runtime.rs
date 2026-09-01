use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use shared_platform_contracts_kernel::pdp::Decision;

use crate::{PdpCircuitState, PdpError, PdpRuntimeConfig, PdpRuntimeGuard, PolicyDecisionPoint};

use super::runtime_fixtures::{
    FastEvaluationPdp, PanicOnceThenAllowPdp, PanicPdp, SlowFastSlowPdp, SlowRefusalPdp,
    SlowSideEffectPdp,
};
use super::{request, slice};

#[test]
fn runtime_guard_elapsed_budget_does_not_spawn_late_workers_or_late_side_effects() {
    let (inner, calls, active_calls, max_active_calls, side_effects) =
        SlowSideEffectPdp::new(Duration::from_millis(10));
    let runtime = PdpRuntimeGuard::new(
        Arc::new(inner),
        PdpRuntimeConfig::new(Duration::from_millis(1), 10),
    );

    for _ in 0..4 {
        let err = runtime.authorize(&request(), &slice()).unwrap_err();
        assert!(matches!(err, PdpError::RuntimeTimeout { .. }));
        assert_eq!(
            active_calls.load(Ordering::SeqCst),
            0,
            "the guard must not return while an inner PDP invocation is still running"
        );
        assert_eq!(
            side_effects.load(Ordering::SeqCst),
            calls.load(Ordering::SeqCst),
            "all side effects must complete before the fail-closed timeout is returned"
        );
    }

    std::thread::sleep(Duration::from_millis(30));
    assert_eq!(calls.load(Ordering::SeqCst), 4);
    assert_eq!(side_effects.load(Ordering::SeqCst), 4);
    assert_eq!(
        max_active_calls.load(Ordering::SeqCst),
        1,
        "repeated timeouts must remain bounded to the caller-owned invocation, not grow workers"
    );
}

#[test]
fn runtime_guard_catches_panic_opens_circuit_and_short_circuits() {
    let runtime = PdpRuntimeGuard::new(
        Arc::new(PanicPdp),
        PdpRuntimeConfig::new(Duration::from_secs(1), 1),
    );

    let first = runtime.authorize(&request(), &slice()).unwrap_err();
    let second = runtime.authorize(&request(), &slice()).unwrap_err();

    assert!(matches!(first, PdpError::RuntimePanic { .. }));
    assert!(matches!(second, PdpError::CircuitOpen { .. }));
    let snapshot = runtime.metrics().snapshot();
    assert_eq!(snapshot.panic_total, 1);
    assert_eq!(snapshot.circuit_open_total, 1);
    assert_eq!(snapshot.circuit_state, PdpCircuitState::Open);
    assert!(
        snapshot
            .prometheus_text()
            .contains("pdp_runtime_circuit_state{state=\"open\"} 1")
    );
    assert!(
        snapshot
            .trace_fields()
            .contains_key("pdp.runtime.latency_p99_ms")
    );
}

#[test]
fn runtime_guard_elapsed_budget_overrides_slow_inner_refusal() {
    let runtime = PdpRuntimeGuard::new(
        Arc::new(SlowRefusalPdp {
            delay: Duration::from_millis(10),
        }),
        PdpRuntimeConfig::new(Duration::from_millis(1), 1),
    );

    let first = runtime.authorize(&request(), &slice()).unwrap_err();
    let second = runtime.authorize(&request(), &slice()).unwrap_err();

    assert!(matches!(first, PdpError::RuntimeTimeout { .. }));
    assert!(matches!(second, PdpError::CircuitOpen { .. }));
    let snapshot = runtime.metrics().snapshot();
    assert_eq!(snapshot.timeout_total, 1);
    assert_eq!(snapshot.circuit_open_total, 1);
    assert_eq!(snapshot.circuit_state, PdpCircuitState::Open);
}

#[test]
fn runtime_guard_caller_shaped_evaluation_refusals_do_not_open_circuit() {
    let runtime = PdpRuntimeGuard::new(
        Arc::new(FastEvaluationPdp),
        PdpRuntimeConfig::new(Duration::from_secs(1), 1),
    );

    let first = runtime.authorize(&request(), &slice()).unwrap_err();
    let second = runtime.authorize(&request(), &slice()).unwrap_err();

    assert!(matches!(first, PdpError::Evaluation { .. }));
    assert!(matches!(second, PdpError::Evaluation { .. }));
    let snapshot = runtime.metrics().snapshot();
    assert_eq!(snapshot.circuit_open_total, 0);
    assert_eq!(snapshot.circuit_state, PdpCircuitState::Closed);
}

#[test]
fn runtime_guard_cooldown_allows_half_open_probe_to_recover() {
    let calls = Arc::new(AtomicU32::new(0));
    let runtime = PdpRuntimeGuard::new(
        Arc::new(PanicOnceThenAllowPdp {
            calls: calls.clone(),
        }),
        PdpRuntimeConfig::new(Duration::from_secs(1), 1)
            .with_circuit_open_cooldown(Duration::from_millis(5)),
    );

    let first = runtime.authorize(&request(), &slice()).unwrap_err();
    let second = runtime.authorize(&request(), &slice()).unwrap_err();

    assert!(matches!(first, PdpError::RuntimePanic { .. }));
    assert!(matches!(second, PdpError::CircuitOpen { .. }));
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "open circuit must not call the inner PDP before cooldown"
    );

    std::thread::sleep(Duration::from_millis(10));
    let recovered = runtime.authorize(&request(), &slice()).unwrap();

    assert_eq!(recovered.response.decision, Decision::Allow);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    let snapshot = runtime.metrics().snapshot();
    assert_eq!(snapshot.circuit_open_total, 1);
    assert_eq!(snapshot.circuit_state, PdpCircuitState::Closed);
}

#[test]
fn runtime_guard_success_resets_partial_failure_streak() {
    let calls = Arc::new(AtomicU32::new(0));
    let runtime = PdpRuntimeGuard::new(
        Arc::new(SlowFastSlowPdp {
            calls: calls.clone(),
            slow: Duration::from_millis(10),
        }),
        PdpRuntimeConfig::new(Duration::from_millis(1), 2),
    );

    let first = runtime.authorize(&request(), &slice()).unwrap_err();
    let second = runtime.authorize(&request(), &slice()).unwrap();
    let third = runtime.authorize(&request(), &slice()).unwrap_err();

    assert!(matches!(first, PdpError::RuntimeTimeout { .. }));
    assert_eq!(second.response.decision, Decision::Allow);
    assert!(
        matches!(third, PdpError::RuntimeTimeout { .. }),
        "success must reset the partial runtime-fault streak; got {third:?}"
    );
    let snapshot = runtime.metrics().snapshot();
    assert_eq!(snapshot.timeout_total, 2);
    assert_eq!(snapshot.circuit_open_total, 0);
    assert_eq!(snapshot.circuit_state, PdpCircuitState::Closed);
}
