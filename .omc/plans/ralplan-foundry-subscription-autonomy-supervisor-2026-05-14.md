---
doc_class: Plan
plan_class: ralplan
mode: deliberate
status: pending approval
authored_by: planner
authored_at: 2026-05-14
milestone: M02-foundry-preview
phases_touched:
- P01-provider-gateway
- P02-visibility-operator-plane
- P05-capability-registry-autonomy
title: Foundry Subscription-Autonomy Supervisor (Codex + Claude Code + Gemini parity
  for claude-heartbeat)
reviewers:
- architect
- critic
length_cap: 220
purpose: Auto-backfilled purpose for ralplan-foundry-subscription-autonomy-supervisor-2026-05-14.md
---
# RALPLAN — Foundry Subscription-Autonomy Supervisor

> Rust-native re-implementation of `Siigari/claude-heartbeat` for **Codex CLI, Claude Code, and Gemini CLI** as a Foundry primitive. Subscription-credit autonomy, restart-per-message statelessness, JSONL inbox/outbox, hyper-only webhook surface. **No axum, no new external crates** (justified exceptions need explicit ADR + benchmark).

---

## A. RALPLAN-DR Summary

### A.1 First-Principles Principles (5)

1. **Subscription-mode autonomy is a Foundry primitive, not a script.** If the Foundry charter says “every provider operates in subscription + API auth modes” (M02 acceptance gate), then a supervised, restartable, hook-driven CLI session must be a first-class kernel surface — not a Node.js sidecar copied per repo.
2. **Fresh subprocess per message ≡ statelessness.** The reason `claude-heartbeat` works is exactly the reason `-p` works: each unit of work starts from a clean context. The kernel must enforce *one task per session* as a contract, not a convention. (Token-budget bench is the proof.)
3. **Cross-CLI parity is a conformance test, not a translation layer.** The three CLIs differ in hook surface (Claude Code has stop-hook + JSON IPC; Codex CLI uses MCP-style stdio; Gemini CLI surfaces stop-hook only since v0.3). Parity is demonstrated by a *shared conformance suite the three adapters all pass*, not by hand-coded translators.
4. **Hyper-only HTTP backbone (ADR-0090).** The webhook receiver, cron-trigger relay, and any Discord/Slack outbound MUST land on `oya-http-runtime-hyper-adapter` (Layer 5). Layers 1–4 stay std-only. No new framework deps.
5. **OpenBao-rooted secrets, JSONL-rooted payloads, evidence-emitting invocations.** Provider tokens are `SecretReference` only (ADR-0090 chain, ADR-0063 doc-coverage). The inbox/outbox files carry message bodies; tokens live in OpenBao and are dereferenced *inside the daemon process*. Every restart emits an evidence row for the capability registry (ADR-0003).

### A.2 Decision Drivers (top 3)

1. **Cross-CLI hook surface asymmetry.** Claude Code stop-hooks are well-documented; Codex CLI’s autonomous-loop primitive is the MCP server harness + `--ask-for-approval never`; Gemini CLI lacks stable session-stop hooks in some versions. Driver: the abstraction MUST work with stdout-sentinel parsing or PTY scrape as fallback, marked as risk lanes.
2. **Cost-ceiling enforcement is non-negotiable.** Runaway-loop risk (poison-message restart cascade) violates `oya-cloud-billing-kernel` cost-ceiling + `oya-foundry-usage-window-kernel` quotas. Driver: the policy port must be in the kernel, not the adapter — same crate boundary as `RoutePolicy`.
3. **Zero new external deps target.** Adding `signal-hook`, `nix`, `notify`, `crossbeam-channel`, `tokio-process` etc. is each an ADR. Driver: rely on `tokio::process::Command`, `tokio::fs`, `tokio::sync`, `std::os::unix::process` (already pulled via hyper/tokio); JSONL parsing uses `serde_json` line-mode (already transitive); pty fallback uses `nix` only behind a `pty` feature flag with ADR.

### A.3 Viable Options (3)

#### Option A — Single shared supervisor crate with CLI-specific `SessionDriver` adapter trait
- **Pros:** (1) one binary to operate; (2) trivial RPC story (in-process); (3) lowest doc surface (single ADR, single runbook).
- **Cons:** (1) single point of failure (one panic kills three CLIs); (2) couples release cadence (Gemini hook change forces full daemon redeploy); (3) cross-CLI memory leaks bleed into each other.

#### Option B — Per-CLI supervisor binaries + shared kernel
- **Pros:** (1) blast radius isolated per CLI; (2) independent release cadence; (3) per-CLI metrics/cgroups easy.
- **Cons:** (1) 3× operator surface (3 systemd units, 3 ports); (2) cron-trigger / webhook receiver needs replication or external mux; (3) higher boilerplate.

