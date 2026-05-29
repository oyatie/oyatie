//! TDD red-phase tests for the signal/await durable-wait slice.
//!
//! These integration tests target behavioral gaps not covered by the existing
//! in-crate unit tests.  They MUST fail on the current implementation and pass
//! only once the identified gaps are closed.
//!
//! Gap inventory:
//!
//! GAP-1 (WF-ENG-1): `signal_await_invalid_input` does not validate the
//!   `timeout_timer` field for unsafe metadata, even though the equivalent
//!   `has_unsafe_domain_metadata` DOES call `is_safe_timer`.  Because
//!   `SlaTimer` fields are all `pub`, a caller can construct a poisoned timer
//!   directly (bypassing `SlaTimer::new`) and pass it in.  The usecase MUST
//!   catch that and return `SignalAwaitStatus::InvalidInput` before touching
//!   the store or arming the timer.  Currently it does not — the test fails.
//!
//! GAP-2 (WF-ENG-2): There is no test that a second `deliver_signal` call with
//!   a *different* idempotency key (i.e., not a same-key replay) on an already-
//!   delivered record returns `SignalDeliverStatus::Unmatched` (the
//!   `record.delivered == true` branch).  The existing test only exercises the
//!   no-record path.
//!
//! GAP-3 (WF-ENG-3): The timeout idempotency-conflict path returns
//!   `SignalTimeoutStatus::InvalidInput` (not a dedicated `IdempotencyConflict`
//!   variant).  This is a documented behavioural oddity; a test must assert the
//!   exact status so that any future variant addition is caught as a breaking
//!   change.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use oya_workflow_engine_execution_engine_usecase::{
    ExecutionStoreError, SignalAuditEventKind, SignalAwaitInput, SignalAwaitStatus,
    SignalAwaitUsecase, SignalDeliverInput, SignalDeliverStatus, SignalTimeoutInput,
    SignalTimeoutStatus, SlaTimer, SlaTimerStore, SignalAwaitRecord, SignalAwaitStore,
    SignalDeliverStore, SignalTimeoutStore,
};

// ── In-memory fake stores (mirrors the structure in lib.rs unit tests) ────────

#[derive(Default)]
struct FakeSignalStore {
    suspended: Vec<(String, String, String, String)>,
    records: std::collections::BTreeMap<String, SignalAwaitRecord>,
    resumed: Vec<(String, String, String)>,
    timed_out: Vec<(String, String, String)>,
    suspend_failure: bool,
    load_failure: bool,
    resume_failure: bool,
    timeout_failure: bool,
}

impl FakeSignalStore {
    fn record_key(tenant_id: &str, run_id: &str, signal_name: &str) -> String {
        format!("{tenant_id}:{run_id}:{signal_name}")
    }
}

impl SignalAwaitStore for FakeSignalStore {
    fn suspend_step_awaiting_signal(
        &mut self,
        tenant_id: &str,
        run_id: &str,
        step_id: &str,
        signal_name: &str,
        _evidence_ref: &str,
    ) -> Result<(), ExecutionStoreError> {
        if self.suspend_failure {
            return Err(ExecutionStoreError::Unavailable {
                evidence_ref: "store:suspend-unavailable".to_owned(),
            });
        }
        self.suspended.push((
            tenant_id.to_owned(),
            run_id.to_owned(),
            step_id.to_owned(),
            signal_name.to_owned(),
        ));
        self.records.insert(
            Self::record_key(tenant_id, run_id, signal_name),
            SignalAwaitRecord {
                tenant_id: tenant_id.to_owned(),
                run_id: run_id.to_owned(),
                step_id: step_id.to_owned(),
                signal_name: signal_name.to_owned(),
                delivered: false,
            },
        );
        Ok(())
    }

    fn load_await_record(
        &self,
        tenant_id: &str,
        run_id: &str,
        signal_name: &str,
    ) -> Result<Option<SignalAwaitRecord>, ExecutionStoreError> {
        if self.load_failure {
            return Err(ExecutionStoreError::Unavailable {
                evidence_ref: "store:load-unavailable".to_owned(),
            });
        }
        Ok(self
            .records
            .get(&Self::record_key(tenant_id, run_id, signal_name))
            .cloned())
    }
}

