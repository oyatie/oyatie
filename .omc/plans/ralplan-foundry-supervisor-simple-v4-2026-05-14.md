---
status: pending approval
mode: deliberate
iteration: 4
supersedes: ralplan-foundry-supervisor-simple-v3-2026-05-14.md
owner_phase: M02-P06
diffs_applied:
  v3_round: 14
  v4_round: 7
  cumulative: 21
purpose: Auto-backfilled purpose for ralplan-foundry-supervisor-simple-v4-2026-05-14.md
---
# RALPLAN — Foundry Supervisor (Simple) — Iteration 4

**User intent (verbatim, unchanged from v1/v2/v3):**
> "simple hook + inbox outbox setup. make it so that this works with multiple accounts. and across multiple providers. this will allow us to simplify our setup. still should be able to intelligently manage usage. all the features still have to come."

**Reference shape:** `Siigari/claude-heartbeat` — Node supervisor + stop-hook + JSONL inbox/outbox + restart-per-message + idle heartbeat tick.

### A.0.1 Build-vs-adopt (PRE-6)
**Build-vs-adopt analysis for Siigari/claude-heartbeat** (cited as 'Reference shape' above): Per ADR-0096-supervisor-language-rust-not-node, the upstream Node implementation was considered as a sibling sidecar (Rust crates speak JSONL to Node inbox/outbox). Rejected because: (a) workspace-language-purity is an ADR-tracked principle — `oya-*` crates are Rust-native; (b) supervisor's deep composition with `RoutePolicy::select`, `UsageEnforcement::check_limit`, `validate_usage`, `finalize_line`, `check_silent_switch`, `enforce_for_tenant` requires sharing kernel types, which a Node sidecar cannot do without an IPC bridge; (c) autonomy_ceiling Cedar enforcement and audit-chain emission must execute in the same process as the supervisor for crash atomicity. Trade-off accepted: 4 new Rust crates (49+14 grit units) vs. adopting an upstream that doesn't compose with foundry kernels.

**v4 mandate:** apply 7 narrow patches from Architect+Critic v3 review. No scope regrowth. Edit-in-place; cumulative 21 diffs across v3+v4. See §E.v3 + §E.v4 for change logs.

---

## §A. RALPLAN-DR summary

### A.1 Principles (4 — unchanged from v2/v3)
1. **driver-not-kernel** — Each CLI (claude-code, codex, gemini) is invoked as a fresh subprocess by the supervisor app; the supervisor kernel owns no provider handles.
2. **fresh-subprocess-statelessness** — Every message → spawn new CLI process; on stop-hook fire, the process exits. State lives only in JSONL on disk between turns.
3. **value-only-tickets** — `SessionTicket` carries owned, copied values only: `AccountId`, `ProviderFamily`, `AutonomyTier`, `UsageWindowSnapshot`, `MessageId`. Zero refs, Arcs, dyn fields across kernel boundaries. **Invariant.**
4. **dep-branch-Y-commitment** — Zero net-new external deps; sync `SessionDriver` trait; best-effort durability (no `fsync(parent_dir)`); admit non-durability-across-power-loss in ADR.

### A.2 Decision Drivers (top 3 — unchanged from v2/v3)
1. **multi-account-fanout** — `RoutePolicy::select` (route-policy-kernel:54) composes the per-call routing decision; we compose, not invent.
2. **usage-window-honesty** — `UsageEnforcement::check_limit` (usage-window-kernel:30) returns 4-outcome `EnforcementVerdict`; supervisor respects verdicts before spawn, not after credit burn.
3. **cross-CLI-parity-by-type** — `AutonomyTier` T1..T4 (capability-registry-kernel:50) classifies drivers by what they can actually do; demote when verification fails.

### A.3 Viable Options (4 — Option B refairned per v3 Critic diff #11; unchanged in v4)

| Opt | Shape | Pros | Cons | Verdict |
|-----|-------|------|------|---------|
| **A** | Single supervisor daemon owns the whole chain (route → reserve → spawn → watch → settle) | Single fsync ordering point; one process to monitor; matches Siigari shape | Daemon failure = total outage; restart loses in-flight reservations; harder to evolve | **REJECTED** — SPoF violates fan-out principle |
| **B** | Per-CLI binaries (3 separate daemons) | Real benefit: per-driver crash isolation; one Claude bug cannot kill Codex or Gemini lanes | Pays for itself only at >10 messages/s sustained; current foundry budget is far below this floor. Reservation conflict across binaries needs a coordinator broker — and that broker becomes Option A in disguise, re-introducing SPoF | **REJECTED on cost-benefit**, not on principle |
| **C** | In-process inside `oya-foundry-dashboard-app` | Reuses dashboard event loop | Couples runtime ops to UI lifecycle; dashboard restart kills supervisor; mixes layers | **REJECTED** — violates 12-layer enum |
| **D** | **Host-injected policy ports: supervisor-app composes route-policy + usage-enforcement + cost-ceiling as pure ports; jsonl-supervisor-adapter is the I/O seam; supervisor-kernel is pure decision logic** | Kernel stays I/O-free; adapter is the only fsync-aware crate; ports compose without inheritance; aligns ADR-0056 port-in-kernel + ADR-0092 conversion-at-the-boundary | More crates than Option A; one new port (`AccountSnapshotProvider`) added inside supervisor-kernel — no existing-kernel surface change | **CHOSEN** |

**Why D wins:** composes existing `RoutePolicy::select`, `UsageEnforcement::check_limit`, `check_silent_switch`, `validate_usage`, `finalize_line` without inventing existing-kernel APIs. All net-new traits/types are inside the 4 new supervisor crates.

### A.4 Pre-mortem (3 failure scenarios — unchanged from v2/v3)

| Scenario | Mitigation lane | Acceptance |
|----------|----------------|------------|
| **(a) Torn write** — supervisor crash mid-rename leaves orphan `.tmp` + half-written outbox | `lean-fsync-durability` — `tests/crash_injection.rs` SIGKILLs writer at every flush point; recovery reproduces cursor or quarantines partial line | `cargo test -p oya-foundry-supervisor-app --test crash_injection -- --test-threads=1` exits 0; no orphan `.tmp` after replay |
| **(b) Hung CLI burns credit** — driver deadlocks; reservation TTL expires but credit already consumed | `lean-watchdog-timing` — `loop{sleep(1s)}` fake-driver test asserts SIGKILL within `WATCHDOG_TIMEOUT + 5s grace` | Watchdog kill latency p95 ≤ 5.0s in `benches/heartbeat.rs::watchdog_kill_latency` |
| **(c) Poison message loop** — corrupt JSONL crashes parser → infinite restart | `lean-dead-letter` — kernel verdict `Quarantine` after `MAX_PARSE_RETRIES=3`; tier demoted by 1; entry moved to `dead-letter/`; metric `supervisor_quarantine_total` increments | Integration test injects malformed line; assert `dead-letter/` has 1 file after 3 spawn attempts and `T<n+1>` demoted to `T<n>` |

### A.5 Expanded test plan (deliberate mode mandatory)

