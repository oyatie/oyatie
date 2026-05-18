---
ip_id: IP-006
ip_status: ready
slice_owner: ops-finops
authored: 2026-05-18
slice: finops-portal/tenant-billing-presentation/app
related_adrs: [ADR-0131, ADR-0186, ADR-0199]
depends_on: [IP-001, IP-002, IP-004, IP-005]
target_lines: 150
---

# IP-006 — `tenant-billing-presentation` app (binary) slice

## Why this slice

The app tier is the deployable binary
(`oya-finops-portal-tenant-billing-presentation-app`) that composes
all upstream slices into a running server. It owns:

- Process wiring: tokio runtime, signal handling, graceful shutdown.
- Adapter selection: Postgres / Mimir / audit-chain HTTP clients.
- Observability: OpenTelemetry tracer init, Prometheus exposer,
  log exporter.
- Health endpoints: `/live`, `/ready`.
- Config loading: `OYA_FINOPS_PORTAL_*` env vars + `config.toml`
  overlay via the shared `oya-config-loader`.

The app tier is the only crate in this BC that imports `tokio`,
runtime config, or process-level wiring. Lower tiers are libraries.

## Acceptance criteria

1. New binary crate
   `crates/oya-finops-portal-tenant-billing-presentation-app/`.
2. Entry point:
   ```rust
   #[tokio::main(flavor = "multi_thread")]
   async fn main() -> Result<()> {
       let cfg = Config::from_env_and_files()?;
       let _otel = init_tracing(&cfg)?;
       let state = AppState::wire(&cfg).await?;
       let router = oya_finops_portal_tenant_billing_presentation_api::router(state);
       serve_with_shutdown(router, cfg.listen, shutdown_signal()).await
   }
   ```
3. `Config` struct hosts:
   - `listen: SocketAddr`.
   - `postgres_url: SecretString`.
   - `audit_chain_endpoint: Url`.
   - `opencost_endpoint: Url`.
   - `mimir_endpoint: Url`.
   - `cedar_policy_dir: PathBuf`.
   - `otel_endpoint: Option<Url>`.
4. `/live` returns `200` if the process is alive.
5. `/ready` returns `200` only when:
   - DB connection pool has at least one free connection,
   - audit-chain endpoint last health check succeeded (cached 10s),
   - Cedar policies loaded successfully.
6. Graceful shutdown on `SIGTERM` / `SIGINT`: drain inflight, then
   exit with code 0. Time-bound at 30 s.
7. Process emits cost-attribution labels via the
   `oya.tenantCostLabels` Helm helper (already covered by IP-003).
8. `cargo run -p oya-finops-portal-tenant-billing-presentation-app
   -- --check-config` returns 0 with a valid config.

## File-level work plan

1. `Cargo.toml`.
2. `src/main.rs` — entrypoint.
3. `src/config.rs` — `Config` struct + loader.
4. `src/state.rs` — `AppState::wire` (instantiate adapters).
5. `src/observability.rs` — OTel + Prom init.
6. `src/health.rs` — live / ready endpoints.
7. `src/shutdown.rs` — signal handling.

## Observability wiring (ADR-0186)

- Logs: JSON to stdout via `tracing-subscriber` with
  `tracing-opentelemetry` layer.
- Metrics: `prometheus` registry exposed on
  `:9090/metrics` (separate port from API).
- Traces: OTLP/HTTP to the configured collector.
- Process exposes the canonical SLI metric families declared in
  IP-003 ServiceMonitor section.

## Adapter selection

The app wires these concrete adapters into the usecase traits:

| Usecase trait                     | Adapter crate                                                |
|-----------------------------------|--------------------------------------------------------------|
| `TenantInvoiceRepository`         | `oya-finops-portal-tenant-billing-presentation-adapter-postgres` |
| `CostDataSource`                  | `oya-finops-portal-tenant-billing-presentation-adapter-opencost` |
| `CreditLedger`                    | `oya-finops-portal-credit-ledger-adapter-postgres`           |
| `AuditEmitter`                    | `oya-audit-chain-client-http`                                |

Each adapter crate is named under BNF v4.1 and the BNF YAML adds an
adapter entry for each one (separate registration, but tracked here).

## Risk + mitigation

- **Risk**: long-poll connections leak on shutdown. **Mitigation**:
  the shutdown future closes the listener, waits up to 30s for
  inflight requests, then aborts; tested via an integration test
  that sends a slow request while triggering shutdown.
- **Risk**: secrets logged via `Debug`. **Mitigation**: `Config`
  uses `secrecy::SecretString` for credentials; redacted in `Debug`.

## Cost attribution at the process layer

The process exports the `cost_center=infra-finops-portal` and
`workload_class=app` labels on every Prometheus metric series via
the `prometheus::Registry` configured with constant labels read from
env (`OYA_FINOPS_PORTAL_COST_CENTER`, etc.). These map 1:1 to the
Helm `costAttribution.*` values from IP-003.

## Out-of-scope

- Cost-allocation policy BC binary — separate app crate later.
- Anomaly-explanation BC binary — separate.

## References

- ADR-0186 — observability backplane.
- ADR-0199 — cost-attribution canonical.

## Verification

- `cargo run -p oya-finops-portal-tenant-billing-presentation-app
  -- --check-config` returns 0.
- Integration test: `tests/graceful_shutdown.rs` confirms 30s drain.
- `oya gate observability-wired --crate
  oya-finops-portal-tenant-billing-presentation-app` green.
