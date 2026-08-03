---
doc_class: Standard
purpose: "I/O performance characteristics and fsync latency budgets"
owner_team: axis-foundry
status: draft
doc_status: published
---

# JSONL Adapter — Benchmarks

## Latency Budgets

Per v4 §C.13, the JSONL adapter contributes:

| Operation | p95 Latency | Notes |
|-----------|------------|-------|
| `peek_lock()` | ≤10ms | file I/O + stat check |
| `commit()` | ≤5ms | rename(2) + cleanup |
| `append()` + fsync | ≤100µs | fsync is the bottleneck |

## Microbenchmarks

### Peek-Lock Latency

```rust
let start = Instant::now();
let ticket = inbox.peek_lock(60).await?;
let latency_µs = start.elapsed().as_micros();
```

**Budget:** p95 ≤ 10,000 µs

**Factors:**
- File open: ~1µs
- Lock file creation: ~10µs
- stat (age check): ~2µs
- **Total:** ~15µs typical

### Commit Latency

```rust
let start = Instant::now();
inbox.commit(&ticket).await?;
let latency_µs = start.elapsed().as_micros();
```

**Budget:** p95 ≤ 5,000 µs

**Factors:**
- Verify lock ownership: ~1µs
- unlink(msg.json): ~5µs
- unlink(msg.lock): ~5µs
- **Total:** ~10µs typical

### Outbox Append + fsync

```rust
let start = Instant::now();
outbox.append(&spend_record).await?;
let latency_µs = start.elapsed().as_micros();
```

**Budget:** p95 ≤ 100 µs

**Factors:**
- JSON serialization (hand-rolled): ~5µs
- open(O_APPEND): ~1µs
- write(): ~2µs
- **fsync():** ~50-80µs (disk dependent)
- **Total:** ~60µs typical

**fsync depends on:**
- SSD vs HDD (SSD: 50µs, HDD: 5ms)
- Kernel page cache state
- Block device queue depth

## Profiling

To identify slow operations:

```bash
# Sample peek_lock latencies
cargo bench -p intelligence-jsonl-supervisor-adapter -- peek_lock_latency

# Profile with flamegraph
cargo flamegraph -p intelligence-jsonl-supervisor-adapter --bench jsonl_ops
```

## Storage Characteristics

### Inbox Size

Average message: ~500 bytes. With lock file: ~1KB per message.

```
100 pending messages = 100KB
1000 pending messages = 1MB
```

Monitor with:
```bash
du -sh ~/.oya/inbox
```

### Outbox Size

Average spend record: ~200 bytes (JSONL-formatted).

```
1000 records = 200KB
10000 records = 2MB
100000 records = 20MB
```

### Growth Rate

At 12 max_in_flight × 1 sec tick = 12 messages/sec:

```
~2.4KB/sec spend records
~140KB/min
~8.4MB/hour
~200MB/day
```

Recommend monthly archival (see OPERATIONS).

## Acceptance Criteria

| Criterion | Verification |
|-----------|--------------|
| **C.peek** | Peek-lock p95 ≤ 10µs on 1000-message fixture |
| **C.commit** | Commit p95 ≤ 5µs on 1000-message fixture |
| **C.append** | Append+fsync p95 ≤ 100µs on SSD |
| **C.idempotency** | Cache lookup ≤ 1µs; O(1) |

## References

- **v4 Plan § C.13:** Latency budgets
- **Benchmarks:** `intelligence/adapters/jsonl-supervisor-adapter/benches/`
