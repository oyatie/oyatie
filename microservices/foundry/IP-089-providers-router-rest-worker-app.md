---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-provider-adapter-substrate
impl_plan_id: IP-014-router-rest-worker-app
status: pending
execution_unit: ChangeSet
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, layer-correctness, integration-test]
---

# IP-014: router-rest + router-worker + router-app (composition root)

## Intent

Three crates: the REST surface (`-rest`), the health-monitor + cost-roll-up worker (`-worker`), and the composition-root binary (`-app`) that wires usecase + adapter + every per-vendor adapter into a runnable service.

## File Targets

### `oya-foundry-providers-router-rest`

| Path | Action |
|---|---|
| `.../Cargo.toml` | create — `axum`, `tower`, OIDC + Cedar middleware deps |
| `.../src/lib.rs` | create |
| `.../src/handlers/decide.rs` | create — POST /router/decide |
| `.../src/handlers/invoke.rs` | create — POST /router/invoke |
| `.../src/handlers/health.rs` | create — GET /providers/health |
| `.../src/handlers/capabilities.rs` | create — GET /providers/capabilities |
| `.../src/handlers/tenant_config.rs` | create — GET/PUT /providers/config/{tenant} |
| `.../src/middleware/oidc.rs` | create |
| `.../src/middleware/cedar.rs` | create |
| `.../src/middleware/spiffe.rs` | create |

### `oya-foundry-providers-router-worker`

| Path | Action |
|---|---|
| `.../Cargo.toml` | create |
| `.../src/lib.rs` | create |
| `.../src/health_monitor.rs` | create — per-provider rolling-window SLI scrape; emits to Mimir |
| `.../src/cost_rollup.rs` | create — per-tenant per-day cost roll-up; emits ceiling-breach events |
| `.../src/event_emitter.rs` | create — `ProviderInvoked` + `RouterDecided` + `CredentialResolved` to NATS |
| `.../src/demote_recover.rs` | create — drives provider-router demote/recover based on health |

### `oya-foundry-providers-router-app`

| Path | Action |
|---|---|
| `.../Cargo.toml` | create |
| `.../src/main.rs` | create — wires every crate via DI |
| `.../src/config.rs` | create — env + file config |
| `.../src/telemetry.rs` | create — OTel init |
| `.../src/signals.rs` | create — graceful shutdown |

## Test Plan

| Test | Verifies |
|---|---|
| `tests/integration/router_rest_decide_happy_path.rs` | REST end-to-end |
| `tests/integration/router_rest_oidc_unauthenticated_denied.rs` | OIDC middleware |
| `tests/integration/router_rest_cedar_cross_tenant_denied.rs` | Cedar middleware |
| `tests/integration/worker_health_monitor_demotes_on_unavailability` | health-monitor logic |
| `tests/integration/worker_cost_rollup_ceiling_breach_emits_event` | cost-rollup |
| `tests/load/router_decision.rs` | router decision p99 ≤ 5 ms over 100K decisions |
| `tests/integration/end_to_end_provider_invoke_emits_signed_envelope` | full path |

## Acceptance Gates

Standard + load test.

## Next IP

[`IP-015-router-sdk.md`](IP-015-router-sdk.md)
