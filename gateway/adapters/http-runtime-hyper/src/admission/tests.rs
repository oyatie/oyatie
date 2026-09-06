use super::*;
use std::sync::{Arc, Barrier};
use std::thread;

fn limits() -> ServingLimits {
    ServingLimits::new(2, 2, 1, Duration::from_secs(1), Duration::from_secs(1)).unwrap()
}

#[test]
fn refuses_zero_limits_and_jobs_exceeding_requests() {
    let second = Duration::from_secs(1);
    for (connections, requests, jobs) in [(0, 1, 1), (1, 0, 1), (1, 1, 0), (1, 1, 2)] {
        assert!(ServingLimits::new(connections, requests, jobs, second, second).is_err());
    }
    assert!(ServingLimits::new(1, 1, 1, Duration::ZERO, second).is_err());
    assert!(ServingLimits::new(1, 1, 1, second, Duration::ZERO).is_err());
}

#[test]
fn every_budget_refuses_at_capacity_and_returns_on_drop() {
    for budget in [Budget::Connection, Budget::Request, Budget::Job] {
        let ledger = Admission::new(limits());
        let maximum = ledger.snapshot().limits.capacity(budget);
        let mut permits = Vec::new();
        for _ in 0..maximum {
            permits.push(ledger.acquire(budget).unwrap());
        }
        assert_eq!(
            ledger.acquire(budget).unwrap_err(),
            AdmissionRefusal::Capacity(budget)
        );
        assert_eq!(ledger.snapshot().active[budget.index()], maximum);
        permits.pop();
        let replacement = ledger.acquire(budget).unwrap();
        drop(replacement);
        drop(permits);
        let snapshot = ledger.snapshot();
        assert_eq!(snapshot.active, [0; 3]);
        assert_eq!(snapshot.high_water[budget.index()], maximum);
        assert_eq!(snapshot.capacity_refusals[budget.index()], 1);
    }
}

#[test]
fn response_cancellation_does_not_refund_job_owned_request() {
    let ledger = Admission::new(limits());
    let response = ledger.acquire(Budget::Request).unwrap();
    let job_request = Arc::clone(&response);
    let job = ledger.acquire(Budget::Job).unwrap();
    drop(response);
    assert_eq!(ledger.snapshot().active, [0, 1, 1]);
    drop(job);
    assert_eq!(ledger.snapshot().active, [0, 1, 0]);
    drop(job_request);
    assert_eq!(ledger.snapshot().active, [0; 3]);
}

#[test]
fn drain_is_idempotent_and_cannot_report_stopped_with_owned_work() {
    let ledger = Admission::new(limits());
    let request = ledger.acquire(Budget::Request).unwrap();
    let started = Instant::now();
    assert_eq!(ledger.request_drain(started), started);
    assert_eq!(
        ledger.request_drain(started + Duration::from_secs(1)),
        started
    );
    assert!(!ledger.finish_if_quiescent());
    assert_eq!(ledger.snapshot().phase, ServingPhase::Draining);
    for budget in [Budget::Connection, Budget::Request, Budget::Job] {
        assert_eq!(
            ledger.acquire(budget).unwrap_err(),
            AdmissionRefusal::Draining
        );
    }
    drop(request);
    assert!(ledger.finish_if_quiescent());
    assert_eq!(ledger.snapshot().phase, ServingPhase::Stopped);
    assert_eq!(
        ledger.acquire(Budget::Request).unwrap_err(),
        AdmissionRefusal::Draining
    );
}

#[test]
fn concurrent_admission_and_drain_have_one_serial_order() {
    for _ in 0..64 {
        let ledger = Admission::new(limits());
        let start = Arc::new(Barrier::new(2));
        let worker_ledger = Arc::clone(&ledger);
        let worker_start = Arc::clone(&start);
        let worker = thread::spawn(move || {
            worker_start.wait();
            worker_ledger.acquire(Budget::Request)
        });
        start.wait();
        ledger.request_drain(Instant::now());
        let result = worker.join().unwrap();
        match &result {
            Ok(_) => assert_eq!(ledger.snapshot().active[1], 1),
            Err(error) => assert_eq!(*error, AdmissionRefusal::Draining),
        }
        assert_eq!(
            ledger.acquire(Budget::Request).unwrap_err(),
            AdmissionRefusal::Draining
        );
        drop(result);
        assert!(ledger.finish_if_quiescent());
    }
}