#### Option C — **Hybrid: one daemon, pluggable session drivers, per-CLI worker subprocesses** (RECOMMENDED)
- **Pros:** (1) one operator surface + hyper listener; (2) each CLI runs in its own *worker* subprocess so a crash quarantines to one provider; (3) shared kernel proves parity; (4) maps cleanly to the existing 12-layer enum (`-kernel`, `-runtime`, `-app`).
- **Cons:** (1) supervisor-of-supervisors complexity in lifecycle; (2) needs IPC contract between daemon and worker (we use stdio + JSONL frames — no new dep); (3) more ADRs to write (worker contract + supervision tree).

**Decision (provisional, subject to Architect/Critic):** **Option C**. Maps to existing `account-{kernel,domain,runtime,app}` + `account-adapter-*` layout. Allows the `runtime` crate to act as the daemon and each `account-adapter-*` to act as a worker driver while keeping the `kernel` pure.

> Invalidation note for any future “Option D — embed each CLI as a library”: rejected — Codex CLI and Gemini CLI ship as binaries with subscription auth caches in user-home; library embedding would duplicate auth state and violate ToS (ADR-pending, see §B.10).

### A.4 Pre-mortem (3 scenarios — DELIBERATE mode mandatory)

1. **Inbox race on concurrent producers.**
   *Scenario:* Two relays (webhook + cron) append to `io/inbox.jsonl` at the same instant. The supervisor reads with byte-offset cursor, but the second writer’s line is partial when the cursor crosses it. Result: corrupted JSON line is treated as poison and the session crashes.
   *Design defenses:* (a) writers MUST use `O_APPEND` + bounded line size (`InboxFrame::MAX_BYTES = 64 KiB`) so the kernel write is atomic on POSIX up to `PIPE_BUF`; (b) reader uses *line-aware* readahead — never advance cursor until `\n` is observed AND `serde_json::from_str` succeeds; (c) on parse failure, write to `io/quarantine.jsonl` and advance cursor *past* the bad line; emit `inbox.parse_failed` evidence row. (d) acceptance: `oya-foundry-supervisor-conformance` lane runs a fuzzer that fan-writes 10 producers × 10k messages and asserts zero loss + zero double-delivery.

2. **Hung CLI process leaks file handles or credits.**
   *Scenario:* The CLI binary hangs after model response but before stdout close (we’ve seen this in Claude Code 1.x when MCP server stalls). Watchdog kills with `SIGTERM`, then `SIGKILL`, but inherited file descriptors (stdout pipe, hook socket) leak in the daemon. Over hours, FD exhaustion. Worse: a partial response counted against subscription budget but never reaches outbox — operator sees “message consumed, no reply.”
   *Design defenses:* (a) supervisor opens worker stdio via a dedicated `Stdio::piped()` set, closes the *daemon-side* read half once child exits, asserted by `cargo test --features=fd-leak-canary`; (b) `WatchdogPolicy::kill_grace_secs` (default 5) between TERM and KILL; (c) on watchdog kill, emit `session.killed_partial` with token-cost estimate from `oya-foundry-usage-window-kernel`; (d) operator-plane shows `partial_response_count` per CLI in `/v1/foundry/supervisor/health`.

3. **Restart loop on poison message.**
   *Scenario:* A message in `inbox.jsonl` crashes the CLI on parse (e.g. malformed tool-call). Supervisor sees subprocess exit code != 0, restarts with the *same* message, crashes again. Burns the entire daily subscription quota in <60s.
   *Design defenses:* (a) the kernel `RestartPolicy` enforces *exponential backoff* with a *budget ceiling* (`max_restarts_per_window`, default 3 per 5min window — exactly mirrors `oya-foundry-usage-window-kernel` 5h-window pattern); (b) on third consecutive crash with the same `message_id`, the message is moved to `io/quarantine.jsonl` and the session resumes with the *next* message; (c) every kill emits an evidence row consumed by `oya-cloud-billing-kernel` so cost-ceiling pre-emption is in the same audit chain; (d) integration test: feed 1 poison + 99 valid messages, assert exactly 1 quarantine row and 99 outbox replies.

### A.5 Expanded Test Plan (DELIBERATE mode)

