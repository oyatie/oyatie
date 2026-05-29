# Task plan: workflow-engine-signal-await-durable-wait

vertical: workflow
crate: oya-workflow-engine-execution-engine-usecase
branch: feat/task-workflow-engine-signal-await-durable-wait-2026-05-28
base: origin/dev

## Objective

Extend the execution-engine usecase layer with a durable signal/await orchestration
slice. A running step can suspend awaiting a named external signal on a
`(tenant_id, run_id, signal_name)` correlation key and deterministically resume (or
time out) on delivery. The slice composes over the existing
`WorkflowRunStore`/`SlaTimerStore` ports and the idempotency/fingerprint mechanism
already in `ExecutionEngineUsecase`. No new crates, no new workspace members, no
new Cargo dependencies.

## Subtasks

---

### [WF-ENG-1] AwaitSignal usecase path — suspend and receipt

**Scope:** `crates/oya-workflow-engine-execution-engine-usecase/src/lib.rs`

Add:

- `SignalAwaitInput` — input struct for an await-signal request:
  ```rust
  pub struct SignalAwaitInput {
      pub request_id: String,        // data_class: INTERNAL_ONLY
      pub idempotency_key: String,   // data_class: INTERNAL_ONLY
      pub trace_ref: String,         // data_class: INTERNAL_ONLY
      pub tenant_id: String,         // data_class: INTERNAL_ONLY
      pub run_id: String,            // data_class: INTERNAL_ONLY
      pub step_id: String,           // data_class: INTERNAL_ONLY
      pub signal_name: String,       // data_class: INTERNAL_ONLY
      pub timeout_timer: Option<SlaTimer>, // data_class: INTERNAL_ONLY
  }
  ```

- `SignalAwaitStatus` enum:
  - `Awaiting` — step suspended, pending signal delivery
  - `IdempotencyConflict` — same key, different fingerprint
  - `InvalidInput` — missing/unsafe fields
  - `StoreUnavailable` — store error on suspend write
  - `TimerUnavailable` — optional timeout timer could not be armed

- `SignalAwaitReceipt` — deterministic receipt:
  ```rust
  pub struct SignalAwaitReceipt {
      pub status: SignalAwaitStatus,
      pub tenant_id: String,
      pub run_id: String,
      pub step_id: String,
      pub signal_name: String,
      pub timer_armed: bool,
      pub audit_events: Vec<ExecutionAuditEvent>,
      pub evidence_refs: Vec<String>,
  }
  ```

- `SignalAwaitStore` port trait (added to `WorkflowRunStore` extension surface — kept
  as a separate trait so adapters implement only what they need):
  ```rust
  pub trait SignalAwaitStore {
      fn suspend_step_awaiting_signal(
          &mut self,
          tenant_id: &str,
          run_id: &str,
          step_id: &str,
          signal_name: &str,
          evidence_ref: &str,
      ) -> Result<(), ExecutionStoreError>;

      fn load_await_record(
          &self,
          tenant_id: &str,
          run_id: &str,
          signal_name: &str,
      ) -> Result<Option<SignalAwaitRecord>, ExecutionStoreError>;
  }
  ```

- `SignalAwaitRecord` — value object stored by the port:
  ```rust
  pub struct SignalAwaitRecord {
      pub tenant_id: String,
      pub run_id: String,
      pub step_id: String,
      pub signal_name: String,
      pub status: SignalAwaitStatus,  // Awaiting or (later) Delivered/TimedOut
  }
  ```

