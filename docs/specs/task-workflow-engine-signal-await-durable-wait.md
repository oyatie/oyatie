# Spec: workflow-engine-signal-await-durable-wait

vertical: workflow
crate: workflow-engine-execution-engine-usecase
task: workflow-engine-signal-await-durable-wait
status: spec

## Objective

Extend the execution-engine usecase layer
(`crates/workflow-engine-execution-engine-usecase/src/lib.rs`) with a durable
signal/await orchestration slice. A running step can suspend awaiting a named external
signal on a `(tenant_id, run_id, signal_name)` correlation key and deterministically
resume (or time out) on delivery.

The slice:

1. Adds three new command paths — `await_signal`, `deliver_signal`, `timeout_signal`
   — each on a new `SignalAwaitUsecase` struct with its own idempotency map
2. Introduces three new port traits — `SignalAwaitStore`, `SignalDeliverStore`,
   `SignalTimeoutStore` — as source-level contracts for later durable adapters
3. Composes over the existing `SlaTimerStore` port for optional timeout arming
4. Preserves the same idempotency/fingerprint/audit-event pattern used by
   `ExecutionEngineUsecase`

All code is pure usecase — no DB, clock, network, filesystem, randomness, queue,
Valkey, Postgres, or cloud runtime work.

## Vertical and crate

```
vertical:  workflow
lane:      workflow-engine-signal-await-durable-wait
crate:     workflow-engine-execution-engine-usecase
path:      crates/workflow-engine-execution-engine-usecase/src/lib.rs
```

## Domain model additions

### Correlation key

The `(tenant_id, run_id, signal_name)` triple is the correlation key for all three
command paths. All three must be safe refs (tenant_id passes `is_safe_tenant`,
run_id and signal_name pass `is_safe_ref`).

### Port traits

#### SignalAwaitStore

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

#### SignalDeliverStore

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

#### SignalTimeoutStore

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

### SignalAwaitRecord

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignalAwaitRecord {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub run_id: String,       // data_class: INTERNAL_ONLY
    pub step_id: String,      // data_class: INTERNAL_ONLY
    pub signal_name: String,  // data_class: INTERNAL_ONLY
    pub delivered: bool,      // data_class: INTERNAL_ONLY
}
```

`delivered = false` → step is awaiting; `delivered = true` → already resumed by a
prior `deliver_signal` call. The `timeout_signal` path checks `delivered` to determine
`TimedOut` vs `AlreadyDelivered`.

### Input/status/receipt triples

#### WF-ENG-1 — AwaitSignal

```rust
pub struct SignalAwaitInput {
    pub request_id: String,
    pub idempotency_key: String,
    pub trace_ref: String,
    pub tenant_id: String,
    pub run_id: String,
    pub step_id: String,
    pub signal_name: String,
    pub timeout_timer: Option<SlaTimer>,
}

pub enum SignalAwaitStatus {
    Awaiting,
    IdempotencyConflict,
    InvalidInput,
    StoreUnavailable,
    TimerUnavailable,
}

pub struct SignalAwaitReceipt {
    pub status: SignalAwaitStatus,
    pub tenant_id: String,
    pub run_id: String,
    pub step_id: String,
    pub signal_name: String,
    pub timer_armed: bool,
    pub audit_events: Vec<SignalAuditEvent>, // separate enum; see SignalAuditEvent below
    pub evidence_refs: Vec<String>,
}
```

#### WF-ENG-2 — SignalDeliver

```rust
pub struct SignalDeliverInput {
    pub request_id: String,
    pub idempotency_key: String,
    pub trace_ref: String,
    pub tenant_id: String,
    pub run_id: String,
    pub signal_name: String,
}

pub enum SignalDeliverStatus {
    Delivered,
    Unmatched,
    IdempotencyConflict,
    InvalidInput,
    StoreUnavailable,
}

pub struct SignalDeliverReceipt {
    pub status: SignalDeliverStatus,
    pub tenant_id: String,
    pub run_id: String,
    pub signal_name: String,
    pub audit_events: Vec<SignalAuditEvent>, // separate enum; see SignalAuditEvent below
    pub evidence_refs: Vec<String>,
}
```

#### WF-ENG-3 — SignalTimeout

```rust
pub struct SignalTimeoutInput {
    pub request_id: String,
    pub idempotency_key: String,
    pub trace_ref: String,
    pub tenant_id: String,
    pub run_id: String,
    pub signal_name: String,
    pub reference_epoch_seconds: u64,  // caller-supplied, no wall-clock
}

