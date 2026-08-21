# Spec: pooling-seat-observability-otel

**Objective**: Expose the pool's health/observability surface inside
`intelligence-provider-pool-app` only, with zero new workspace members.

---

## Contracts

### HTTP surface (OpenAPI 3.2.0 summary)

| Method | Path                      | Auth        | Description                        |
|--------|---------------------------|-------------|------------------------------------|
| GET    | /healthz                  | none        | Liveness (already exists)          |
| GET    | /metrics                  | none        | Prometheus text-format metrics     |
| GET    | /internal/seats           | localhost   | Per-seat snapshot JSON             |
| POST   | /internal/seats/reload    | localhost   | Upsert-only seat reconcile         |

`/internal/*` returns HTTP 403 for any request whose peer address is not
`127.0.0.0/8`. The hyper server sees the peer address via the connection
accept loop; the handler receives a `peer_addr: SocketAddr` injected by the
router.

### GET /metrics — Prometheus text format

```
# HELP provider_pool_dispatch_attempts_total Dispatch attempt counter per account
# TYPE provider_pool_dispatch_attempts_total counter
provider_pool_dispatch_attempts_total{account_id="…",provider="…"} N

# HELP provider_pool_dispatch_successes_total Dispatch success counter per account  
# TYPE provider_pool_dispatch_successes_total counter
provider_pool_dispatch_successes_total{account_id="…"} N

# HELP provider_pool_dispatch_failures_total Dispatch failure counter per account
# TYPE provider_pool_dispatch_failures_total counter
provider_pool_dispatch_failures_total{account_id="…",retryable="true|false"} N

# HELP provider_pool_dispatch_failovers_total Failover counter
# TYPE provider_pool_dispatch_failovers_total counter
provider_pool_dispatch_failovers_total{from="…",to="…"} N

# HELP provider_pool_quarantine_transitions_total Quarantine state-change counter
# TYPE provider_pool_quarantine_transitions_total counter
provider_pool_quarantine_transitions_total{account_id="…",new_state="…"} N

# HELP provider_pool_seat_available Seat availability gauge (1=available, 0=cooldown)
# TYPE provider_pool_seat_available gauge
provider_pool_seat_available{account_id="…"} 1

# HELP provider_pool_seat_consecutive_failures Current consecutive failure count per seat
# TYPE provider_pool_seat_consecutive_failures gauge
provider_pool_seat_consecutive_failures{account_id="…"} 0
```

### GET /internal/seats — JSON schema

```json
{
  "seats": [
    {
      "provider_account_id": "sref://my-key",
      "provider": "Claude",
      "available": true,
      "cooldown_until": null,
      "consecutive_failures": 0,
      "last_error": null,
      "expires_at": null,
      "refreshing": false,
      "token_totals": {
        "requests_in_window": 0,
        "tokens_in_window": 0,
        "latency_ms_p50": 0
      }
    }
  ],
  "total": 1
}
```

### POST /internal/seats/reload — request/response

Request body: empty or `{}`.
Response (200):
```json
{ "added": 0, "updated": 1, "total": 1 }
```

---

## Mod layout (flat-clean-arch, one crate)

All code lives inside `intelligence-provider-pool-app`:

```
src/
  lib.rs         ← existing ports + adapters + dispatch use-cases
                   + new: OtelMetricsSink (mod otel section)
                   + new: SeatSnapshot, SeatRegistry port, InMemorySeatRegistry
  main.rs        ← existing composition root
                   + new: /metrics handler
                   + new: /internal/seats handler
                   + new: /internal/seats/reload handler
                   + new: localhost guard fn
tests/
  acceptance.rs  ← existing
  observability.rs ← NEW: OtelMetricsSink + admin route hermetic tests
```

---

## OtelMetricsSink design

`OtelMetricsSink` implements `MetricsSink` using `opentelemetry_sdk::SdkMeterProvider`.
It holds:
- `Counter<u64>` for attempts, successes, failures, failovers, quarantine_transitions
- No external collector required; the `SdkMeterProvider` accumulates in-memory.
- The `/metrics` handler reads the accumulated data by calling a separate
  `PrometheusTextRenderer` that walks the recorded events via the `RecordingMetricsSink`
  snapshot (simpler and zero-dep for the Prometheus text output).

Because the `opentelemetry` SDK's in-memory exporter does not produce Prometheus
text format directly without pulling in `opentelemetry-prometheus` (which would
add reqwest + tonic), we use a **dual-sink pattern**:
1. `OtelMetricsSink` wraps `RecordingMetricsSink` internally for state tracking.
2. The `/metrics` handler renders Prometheus text from the accumulated counters
   stored in an `Arc<Mutex<MetricsCounters>>` (a plain struct of atomic-friendly
   counters), which `OtelMetricsSink` updates via interior mutability.
3. This avoids pulling `opentelemetry-prometheus` or `prometheus` as new deps.

`MetricsCounters` is a new lightweight struct:
```rust
pub struct MetricsCounters {
    // per-(account_id, provider) attempt counter
    attempts: HashMap<(String, String), u64>,
    successes: HashMap<String, u64>,
    failures_retryable: HashMap<String, u64>,
    failures_non_retryable: HashMap<String, u64>,
    failovers: HashMap<(String, String), u64>,
    quarantine_transitions: HashMap<(String, String), u64>,
}
```

The `/metrics` endpoint renders this as Prometheus text without any extra deps.

---

## SeatRegistry port

```rust
pub trait SeatRegistry: Send + Sync {
    fn snapshot(&self) -> Vec<SeatSnapshot>;
    fn upsert(&mut self, seats: Vec<SeatSnapshot>) -> ReloadResult;
}
```

`InMemorySeatRegistry` stores `BTreeMap<ProviderAccountId, SeatSnapshot>`.
The `reload` handler locks the registry, computes diff (add/update), applies it,
and returns counts.

---

## Testing strategy

All tests are hermetic (no network egress, no external process):

- `OtelMetricsSink` unit tests: drive dispatch_to_pool with OtelMetricsSink,
  then call `render_prometheus_text()` and assert metric lines present.
- Admin route tests: call the handler functions directly (not via hyper server)
  with simulated peer addresses (127.0.0.1 and 192.168.1.1).
- Reload tests: seed initial seats, call reload handler, assert added/updated
  counts and that pre-existing seats are not removed.

---

## Observability / SLO

OpenSLO manifest: `microservices/intelligence/slos/providers-pool-seat-availability.openslo.yaml`
- SLI: ratio of seats in `available=true` state at any point in time.
- Objective: ≥ 1 seat available per pool (availability = not all seats quarantined).

Per ADR-0130, this manifest is mandatory before promotion past dev.

---

## Crate boundary

ALL changes stay inside:
- `intelligence/core/provider-pool-app/`
- `microservices/intelligence/slos/providers-pool-seat-availability.openslo.yaml`
- `tasks/pooling-seat-observability-otel-plan.md`
- `docs/specs/task-pooling-seat-observability-otel.md`

No new workspace member. No root `Cargo.toml` edit. No other crate touched.