| Lane | Scope | Owner | Acceptance |
|---|---|---|---|
| **Unit** | `kernel` value types: `InboxFrame`, `OutboxFrame`, `RestartPolicy`, `WatchdogPolicy`, `SessionState`, `OffsetCursor` | `oya-foundry-supervisor-kernel` | ≥ 60 tests, ≥ 95 % line coverage |
| **Integration (in-proc)** | `runtime` driving a *mock* `SessionDriver` that scripts (a) clean exit, (b) hang, (c) crash, (d) partial-output, (e) poison-restart | `oya-foundry-supervisor-runtime` | Pre-mortem scenarios 1-3 reproduced + defenses verified |
| **Integration (live)** | Real Codex / Claude Code / Gemini binaries against a stub MCP echo server | `oya-foundry-supervisor-conformance` | 3 × 50-message round-trip + restart-per-message bench |
| **E2E** | Operator-plane HTTP control surface (start/stop/inject/drain) via hyper, evidence chain reaches `oya-cloud-billing-kernel`, cost-ceiling triggers stop | `oya-foundry-supervisor-app` | Cost-ceiling cuts off within 1 message of breach |
| **Observability** | OTel spans per restart, per hook fire, per outbox flush; metrics: `restart_latency_p95_ms`, `idle_tick_tokens_p50`, `partial_response_count_total` | `oya-http-telemetry-middleware-infrastructure` | All three metrics shipped to ops dashboard P02 |
| **Perf budget** | Idle tick ≤ 25 tokens (95th percentile); restart latency p95 ≤ 1.5 s; supervisor RSS ≤ 64 MiB with 3 idle workers | bench harness | Stat-significant (n ≥ 1000) |

---

## B. PRD-style Implementation Plan

### B.1 Crate decomposition (new + extended)

| Crate | Layer | Status | One-line BNF / 12-layer justification | Wires into |
|---|---|---|---|---|
| `oya-foundry-supervisor-kernel` | kernel (Layer 2) | **new** | `<oya>-<foundry>-<supervisor>-<kernel>` — pure value types for sessions/inboxes/policies; no I/O. Mirrors `oya-foundry-usage-window-kernel`. | consumed by runtime + every adapter |
| `oya-foundry-supervisor-domain` | domain (Layer 3) | **new** | Owns the `SessionLifecycle` state machine (`Spawned → Injected → Draining → Exited → Quarantined`); std-only. | re-exports kernel; consumed by runtime |
| `oya-foundry-supervisor-app` | app (Layer 4) | **new** | Operator-plane use-cases: `StartSupervisor`, `InjectMessage`, `DrainAndStop`, `QueryHealth`. Calls ports only. | hyper layer 5 above |
| `oya-foundry-supervisor-runtime` | runtime (Layer 5) | **new** | The daemon binary entrypoint. Hosts hyper listener + `tokio::process::Command` worker tree + JSONL persistence. The ONLY new crate that touches the network. | parents: supervisor-app + hyper-runtime |
| `oya-foundry-supervisor-adapter-jsonl` | adapter (Layer 5) | **new** | Reference `InboxSource`/`OutboxSink` impl over `io/inbox.jsonl` + `io/outbox.jsonl` + `io/.inbox-offset` (byte cursor). | supervisor-runtime |
| `oya-foundry-supervisor-adapter-claude-code` | adapter (Layer 5) | **new** | `SessionDriver` impl that spawns `claude` CLI, mounts stop-hook via `~/.claude/hooks/`, parses JSON IPC, signals `.restart`. | supervisor-runtime; reuses `oya-foundry-account-adapter-claude-code` for auth |
| `oya-foundry-supervisor-adapter-codex-cli` | adapter (Layer 5) | **new** | `SessionDriver` impl for `codex` CLI; mounts stdio-MCP harness; uses sentinel parsing if stop-hook unavailable (RISK §B.4). | reuses `oya-foundry-account-adapter-codex-cli` |
| `oya-foundry-supervisor-adapter-gemini-cli` | adapter (Layer 5) | **new** | `SessionDriver` impl for `gemini` CLI; stop-hook on ≥ v0.3, PTY-scrape fallback gated behind `pty` feature (RISK §B.4). | reuses `oya-foundry-account-adapter-gemini-cli` |
| `oya-foundry-supervisor-adapter-webhook-hyper` | adapter (Layer 5) | **new** | Hyper-only inbound relay; POST `/v1/foundry/supervisor/inbox` writes one frame to JSONL. | depends only on hyper-runtime |
| `oya-foundry-supervisor-adapter-cron-tokio` | adapter (Layer 5) | **new** | Cron-trigger using `tokio::time::interval` + a 5-field cron parser (std-only, hand-rolled — see B.10 ADR). | no new external dep |
| `oya-foundry-supervisor-conformance` | dev-tool (Layer 6) | **new** | Shared conformance suite the three CLI adapters all run. Lives under `tools/` not `crates/` if test-only. | dev-dependency of each adapter |
| `oya-foundry-capability-registry-kernel` | kernel | **extended** | Adds `foundry.supervisor.*` capability rows (start/stop/inject/quarantine/health). | P05 |
| `oya-cloud-billing-kernel` | kernel | **extended** | Adds `LineItem::session_kind = Subscription` flag + restart-cost-row emission contract. | M-CC-M03-P03 |
| `oya-foundry-route-policy-kernel` | kernel | **extended (small)** | Adds policy hook: `RestartPolicy::should_restart(now, history) -> Allow | DenyForBudget`. | P01 |