- `SignalAwaitUsecase` struct (parallel to `ExecutionEngineUsecase`, own idempotency map):
  ```rust
  pub struct SignalAwaitUsecase { ... }
  impl SignalAwaitUsecase {
      pub fn await_signal<S, T>(
          &mut self,
          store: &mut S,
          timers: &mut T,
          input: SignalAwaitInput,
      ) -> SignalAwaitReceipt
      where
          S: SignalAwaitStore,
          T: SlaTimerStore,
      { ... }
  }
  ```

  Logic:
  1. Validate input fields (request_id, idempotency_key, trace_ref safe refs;
     tenant_id safe tenant; run_id, step_id, signal_name safe refs). Return
     `InvalidInput` receipt on failure.
  2. Fingerprint + idempotency check — replay exact receipt; conflict returns
     `IdempotencyConflict`.
  3. Call `store.suspend_step_awaiting_signal(...)` → `StoreUnavailable` on error.
  4. If `timeout_timer` present: call `timers.arm_timer(timer)` → `TimerUnavailable`
     on error.
  5. Cache and return `Awaiting` receipt with audit events + evidence refs.

**Acceptance:**
- `cargo check -p oya-workflow-engine-execution-engine-usecase --all-targets` passes
- Unit tests assert:
  - Fresh await → `Awaiting` receipt, `suspend_step_awaiting_signal` called once
  - Duplicate idempotency key with same fingerprint → identical receipt replayed
  - Mismatched fingerprint on same key → `IdempotencyConflict`
  - Invalid input (empty signal_name, bad tenant) → `InvalidInput`, no store call

---

### [WF-ENG-2] SignalDeliver usecase path — resume and idempotency

**Scope:** `crates/oya-workflow-engine-execution-engine-usecase/src/lib.rs`

Add:

- `SignalDeliverInput`:
  ```rust
  pub struct SignalDeliverInput {
      pub request_id: String,      // data_class: INTERNAL_ONLY
      pub idempotency_key: String, // data_class: INTERNAL_ONLY
      pub trace_ref: String,       // data_class: INTERNAL_ONLY
      pub tenant_id: String,       // data_class: INTERNAL_ONLY
      pub run_id: String,          // data_class: INTERNAL_ONLY
      pub signal_name: String,     // data_class: INTERNAL_ONLY
  }
  ```

- `SignalDeliverStatus` enum:
  - `Delivered` — awaiting step resumed
  - `Unmatched` — no active await for this correlation key (non-error typed receipt)
  - `IdempotencyConflict` — same key, different fingerprint
  - `InvalidInput` — missing/unsafe fields
  - `StoreUnavailable` — store error on resume write

- `SignalDeliverReceipt`:
  ```rust
  pub struct SignalDeliverReceipt {
      pub status: SignalDeliverStatus,
      pub tenant_id: String,
      pub run_id: String,
      pub signal_name: String,
      pub audit_events: Vec<ExecutionAuditEvent>,
      pub evidence_refs: Vec<String>,
  }
  ```

- `SignalDeliverStore` port extension:
  ```rust
  pub trait SignalDeliverStore: SignalAwaitStore {
      fn resume_step_on_signal(
          &mut self,
          tenant_id: &str,
          run_id: &str,
          signal_name: &str,
          evidence_ref: &str,
      ) -> Result<(), ExecutionStoreError>;
  }
  ```

- `impl SignalAwaitUsecase`:
  ```rust
  pub fn deliver_signal<S>(
      &mut self,
      store: &mut S,
      input: SignalDeliverInput,
  ) -> SignalDeliverReceipt
  where
      S: SignalDeliverStore,
  ```

  Logic:
  1. Validate input fields.
  2. Fingerprint + idempotency check.
  3. Call `store.load_await_record(...)`:
     - `Err` → `StoreUnavailable`
     - `None` or non-`Awaiting` record → `Unmatched` receipt (no store write, no panic)
  4. Call `store.resume_step_on_signal(...)` → `StoreUnavailable` on error.
  5. Cache and return `Delivered` receipt.

**Acceptance:**
- `cargo nextest run -p oya-workflow-engine-execution-engine-usecase` is green
- Tests prove:
  - Deliver after await → `Delivered`, `resume_step_on_signal` called once
  - Re-deliver same signal (same idempotency_key) → identical `Delivered` receipt,
    `resume_step_on_signal` called only once total
  - Signal with no prior await → `Unmatched` receipt (no panic, no store write)