pub enum SignalTimeoutStatus {
    TimedOut,
    AlreadyDelivered,
    InvalidInput,
    StoreUnavailable,
}

pub struct SignalTimeoutReceipt {
    pub status: SignalTimeoutStatus,
    pub tenant_id: String,
    pub run_id: String,
    pub signal_name: String,
    pub audit_events: Vec<SignalAuditEvent>, // separate enum; see SignalAuditEvent below
    pub evidence_refs: Vec<String>,
}
```

### SignalAwaitUsecase

```rust
#[derive(Default, Debug)]
pub struct SignalAwaitUsecase {
    // separate idempotency maps per command kind
    await_receipts:   BTreeMap<String, (SignalAwaitIntent,   SignalAwaitReceipt)>,
    deliver_receipts: BTreeMap<String, (SignalDeliverIntent, SignalDeliverReceipt)>,
    timeout_receipts: BTreeMap<String, (SignalTimeoutIntent, SignalTimeoutReceipt)>,
}

impl SignalAwaitUsecase {
    pub fn await_signal<S, T>(
        &mut self,
        store: &mut S,
        timers: &mut T,
        input: SignalAwaitInput,
    ) -> SignalAwaitReceipt
    where S: SignalAwaitStore, T: SlaTimerStore { ... }

    pub fn deliver_signal<S>(
        &mut self,
        store: &mut S,
        input: SignalDeliverInput,
    ) -> SignalDeliverReceipt
    where S: SignalDeliverStore { ... }

    pub fn timeout_signal<S>(
        &mut self,
        store: &mut S,
        input: SignalTimeoutInput,
    ) -> SignalTimeoutReceipt
    where S: SignalTimeoutStore { ... }
}
```

## Audit event kinds

New variants added to `SignalAuditEventKind` (separate enum, not merged into
`ExecutionAuditEventKind` to keep concerns distinct):

```rust
pub enum SignalAuditEventKind {
    AwaitRequested,
    AwaitSuspended,
    AwaitInvalid,
    AwaitIdempotencyConflict,
    SignalDelivered,
    SignalUnmatched,
    SignalDeliverInvalid,
    SignalDeliverIdempotencyConflict,
    SignalTimedOut,
    SignalAlreadyDelivered,
    SignalTimeoutInvalid,
}

impl SignalAuditEventKind {
    pub fn as_wire(self) -> &'static str { ... }
}
```

And a corresponding `SignalAuditEvent`:

```rust
pub struct SignalAuditEvent {
    pub kind: SignalAuditEventKind,
    pub tenant_id: String,
    pub run_id: String,
    pub signal_name: String,
    pub evidence_refs: Vec<String>,
}
```

## Idempotency and fingerprint contract

Each command path maintains its own `BTreeMap<String, (Intent, Receipt)>` exactly as
`ExecutionEngineUsecase` does. A `SignalAwaitIntent` (etc.) holds a canonical
fingerprint string built from the input fields using the same length-prefixed
`canonical_entry` scheme already in the usecase. Same-key same-fingerprint → replay
the cached receipt. Same-key different-fingerprint → `IdempotencyConflict` (not cached).

`InvalidInput` receipts are NOT cached (same rule as `ExecutionEngineUsecase`).

## Evidence ref contract

All evidence_refs vectors are `sorted_unique` — sorted and deduplicated strings, no
empty values. Wire strings use the prefix `workflow-signal-usecase:` to distinguish
from the existing `workflow-execution-usecase:` refs.

Examples:
- `"workflow-signal-usecase:awaiting"`
- `"workflow-signal-usecase:delivered"`
- `"workflow-signal-usecase:unmatched"`
- `"workflow-signal-usecase:timed-out"`
- `"workflow-signal-usecase:already-delivered"`
- `"workflow-signal-usecase:idempotency-conflict"`
- `"workflow-signal-usecase:invalid-input"`
- `"workflow-signal-usecase:store-unavailable"`
- `"workflow-signal-usecase:timer-unavailable"`

The `reference_epoch_seconds` from `SignalTimeoutInput` is emitted as a diagnostic
evidence ref: `format!("workflow-signal-usecase:reference-epoch:{}", value)`.

## Mod layout (flat clean-arch, single file)

The usecase crate is a single `src/lib.rs` (flat clean-arch per ADR-0509). All new
types land in that file, logically grouped:

```
// -- existing ExecutionEngineUsecase (unchanged) --