> All names verified against v4 BNF (`oya-<axis>-<capability>-<layer>`) and the 12-layer enum (`kernel|domain|adapter|app|runtime`). Each crate carries a one-line justification in its `Cargo.toml` `description` field — required by `oya-foundry-fitness-predictable-naming-kernel`.

### B.2 Public contracts (kernel trait sketch — `oya-foundry-supervisor-kernel`)

```rust
// kernel; std-only; no async; no I/O
pub struct MessageId(pub String);                       // data_class: INTERNAL_ONLY
pub struct WorkerId(pub String);                        // data_class: INTERNAL_ONLY
pub enum CliFamily { ClaudeCode, CodexCli, GeminiCli }
pub struct InboxFrame { id: MessageId, body: String, received_at_unix_ms: u64 }
pub struct OutboxFrame { id: MessageId, worker: WorkerId, body: String, emitted_at_unix_ms: u64, tokens_used: u64 }
pub enum SessionState { Spawned, Injected, Draining, Exited(ExitKind), Quarantined(QuarantineReason) }
pub enum ExitKind { Clean, Crashed { code: i32 }, WatchdogKilled }
pub enum QuarantineReason { PoisonMessage { msg: MessageId, attempts: u8 }, BudgetExceeded }
pub struct RestartPolicy {
    pub max_restarts_per_window: u8,                    // default 3
    pub window_seconds: u32,                            // default 300
    pub backoff_initial_ms: u32,                        // default 250
    pub backoff_max_ms: u32,                            // default 4_000
}
pub struct WatchdogPolicy {
    pub idle_tick_seconds: u32,                         // default 60
    pub hang_timeout_seconds: u32,                      // default 300
    pub kill_grace_seconds: u32,                        // default 5
}
pub struct OffsetCursor { pub path: String, pub byte_offset: u64 }
```

```rust
// kernel ports (sync traits — adapters wrap async in runtime layer)
pub trait SessionDriver {
    fn spawn(&self, auth: &AccountRef) -> Result<WorkerHandle, SpawnError>;
    fn inject(&self, w: &WorkerHandle, frame: &InboxFrame) -> Result<(), InjectError>;
    fn drain(&self, w: &WorkerHandle) -> Result<Vec<OutboxFrame>, DrainError>;
    fn kill(&self, w: &WorkerHandle, grace: WatchdogPolicy) -> Result<ExitKind, KillError>;
    fn health(&self, w: &WorkerHandle) -> SessionHealth;
}
pub trait InboxSource {
    fn next_frame(&mut self, cur: &mut OffsetCursor) -> Result<Option<InboxFrame>, IoErr>;
    fn quarantine(&mut self, frame: InboxFrame, reason: QuarantineReason) -> Result<(), IoErr>;
}
pub trait OutboxSink {
    fn append(&mut self, frame: OutboxFrame) -> Result<(), IoErr>;
    fn flush(&mut self) -> Result<(), IoErr>;
}
pub trait HeartbeatPolicy {
    fn should_idle_tick(&self, last_activity_unix_ms: u64, now_unix_ms: u64) -> bool;
    fn idle_tick_payload(&self) -> InboxFrame;          // the ~20-token tick
}
```

> Async lives in the **runtime** crate. The kernel is sync because (a) `oya-foundry-account-kernel` is sync, (b) test scaffolding is simpler, (c) keeps the kernel free of tokio. Adapters wrap each port in `tokio::task::spawn_blocking` or use `tokio::process` directly.

### B.3 Daemon lifecycle

```
oya-foundry-supervisor-runtime  (daemon, 1 process)
├── hyper listener (Layer 5; routes /v1/foundry/supervisor/*)
├── inbox watcher (tokio task; tails io/inbox.jsonl via OffsetCursor + inotify-free poll)
├── worker tree (one tokio supervisor task per CliFamily)
│    ├── worker[claude-code]
│    │    └── tokio::process::Command → child binary `claude` (PID N)
│    ├── worker[codex-cli]   → child binary `codex`     (PID M)
│    └── worker[gemini-cli]  → child binary `gemini`    (PID K)
├── outbox writer (tokio task; appends to io/outbox.jsonl; fsync per flush)
└── evidence emitter (tokio task; POSTs to capability-registry app)
```

**Restart-per-message contract:**
1. Inbox watcher reads next `InboxFrame`.
2. Worker supervisor sends `inject` to `SessionDriver`.
3. Adapter writes message to CLI stdin / hook payload, awaits `outbox` line(s) until CLI emits `__SESSION_DONE__` sentinel OR stop-hook fires `restart=true`.
4. Adapter calls `kill(w, grace)` and `spawn(new)` for the NEXT message. The *daemon process never forks* — only the worker subprocess is replaced. Daemon PID is stable for systemd / k8s `RestartPolicy: Never`.

