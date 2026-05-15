---
doc_class: Standard
purpose: "Atomicity model, fsync placement, and crash safety guarantees"
owner_team: axis-foundry
status: draft
---

# JSONL Adapter — Architecture

## Atomicity Model

All operations use **tempfile + atomic rename(2)** for crash safety:

```
write_atomically(target_path, content):
  1. fd = open(target + ".tmp", O_CREAT|O_EXCL|O_WRONLY)
  2. write(fd, content)
  3. fsync(fd)  // for durability operations like outbox
  4. rename(target + ".tmp", target)  // atomic on POSIX
```

## Peek-Lock Atomicity

Lock acquisition is **not** atomic (distributed systems trade-off). Instead:

1. **Check:** is msg-abc123.lock younger than TTL?
2. **If yes:** return Locked error
3. **If no (or absent):** proceed (potential race on old systems, acceptable)
4. **Create lock:** write msg-abc123.lock with atomic rename
5. **Verify:** stat again to confirm ownership

## Commit Atomicity

```
delete_atomically(msg-abc123.json, msg-abc123.lock):
  1. rename(msg-abc123.lock, msg-abc123.lock.deleting)
  2. rename(msg-abc123.json, msg-abc123.json.deleting)
  3. unlink(msg-abc123.lock.deleting)
  4. unlink(msg-abc123.json.deleting)
```

If crash between steps 2-3, cleanup task deletes `.deleting` files on next startup.

## Dead-Letter Atomicity

```
dead_letter_atomically(msg-abc123):
  1. mkdir -p dead-letter/
  2. write dead-letter/msg-abc123.reason (reason text)
  3. rename(msg-abc123.json, dead-letter/msg-abc123.json)
  4. rename(msg-abc123.lock, dead-letter/msg-abc123.lock)  // if locked
```

## Outbox Durability

Every `append()` call fsync's immediately:

```rust
async fn append(&self, record: &SpendRecord) -> Result<(), OutboxError> {
    let json = serde_json_mini(&record)?;  // hand-rolled JSON
    let fd = open(self.path, O_APPEND|O_WRONLY)?;
    write(fd, &json)?;
    write(fd, b"\n")?;
    fsync(fd)?;  // <- CRITICAL: wait for disk
    Ok(())
}
```

This guarantees that `append().await` returning means the record is durable.

## Crash Safety

**Scenario 1: Crash during peek_lock**
→ Lock file may exist but be stale. Next startup, TTL cleanup removes it.

**Scenario 2: Crash during commit**
→ `.deleting` files linger. Next startup cleanup script removes them.

**Scenario 3: Crash during outbox append**
→ Partial write may be in kernel buffer. fsync() ensures it hits disk before returning.

**Scenario 4: Duplicate message in inbox**
→ Idempotency cache prevents re-processing within 24 hours.

## References

- **POSIX atomicity:** IEEE 1003.1-2017 (open, rename, fsync)
- **v4 Plan § B.3:** JSONL adapter spec
- **Crash injection tests:** v4 §B.12 grit units 4.51, 4.57
