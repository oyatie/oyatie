---
doc_class: Standard
purpose: "Overview and public API reference for the foundry supervisor kernel layer"
owner_team: axis-foundry
status: draft
doc_status: published
---

# Foundry Supervisor Kernel — README

**Crate:** `intelligence-supervisor-kernel`  
**Layer:** Kernel (12-layer-enum L1)  
**Wave:** 2b (M02-P06)  
**Dependencies:** std-only (Branch Y)

## Overview

The supervisor kernel is the foundational layer of the Foundry supervisor lane. It defines all port traits that cross crate boundaries and houses pure value types for session lifecycle management. No I/O, no external dependencies, no provider-specific code.

### Purpose

Enable multi-account, multi-provider session supervision with:
- **Session lifecycle tracking** via `SessionTicket` value-only type
- **Message inbox/outbox abstraction** via `InboxStore` and `OutboxSink` port traits
- **Account-eligibility routing** via `AccountSnapshotProvider` and `RoutePolicy` composition
- **Atomic failure semantics** via dead-letter and quarantine contracts
- **Autonomy-tier enforcement** via `UsageEnforcement` composition shim

## Public API Summary

### Value Types

#### `SessionTicket`
```rust
pub struct SessionTicket {
    pub account_id: AccountId,                    // tenant-scoped account
    pub provider_family: ProviderFamily,           // Claude / Codex / Gemini / AWS / OCI
    pub autonomy_tier: AutonomyTier,              // Cedar-enforced usage ceiling
    pub usage_window_snapshot: UsageWindowSnapshot,  // immutable snapshot at tick start
    pub message_id: MessageId,                    // ULID, idempotency key
    pub request_id: RequestId,                    // opaque idempotency token
}
```
**Invariant:** value-only, no refs, no Arc. Safe to move. Audit-transportable across blocking-pool boundaries.

#### `InboxState`
```rust
pub enum InboxState {
    Locked(MessageId),      // peek_lock granted; TTL active
    Unlocked,               // ready for next pick_lock
    DeadLettered,           // moved to dead-letter/ on terminal error
}
```

#### `SpendRecord`
```rust
pub struct SpendRecord {
    pub account_id: AccountId,
    pub message_id: MessageId,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost_microdollars: u64,
    pub timestamp_epoch_micros: u64,
}
```
**Data class:** `TENANT_SCOPED`. Bridges to tenant usage accounting via `spend_to_usage_record` flow.

#### `TickOutcome`
```rust
pub enum TickOutcome {
    Spawned(MessageId),      // session spawned; supervisor accepted the message
    Saturated,               // inbox has messages; max_in_flight reached
    Idle,                    // inbox empty; no work this tick
    Quarantined(MessageId),  // account exceeded usage ceiling; session killed
    DriftExcluded {          // settings drift detected; account excluded from routing
        excluded_accounts: Vec<AccountId>,
        eligible_count: usize,
    },
}
```

#### `SupervisorConfig`
```rust
pub struct SupervisorConfig {
    pub max_in_flight: usize,                    // max concurrent sessions per tick
    pub watchdog_timeout_secs: u64,              // SIGKILL after timeout
    pub settings_renderer_mode: RendererMode,    // Disabled | VerifyOnly | Reconcile
    pub settings_verify_debounce_secs: u64,      // cache TTL for drift checks
    pub minimum_eligible_accounts: usize,        // minimum eligible before DriftExcluded
}
```

### Port Traits

#### `AccountSnapshotProvider`
```rust
pub trait AccountSnapshotProvider {
    fn snapshot(&self) -> Vec<ProviderAccount>;
}
```
**Semantics (v5 extension):** implementor MAY invoke `SettingsRenderer::verify` for each account; if drift detected, account is excluded from returned vec (or reconciled, per `RendererMode`).

#### `InboxStore`
```rust
pub trait InboxStore {
    async fn peek_lock(&self, ttl_secs: u64) -> Result<SessionTicket, InboxError>;
    async fn commit(&self, ticket: &SessionTicket) -> Result<(), InboxError>;
    async fn rollback(&self, ticket: &SessionTicket) -> Result<(), InboxError>;
    async fn dead_letter(&self, ticket: &SessionTicket, reason: &str) -> Result<(), InboxError>;
}
```
**Atomic contract:** peek_lock + commit form a linearizable pair; race on TTL expiry → commit fails.

#### `OutboxSink`
```rust
pub trait OutboxSink {
    async fn append(&self, record: &SpendRecord) -> Result<(), OutboxError>;
    async fn flush(&self) -> Result<(), OutboxError>;
}
```
**Durability:** fsync'd before return.

#### `SessionDriver`
```rust
pub trait SessionDriver {
    async fn spawn_for_message(
        &self,
        ticket: &SessionTicket,
    ) -> Result<SessionOutcome, SessionError>;
    
    async fn inject_response(&self, ticket: &SessionTicket, response: &[u8])
        -> Result<(), SessionError>;
    
    async fn drain_response(&self, ticket: &SessionTicket) -> Result<Vec<u8>, SessionError>;
    
    async fn kill(&self, ticket: &SessionTicket) -> Result<(), SessionError>;
}
```
**Per-provider:** one impl each for Claude, Codex, Gemini CLIs.

### Composition Shims

#### `RoutePolicy`
Wraps `intelligence-route-policy-kernel`. Method: `select(eligible_accounts: &[ProviderAccount]) -> Result<AccountId, NoEligibleAccount>`.

#### `UsageEnforcement`
Wraps `intelligence-usage-window-kernel`. Enforces Cedar autonomy-tier ceilings via `check_limit(&ticket, &spend_record) -> Result<(), OverLimit>`.

## Usage Example

```rust
use intelligence_supervisor_kernel::*;

// In supervisor-app:
let accounts = snapshot_provider.snapshot().await?;
let ticket = inbox.peek_lock(60).await?;
let outcome = driver.spawn_for_message(&ticket).await?;
inbox.commit(&ticket).await?;
outbox.append(&spend_record).await?;
```

## Data Flow

```
┌─────────────────────────┐
│  tick_once() loop       │
│  (supervisor-app)       │
└────────┬────────────────┘
         │
         ├─→ AccountSnapshotProvider::snapshot()
         │   └─→ (includes SettingsRenderer::verify per v5)
         │
         ├─→ InboxStore::peek_lock()
         │
         ├─→ RoutePolicy::select(eligible_accounts)
         │
         ├─→ SessionDriver::spawn_for_message()
         │
         └─→ OutboxSink::append(SpendRecord)
```

## Audit Trail

Every message transition emits an audit event per ADR-0003:
- `foundry_supervisor_spawn`
- `foundry_supervisor_degrade_account` (usage exceeded)
- `foundry_supervisor_quarantine` (autonomy tier blocked)
- `foundry_supervisor_rotate_window` (usage window expired)
- `foundry_supervisor_settings_drift_exclude` (drift detected)

## References

- **Plan:** `ralplan-foundry-supervisor-simple-v4-2026-05-14.md` §B.2
- **ADRs:** ADR-0056 (12-layer enum), ADR-0003 (audit chain), ADR-0024 (autonomy ceiling)
- **Design:** `docs/DESIGN.md` (foundry supervisor axis contract row)
