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
title: "Foundry Supervisor \u2014 minimal hook + inbox/outbox driver, multi-account\
  \ \xD7 multi-provider"
reviewers:
- architect
- critic
supersedes: ralplan-foundry-subscription-autonomy-supervisor-2026-05-14.md
length_cap: 220
purpose: Auto-backfilled purpose for ralplan-foundry-supervisor-simple-2026-05-14.md
---
# RALPLAN — Foundry Supervisor (simple hook + inbox/outbox)

> **Premise.** M02 already shipped the kernels we used to plan around: account FSM, OpenBao secret port, 3 CLI adapters, 6 provider cells, route-policy, usage-window, provider-pool, capability-registry, autonomy-ceiling, evidence/run/step, dashboard, hyper transports. The supervisor is therefore a **driver**, not a new heavy subsystem. We add **three small crates** and **two surface expansions** — nothing else. Multi-account fan-out and multi-provider failover come from the *existing* kernels, not from new policy code in the supervisor.
>
> Reference shape: `Siigari/claude-heartbeat` (stop-hook + JSONL inbox/outbox + restart-per-message + idle heartbeat tick), reimplemented in Rust, composed over Foundry M02.

---

## A. RALPLAN-DR Summary

### Principles (4)
1. **Supervisor is a driver, not a kernel.** All routing, budgeting, and policy stay in `route-policy-kernel` + `usage-window-kernel` + `provider-pool-kernel`. The supervisor only spawns processes, moves bytes, and emits audit rows.
2. **Fresh subprocess per message ≡ statelessness.** Restart-per-message is the contract; idle heartbeat is a measured 25-token tick. The supervisor owns no per-session state in memory beyond `SessionTicket` + cursor.
3. **Multi-account × multi-provider parity by type.** Every inbox item is bound to a `SessionTicket { account_id, provider_id, autonomy_tier, usage_window_ref, cost_ceiling_ref, route_policy_ref }` — drivers cannot diverge because the type carries the routing decision.
4. **Zero net-new external Cargo deps.** Std + `tokio` (workspace baseline) + Oya-owned crates only. Any exception requires explicit ADR + measured benchmark. Hyper stays the HTTP backbone.

### Decision Drivers (top 3)
1. **Multi-account fan-out × cost ceiling.** Picking which subscription to spend on must remain a single, audited decision per message (today in `route-policy-kernel` + `usage-window-kernel`). Any duplication in the supervisor is a freelance violation.
2. **Cross-CLI hook parity.** Claude Code ships a native stop-hook; Codex CLI and Gemini CLI surfaces are non-uniform. We must classify this honestly as a RISK rather than invent capabilities.
3. **Operational simplicity.** The user explicitly cancelled the 11-crate plan. Anything beyond 3 new crates is rejected; complexity has to be earned by a measured benchmark, not by speculation.

### Viable Options
**Option A — Single supervisor daemon, pluggable `SessionDriver` (RECOMMENDED).**
+ Pros: one process tree to operate; one audit pipeline; one capability-registry namespace; one cursor file format; cross-CLI parity enforced by trait.
+ Pros: matches Foundry topology — daemon is the read/exec plane analog of `dashboard-app`.
- Cons: a single supervisor crash temporarily idles all drivers (mitigated by per-message restart and `dashboard-dry-run-kernel` standby projection).

**Option B — Per-CLI supervisor binary sharing the kernel.**
+ Pros: blast-radius isolation per CLI; per-binary deploy cadence.
- Cons: 3× the operational surface (3 systemd units, 3 inbox roots, 3 cursors); duplicates routing/usage call chain; violates "one decision per message".
- Cons: makes the multi-account fan-out plumbing harder, not easier.

**Option C — In-process driver inside `oya-intelligence-dashboard-app`.**
+ Pros: zero new binary; reuses dashboard projection.
- Cons: dashboard is the *read* plane; embedding `fork+exec` violates inward-only flow and breaks the 12-layer cohesion lane.
- Cons: any subprocess hang takes the dashboard read API with it.

**Picked: Option A.** It is the only shape that (i) keeps routing in one place, (ii) keeps the supervisor as a driver, (iii) survives the cohesion lane, and (iv) is operable as a single systemd unit.

