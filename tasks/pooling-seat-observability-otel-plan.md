# pooling-seat-observability-otel — Implementation Plan

Lane: `pooling` | Priority: med | Effort: M
Crate: `oya-intelligence-provider-pool-app` (ONLY crate modified)

## Requirements Analysis

### Problem
The provider-pool service currently ships `NoOpMetricsSink` (no telemetry), no
`GET /metrics` endpoint, and no internal admin surface for seat inspection or
hot reload. Operators and the Oyatie observability stack are blind to per-seat
health, cooldown windows, and failure history.

### Acceptance Criteria

1. **GET /healthz** — already exists; must remain green after all changes.
2. **GET /metrics** — Prometheus-text format; exposes per-seat OTel-bridge gauges
   (available, cooldown, consecutive_failures) and per-failure-kind counters
   (dispatch.attempts, dispatch.successes, dispatch.failures, dispatch.failovers,
   quarantine_transitions).  Metric names follow `provider_pool.<family>` naming.
3. **GET /internal/seats** — localhost-only; JSON snapshot of every seat:
   `provider, available, cooldown_until, consecutive_failures, last_error,
   expires_at, refreshing, token_totals`.
4. **POST /internal/seats/reload** — localhost-only; await in-flight refreshes,
   re-read seats from config, upsert-only reconcile (never removes existing
   seats, only adds/updates). Returns 200 JSON with seat count delta.
5. **OtelMetricsSink** — real OTel-bridge `MetricsSink` implementation alongside
   the existing `NoOpMetricsSink` / `RecordingMetricsSink`. Uses
   `opentelemetry_sdk::SdkMeterProvider` (no external collector required;
   memory exporter for tests, prometheus scrape for production).
6. **TESTS HERMETIC** — all tests are in-process, no network egress.
7. **Flat clean arch** — all changes inside `oya-intelligence-provider-pool-app`,
   no new workspace members.

### Edge Cases
- Reload during active dispatch: upsert-only; never remove a seat that is
  currently in use. Reload is idempotent.
- `/internal/*` reachable only from localhost (127.0.0.x). Non-localhost
  requests → 403.
- Metrics endpoint must not block the dispatch path; instrument recording is
  `&self` interior-mutability.
- `cooldown_until` is `null` in the JSON snapshot when the seat has no active
  cooldown.
- `last_error` is `null` when no failure has been recorded for the seat.

### K8s / Cloud-Native Implications
- `GET /metrics` is the canonical Prometheus scrape target wired by the
  ServiceMonitor/PodMonitor per ADR-0130.
- `GET /internal/seats` is operator-only; K8s NetworkPolicy should restrict
  to the node-local pod CIDR or debug tooling. The service enforces localhost
  at the HTTP layer.
- OpenSLO SLO manifest `microservices/intelligence/slos/providers-pool-seat-availability.openslo.yaml`
  is required per ADR-0130 before promotion past dev.

## Ordered Subtasks

### PHASE 1 — Plan + Spec
1. [x] Write `tasks/pooling-seat-observability-otel-plan.md`
2. [ ] Write `docs/specs/task-pooling-seat-observability-otel.md`

### PHASE 2 — New mod layout (lib.rs additions)
3. [ ] Add `mod otel_sink` in lib.rs: `OtelMetricsSink` backed by
   `opentelemetry_sdk::SdkMeterProvider` with counters + gauges per
   `MetricsSink` trait. Add to Cargo.toml deps.
4. [ ] Add `mod seat_snapshot` in lib.rs: `SeatSnapshot` value type (all fields
   for `/internal/seats`), `SeatRegistry` port trait, `InMemorySeatRegistry`
   reference adapter.
5. [ ] Add `mod admin_routes` in main.rs: localhost guard, `/internal/seats`
   handler, `/internal/seats/reload` handler, `/metrics` handler.

### PHASE 3 — Tests (red)
6. [ ] Write hermetic tests in `tests/acceptance.rs` (or a new
   `tests/observability.rs`) for:
   - `OtelMetricsSink` records correct counter increments
   - `/internal/seats` returns per-seat snapshot JSON
   - `/internal/seats/reload` upserts without removing existing seats
   - `/metrics` returns Prometheus-text with expected metric names
   - localhost-only guard rejects non-127.0.0.1 requests

### PHASE 4 — Build (green)
7. [ ] Implement `OtelMetricsSink` in lib.rs `mod otel` section.
8. [ ] Implement `SeatSnapshot` + `SeatRegistry` port + `InMemorySeatRegistry`.
9. [ ] Wire `GET /metrics`, `GET /internal/seats`, `POST /internal/seats/reload`
   into `build_app` (main.rs), gated behind localhost guard.
10. [ ] Update `AppState` to carry `Arc<RwLock<InMemorySeatRegistry>>` and
    `OtelMetricsSink` (or a `NoOpMetricsSink` default for tests).
11. [ ] `cargo check -p oya-intelligence-provider-pool-app --all-targets` → green.
12. [ ] `cargo nextest run -p oya-intelligence-provider-pool-app` → green.

### PHASE 5 — Review + Simplify
13. [ ] Multi-axis review: correctness, security, cloud-native, perf.
14. [ ] Simplify: guard clauses, dead code, naming.

### PHASE 6 — OpenSLO + Ship
15. [ ] Write `microservices/intelligence/slos/providers-pool-seat-availability.openslo.yaml`.
16. [ ] Git commit, push, open PR.