**fsync points:**
- After every `OutboxSink::append` → `fsync(outbox.jsonl)`.
- After advancing `OffsetCursor` → `fsync(.inbox-offset)`.
- After moving a frame to quarantine → `fsync(quarantine.jsonl)` before cursor advances.

**Crash recovery:** on daemon restart, read `.inbox-offset` and resume. Any in-flight worker is treated as `Quarantined::DaemonCrash` and the next read uses the cursor as-of last fsync — at-least-once semantics, message dedup via `MessageId` on the consumer side.

**Signal handling:** `SIGTERM` → drain workers gracefully (5 s budget × N workers, parallel) → flush outbox → exit 0. `SIGINT` → same. `SIGKILL` is the operator’s last resort; recovery via the cursor-fsync invariant.

### B.4 Cross-CLI hook bridge (with explicit RISK)

| CLI | Mount mechanism | Fallback | Risk level | Mitigation |
|---|---|---|---|---|
| Claude Code | `~/.claude/hooks/heartbeat-{cli}.sh` writes JSON to a unix-domain socket the daemon owns. Mirrors `claude-heartbeat/hooks/heartbeat.js`. | stdout sentinel `__SESSION_DONE__` if hook fails | **LOW** | Documented hook API, used by ref impl |
| Codex CLI | Codex doesn’t ship a stop-hook in OSS builds (≤ v0.2). Driver runs `codex --no-interactive --json-output` and uses **stdout JSON streaming** to detect `{"type":"session_end"}`. | PTY scrape if streaming unavailable | **MEDIUM** | Codex driver carries a `protocol_version` probe at spawn; on mismatch, raises `DriverDegraded` and operator-plane shows a yellow indicator |
| Gemini CLI | Gemini ≥ v0.3 exposes `~/.gemini/hooks/`. Below v0.3 we need PTY scrape. | PTY scrape (`pty` feature, requires `nix` crate — **gated by ADR**) | **HIGH** | If `pty` feature is disabled, the Gemini adapter refuses to spawn and emits `DriverUnavailable`; operator must upgrade Gemini CLI to v0.3+ |

**Common contract (kernel-enforced):** every adapter MUST produce at least one `OutboxFrame` per `InboxFrame`, OR a `Quarantined` state with a reason. No silent drops.

### B.5 Cost-ceiling integration

- The `oya-cloud-billing-kernel::LineItem` is extended with `session_kind: SessionKind { Subscription, Api }` so subscription-mode usage is tracked separately for the M02 cost-ceiling gate.
- Every restart emits a `LineItem { unit: Token, quantity: estimated_tokens, session_kind: Subscription, line_id: <msg_id>-restart }`.
- `oya-foundry-route-policy-kernel::RestartPolicy::should_restart` consults `oya-foundry-usage-window-kernel::UsageEnforcement::check_limit`. On `OverUsageLimit` or `ReserveBreached` the verdict is `DenyForBudget` and the next message is **not** processed; supervisor enters `BudgetPaused` state surfaced via `/v1/foundry/supervisor/health`.
- This closes the “runaway loop” pre-mortem (§A.4-3) via the *same* audit chain that backs API-mode costs (ADR-0003 evidence).

### B.6 Hyper integration

The webhook receiver in `oya-foundry-supervisor-adapter-webhook-hyper`:
- Mounts on the existing daemon’s `oya-http-runtime-hyper-adapter` `Router`.
- Route: `POST /v1/foundry/supervisor/inbox` → body decoded as one JSONL line → appended to `io/inbox.jsonl` via `OutboxSink` (sic — same trait, the inbox is conceptually an outbox of the webhook).
- Auth: `oya-http-tenant-middleware-infrastructure` for tenant-id, `oya-foundry-account-adapter-openbao` for inbound HMAC secret.
- Operator-plane reads: `GET /v1/foundry/supervisor/health`, `GET /v1/foundry/supervisor/workers`, `GET /v1/foundry/supervisor/messages?since=<cursor>` — all flow through P02 read-only kernel projections (no writes here — writes only on `/inbox` and `/drain`).
- **No axum, no tower-http, no warp.** Layers 1-4 stay std-only; only the runtime crate touches hyper.

### B.7 Secret handling

- Provider tokens (Claude Code subscription token, Codex OpenAI token, Gemini Google token) live in OpenBao and surface as `SecretReference` only — never serialized to `io/inbox.jsonl`, `io/outbox.jsonl`, logs, or telemetry.
- Each `SessionDriver::spawn` dereferences `SecretReference → AuthToken` inside the worker process via the existing `oya-foundry-account-adapter-openbao` (no new dep).
- `Debug` impls stay redacted (mirrors `SecretReference` already in `oya-foundry-account-kernel`).
- Inbox/outbox HMAC secret similarly via `SecretReference`.
- **Audit-chain rule:** any failed dereference emits `auth.secret_missing` evidence WITHOUT logging the reference body.

