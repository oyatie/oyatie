---
doc_class: Standard
purpose: "12-layer placement, inward-only dependency flow, and port trait locations"
owner_team: axis-foundry
status: draft
doc_status: published
---

# Supervisor Kernel — Architecture

## 12-Layer Placement

```
L1: Kernel          ← oya-intelligence-supervisor-kernel (THIS CRATE)
    ├─ SessionTicket (value type)
    ├─ InboxState (enum)
    ├─ SpendRecord (value type)
    ├─ TickOutcome (enum)
    ├─ SupervisorConfig (value type)
    ├─ AccountSnapshotProvider (port trait)
    ├─ InboxStore (port trait)
    ├─ OutboxSink (port trait)
    └─ SessionDriver (port trait)

L2-L3: (reserved for future kernel extensions)

L4: Adapter         ← intelligence-jsonl-supervisor-adapter
    ├─ JsonlInboxStore (impl InboxStore)
    ├─ JsonlOutboxSink (impl OutboxSink)
    └─ (dead-letter, peek_lock TTL, fsync)

L4: Adapter         ← intelligence-settings-template-adapter
    ├─ ClaudeRenderer (impl SettingsRenderer)
    ├─ CodexRenderer (impl SettingsRenderer)
    └─ GeminiRenderer (impl SettingsRenderer)

L5: Application     ← oya-intelligence-supervisor-app
    ├─ SupervisorApp (daemon)
    ├─ tick_once() (call chain)
    ├─ hyper webhook (MiddlewareChain)
    └─ watchdog loop (SIGKILL)
```

## Dependency Flow (Inward-Only)

```
oya-intelligence-supervisor-app
  │
  ├─→ oya-intelligence-supervisor-kernel (port trait use)
  ├─→ intelligence-jsonl-supervisor-adapter (impl InboxStore)
  ├─→ intelligence-settings-template-adapter (impl SettingsRenderer)
  │
  └─→ oya-intelligence-account-adapter-* (SessionDriver impls)

intelligence-jsonl-supervisor-adapter
  └─→ oya-intelligence-supervisor-kernel (port trait def)

intelligence-settings-template-adapter
  └─→ oya-intelligence-settings-template-kernel (value types)

oya-intelligence-supervisor-kernel
  └─→ (no supervisor deps — std only)
```

**Rule:** Adapter layers never call each other; all inter-adapter flow goes through the Application layer's composition shims.

## Port Locations

### Inbound Ports (Implemented by Adapters)

| Port | Layer | Implementor | Purpose |
|------|-------|-------------|---------|
| `InboxStore` | L4 Adapter | jsonl-supervisor-adapter | Read messages from file-backed inbox |
| `OutboxSink` | L4 Adapter | jsonl-supervisor-adapter | Write spend records to file-backed outbox |
| `SessionDriver` | L4 Adapter | account-adapter-{claude,codex,gemini} | Spawn/inject/drain CLI sessions |
| `SettingsRenderer` | L4 Adapter | settings-template-adapter | Render/verify provider-specific settings |

### Outbound Ports (Used by Kernel)

| Port | Consumed by | Purpose |
|------|-------------|---------|
| `AccountSnapshotProvider` | tick_once() | Query eligible accounts + settings drift state |
| `RoutePolicy` | tick_once() step 3 | Select account for this message |
| `UsageEnforcement` | tick_once() step 7 | Check autonomy ceiling |

## Data Structure Invariants

### `SessionTicket` — Transport Invariant

**Guarantee:** A `SessionTicket` can be moved across async boundaries (tasks, blocking pools) without serialization or allocation.

```rust
// Safe:
let ticket = inbox.peek_lock(60).await?;
tokio::task::spawn_blocking(move || {
    driver.spawn_for_message(&ticket).await?  // moved; no Arc needed
}).await?;
```

**Constraint:** value-only; no `&`/`Arc`/`Box<dyn>` in fields.

### `SpendRecord` — Durability Invariant

**Guarantee:** Once `outbox.append(&record)` returns `Ok(())`, the record is fsync'd to disk.

```
Driver writes session output  →  parse tokens  →  SpendRecord  →  fsync'd
                                                                    (OutboxSink)
```

### `TickOutcome` — Exhaustive Routing

Every tick must produce exactly one outcome. Supervisor routes based on the variant:

| Variant | Next action |
|---------|------------|
| `Spawned(msg_id)` | Log; continue to next tick |
| `Saturated` | Back-pressure; wait before next tick |
| `Idle` | Sleep; poll inbox periodically |
| `Quarantined(msg_id)` | Alert ops; exclude account temporarily |
| `DriftExcluded{…}` | Alert; trigger settings reconciliation (if auto_reconcile=true) |

## Trait Composition

### `RoutePolicy` Shim

```rust
// Kernel defines the port; RoutePolicy wraps the implementation:
impl RoutePolicy {
    pub fn select(&self, eligible: &[ProviderAccount]) 
        -> Result<AccountId, NoEligibleAccount>
    {
        // Delegates to oya-intelligence-route-policy-kernel
        // Per ADR-0055: multi-policy composition (round-robin, least-loaded, …)
    }
}
```

### `UsageEnforcement` Shim

```rust
// Similarly, wraps oya-intelligence-usage-window-kernel:
impl UsageEnforcement {
    pub fn check_limit(&self, ticket: &SessionTicket, spend: &SpendRecord)
        -> Result<(), OverLimit>
    {
        // Enforces Cedar autonomy_tier ceiling
        // Consults live UsageWindow (account-domain); NOT the snapshot
    }
}
```

## References

- **Design:** `docs/DESIGN.md` §10 (foundry axis contracts)
- **ADR-0056:** 12-layer enum + port-in-kernel
- **v4 Plan:** `ralplan-foundry-supervisor-simple-v4-2026-05-14.md` §B.1..B.5
