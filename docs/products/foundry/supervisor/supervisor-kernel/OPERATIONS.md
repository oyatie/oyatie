---
doc_class: Runbook
purpose: "Operational runbooks for supervisor kernel layer (no daemon, kernel-only concerns)"
owner_team: axis-foundry
status: draft
doc_status: published
---

# Supervisor Kernel — Operations

## Scope

The kernel itself is pure value types and port traits. Operational concerns belong in the **Application layer** (`supervisor-app`). This document covers the kernel's role in the larger supervisor lifecycle.

## Port Trait Implementation Checklist

When implementing adapter crates, ensure:

### `InboxStore` Implementation
- [ ] `peek_lock()` returns unique `MessageId` on success
- [ ] `peek_lock()` fails with `InboxError::Locked` if TTL not expired
- [ ] `commit()` succeeds only if `MessageId` matches current lock holder
- [ ] `rollback()` releases lock without consuming the message
- [ ] `dead_letter()` moves message to `dead-letter/` directory with reason file
- [ ] All operations are atomic (tempfile + rename, or equivalent)


### `OutboxSink` Implementation
- [ ] `append()` fsync's immediately before returning
- [ ] `flush()` is idempotent
- [ ] Records are never duplicated (ULID idempotency)
- [ ] `SpendRecord` fields are in canonical order for audit replay


### `SessionDriver` Implementation (Per-Provider)
- [ ] `spawn_for_message()` initializes a new session with the provider CLI
- [ ] `inject_response()` writes to the session's request FIFO/socket
- [ ] `drain_response()` reads from the session's response channel
- [ ] `kill()` sends SIGKILL to the spawned process
- [ ] All operations preserve the `SessionTicket` invariant (value-only)


## Debugging Common Issues

### `SessionTicket` Invariant Violation

**Symptom:** Compiler error about `Arc` or `Box<dyn>` in `SessionTicket` field.

**Root cause:** Attempting to store a reference that requires allocation.

**Fix:** Extract the reference data into a owned value type in the kernel. Example:

```rust
// WRONG:
pub struct SessionTicket {
    pub account: &'static ProviderAccount,  // ✗
}

// RIGHT:
pub struct SessionTicket {
    pub account_id: AccountId,  // ✓
    pub provider_family: ProviderFamily,  // ✓
}
```

### `InboxStore` Deadlock

**Symptom:** `peek_lock()` returns `Locked` indefinitely.

**Root cause:** Previous lock holder crashed without releasing; TTL expired but cleanup didn't run.

**Fix:** Implement a background cleanup task that scans locks older than TTL and removes them. **OR** use distributed consensus (Etcd, Zookeeper) for lock management in multi-instance deployments.


### `TickOutcome::DriftExcluded` False Positives

**Symptom:** Legitimate accounts excluded from routing due to settings drift.

**Root cause:** `SettingsRenderer::verify()` is comparing against a stale template or incomplete render manifest.

**Fix:**
1. `templates/foundry-supervisor/` was deleted (hooks pointed at missing `tools/foundry-supervisor-*` binaries). Do not expect that path; settings-template drift for that tree is retired until a replacement lands.
2. Force re-render (local bridge only): `cargo run -p dev-cli -- settings-drift --reconcile`
3. Check drift report: `cat .omc/state/settings-drift-report.json`

## Audit Trail Verification

Every message passing through the supervisor system emits audit events (ADR-0003). To verify:

```bash
# Check audit chain integrity:
cargo run -p dev-cli -- audit verify-chain \
  --input evidence/audit-chain.jsonl \
  --expected-events 'foundry_supervisor_*'

# Count events by type:
jq -r '.event_class' evidence/audit-chain.jsonl | sort | uniq -c
```

Expected events per `tick_once()` call:
- `foundry_supervisor_spawn` (1 per successful spawn)
- `foundry_supervisor_degrade_account` (1 if usage exceeded)
- `foundry_supervisor_quarantine` (1 if autonomy tier hit)
- `foundry_supervisor_rotate_window` (1 per usage window rollover)
- `foundry_supervisor_settings_drift_exclude` (1 per excluded account)

## References

- **Kernel source:** `crates/intelligence-supervisor-kernel/src/lib.rs`
- **Application runbook:** `docs/products/foundry/supervisor/supervisor-app/OPERATIONS.md`
- **Adapter runbooks:**
  - JSONL: `docs/products/foundry/supervisor/jsonl-supervisor-adapter/OPERATIONS.md`
  - Settings: `docs/products/foundry/supervisor/settings-template-adapter/OPERATIONS.md`