impl SignalDeliverStore for FakeSignalStore {
    fn resume_step_on_signal(
        &mut self,
        tenant_id: &str,
        run_id: &str,
        signal_name: &str,
        _evidence_ref: &str,
    ) -> Result<(), ExecutionStoreError> {
        if self.resume_failure {
            return Err(ExecutionStoreError::Unavailable {
                evidence_ref: "store:resume-unavailable".to_owned(),
            });
        }
        self.resumed.push((
            tenant_id.to_owned(),
            run_id.to_owned(),
            signal_name.to_owned(),
        ));
        if let Some(record) = self
            .records
            .get_mut(&Self::record_key(tenant_id, run_id, signal_name))
        {
            record.delivered = true;
        }
        Ok(())
    }
}

impl SignalTimeoutStore for FakeSignalStore {
    fn timeout_step_awaiting_signal(
        &mut self,
        tenant_id: &str,
        run_id: &str,
        signal_name: &str,
        _evidence_ref: &str,
    ) -> Result<(), ExecutionStoreError> {
        if self.timeout_failure {
            return Err(ExecutionStoreError::Unavailable {
                evidence_ref: "store:timeout-unavailable".to_owned(),
            });
        }
        self.timed_out.push((
            tenant_id.to_owned(),
            run_id.to_owned(),
            signal_name.to_owned(),
        ));
        Ok(())
    }
}

#[derive(Default)]
struct FakeTimers {
    armed: Vec<SlaTimer>,
    unavailable: bool,
}

impl SlaTimerStore for FakeTimers {
    fn arm_timer(&mut self, timer: SlaTimer) -> Result<(), ExecutionStoreError> {
        if self.unavailable {
            return Err(ExecutionStoreError::Unavailable {
                evidence_ref: "timer-store:unavailable".to_owned(),
            });
        }
        self.armed.push(timer);
        Ok(())
    }

    fn cancel_timer(
        &mut self,
        _tenant_id: &str,
        _timer_id: &str,
    ) -> Result<(), ExecutionStoreError> {
        Ok(())
    }

    fn fire_expired(
        &mut self,
        _tenant_id: &str,
        _now_epoch_seconds: u64,
    ) -> Result<Vec<SlaTimer>, ExecutionStoreError> {
        Ok(Vec::new())
    }
}

// ── Builder helpers ───────────────────────────────────────────────────────────

fn base_await_input() -> SignalAwaitInput {
    SignalAwaitInput {
        request_id: "req:signal-await:gap-test".to_owned(),
        idempotency_key: "idem:signal-await:gap-test".to_owned(),
        trace_ref: "trace:signal-await:gap-test".to_owned(),
        tenant_id: "ten_a".to_owned(),
        run_id: "run:signal-await:gap-test".to_owned(),
        step_id: "step:signal-await:gap-test".to_owned(),
        signal_name: "signal:approval:gap-test".to_owned(),
        timeout_timer: None,
    }
}

fn base_deliver_input() -> SignalDeliverInput {
    SignalDeliverInput {
        request_id: "req:signal-deliver:gap-test".to_owned(),
        idempotency_key: "idem:signal-deliver:gap-test".to_owned(),
        trace_ref: "trace:signal-deliver:gap-test".to_owned(),
        tenant_id: "ten_a".to_owned(),
        run_id: "run:signal-await:gap-test".to_owned(),
        signal_name: "signal:approval:gap-test".to_owned(),
    }
}

fn base_timeout_input() -> SignalTimeoutInput {
    SignalTimeoutInput {
        request_id: "req:signal-timeout:gap-test".to_owned(),
        idempotency_key: "idem:signal-timeout:gap-test".to_owned(),
        trace_ref: "trace:signal-timeout:gap-test".to_owned(),
        tenant_id: "ten_a".to_owned(),
        run_id: "run:signal-await:gap-test".to_owned(),
        signal_name: "signal:approval:gap-test".to_owned(),
        reference_epoch_seconds: 1_000,
    }
}

