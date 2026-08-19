---
doc_class: Standard
purpose: "Performance budgets, benchmark harness, and metrics collection"
owner_team: axis-foundry
status: draft
doc_status: published
---

# Supervisor Kernel — Benchmarks

## Performance Budget

Per v4 §C.13a-d and v6 BLOCKER-2, the supervisor kernel contributes to the following overall latency budgets:

| Component | p95 Latency | Notes |
|-----------|------------|-------|
| **Session spawn** | ≤100ms | time from `peek_lock` to `SessionDriver::spawn_for_message` return |
| **Message encode** | ≤10ms | SessionTicket → wire format |
| **Routing decision** | ≤5ms | RoutePolicy::select + UsageEnforcement::check_limit |
| **Audit emission** | ≤1ms | ADR-0003 event write |
| **tick_once() total** | ≤250ms | end-to-end including inbox + outbox I/O |

**Baseline:** Measured on 100-account fixture with 1000 messages in inbox (see Bench Harness below).

## Benchmark Harness

The supervisor-app crate hosts the harness in `benches/heartbeat.rs`. The kernel contributes type definitions and trait boundaries.

### Running Benchmarks

```bash
# Run with 200 iterations per metric (v6 BLOCKER-2):
cargo bench -p intelligence-supervisor-app -- heartbeat

# Output: .omc/state/benchmark-results/heartbeat-p95.jsonl
```

### Output Format

Each metric emits one JSONL row:

```json
{
  "metric": "session_spawn_latency_micros",
  "samples_count": 200,
  "p50": 25000,
  "p95": 95000,
  "p99": 120000,
  "max": 150000,
  "unit": "microseconds",
  "fixture": "100-accounts-1000-messages",
  "timestamp_iso": "2026-05-15T10:30:45Z"
}
```

**Samples collection:** Loop ≥200 iterations per metric. Collect samples in `Vec<u64>`. Sort. Report `vec[(0.95 * len as f64) as usize]` as p95.

### Multi-Sample p95 Guarantee

The harness runs each metric at least 200 times. Per-sample variance is captured:

```bash
# Extract p95 variance:
jq '.[] | select(.metric == "session_spawn_latency_micros") | .p95' \
  .omc/state/benchmark-results/heartbeat-p95.jsonl | \
  awk '{sum+=$1; sumsq+=$1*$1; n++} END {print sqrt(sumsq/n - (sum/n)^2)}'
```

**Acceptance:** p95 variance ≤ 10% of p95 value (stable latency distribution).

## Key Metrics

### 1. `session_spawn_latency_micros` (v4 §C.13a)

Time from `tick_once()` step 1 to `SessionDriver::spawn_for_message()` return.

```rust
let start = std::time::Instant::now();
let outcome = driver.spawn_for_message(&ticket).await?;
let elapsed = start.elapsed().as_micros() as u64;
```

**Budget:** p95 ≤ 100_000 µs

**Test fixture:** 100 accounts, 1000 messages in inbox.

### 2. `message_encode_latency_micros` (v4 §C.13b)

Time to serialize `SessionTicket` to wire format (JSONL or protobuf, per adapter).

**Budget:** p95 ≤ 10_000 µs

**Test:** `SessionTicket { account_id, provider, tier, window_snapshot, message_id, request_id }` → bytes.

### 3. `routing_decision_latency_micros` (v4 §C.13c)

Time for `RoutePolicy::select()` + `UsageEnforcement::check_limit()`.

**Budget:** p95 ≤ 5_000 µs

**Test:** 100 eligible accounts; policy returns a deterministic choice.

### 4. `tick_once_total_latency_micros` (v4 §C.13d)

End-to-end time from `tick_once()` entry to return (excluding I/O to adapters).

**Budget:** p95 ≤ 250_000 µs

**Breakdown:**
- Step 1–2 (snapshot): ≤50ms
- Step 3–5 (route + enforce): ≤10ms
- Step 6 (spawn): ≤100ms
- Step 7–14 (inject, drain, spend): ≤60ms
- Step 15 (audit): ≤1ms
- **Total:** ≤250ms (12 concurrent sessions × 20ms each ≈ 240ms)

## Acceptance Criteria

| Criterion | Verification |
|-----------|--------------|
| **C.13a** | `cargo bench heartbeat` emits `session_spawn_latency_micros` row with `p95 ≤ 100000` |
| **C.13b** | `message_encode_latency_micros` row with `p95 ≤ 10000` |
| **C.13c** | `routing_decision_latency_micros` row with `p95 ≤ 5000` |
| **C.13d** | `tick_once_total_latency_micros` row with `p95 ≤ 250000` |
| **C.13-p95-stability** | p95 variance across 200 samples ≤ 10% of p95 value |
| **C.13-200-samples** | Every metric has `samples_count: 200` (v6 BLOCKER-2) |

## Profiling

To identify bottlenecks:

```bash
# Generate flamegraph:
cargo flamegraph -p intelligence-supervisor-app --bench heartbeat -- --profile-time 30

# Opens flamegraph.svg
```

Common hotspots:
1. **Snapshot iteration** (100 accounts) → optimize `AccountSnapshotProvider` caching
2. **Route policy selection** → add memoization per v6 BLOCKER-1
3. **Audit event serialization** → pre-allocate buffers
4. **JSONL encoding** → hand-rolled codec is faster than serde

## References

- **v4 Plan § C.13a-d:** Performance budgets
- **v6 Amendments § BLOCKER-2:** Multi-sample p95 exact methodology
- **Bench harness:** `intelligence/core/supervisor-app/benches/heartbeat.rs`