### Pre-mortem (3 scenarios — deliberate mode)
- **(a) Inbox JSONL torn-write across producers.** Two webhook handlers append concurrently; reader sees a partial frame; cursor advances past garbage. **Mitigation:** single-writer cursor enforced by `OutboxSink` impl; producers stage into `io/inbox/incoming/<uuid>.json`, then `rename(2)` into `io/inbox/queued/`; reader is the only process that touches `io/.inbox-offset`; `fsync(file)` + `fsync(parent_dir)` after every commit. Atomic-rename invariants are documented in `ADR-supervisor-jsonl-durability`.
- **(b) Hung CLI subprocess burns subscription credits.** Driver injects a message, child hangs in network I/O, parent never sees `drain_response`. **Mitigation:** `WATCHDOG_TIMEOUT` (default 90s, configurable); on expiry, parent SIGTERM then SIGKILL after `kill_grace_seconds`; reconcile real spend via `usage_window_kernel::observed_spend()`; emit `SupervisorEvent::WatchdogKilled` to evidence chain; `autonomy_ceiling` demotion on three consecutive kills.
- **(c) Poison-message restart loop.** A specific inbox row causes the child to crash on inject. **Mitigation:** per-message retry budget = 2; on the 3rd attempt, push to `io/outbox/dead-letter/<msg-id>.json`, emit `SupervisorEvent::DeadLettered`, advance cursor, demote `autonomy_tier` for the originating ticket if the failure rate window > threshold.

### Expanded Test Plan (deliberate mode)
- **Unit (kernel)** — trait conformance for `SessionDriver`, `InboxSource`, `OutboxSink`; `SessionTicket` serde round-trip; `HeartbeatPolicy` boundary cases (idle 0, idle ≥ window, kill_grace ≥ watchdog).
- **Integration (jsonl adapter ↔ kernel)** — single-writer cursor invariants, crash mid-fsync recovery (cursor never advances past unflushed data), atomic-rename under concurrent producers.
- **E2E matrix** — 3 CLIs × ≥ 2 accounts × {API-key mode, subscription mode} × ≥ 1 message each; verify no silent-switch, no double-spend, audit chain complete.
- **Observability** — `dashboard-kernel` projects `session`, `inbox_depth`, `outbox_tail`, `idle_ticks_total`, `watchdog_kills_total`, `dead_letters_total`; smoke asserts every row visible read-only and 405 on write.
- **Perf budget** — idle tick **≤ 25 tokens p95**; restart latency **p95 ≤ 1.5 s**; supervisor RSS **≤ 64 MiB** at 200 inbox depth; cursor write **≤ 1 ms p99** on local nvme. Bench harness lives in `crates/oya-intelligence-supervisor-app/benches/`.

---

## B. PRD-style Implementation Plan

### B.1 Crate decomposition (3 new + 2 surface expansions + 3 adapter impls)

Naming-justification (v4 BNF + 12-layer enum):

1. **`oya-intelligence-supervisor-kernel`** — *kernel layer.* Pure types/traits, std + `core` only. BNF: `oya-<product:foundry>-<service:supervisor>-<layer:kernel>`. 12-layer enum: `kernel`. Cohesion: zero I/O, no `tokio`.
2. **`oya-intelligence-supervisor-adapter-jsonl`** — *adapter layer.* Reference file-backed inbox/outbox. BNF: `oya-<product>-<service>-<role:adapter>-<medium:jsonl>`. 12-layer enum: `adapter`. Cohesion: only filesystem I/O + `tokio::fs`.
3. **`oya-intelligence-supervisor-app`** — *app layer.* Daemon entrypoint that composes (a) `supervisor-kernel`, (b) `supervisor-adapter-jsonl`, (c) the 3 CLI driver impls, (d) `route-policy-kernel`, (e) `usage-window-kernel`, (f) `provider-pool-kernel`, (g) `evidence-domain`, (h) `dashboard-kernel` projection. BNF: `oya-<product>-<service>-<role:app>`. 12-layer enum: `app`. Cohesion: composition only — no business logic.