/// Build a `SlaTimer` with a poisoned `timer_id` ("sk-…") via struct literal,
/// bypassing `SlaTimer::new` which would reject it.  All fields are `pub`, so
/// this is well-formed Rust.  The usecase `signal_await_invalid_input` is the
/// last line of defence and MUST call `is_safe_timer` on `timeout_timer`
/// (it currently does not — that is GAP-1).
fn poisoned_timer() -> SlaTimer {
    SlaTimer {
        timer_id: "sk-poisoned:timer-id".to_owned(), // "sk-" prefix → unsafe
        tenant_id: "ten_a".to_owned(),
        run_id: "run:signal-await:gap-test".to_owned(),
        step_index: None,
        armed_at_epoch_seconds: 100,
        deadline_epoch_seconds: 200,
        evidence_refs: vec!["workflow-signal-usecase:timer".to_owned()],
    }
}

// ── GAP-1: timeout_timer with unsafe metadata must be rejected at input validation ──

/// GAP-1 (WF-ENG-1): await_signal with a timeout_timer whose timer_id carries
/// secret material ("sk-…") must return InvalidInput without calling the store
/// or arming the timer.
///
/// FAILS on current impl because `signal_await_invalid_input` does not validate
/// the `timeout_timer` field.
#[test]
fn await_signal_unsafe_timer_id_yields_invalid_input_without_store_call() {
    let mut uc = SignalAwaitUsecase::default();
    let mut store = FakeSignalStore::default();
    let mut timers = FakeTimers::default();

    let mut input = base_await_input();
    input.timeout_timer = Some(poisoned_timer());

    let r = uc.await_signal(&mut store, &mut timers, input);

    assert_eq!(
        r.status,
        SignalAwaitStatus::InvalidInput,
        "await with unsafe timer_id must be rejected as InvalidInput by the usecase layer"
    );
    assert!(
        store.suspended.is_empty(),
        "store must not be called when input is invalid"
    );
    assert!(
        timers.armed.is_empty(),
        "timer must not be armed when input is invalid"
    );
    assert_eq!(
        uc.cached_await_count(),
        0,
        "invalid-input receipts must not be cached"
    );
    // The poisoned timer_id must not appear verbatim in evidence refs.
    let rendered = format!("{:?}", r.evidence_refs).to_ascii_lowercase();
    assert!(!rendered.contains("sk-poisoned"), "poisoned timer_id must not appear in evidence refs");
}

// ── GAP-2: second deliver with different key on already-delivered record → Unmatched ──

/// GAP-2 (WF-ENG-2): after a successful deliver, a subsequent deliver call
/// using a *different* idempotency key (not a same-key replay) hits the
/// `record.delivered == true` branch and must return `Unmatched` — proving
/// the double-resume guard is in place and functions regardless of idempotency
/// key.
///
/// The existing unit tests only cover: (a) same-key replay (idempotent),
/// and (b) deliver with no prior await record at all.  This test covers the
/// third case: prior record exists but is already delivered.
#[test]
fn deliver_signal_second_different_key_after_delivery_yields_unmatched() {
    let mut uc = SignalAwaitUsecase::default();
    let mut store = FakeSignalStore::default();
    let mut timers = FakeTimers::default();

    // Step 1: suspend the step.
    uc.await_signal(&mut store, &mut timers, base_await_input());

    // Step 2: first delivery — succeeds.
    let first = uc.deliver_signal(&mut store, base_deliver_input());
    assert_eq!(first.status, SignalDeliverStatus::Delivered);
    assert_eq!(store.resumed.len(), 1);

    // Step 3: second delivery with a DIFFERENT idempotency key — the record
    // exists but is already marked delivered, so this must NOT call resume
    // again and must return Unmatched.
    let mut second_input = base_deliver_input();
    second_input.idempotency_key = "idem:signal-deliver:gap-second".to_owned();
    second_input.request_id = "req:signal-deliver:gap-second".to_owned();

    let r = uc.deliver_signal(&mut store, second_input);

    assert_eq!(
        r.status,
        SignalDeliverStatus::Unmatched,
        "second deliver with different key on already-delivered record must be Unmatched"
    );
    assert_eq!(
        store.resumed.len(),
        1,
        "resume must not be called a second time"
    );
    assert!(
        r.evidence_refs
            .contains(&"workflow-signal-usecase:unmatched".to_owned()),
        "unmatched evidence ref must be present"
    );
}

