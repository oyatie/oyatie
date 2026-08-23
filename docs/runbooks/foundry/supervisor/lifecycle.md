---
doc_status: published
---

# RB-SUPERVISOR-001: Foundry Supervisor Lifecycle Management

> **Status:** Active supervisor lifecycle procedure
>
> **Severity scope:** Sev 2
>
> **Last verified:** 2026-05-15 (reviewed during M02 exit-readiness gate sweep)

## 1. Context
The Foundry Supervisor is a long-lived daemon that orchestrates sessions across multiple provider accounts.

## 2. Procedures

### 2.1 Starting the Supervisor
```bash
# Production
./target/release/intelligence-supervisor

# Development
cargo run -p intelligence-supervisor-app
```

### 2.2 Graceful Shutdown
Send `SIGTERM` or `SIGINT`. The supervisor will wait for in-flight sessions up to `watchdog_secs` before exiting.
```bash
kill -TERM <pid>
```

### 2.3 Handling "Drift Blackhole"
If the supervisor logs `TickOutcome::DriftExcluded`, it means too many accounts have drifted from their canonical templates.
1. Inspect `registry/accounts/*.toml` for new/modified entries.
2. Check `.omc/supervisor/logs` for specific drift hashes.
3. If valid, set `settings_renderer_mode = "Reconcile"` in `SupervisorConfig` (via env or config file) and restart to auto-reconcile.

### 2.4 Dead-Letter Recovery
Messages that fail repeatedly or reach usage limits move to `.omc/supervisor/dead-letter/*.json`.
1. Inspect the reason field in the JSON.
2. Fix underlying account/budget issue.
3. Move JSON back to `inbox.jsonl` to retry (requires re-formatting to line-delimited).