### B.8 Capability-registry hook (operator-plane discovery)

New rows in `oya-foundry-capability-registry-kernel`:

| Capability ID | Autonomy tier | Note |
|---|---|---|
| `foundry.supervisor.start` | T2 | Operator-initiated; emits evidence |
| `foundry.supervisor.stop` | T2 | Drain-and-stop; emits evidence |
| `foundry.supervisor.inject_message` | T3 | Hot-path; webhook & cron both use this; evidence per call |
| `foundry.supervisor.quarantine_message` | T3 | Operator override |
| `foundry.supervisor.query_health` | T1 | Read-only; no evidence required |
| `foundry.supervisor.idle_tick` | T1 | Internal; included for token-budget visibility |

> These are *capabilities of the supervisor itself*, not of the CLIs it operates. The CLIs’ own capabilities remain whatever each CLI already publishes via MCP / hook.

### B.9 Phase plan (Milestone > Phase > IP)

Maps to **M02-foundry-preview**. Three phases touched:

#### M02-P01 (Provider Gateway) — new IPs

- **IP-005-supervisor-kernel-and-driver-contract**
  Deliverable: `oya-foundry-supervisor-{kernel,domain}` crates green; 60 unit tests; ports compile against `oya-foundry-account-kernel`.
  Completion bar: `cargo build -p oya-foundry-supervisor-kernel -p oya-foundry-supervisor-domain && cargo test`.

- **IP-006-supervisor-runtime-and-jsonl-adapter**
  Deliverable: `oya-foundry-supervisor-runtime` daemon + `oya-foundry-supervisor-adapter-jsonl` reference impl; in-proc mock driver passes pre-mortem scenarios 1-3.
  Completion bar: integration test `crates/oya-foundry-supervisor-runtime/tests/pre_mortem_scenarios.rs` green; fsync invariants verified.

- **IP-007-supervisor-adapter-claude-code** (parallel)
  Deliverable: stop-hook + JSON IPC driver; live-smoke against real `claude` binary.
  Completion bar: `oya-foundry-supervisor-conformance` 50-message round-trip green.

- **IP-008-supervisor-adapter-codex-cli** (parallel)
  Deliverable: stdout-JSON streaming driver; protocol-version probe; sentinel fallback.
  Completion bar: same conformance suite; degradation behavior asserted.

- **IP-009-supervisor-adapter-gemini-cli** (parallel)
  Deliverable: stop-hook driver (≥ v0.3); `pty` feature flag carries `nix` dep behind ADR.
  Completion bar: same conformance suite; `--no-default-features` build green.

#### M02-P02 (Visibility / Operator-plane) — new IPs

- **IP-004-supervisor-operator-plane-hyper-routes**
  Deliverable: `oya-foundry-supervisor-adapter-webhook-hyper` mounted on dashboard kernel; six read endpoints + two write endpoints (`inject`, `drain`).
  Completion bar: e2e via `tools/oya-dashboard-e2e`; negative test asserts every other verb returns 405 + `forbidden_write_attempt`.

#### M02-P05 (Capability Registry / Autonomy)

- **IP-004-supervisor-capabilities-and-autonomy-policy**
  Deliverable: six capability rows registered; Cedar policy entries for T2/T3 supervisor capabilities; evidence emission per inject/quarantine/restart.
  Completion bar: `cargo test -p oya-foundry-autonomy-ceiling-app -- supervisor_caps` green; evidence count = restart count.

#### Cross-phase

- **IP-cost-ceiling-integration** (touches `oya-cloud-billing-kernel` + `oya-foundry-route-policy-kernel` + `oya-foundry-usage-window-kernel`)
  Deliverable: `SessionKind::Subscription` LineItem flag; `RestartPolicy::should_restart` enforcement; `BudgetPaused` state.
  Completion bar: e2e — feed message stream past `usage_limit_pct`; assert `BudgetPaused` within 1 message of breach.

**Total: 7 implementation packages.** Parallelism: IP-005 serializes (kernel); IP-006 + IP-007/008/009 fan out 4-way after IP-005 merges; IP-004 (P02) + IP-004 (P05) + IP-cost can run alongside.

### B.10 Doc + ADR footprint

ADRs to write:

| ADR | Title | Why |
|---|---|---|
| ADR-0096 | Subscription-mode autonomy pattern (Foundry primitive) | Charter the supervisor as a first-class kernel surface; locks Option C |
| ADR-0097 | Restart-per-message contract (statelessness invariant) | Token-budget proof + audit-chain rule |
| ADR-0098 | Cross-CLI hook bridge (stop-hook + sentinel + PTY fallback) | Document the three protocols + degradation matrix |
| ADR-0099 | `pty` feature flag + `nix` dependency exception | Required by the zero-new-dep policy (ADR-0092) |
| ADR-0100 | JSONL inbox/outbox + byte-cursor durability semantics | At-least-once + fsync invariants |
| ADR-0101 | Cost-ceiling integration of subscription restarts | Extends `LineItem::session_kind`; closes runaway-loop risk |

Doc-coverage entries (lean-a5-doc-coverage gate):

- `docs/products/foundry/supervisor/README.md` (overview)
- `docs/products/foundry/supervisor/RUNBOOK.md` (start/stop/drain/quarantine)
- `docs/products/foundry/supervisor/INCIDENT-PLAYBOOK.md` (poison restart, FD leak, BudgetPaused recovery)
- `docs/products/foundry/supervisor/PERFORMANCE-BUDGET.md` (token + latency + RSS budgets w/ benchmark protocol)
- `docs/products/foundry/supervisor/PROVIDER-MATRIX.md` (per-CLI capability + degradation table)
- KR localization pack overlay under `docs/localization-packs/kr/foundry/supervisor/` (pack #1 per ADR-0064)
- Capability rows documented in `/registries/cross-cutting/artifact-capabilities-registry.json`

### B.11 Verification gates (concrete commands)

```bash
# per-crate build
cargo build -p oya-foundry-supervisor-kernel
cargo build -p oya-foundry-supervisor-domain
cargo build -p oya-foundry-supervisor-app
cargo build -p oya-foundry-supervisor-runtime
cargo build -p oya-foundry-supervisor-adapter-jsonl
cargo build -p oya-foundry-supervisor-adapter-claude-code
cargo build -p oya-foundry-supervisor-adapter-codex-cli
cargo build -p oya-foundry-supervisor-adapter-gemini-cli
cargo build -p oya-foundry-supervisor-adapter-webhook-hyper
cargo build -p oya-foundry-supervisor-adapter-cron-tokio

# unit + integration
cargo test -p oya-foundry-supervisor-kernel
cargo test -p oya-foundry-supervisor-runtime --test pre_mortem_scenarios
cargo test -p oya-foundry-supervisor-conformance

# e2e bench harness (token-per-idle-tick measurement)
cargo run --bin oya-foundry-supervisor-bench --   \
  --cli claude-code --duration 600 --report json   \
  > /evidence/supervisor-bench-claude-$(date +%s).json

# 3 × CLI live-smoke matrix (the M02 acceptance gate clause)
oya-foundry-supervisor-conformance --cli claude-code,codex-cli,gemini-cli --messages 50

# fitness lanes
oya-foundry-fitness-predictable-naming-kernel run --paths crates/oya-foundry-supervisor-*
oya-foundry-fitness-pool-routing-honor run
lean-a5-doc-coverage run --product foundry/supervisor
lean-a10-no-silent-regression run --base origin/main --head HEAD
```

### B.12 Migration / sequencing

Hard order:

1. **M02-P00 already complete** (account-kernel, SecretReference, ProviderFamily) — supervisor builds on this.
2. **IP-005 (supervisor kernel)** must merge first; it is the serialization bottleneck under `scaffold-locks-oyatie` (ADR-0054 grit claim).
3. **IP-006 + IP-007/008/009 fan out 4-way** after IP-005.
4. **IP-004 (P02)** + **IP-004 (P05)** require IP-006 (need a daemon to operate on).
5. **IP-cost-ceiling-integration** requires IP-006 + the already-merged `oya-foundry-usage-window-kernel`.
6. **Live-smoke matrix gate** (M02 acceptance clause) is the *last* merge; can't run before IP-007/008/009 all green.

---

## C. Acceptance Bar (concrete, testable “done”)

A senior engineer signs off iff *all* of the following are demonstrable:

1. **Three CLI adapters pass the shared conformance suite.** `oya-foundry-supervisor-conformance --cli claude-code,codex-cli,gemini-cli --messages 50` exits 0 with `lost=0, double_delivered=0, quarantined<=1`.
2. **Restart-per-message preserves fresh-session semantics.** Token-budget bench shows mean restart token cost within ±10 % of the upstream `claude-heartbeat` baseline (~500 tokens/restart for Claude Code; per-CLI baselines published in `PERFORMANCE-BUDGET.md`).
3. **Idle tick ≤ 25 tokens (p95).** `supervisor-bench` over 600 s of idle reports `idle_tick_tokens_p95 <= 25` with n ≥ 600.
4. **Watchdog kills hung sessions within `hang_timeout_seconds`.** Integration test `pre_mortem_scenarios::hung_cli_is_killed` asserts elapsed time between hang detection and `ExitKind::WatchdogKilled` ≤ `hang_timeout_seconds + kill_grace_seconds + 100 ms`.
5. **Hyper-based webhook + cron-trigger compile against workspace.** `cargo build -p oya-foundry-supervisor-adapter-webhook-hyper -p oya-foundry-supervisor-adapter-cron-tokio` green; `cargo deny check` shows no new external crate (or, if `pty` enabled, only `nix` behind ADR-0099).
6. **Live-smoke matrix green in CI.** 1ES-templated lane `oya-foundry-supervisor-live-smoke` green nightly across 3 CLIs × 2 auth modes = 6 cells.
7. **Zero new external crates introduced** OR ADR-0099 (`nix` for PTY fallback) + benchmark filed and accepted.
8. **No silent regression.** `lean-a10-no-silent-regression` green; any change to `oya-foundry-supervisor-kernel` public surface carries an ADR + version bump.
9. **Doc coverage green.** `lean-a5-doc-coverage --product foundry/supervisor` reports 100 % coverage across the six doc files in §B.10.
10. **Cost-ceiling cuts off within 1 message of breach.** `oya-foundry-supervisor-runtime/tests/budget_breach.rs` asserts `BudgetPaused` is entered after the first `EnforcementVerdict::OverUsageLimit`.

---

## D. ADR Block (mandatory for consensus-mode final plan)

> *(Final ADR will be authored on plan approval; sketch follows.)*

- **Decision:** Adopt Option C — single Foundry-owned daemon with pluggable per-CLI `SessionDriver` adapters; restart-per-message contract; hyper-only HTTP backbone; JSONL inbox/outbox with byte-cursor durability.
- **Drivers:** subscription-mode autonomy as Foundry primitive; cross-CLI parity via shared conformance suite; runaway-loop / cost-ceiling enforcement; zero-new-external-dep target.
- **Alternatives considered:**
  - Option A (shared supervisor, one process per CLI internally) — rejected: blast radius too large; release-cadence coupling.
  - Option B (per-CLI binaries + shared kernel) — rejected: 3× operator surface; cron/webhook duplication.
  - Option D (embed each CLI as a library) — rejected: violates ToS; duplicates auth state.
  - Keep Node.js claude-heartbeat as sidecar — rejected: violates ADR-0090 (hyper backbone), ADR-0092 (workspace-dep-seam), ADR-0064 (Rust-native standard).
- **Why chosen:** maps to existing `oya-foundry-account-{kernel,domain,runtime,app}` topology; allows per-CLI worker quarantine; lets the kernel stay sync + std-only; lets new doc-coverage land cleanly under `foundry/supervisor`.
- **Consequences:**
  - *Positive:* unified ops surface; cost-ceiling re-uses M02 audit chain; ADR-0090 hyper-only invariant preserved.
  - *Negative:* one daemon = one upgrade window for three CLIs; supervisor-of-supervisors complexity in lifecycle code.
- **Follow-ups:**
  - Decide whether to promote `oya-foundry-supervisor-conformance` to a workspace-wide fitness lane (probably yes — M02-P03 gate).
  - Track Codex CLI’s stop-hook roadmap; once available, retire stdout-sentinel fallback.
  - Track Gemini CLI versions; once v0.3+ is the floor, drop the `pty` feature flag and retire ADR-0099’s `nix` dep.

---

## E. Open Questions (extracted by planner; will be persisted to `.omc/plans/open-questions.md`)

- [ ] Confirm Codex CLI subscription-mode binary actually emits `{"type":"session_end"}` on `--json-output` in current OSS release (we assume yes; if not, the sentinel-only path is the primary, not the fallback). — Driver bar correctness.
- [ ] Confirm `oya-foundry-account-adapter-openbao` exposes a sync `dereference` or whether the supervisor needs an async wrapper (kernel stays sync but runtime can await). — IP-007/008/009 implementation detail.
- [ ] Decide whether `oya-foundry-supervisor-conformance` lives under `crates/` (as a regular crate) or `tools/` (as a runner) — affects whether it counts toward `cargo deny` external-dep accounting.
- [ ] Confirm the existing `tools/oya-dashboard-e2e` framework can host the negative-test for write-method rejection on the supervisor read endpoints (P02 already has this for visibility; supervisor inherits).
- [ ] Decide whether `oya-foundry-supervisor-adapter-cron-tokio`’s hand-rolled cron parser belongs in its own kernel (`oya-foundry-cron-kernel`) for re-use across Foundry, or stays inlined.
- [ ] Confirm the M02 acceptance gate’s “3 × 2 = 6 cells live-smoke” clause counts subscription-mode autonomy *via supervisor* as satisfying the subscription cell, or whether the supervisor is additive (likely additive, but needs explicit confirmation).

---

> **Status:** `pending approval`. **Next:** Architect review → Critic review → revised plan with ADR finalized.
