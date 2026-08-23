---
doc_class: Standard
purpose: "File-backed inbox and outbox adapter using JSONL format"
owner_team: axis-foundry
status: draft
doc_status: published
---

# JSONL Supervisor Adapter — README

**Crate:** `intelligence-jsonl-supervisor-adapter`  
**Layer:** Adapter (12-layer-enum L4)  
**Wave:** 2c (M02-P06)  
**Key trait:** `InboxStore`, `OutboxSink`

## Overview

The JSONL supervisor adapter is the **only fsync-aware crate in the supervisor stack** (Option D, v4 §A.3). All disk I/O — inbox read/write, dead-letter moves, outbox appends — is isolated here. The supervisor-kernel's port traits are implemented on file-backed structs.

### Purpose

Provide persistent, durable inbox and outbox storage via:
- **File-backed inbox** — messages as `.jsonl` files in a directory tree
- **Peek-lock mechanism** — atomic state transitions (Locked/Unlocked/DeadLettered)
- **TTL enforcement** — lock expiry after configurable timeout
- **Atomic operations** — tempfile + rename(2) for crash safety
- **Dead-letter routing** — failed messages moved to quarantine directory
- **Idempotency cache** — request_id → outcome memoization (24-hour TTL)

## Public API Summary

### `JsonlInboxStore`

Implements `InboxStore` port trait.

```rust
pub struct JsonlInboxStore {
    root: PathBuf,  // typically ~/.oya/inbox
}

impl InboxStore for JsonlInboxStore {
    async fn peek_lock(&self, ttl_secs: u64) -> Result<SessionTicket, InboxError>;
    async fn commit(&self, ticket: &SessionTicket) -> Result<(), InboxError>;
    async fn rollback(&self, ticket: &SessionTicket) -> Result<(), InboxError>;
    async fn dead_letter(&self, ticket: &SessionTicket, reason: &str) -> Result<(), InboxError>;
}
```

**Files on disk:**
```
~/.oya/inbox/
├─ msg-abc123.json         # message content (unlocked)
├─ msg-abc123.lock         # lock file (if locked)
├─ msg-abc123.reason       # reason file (if dead-lettered)
└─ dead-letter/
   └─ msg-xyz789.json      # moved from inbox
```

### `JsonlOutboxSink`

Implements `OutboxSink` port trait.

```rust
pub struct JsonlOutboxSink {
    root: PathBuf,  // typically ~/.oya/outbox
}

impl OutboxSink for JsonlOutboxSink {
    async fn append(&self, record: &SpendRecord) -> Result<(), OutboxError>;
    async fn flush(&self) -> Result<(), OutboxError>;
}
```

**Files on disk:**
```
~/.oya/outbox/
├─ spend-records.jsonl     # append-only; every append fsync'd
└─ spend-records.json.bak  # backup after flush
```

## Usage Example

```rust
use intelligence_jsonl_supervisor_adapter::*;
use intelligence_supervisor_kernel::*;

let inbox = JsonlInboxStore::new("~/.oya/inbox")?;
let outbox = JsonlOutboxSink::new("~/.oya/outbox")?;

// Peek next message (blocks up to TTL seconds if locked)
let ticket = inbox.peek_lock(60).await?;

// ... process message ...

// Commit (acknowledge; message removed from inbox)
inbox.commit(&ticket).await?;

// Record spend
let spend = SpendRecord { ... };
outbox.append(&spend).await?;
```

## Atomic Operations

### Peek-Lock

```
1. open msg-abc123.json with O_CLOEXEC
2. write msg-abc123.lock (contains lock_holder=<uuid>, ttl_expiry=<epoch>)
3. return SessionTicket
```

**TTL expiry:** Next `peek_lock()` checks the lock file's age; if >ttl_secs, removes lock and proceeds.

### Commit

```
1. Verify lock holder matches ticket.request_id
2. Remove msg-abc123.lock
3. Remove msg-abc123.json
4. Atomicity: both files MUST be gone (no partial state)
```

### Dead-Letter

```
1. mkdir -p dead-letter/
2. write dead-letter/msg-abc123.reason (reason text)
3. mv msg-abc123.lock dead-letter/msg-abc123.lock (if locked)
4. mv msg-abc123.json dead-letter/msg-abc123.json
```

### Outbox Append (fsync'd)

```
1. open ~/.oya/outbox/spend-records.jsonl with O_APPEND
2. write one JSONL row
3. fsync() — wait until written to disk
4. return Ok(())
```

## Idempotency Cache

Every request carries a `RequestId` (opaque string). Duplicate requests within 24 hours return cached outcomes:

```
~/.oya/.idempotency-cache/
├─ req-abc123.json  # { request_id: "abc123", outcome: "Spawned(...)" }
├─ req-xyz789.json
└─ ...
```

Cache entries are pruned daily (background cleanup task).

## File Layout

```
~/.oya/supervisor/
├─ config.toml                           # daemon config
├─ inbox/
│  ├─ msg-20260515-001.json             # pending message
│  ├─ msg-20260515-002.json
│  ├─ msg-20260515-001.lock             # locked message (TTL active)
│  └─ dead-letter/
│     ├─ msg-20260515-003.json          # failed message
│     └─ msg-20260515-003.reason        # failure reason
└─ outbox/
   ├─ spend-records.jsonl               # append-only journal
   └─ spend-records.jsonl.bak           # last-flush backup
```

## Performance Characteristics

- **Peek-lock:** O(1) file I/O
- **Commit:** O(1) file delete (only if no pending writes)
- **Outbox append:** O(1) but fsync is slow (≤100µs per write, shared with kernel)
- **Idempotency lookup:** O(1) file stat

## References

- **Trait definition:** `intelligence-supervisor-kernel::InboxStore`, `OutboxSink`
- **Plan:** v4 §B.3 (JSONL adapter spec), §A.3 (Option D architecture)
- **Kernel README:** `docs/products/foundry/supervisor/supervisor-kernel/README.md`