---

### [WF-ENG-3] Timeout edge — timed-out receipt + audit evidence

**Scope:** `crates/oya-workflow-engine-execution-engine-usecase/src/lib.rs`

Add:

- `SignalTimeoutInput`:
  ```rust
  pub struct SignalTimeoutInput {
      pub request_id: String,              // data_class: INTERNAL_ONLY
      pub idempotency_key: String,         // data_class: INTERNAL_ONLY
      pub trace_ref: String,               // data_class: INTERNAL_ONLY
      pub tenant_id: String,               // data_class: INTERNAL_ONLY
      pub run_id: String,                  // data_class: INTERNAL_ONLY
      pub signal_name: String,             // data_class: INTERNAL_ONLY
      pub reference_epoch_seconds: u64,    // caller-supplied, no wall-clock
  }
  ```

- `SignalTimeoutStatus` enum:
  - `TimedOut` — await expired before delivery
  - `AlreadyDelivered` — signal already delivered before timer fired (no double-transition)
  - `InvalidInput`
  - `StoreUnavailable`

- `SignalTimeoutReceipt`:
  ```rust
  pub struct SignalTimeoutReceipt {
      pub status: SignalTimeoutStatus,
      pub tenant_id: String,
      pub run_id: String,
      pub signal_name: String,
      pub audit_events: Vec<ExecutionAuditEvent>,
      pub evidence_refs: Vec<String>,
  }
  ```

- `SignalTimeoutStore` port extension:
  ```rust
  pub trait SignalTimeoutStore: SignalDeliverStore {
      fn timeout_step_awaiting_signal(
          &mut self,
          tenant_id: &str,
          run_id: &str,
          signal_name: &str,
          evidence_ref: &str,
      ) -> Result<(), ExecutionStoreError>;
  }
  ```

- `impl SignalAwaitUsecase`:
  ```rust
  pub fn timeout_signal<S>(
      &mut self,
      store: &mut S,
      input: SignalTimeoutInput,
  ) -> SignalTimeoutReceipt
  where
      S: SignalTimeoutStore,
  ```

  Logic:
  1. Validate input.
  2. Idempotency check.
  3. Load await record:
     - `None` or `Delivered` → `AlreadyDelivered` (deterministic no-op, cached)
  4. Call `store.timeout_step_awaiting_signal(...)` → `StoreUnavailable` on error.
  5. Cache and return `TimedOut` receipt with `reference_epoch_seconds` in evidence refs.

**Acceptance:**
- `cargo nextest run -p oya-workflow-engine-execution-engine-usecase` green
- Table-style test asserts:
  - Timeout-before-delivery → `TimedOut` receipt + audit event with evidence ref
  - Delivery-before-timeout (await record status = Delivered) → `AlreadyDelivered`
  - Crate still performs zero I/O (source-level only, no new non-allowlisted deps)

---

## Acceptance summary

| Subtask | Gate command | Key assertion |
|---------|-------------|---------------|
| WF-ENG-1 | `cargo check -p oya-workflow-engine-execution-engine-usecase --all-targets` | Fresh await suspends, idempotency replay, conflict on mismatch |
| WF-ENG-2 | `cargo nextest run -p oya-workflow-engine-execution-engine-usecase` | Deliver resumes once, re-deliver idempotent, unmatched returns typed receipt |
| WF-ENG-3 | `cargo nextest run -p oya-workflow-engine-execution-engine-usecase` | Timeout-before-delivery = TimedOut; delivery-before-timeout = AlreadyDelivered; zero IO |

## Boundaries

- Single file: `crates/oya-workflow-engine-execution-engine-usecase/src/lib.rs`
- Lane docs: `docs/specs/task-workflow-engine-signal-await-durable-wait.md`
- No root `Cargo.toml` changes
- No kernel or domain crate changes
- No other crate changes
- No new workspace members
- No new Cargo dependencies
- Pure usecase: no DB, clock, network, filesystem, randomness, queue, Valkey, Postgres
