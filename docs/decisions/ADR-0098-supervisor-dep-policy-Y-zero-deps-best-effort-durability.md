---
id: ADR-0098
title: "Supervisor dependency policy Branch Y — zero net-new external Cargo deps + best-effort durability"
status: Accepted
doc_status: published
owner: council-architecture
date: 2026-05-15
owner_phase: M02-P06
deciders:
  - Architect (v4 §A.1 principle 4)
  - Critic (v6 F-BRANCH-Y-COST-BENEFIT-QUANTIFY-ADR-1)
supersedes: []
related:
  - ADR-0092  # workspace dependency seam policy (canonical dep-growth governance)
  - ADR-0096  # supervisor language: Rust (establishes crate decomposition)
  - ADR-0003  # audit-chain emission (crash-atomicity requirement)
---

# ADR-0098: Supervisor Dep-Policy Branch Y: Zero Net-New External Deps + Best-Effort Durability

## Status

Accepted (v4 §A.1 principle 4; v6 FixupTask F-BRANCH-Y-COST-BENEFIT-QUANTIFY-ADR-1).

## Context

The foundry supervisor (M02-P06) writes JSONL to disk for inbox/outbox and dead-letter queues.
The implementation must choose between three dependency strategies (Branches X, Y, Z) that differ
in external crate count, I/O model, and durability guarantees.

**What the supervisor writes:**
- Per-session inbox files (`registry/sessions/<id>/inbox.jsonl`) — append-only, one JSON line per
  injected message
- Per-session outbox files (`registry/sessions/<id>/outbox.jsonl`) — append-only, one JSON line
  per completed tick emission
- Dead-letter file (`registry/sessions/dead-letter.jsonl`) — append-only, poison-message record

**Durability question:** Should writes be durable across power loss (requiring `fsync` of both file
and parent directory), or best-effort (relying on OS page-cache flush on clean shutdown only)?

**I/O model question:** Should the supervisor use async file I/O (tokio::fs, requiring async_trait
on `SessionDriver`) or synchronous file I/O on a tokio blocking thread pool?

## Decision

**Branch Y — zero net-new external Cargo deps, sync I/O on tokio blocking pool, best-effort durability.**

Concrete shape:

```rust
// SessionDriver trait — synchronous; no async_trait dependency
pub trait SessionDriver: Send + Sync {
    fn start_session(&self, ticket: &SessionTicket) -> Result<SessionHandle, SessionError>;
    fn stop_session(&self, handle: &SessionHandle) -> Result<(), SessionError>;
    fn inject_message(&self, handle: &SessionHandle, msg: &InboxMessage) -> Result<(), SessionError>;
    fn idle_tick(&self, handle: &SessionHandle) -> Result<TickOutcome, SessionError>;
}

// Blocking pool: tokio default (512 threads max); exposed via tokio::task::spawn_blocking
// max_in_flight: Semaphore::new(MAX_IN_FLIGHT) where MAX_IN_FLIGHT is a const (default 64)
// File writes: std::fs::OpenOptions + std::io::BufWriter — no async_trait, no rustix
```

**Durability policy:**

```rust
// fsync the file after each append — best-effort
file.flush()?;
file.sync_all()?;  // syncs file data + metadata to storage device

// fsync(parent_dir) — NOT implemented
// Rationale: std::fs::sync_all operates on File, not dirfd.
// Implementing fsync(parent_dir) requires opening the directory as a File (Unix-only)
// or using platform-specific syscalls (rustix/libc). This would add a net-new external
// dep (rustix) or platform-specific unsafe code — violating Branch Y.
// Consequence: a power-loss event between directory-entry creation and a subsequent
// fsync of the parent directory can cause the file to be invisible on remount even
// though its contents are durable. This is the documented non-durability.
```

**JSONL adapter documentation (non-power-loss-durability notice):**

```
/// # Durability
///
/// `JsonlInboxStore` and `JsonlOutboxSink` guarantee:
/// - Append atomicity per-line on POSIX systems (write(2) < PIPE_BUF is atomic).
/// - File-level durability: `sync_all()` is called after each append.
///
/// They do NOT guarantee:
/// - Directory-entry durability across power loss (`fsync(dirfd)` is not called).
///   A file created immediately before power loss may be invisible on remount even
///   if its data was flushed.
///
/// Consequence: the dead-letter queue and inbox/outbox files are suitable for
/// operational resilience (process crash, clean shutdown) but not for power-loss-safe
/// write-ahead logging. If power-loss durability is required in a future wave, reopen
/// this ADR and evaluate Branch X (rustix) per F-BRANCH-Y-COST-BENEFIT-QUANTIFY-ADR-1.
```

## Decision Drivers

1. **ADR-0092 — workspace dependency seam policy** — External Cargo deps require a seam decision.
   The workspace has an existing supply-chain check (LEAN-A3, `cargo-deny`) that blocks unapproved
   additions. Adding `rustix` or `async_trait` would require a new seam justification PR
   per ADR-0092 protocol.

2. **v4 §A.1 principle 4 — dep-branch-Y-commitment** — The v4 plan explicitly commits to
   Branch Y as default for M02-P06 and defers dep-growth evaluation to a post-quantification
   reopener (F-BRANCH-Y-COST-BENEFIT-QUANTIFY-ADR-1). No benchmark data yet justifies the cost.