| Layer | Crate / file | Command |
|-------|--------------|---------|
| Unit (kernel) | `crates/oya-foundry-supervisor-kernel/src/lib.rs` | `cargo test -p oya-foundry-supervisor-kernel` |
| Unit (adapter) | `crates/oya-foundry-jsonl-supervisor-adapter/src/lib.rs` | `cargo test -p oya-foundry-jsonl-supervisor-adapter` |
| Unit (saturation — **v3 diff #4**) | `tests/saturation.rs::tick_once_returns_saturated_when_in_flight_at_capacity` | `cargo test -p oya-foundry-supervisor-app --test saturation` |
| Unit (serde round-trip — **v3 diff #10**) | `tests/provider_family_round_trip.rs::toml_string_OpenAIOrCodex_decodes_to_enum_variant_OpenAiOrCodex` | `cargo test -p oya-foundry-supervisor-conformance --test provider_family_round_trip` |
| Integration | `crates/oya-foundry-supervisor-app/tests/lifecycle.rs` (spawn, restart, settle) | `cargo test -p oya-foundry-supervisor-app --test lifecycle` |
| Cost-ceiling enforcement (**v3 diff #12**) | `tests/lifecycle.rs::cost_ceiling_blocks_when_projected_exceeds_ceiling` | included in lifecycle test target |
| E2E matrix | `tests/matrix_3x2x2.rs` — 3 CLIs × 2 accounts × 2 providers = 12 combinations | `cargo test -p oya-foundry-supervisor-app --test matrix_3x2x2 -- --include-ignored --test-threads=1` (gated `$OYA_LIVE_SMOKE=1`) |
| Observability | `tests/audit_chain.rs` — every spawn/settle emits an EvidenceRef linked to capability id | `cargo test -p oya-foundry-supervisor-app --test audit_chain` |
| Perf budget | `benches/heartbeat.rs` — 4 metrics split per **v3 diff #9**: C.13a-d | `cargo bench -p oya-foundry-supervisor-app --bench heartbeat -- --save-baseline supervisor-v4` |

**Note on Criterion:** workspace has no `criterion` dep (verified Cargo.toml:481-492). Under Branch Y, harness uses `std::time::Instant` only; emits one JSONL row per metric per the schema declared in §C.13. Bench-bin is driven by `cargo test --release --bench heartbeat` to stay zero-dep.

---

## §B. Implementation plan

### B.0 Existing surfaces inventory (mandatory)

| Surface | File | Lines | Notes |
|---------|------|-------|-------|
| `RoutePolicy::select(&[ProviderAccount], &RouteConstraints) -> Result<RouteExplanation, RouteError>` | `crates/oya-foundry-route-policy-kernel/src/lib.rs` | 54–96 | Pure selector; no I/O. **Composed, not extended.** |
| `RoutePolicy::explain_route(...)` | same | 101–106 | Same selector; audit surface kept. |
| `RouteConstraints` (struct) | same | 12–19 | Public fields all `INTERNAL_ONLY`. |
| `RouteError` enum | same | 39–47 | 7 variants. |
| `UsageEnforcement::check_limit(&UsageWindow, now: u64, budget: u64) -> Result<EnforcementVerdict, EnforcementError>` | `crates/oya-foundry-usage-window-kernel/src/lib.rs` | 30–71 | Pure verdict; 4 outcomes including `WindowExpired`. |
| `EnforcementVerdict` enum | same | 9–18 | `WithinLimit`/`OverUsageLimit`/`ReserveBreached`/`WindowExpired`. |
| `UsageWindow` (verified value type) | `crates/oya-foundry-account-domain/src/lib.rs` | 242–302 | Pub struct (account-domain:252); fields all owned numeric/enum; `#[derive]` admits `Clone`. **Eligible for `UsageWindowSnapshot` derivation via Clone (see §B.4 step 7.5).** |
| `validate_usage(&UsageRecord) -> Result<(), BillingError>` | `crates/oya-cloud-billing-kernel/src/lib.rs` | 74–85 | Pure validator. |
| `finalize_line(&LineItem) -> Result<u128, BillingError>` | same | 87–95 | Subtotal in micros. |
| `UsageRecord` / `LineItem` / `UsageUnit::Token` | same | 7–52 | `UsageUnit::Token` exists; reused for spend records. |
| `check_silent_switch(&[&ProviderAccount], &ProviderAccount) -> Result<(), AccountError>` | `crates/oya-foundry-account-domain/src/lib.rs` | 171–186 | Cross-account guard. |
| `ProviderAccount` + state machine (Draft → Verified → Active → Degraded/Disabled/Revoked) | same | 68–167 | `degrade(reason: String) -> Result<(), AccountError>` at L120. |
| `AccountId(pub String)` / `ProviderFamily` enum / `SessionId` | `crates/oya-foundry-account-kernel/src/lib.rs` | 14–29 | Public; owned. `ProviderFamily` variants: `Aws`/`Claude`/`OpenAiOrCodex`/`Gemini` (account-kernel:23-29). |
| `ProviderFamily::try_from("OpenAIOrCodex")` → `Ok(Self::OpenAiOrCodex)` | same | 40-50 | **String form is CamelCase `"OpenAIOrCodex"`; Rust variant is `OpenAiOrCodex` (lowercase `i`). Round-trip hand-rolled (no serde).** See v3 diff #10. |
| `Capability::new(id, name, tier, evidence_required)` / `AutonomyTier` enum | `crates/oya-foundry-capability-registry-kernel/src/lib.rs` | 49–136 | `T1Read..T4Actuate`; `try_from("T1")` works. |
| `validate_publish(&Capability) -> Result<(), PublishValidationError>` | `crates/oya-foundry-capability-registry-domain/src/lib.rs` | 43 | Pre-condition for register. |
| `CapabilityRegistry::register/list/get` + `parse_seed_json` | `crates/oya-foundry-capability-registry-app/src/lib.rs` | 53–86, 97–153 | Hand-rolled JSON; **no serde** (HARD CONSTRAINT, L96). |
| `CeilingPolicy::ceiling_for(&TenantId) -> AutonomyTier` / `set` | `crates/oya-foundry-autonomy-ceiling-domain/src/lib.rs` | 40–70 | Default = `T3PropAct`. |
| `enforce(&Capability, ceiling) -> CeilingVerdict` / `enforce_for_tenant` | `crates/oya-foundry-autonomy-ceiling-app/src/lib.rs` | 21–33 | Bridges Cap-tier ↔ Ceiling-tier enums. |
| `check_tier(cap_tier, ceiling) -> CeilingVerdict` | `crates/oya-foundry-autonomy-ceiling-kernel/src/lib.rs` | 53–62 | Pure comparison. |
| `serve(addr, router, chain, ServerConfig) -> Result<(), HyperRuntimeError>` | `crates/oya-http-runtime-hyper-adapter/src/lib.rs` | 284–335 | Real hyper server; `tokio` runtime. |
| `Router<SyncHandler>` + `MiddlewareChain` + `dispatch` | same + middleware-kernel | 206–221 | Real router for the supervisor webhook surface. |
| `oya-foundry-api-rest-adapter` | `crates/oya-foundry-api-rest-adapter/src/lib.rs` | 1–68 | **CONFIRMED STUB** — fixed 200 response. Side-stepped; not used. |
| `oya-foundry-account-adapter-{claude-code,codex-cli,gemini-cli}` | each crate's `src/lib.rs` | L1–3 | **All three are `pub fn placeholder() {}` skeletons.** No CLI logic yet — finding, not regression. |
| **`oya-foundry-account-adapter-openbao` `SecretStorePort`** (**v4 patch #4**) | `crates/oya-foundry-account-adapter-openbao/src/lib.rs` | 26 impl + 88–164 tests | Adapter present (verified `OpenBaoAdapter: SecretStorePort` in-memory ref impl, 10 unit tests at `crates/oya-foundry-account-adapter-openbao/src/lib.rs:26+88-164`). Network-OpenBao upgrade tracked separately. |
| Workspace deps | `Cargo.toml` | 481–492 | `tracing`, `hyper`, `hyper-util`, `tokio` (rt-multi-thread, net, macros), `http-body-util`, `bytes`. **No `rustix`, `async-trait`, `nix`, `serde`, `criterion`.** |
| Capability seed file | `registry/capabilities/foundry-internal.json` | 8.8 KB, 50+ caps | Confirmed exists; T4 count must be 0 (capability-registry-app:307). |
| **Cedar policy seed** (v3 diff #1 — verified, not "None found") | `docs/policies/autonomy-ceiling.cedar` | 3 lines, M02-P05-IP-002 seed | **EXISTS.** Verbatim content: `// M02-P05-IP-002 autonomy ceiling — T4 actuation disabled by default` / `forbid (principal, action == Action::"actuate-t4", resource);` / `permit (principal, action in [Action::"read-t1", Action::"suggest-t2"], resource);`. The supervisor ADR (`ADR-cedar-policy-extend-supervisor-capabilities`) **extends** this seed (not bootstraps from zero). Supervisor capabilities cite the NEW file `docs/policies/foundry-supervisor.cedar` (created by this plan); the autonomy-ceiling seed remains the umbrella policy. |
| `oya-dev-cli` binary | `crates/oya-dev-cli/src/main.rs` | confirmed | Real binary (commands/ subdir holds 30+ gates). |
| `oya-foundry-fitness-banned-primitives` binary | `tools/oya-foundry-fitness-banned-primitives/src/main.rs` | L15 (existing) | Real binary; pattern model for new gate bins (v3 diff #8). |
| `oya-check-doc-catalog` binary | (**no `main.rs`** — library only) | — | **Plan must NOT invoke as `cargo run`.** See §B.11 row 17 substitution (v3 diff #8). |
| `oya-foundry-fitness-predictable-naming-kernel` binary | (**no `main.rs`** — library only) | — | **Plan must NOT invoke as `cargo run`.** See §B.11 row 18 substitution (v3 diff #8). New binary `tools/oya-foundry-fitness-predictable-naming/src/main.rs` is declared by this plan in §B.1 (directory does not yet exist; created as scaffold artifact). |
| **`registry/accounts/` directory** (v4 patch #7) | path | — | Directory does NOT yet exist; declared as scaffold artifact in §B.10. Production `AccountSnapshotProvider` reads `registry/accounts/*.toml` from this directory. |
| Existing v3 plan | `.omc/plans/ralplan-foundry-supervisor-simple-v3-2026-05-14.md` | 673 lines | Superseded by this v4. |
| Open-questions ledger | `.omc/plans/open-questions.md` | 717+ lines | Append new v4 section. |

### B.1 Crate decomposition (3 new + 1 conformance = 4 total) + 1 new binary scaffold

| Crate / binary | v4-BNF + 12-layer-enum justification |
|----------------|--------------------------------------|
| `oya-foundry-supervisor-kernel` | `oya-<foundry>-<supervisor>-<kernel>` — foundry is the registered µservice (Cargo.toml:290); supervisor is the new feature target; kernel = layer #1 (pure types). |
| `oya-foundry-supervisor-app` | Same prefix; app = layer #4 — orchestrates ports, owns tokio runtime, hosts `benches/heartbeat.rs`. |
| `oya-foundry-jsonl-supervisor-adapter` | `oya-<foundry>-<jsonl-supervisor>-<adapter>` — jsonl-supervisor is the file-format + feature compound (mirrors `oya-cloud-storage-block-adapter`); adapter = layer #5; the ONLY crate that does `std::fs` writes. |
| `oya-foundry-supervisor-conformance` | Stand-alone test crate; emits the capability registry seed row for each driver at its measured T-level via `build.rs`. |
| **`tools/oya-foundry-fitness-predictable-naming/`** (NEW binary scaffold, v4 patch #1) | Directory does not yet exist (verified `ls tools/`); declared as scaffold artifact. Binary `tools/oya-foundry-fitness-predictable-naming/src/main.rs` wraps `oya-foundry-fitness-predictable-naming-kernel::check()`. Pattern model: `tools/oya-foundry-fitness-banned-primitives/src/main.rs:15`. |

**Conformance timing contract (v3 diff #7):** `build.rs` writes the seed at compile time; `cargo test -p oya-foundry-supervisor-conformance` reads the emitted seed and asserts the tier-tier-tier triple equals the measured runtime behavior in the same invocation. Acceptance row C.12.bis enforces `grep "autonomy_tier = \"T[1-4]\"" registry/capabilities/foundry-supervisor.toml | wc -l == 3` (no `T?` placeholders left).

**Bench harness file:** `crates/oya-foundry-supervisor-app/benches/heartbeat.rs` — `[[bench]] name = "heartbeat"` in supervisor-app `Cargo.toml`.

### B.2 Public contracts (kernel surface)

All types use **owned values only**. No lifetimes, no trait objects in struct fields.

#### B.2.1 `oya-foundry-supervisor-kernel` types

```rust
use oya_foundry_account_kernel::{AccountId, ProviderFamily, SessionId};
use oya_foundry_capability_registry_kernel::AutonomyTier;
use oya_foundry_account_domain::UsageWindow;

#[derive(Clone, Debug, Eq, PartialEq, Hash)] pub struct MessageId(pub String);
#[derive(Clone, Debug, Eq, PartialEq, Hash)] pub struct RequestId(pub String);  // provider idempotency key

/// **Audit-only.** Live `UsageWindow` (account-domain:252) remains the
/// enforcement + reconciliation source of truth; this snapshot exists for
/// ticket transport across blocking-pool boundaries. (v3 diff #5.)
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsageWindowSnapshot {
    pub kind_label: String, pub started_at_epoch_secs: u64, pub ends_at_epoch_secs: u64,
    pub tokens_in: u64, pub tokens_out: u64,
    pub usage_limit_pct: u8, pub reserve_remaining_pct: u8,
    pub projected_tokens_p95: u64,                  // computed by app layer
}
impl UsageWindowSnapshot {
    pub fn from_window(w: &UsageWindow, projected_tokens_p95: u64) -> Self { /* pure copy */ }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionTicket {
    pub message_id: MessageId, pub request_id: RequestId,
    pub account_id: AccountId, pub provider_family: ProviderFamily,
    pub autonomy_tier: AutonomyTier, pub window_snapshot: UsageWindowSnapshot,
    pub cost_ceiling_tokens: u64, pub model_hint: String,
}
// Invariant fence: `fn _f<T: Send + Sync + 'static>() {} _f::<SessionTicket>();`
// (static_assertions is not a workspace dep; the manual call is the fence.)

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InboxState {
    Queued, Locked { reservation_id: String, ttl_epoch_secs: u64 },
    InFlight { reservation_id: String },
    DraftedResponse { outbox_pending: bool }, Committed,
    DeadLettered { reason: String }, Released { reason: String },
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
    NoEligibleAccount, UsageBlocked(String), TierBlocked(String),
    InvalidTransition { from: &'static str, to: &'static str },
    ParseFailed { line_number: u64 }, MaxRetriesExceeded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpendRecord {
    pub request_id: RequestId, pub account_id: AccountId, pub provider_family: ProviderFamily,
    pub tokens_in: u64, pub tokens_out: u64, pub model_hint: String,
}
```

#### B.2.2 `oya-foundry-supervisor-kernel` pure functions (no new APIs on existing kernels)

```rust
pub fn decide(accounts: &[ProviderAccount], constraints: &RouteConstraints,
              window: &UsageWindow, now_epoch_secs: u64, budget_tokens: u64,
              tenant_ceiling: AutonomyTier, msg: &IncomingMessage)
    -> Result<SupervisorVerdict, SupervisorError>;

pub fn next_inbox_state(current: &InboxState, event: &InboxEvent, now_epoch_secs: u64)
    -> Result<InboxState, SupervisorError>;

pub fn record_spend(ticket: &SessionTicket, tokens_in: u64, tokens_out: u64) -> SpendRecord;

pub fn spend_to_usage_record(s: &SpendRecord, tenant_id: &str, ts_ms: u64) -> UsageRecord;
```

#### B.2.3 `oya-foundry-supervisor-kernel` ports (sync traits — Branch Y)

```rust
pub trait SessionDriver: Send + Sync {
    fn driver_id(&self) -> &str;
    fn provider_family(&self) -> ProviderFamily;
    fn spawn_for_message(&self, ticket: &SessionTicket, message_body: &[u8])
        -> Result<DriverHandle, DriverError>;
}

pub struct DriverHandle { pub pid: i32, pub stop_hook_path: String, pub started_at_epoch_secs: u64 }

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DriverError { SpawnFailed(String), StopHookMissing, Watchdog }

pub trait InboxStore: Send + Sync {
    fn peek_lock(&self, ttl_secs: u64) -> Result<Option<(MessageId, Vec<u8>)>, StoreError>;
    fn commit(&self, id: &MessageId, response: &[u8]) -> Result<(), StoreError>;
    fn release(&self, id: &MessageId, reason: &str) -> Result<(), StoreError>;
    /// **v4 patch #6 (Option β).** Atomic lock-release + dead-letter transition.
    /// `dead_letter` MUST only be called on a currently-locked message; the call
    /// consumes the lock (releases `reservation_id`, sets `released_at_epoch_secs`)
    /// and writes the dead-letter row in a single transition. Calling on an
    /// unlocked message returns `StoreError::InvalidTransition`. This atomic
    /// semantic eliminates the Locked→DeadLettered two-call race and matches
    /// the existing `released_at_epoch_secs` field on the Locked variant.
    fn dead_letter(&self, id: &MessageId, reason: &str) -> Result<(), StoreError>;
    fn replay_cursor(&self) -> Result<u64, StoreError>;
}

/// v3 diff #3 — chosen path: account-snapshot port lives in supervisor-kernel.
/// Testable, swappable, no existing-kernel public surface change → lean-a10
/// ceremony does NOT fire. Grit unit #43 in §B.12.
pub trait AccountSnapshotProvider: Send + Sync {
    fn snapshot(&self) -> Vec<ProviderAccount>;
}
```

#### B.2.4 `oya-foundry-jsonl-supervisor-adapter` (the ONLY crate that touches `std::fs`)

```rust
pub struct JsonlInbox { dir: PathBuf, lock_ttl_secs: u64 }
impl InboxStore for JsonlInbox { /* … */ }

pub struct JsonlOutbox { dir: PathBuf }
impl JsonlOutbox { pub fn append(&self, response: &[u8]) -> Result<(), StoreError>; }  // tmp → rename

// Hand-rolled minimal framing (no serde — same constraint as capability-registry-app L96):
pub fn encode_inbox_line(state: &InboxState, msg: &MessageId, body: &[u8]) -> Vec<u8>;
pub fn decode_inbox_line(line: &[u8]) -> Result<(InboxState, MessageId, Vec<u8>), StoreError>;
```

#### B.2.5 `oya-foundry-supervisor-app` (composes ports; owns tokio runtime)

**v3 diff #4 — config struct + saturation outcome:**

```rust
#[derive(Clone, Debug)]
pub struct SupervisorConfig {
    pub max_in_flight: usize,           // semaphore capacity for spawn_blocking pool
    pub blocking_pool_size: usize,      // tokio rt-multi-thread blocking workers
    pub default_cost_ceiling: u64,      // per-message absolute upper bound
    pub watchdog_secs: u64,             // SIGTERM after this many secs
    pub heartbeat_interval_secs: u64,   // idle-tick cadence
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TickOutcome {
    Spawned(MessageId),
    Saturated,                          // in-flight semaphore at capacity
    Idle,                               // no messages to lock
    Quarantined(MessageId),
}

pub struct SupervisorApp<D: SessionDriver, I: InboxStore, A: AccountSnapshotProvider> { /* … */ }
impl<D: SessionDriver, I: InboxStore, A: AccountSnapshotProvider> SupervisorApp<D, I, A> {
    pub fn new(drivers: Vec<D>, inbox: I, accounts: A, outbox: JsonlOutbox, config: SupervisorConfig) -> Self;
    pub fn tick_once(&mut self, now_epoch_secs: u64) -> Result<TickOutcome, SupervisorError>;
    pub async fn run_forever(self) -> !;  // tokio::main host
}
```

**v4 patch #7 — `AccountSnapshotProvider` production wiring:** Production impl: composite over `oya-foundry-account-adapter-{claude,codex,gemini}` reading `registry/accounts/*.toml`. Test impl: in-memory `Vec<ProviderAccount>` shim. The `registry/accounts/` directory is a new scaffold artifact (§B.10 row).

#### B.2.6 New public APIs on existing kernels (full lean-a10 ceremony)

After auditing §B.0, **the supervisor needs zero new public APIs on existing kernels** for the core call chain. The new `AccountSnapshotProvider` port lives **inside `oya-foundry-supervisor-kernel`** (new crate) — no existing kernel signature changes.

We compose: `RoutePolicy::select`, `UsageEnforcement::check_limit`, `check_silent_switch`, `validate_usage`, `finalize_line`, `enforce`/`enforce_for_tenant`, `ProviderAccount::degrade`.

**Lean-a10 declaration count: 0 new public APIs on existing kernels.** If during execution a missing primitive surfaces, the executor MUST stop, open an ADR, and bump the owning kernel's version BEFORE adding the API. Codified as an open question.

### B.3 Daemon lifecycle (process tree + signals + fsync points — unchanged structurally from v3)

```
                supervisor-app (PID S)
                    │  tokio::spawn_blocking per spawn (max_in_flight semaphore)
                    ▼
              SessionDriver::spawn_for_message  →  fork+exec  →  CLI subprocess (PID C)
                    │  ── reads message_body from stdin, writes draft response to stdout
                    │  ── on completion, touches `stop_hook_path` (empty file); exits
                    ▼
              supervisor-app polls (stop_hook_path exists) every 100 ms
                    │  on hook: read child stdout buffer; inbox.commit(response)
                    │  on watchdog (T > config.watchdog_secs): SIGTERM, wait 1s, SIGKILL
                    ▼
              outbox.append → write `outbox/.tmp-{uuid}` → rename to `outbox/{seq}.jsonl`
```

**fsync points (Branch Y best-effort):** unchanged from v2 §B.3 (lock file `std::fs::write` no fsync; outbox tmp→rename only; commit unlinks lock AFTER outbox rename). Crash recovery: `JsonlInbox::replay_cursor()` reads `inbox/cursor`; orphan `.tmp-` GC'd by `JsonlOutbox::recover()`; expired-TTL locks released.

**Inbox state machine table** identical to v2 §B.3 (canonical reference v2:368-380). **v4 patch #6 amendment:** the `Locked → DeadLettered` transition is implemented atomically by `InboxStore::dead_letter()` per §B.2.3 doc-comment — the lock is consumed by the same call that writes the dead-letter row; no separate `release` precedes it.

### B.4 Multi-account × multi-provider call chain (composes B.0 only)

```
SupervisorApp::tick_once(now):
  1. inbox.peek_lock(TTL=30s)?
  2. let accounts = self.accounts.snapshot();          // AccountSnapshotProvider port (v3 diff #3)
  3. let constraints = RouteConstraints::new(model_hint);
  4. let exp = RoutePolicy::select(&accounts, &constraints)?;
  5. let acc = accounts.iter().find(|a| a.id == exp.chosen_account_id).unwrap();
  6. check_silent_switch(&others, acc)?;               // account-domain:171
  7. let window: &UsageWindow = self.window_for(acc.id);
  7.5 // v3 diff #2 — pre-flight projection check before live check_limit:
     //   `UsageWindow` is a value type (account-domain:252; fields all owned
     //   numeric/enum → Clone is safe). Project the tentative spend BEFORE
     //   spawning so credit isn't burned on a request that already exceeds budget.
     let mut projected: UsageWindow = window.clone();
     projected.tokens_out = projected.tokens_out.saturating_add(snap.projected_tokens_p95);
     UsageEnforcement::check_limit(&projected, now, budget_tokens)?;   // FAIL-FAST projection
  8. let verdict = UsageEnforcement::check_limit(window, now, budget_tokens)?;   // live (source of truth)
  9. match verdict {
        WithinLimit{..}    => proceed,
        OverUsageLimit{..} => acc.degrade("over usage limit"); RETURN DemoteAndRetry,
        ReserveBreached{..}=> acc.degrade("reserve breached"); RETURN DemoteAndRetry,
        WindowExpired      => self.rotate_window(); recurse once,
     }
 10. let cap = self.capability_for_driver(driver_id);
 11. let tier_verdict = autonomy_ceiling_app::enforce(&cap, tenant_ceiling);
     match tier_verdict { Block{..} => RETURN TierBlocked, Allow => proceed }
 12. let snap = UsageWindowSnapshot::from_window(window, projected_p95);
 13. let ticket = SessionTicket { /* owned values */ };
 13.5 // v3 diff #12 — explicit cost-ceiling enforcement loop BEFORE spawn.
     // v4 patch #6 — `inbox.dead_letter()` is atomic over Locked→DeadLettered
     // (§B.2.3 doc-comment); no explicit release call required.
     if snap.projected_tokens_p95 > ticket.cost_ceiling_tokens {
         inbox.dead_letter(&message_id, "cost-ceiling exceeded")?;   // consumes the lock atomically
         emit_audit_event("UsageBlocked", &ticket);
         return Err(SupervisorError::UsageBlocked(
             format!("projected {} > ceiling {}", snap.projected_tokens_p95, ticket.cost_ceiling_tokens)));
     }
 14. driver.spawn_for_message(&ticket, &body)?;
 15. on completion: spend = supervisor_kernel::record_spend(&ticket, ti, to);
     let rec = supervisor_kernel::spend_to_usage_record(&spend, tenant_id, ts_ms);
     cloud_billing_kernel::validate_usage(&rec)?;                       // existing API
 16. inbox.commit(&message_id, &response_bytes);
 17. outbox.append(&response_bytes);
```

**Early-return lock-release rule (v4 patch #6 universality):** every `tick_once` early return after step 1 must terminate the lock through one of: `inbox.commit` (success), `inbox.release` (recoverable error), or `inbox.dead_letter` (terminal poison/ceiling). Steps 8, 9, 11 returning errors go through `inbox.release(id, reason)` before propagating the `Err`; step 13.5 uses `inbox.dead_letter` (atomic). No early return ever leaves the lock in `Locked` state.

**Multi-account fanout** is implemented by step 4: `RoutePolicy::select` walks `failover_order` (route-policy-kernel:64–94). Multiple Claude accounts register with the same `ProviderFamily::Claude` and distinct `AccountId` + `subscription_id`. The silent-switch guard (step 6) prevents accidental swaps mid-stream.

**Saturation (v3 diff #4):** before step 1, the app checks `self.in_flight_permits.try_acquire()`. If at capacity, return `TickOutcome::Saturated` immediately — no lock held, no driver spawned.

### B.5 Cross-CLI hook bridge + capability seed file

Three driver crates (renamed per prerequisite ADR; reference names assumed post-rename):
- `oya-foundry-claude-account-adapter` — Claude Code stop-hook
- `oya-foundry-codex-account-adapter` — Codex CLI exit-or-no-stop-hook semantics
- `oya-foundry-gemini-account-adapter` — Gemini CLI available signaling

Each implements `SessionDriver` (§B.2.3) and registers a stop-hook script. Supervisor polls file existence at 100 ms cadence.

**Capability seed file (real path):** `registry/capabilities/foundry-supervisor.toml` — **NEW FILE**, parallel to `registry/capabilities/foundry-internal.json`. TOML hand-rolled (new schema needs new parser; flat list is simpler than the JSON shape). Schema (v3 diff #10 clarifies the round-trip):

```toml
[[driver]]
id = "claude-driver"
provider_family = "Claude"            # TOML string "Claude" ↔ Rust ProviderFamily::Claude
autonomy_tier = "T?"                  # FILLED BY CONFORMANCE build.rs (v3 diff #7)
stop_hook_supported = ?               # FILLED BY CONFORMANCE
request_id_supported = true           # anthropic-idempotency-key (≤256 chars)

[[driver]]
id = "codex-driver"
provider_family = "OpenAIOrCodex"     # CamelCase TOML string ↔ Rust ProviderFamily::OpenAiOrCodex (lowercase `i`)
autonomy_tier = "T?"
stop_hook_supported = ?
request_id_supported = true           # OpenAI Idempotency-Key header (≤255 chars, 24h)

[[driver]]
id = "gemini-driver"
provider_family = "Gemini"
autonomy_tier = "T?"
stop_hook_supported = ?
request_id_supported = false          # RISK: no documented idempotency key → tier demoted ≤ T2
```

**v3 diff #10 round-trip contract:** the TOML string `"OpenAIOrCodex"` (CamelCase, capital `I`) is parsed by `ProviderFamily::try_from` at account-kernel:48 into the Rust variant `ProviderFamily::OpenAiOrCodex` (lowercase `i`). Since `serde` is not a workspace dep, the round-trip is hand-rolled: the conformance crate calls `ProviderFamily::try_from(toml_string)` on parse, and `provider_family_to_str(variant)` on emit. A new unit test row (`tests/provider_family_round_trip.rs`, §A.5) asserts the round-trip is total over all 4 variants.

**Conformance crate (v3 diff #7):** `build.rs` runs each driver against a fake server, measures stop-hook count + idempotency replay + crash-resilience, and writes the measured T-tier into `registry/capabilities/foundry-supervisor.toml`. `cargo test -p oya-foundry-supervisor-conformance` then reads the same file and asserts the triple equals the runtime measurement in the same invocation.

**Idempotency-key shape per provider:** Claude (`anthropic-idempotency-key`, ≤256 chars), OpenAI/Codex (`Idempotency-Key`, ≤255, 24h dedup), Gemini (**no public key** → degrade).

### B.6 Cost-ceiling integration (composes existing billing-kernel + v3 diff #12 enforcement)

- Supervisor-kernel emits `SpendRecord` (§B.2.1) per completed message.
- `spend_to_usage_record` packs into `cloud_billing_kernel::UsageRecord` using `UsageUnit::Token` (billing-kernel:14).
- Supervisor-app calls `cloud_billing_kernel::validate_usage(&record)` (billing-kernel:74).
- Line-item finalization (`finalize_line`, billing-kernel:87) requires `tax_jurisdiction` (billing-kernel:88–93). Supervisor does NOT finalize; that is a separate billing-app concern.

**v3 diff #12 enforcement loop (now explicit at §B.4 step 13.5):** before `driver.spawn_for_message`, assert `snap.projected_tokens_p95 <= ticket.cost_ceiling_tokens` OR call `inbox.dead_letter` (atomic; v4 patch #6) + emit audit event + return `SupervisorError::UsageBlocked`. Acceptance row C.21 enforces: ticket with `cost_ceiling_tokens=100` and `projected_tokens_p95=500` produces a `UsageBlocked` dead-letter row + audit event (zero spawn).

**`cost_ceiling_tokens` source:** SessionTicket carries an absolute upper bound from `SupervisorConfig::default_cost_ceiling`; the app may lower it based on `UsageWindowSnapshot::projected_tokens_p95`. `projected_tokens_p95` is computed from a per-`(provider, model_hint)` rolling window of the last 100 observed `tokens_in + tokens_out`, persisted in `inbox/.../stats.jsonl`. When `n < 10`: `projected_tokens_p95 = cost_ceiling_tokens` (conservative seed).

### B.7 Mountpoint decision (unchanged from v2/v3)

**Chosen: (ii) Direct hyper via `oya-http-runtime-hyper-adapter`.** `oya-foundry-api-rest-adapter` is confirmed-stub.

```rust
let router = Router::<SyncHandler>::new()
    .route(HttpMethod::Post, "/v1/supervisor/inbox", handler_to_sync(InboxIngest::new(...)))?
    .route(HttpMethod::Get,  "/v1/supervisor/health", handler_to_sync(HealthCheck))?
    .route(HttpMethod::Get,  "/v1/supervisor/outbox/{seq}", handler_to_sync(OutboxRead::new(...)))?;
oya_http_runtime_hyper_adapter::serve(addr, Arc::new(router), Arc::new(chain),
    ServerConfig::default().with_max_body_bytes(256*1024)).await?;
```

ADR: `ADR-supervisor-mountpoint-direct-hyper` documents the side-step; api-rest-adapter remains stub until M02-P04 lands a real router.

### B.8 Capability registry rows + Cedar policy paths (v3 diff #1 — extends, not bootstraps)

The supervisor publishes 6 capability rows. **Cedar paths real, not "TBD"**:

| capability_id | autonomy_tier | evidence_required | Cedar policy path |
|---------------|---------------|--------------------|---------------|
| `foundry.supervisor.spawn` | (per-driver via conformance) | true | `docs/policies/foundry-supervisor.cedar` (NEW) — extends `docs/policies/autonomy-ceiling.cedar` seed |
| `foundry.supervisor.commit` | T3PropAct | true | `docs/policies/foundry-supervisor.cedar` |
| `foundry.supervisor.dead_letter` | T2Suggest | true | `docs/policies/foundry-supervisor.cedar` |
| `foundry.supervisor.degrade_account` | T3PropAct | true | `docs/policies/foundry-supervisor.cedar` |
| `foundry.supervisor.rotate_window` | T2Suggest | true | `docs/policies/foundry-supervisor.cedar` |
| `foundry.supervisor.quarantine` | T2Suggest | true | `docs/policies/foundry-supervisor.cedar` |

**Seed policy (verified existing, `docs/policies/autonomy-ceiling.cedar`, 3 lines verbatim):**
```cedar
// M02-P05-IP-002 autonomy ceiling — T4 actuation disabled by default
forbid (principal, action == Action::"actuate-t4", resource);
permit (principal, action in [Action::"read-t1", Action::"suggest-t2"], resource);
```

**New file `docs/policies/foundry-supervisor.cedar`** (created by this plan, NOT a TBD) extends the seed with supervisor-specific `permit`/`forbid` clauses for the 6 capability ids. ADR `ADR-cedar-policy-extend-supervisor-capabilities` documents the extension contract: the supervisor file MUST be evaluated together with the autonomy-ceiling seed; the seed wins when both speak to the same action.

### B.9 Phase fan-in (v3 diff #6 — new phase carved)

**Owner phase: M02-P06-foundry-supervisor** (NEW; P01..P05 are existing — P01 stays `status: complete`). Phase INDEX skeleton declared at `.omc/plans/milestones/M02-foundry-preview/phases/P06-foundry-supervisor/INDEX.md` (this plan **declares the path**, does not create the file — see §B.10 row).

Cross-cutting rows:
- **M02-P01-provider-gateway** (cross-cut, complete; consumed surfaces only — RoutePolicy + UsageEnforcement + check_silent_switch).
- **M02-P02-multi-subscription-pool** — `AccountId × subscription_id` fanout (Cargo.toml:266–270 confirms `oya-foundry-provider-pool-kernel` exists).
- **M02-P05-capability-registry-autonomy** — `Capability` + `AutonomyTier` + autonomy-ceiling seed (capability-registry-kernel:49, autonomy-ceiling-kernel:13, `docs/policies/autonomy-ceiling.cedar`).

**Phase-slot verification (v4 patch #2):** ran `ls .omc/plans/milestones/M02-foundry-preview/phases/` — P00..P05 present, **P06 slot is free**. Execute under `grit claim --intent M02-P06-foundry-supervisor` BEFORE writing INDEX.md; grit protocol prevents collision.

The lean-a10 lane runs **only on the owner phase (P06)**. lean-a5-doc-coverage runs on owner. Cross-phase rows acknowledge dependency only; gates don't fire on them.

### B.10 Doc + ADR footprint (all paths re-anchored to P06; v3 diff #6)

| Artifact | Path |
|----------|------|
| Phase INDEX (path declared, not created here) | `.omc/plans/milestones/M02-foundry-preview/phases/P06-foundry-supervisor/INDEX.md` |
| Implementation plan | `.omc/plans/milestones/M02-foundry-preview/phases/P06-foundry-supervisor/IP-001-supervisor.md` |
| ADR — dep policy Y | `docs/decisions/ADR-NNNN-supervisor-dep-policy-branch-y.md` |
| ADR — mountpoint | `docs/decisions/ADR-NNNN-supervisor-mountpoint-direct-hyper.md` |
| ADR — durability | `docs/decisions/ADR-NNNN-jsonl-best-effort-durability.md` |
| ADR — public contract | `docs/decisions/ADR-NNNN-supervisor-public-contract-lean-a10.md` (declares zero new public APIs on existing kernels) |
| ADR — Cedar extension (v3 diff #1) | `docs/decisions/ADR-NNNN-cedar-policy-extend-supervisor-capabilities.md` — extends existing `docs/policies/autonomy-ceiling.cedar` seed |
| Cedar policy file (v3 diff #1) | `docs/policies/foundry-supervisor.cedar` (NEW, supervisor-scoped) |
| Doc-coverage suite (lean-a5) per crate | `docs/foundry/supervisor/{README,architecture,operations,security,sample-payloads}.md` for each of 4 new crates |
| Capability seed | `registry/capabilities/foundry-supervisor.toml` |
| Bench harness | `crates/oya-foundry-supervisor-app/benches/heartbeat.rs` |
| Conformance build script | `crates/oya-foundry-supervisor-conformance/build.rs` |
| **OpenBao default (v4 patch #4)** | Secret material lives in OpenBao via `oya-foundry-account-adapter-openbao` `SecretStorePort`. Adapter present (verified `OpenBaoAdapter: SecretStorePort` in-memory ref impl, 10 unit tests at `crates/oya-foundry-account-adapter-openbao/src/lib.rs:26+88-164`). Network-OpenBao upgrade tracked separately. Supervisor + drivers access secrets ONLY via the adapter port; never in-line secrets. Compliant with project-memory OpenBao-default directive. |
| **`registry/accounts/` directory (v4 patch #7)** | `registry/accounts/` — new scaffold artifact; directory does not yet exist (verified `ls registry/` shows adr/audit-chain/capabilities/catalog/data-class/docs/foundation-bypasses/glossary-vocabulary/mobile-native/openapi/placeholder-debt/quality/release only). Production `AccountSnapshotProvider` reads `registry/accounts/*.toml` from this directory. |
| **`tools/oya-foundry-fitness-predictable-naming/` binary scaffold (v4 patch #1)** | `tools/oya-foundry-fitness-predictable-naming/src/main.rs` — new binary scaffold; directory does not yet exist. Wraps `oya-foundry-fitness-predictable-naming-kernel::check()`. Pattern model: `tools/oya-foundry-fitness-banned-primitives/src/main.rs:15`. |

**Renames (prerequisite ADR — NOT part of this plan):**
- `oya-foundry-account-adapter-claude-code` → `oya-foundry-claude-account-adapter`
- `oya-foundry-account-adapter-codex-cli` → `oya-foundry-codex-account-adapter`
- `oya-foundry-account-adapter-gemini-cli` → `oya-foundry-gemini-account-adapter`

### B.11 Verification gates (concrete commands; v3 diff #8 + v4 patch #1 substitutions applied)

| # | Gate | Command |
|---|------|---------|
| 1 | Workspace builds | `cargo build --workspace` |
| 2 | Kernel unit tests | `cargo test -p oya-foundry-supervisor-kernel` |
| 3 | Adapter unit tests | `cargo test -p oya-foundry-jsonl-supervisor-adapter` |
| 4 | App lifecycle | `cargo test -p oya-foundry-supervisor-app --test lifecycle` |
| 5 | Crash injection | `cargo test -p oya-foundry-supervisor-app --test crash_injection -- --test-threads=1` |
| 6 | Watchdog timing | `cargo test -p oya-foundry-supervisor-app --test lifecycle watchdog_kill_p95` |
| 7 | Dead-letter lane | `cargo test -p oya-foundry-supervisor-app --test lifecycle poison_message_quarantine_after_3` |
| 7.5 | **Cost-ceiling enforcement (v3 diff #12)** | `cargo test -p oya-foundry-supervisor-app --test lifecycle cost_ceiling_blocks_when_projected_exceeds_ceiling` |
| 7.6 | **Saturation outcome (v3 diff #4)** | `cargo test -p oya-foundry-supervisor-app --test saturation` |
| 7.7 | **ProviderFamily round-trip (v3 diff #10)** | `cargo test -p oya-foundry-supervisor-conformance --test provider_family_round_trip` |
| 8 | Audit chain | `cargo test -p oya-foundry-supervisor-app --test audit_chain` |
| 9 | Conformance — Codex | `OYA_DRIVER=codex cargo test -p oya-foundry-supervisor-conformance -- stop_hook_codex --test-threads=1` |
| 10 | Conformance — Gemini | `OYA_DRIVER=gemini cargo test -p oya-foundry-supervisor-conformance -- stop_hook_gemini --test-threads=1` |
| 11 | Conformance — Claude | `OYA_DRIVER=claude cargo test -p oya-foundry-supervisor-conformance -- stop_hook_claude --test-threads=1` |
| 12 | Capability seed emitted (v3 diff #7 C.12.bis) | `test -f registry/capabilities/foundry-supervisor.toml && [ "$(grep -c '^autonomy_tier = \"T[1-4]\"' registry/capabilities/foundry-supervisor.toml)" -eq 3 ]` |
| 13 | Capability seed loads | `cargo test -p oya-foundry-supervisor-conformance -- seed_loads_and_publishes_to_registry` |
| 14 | Bench harness | `cargo test --release -p oya-foundry-supervisor-app --bench heartbeat` |
| 15 | Live smoke matrix | `OYA_LIVE_SMOKE=1 cargo test -p oya-foundry-supervisor-app --test matrix_3x2x2 -- --include-ignored --test-threads=1` |
| 16 | Lean-a10 (no kernel surface change) | `cargo public-api -p oya-foundry-route-policy-kernel \| diff - .omc/snapshots/route-policy-kernel.public-api.txt` (byte-identical) |
| 17 | Lean-a5 doc-coverage (substituted: `oya-check-doc-catalog` is library-only, no `main.rs`; verified path is `oya-dev-cli` which exists at `crates/oya-dev-cli/src/main.rs`) | `cargo run -p oya-dev-cli -- gate validate doc-catalog --crate oya-foundry-supervisor-kernel --require README,architecture,operations,security,sample-payloads` (repeat for each of 4 crates). |
| **18 — v4 patch #1** | Predictable-naming (new `[[bin]]` target `oya-foundry-fitness-predictable-naming` — grit unit #49 in §B.12; pattern model `tools/oya-foundry-fitness-banned-primitives/src/main.rs:15`) | `cargo run -p oya-foundry-fitness-predictable-naming -- --check crates/oya-foundry-supervisor-{kernel,app,conformance} crates/oya-foundry-jsonl-supervisor-adapter` |
| 19 | Banned primitives | `cargo run -p oya-foundry-fitness-banned-primitives -- --check crates/oya-foundry-supervisor-app` (binary already real per §B.0) |
| 20 | Net-new dep count | `cargo metadata --format-version 1 \| jq '[.packages[] \| select(.source != null) \| .name] \| length'` baseline-vs-after **equal** (Branch Y) |
| 21 | **Cedar fragment coverage (v3 diff #1)** | `cargo run -p oya-dev-cli -- gate cedar fragment-coverage --policy docs/policies/foundry-supervisor.cedar --capabilities registry/capabilities/foundry-supervisor.toml` (uses existing `cedar_fragment_coverage_gate.rs` in dev-cli, observed in §B.0 ls) |

### B.12 Sequencing + grit claim units (expanded per v3 diff #13)

| Order | Unit (file::Identifier) | Phase ref | Notes |
|-------|------------------------|-----------|-------|
| 1 | `docs/decisions/ADR-NNNN-supervisor-dep-policy-branch-y.md::header` | M02-P06 | Branch Y codified |
| 2 | `docs/decisions/ADR-NNNN-supervisor-public-contract-lean-a10.md::header` | M02-P06 | Zero new kernel APIs |
| 3 | `docs/decisions/ADR-NNNN-jsonl-best-effort-durability.md::header` | M02-P06 | Branch Y consequence |
| 4 | `docs/decisions/ADR-NNNN-supervisor-mountpoint-direct-hyper.md::header` | M02-P06 | Bypass rest-adapter stub |
| 5 | `docs/decisions/ADR-NNNN-cedar-policy-extend-supervisor-capabilities.md::header` | M02-P06 | v3 diff #1 |
| 6 | `docs/policies/foundry-supervisor.cedar::header` | M02-P06 | NEW cedar file (v3 diff #1) |
| 7 | `Cargo.toml::members` | M02-P06 | Add 4 new crates + 1 new binary to workspace |
| 8 | `crates/oya-foundry-supervisor-kernel/src/lib.rs::MessageId` | M02-P06 | New file |
| 9 | `crates/oya-foundry-supervisor-kernel/src/lib.rs::RequestId` | M02-P06 | |
| 10 | `crates/oya-foundry-supervisor-kernel/src/lib.rs::UsageWindowSnapshot` | M02-P06 | Audit-only (v3 diff #5) |
| 11 | `crates/oya-foundry-supervisor-kernel/src/lib.rs::SessionTicket` | M02-P06 | Value-only |
| 12 | `crates/oya-foundry-supervisor-kernel/src/lib.rs::InboxState` | M02-P06 | |
| 13 | `crates/oya-foundry-supervisor-kernel/src/lib.rs::SupervisorVerdict` | M02-P06 | |
| 14 | `crates/oya-foundry-supervisor-kernel/src/lib.rs::SupervisorError` | M02-P06 | |
| 15 | `crates/oya-foundry-supervisor-kernel/src/lib.rs::SpendRecord` | M02-P06 | |
| 16 | `crates/oya-foundry-supervisor-kernel/src/lib.rs::SupervisorConfig` | M02-P06 | v3 diff #4 |
| 17 | `crates/oya-foundry-supervisor-kernel/src/lib.rs::TickOutcome` | M02-P06 | v3 diff #4 |
| 18 | `crates/oya-foundry-supervisor-kernel/src/lib.rs::decide` | M02-P06 | Pure decision fn |
| 19 | `crates/oya-foundry-supervisor-kernel/src/lib.rs::next_inbox_state` | M02-P06 | State machine |
| 20 | `crates/oya-foundry-supervisor-kernel/src/lib.rs::record_spend` | M02-P06 | |
| 21 | `crates/oya-foundry-supervisor-kernel/src/lib.rs::spend_to_usage_record` | M02-P06 | Bridge to billing-kernel |
| 22 | `crates/oya-foundry-supervisor-kernel/src/lib.rs::SessionDriver` | M02-P06 | Sync port |
| 23 | `crates/oya-foundry-supervisor-kernel/src/lib.rs::InboxStore` | M02-P06 | Sync port; `dead_letter` atomic semantics per v4 patch #6 |
| 24 | `crates/oya-foundry-supervisor-kernel/src/lib.rs::AccountSnapshotProvider` | M02-P06 | v3 diff #3 port |
| 25 | `crates/oya-foundry-jsonl-supervisor-adapter/src/lib.rs::JsonlInbox` | M02-P06 | New file |
| 26 | `crates/oya-foundry-jsonl-supervisor-adapter/src/lib.rs::JsonlOutbox` | M02-P06 | |
| 27 | `crates/oya-foundry-jsonl-supervisor-adapter/src/lib.rs::encode_inbox_line` | M02-P06 | Hand-rolled framing |
| 28 | `crates/oya-foundry-jsonl-supervisor-adapter/src/lib.rs::decode_inbox_line` | M02-P06 | |
| 29 | `crates/oya-foundry-supervisor-app/src/lib.rs::SupervisorApp` | M02-P06 | |
| 30 | `crates/oya-foundry-supervisor-app/src/lib.rs::tick_once` | M02-P06 | |
| 31 | `crates/oya-foundry-supervisor-app/src/lib.rs::run_forever` | M02-P06 | tokio host |
| 32 | `crates/oya-foundry-supervisor-app/benches/heartbeat.rs::main` | M02-P06 | Bench bin |
| **v3 diff #13 — lifecycle.rs test expansion (sub-claim pattern `tests/lifecycle.rs::test_*`; expanded at scaffold time)** ||||
| 33a | `crates/oya-foundry-supervisor-app/tests/lifecycle.rs::test_spawn_and_commit_single_message` | M02-P06 | |
| 33b | `crates/oya-foundry-supervisor-app/tests/lifecycle.rs::test_restart_replays_from_cursor` | M02-P06 | |
| 33c | `crates/oya-foundry-supervisor-app/tests/lifecycle.rs::test_settle_writes_outbox` | M02-P06 | |
| 33d | `crates/oya-foundry-supervisor-app/tests/lifecycle.rs::test_watchdog_kill_p95` | M02-P06 | |
| 33e | `crates/oya-foundry-supervisor-app/tests/lifecycle.rs::test_poison_message_quarantine_after_3` | M02-P06 | |
| 33f | `crates/oya-foundry-supervisor-app/tests/lifecycle.rs::cost_ceiling_blocks_when_projected_exceeds_ceiling` | M02-P06 | v3 diff #12 |
| 33g | `crates/oya-foundry-supervisor-app/tests/lifecycle.rs::projected_tokens_p95_source` | M02-P06 | C.19 |
| 34a | `crates/oya-foundry-supervisor-app/tests/crash_injection.rs::test_sigkill_during_outbox_rename` | M02-P06 | |
| 34b | `crates/oya-foundry-supervisor-app/tests/crash_injection.rs::test_sigkill_during_lock_write` | M02-P06 | |
| 34c | `crates/oya-foundry-supervisor-app/tests/crash_injection.rs::test_sigkill_during_commit_unlink` | M02-P06 | |
| 35 | `crates/oya-foundry-supervisor-app/tests/audit_chain.rs::*` (pattern `test_audit_*`, expanded at scaffold) | M02-P06 | |
| 36 | `crates/oya-foundry-supervisor-app/tests/matrix_3x2x2.rs::*` (12 combinations, expanded at scaffold) | M02-P06 | Gated `$OYA_LIVE_SMOKE` |
| 37 | `crates/oya-foundry-supervisor-app/tests/saturation.rs::tick_once_returns_saturated_when_in_flight_at_capacity` | M02-P06 | v3 diff #4 |
| 38 | `crates/oya-foundry-supervisor-conformance/build.rs::main` | M02-P06 | Emits T-tier into seed (v3 diff #7) |
| 39a | `crates/oya-foundry-supervisor-conformance/src/lib.rs::measure_claude_tier` | M02-P06 | |
| 39b | `crates/oya-foundry-supervisor-conformance/src/lib.rs::measure_codex_tier` | M02-P06 | |
| 39c | `crates/oya-foundry-supervisor-conformance/src/lib.rs::measure_gemini_tier` | M02-P06 | |
| 39d | `crates/oya-foundry-supervisor-conformance/src/lib.rs::seed_loads_and_publishes_to_registry` | M02-P06 | |
| 40 | `crates/oya-foundry-supervisor-conformance/tests/provider_family_round_trip.rs::toml_string_round_trips_to_enum_variant_for_all_four_families` | M02-P06 | v3 diff #10 |
| 41 | `registry/capabilities/foundry-supervisor.toml::header` | M02-P06 | Seed file (build.rs writes T-tiers) |
| 42 | `.omc/plans/milestones/M02-foundry-preview/phases/P06-foundry-supervisor/INDEX.md::header` | M02-P06 | Phase INDEX skeleton |
| 43 | `.omc/plans/milestones/M02-foundry-preview/phases/P06-foundry-supervisor/IP-001-supervisor.md::header` | M02-P06 | Phase IP |
| 44 | `docs/foundry/supervisor/README.md::header` | M02-P06 | Doc-coverage row 1 |
| 45 | `docs/foundry/supervisor/architecture.md::header` | M02-P06 | Doc-coverage row 2 |
| 46 | `docs/foundry/supervisor/operations.md::header` | M02-P06 | Doc-coverage row 3 |
| 47 | `docs/foundry/supervisor/security.md::header` | M02-P06 | Doc-coverage row 4 |
| 48 | `docs/foundry/supervisor/sample-payloads.md::header` | M02-P06 | Doc-coverage row 5 |
| 49 | `tools/oya-foundry-fitness-predictable-naming/src/main.rs::main` | M02-P06 | **v4 patch #1** — new `[[bin]]` target wrapping `oya-foundry-fitness-predictable-naming-kernel::check()`. Unconditional grit claim. Pattern model: `tools/oya-foundry-fitness-banned-primitives/src/main.rs:15`. |

**Total: 49 base + 14 explicit sub-units (33a-g + 34a-c + 39a-d) + ~M scaffold-time wildcards for audit_chain (row 35) and matrix_3x2x2 (row 36, 12 combos).** Concretely: 63 explicit grit claim units + ~M scaffold-time wildcards (lower bound ~12 from matrix_3x2x2; audit_chain `test_audit_*` count emerges at scaffold).

---

## §C. Acceptance bar (testable; every row maps to a §B.11 command)

| # | Acceptance criterion | §B.11 command |
|---|---------------------|---------------|
| C.1 | `cargo build --workspace` exits 0 with 4 new crates compiled | row 1 |
| C.2 | Kernel unit tests pass; `SessionTicket` field-level inspection confirms zero `&`/`Arc`/`Box<dyn>` | row 2 |
| C.3 | Adapter unit tests pass; round-trip `encode_inbox_line` ↔ `decode_inbox_line` byte-identical for 100 random samples | row 3 |
| C.4 | Lifecycle test passes; supervisor processes 10 messages, fake driver, 10 outbox lines committed | row 4 |
| C.5 | **Crash-injection lane green** — SIGKILL at every flush point; recovery yields zero data loss or quarantine-with-reason | row 5 |
| C.6 | **Watchdog kill timing lane green** — fake driver `loop{sleep(1s)}` killed within 5.0s p95 | row 6 |
| C.7 | **Dead-letter lane green** — poison line → `dead-letter/` after 3 retries; tier demoted by 1 | row 7 |
| C.8 | Audit chain test passes; every spawn/commit emits `EvidenceRef` linked to capability_id | row 8 |
| C.9 | **Codex stop-hook verification** — artifact `codex-stop-hook.json` exists with `{stop_hook_count:1, idempotency_replay:ok}` | row 9 |
| C.10 | **Gemini stop-hook verification** — if `request_id_supported=false`, recorded tier demotion to ≤T2Suggest in artifact | row 10 |
| C.11 | Claude stop-hook verification — equivalent artifact at T1 baseline | row 11 |
| C.12 | Capability seed file `registry/capabilities/foundry-supervisor.toml` exists, exactly 3 `[[driver]]` blocks | row 12 |
| C.12.bis | v3 diff #7 — `grep -c '^autonomy_tier = "T[1-4]"' registry/capabilities/foundry-supervisor.toml == 3` (no `T?` placeholders survive build.rs) | row 12 |
| **C.13a — v3 diff #9** | Bench harness writes JSONL row `{"metric":"idle_tick_p95_tokens","value":<n>,"p":95}` with `value ≤ 25` | row 14 |
| **C.13b — v3 diff #9** | Bench harness writes JSONL row `{"metric":"restart_latency_p95_ms","value":<n>,"p":95}` with `value ≤ 1500` | row 14 |
| **C.13c — v3 diff #9 + v4 patch #3** | Bench harness writes JSONL row `{"metric":"rss_max_kib","value":<n>}` (sampled from `/proc/self/status` `VmRSS`) with `value ≤ 65536` (64 MiB). CI-only (Linux runners); macOS dev runs skip via `cfg(target_os="linux")`. macOS RSS path tracked in open-questions. | row 14 |
| **C.13d — v3 diff #9** | Bench harness writes JSONL row `{"metric":"watchdog_kill_p95_ms","value":<n>,"p":95}` with `value ≤ 5000` | row 14 |
| C.14 | **Live smoke matrix green** — 12 combinations each process ≥1 message, commit ≥1 outbox line | row 15 |
| C.15 | **Lean-a10 public-API snapshot byte-identical** for `oya-foundry-route-policy-kernel`, `oya-foundry-usage-window-kernel`, `oya-cloud-billing-kernel`, `oya-foundry-account-domain`, `oya-foundry-autonomy-ceiling-{kernel,domain,app}`, `oya-foundry-capability-registry-{kernel,domain,app}` | row 16 |
| C.16 | Lean-a5 doc-coverage green for all 4 new crates | row 17 |
| C.17 | Predictable-naming + banned-primitives lanes green for the 4 new crates | rows 18, 19 |
| C.18 | **Net-new external dep count = 0** (Branch Y) | row 20 |
| C.19 | `projected_tokens_p95` populated from rolling p95 of last 100 (provider, model_hint) samples; conservative seed = `cost_ceiling_tokens` when n<10 | row 4 (test 33g) |
| C.20 | Per-CLI `request_id` idempotency demonstrated under crash-replay (claude+codex byte-identical reply; gemini documented-degrade) | rows 9, 10, 11 |
| **C.21 — v3 diff #12** | Ticket with `cost_ceiling_tokens=100` and `projected_tokens_p95=500` produces a `UsageBlocked` dead-letter row + audit event; zero driver spawns | row 7.5 |
| **C.22 — v3 diff #4** | `tick_once` returns `TickOutcome::Saturated` (no lock acquired, no driver spawned) when `in_flight_permits == max_in_flight` | row 7.6 |
| **C.23 — v3 diff #10** | `ProviderFamily::try_from("OpenAIOrCodex") == Ok(ProviderFamily::OpenAiOrCodex)` round-trip total over all 4 variants (Aws/Claude/OpenAiOrCodex/Gemini) | row 7.7 |
| **C.24 — v3 diff #1** | `docs/policies/foundry-supervisor.cedar` exists and Cedar fragment coverage gate (B.11 row 21) reports 6/6 capability rows mapped | row 21 |

---

## §D. Output path

This plan: `/Users/jasonlee/oyatie/.omc/plans/ralplan-foundry-supervisor-simple-v4-2026-05-14.md`

Open-questions appended to: `/Users/jasonlee/oyatie/.omc/plans/open-questions.md` (new section `ralplan-foundry-supervisor-simple-v4 — 2026-05-14`).

---

## §E. Iteration change logs

### §E.v3 — Iteration 3 change log (14 diffs, preserved verbatim)

| Diff # | Section touched | Before (v2) | After (v3) |
|--------|----------------|-------------|-----------|
| 1 | §B.0 row "Cedar policy files", §B.8, §B.10 ADR list, §B.12 unit #5–6, §C.24 | "**None found**" / capability rows carry "Cedar policy path = TBD" | Verified `docs/policies/autonomy-ceiling.cedar` exists (3 lines, verbatim quoted in §B.0+§B.8); ADR refocused as `ADR-cedar-policy-extend-supervisor-capabilities`; new file `docs/policies/foundry-supervisor.cedar` added; cedar fragment coverage gate row added at B.11 row 21 |
| 2 | §B.4 step 7.5 (NEW) | Spawn proceeded after live `check_limit` only | Pre-flight projection: clone `UsageWindow` (safe — value type at account-domain:252, fields owned numeric/enum), add `snap.projected_tokens_p95` to `tokens_out`, fail-fast call `check_limit(&projected, …)` BEFORE live check |
| 3 | §B.2.3 (port) + §B.2.5 (config) + §B.12 unit #24 | Account source ambiguous (`self.accounts.snapshot()` undefined) | Declared `pub trait AccountSnapshotProvider { fn snapshot(&self) -> Vec<ProviderAccount>; }` inside supervisor-kernel; threaded as 3rd type param `A: AccountSnapshotProvider` on `SupervisorApp`; testable + swappable; zero existing-kernel surface change so lean-a10 does NOT fire |
| 4 | §B.2.5 (SupervisorConfig, TickOutcome::Saturated) + §A.5 row + §C.22 + §B.11 row 7.6 + grit units #16-17 + §B.12 unit #37 | `tick_once -> Result<MessageId, SupervisorError>` only Spawn/error outcomes | Added `SupervisorConfig { max_in_flight, blocking_pool_size, default_cost_ceiling, watchdog_secs, heartbeat_interval_secs }` and `TickOutcome { Spawned, Saturated, Idle, Quarantined }`; saturation test row + acceptance row added |
| 5 | §B.2.1 `UsageWindowSnapshot` doc | Snapshot framed as freezing the window | Explicit "Audit-only. Live `UsageWindow` (account-domain:252) remains the enforcement+reconciliation source of truth; snapshot exists for ticket transport across blocking-pool boundaries." |
| 6 | Frontmatter + §B.9 + §B.10 + §B.12 (42 units re-anchored) | `owner_phase: M02-P01-provider-gateway` (P01 is complete; cross-cut storm) | New phase `M02-P06-foundry-supervisor` carved; P06 slot verified free; INDEX.md path declared; all grit units re-anchored to P06; P01 stays `complete` |
| 7 | §B.1 + §C.12.bis | Conformance crate "emits seed" — timing untold | "`build.rs` writes the seed at compile time; `cargo test` reads + asserts in same invocation"; C.12.bis enforces `grep -c '^autonomy_tier = "T[1-4]"' == 3` |
| 8 | §B.11 rows 17, 18 + §B.12 unit #49 | Invoked `cargo run -p oya-check-doc-catalog` and `cargo run -p oya-foundry-fitness-predictable-naming-kernel` (both library-only, no `main.rs`) | Row 17: substituted with `oya-dev-cli` (`crates/oya-dev-cli/src/main.rs`). Row 18: new `[[bin]]` target `oya-foundry-fitness-predictable-naming` declared as grit unit #49 |
| 9 | §C.13 → 4 rows (a,b,c,d) | Single combined "C.13 Bench produces 4 metrics" | Split into C.13a (idle-tick p95 ≤ 25 tok), C.13b (restart p95 ≤ 1500 ms), C.13c (RSS ≤ 64 MiB via `/proc/self/status` `VmRSS`), C.13d (watchdog kill p95 ≤ 5 s); JSONL schema declared |
| 10 | §B.2.1 + §B.5 + §A.5 + §C.23 + §B.11 row 7.7 + §B.12 unit #40 | TOML string was just `"OpenAIOrCodex"` with no contract callout | Explicit round-trip: TOML `"OpenAIOrCodex"` ↔ Rust `ProviderFamily::OpenAiOrCodex` (lowercase `i`, account-kernel:48); hand-rolled; new test `provider_family_round_trip` asserts totality |
| 11 | §A.3 Option B cons | Dismissive | Fairer: real benefit (per-driver crash isolation); pays for itself only at >10 msg/s sustained; broker = Option A in disguise. Verdict reframed: rejected on cost-benefit, not principle |
| 12 | §B.4 step 13.5 (NEW) + §C.21 + §B.11 row 7.5 + §B.6 enforcement paragraph | Cost-ceiling carried on ticket but no enforcement | Explicit pre-spawn check: `if snap.projected_tokens_p95 > ticket.cost_ceiling_tokens { dead_letter + audit event + return UsageBlocked }`. C.21: ceiling=100, projected=500 → `UsageBlocked` dead-letter + zero spawns |
| 13 | §B.12 rows 29-32 expansion | Wildcards | Lifecycle expanded to 7 explicit sub-rows (33a–33g); crash_injection 3 sub-rows (34a–34c); audit_chain + matrix retain sub-claim patterns for scaffold-time expansion |
| 14 | §B.0 row "OpenBao adapter" + §B.10 row "OpenBao default" + open-questions | OpenBao constraint implicit | Explicit row: secrets via `oya-foundry-account-adapter-openbao` `SecretStorePort`; supervisor + drivers never in-line secrets |

### §E.v4 — Iteration 4 change log (7 narrow patches, addendum to §E.v3)

| Patch # | Section touched | Before (v3) | After (v4) |
|---------|----------------|-------------|-----------|
| 1 | §B.11 row 18 + §B.12 unit #49 + §B.1 + §B.10 | Row 18 carried `Option A (preferred)` + `Option B (fallback): cargo test …`; §B.12 unit #49 marked "ONLY claimed if Option A path is taken … drops to grit unit #49(unused) … if Option B fallback is taken" | Row 18 collapsed to single Option A command: `cargo run -p oya-foundry-fitness-predictable-naming -- --check crates/oya-foundry-supervisor-{kernel,app,conformance} crates/oya-foundry-jsonl-supervisor-adapter`. §B.12 unit #49 unconditional (Option B language deleted). New scaffold artifact `tools/oya-foundry-fitness-predictable-naming/` declared in §B.1 + §B.10 (verified directory absent via `ls tools/`; pattern model `tools/oya-foundry-fitness-banned-primitives/src/main.rs:15`) |
| 2 | §B.9 v3:466 | "If during execution P06 is found taken by a parallel claim, fall back to P07; do not silently reuse a complete phase." | "Execute under `grit claim --intent M02-P06-foundry-supervisor` BEFORE writing INDEX.md; grit protocol prevents collision." |
| 3 | §C.13c | `value ≤ 65536` (64 MiB) — no OS qualifier | Appended exactly: `CI-only (Linux runners); macOS dev runs skip via `cfg(target_os="linux")`. macOS RSS path tracked in open-questions.` Annotation is one-off because `/proc/self/status` `VmRSS` is Linux-specific; not extended to other rows |
| 4 | §B.0 OpenBao row + §B.10 OpenBao row | "(verify before merge — RISK if absent)" + "RISK: if `oya-foundry-account-adapter-openbao` is not yet a real crate at execution time, executor stops and opens an ADR to bootstrap it" | "Adapter present (verified `OpenBaoAdapter: SecretStorePort` in-memory ref impl, 10 unit tests at `crates/oya-foundry-account-adapter-openbao/src/lib.rs:26+88-164`). Network-OpenBao upgrade tracked separately." RISK clauses deleted |
| 5 | §B.12 footer + §F | "49 base + N test-expansion units" (stale; 33a-g + 34a-c + 39a-d already explicit) | "49 base + 14 explicit sub-units (33a-g + 34a-c + 39a-d) + ~M scaffold-time wildcards for audit_chain (row 35) and matrix_3x2x2 (row 36, 12 combos)." Concrete: 63 explicit + ~M wildcards. §F mirrored |
| 6 | §B.2.3 `InboxStore::dead_letter` doc + §B.3 state machine note + §B.4 step 13.5 + §B.4 early-return rule + §B.6 | `dead_letter` had no atomicity contract; step 13.5 called `dead_letter` after `peek_lock` without releasing the lock; `Locked → DeadLettered` transition undeclared | Option β chosen: `InboxStore::dead_letter()` doc-comment now declares atomic Locked→DeadLettered (consumes lock; `InvalidTransition` if called on unlocked message). §B.3 amendment + §B.4 step 13.5 comment cite atomic semantics. §B.4 adds universal early-return rule: every early return after step 1 terminates lock via `commit` | `release` | `dead_letter`. §B.6 updated to cite atomicity. Open question opened for trait doc-comment authority |
| 7 | §B.2.5 + §B.10 | `AccountSnapshotProvider` had no production-wiring note; `registry/accounts/` undeclared as scaffold | §B.2.5: "Production impl: composite over `oya-foundry-account-adapter-{claude,codex,gemini}` reading `registry/accounts/*.toml`. Test impl: in-memory `Vec<ProviderAccount>` shim." §B.10: new `registry/accounts/` scaffold-artifact row (verified directory absent via `ls registry/`) |

**Cumulative diff count: 21 (v3: 14 + v4: 7) — all applied with cited file paths and §-anchored placement.**

---

## §F. Plan summary (executor cheat sheet)

- **Mode:** deliberate; iteration 4 of 5; supersedes v3.
- **Dep branch:** Y (zero net-new external deps).
- **New crates:** 4 (kernel + app + adapter + conformance).
- **New binaries:** 1 (`tools/oya-foundry-fitness-predictable-naming` — v4 patch #1).
- **New public APIs on existing kernels:** 0.
- **Phase owner:** **M02-P06-foundry-supervisor** (claimed via `grit claim --intent M02-P06-foundry-supervisor` before INDEX.md).
- **Cedar:** seed `docs/policies/autonomy-ceiling.cedar` exists (3 lines, verified); new `docs/policies/foundry-supervisor.cedar` extends it.
- **Grit claim units:** 49 base + 14 explicit sub-units (33a-g + 34a-c + 39a-d) + ~M scaffold-time wildcards (audit_chain row 35; matrix_3x2x2 row 36 = 12 combos). Concrete: **63 explicit + ~M wildcards**.
- **24 acceptance rows, 21 verification commands.**
- **3 critical RISKs declared with mitigation lanes:** torn-write, hung-CLI, poison-message. OpenBao adapter RISK retired in v4 (adapter verified present, `crates/oya-foundry-account-adapter-openbao/src/lib.rs:26+88-164`).
- **InboxStore::dead_letter() atomic semantics** (v4 patch #6) — Locked→DeadLettered consumed in single call; no orphan reservations.

**Does this plan capture your intent?**
- "proceed" — hand off to Architect + Critic for iteration-4 consensus review
- "adjust [X]" — return to interview
- "restart" — discard
