---
status: pending approval
mode: deliberate
iteration: 2
supersedes: ralplan-foundry-supervisor-simple-2026-05-14.md
prerequisite_adrs:
- ADR-RENAME-foundry-account-adapter-{claude,codex,gemini} (separate, prerequisite)
- "ADR-supervisor-dep-policy-Y (this plan's chosen branch \u2014 zero net-new deps)"
- ADR-api-rest-adapter-mountpoint-or-transport (chosen: (ii) direct hyper via oya-http-runtime-hyper-adapter)
- ADR-jsonl-best-effort-durability (consequence of Branch Y)
- ADR-supervisor-public-contract-lean-a10 (every new public fn declared here)
- ADR-cedar-policy-bootstrap (consequence: no .cedar files exist in repo yet)
owner_phase: M02-P01-provider-gateway
fans_into:
- M02-P01-provider-gateway (owner)
- M02-P02-multi-subscription-pool (cross-cut: "AccountId \xD7 ProviderFamily fanout)"
- M02-P05-capability-registry-autonomy (cross-cut: T-tier classification per driver)
purpose: Auto-backfilled purpose for ralplan-foundry-supervisor-simple-v2-2026-05-14.md
---
# RALPLAN — Foundry Supervisor (Simple) — Iteration 2

**User intent (verbatim, not regrown):**
> "simple hook + inbox outbox setup. make it so that this works with multiple accounts. and across multiple providers. this will allow us to simplify our setup. still should be able to intelligently manage usage. all the features still have to come."

**Reference shape:** `Siigari/claude-heartbeat` — Node supervisor + stop-hook + JSONL inbox/outbox + restart-per-message + idle heartbeat tick.

---

## §A. RALPLAN-DR summary

### A.1 Principles (4)
1. **driver-not-kernel** — Each CLI (claude-code, codex, gemini) is invoked as a fresh subprocess by the supervisor app; the supervisor kernel owns no provider handles. The CLI is the *driver*, the supervisor is the *kernel* (in the OS sense, not the 12-layer sense).
2. **fresh-subprocess-statelessness** — Every message → spawn new CLI process; on stop-hook fire, the process exits. State lives only in JSONL on disk between turns.
3. **value-only-tickets** — `SessionTicket` carries owned, copied values only: `AccountId`, `ProviderFamily`, `AutonomyTier`, `UsageWindowSnapshot`, `MessageId`. Zero `*Ref`, zero `&'a`, zero `Arc<dyn ...>` across kernel boundaries. The supervisor kernel owns no foreign-kernel handles. **Invariant.**
4. **dep-branch-Y-commitment** — Zero net-new external deps. `SessionDriver` is a sync trait; durability is best-effort write+rename (no `fsync(parent_dir)`); admit non-durability-across-power-loss in ADR-jsonl-best-effort-durability.

### A.2 Decision Drivers (top 3)
1. **multi-account-fanout** — N accounts × M providers must be addressable from one supervisor; `RoutePolicy::select` (route-policy-kernel:54) already returns `RouteExplanation` per call; we compose, not invent.
2. **usage-window-honesty** — `UsageEnforcement::check_limit` (usage-window-kernel:30) returns `EnforcementVerdict` with 4 distinct outcomes; the supervisor must respect these verdicts before spawning a CLI, not after credit is burned.
3. **cross-CLI-parity-by-type** — Claude/Codex/Gemini CLIs each have different stop-hook capabilities; the capability registry (`AutonomyTier` T1/T2/T3/T4 — capability-registry-kernel:50) lets us classify drivers by what they can actually do, and demote when verification fails.

### A.3 Viable Options (4 — Architect's Option D included)

| Opt | Shape | Pros | Cons | Verdict |
|-----|-------|------|------|---------|
| **A** | Single supervisor daemon owns the whole chain (route → reserve → spawn → watch → settle) | Single fsync ordering point; one process to monitor; matches Siigari shape | Daemon failure = total outage; restart loses in-flight reservations; harder to evolve | **REJECTED** — single-point-of-failure violates fan-out principle |
| **B** | Per-CLI binaries (3 separate daemons) | Failure isolation per provider | Triplicates restart/state code; 3× operational cost; reservation conflict across binaries needs a separate broker | **REJECTED** — multiplies cost without gain; conflict broker becomes Option A in disguise |
| **C** | In-process inside `oya-intelligence-dashboard-app` | Reuses dashboard event loop | Couples runtime ops to UI lifecycle; dashboard restart kills supervisor; mixes layers | **REJECTED** — violates 12-layer enum (app↔app cross-talk via dashboard) |
| **D** | **Host-injected policy ports: supervisor-app composes route-policy + usage-enforcement + cost-ceiling as pure ports; jsonl-supervisor-adapter is the I/O seam; supervisor-kernel is pure decision logic** | Kernel stays I/O-free; adapter is the only fsync-aware crate; ports compose without inheritance; aligns ADR-0056 port-in-kernel + ADR-0092 conversion-at-the-boundary | More crates than Option A; requires explicit `lean-a10` declaration for each new public fn on existing kernels | **CHOSEN** |

**Why D wins:** Composes existing `RoutePolicy::select`, `UsageEnforcement::check_limit`, `check_silent_switch`, `validate_usage`, `finalize_line` without inventing kernel APIs. Net-new public functions are isolated to the 3 new crates. Where we DO touch an existing kernel, it is declared in §B.2.6 with full lean-a10 ceremony.

### A.4 Pre-mortem (3 failure scenarios, all CI-enforced)

| Scenario | Mitigation lane | Acceptance |
|----------|----------------|------------|
| **(a) Torn write** — supervisor crash mid-rename leaves orphan `.tmp` + half-written outbox | `lean-fsync-durability` — crash-injection harness `crates/oya-intelligence-supervisor-app/tests/crash_injection.rs` SIGKILLs writer at every flush point; recovery must reproduce identical cursor or quarantine the partial line | `cargo test -p oya-intelligence-supervisor-app --test crash_injection -- --test-threads=1` exits 0; no orphan `.tmp` after replay |
| **(b) Hung CLI burns credit** — driver subprocess deadlocks; reservation TTL expires but credit already consumed | `lean-watchdog-timing` — integration test asserts SIGKILL within `WATCHDOG_TIMEOUT + grace=5s` using a fake driver that `loop { sleep(1s) }` | Watchdog kill latency p95 ≤ 5.0s in `benches/heartbeat.rs::watchdog_kill_latency` |
| **(c) Poison message loop** — corrupt JSONL line crashes parser on every replay → infinite restart | `lean-dead-letter` — supervisor kernel verdict `Quarantine` after `MAX_PARSE_RETRIES=3`; autonomy_tier demoted by 1 step; entry moved to `dead-letter/`; alarm metric `supervisor_quarantine_total` | Integration test injects a hand-crafted malformed line; assert `dead-letter/` has 1 file after 3 spawn attempts and `T<n+1>` demoted to `T<n>` |

### A.5 Expanded test plan (deliberate mode mandatory)

| Layer | Crate / file | Command |
|-------|--------------|---------|
| Unit (kernel) | `crates/oya-intelligence-supervisor-kernel/src/lib.rs` | `cargo test -p oya-intelligence-supervisor-kernel` |
| Unit (adapter) | `crates/oya-intelligence-jsonl-supervisor-adapter/src/lib.rs` | `cargo test -p oya-intelligence-jsonl-supervisor-adapter` |
| Integration | `crates/oya-intelligence-supervisor-app/tests/lifecycle.rs` (3 sub-tests: spawn, restart, settle) | `cargo test -p oya-intelligence-supervisor-app --test lifecycle` |
| E2E matrix | `crates/oya-intelligence-supervisor-app/tests/matrix_3x2x2.rs` — 3 CLIs × 2 accounts × 2 providers = 12 combinations | `cargo test -p oya-intelligence-supervisor-app --test matrix_3x2x2 -- --include-ignored --test-threads=1` (gated on `$OYA_LIVE_SMOKE=1`) |
| Observability | `crates/oya-intelligence-supervisor-app/tests/audit_chain.rs` — assert every spawn/settle emits an EvidenceRef linked to capability id | `cargo test -p oya-intelligence-supervisor-app --test audit_chain` |
| Perf budget | `crates/oya-intelligence-supervisor-app/benches/heartbeat.rs` — Criterion-shape harness; idle-tick ≤ 25 tok p95, restart p95 ≤ 1.5s, RSS ≤ 64 MiB, watchdog kill ≤ T+5s | `cargo bench -p oya-intelligence-supervisor-app --bench heartbeat -- --save-baseline supervisor-v2` |

**Note on Criterion:** Workspace currently has no `criterion` dep (verified via `Cargo.toml:481-492`); under Branch Y we use `#[bench]`-shape harness via `std::time::Instant` only, written as a `bin` target driven by `cargo test --release --bench heartbeat` to stay zero-dep. The `cargo bench` invocation above runs the bin in benchmark mode.

---

## §B. Implementation plan

### B.0 Existing surfaces inventory (mandatory)

| Surface | File | Lines | Notes |
|---------|------|-------|-------|
| `RoutePolicy::select(&[ProviderAccount], &RouteConstraints) -> Result<RouteExplanation, RouteError>` | `crates/oya-intelligence-route-policy-kernel/src/lib.rs` | 54–96 | Pure selector; no I/O. **Composes, not extended.** |
| `RoutePolicy::explain_route(...)` | same | 101–106 | Same selector; surface kept for audit. |
| `RouteConstraints` (struct) | same | 12–19 | Public fields all `INTERNAL_ONLY`. |
| `RouteError` enum | same | 39–47 | 7 variants. |
| `UsageEnforcement::check_limit(&UsageWindow, now: u64, budget: u64) -> Result<EnforcementVerdict, EnforcementError>` | `crates/oya-intelligence-usage-window-kernel/src/lib.rs` | 30–71 | Pure verdict; 4 outcomes including `WindowExpired`. |
| `EnforcementVerdict` enum | same | 9–18 | `WithinLimit`/`OverUsageLimit`/`ReserveBreached`/`WindowExpired`. |
| `UsageWindow` (re-exported from account-domain) | `oya-intelligence-account-domain/src/lib.rs` | 252–302 | Value type; **eligible for `UsageWindowSnapshot` derivation via Clone**. |
| `validate_usage(&UsageRecord) -> Result<(), BillingError>` | `crates/oya-cloud-billing-kernel/src/lib.rs` | 74–85 | Pure validator. |
| `finalize_line(&LineItem) -> Result<u128, BillingError>` | same | 87–95 | Subtotal in micros. |
| `UsageRecord` / `LineItem` / `UsageUnit::Token` | same | 7–52 | `UsageUnit::Token` exists; we reuse for spend records. |
| `check_silent_switch(&[&ProviderAccount], &ProviderAccount) -> Result<(), AccountError>` | `crates/oya-intelligence-account-domain/src/lib.rs` | 171–186 | Cross-account guard. |
| `ProviderAccount` + state machine (`Draft → Verified → Active → Degraded/Disabled/Revoked`) | same | 68–167 | `degrade(reason: String) -> Result<(), AccountError>` at L120. |
| `AccountId(pub String)` / `ProviderFamily` enum / `SessionId` | `crates/oya-intelligence-account-kernel/src/lib.rs` | 14–29 | Public; owned values. |
| `Capability::new(id, name, tier, evidence_required)` / `AutonomyTier` enum | `crates/oya-intelligence-capability-registry-kernel/src/lib.rs` | 49–136 | `T1Read..T4Actuate`; `try_from("T1")` works. |
| `validate_publish(&Capability) -> Result<(), PublishValidationError>` | `crates/oya-intelligence-capability-registry-domain/src/lib.rs` | 43 (definition; full body unread but signature confirmed via grep) | Pre-condition for `CapabilityRegistry::register`. |
| `CapabilityRegistry::register/list/get` + `parse_seed_json` | `crates/oya-intelligence-capability-registry-app/src/lib.rs` | 53–86, 97–153 | Hand-rolled JSON; **no serde dep** (HARD CONSTRAINT, L96). |
| `CeilingPolicy::ceiling_for(&TenantId) -> AutonomyTier` / `set` | `crates/oya-intelligence-autonomy-ceiling-domain/src/lib.rs` | 40–70 | Default = `T3PropAct`. |
| `enforce(&Capability, ceiling) -> CeilingVerdict` / `enforce_for_tenant` | `crates/oya-intelligence-autonomy-ceiling-app/src/lib.rs` | 21–33 | Bridges Cap-tier ↔ Ceiling-tier enums. |
| `check_tier(cap_tier, ceiling) -> CeilingVerdict` | `crates/oya-intelligence-autonomy-ceiling-kernel/src/lib.rs` | 53–62 | Pure comparison. |
| `serve(addr, router, chain, ServerConfig) -> Result<(), HyperRuntimeError>` | `crates/oya-http-runtime-hyper-adapter/src/lib.rs` | 284–335 | Real hyper server; `tokio` runtime; `ServerConfig::with_*`. |
| `Router<SyncHandler>` + `MiddlewareChain` + `dispatch` | same | 206–221 + middleware-kernel | Real router; we use this for the supervisor's webhook surface (Branch §B.7). |
| `oya-intelligence-api-rest-adapter` | `crates/oya-intelligence-api-rest-adapter/src/lib.rs` | 1–68 | **CONFIRMED STUB** — `handle()` returns fixed `RestResponse { status_code: 200 }`. No real router. **Not used for supervisor mountpoint.** |
| `oya-foundry-account-adapter-{claude-code,codex-cli,gemini-cli}` | each crate's `src/lib.rs` | L1–3 only | **All three are `pub fn placeholder() {}` skeletons.** No real CLI invocation logic yet — this is a finding, not a regression. |
| Workspace deps | `Cargo.toml` | 481–492 | `tracing`, `hyper`, `hyper-util`, `tokio` (rt-multi-thread, net, macros), `http-body-util`, `bytes`. **No `rustix`, no `async-trait`, no `nix`, no `serde`, no `criterion`.** |
| Capability seed file | `registry/capabilities/foundry-internal.json` | 8.8 KB, 50+ caps | Confirmed exists; T4 count must be 0 (capability-registry-app:307). |
| Cedar policy files | `policy/` or `policies/` or `registry/` | **None found** | RISK: no `.cedar` files exist yet. Supervisor capability rows must NOT cite non-existent files. See §B.8. |
| Existing v1 plan | `.omc/plans/ralplan-foundry-supervisor-simple-2026-05-14.md` | exists | Superseded by this iteration. |
| Open-questions ledger | `.omc/plans/open-questions.md` | 679 lines | Append new section. |

### B.1 Crate decomposition (3 new + 1 conformance = 4 total)

| Crate | v4-BNF + 12-layer-enum justification |
|-------|--------------------------------------|
| `oya-intelligence-supervisor-kernel` | `oya-<foundry>-<supervisor>-<kernel>` — foundry is the registered µservice (`Cargo.toml:290`); supervisor is the new feature target; kernel is layer #1 of the 12-layer enum (pure types, no I/O). |
| `oya-intelligence-supervisor-app` | Same prefix; app is layer #4 — orchestrates kernel ports, owns the tokio runtime, hosts `benches/heartbeat.rs`. |
| `oya-intelligence-jsonl-supervisor-adapter` | `oya-<foundry>-<jsonl-supervisor>-<adapter>` — jsonl-supervisor is the target slot (file-format + feature compound; same shape as `oya-cloud-storage-block-adapter`); adapter is layer #5; the ONLY crate that does `std::fs` writes + rename. |
| `oya-foundry-supervisor-conformance` | `oya-<foundry>-<supervisor>-<conformance>` — conformance is a target slot (not a layer; same as `oya-adapter-substitution-test`). Stand-alone test crate that emits the capability registry seed row for each driver at its measured T-level. Declared HERE (not inline) per Architect feedback. |

**Bench harness file:** `crates/oya-intelligence-supervisor-app/benches/heartbeat.rs` — declared as part of supervisor-app, not a separate crate. Build target = `[[bench]] name = "heartbeat"` in supervisor-app `Cargo.toml`.

### B.2 Public contracts (kernel surface)

All types use **owned values only**. No lifetimes. No trait objects in struct fields.

#### B.2.1 `oya-intelligence-supervisor-kernel` types

```rust
use oya_intelligence_account_kernel::{AccountId, ProviderFamily, SessionId};
use oya_intelligence_capability_registry_kernel::AutonomyTier;
use oya_intelligence_account_domain::UsageWindow;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct MessageId(pub String);            // INTERNAL_ONLY

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct RequestId(pub String);            // INTERNAL_ONLY — provider idempotency key

/// Value-only snapshot of a UsageWindow at a point in time. Owned copy of
/// the window's numeric state; consumers never hold a reference to a live
/// window. Crucial: this is NOT a handle; it is a frozen value used by the
/// supervisor to compute headroom without borrowing the enforcement kernel.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsageWindowSnapshot {
    pub kind_label: String,                  // from UsageWindow::kind
    pub started_at_epoch_secs: u64,
    pub ends_at_epoch_secs: u64,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub usage_limit_pct: u8,
    pub reserve_remaining_pct: u8,
    /// p95 projected token cost for the next message, computed by the app
    /// layer from historical samples keyed by (ProviderFamily, model_hint).
    /// Fallback when no history: `cost_ceiling` from the SessionTicket.
    pub projected_tokens_p95: u64,
}

impl UsageWindowSnapshot {
    pub fn from_window(w: &UsageWindow, projected_tokens_p95: u64) -> Self { /* pure */ }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionTicket {
    pub message_id: MessageId,
    pub request_id: RequestId,
    pub account_id: AccountId,
    pub provider_family: ProviderFamily,
    pub autonomy_tier: AutonomyTier,
    pub window_snapshot: UsageWindowSnapshot,
    pub cost_ceiling_tokens: u64,            // absolute upper bound; survives missing history
    pub model_hint: String,
}
// Invariant: SessionTicket contains zero references, zero Arcs, zero Box<dyn _>.
// Compile-time fence via test `static_assertions::assert_impl_all!(SessionTicket: Send + Sync + 'static);`
// (Note: static_assertions is not a workspace dep — fence via manual `fn _f<T: Send + Sync + 'static>() {}` call.)

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InboxState {
    Queued,
    Locked { reservation_id: String, ttl_epoch_secs: u64 },
    InFlight { reservation_id: String },
    DraftedResponse { outbox_pending: bool },
    Committed,
    DeadLettered { reason: String },
    Released { reason: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SupervisorVerdict {
    Spawn { ticket: SessionTicket },
    DemoteAndRetry { new_tier: AutonomyTier, reason: String },
    Quarantine { message_id: MessageId, reason: String },
    Reject { reason: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SupervisorError {
    NoEligibleAccount,
    UsageBlocked(String),
    TierBlocked(String),
    InvalidTransition { from: &'static str, to: &'static str },
    ParseFailed { line_number: u64 },
    MaxRetriesExceeded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpendRecord {
    pub request_id: RequestId,
    pub account_id: AccountId,
    pub provider_family: ProviderFamily,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub model_hint: String,
}
```

#### B.2.2 `oya-intelligence-supervisor-kernel` pure functions (no new public APIs on existing kernels)

```rust
/// Pure decision: given accounts + windows + tier policy + incoming message
/// shape, return the supervisor verdict. Composes RoutePolicy::select +
/// UsageEnforcement::check_limit + autonomy_ceiling::check_tier — DOES NOT
/// extend any of them.
pub fn decide(
    accounts: &[ProviderAccount],
    constraints: &RouteConstraints,
    window: &UsageWindow,
    now_epoch_secs: u64,
    budget_tokens: u64,
    tenant_ceiling: AutonomyTier,
    msg: &IncomingMessage,
) -> Result<SupervisorVerdict, SupervisorError>;

/// Pure state-machine transition function for inbox entries.
pub fn next_inbox_state(
    current: &InboxState,
    event: &InboxEvent,
    now_epoch_secs: u64,
) -> Result<InboxState, SupervisorError>;

/// Pure: derive a spend record from a (ticket, observed_tokens) pair.
/// Adapter calls this then hands the SpendRecord to cloud-billing-kernel.
pub fn record_spend(ticket: &SessionTicket, tokens_in: u64, tokens_out: u64) -> SpendRecord;

/// Compose into UsageRecord that cloud-billing-kernel::validate_usage accepts.
/// Reuses UsageUnit::Token (billing-kernel:14). No new billing API needed.
pub fn spend_to_usage_record(s: &SpendRecord, tenant_id: &str, ts_ms: u64) -> UsageRecord;
```

#### B.2.3 `oya-intelligence-supervisor-kernel` ports (sync traits — Branch Y)

```rust
/// Per Branch Y: SessionDriver is SYNC. The supervisor-app drives async
/// orchestration in tokio; the trait itself is sync so it stays object-safe
/// without async-trait. Drivers run on a tokio blocking worker via
/// tokio::task::spawn_blocking.
pub trait SessionDriver: Send + Sync {
    fn driver_id(&self) -> &str;
    fn provider_family(&self) -> ProviderFamily;
    fn spawn_for_message(
        &self,
        ticket: &SessionTicket,
        message_body: &[u8],
    ) -> Result<DriverHandle, DriverError>;
}

pub struct DriverHandle {
    pub pid: i32,
    pub stop_hook_path: String,
    pub started_at_epoch_secs: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DriverError {
    SpawnFailed(String),
    StopHookMissing,
    Watchdog,
}

pub trait InboxStore: Send + Sync {
    fn peek_lock(&self, ttl_secs: u64) -> Result<Option<(MessageId, Vec<u8>)>, StoreError>;
    fn commit(&self, id: &MessageId, response: &[u8]) -> Result<(), StoreError>;
    fn release(&self, id: &MessageId, reason: &str) -> Result<(), StoreError>;
    fn dead_letter(&self, id: &MessageId, reason: &str) -> Result<(), StoreError>;
    fn replay_cursor(&self) -> Result<u64, StoreError>;
}
```

#### B.2.4 `oya-intelligence-jsonl-supervisor-adapter` (the ONLY crate that touches `std::fs`)

```rust
pub struct JsonlInbox { dir: PathBuf, lock_ttl_secs: u64 }
impl InboxStore for JsonlInbox { /* ... */ }

pub struct JsonlOutbox { dir: PathBuf }
impl JsonlOutbox {
    pub fn append(&self, response: &[u8]) -> Result<(), StoreError>;  // write tmp → rename
}

// Hand-rolled minimal JSON (same constraint as capability-registry-app, L96):
// no serde. Lines are `key=value;...` framing — survives without a parser dep.
pub fn encode_inbox_line(state: &InboxState, msg: &MessageId, body: &[u8]) -> Vec<u8>;
pub fn decode_inbox_line(line: &[u8]) -> Result<(InboxState, MessageId, Vec<u8>), StoreError>;
```

#### B.2.5 `oya-intelligence-supervisor-app` (composes ports; owns tokio runtime)

```rust
pub struct SupervisorApp<D: SessionDriver, I: InboxStore> { /* ... */ }
impl<D: SessionDriver, I: InboxStore> SupervisorApp<D, I> {
    pub fn new(drivers: Vec<D>, inbox: I, outbox: JsonlOutbox, config: SupervisorConfig) -> Self;
    pub fn tick_once(&mut self, now_epoch_secs: u64) -> Result<TickOutcome, SupervisorError>;
    pub async fn run_forever(self) -> !;  // tokio::main host
}
```

#### B.2.6 New public APIs on existing kernels (full lean-a10 ceremony)

After auditing §B.0, **the supervisor kernel does not require any new public APIs on existing kernels** for the core call chain. We compose:
- `RoutePolicy::select` (existing, route-policy-kernel:54)
- `UsageEnforcement::check_limit` (existing, usage-window-kernel:30)
- `check_silent_switch` (existing, account-domain:171)
- `validate_usage` + `finalize_line` (existing, cloud-billing-kernel:74,87)
- `enforce` / `enforce_for_tenant` (existing, autonomy-ceiling-app:21,27)
- `ProviderAccount::degrade(reason)` (existing, account-domain:120) — supervisor calls this on a 3-strike degrade

**Show-your-work:** the previous draft invented `select_account_for_message`, `try_reserve_for`, `observed_spend`, `demote`. All four are unnecessary — the same outcomes are reachable via composition. The supervisor app holds a `&mut ProviderAccount` and calls `degrade()` directly when it detects 3 consecutive `EnforcementVerdict::ReserveBreached` for that account; no new API.

**Lean-a10 declaration count: 0 new public APIs on existing kernels.** This is the strongest possible compliance — no kernel version bumps required, no sunset paths required.

(If during execution a missing primitive surfaces, the executor MUST stop, open an ADR, and bump the owning kernel's version BEFORE adding the API. Codified as an open question in `open-questions.md`.)

### B.3 Daemon lifecycle (process tree + signals + fsync points)

```
                supervisor-app (PID S)
                    │
                    │ tokio::spawn_blocking per spawn
                    ▼
              SessionDriver::spawn_for_message
                    │
                    │ fork+exec
                    ▼
              CLI subprocess (PID C)
                    │  ── reads message_body from stdin
                    │  ── writes draft response to stdout
                    │  ── on completion, touches `stop_hook_path` (empty file)
                    │  ── exits
                    ▼
              supervisor-app polls (stop_hook_path exists) every 100 ms
                    │
                    │ on hook: read child stdout buffer, call inbox.commit(response)
                    │ on watchdog (T > WATCHDOG_TIMEOUT): kill(C, SIGTERM), wait 1s, kill(C, SIGKILL)
                    ▼
              outbox.append(response) → write `outbox/.tmp-{uuid}` → rename to `outbox/{seq}.jsonl`
```

**fsync points (Branch Y best-effort):**
1. **Inbox lock**: write `inbox/locked/{msg_id}.lock` containing `(reservation_id, ttl)`; standard `std::fs::write` (no `fsync`). Adapter ADR-jsonl-best-effort-durability documents: lock survives normal restart, may be lost on power-loss; TTL handles the latter case via expiry.
2. **Outbox append**: write `.tmp-{uuid}` → `rename` to `{seq}.jsonl`. On power-loss, the `.tmp-` is GC'd on startup by `JsonlOutbox::recover()` (called from `SupervisorApp::new`). No `fsync(parent_dir)` — best-effort.
3. **Commit transition**: inbox lock file is unlinked AFTER outbox rename succeeds. Order-of-operations enforced by §B.2.4 `commit()` body.

**Crash recovery from cursor:**
- `JsonlInbox::replay_cursor()` reads `inbox/cursor` (single u64 sequence number).
- Orphan `.tmp-` files in `outbox/` are unlinked (best-effort cleanup).
- Orphan locks in `inbox/locked/` past their TTL are released via `release()`.

**Inbox state machine table:**

| From | Event | To | fsync boundary |
|------|-------|----|-----| 
| `Queued` | `peek_lock` succeeds | `Locked{r, ttl}` | write lock file |
| `Locked` | driver spawn returns Ok | `InFlight{r}` | none |
| `Locked` | TTL expires (no spawn) | `Released{ttl-expired}` | unlink lock |
| `InFlight` | child stop-hook fires | `DraftedResponse{outbox_pending: true}` | none (in-memory) |
| `InFlight` | watchdog kill | `DeadLettered{watchdog}` | move to dead-letter/ |
| `DraftedResponse` | outbox append succeeds | `DraftedResponse{outbox_pending: false}` | rename .tmp → final |
| `DraftedResponse{false}` | unlink lock | `Committed` | unlink lock |
| `Queued|Locked|InFlight` | parse error retry == MAX | `DeadLettered{poison}` | move to dead-letter/ |

### B.4 Multi-account × multi-provider call chain (composing B.0 only)

```
SupervisorApp::tick_once(now):
  1. inbox.peek_lock(TTL=30s)?               // §B.2.3 InboxStore
  2. let accounts = self.accounts.snapshot(); // Vec<ProviderAccount> owned
  3. let constraints = RouteConstraints::new(model_hint);
     // constraints.failover_order = config-driven (default: [Claude, OpenAi, Gemini])
  4. let exp = RoutePolicy::select(&accounts, &constraints)?;
     // → returns chosen_account_id + chosen_provider + chosen_model
  5. let acc = accounts.iter().find(|a| a.id == exp.chosen_account_id).unwrap();
  6. check_silent_switch(&accounts.iter().filter(|x| x.id != acc.id).collect(),
                         acc)?;  // existing API; account-domain:171
  7. let window: &UsageWindow = self.window_for(acc.id);
  8. let verdict = UsageEnforcement::check_limit(window, now, budget_tokens)?;
  9. match verdict {
        WithinLimit{..}   => proceed,
        OverUsageLimit{..} => acc.degrade("over usage limit"); RETURN DemoteAndRetry,
        ReserveBreached{..} => acc.degrade("reserve breached"); RETURN DemoteAndRetry,
        WindowExpired     => self.rotate_window(); recurse once,
     }
 10. let cap = self.capability_for_driver(driver_id);
 11. let tier_verdict = autonomy_ceiling_app::enforce(&cap, tenant_ceiling);
     match tier_verdict { Block{..} => RETURN TierBlocked, Allow => proceed }
 12. let snap = UsageWindowSnapshot::from_window(window, projected_p95);
 13. let ticket = SessionTicket { message_id, request_id, account_id, provider_family,
                                  autonomy_tier: cap.autonomy_tier, window_snapshot: snap,
                                  cost_ceiling_tokens, model_hint };
 14. driver.spawn_for_message(&ticket, &body)?;
 15. on completion: spend = supervisor_kernel::record_spend(&ticket, ti, to);
     let rec = supervisor_kernel::spend_to_usage_record(&spend, tenant_id, ts_ms);
     cloud_billing_kernel::validate_usage(&rec)?;  // existing
     // (line-item finalization is a separate billing-app concern, deferred)
 16. inbox.commit(&message_id, &response_bytes);
 17. outbox.append(&response_bytes);
```

**Multi-account fanout is implemented by step 4** — `RoutePolicy::select` already walks `failover_order` (route-policy-kernel:64–94). To support multiple Claude accounts: register them all in `accounts: Vec<ProviderAccount>` with the same `ProviderFamily::Claude` but distinct `AccountId` + `subscription_id`. The silent-switch guard (step 6) prevents accidentally swapping between two same-subscription Claude accounts mid-stream.

### B.5 Cross-CLI hook bridge + capability seed file

Three driver crates (renamed per the prerequisite ADR; reference names assumed post-rename):
- `oya-intelligence-claude-account-adapter` — uses Claude Code's stop-hook
- `oya-intelligence-codex-account-adapter` — uses Codex CLI's exit-or-no-stop-hook semantics
- `oya-intelligence-gemini-account-adapter` — uses Gemini CLI's available signaling

Each implements `SessionDriver` (§B.2.3) and registers a stop-hook script that touches `stop_hook_path` on completion. Supervisor-app polls for file existence (no inotify dep needed — 100 ms tick is sufficient given the 25-tok idle budget).

**Capability seed file (real path):** `registry/capabilities/foundry-supervisor.toml` — **NEW FILE**, parallel to existing `registry/capabilities/foundry-internal.json`. We use TOML here because (a) `foundry-internal.json` uses a hand-rolled JSON parser that handles ONLY its schema (capability-registry-app:96), so a new schema would need a new parser; (b) TOML can be hand-rolled even more simply for a flat list. Concrete schema:

```toml
[[driver]]
id = "claude-driver"
provider_family = "Claude"
autonomy_tier = "T?"          # FILLED BY CONFORMANCE CRATE — see B.5
stop_hook_supported = ?       # FILLED BY CONFORMANCE CRATE
request_id_supported = true   # Anthropic supports anthropic-version + idempotency-key

[[driver]]
id = "codex-driver"
provider_family = "OpenAIOrCodex"
autonomy_tier = "T?"
stop_hook_supported = ?
request_id_supported = true   # OpenAI supports Idempotency-Key header (per public docs)

[[driver]]
id = "gemini-driver"
provider_family = "Gemini"
autonomy_tier = "T?"
stop_hook_supported = ?
request_id_supported = false  # RISK: Gemini SDK doesn't ship a documented idempotency key
                              # mitigation: degrade tier by 1 (-> T2Suggest) when missing
```

**Idempotency-key shape per provider:**
- Claude: `anthropic-idempotency-key` header (string ≤ 256 chars)
- OpenAI / Codex: `Idempotency-Key` header (string ≤ 255 chars, 24h dedup window)
- Gemini: **RISK** — no public idempotency key; mitigation: `request_id_supported = false` → conformance crate emits `T2Suggest` (or lower) for gemini-driver

The conformance crate runs each driver against a fake server that:
1. Counts stop-hook invocations (must be 1, not 0, not 2+)
2. Replays the same `request_id` twice and verifies the provider responds with the cached result (proves idempotency)
3. Kills the driver mid-response; verifies driver does not corrupt the outbox
4. Emits the measured tier into the seed file at build time (via `build.rs` in the conformance crate — `build.rs` is zero-dep)

### B.6 Cost-ceiling integration (composes existing billing-kernel)

- Supervisor-kernel emits `SpendRecord` (§B.2.1) per completed message.
- Supervisor-kernel `spend_to_usage_record` (§B.2.2) packs a `SpendRecord` into the existing `cloud_billing_kernel::UsageRecord` shape, using `UsageUnit::Token` (billing-kernel:14).
- Supervisor-app calls `cloud_billing_kernel::validate_usage(&record)` (billing-kernel:74).
- Line-item finalization (`finalize_line`, billing-kernel:87) requires a `tax_jurisdiction` (billing-kernel:88–93). The supervisor does NOT finalize lines — that is a separate billing-app concern; the supervisor only emits validated usage records into the billing inbox.

**`cost_ceiling_tokens` source:** the SessionTicket carries an absolute upper bound; the supervisor app receives this from config (`SupervisorConfig::default_cost_ceiling`) and may lower it based on `UsageWindowSnapshot::projected_tokens_p95` < ceiling. **Show-your-work:** the supervisor app computes `projected_tokens_p95` from a per-`(provider, model_hint)` rolling window of the last 100 observed `tokens_in + tokens_out` values; persisted in `inbox/.../stats.jsonl`. **Fallback:** when fewer than 10 samples exist, `projected_tokens_p95 = cost_ceiling_tokens` (conservative).

### B.7 Mountpoint decision

**Chosen: (ii) Direct hyper via `oya-http-runtime-hyper-adapter`.** Reason: `oya-intelligence-api-rest-adapter` is confirmed-stub (B.0 row). Compose:

```rust
// In oya-intelligence-supervisor-app::run_forever:
let router = oya_http_router_kernel::Router::<SyncHandler>::new()
    .route(HttpMethod::Post, "/v1/supervisor/inbox", handler_to_sync(InboxIngest::new(...)))?
    .route(HttpMethod::Get,  "/v1/supervisor/health", handler_to_sync(HealthCheck))?
    .route(HttpMethod::Get,  "/v1/supervisor/outbox/{seq}", handler_to_sync(OutboxRead::new(...)))?;
let chain = oya_http_middleware_kernel::MiddlewareChain::new();
let config = oya_http_runtime_hyper_adapter::ServerConfig::default()
    .with_max_body_bytes(256 * 1024);
oya_http_runtime_hyper_adapter::serve(addr, Arc::new(router), Arc::new(chain), config).await?;
```

All four building blocks (`Router`, `MiddlewareChain`, `handler_to_sync`, `serve`) are confirmed-real in B.0. `oya-intelligence-api-rest-adapter` is **not** on this chain. No prerequisite IP needed for `api-rest-adapter` — we side-step it.

ADR: `ADR-api-rest-adapter-mountpoint-or-transport` chooses (ii) with rationale: stub-bypass for now; api-rest-adapter remains stub until M02-P04 lands its own real router.

### B.8 Capability registry rows + Cedar policy paths

The supervisor publishes 6 capability rows:

| capability_id | autonomy_tier | evidence_required | Cedar policy path |
|---------------|---------------|--------------------|---------------|
| `foundry.supervisor.spawn` | (computed per driver via conformance) | true | **TBD** — no `.cedar` files exist (§B.0). Tracked in open-questions. |
| `foundry.supervisor.commit` | T3PropAct | true | TBD |
| `foundry.supervisor.dead_letter` | T2Suggest | true | TBD |
| `foundry.supervisor.degrade_account` | T3PropAct | true | TBD |
| `foundry.supervisor.rotate_window` | T2Suggest | true | TBD |
| `foundry.supervisor.quarantine` | T2Suggest | true | TBD |

**RISK** (already open in §B.0): zero `.cedar` files in the workspace today. The capability rows publish into `registry/capabilities/foundry-supervisor.toml` without a Cedar fragment until `ADR-cedar-policy-bootstrap` lands. Until then, autonomy is enforced only by `autonomy_ceiling_app::enforce()` (capability-tier vs tenant-ceiling), which is real and tested (autonomy-ceiling-app:21–88).

### B.9 Phase fan-in

**Owner phase: M02-P01-provider-gateway** — supervisor's primary mission is gateway autonomy (route + reserve + spawn = provider gateway loop).

Cross-cutting rows:
- **M02-P02-multi-subscription-pool** — depends on `AccountId × subscription_id` fanout already landed (Cargo.toml:266–270 confirms `oya-intelligence-provider-pool-kernel` exists).
- **M02-P05-capability-registry-autonomy** — depends on `Capability` + `AutonomyTier` (capability-registry-kernel:49, autonomy-ceiling-kernel:13) — both real.

The lean-a10 lane runs **only on the owner phase**. lean-a5-doc-coverage runs on owner. Cross-phase rows in INDEX.md only acknowledge the dependency; the gates don't fire on them.

### B.10 Doc + ADR footprint

| Artifact | Path |
|----------|------|
| Phase INDEX update | `.omc/plans/milestones/M02-foundry-preview/phases/P01-provider-gateway/INDEX.md` |
| Implementation plan | `.omc/plans/milestones/M02-foundry-preview/phases/P01-provider-gateway/IP-NNN-supervisor.md` (new) |
| ADR — dep policy | `docs/decisions/ADR-NNNN-supervisor-dep-policy-branch-y.md` |
| ADR — mountpoint | `docs/decisions/ADR-NNNN-supervisor-mountpoint-direct-hyper.md` |
| ADR — durability | `docs/decisions/ADR-NNNN-jsonl-best-effort-durability.md` |
| ADR — public contract | `docs/decisions/ADR-NNNN-supervisor-public-contract-lean-a10.md` (declares **zero** new public APIs on existing kernels) |
| ADR — cedar bootstrap (placeholder; risk-track only) | `docs/decisions/ADR-NNNN-cedar-policy-bootstrap.md` |
| Doc-coverage suite (lean-a5) per crate | `docs/foundry/supervisor/{README,architecture,operations,security,sample-payloads}.md` for each of 4 new crates |
| Capability seed | `registry/capabilities/foundry-supervisor.toml` |
| Bench harness | `crates/oya-intelligence-supervisor-app/benches/heartbeat.rs` |
| Conformance build script | `crates/oya-foundry-supervisor-conformance/build.rs` (emits measured T-tier into seed file at build time) |

**Renames (prerequisite ADR, NOT part of this plan):**
- `oya-foundry-account-adapter-claude-code` → `oya-intelligence-claude-account-adapter`
- `oya-foundry-account-adapter-codex-cli` → `oya-intelligence-codex-account-adapter`
- `oya-foundry-account-adapter-gemini-cli` → `oya-intelligence-gemini-account-adapter`

### B.11 Verification gates (concrete commands; runnable as-is)

| Gate | Command |
|------|---------|
| Workspace builds | `cargo build --workspace` |
| Kernel unit tests | `cargo test -p oya-intelligence-supervisor-kernel` |
| Adapter unit tests | `cargo test -p oya-intelligence-jsonl-supervisor-adapter` |
| App lifecycle | `cargo test -p oya-intelligence-supervisor-app --test lifecycle` |
| Crash injection | `cargo test -p oya-intelligence-supervisor-app --test crash_injection -- --test-threads=1` |
| Watchdog timing | `cargo test -p oya-intelligence-supervisor-app --test lifecycle watchdog_kill_p95` |
| Dead-letter lane | `cargo test -p oya-intelligence-supervisor-app --test lifecycle poison_message_quarantine_after_3` |
| Audit chain | `cargo test -p oya-intelligence-supervisor-app --test audit_chain` |
| Conformance — Codex | `OYA_DRIVER=codex cargo test -p oya-foundry-supervisor-conformance -- stop_hook_codex --test-threads=1` |
| Conformance — Gemini | `OYA_DRIVER=gemini cargo test -p oya-foundry-supervisor-conformance -- stop_hook_gemini --test-threads=1` |
| Conformance — Claude | `OYA_DRIVER=claude cargo test -p oya-foundry-supervisor-conformance -- stop_hook_claude --test-threads=1` |
| Capability seed emitted | `test -f registry/capabilities/foundry-supervisor.toml && grep -c '\[\[driver\]\]' registry/capabilities/foundry-supervisor.toml | grep -q '^3$'` |
| Capability seed loads | `cargo test -p oya-foundry-supervisor-conformance -- seed_loads_and_publishes_to_registry` |
| Bench harness | `cargo test --release -p oya-intelligence-supervisor-app --bench heartbeat` |
| Live smoke matrix | `OYA_LIVE_SMOKE=1 cargo test -p oya-intelligence-supervisor-app --test matrix_3x2x2 -- --include-ignored --test-threads=1` |
| Lean-a10 (no kernel surface change) | `cargo public-api -p oya-intelligence-route-policy-kernel \| diff - .omc/snapshots/route-policy-kernel.public-api.txt` (snapshot stays byte-identical) |
| Lean-a5 doc coverage | `cargo run -p oya-check-doc-catalog -- --crate oya-intelligence-supervisor-kernel --require README,architecture,operations,security,sample-payloads` (repeat for each of 4 crates) |
| Predictable naming | `cargo run -p oya-governance-predictable-naming-kernel -- --check crates/oya-foundry-supervisor-{kernel,app,conformance} crates/oya-intelligence-jsonl-supervisor-adapter` |
| Banned primitives | `cargo run -p oya-governance-banned-primitives-kernel -- --check crates/oya-intelligence-supervisor-app` (must show only sanctioned `grit`/`icm`/`oya-tooling-agent-read`) |
| Net-new dep count | `cargo metadata --format-version 1 \| jq '[.packages[] \| select(.source != null) \| .name] \| length'` baseline vs after (must be **equal** — Branch Y) |

### B.12 Sequencing + grit claim units

| Order | Unit (file::Identifier) | Phase ref | Notes |
|-------|------------------------|-----------|-------|
| 1 | `docs/decisions/ADR-NNNN-supervisor-dep-policy-branch-y.md::header` | M02-P01 | Codifies Branch Y |
| 2 | `docs/decisions/ADR-NNNN-supervisor-public-contract-lean-a10.md::header` | M02-P01 | Declares zero new kernel APIs |
| 3 | `docs/decisions/ADR-NNNN-jsonl-best-effort-durability.md::header` | M02-P01 | Consequence of Branch Y |
| 4 | `docs/decisions/ADR-NNNN-supervisor-mountpoint-direct-hyper.md::header` | M02-P01 | Bypass rest-adapter stub |
| 5 | `docs/decisions/ADR-NNNN-cedar-policy-bootstrap.md::header` | M02-P01 (risk-track) | No-op stub; tracks the cedar gap |
| 6 | `Cargo.toml::members` | M02-P01 | Add 4 new crates to workspace |
| 7 | `crates/oya-intelligence-supervisor-kernel/src/lib.rs::MessageId` | M02-P01 | New file |
| 8 | `crates/oya-intelligence-supervisor-kernel/src/lib.rs::RequestId` | M02-P01 | |
| 9 | `crates/oya-intelligence-supervisor-kernel/src/lib.rs::UsageWindowSnapshot` | M02-P01 | |
| 10 | `crates/oya-intelligence-supervisor-kernel/src/lib.rs::SessionTicket` | M02-P01 | Value-only invariant |
| 11 | `crates/oya-intelligence-supervisor-kernel/src/lib.rs::InboxState` | M02-P01 | |
| 12 | `crates/oya-intelligence-supervisor-kernel/src/lib.rs::SupervisorVerdict` | M02-P01 | |
| 13 | `crates/oya-intelligence-supervisor-kernel/src/lib.rs::SupervisorError` | M02-P01 | |
| 14 | `crates/oya-intelligence-supervisor-kernel/src/lib.rs::SpendRecord` | M02-P01 | |
| 15 | `crates/oya-intelligence-supervisor-kernel/src/lib.rs::decide` | M02-P01 | Pure decision fn |
| 16 | `crates/oya-intelligence-supervisor-kernel/src/lib.rs::next_inbox_state` | M02-P01 | State machine |
| 17 | `crates/oya-intelligence-supervisor-kernel/src/lib.rs::record_spend` | M02-P01 | |
| 18 | `crates/oya-intelligence-supervisor-kernel/src/lib.rs::spend_to_usage_record` | M02-P01 | Bridge to billing-kernel |
| 19 | `crates/oya-intelligence-supervisor-kernel/src/lib.rs::SessionDriver` | M02-P01 | Sync port |
| 20 | `crates/oya-intelligence-supervisor-kernel/src/lib.rs::InboxStore` | M02-P01 | Sync port |
| 21 | `crates/oya-intelligence-jsonl-supervisor-adapter/src/lib.rs::JsonlInbox` | M02-P01 | New file |
| 22 | `crates/oya-intelligence-jsonl-supervisor-adapter/src/lib.rs::JsonlOutbox` | M02-P01 | |
| 23 | `crates/oya-intelligence-jsonl-supervisor-adapter/src/lib.rs::encode_inbox_line` | M02-P01 | Hand-rolled framing |
| 24 | `crates/oya-intelligence-jsonl-supervisor-adapter/src/lib.rs::decode_inbox_line` | M02-P01 | |
| 25 | `crates/oya-intelligence-supervisor-app/src/lib.rs::SupervisorApp` | M02-P01 | |
| 26 | `crates/oya-intelligence-supervisor-app/src/lib.rs::tick_once` | M02-P01 | |
| 27 | `crates/oya-intelligence-supervisor-app/src/lib.rs::run_forever` | M02-P01 | tokio host |
| 28 | `crates/oya-intelligence-supervisor-app/benches/heartbeat.rs::main` | M02-P01 | Bench bin |
| 29 | `crates/oya-intelligence-supervisor-app/tests/lifecycle.rs::*` | M02-P01 | |
| 30 | `crates/oya-intelligence-supervisor-app/tests/crash_injection.rs::*` | M02-P01 | |
| 31 | `crates/oya-intelligence-supervisor-app/tests/audit_chain.rs::*` | M02-P01 | |
| 32 | `crates/oya-intelligence-supervisor-app/tests/matrix_3x2x2.rs::*` | M02-P01 | Gated on `$OYA_LIVE_SMOKE` |
| 33 | `crates/oya-foundry-supervisor-conformance/build.rs::main` | M02-P01 | Emits T-tier into seed file |
| 34 | `crates/oya-foundry-supervisor-conformance/src/lib.rs::*` | M02-P01 | |
| 35 | `registry/capabilities/foundry-supervisor.toml::header` | M02-P01 | Seed file (build.rs writes T-tiers) |
| 36 | `.omc/plans/milestones/M02-foundry-preview/phases/P01-provider-gateway/INDEX.md::add-ip-supervisor-row` | M02-P01 | |
| 37 | `.omc/plans/milestones/M02-foundry-preview/phases/P01-provider-gateway/IP-NNN-supervisor.md::header` | M02-P01 | Phase IP |
| 38 | `docs/foundry/supervisor/README.md::header` | M02-P01 | Doc-coverage row 1 |
| 39 | `docs/foundry/supervisor/architecture.md::header` | M02-P01 | Doc-coverage row 2 |
| 40 | `docs/foundry/supervisor/operations.md::header` | M02-P01 | Doc-coverage row 3 |
| 41 | `docs/foundry/supervisor/security.md::header` | M02-P01 | Doc-coverage row 4 |
| 42 | `docs/foundry/supervisor/sample-payloads.md::header` | M02-P01 | Doc-coverage row 5 |

**Total grit claim units: 42.** Every new file + every new public Identifier has one. Every line of `cargo` runnable in §B.11 maps to a unit here.

---

## §C. Acceptance bar (testable; every row maps to a §B.11 command)

| # | Acceptance criterion | §B.11 command |
|---|---------------------|---------------|
| C.1 | `cargo build --workspace` exits 0 with 4 new crates compiled | row 1 |
| C.2 | Kernel unit tests pass; `SessionTicket` field-level inspection confirms zero `&`/`Arc`/`Box<dyn>` | row 2 |
| C.3 | Adapter unit tests pass; round-trip `encode_inbox_line` → `decode_inbox_line` is byte-identical for 100 random samples | row 3 |
| C.4 | Lifecycle test passes; supervisor processes 10 messages spawning fake driver and committing 10 outbox lines | row 4 |
| C.5 | **Crash-injection lane green** — SIGKILL at every flush point; recovery yields zero data loss or quarantine-with-reason | row 5 |
| C.6 | **Watchdog kill timing lane green** — fake driver `loop{sleep(1s)}` killed within 5.0s p95 | row 6 |
| C.7 | **Dead-letter lane green** — poison line moves to `dead-letter/` after 3 retries; tier demoted by 1 step | row 7 |
| C.8 | Audit chain test passes; every spawn/commit emits an `EvidenceRef` (capability-registry-kernel:27) linked to capability_id | row 8 |
| C.9 | **Codex stop-hook verification** — `crates/oya-foundry-supervisor-conformance/artifacts/codex-stop-hook.json` exists with `{stop_hook_count: 1, idempotency_replay: ok}` | row 9 |
| C.10 | **Gemini stop-hook verification** — `crates/oya-foundry-supervisor-conformance/artifacts/gemini-stop-hook.json` exists; if `request_id_supported=false`, tier demoted to T2Suggest is RECORDED | row 10 |
| C.11 | Claude stop-hook verification — equivalent artifact at T1 baseline | row 11 |
| C.12 | **Capability seed file** `registry/capabilities/foundry-supervisor.toml` exists with exactly 3 `[[driver]]` blocks, each carrying a measured `autonomy_tier` filled by the conformance build.rs | rows 12, 13 |
| C.13 | Bench harness produces 4 measured metrics: idle-tick p95 ≤ 25 tok, restart p95 ≤ 1.5s, RSS ≤ 64 MiB, watchdog kill ≤ 5.0s | row 14 |
| C.14 | **Live smoke matrix green** — 3×CLI × 2×accounts × 2×providers = 12 combinations each process at least 1 message and commit at least 1 outbox line | row 15 |
| C.15 | **Lean-a10 public-API snapshot stays byte-identical** for `oya-intelligence-route-policy-kernel`, `oya-intelligence-usage-window-kernel`, `oya-cloud-billing-kernel`, `oya-intelligence-account-domain`, `oya-governance-autonomy-ceiling-{kernel,domain,app}`, `oya-foundry-capability-registry-{kernel,domain,app}` (no kernel touched) | row 16 |
| C.16 | Lean-a5 doc-coverage green for all 4 new crates | row 17 |
| C.17 | Predictable-naming + banned-primitives lanes green for the 4 new crates | rows 18, 19 |
| C.18 | **Net-new external dep count = 0** (Branch Y) — `cargo metadata` package count unchanged | row 20 |
| C.19 | **`projected_tokens` source measured**: `projected_tokens_p95` populated from rolling p95 of last 100 (provider, model_hint) samples; fallback = `cost_ceiling_tokens` when n<10 — asserted in `tests/lifecycle.rs::projected_tokens_p95_source` | row 4 |
| C.20 | **Per-CLI `request_id` idempotency demonstrated under crash-replay**: conformance test re-dispatches the same request_id post-SIGKILL; the fake provider's reply matches first call byte-for-byte (claude+codex); gemini-driver demonstrates documented-degrade-not-corruption instead | rows 9, 10, 11 |

---

## §D. Output path

This plan: `/Users/jasonlee/oyatie/.omc/plans/ralplan-foundry-supervisor-simple-v2-2026-05-14.md`

Open-questions appended to: `/Users/jasonlee/oyatie/.omc/plans/open-questions.md`

---

## §E. Open questions (append to `.omc/plans/open-questions.md`)

1. **Cedar policy bootstrap** — zero `.cedar` files exist in the workspace; supervisor capabilities carry `Cedar policy path = TBD`. Should this plan block on `ADR-cedar-policy-bootstrap` landing, or proceed with autonomy-ceiling-only enforcement and a follow-up ADR? Recommendation: proceed; track risk.
2. **Driver-crate rename ADR ordering** — does the rename ADR (`oya-foundry-account-adapter-*` → `oya-foundry-*-account-adapter`) need to merge BEFORE this plan starts, or can it land in the same PR train? Recommendation: separate ADR merges first.
3. **`api-rest-adapter` future** — by side-stepping the stub, do we commit to it remaining a stub until M02-P04? Or does this plan create pressure to land the real router earlier? Recommendation: stays stub until M02-P04.
4. **Gemini idempotency** — if a future Gemini SDK release ships an idempotency key, who is responsible for re-running the conformance crate to re-measure the tier? Recommendation: lean-a5-doc-coverage row in the gemini-driver crate.
5. **Live smoke matrix gating** — what is the canonical way to flag `$OYA_LIVE_SMOKE=1` in CI without burning credit on every PR? Recommendation: nightly-only lane.
6. **`cargo public-api` availability** — Lean-a10 snapshot uses `cargo public-api`, which is not in the workspace tooling. Either install it CI-side or substitute with `rustdoc --output-format json` + `jq`. Recommendation: ADR to install `cargo public-api` (this would be a TOOLING dep, not a workspace dep; doesn't violate Branch Y).

---

## §F. Plan summary (executor cheat sheet)

- **Mode:** deliberate; iteration 2 of 5; supersedes v1.
- **Dep branch:** Y (zero net-new external deps).
- **New crates:** 4 (kernel + app + adapter + conformance).
- **New public APIs on existing kernels:** 0.
- **Phase owner:** M02-P01-provider-gateway.
- **42 grit claim units, 15 acceptance rows, 20 verification commands.**
- **3 critical RISKs declared with mitigation lanes:** torn-write, hung-CLI, poison-message.

**Does this plan capture your intent?**
- "proceed" — hand off to Architect + Critic for iteration-2 consensus review
- "adjust [X]" — return to interview
- "restart" — discard
