//! Scheduling: joining, running-once, background loops, and what a dropped handle does.
//!
//! Every test here corresponds to a place where the obvious Rust construct means something
//! DIFFERENT from the Go one. A crate that merely compiled would pass none of them and would look
//! finished.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use std::sync::Mutex;

use k8s_go_runtime::{Once, Signal, WaitGroup, spawn, wait};

/// `wg.Wait()` releases when the counter reaches zero, from any goroutine.
#[test]
fn a_wait_group_joins_work_it_did_not_spawn() {
    let group = WaitGroup::new();
    let done = Arc::new(AtomicUsize::new(0));

    group.add(4);
    for _ in 0..4 {
        let group = group.clone();
        let done = Arc::clone(&done);
        spawn(move || {
            std::thread::sleep(Duration::from_millis(10));
            done.fetch_add(1, Ordering::SeqCst);
            group.done();
        });
    }

    group.wait();
    assert_eq!(done.load(Ordering::SeqCst), 4);
    assert_eq!(group.outstanding(), 0);
}

/// run the closure still may not proceed before the initialisation it depends on is finished.
#[test]
fn a_once_runs_exactly_once_and_every_caller_waits_for_it() {
    let once = Once::new();
    let runs = Arc::new(AtomicUsize::new(0));
    let observed = Arc::new(Mutex::new(Vec::new()));

    let callers: Vec<_> = (0..8)
        .map(|_| {
            let once = once.clone();
            let runs = Arc::clone(&runs);
            let observed = Arc::clone(&observed);
            spawn(move || {
                once.call(|| {
                    std::thread::sleep(Duration::from_millis(20));
                    runs.fetch_add(1, Ordering::SeqCst);
                });
                let complete = runs.load(Ordering::SeqCst);
                observed
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(complete);
            })
        })
        .collect();

    for caller in callers {
        assert!(caller.join());
    }
    assert_eq!(runs.load(Ordering::SeqCst), 1);
    let observed = observed
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    assert!(
        observed.iter().all(|seen| *seen == 1),
        "every caller must observe the closure as finished: {observed:?}"
    );
}

/// background loops in the corpus, that is the difference between a clean shutdown and a hang.
#[test]
fn until_runs_immediately_and_stops_promptly() {
    let (trigger, waiter) = Signal::pair();
    let runs = Arc::new(AtomicUsize::new(0));

    let loop_thread = {
        let runs = Arc::clone(&runs);
        spawn(move || {
            wait::until(
                || {
                    runs.fetch_add(1, Ordering::SeqCst);
                },
                Duration::from_secs(30),
                &waiter,
            );
        })
    };

    // The first run is immediate, and the second is 30 seconds away.
    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(runs.load(Ordering::SeqCst), 1, "the first run is immediate");

    let stopped_at = std::time::Instant::now();
    trigger.fire();
    assert!(loop_thread.join());
    assert!(
        stopped_at.elapsed() < Duration::from_secs(5),
        "a stopped loop must not wait out its period"
    );
    assert_eq!(runs.load(Ordering::SeqCst), 1);
}

/// on every shutdown, once per loop.
#[test]
fn until_does_nothing_when_it_is_already_stopped() {
    let (trigger, waiter) = Signal::pair();
    trigger.fire();

    let runs = AtomicUsize::new(0);
    wait::until(
        || {
            runs.fetch_add(1, Ordering::SeqCst);
        },
        Duration::from_millis(1),
        &waiter,
    );
    assert_eq!(runs.load(Ordering::SeqCst), 0);
}

/// `PollImmediateUntil` runs its condition before waiting, and reports which of the two ended it.
#[test]
fn poll_immediate_until_checks_before_it_waits() {
    let (_trigger, waiter) = Signal::pair();
    let checks = AtomicUsize::new(0);
    let satisfied = wait::poll_immediate_until(
        || checks.fetch_add(1, Ordering::SeqCst) >= 2,
        Duration::from_millis(1),
        &waiter,
    );
    assert!(satisfied);
    assert_eq!(checks.load(Ordering::SeqCst), 3);

    let (trigger, waiter) = Signal::pair();
    trigger.fire();
    assert!(!wait::poll_immediate_until(
        || false,
        Duration::from_millis(1),
        &waiter
    ));
}

/// one of them into a synchronous call.
#[test]
fn dropping_a_goroutine_handle_does_not_join_it() {
    let (trigger, waiter) = Signal::pair();
    let finished = Arc::new(AtomicUsize::new(0));

    {
        let finished = Arc::clone(&finished);
        let waiter = waiter.clone();
        drop(spawn(move || {
            waiter.wait();
            finished.fetch_add(1, Ordering::SeqCst);
        }));
    }

    assert_eq!(
        finished.load(Ordering::SeqCst),
        0,
        "dropping the handle must not have waited"
    );
    trigger.fire();
    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(finished.load(Ordering::SeqCst), 1);
}
