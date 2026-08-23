---
doc_class: Standard
purpose: "Application-layer security: signals, graceful shutdown, audit conformance"
owner_team: axis-foundry
status: draft
doc_status: published
---

# Supervisor App — Security

## Signal Safety

The daemon must handle POSIX signals safely. Per v6 BLOCKER-4:

### SIGTERM (Graceful Shutdown)

```
Received SIGTERM
  │
  ├─ Set atomic flag: shutdown_requested = true
  ├─ Stop accepting new inbox messages
  ├─ Wait for in-flight sessions ≤ 5 min
  ├─ Flush outbox: ensure all SpendRecords are fsync'd
  ├─ Emit audit event: "foundry_supervisor_shutdown_graceful"
  └─ exit(0) with success code
```

**Handler (async-safe):**
```rust
#[tokio::main]
async fn main() {
    let shutdown_flag = Arc::new(AtomicBool::new(false));
    let shutdown_clone = shutdown_flag.clone();
    
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        shutdown_clone.store(true, Ordering::Relaxed);
    });
    
    run_daemon(&config, shutdown_flag).await
}
```

### SIGKILL (Emergency Exit)

The daemon **does not** handle SIGKILL. It is sent by the watchdog to **spawned sessions**, not to the supervisor itself.

```
Watchdog (supervisor-app)
  │
  └─ SIGKILL → spawned CLI session (e.g., claude)
                (NOT the supervisor daemon)
```

## Audit Conformance (v6 BLOCKER-3)

Every state transition emits an audit event per ADR-0003:

### Required Audit Events

| Event | When | Fields |
|-------|------|--------|
| `foundry_supervisor_spawn` | Session spawned successfully | account_id, message_id, provider_family |
| `foundry_supervisor_degrade_account` | Usage ceiling exceeded | account_id, autonomy_tier, tokens_consumed |
| `foundry_supervisor_quarantine` | Account moved to Locked tier | account_id, reason |
| `foundry_supervisor_rotate_window` | Usage window rolls over | account_id, old_window_id, new_window_id |
| `foundry_supervisor_settings_drift_exclude` | Account excluded due to drift | account_id, drifted_files |
| `foundry_supervisor_session_killed` | Watchdog SIGKILL'd session | account_id, message_id, duration_secs |
| `foundry_supervisor_shutdown_graceful` | Daemon graceful shutdown | in_flight_count, uptime_secs |

### Event Format

Per ADR-0003, every event includes:

```json
{
  "event_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",  // ULID
  "timestamp_iso": "2026-05-15T10:30:45Z",
  "event_class": "foundry_supervisor_spawn",
  "principal": "account-uuid",
  "capability": "foundry.supervisor.tick_once",
  "autonomy_tier_at_decision": "L2",
  "data_classes_touched": ["TENANT_SCOPED"],
  "regulatory_packs_consumed": [],
  "payload": {
    "message_id": "msg-abc123",
    "provider_family": "Claude",
    "tokens_consumed": { "input": 15000, "output": 5000 }
  }
}
```

### Audit Chain Replay

To verify audit chain integrity:

```bash
# Replay and verify hashes
cargo run -p dev-cli -- audit verify-chain \
  --input evidence/audit-chain.jsonl \
  --expected-events 'foundry_supervisor_*'

# Count completeness
jq -r '.event_class' evidence/audit-chain.jsonl | \
  grep foundry_supervisor | \
  sort | uniq -c | sort -rn
```

**Acceptance:** Every `tick_once()` call produces exactly 1 event (spawn, degrade, quarantine, idle).

## Observability as Security

Per v6 BLOCKER-4, structured logs and OTel spans serve as security audit trail:

### Structured Logs

Every critical action is logged JSON:

```json
{
  "timestamp": "2026-05-15T10:30:45.123Z",
  "level": "info",
  "event": "foundry.supervisor.tick_outcome",
  "outcome": "Spawned",
  "account_id": "acct-xyz",
  "message_id": "msg-abc",
  "input_tokens": 15000,
  "output_tokens": 5000,
  "autonomy_tier": "L2",
  "duration_micros": 45000
}
```

**No secrets in logs.** All `sref://` references are logged as-is; resolved values are never printed.

### OTel Spans

Per ADR-0042, spans include context for root-cause analysis:

```
Span: foundry.supervisor.tick
├─ gen_ai.system: "claude"
├─ gen_ai.request.model: "claude-3-opus"
├─ gen_ai.usage.input_tokens: 15000
├─ gen_ai.usage.output_tokens: 5000
├─ oyatie.foundry.capability: "foundry.supervisor.tick_once"
├─ oyatie.tenant.id: "tenant-uuid"
└─ trace_id: "4bf92f3577b34da6a3ce929d0e0e4736"
```

## Request ID Idempotency

Every request carries an opaque `RequestId` (string, no format requirements). Duplicates within 24 hours are rejected:

```
First request (request_id=abc)
  → spawns session, records spend
  → returns TickOutcome::Spawned
  ↓
Second request (request_id=abc)
  → cache hit
  → returns prior TickOutcome::Spawned (no session spawned)
  → audited as "foundry_supervisor_idempotent_replay"
```

## References

- **ADR-0003:** Audit chain + evidence emission
- **ADR-0042:** OTel semconv (gen_ai)
- **Signals:** Linux man pages: signal(7), sigterm(7), sigkill(7)
- **v6 Amendments § BLOCKER-3:** Audit-chain conformance
- **v6 Amendments § BLOCKER-4:** Observability + OTel spans