// ── GAP-3: timeout idempotency-conflict returns InvalidInput (not a dedicated variant) ──

/// GAP-3 (WF-ENG-3): the `signal_timeout_conflict_receipt` path reuses
/// `SignalTimeoutStatus::InvalidInput` rather than introducing a new
/// `IdempotencyConflict` variant.  This is a deliberate (if surprising) design
/// choice that must be pinned as a regression test so any future variant
/// addition is caught as a breaking change requiring conscious review.
#[test]
fn timeout_signal_idempotency_conflict_returns_invalid_input_status() {
    let mut uc = SignalAwaitUsecase::default();
    let mut store = FakeSignalStore::default();
    let mut timers = FakeTimers::default();

    // Suspend the step so the record exists.
    uc.await_signal(&mut store, &mut timers, base_await_input());

    // First timeout call — establishes the idempotency entry.
    let first = uc.timeout_signal(&mut store, base_timeout_input());
    assert_eq!(first.status, SignalTimeoutStatus::TimedOut);

    // Second timeout call with the SAME idempotency key but a different
    // trace_ref → fingerprint mismatch → conflict path.
    let mut conflict = base_timeout_input();
    conflict.trace_ref = "trace:signal-timeout:conflict-other".to_owned();

    let r = uc.timeout_signal(&mut store, conflict);

    // Current impl returns InvalidInput on conflict (no dedicated variant).
    // This test pins that behavior: if this fails, a new variant was added
    // and the status mapping must be reviewed.
    assert_eq!(
        r.status,
        SignalTimeoutStatus::InvalidInput,
        "timeout idempotency-conflict must return InvalidInput (no dedicated variant per current design)"
    );
    // The timeout store must not be called a second time.
    assert_eq!(
        store.timed_out.len(),
        1,
        "timeout store must not be called on a conflict replay"
    );
}

// ── Additional error-path coverage (store/timer failures) ────────────────────
// These are not strictly "gap" tests (the impl has the paths) but they are
// missing from the existing test suite and are required by the acceptance
// criteria for completeness.

/// WF-ENG-1: store failure during suspend_step returns StoreUnavailable.
#[test]
fn await_signal_store_failure_returns_store_unavailable() {
    let mut uc = SignalAwaitUsecase::default();
    let mut store = FakeSignalStore {
        suspend_failure: true,
        ..Default::default()
    };
    let mut timers = FakeTimers::default();

    let r = uc.await_signal(&mut store, &mut timers, base_await_input());

    assert_eq!(r.status, SignalAwaitStatus::StoreUnavailable);
    assert!(timers.armed.is_empty(), "timer must not be armed after store failure");
    assert_eq!(
        uc.cached_await_count(),
        0,
        "store-failure receipts must not be cached"
    );
}

/// WF-ENG-1: timer arm failure after successful suspend returns TimerUnavailable.
#[test]
fn await_signal_timer_failure_after_suspend_returns_timer_unavailable() {
    let mut uc = SignalAwaitUsecase::default();
    let mut store = FakeSignalStore::default();
    let mut timers = FakeTimers {
        unavailable: true,
        ..Default::default()
    };

    let mut input = base_await_input();
    input.timeout_timer = Some(
        SlaTimer::new(
            "timer:signal-await:gap-timer-fail",
            "ten_a",
            "run:signal-await:gap-test",
            None,
            100,
            200,
            vec!["workflow-signal-usecase:timer".to_owned()],
        )
        .unwrap(),
    );

    let r = uc.await_signal(&mut store, &mut timers, input);

    assert_eq!(r.status, SignalAwaitStatus::TimerUnavailable);
    // The suspend already happened before the timer failure.
    assert_eq!(
        store.suspended.len(),
        1,
        "suspend is called before timer arm"
    );
}

