---
doc_class: Standard
purpose: "Overview and public API for the supervisor application layer daemon"
owner_team: axis-foundry
status: draft
doc_status: published
---

# Foundry Supervisor App — README

**Crate:** `intelligence-supervisor-app`  
**Layer:** Application (12-layer-enum L5)  
**Wave:** 2d (M02-P06)  
**Entry point:** `bin/intelligence-supervisor` (daemon binary)

## Overview

The supervisor app is the daemon that orchestrates multi-account, multi-provider session supervision. It composes the kernel's port traits with adapter implementations (JSONL file I/O, settings-template rendering, per-provider CLI drivers) into a single runnable process.

### Purpose

Provide a long-running daemon that:
- **Polls** file-backed inbox (JSONL) for new session requests
- **Routes** each message to an eligible account via `RoutePolicy`
- **Spawns** and **manages** CLI sessions with bounded concurrency (`max_in_flight`)
- **Enforces** usage ceilings per `UsageEnforcement` policy
- **Drains** session output, records spend, and acknowledges messages
- **Exposes** a hyper HTTP webhook for remote session injection
- **Watchdog-kills** hung sessions after a timeout
- **Emits** structured tracing and OTel spans for observability

## Public API Summary

### Daemon Entry Point

```rust
pub async fn run_daemon(config: SupervisorConfig) -> Result<(), SupervisorError>
```

Starts the supervisor daemon with:
- **Config source:** env vars + TOML file (see Configuration below)
- **Logger:** JSON structured logs (tracing subscriber)
- **Signals:** SIGTERM → graceful shutdown; SIGKILL → emergency exit
- **Health check:** HTTP endpoint at `/health` (hyper)

### Main Routine

```rust
pub async fn tick_once(
    config: &SupervisorConfig,
    snapshot_provider: &dyn AccountSnapshotProvider,
    inbox: &dyn InboxStore,
    outbox: &dyn OutboxSink,
    driver: &dyn SessionDriver,
) -> Result<TickOutcome, SupervisorError>
```

The core call chain (v4 §B.4, 17 steps):

1. **Snapshot** — get eligible accounts (includes settings drift check)
2. **Peek-lock** — read next message from inbox
3. **Route** — select account via `RoutePolicy::select()`
4. **Enforce** — check usage ceiling via `UsageEnforcement::check_limit()`
5. **Spawn** — start session on provider CLI
6. **Inject** — send request into session stdin
7. **Drain** — read session output
8. **Parse** — extract token usage from response
9. **Check** — verify autonomy tier ceiling not exceeded
10. **Commit** — ack message in inbox
11. **Record** — append spend record to outbox (fsync'd)
12. **Watchdog** — enforce session timeout via SIGKILL
13. **Audit** — emit event per ADR-0003
14. **Quarantine** — move over-limit accounts to blocked tier
15. **Dead-letter** — move unprocessable messages aside
16. **Emit** — telemetry (structured logs + OTel spans)
17. **Return** — `TickOutcome` for caller

### Configuration

```rust
pub struct SupervisorConfig {
    pub max_in_flight: usize,                    // default: 12
    pub watchdog_timeout_secs: u64,              // default: 300
    pub settings_renderer_mode: RendererMode,    // default: Disabled
    pub settings_verify_debounce_secs: u64,      // default: 60
    pub minimum_eligible_accounts: usize,        // default: 1
}
```

**Source (in order of precedence):**
1. Environment variables: `OYATIE_SUPERVISOR_MAX_IN_FLIGHT=12`
2. TOML config file: `$OYATIE_CONFIG_PATH/supervisor.toml`
3. Compiled defaults

### HTTP Webhook

```rust
pub fn build_router() -> MiddlewareChain
```

Mounts supervisor routes on a hyper router:

| Method | Path | Purpose |
|--------|------|---------|
| `GET` | `/health` | Liveness probe (always 200) |
| `POST` | `/inbox` | Inject message into inbox (returns message_id) |
| `GET` | `/inbox/peek` | Peek next message without locking |
| `GET` | `/metrics` | Prometheus metrics (OTel exposition) |

### Watchdog Loop

The daemon runs an async watchdog that:
- Tracks all spawned sessions by `MessageId`
- After `watchdog_timeout_secs` elapses, sends SIGKILL to hung process
- Logs session death + duration
- Emits `foundry_supervisor_session_killed` audit event

## Usage Example

```bash
# Start daemon with default config:
export OYATIE_SUPERVISOR_MAX_IN_FLIGHT=12
export OYATIE_SUPERVISOR_WATCHDOG_TIMEOUT_SECS=300
intelligence-supervisor

# Or with config file:
intelligence-supervisor --config /etc/oya/supervisor.toml

# Inject message via webhook:
curl -X POST http://localhost:8080/inbox \
  -H 'Content-Type: application/json' \
  -d '{
    "account_id": "acct-uuid",
    "provider_family": "Claude",
    "request_json": "{ ... }"
  }'
```

## Data Flow

```
┌──────────────────────────────────┐
│  run_daemon() async loop         │
│  (tick interval: 100ms default)  │
└────────────────┬─────────────────┘
                 │
     ┌───────────┴────────────┐
     │                        │
     ↓                        ↓
┌─────────────────┐  ┌──────────────┐
│  tick_once()    │  │  HTTP server │
│  (message poll) │  │  (webhook)   │
└─────────────────┘  └──────────────┘
     │
     ├─→ AccountSnapshotProvider
     ├─→ InboxStore::peek_lock()
     ├─→ RoutePolicy::select()
     ├─→ UsageEnforcement::check_limit()
     ├─→ SessionDriver::spawn_for_message()
     ├─→ OutboxSink::append(SpendRecord)
     └─→ emit audit events
```

## Observability

### Structured Logs

Every `tick_once()` outcome is logged:

```json
{
  "event": "foundry.supervisor.tick_outcome",
  "outcome": "Spawned",
  "account_id": "acct-xyz",
  "message_id": "msg-abc",
  "duration_micros": 45000,
  "level": "info"
}
```

Saturation and quarantine escalate to `warn` and `error` levels.

### OTel Spans

Per ADR-0042 (gen_ai semconv):

```rust
let span = tracing::info_span!("foundry.supervisor.tick",
    gen_ai.system = "claude",
    gen_ai.request.model = "claude-3-opus",
    oyatie.foundry.capability = "foundry.supervisor.tick_once",
    oyatie.tenant.id = "tenant-uuid"
);
// Records gen_ai.usage.input_tokens, output_tokens after SpendRecord
```

### Metrics

Per v6 BLOCKER-4:

```
foundry_supervisor_inbox_depth{account_id}
foundry_supervisor_outbox_tail{account_id}
foundry_supervisor_idle_ticks_total
foundry_supervisor_quarantine_total
foundry_supervisor_session_active{provider_family}
foundry_supervisor_settings_drift_excluded_total{provider_family}
```

## References

- **Plan:** `ralplan-foundry-supervisor-simple-v4-2026-05-14.md` §B.4
- **v6 Amendments:** `ralplan-foundry-supervisor-simple-v6-amendments-2026-05-15.md` §BLOCKER-4
- **ADRs:** ADR-0042 (OTel semconv), ADR-0003 (audit), ADR-0024 (autonomy)
- **Kernel API:** `docs/products/foundry/supervisor/supervisor-kernel/README.md`