Surface expansions (no new crate):
- `oya-intelligence-route-policy-kernel` — add `select_account_for_message()` returning `SessionTicket` (carries the existing `RouteExplanation` + the new fields).
- `oya-intelligence-usage-window-kernel` — add `try_reserve_for(ticket, projected_tokens) -> Reservation { Reserved | RefuseRetryAfter(Duration) | HardDeny(reason) }` and `observed_spend(ticket, actual_tokens)`.

Adapter impls (no new crate; impl lives in the existing CLI adapter):
- `oya-intelligence-account-adapter-claude-code` — `impl SessionDriver` (native stop-hook).
- `oya-intelligence-account-adapter-codex-cli` — `impl SessionDriver` (stop-hook IF available, else stdout-sentinel + exit-code fallback; documented RISK).
- `oya-intelligence-account-adapter-gemini-cli` — `impl SessionDriver` (same fallback rule; documented RISK).

### B.2 Public contracts (rough compilable shape)

```rust
// oya-intelligence-supervisor-kernel/src/lib.rs

pub use oya_intelligence_account_kernel::{AccountId, ProviderFamily, SessionId};
pub use oya_intelligence_capability_registry_kernel::AutonomyTier;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionTicket {
    pub session_id: SessionId,            // data_class: INTERNAL_ONLY
    pub account_id: AccountId,            // data_class: TENANT_SCOPED
    pub provider_id: ProviderFamily,      // data_class: INTERNAL_ONLY
    pub autonomy_tier: AutonomyTier,      // data_class: INTERNAL_ONLY
    pub usage_window_ref: UsageWindowRef, // data_class: INTERNAL_ONLY
    pub cost_ceiling_ref: CostCeilingRef, // data_class: INTERNAL_ONLY (oya-cloud-billing-kernel)
    pub route_policy_ref: RoutePolicyRef, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InboxItem {
    pub message_id: MessageId,            // data_class: INTERNAL_ONLY
    pub ticket: SessionTicket,            // routing decision baked in
    pub body_ref: BodyRef,                // sref:// or fs:// — NEVER raw secret
}

pub trait InboxSource {
    type Stream: futures_core::Stream<Item = InboxItem> + Send + 'static;
    fn stream(&self) -> Self::Stream;
    fn commit(&self, msg: MessageId) -> Result<(), SupervisorIoError>; // advances cursor
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxRecord {
    pub message_id: MessageId,
    pub ticket: SessionTicket,
    pub body_ref: BodyRef,
    pub tokens_observed: u64,
    pub emitted_at_unix_ms: u64,
    pub kind: OutboxKind, // Response | DeadLetter | WatchdogReport | IdleTick
}

pub trait OutboxSink {
    fn write(&self, record: OutboxRecord) -> Result<(), SupervisorIoError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HeartbeatPolicy {
    pub idle_tick_seconds: u32,
    pub idle_tick_token_budget: u16,   // hard cap (≤ 25 by default)
    pub watchdog_timeout_seconds: u32,
    pub kill_grace_seconds: u32,
    pub max_retries_per_message: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SupervisorEvent {
    Spawned       { ticket: SessionTicket, pid: u32 },
    Injected      { ticket: SessionTicket, msg: MessageId },
    Drained       { ticket: SessionTicket, msg: MessageId, tokens: u64 },
    IdleTick      { ticket: SessionTicket, tokens: u16 },
    WatchdogKilled{ ticket: SessionTicket, msg: MessageId, reason: String },
    DeadLettered  { ticket: SessionTicket, msg: MessageId, attempts: u8 },
    HealthFailed  { ticket: SessionTicket, reason: String },
}

#[async_trait::async_trait]                 // already in workspace via existing crates
pub trait SessionDriver: Send + Sync + 'static {
    fn family(&self) -> ProviderFamily;
    async fn spawn(&self, ticket: &SessionTicket) -> Result<SessionHandle, DriverError>;
    async fn inject_message(&self, handle: &mut SessionHandle, body: &BodyRef) -> Result<(), DriverError>;
    async fn drain_response(&self, handle: &mut SessionHandle) -> Result<DrainedResponse, DriverError>;
    async fn idle_tick(&self, handle: &mut SessionHandle) -> Result<IdleTickReport, DriverError>;
    async fn kill(&self, handle: SessionHandle) -> Result<(), DriverError>;
    async fn health_check(&self, handle: &SessionHandle) -> Result<HealthReport, DriverError>;
}
```

