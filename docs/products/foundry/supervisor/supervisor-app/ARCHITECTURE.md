---
doc_class: Standard
purpose: "Daemon architecture, 12-layer placement, composition of adapters, and signal handling"
owner_team: axis-foundry
status: draft
doc_status: published
---

# Supervisor App — Architecture

## 12-Layer Placement

```
L1: Kernel           ← intelligence-supervisor-kernel (port traits)

L2-L3: (reserved)

L4: Adapter          ← intelligence-jsonl-supervisor-adapter (InboxStore, OutboxSink)
     ← intelligence-settings-template-adapter (SettingsRenderer)
     ← intelligence-account-adapter-{claude,codex,gemini} (SessionDriver)

L5: Application      ← intelligence-supervisor-app (THIS CRATE)
     ├─ SupervisorApp (daemon orchestrator)
     ├─ tick_once() (call chain, 17 steps)
     ├─ build_router() (hyper webhook)
     ├─ watchdog loop (SIGKILL enforcement)
     └─ signal handler (SIGTERM graceful shutdown)
```

## Composition Pattern

The application layer composes adapters via dependency injection:

```rust
pub struct SupervisorApp {
    inbox: Box<dyn InboxStore>,
    outbox: Box<dyn OutboxSink>,
    drivers: Vec<Box<dyn SessionDriver>>,  // one per provider
    snapshot_provider: Box<dyn AccountSnapshotProvider>,
    usage_enforcement: UsageEnforcement,
    route_policy: RoutePolicy,
    config: SupervisorConfig,
}

impl SupervisorApp {
    pub async fn new(
        inbox: Box<dyn InboxStore>,
        outbox: Box<dyn OutboxSink>,
        drivers: Vec<Box<dyn SessionDriver>>,
        snapshot_provider: Box<dyn AccountSnapshotProvider>,
        config: SupervisorConfig,
    ) -> Result<Self> {
        Ok(Self {
            inbox,
            outbox,
            drivers,
            snapshot_provider,
            usage_enforcement: UsageEnforcement::new(),
            route_policy: RoutePolicy::new(),
            config,
        })
    }
}
```

## Async Architecture

```
┌─────────────────────────────────────┐
│  main() — tokio runtime             │
│  run_daemon(&config)                │
└────────────┬────────────────────────┘
             │
    ┌────────┴──────────────┐
    │                       │
    ↓                       ↓
┌─────────────┐  ┌──────────────────┐
│ Tick loop   │  │ HTTP server      │
│ (100ms)     │  │ (hyper)          │
│ tick_once() │  │ webhook endpoint │
└─────────────┘  └──────────────────┘
```

**Concurrency model:**
- Main tick loop: single-threaded async (tokio-runtime)
- HTTP server: multi-threaded (hyper Acceptor)
- Session watchdog: spawned task per session (tokio::spawn)

**Graceful shutdown:** SIGTERM → flushes pending outbox writes → exits.

## Call Chain (17 Steps)

```rust
pub async fn tick_once(...) -> Result<TickOutcome> {
    // 1. Snapshot + settings drift check
    let accounts = snapshot_provider.snapshot().await?;
    
    // 2. Peek next message
    let ticket = inbox.peek_lock(60).await?;
    
    // 3. Select eligible account
    let account_id = route_policy.select(&accounts)?;
    
    // 4. Lookup account struct
    let account = accounts.iter().find(|a| a.id == account_id)
        .ok_or(NoEligibleAccount)?;
    
    // 5. Prepare ticket
    let ticket = SessionTicket { account_id, ... };
    
    // 6. Enforce usage ceiling
    usage_enforcement.check_limit(&ticket)?;
    
    // 7. Select driver by provider
    let driver = drivers.iter()
        .find(|d| d.provider() == account.provider)?;
    
    // 8. Spawn session
    let outcome = driver.spawn_for_message(&ticket).await?;
    
    // 9. Inject request
    driver.inject_response(&ticket, &request_bytes).await?;
    
    // 10. Drain response
    let response = driver.drain_response(&ticket).await?;
    
    // 11. Parse tokens
    let (input_tokens, output_tokens) = parse_tokens(&response)?;
    
    // 12. Create spend record
    let spend = SpendRecord { input_tokens, output_tokens, ... };
    
    // 13. Commit to inbox
    inbox.commit(&ticket).await?;
    
    // 14. Append to outbox (fsync'd)
    outbox.append(&spend).await?;
    
    // 15. Kill session (watchdog)
    // (spawned as async task with TTL)
    
    // 16. Emit audit event
    emit_audit_event("foundry_supervisor_spawn", &ticket).await?;
    
    // 17. Return outcome
    Ok(TickOutcome::Spawned(ticket.message_id))
}
```

## Signal Handling

```
SIGTERM (graceful shutdown)
  │
  ├─ Stop accepting new messages
  ├─ Wait for in-flight sessions to complete (up to 5 min)
  ├─ Flush outbox (fsync all pending SpendRecords)
  ├─ Emit audit event "foundry_supervisor_shutdown_graceful"
  └─ exit(0)

SIGKILL (from watchdog)
  │
  └─ Process killed; OS cleans up FDs
     (should not happen if watchdog tuning is correct)
```

## Error Handling

```
InboxError (locked, corrupted)
  → log warn; retry next tick

OutboxError (full, permission denied)
  → log error; emit "foundry_supervisor_outbox_error" audit
  → block further message processing until resolved

SessionDriver error (spawn failed, CLI crashed)
  → dead_letter the message
  → emit "foundry_supervisor_session_spawn_failed"
  → increment quarantine counter

RoutePolicy error (no eligible accounts)
  → return Idle (no work this tick)
  → may emit "foundry_supervisor_no_eligible_accounts" audit
```

## References

- **Kernel:** `docs/products/foundry/supervisor/supervisor-kernel/ARCHITECTURE.md`
- **v4 Plan § B.4:** Call chain (17 steps)
- **Signal handling:** Standard POSIX signal semantics per Linux Programmer's Manual