/// WF-ENG-2: load failure during deliver returns StoreUnavailable.
#[test]
fn deliver_signal_load_failure_returns_store_unavailable() {
    let mut uc = SignalAwaitUsecase::default();
    let mut store = FakeSignalStore {
        load_failure: true,
        ..Default::default()
    };

    let r = uc.deliver_signal(&mut store, base_deliver_input());

    assert_eq!(r.status, SignalDeliverStatus::StoreUnavailable);
    assert!(store.resumed.is_empty());
}

/// WF-ENG-2: resume failure after successful load returns StoreUnavailable.
#[test]
fn deliver_signal_resume_failure_returns_store_unavailable() {
    let mut uc = SignalAwaitUsecase::default();
    let mut store = FakeSignalStore::default();
    let mut timers = FakeTimers::default();

    // Suspend first so the record exists.
    uc.await_signal(&mut store, &mut timers, base_await_input());

    // Now make resume fail.
    store.resume_failure = true;

    let r = uc.deliver_signal(&mut store, base_deliver_input());

    assert_eq!(r.status, SignalDeliverStatus::StoreUnavailable);
}

/// WF-ENG-3: timeout store failure returns StoreUnavailable.
#[test]
fn timeout_signal_store_failure_returns_store_unavailable() {
    let mut uc = SignalAwaitUsecase::default();
    let mut store = FakeSignalStore::default();
    let mut timers = FakeTimers::default();

    // Suspend so the record exists (not yet delivered → timeout path will proceed).
    uc.await_signal(&mut store, &mut timers, base_await_input());

    // Make timeout_step fail.
    store.timeout_failure = true;

    let r = uc.timeout_signal(&mut store, base_timeout_input());

    assert_eq!(r.status, SignalTimeoutStatus::StoreUnavailable);
    assert!(
        store.timed_out.is_empty(),
        "timed_out list must be empty when store returns error"
    );
}

/// WF-ENG-2: invalid deliver input (missing colon in signal_name) returns
/// InvalidInput without touching the store.
#[test]
fn deliver_signal_invalid_signal_name_returns_invalid_input() {
    let mut uc = SignalAwaitUsecase::default();
    let mut store = FakeSignalStore::default();

    let mut input = base_deliver_input();
    input.signal_name = "no-colon-here".to_owned();

    let r = uc.deliver_signal(&mut store, input);

    assert_eq!(r.status, SignalDeliverStatus::InvalidInput);
    assert!(store.resumed.is_empty());
    assert_eq!(
        uc.cached_deliver_count(),
        0,
        "invalid-input receipts must not be cached"
    );
}

/// WF-ENG-3: invalid timeout input (empty run_id) returns InvalidInput.
#[test]
fn timeout_signal_invalid_run_id_returns_invalid_input() {
    let mut uc = SignalAwaitUsecase::default();
    let mut store = FakeSignalStore::default();

    let mut input = base_timeout_input();
    input.run_id = String::new(); // empty → fails is_safe_ref

    let r = uc.timeout_signal(&mut store, input);

    assert_eq!(r.status, SignalTimeoutStatus::InvalidInput);
    assert!(
        store.timed_out.is_empty(),
        "store must not be called for invalid input"
    );
    assert_eq!(
        uc.cached_timeout_count(),
        0,
        "invalid-input receipts must not be cached"
    );
}

/// WF-ENG-1: await audit event chain contains AwaitRequested then AwaitSuspended
/// in that order, verifying the two-event contract.
#[test]
fn await_signal_receipt_contains_requested_then_suspended_audit_events() {
    let mut uc = SignalAwaitUsecase::default();
    let mut store = FakeSignalStore::default();
    let mut timers = FakeTimers::default();

    let r = uc.await_signal(&mut store, &mut timers, base_await_input());

    assert_eq!(r.audit_events.len(), 2, "receipt must contain exactly 2 audit events");
    assert_eq!(
        r.audit_events[0].kind,
        SignalAuditEventKind::AwaitRequested,
        "first audit event must be AwaitRequested"
    );
    assert_eq!(
        r.audit_events[1].kind,
        SignalAuditEventKind::AwaitSuspended,
        "second audit event must be AwaitSuspended"
    );
}