(Note: `async_trait` is **already in workspace** via existing kernel/app crates — verified by `grep -R '^async-trait' crates/*/Cargo.toml`. No net-new dep.)

### B.3 Daemon lifecycle

Process tree:
```
oya-intelligence-supervisor-app (parent, long-lived, systemd-managed)
├── inbox reader task     (single-writer cursor; tokio task)
├── outbox writer task    (single-writer; tokio task)
├── dispatcher task       (consumes InboxItem → SessionDriver)
└── per-message child:  fork+exec(claude|codex|gemini), short-lived, killed after drain
```

Signal flow:
- `SIGTERM` to parent → graceful drain: stop accepting new inbox items, finish in-flight messages up to `drain_deadline_seconds`, then SIGTERM each child, then SIGKILL after `kill_grace_seconds`.
- Watchdog elapsed → parent SIGTERMs child → after `kill_grace_seconds` SIGKILL → `SupervisorEvent::WatchdogKilled` → evidence emit.

Fsync points:
1. After write to `io/inbox/queued/<msg>.json` (producer).
2. After atomic-rename into `io/inbox/queued/`.
3. After cursor advance in `io/.inbox-offset`.
4. After every outbox append.

Crash recovery: cursor file is the source of truth. On boot, reader replays from cursor; any `io/inbox/queued/` entries before cursor are ignored; entries after cursor are re-dispatched. **Restart safety:** the cursor only advances *after* `OutboxSink.write()` returns Ok AND `usage_window_kernel::observed_spend()` returns Ok. Same-row double-spend is impossible because reservation is keyed on `MessageId`.

### B.4 Multi-account × multi-provider routing — exact call chain

```
inbox.next() → InboxItem (NO ticket yet, only msg ref + optional hints)
  │
  ├─→ route_policy_kernel.select_account_for_message(item, constraints, candidates)
  │       returns SessionTicket { account_id, provider_id, autonomy_tier,
  │                               usage_window_ref, cost_ceiling_ref,
  │                               route_policy_ref }
  │
  ├─→ usage_window_kernel.try_reserve_for(ticket, projected_tokens)
  │       Reserved             → continue
  │       RefuseRetryAfter(d)  → outbox.write(OutboxKind::RetryAfter); cursor advance; loop
  │       HardDeny(reason)     → outbox.write(OutboxKind::HardDeny);   cursor advance; audit emit; demotion if rate-exceeded
  │
  ├─→ provider_pool_kernel.acquire_session_slot(ticket)
  │       on driver health-fail → re-enter route_policy with failover_order
  │
  ├─→ SessionDriver::spawn(ticket) → handle
  ├─→ SessionDriver::inject_message(handle, body_ref)
  ├─→ SessionDriver::drain_response(handle) → DrainedResponse { tokens, body_ref_out }
  │       on hang → watchdog fires → SessionDriver::kill(handle); fallthrough to retry/dead-letter
  │
  ├─→ outbox.write(OutboxRecord { Response, tokens_observed, ticket })
  ├─→ run/step/evidence emit (via existing evidence-domain ports)
  ├─→ usage_window_kernel.observed_spend(ticket, tokens_observed)
  └─→ inbox.commit(msg_id)   // cursor advances AFTER all the above
```

Failover semantics (no silent-switch): the `RouteExplanation` from `route_policy_kernel.explain_route()` is persisted into the audit row before the driver runs; if the driver health-fails, we **re-call route_policy_kernel** with `previous_account_id` set — `route-policy-kernel` already prevents reuse of the same subscription, and `check_silent_switch` in `account-domain` is the existing canonical guard.

### B.5 Cross-CLI hook bridge — honest capability map

| CLI         | Native stop-hook | Fallback used               | RISK class |
|-------------|------------------|-----------------------------|------------|
| Claude Code | Yes (CLI ships it)| —                          | Low        |
| Codex CLI   | Unknown (verify) | stdout sentinel + exit-code | **Medium — RISK row** |
| Gemini CLI  | Unknown (verify) | stdout sentinel + exit-code | **Medium — RISK row** |

