---
doc_class: Standard
purpose: "Performance budgets and heartbeat benchmark harness for the daemon"
owner_team: axis-foundry
status: draft
doc_status: published
---

# Supervisor App — Benchmarks

## Performance Budget

Per v4 §C.13a-d, the end-to-end `tick_once()` call must complete within the following budget:

| Metric | p95 Latency | Notes |
|--------|------------|-------|
| `session_spawn_latency_micros` | ≤100,000 µs | Including driver spawn + CLI startup |
| `message_encode_latency_micros` | ≤10,000 µs | SessionTicket serialization |
| `routing_decision_latency_micros` | ≤5,000 µs | RoutePolicy + UsageEnforcement |
| `tick_once_total_latency_micros` | ≤250,000 µs | End-to-end (17 steps) |

**Baseline fixture:** 100 accounts, 1000 messages in inbox.

## Benchmark Harness

### Running the Benchmark

```bash
# Run with 200 iterations (v6 BLOCKER-2)
cargo bench -p intelligence-supervisor-app -- heartbeat

# Output file
.omc/state/benchmark-results/heartbeat-p95.jsonl
```

### Output Format

Each metric emits one JSONL row with multi-sample statistics:

```json
{
  "metric": "tick_once_total_latency_micros",
  "samples_count": 200,
  "p50": 180000,
  "p95": 235000,
  "p99": 248000,
  "max": 252000,
  "unit": "microseconds",
  "fixture": "100-accounts-1000-messages",
  "timestamp_iso": "2026-05-15T10:30:45Z",
  "hardware": "arm64-darwin-m2"
}
```

**Multi-sample requirement (v6 BLOCKER-2):** Every metric runs ≥200 iterations. Samples are sorted and p95 computed as `vec[(0.95 * len as f64) as usize]`.

### Metrics

#### 1. `session_spawn_latency_micros`

Time from `SessionDriver::spawn_for_message()` entry to return.

**Includes:**
- Provider CLI startup
- Settings injection
- First message acceptance

**Budget:** p95 ≤ 100,000 µs

#### 2. `message_encode_latency_micros`

Time to serialize `SessionTicket` to wire format.

**Budget:** p95 ≤ 10,000 µs

#### 3. `routing_decision_latency_micros`

Time for `RoutePolicy::select()` + `UsageEnforcement::check_limit()`.

**Budget:** p95 ≤ 5,000 µs

#### 4. `tick_once_total_latency_micros`

End-to-end latency from entry to return.

**Breakdown (in-budget estimates):**
- Steps 1-2 (snapshot + peek_lock): 50ms
- Steps 3-5 (route + enforce): 10ms
- Step 6 (spawn): 100ms
- Steps 7-14 (inject, drain, spend): 60ms
- Steps 15-17 (audit, return): 30ms
- **Total:** 250ms

**Budget:** p95 ≤ 250,000 µs

## Acceptance Criteria

| Criterion | Verification |
|-----------|--------------|
| **C.13a** | `cargo bench heartbeat` emits `session_spawn_latency_micros` with `p95 ≤ 100000` |
| **C.13b** | `message_encode_latency_micros` row with `p95 ≤ 10000` |
| **C.13c** | `routing_decision_latency_micros` row with `p95 ≤ 5000` |
| **C.13d** | `tick_once_total_latency_micros` row with `p95 ≤ 250000` |
| **C.13-p95-stability** | p95 variance ≤ 10% across 200 samples |
| **C.13-200-samples** | Every metric has `samples_count: 200` (v6 BLOCKER-2) |

## Profiling

To identify hot paths:

```bash
# Generate flamegraph (requires flamegraph installed)
cargo flamegraph -p intelligence-supervisor-app --bench heartbeat -- --profile-time 30

# Opens flamegraph.svg in browser
```

### Common Bottlenecks

1. **Snapshot iteration** (100 accounts) → optimize `AccountSnapshotProvider` with batching
2. **RoutePolicy selection** → add memoization per v6 BLOCKER-1
3. **Audit event serialization** → hand-rolled JSON faster than serde
4. **Session spawn** → measure CLI startup time separately

### Optimization Tips

```rust
// Cache AccountSnapshot between ticks (60-second memoization)
pub struct CachedSnapshotProvider {
    inner: Box<dyn AccountSnapshotProvider>,
    cache: Mutex<Option<(Instant, Vec<ProviderAccount>)>>,
    ttl_secs: u64,
}

// Hand-rolled JSON for audit events (faster than serde)
fn emit_audit_json(event: &AuditEvent) -> String {
    format!(
        r#"{{"event_id":"{}","event_class":"{}","principal":"{}"}}"#,
        event.id, event.class, event.principal
    )
}
```

## Continuous Benchmarking

The CI lane `lean-settings-drift` (part of the larger `oya-governance-*` suite) runs benchmarks on every PR:

```bash
# Manual run (mimics CI)
./scripts/bench-ci.sh \
  --baseline main \
  --head feature-branch \
  --threshold 10%  # fail if p95 regresses >10%
```

## References

- **v4 Plan § C.13a-d:** Performance budgets
- **v6 Amendments § BLOCKER-2:** Multi-sample p95 methodology
- **Benchmark harness:** `intelligence/core/supervisor-app/benches/heartbeat.rs`
- **Hardware baseline:** Measured on arm64-darwin Apple silicon (MacBook Air class)
