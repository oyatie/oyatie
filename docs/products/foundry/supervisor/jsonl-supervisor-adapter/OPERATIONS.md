---
doc_class: Runbook
purpose: "Inbox/outbox maintenance, cleanup, and recovery procedures"
owner_team: axis-foundry
status: draft
doc_status: published
---

# JSONL Adapter — Operations

## Initialization

```bash
# Create inbox/outbox directories
mkdir -p ~/.oya/inbox
mkdir -p ~/.oya/inbox/dead-letter
mkdir -p ~/.oya/outbox

# Verify permissions
ls -la ~/.oya/inbox
ls -la ~/.oya/outbox

# Clean up stale lock files (from crashes)
find ~/.oya/inbox -name "*.lock" -mmin +60 -delete
```

## Monitoring

### Inbox Depth

```bash
# Count pending messages
ls ~/.oya/inbox/*.json | wc -l

# Count locked messages
ls ~/.oya/inbox/*.lock | wc -l

# Count dead-lettered
ls ~/.oya/inbox/dead-letter/ | wc -l
```

### Outbox Status

```bash
# Check spend records written
wc -l ~/.oya/outbox/spend-records.jsonl

# Size on disk
du -h ~/.oya/outbox/spend-records.jsonl

# Last record
tail -1 ~/.oya/outbox/spend-records.jsonl
```

## Cleanup Tasks

### Daily Lock Cleanup

Stale locks prevent inbox progress. Remove locks older than TTL:

```bash
#!/bin/bash
# cleanup-locks.sh
find ~/.oya/inbox -name "*.lock" -type f | while read lock; do
  age_secs=$(( $(date +%s) - $(stat -f%m "$lock" 2>/dev/null || stat -c%Y "$lock") ))
  if [ $age_secs -gt 300 ]; then  # 300 = 5-minute TTL
    rm "$lock"
    echo "Removed stale lock: $lock"
  fi
done
```

Run via cron:

```bash
# /etc/cron.d/supervisor-cleanup
0 * * * * /opt/oya/bin/cleanup-locks.sh
```

### Weekly Idempotency Cache Cleanup

Remove cache entries older than 24 hours:

```bash
#!/bin/bash
find ~/.oya/.idempotency-cache -name "*.json" -type f -mtime +1 -delete
```

### Monthly Outbox Archival

Compress and archive old spend records:

```bash
#!/bin/bash
if [ -f ~/.oya/outbox/spend-records.jsonl ]; then
  lines_before=$(wc -l < ~/.oya/outbox/spend-records.jsonl)
  gzip -c ~/.oya/outbox/spend-records.jsonl > ~/.oya/outbox/spend-records-$(date +%Y%m%d).jsonl.gz
  echo "Archived $lines_before records"
fi
```

## Recovery Procedures

### Recover from Dead-Letter

If a message is incorrectly dead-lettered:

```bash
# List dead-lettered messages
ls ~/.oya/inbox/dead-letter/

# Check reason
cat ~/.oya/inbox/dead-letter/msg-abc123.reason

# Re-queue
mv ~/.oya/inbox/dead-letter/msg-abc123.json ~/.oya/inbox/
```

### Force-Release a Locked Message

If a lock is stuck (daemon crashed without cleanup):

```bash
# Remove lock file
rm ~/.oya/inbox/msg-abc123.lock

# Message is now available for next peek_lock
```

### Rebuild Idempotency Cache

If cache is corrupted:

```bash
# Clear cache (requests within 24h may re-process)
rm -rf ~/.oya/.idempotency-cache/

# Recreate empty cache
mkdir -p ~/.oya/.idempotency-cache
```

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| "Inbox locked" repeated | Stale lock file | Run cleanup-locks.sh |
| "Outbox full" errors | Disk quota exceeded | Archive spend-records.jsonl |
| Duplicate spend records | Idempotency cache miss | Ensure cache directory is writable |
| Dead-letter accumulating | Bug in message parsing | Investigate reason file |

## Performance Tips

1. **Keep inbox small** — drain processed messages regularly
2. **Archive outbox monthly** — prevent file from growing unbounded
3. **Monitor lock count** — alerts if > 10% of messages are locked
4. **Check disk space** — spend-records.jsonl grows ~1KB per message

## References

- **Atomicity model:** `docs/products/foundry/supervisor/jsonl-supervisor-adapter/ARCHITECTURE.md`
- **v4 Plan § B.3:** JSONL adapter spec