Acceptance gate: a conformance test crate validates each adapter against the same `SessionDriver` trait test suite. If Codex/Gemini cannot honor `drain_response` cleanly via fallback, we explicitly **downgrade their `autonomy_tier`** in capability-registry (T2 instead of T3) and document it; we do not invent capability.

### B.6 Cost-ceiling integration with `oya-cloud-billing-kernel`

`CostCeilingRef` in `SessionTicket` is the existing billing-kernel reservation handle. Wiring:
1. On `try_reserve_for(ticket, projected_tokens)`, `usage-window-kernel` also calls `oya_cloud_billing_kernel::reserve(cost_ceiling_ref, projected_micros)`. Single transactional contract (already exists).
2. On `observed_spend`, both kernels are updated atomically (existing pattern in `usage-window-kernel`).
3. Runaway-loop kill condition: if `watchdog_kills_total` for a `ticket` ≥ N within window W, the supervisor calls `autonomy_ceiling_app::demote(ticket, reason)` AND emits `SupervisorEvent::HealthFailed` — driver is quarantined until operator clears via dashboard.

### B.7 Hyper integration (no new HTTP stack)

Webhook receiver + cron trigger are **routes registered on the existing `oya-intelligence-api-rest-adapter`** (hyper). The supervisor crate exposes a `RestRoutes` value (path + handler closure list) that `dashboard-app` mounts at boot. No axum, no tower-http, no warp — this is enforced by `fitness-banned-primitives-kernel`.

Routes added:
- `POST /v1/supervisor/inbox` — enqueue (auth via existing Cedar policy on `foundry.supervisor.inject_message`).
- `POST /v1/supervisor/tick` — cron-triggered idle tick.
- `GET  /v1/supervisor/sessions` — read-only projection (delegates to `dashboard-kernel`).
- `GET  /v1/supervisor/health` — liveness/readiness.

### B.8 Capability-registry rows (`foundry.supervisor.*`)

| Row                                | Default tier | Cedar policy   | Notes |
|------------------------------------|--------------|----------------|-------|
| `foundry.supervisor.inject_message`| T2           | `inject.cedar` | Webhook write |
| `foundry.supervisor.idle_tick`     | T1           | `idle.cedar`   | Heartbeat only |
| `foundry.supervisor.restart_session`| T3          | `restart.cedar`| Watchdog action |
| `foundry.supervisor.dead_letter`   | T3           | `deadletter.cedar` | Quarantine |
| `foundry.supervisor.dashboard_read`| T1           | `read.cedar`   | Read-only projection |

All rows registered via existing `oya-intelligence-capability-registry-app` at boot — supervisor does **not** invent a registration mechanism.

### B.9 Phase plan — M02 fan-in (NOT a new phase)

This lands as a thin lane attached to existing M02 phases:

| IP   | Phase       | Title                                                          | Days |
|------|-------------|----------------------------------------------------------------|------|
| IP-A | M02-P01     | `supervisor-kernel` traits + types + ADRs + naming-justification | 2 |
| IP-B | M02-P01     | `supervisor-adapter-jsonl` (cursor + atomic-rename + fsync inv.) | 2 |
| IP-C | M02-P01     | `route-policy-kernel` + `usage-window-kernel` surface expansion  | 1 |
| IP-D | M02-P01     | `SessionDriver` impls in 3 CLI adapters (+ conformance crate)   | 3 |
| IP-E | M02-P02     | `supervisor-app` daemon + hyper webhook routes + dashboard projection | 3 |
| IP-F | M02-P05     | capability-registry rows + Cedar policies + e2e matrix + perf bench | 2 |

Total: **6 IPs, each ≤ 3 days single-agent.** Sequencing in §B.12.

### B.10 Doc + ADR footprint

ADRs:
- `ADR-00XX-supervisor-as-driver-not-kernel.md` — why the supervisor is an app/driver and policy stays in the existing kernels.
- `ADR-00YY-restart-per-message-statelessness-contract.md` — measured restart latency budget + idle-tick token budget.
- `ADR-00ZZ-cross-CLI-hook-bridge-with-stdout-fallback.md` — honest capability map for Codex/Gemini, autonomy-tier downgrade rule.
- `ADR-00AA-supervisor-jsonl-durability.md` — single-writer cursor, atomic-rename, fsync invariants, crash recovery.

