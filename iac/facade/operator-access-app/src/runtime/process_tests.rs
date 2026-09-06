use super::process::run_with_timeout;
use super::*;

#[test]
fn exited_leader_with_inherited_pipes_cannot_outlive_deadline() {
    let started = Instant::now();
    let result = run_with_timeout(
        "/bin/sh",
        &strings(&["-c", "sleep 2 & exit 0"]),
        &[],
        false,
        Duration::from_millis(80),
        &AtomicBool::new(false),
    );
    assert_eq!(result.unwrap_err(), AccessError::Timeout);
    assert!(started.elapsed() < Duration::from_secs(1));
}

#[test]
fn cancellation_remains_effective_while_descendant_holds_pipes() {
    let cancelled = AtomicBool::new(false);
    let started = Instant::now();
    thread::scope(|scope| {
        scope.spawn(|| {
            thread::sleep(Duration::from_millis(80));
            cancelled.store(true, Ordering::Relaxed);
        });
        let result = run_with_timeout(
            "/bin/sh",
            &strings(&["-c", "sleep 2 & exit 0"]),
            &[],
            false,
            Duration::from_secs(3),
            &cancelled,
        );
        assert_eq!(result.unwrap_err(), AccessError::Cancelled);
    });
    assert!(started.elapsed() < Duration::from_secs(1));
}