3. **tokio blocking pool sufficiency** — The supervisor processes one tick per active session per
   idle interval (default 30 s). At 64 concurrent sessions (`max_in_flight = 64`) with ≤ 10 ms
   blocking I/O per tick, the blocking pool is idle > 97% of the time. No async I/O is warranted.

4. **`async_trait` dep cost** — `async_trait` (proc-macro) adds compile-time overhead and a
   new dependency audit surface. Synchronous `SessionDriver` avoids it entirely. If the trait
   must cross an async boundary, `spawn_blocking` at the call site is sufficient.

5. **`rustix` dep cost** — `rustix` provides `fsync_all(dirfd)` but is an external crate not
   currently in the workspace. Adding it for a single `fsync(parent_dir)` call fails the
   cost-benefit threshold when best-effort durability is acceptable for this workload
   (operational resilience, not financial ledger).

## Alternatives Considered

### Branch X — rustix + async_trait

**Shape:** `async_trait` on `SessionDriver`; `rustix::io::fsync_all(parent_dir_fd)` for full
power-loss durability.

**Pros:**
- Full POSIX power-loss durability (directory entry flushed before file content flush).
- Async trait allows direct `await` in `SessionDriver` implementations without `spawn_blocking`.

**Cons:**
- 2 net-new external Cargo deps: `rustix` (unsafe FFI surface) + `async_trait` (proc-macro).
- `cargo-deny` LEAN-A3 requires justification PR per ADR-0092 before either dep can land.
- `rustix` adds platform-specific unsafe code to the supply chain.
- Async `SessionDriver` complicates the `max_in_flight` semaphore: callers must hold the permit
  across `.await` points, requiring `Arc<Semaphore>` threading through async call stacks.
- No measured benchmark shows blocking pool is the bottleneck at target concurrency (64 sessions).

**Verdict: REJECTED** — Dep-growth cost not justified by measured need. Reopen via
F-BRANCH-Y-COST-BENEFIT-QUANTIFY-ADR-1 after benchmark data is available.

### Branch Z — Hybrid (partial async, tokio::fs for outbox; sync for dead-letter)

**Shape:** `tokio::fs` for high-frequency outbox writes (async, zero new deps); `std::fs` for
low-frequency dead-letter writes (sync). `SessionDriver` trait remains sync.

**Pros:**
- Zero new deps (tokio::fs is already in the workspace via tokio full feature).
- Async outbox writes avoid blocking pool for the hot path.

**Cons:**
- Two different I/O models in the same adapter crate increases cognitive surface.
- `tokio::fs` does not call `fsync` by default; achieving file-level durability still requires
  explicit `file.sync_all().await` — identical semantics to Branch Y sync path.
- No power-loss durability improvement over Branch Y (same `fsync(parent_dir)` limitation).
- Complexity overhead not justified at 64-session target concurrency.

**Verdict: REJECTED** — No durability improvement; complexity overhead not justified.

## Consequences

### Positive
- Zero net-new external Cargo deps; LEAN-A3 passes without new seam justification.
- `SessionDriver` trait is simpler to implement: no `async_trait`, no `Pin<Box<dyn Future>>`.
- `max_in_flight` semaphore is straightforward: `let _permit = semaphore.acquire().await?;`
  wraps a `spawn_blocking` call; no permit leak across async boundaries.
- Blocking pool default (512 threads) is sufficient for target concurrency; no tuning needed
  at M02-P06 scope.

### Negative / Trade-offs
- Power-loss between file creation and `fsync(parent_dir)` can make a newly created JSONL
  file invisible on remount. This is documented in the JSONL adapter's `/// # Durability`
  rustdoc block (see §Decision above).
- If a future wave requires power-loss-safe WAL semantics (e.g., financial audit trail),
  Branch X must be re-evaluated with benchmark evidence.

### Accepted non-durability

Per v4 §A.1 principle 4 and ADR-0096 §"Accepted non-durability":
> Best-effort durability; no `fsync(parent_dir)`; non-durability-across-power-loss is explicitly
> accepted for M02-P06 JSONL inbox/outbox files.

This decision is re-openable via F-BRANCH-Y-COST-BENEFIT-QUANTIFY-ADR-1 after a measured
benchmark demonstrates that blocking-pool I/O is the latency bottleneck at production concurrency.

## Follow-ups

1. **F-BRANCH-Y-COST-BENEFIT-QUANTIFY-ADR-1** — After Wave 2f bench harness (`heartbeat.rs`)
   produces JSONL latency data at 64-session concurrency, re-evaluate Branch X threshold.
   If p99 tick latency > 50 ms, reopen this ADR.
2. **JSONL adapter rustdoc** — `oya-intelligence-jsonl-supervisor-adapter/src/lib.rs` must include
   the `/// # Durability` block verbatim as specified in §Decision above.
3. **Semaphore constant** — `MAX_IN_FLIGHT: usize = 64` must be a workspace-level const
   declared in `oya-intelligence-supervisor-kernel` so all callers share the same default.

## References

- ADR-0092 — workspace dependency seam policy (dep-growth governance)
- ADR-0096 §"Accepted non-durability" — predecessor decision establishing Branch Y commitment
- v4 §A.1 principle 4 — dep-branch-Y-commitment
- v6 FixupTask F-BRANCH-Y-COST-BENEFIT-QUANTIFY-ADR-1 — reopener trigger
- POSIX `write(2)` — atomicity guarantee for writes < `PIPE_BUF`
- `std::fs::File::sync_all` — Rust stdlib file-level fsync (data + metadata, not dirfd)