Doc-coverage (ADR-0063 mandates the full suite per crate):
- `crates/oya-intelligence-supervisor-kernel/docs/{overview,prd,api,operations,security,observability,compatibility}.md`
- Same suite for `-adapter-jsonl` and `-app`.
- `docs/foundry/supervisor/overview.md` — cross-crate operator narrative.

### B.11 Verification gates

Build:
```
rtk cargo build -p oya-intelligence-supervisor-kernel
rtk cargo build -p oya-intelligence-supervisor-adapter-jsonl
rtk cargo build -p oya-intelligence-supervisor-app
```

Conformance:
```
rtk cargo test -p oya-intelligence-supervisor-conformance --features claude,codex,gemini
```

E2E live-smoke matrix (3 CLI × ≥ 2 accounts × {api, subscription} × ≥ 1 msg):
```
rtk cargo run -p oya-intelligence-supervisor-app -- e2e-matrix --config tests/e2e/supervisor.toml
```

Bench harness (acceptance numbers):
```
rtk cargo bench -p oya-intelligence-supervisor-app -- idle_tick restart_latency rss_at_depth_200
```

Acceptance: idle tick **≤ 25 tokens p95**, restart latency **p95 ≤ 1.5 s**, RSS **≤ 64 MiB** at 200 inbox depth.

CI lanes that MUST be green:
- `lean-a5-doc-coverage`
- `lean-a10-public-contract`
- `fitness-banned-primitives-kernel` (no axum/warp/tower-http; no new external deps)
- `fitness-supply-chain-kernel` (Cargo.lock diff inspection)
- `fitness-claim-ceiling-kernel`
- `fitness-pre-push-kernel`
- `fitness-quality-lane-kernel`
- `fitness-cohesion-kernel`

### B.12 Sequencing (claim units for grit)

Merge order (each row = one `grit claim --intent ... <file::Identifier>`):

1. `crates/oya-intelligence-supervisor-kernel/src/lib.rs::SessionTicket`
2. `crates/oya-intelligence-supervisor-kernel/src/lib.rs::SessionDriver`
3. `crates/oya-intelligence-supervisor-kernel/src/lib.rs::InboxSource`
4. `crates/oya-intelligence-supervisor-kernel/src/lib.rs::OutboxSink`
5. `crates/oya-intelligence-supervisor-kernel/src/lib.rs::HeartbeatPolicy`
6. `crates/oya-intelligence-supervisor-adapter-jsonl/src/lib.rs::JsonlInbox`
7. `crates/oya-intelligence-supervisor-adapter-jsonl/src/lib.rs::JsonlOutbox`
8. `crates/oya-intelligence-route-policy-kernel/src/lib.rs::select_account_for_message`
9. `crates/oya-intelligence-usage-window-kernel/src/lib.rs::try_reserve_for`
10. `crates/oya-intelligence-usage-window-kernel/src/lib.rs::observed_spend`
11. `crates/oya-intelligence-account-adapter-claude-code/src/lib.rs::ClaudeCodeSessionDriver`
12. `crates/oya-intelligence-account-adapter-codex-cli/src/lib.rs::CodexCliSessionDriver`
13. `crates/oya-intelligence-account-adapter-gemini-cli/src/lib.rs::GeminiCliSessionDriver`
14. `crates/oya-intelligence-supervisor-app/src/lib.rs::SupervisorDaemon`
15. `crates/oya-intelligence-supervisor-app/src/lib.rs::rest_routes`
16. `crates/oya-intelligence-capability-registry-app/src/lib.rs::register_supervisor_rows`

Each claim unit lands its own ADR/doc deltas where applicable. `grit done --agent <id>` is the merge primitive per ADR-0054.

---

## C. Acceptance Bar (every item must be true to ship)