// -- signal port traits --
pub struct SignalAwaitRecord { ... }
pub trait SignalAwaitStore { ... }
pub trait SignalDeliverStore: SignalAwaitStore { ... }
pub trait SignalTimeoutStore: SignalDeliverStore { ... }

// -- signal audit types --
pub enum SignalAuditEventKind { ... }
pub struct SignalAuditEvent { ... }

// -- signal input/status/receipt types --
// (AwaitSignal, SignalDeliver, SignalTimeout triples)

// -- SignalAwaitUsecase --
pub struct SignalAwaitUsecase { ... }
impl SignalAwaitUsecase { ... }

// -- signal private helpers --
// (validation, fingerprint, audit helpers — private, not pub)

// -- existing tests (unchanged) --

// -- new #[cfg(test)] signal tests --
```

No new modules, no new files, no new crates.

## Contracts

### OpenAPI 3.2.0 surface (future adapter layer)

This crate has no HTTP surface; the contracts below describe the wire shape that the
adapter layer above will expose. Specified here for alignment:

```yaml
# POST /tenants/{tenant_id}/runs/{run_id}/signals/{signal_name}/await
# POST /tenants/{tenant_id}/runs/{run_id}/signals/{signal_name}/deliver
# POST /tenants/{tenant_id}/runs/{run_id}/signals/{signal_name}/timeout
```

All three are idempotency-key-gated (header: `Idempotency-Key`). Response bodies
carry `status`, `evidence_refs`, and `audit_events`. The usecase receipt maps 1:1 to
the JSON body shape.

### proto3 surface (future gRPC adapter)

```protobuf
// workflow/engine/v1/signal.proto
service SignalService {
  rpc AwaitSignal    (AwaitSignalRequest)   returns (SignalAwaitReceipt);
  rpc DeliverSignal  (DeliverSignalRequest) returns (SignalDeliverReceipt);
  rpc TimeoutSignal  (TimeoutSignalRequest) returns (SignalTimeoutReceipt);
}
```

Field mapping: input struct fields → proto message fields. `evidence_refs` →
`repeated string evidence_refs`. All correlation-key fields are `string` with
validation enforced in the usecase layer, not the proto layer.

## Testing strategy

All tests in `#[cfg(test)] mod tests` inside `src/lib.rs`. No new test files.

| Subtask | Test name | Assertion |
|---------|-----------|-----------|
| WF-ENG-1 | `await_signal_fresh_suspends_and_returns_awaiting` | `Awaiting`, store called once |
| WF-ENG-1 | `await_signal_duplicate_key_replays_identical_receipt` | Same receipt, store not called again |
| WF-ENG-1 | `await_signal_mismatched_key_yields_idempotency_conflict` | `IdempotencyConflict` |
| WF-ENG-1 | `await_signal_invalid_input_returns_invalid_no_store_call` | `InvalidInput`, zero store calls |
| WF-ENG-2 | `deliver_signal_after_await_resumes_exactly_once` | `Delivered`, resume called once total |
| WF-ENG-2 | `deliver_signal_redelivery_is_idempotent` | Replay receipt, resume still once total |
| WF-ENG-2 | `deliver_signal_no_prior_await_yields_unmatched` | `Unmatched`, no resume call, no panic |
| WF-ENG-3 | `timeout_signal_before_delivery_yields_timed_out` | `TimedOut`, timeout store called, audit event present |
| WF-ENG-3 | `timeout_signal_after_delivery_yields_already_delivered` | `AlreadyDelivered`, no timeout store call |
| WF-ENG-3 | `signal_slice_zero_io_source_level_only` | All three paths pass with in-memory fakes; no non-allowlisted deps |

Determinism gate: same input twice → byte-stable `evidence_refs` and `audit_events`.

## Boundaries

- Single file: `crates/workflow-engine-execution-engine-usecase/src/lib.rs`
- No root `Cargo.toml` changes
- No kernel or domain crate changes
- No other crate changes
- No new workspace members
- No new Cargo dependencies
- Pure usecase: no DB, clock, network, filesystem, randomness, queue, Valkey, Postgres
- `ExecutionEngineUsecase` and all existing public API remain unchanged

## Dependencies

No new Cargo dependencies. The crate already depends on
`workflow-engine-execution-engine-domain` (which re-exports the kernel types:
`SlaTimer`, `SlaTimerStore`, `ExecutionStoreError`, etc.). All new types compose over
already-imported items.
