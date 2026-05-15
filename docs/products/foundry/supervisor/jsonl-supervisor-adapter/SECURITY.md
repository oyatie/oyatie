---
doc_class: Standard
purpose: "File permissions, race conditions, and symlink defense"
owner_team: axis-foundry
status: draft
---

# JSONL Adapter — Security

## File Permissions

All files are created with restricted access:

```
~/.oya/inbox/*.json          mode 0o600 (rw-------)
~/.oya/inbox/*.lock          mode 0o600 (rw-------)
~/.oya/inbox/dead-letter/    mode 0o700 (rwx------)
~/.oya/outbox/spend-*.jsonl  mode 0o600 (rw-------)
```

**Set at creation:**
```rust
std::fs::OpenOptions::new()
    .create(true)
    .write(true)
    .mode(0o600)
    .open(path)?
```

## Race Conditions

### Peek-Lock Race

**Scenario:** Two processes call `peek_lock()` simultaneously on the same message.

**Mitigation:** Lock file creation is atomic (single `rename(2)` syscall).

1. Process A: creates `.tmp`, renames to `.lock` ← atomic
2. Process B: attempts rename, gets EEXIST
3. Process B: backs off, checks lock age

**Residual race:** If two processes write `.tmp` simultaneously, one loses. This is acceptable (rare).

### Commit Race

**Scenario:** One process commits while another locks the same message.

**Mitigation:**
```
commit() checks:
  1. lock_holder == ticket.request_id (matches our lock)
  2. Proceeds only if owned
```

If lock ownership mismatches, commit fails with `InboxError::NotLocked`.

## No Plaintext Secrets

JSONL files contain **only message metadata**, never secrets:

```json
{
  "message_id": "msg-abc123",
  "account_id": "acct-xyz",
  "provider_family": "Claude",
  "request_json": "{ ... }"
}
```

**Never:**
```json
{
  "api_token": "sk-ant-xxxxx",
  "secret_value": "..."
}
```

Secrets are resolved at spawn-time via OpenBao; the JSONL layer never sees them.

## Audit Trail

Every inbox operation emits an event:

```json
{
  "event_class": "foundry_supervisor_inbox_peek_lock",
  "message_id": "msg-abc123",
  "ttl_expiry": "2026-05-15T10:35:45Z"
}
```

Dead-letter operations include reason:

```json
{
  "event_class": "foundry_supervisor_dead_letter",
  "message_id": "msg-abc123",
  "reason": "InvalidTemplate(duplicate-keys)"
}
```

## References

- **Atomicity:** `docs/products/foundry/supervisor/jsonl-supervisor-adapter/ARCHITECTURE.md`
- **POSIX files:** IEEE 1003.1-2017 (rename, chmod, stat)