1. **Three new crates green** under `rtk cargo build`; **no new external Cargo dep** (verified by `fitness-banned-primitives-kernel` + `fitness-supply-chain-kernel`).
2. **Conformance test suite** passes for all 3 CLI `SessionDriver` impls, OR Codex/Gemini explicitly documented as RISK and Cedar tier downgraded.
3. **Multi-account fan-out**: live-smoke completes ≥ 2 accounts × ≥ 2 providers × ≥ 1 message each, with `RouteExplanation` rows in the audit chain showing distinct decisions; `check_silent_switch` green.
4. **Usage-window deny path**: at the (N+1)th message that would breach `reserve_remaining_pct`, `usage_window_kernel` returns `HardDeny`; outbox shows `HardDeny` row; audit run/step emitted; capability tier demotion fired when failure rate threshold crossed.
5. **Perf**: idle tick ≤ 25 tokens p95, restart latency p95 ≤ 1.5 s, RSS ≤ 64 MiB at inbox depth 200 — all in committed bench output.
6. **Hyper-only**: webhook + cron served by `oya-intelligence-api-rest-adapter`; no new HTTP crate; `fitness-banned-primitives-kernel` lane green.
7. **Doc + ADR + naming-justification**: 4 ADRs landed; doc-suite present for all 3 new crates; lean-a5 + lean-a10 + 7 fitness lanes green.
8. **Dashboard projection**: new rows (`session`, `inbox_depth`, `outbox_tail`, `idle_ticks_total`, `watchdog_kills_total`, `dead_letters_total`) visible read-only; **405** on any write.
9. **No raw secrets in inbox/outbox**: only `SecretReference` (`sref://...`) traversed; `account-domain` silent-switch detection assertion green in conformance suite.
10. **Restart-per-message semantics measured, not asserted**: the bench output in §B.11 is committed to the repo.

---

## D. ADR — Decision record

- **Decision.** Build the Foundry supervisor as a **driver** composed of three new crates (`supervisor-kernel`, `supervisor-adapter-jsonl`, `supervisor-app`) plus two surface expansions on the existing routing/usage kernels and three trait impls on the existing CLI adapters. Multi-account × multi-provider behavior is delegated to the kernels already shipped in M02.
- **Drivers.** (1) Multi-account fan-out × cost ceiling must remain one audited decision per message. (2) Cross-CLI hook parity must be enforced by type, not by replicated logic. (3) The user explicitly cancelled the 11-crate plan; simplicity is non-negotiable.
- **Alternatives considered.**
  - Option B (per-CLI binary): rejected — triples operational surface, duplicates routing call chain.
  - Option C (embed in dashboard-app): rejected — violates inward-only flow; subprocess hangs would take the read API down.
- **Why chosen.** Option A is the only shape that satisfies (i) "driver not kernel", (ii) cross-CLI parity via `SessionTicket`, (iii) cohesion lane green, (iv) one systemd unit.
- **Consequences.**
  - Positive: single audit pipeline, single capability-registry namespace, one cursor format, measurable perf budget.
  - Negative: supervisor crash idles all drivers temporarily — mitigated by per-message restart + dashboard standby projection.
  - Operational: adds one systemd unit + one inbox directory hierarchy + one cursor file per deploy.
- **Follow-ups.**
  - Verify Codex CLI stop-hook capability before claiming IP-D done; if absent, ratify ADR-00ZZ fallback + autonomy-tier demotion.
  - Verify Gemini CLI stop-hook capability under same gate.
  - Run perf bench on local nvme AND on the target deploy disk; commit both numbers.
  - After 30 days of production telemetry, revisit the 25-token idle-tick budget against observed token cost.

---

## E. Notes for Architect + Critic

- The supersedes target (`ralplan-foundry-subscription-autonomy-supervisor-2026-05-14.md`) proposed **11 crates** and a parallel routing/usage subsystem. This plan reduces to **3 new crates + 2 surface expansions + 3 trait impls**, and routes everything through the kernels already in M02. Please reject any review feedback that re-grows the surface area without a measured benchmark justifying the addition.
- The honest-capability stance on Codex/Gemini stop-hooks is load-bearing. If a reviewer asserts "just add a stop-hook for Codex", that requires a verified citation to the Codex CLI release notes; otherwise we ship the stdout-sentinel fallback and the autonomy-tier downgrade.
