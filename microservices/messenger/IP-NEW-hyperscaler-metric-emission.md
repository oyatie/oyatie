---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-cloud-saas-platform-substrate
phase: P14-messenger
impl_plan_id: IP-NEW-hyperscaler-metric-emission
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-connect-messenger + axis-observability
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, hyperscaler-arch-invariants]
related_adrs:
  - ADR-0064
  - ADR-0128
  - ADR-0131
  - ADR-0139
related_crates:
  - oya-shared-hyperscaler-metrics-kernel
  - oya-shared-hyperscaler-metrics-adapter-prometheus
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (per ADR-0064) -->

# IP-NEW: messenger µservice — wire HyperscalerMetrics trait at every canonical emission site

## Intent

Integrate `oya-shared-hyperscaler-metrics-kernel::HyperscalerMetrics`
into messenger so the canonical PrometheusRule
`hyperscaler-invariants-canonical-prometheusrule.yaml` sees live
metric series for messenger. Messenger is the **hot-path** µservice
(highest cross-µservice traffic per
`docs/standards/cross-microservice-latency-budget.md` Flow B) and the
canonical pilot site for the trait integration pattern.

## ChangeSet boundary

- Add `oya-shared-hyperscaler-metrics-kernel` and
  `oya-shared-hyperscaler-metrics-adapter-prometheus` as workspace
  dependencies of every messenger crate that touches the request /
  capability / response surfaces.
- Wire the adapter at the messenger composition root (the
  `oya-messenger-app` crate).
- Pass `Arc<dyn HyperscalerMetrics>` into the four messenger BCs
  that need to emit:
  - channel-store (HTTP REST + WS request handlers)
  - message-stream (capability execution loop)
  - presence (rate-limit gate)
  - thread-tree (notification fan-out)
- Author per-site call patterns; verify zero alerts fire absent-series
  after the change lands.

## Concrete file targets

| Path | Action |
|---|---|
| `microservices/messenger/src/Cargo.toml` (workspace dep section) | edit — add the two kernel/adapter crates |
| `microservices/messenger/src/crates/oya-messenger-app/Cargo.toml` | edit — depend on kernel + prometheus-adapter |
| `microservices/messenger/src/crates/oya-messenger-app/src/main.rs` | edit — construct adapter, share via Arc |
| `microservices/messenger/src/crates/oya-messenger-channel-store-app/src/handler.rs` | edit — call `record_responses_total/_5xx/_429` |
| `microservices/messenger/src/crates/oya-messenger-message-stream-app/src/capability.rs` | edit — call `record_capability_circuit_state/_retry_budget_exhausted` |
| `microservices/messenger/src/crates/oya-messenger-presence-app/src/ratelimit.rs` | edit — call `record_responses_429` |
| `microservices/messenger/src/crates/oya-messenger-thread-tree-app/src/notify.rs` | edit — call `record_request_success/_total` |
| `microservices/messenger/dashboards/hyperscaler-invariants.json` | create — Grafana dash referencing the seven metric families |

## Code shape

The composition root constructs ONE adapter and shares it:

```rust
// microservices/messenger/src/crates/oya-messenger-app/src/main.rs
use std::sync::Arc;

use oya_shared_hyperscaler_metrics_kernel::{
    HyperscalerMetrics, MetricsContext,
};
use oya_shared_hyperscaler_metrics_adapter_prometheus::PrometheusHyperscalerMetrics;
use prometheus::Registry;

fn build_metrics(registry: &Registry) -> Arc<dyn HyperscalerMetrics> {
    let ctx = MetricsContext::new("messenger").expect("canonical slug");
    let adapter = PrometheusHyperscalerMetrics::register(registry, ctx)
        .expect("metrics register cleanly at startup");
    Arc::new(adapter)
}
```

Each BC accepts the trait object:

```rust
// channel-store HTTP handler — emits responses_total + responses_5xx
use oya_shared_hyperscaler_metrics_kernel::HyperscalerMetrics;
use std::sync::Arc;

pub struct ChannelStoreHandler {
    metrics: Arc<dyn HyperscalerMetrics>,
    // …
}

impl ChannelStoreHandler {
    pub async fn handle(&self, req: HttpRequest) -> HttpResponse {
        let _ = self.metrics.record_responses_total();
        let _ = self.metrics.record_request_total();
        match self.execute(req).await {
            Ok(resp) => {
                let _ = self.metrics.record_request_success();
                resp
            }
            Err(e) if e.is_rate_limit() => {
                let _ = self.metrics.record_responses_429(&e.tenant_id());
                HttpResponse::too_many_requests()
            }
            Err(e) if e.is_server_error() => {
                let _ = self.metrics.record_responses_5xx();
                HttpResponse::internal_server_error()
            }
            Err(_) => HttpResponse::bad_request(),
        }
    }
}
```

The capability execution loop is the canonical emission site for
circuit-breaker state:

```rust
// message-stream capability execution
use oya_shared_hyperscaler_metrics_kernel::{CircuitState, HyperscalerMetrics};

pub async fn run_capability(
    metrics: Arc<dyn HyperscalerMetrics>,
    cap: &Capability,
    breaker: &CircuitBreaker,
) -> Result<()> {
    let _ = metrics.record_capability_circuit_state(cap.id(), breaker.state().into());
    match cap.execute(breaker).await {
        Ok(_) => Ok(()),
        Err(e) if e.retry_budget_exhausted() => {
            let _ = metrics.record_capability_retry_budget_exhausted(cap.id());
            Err(e)
        }
        Err(e) => Err(e),
    }
}
```

`From<BreakerState> for CircuitState` lives in the BC crate to keep
the kernel pure.

## Acceptance gates

```bash
cargo check  -p oya-messenger-app
cargo nextest run -p oya-messenger-app
cargo run -p oya-dev-cli -- gate validate hyperscaler-arch-invariants
# Mock-scrape messenger's /metrics endpoint and assert all 7 canonical
# families appear under steady-state traffic.
```

## Test plan

| Test | Verifies |
|---|---|
| `test_metrics_register_at_startup` | adapter constructs cleanly with canonical slug "messenger" |
| `test_429_emits_with_tenant_label` | per-tenant 429 increments |
| `test_circuit_breaker_state_flip` | open → closed transition flips the gauge correctly |
| `test_request_success_only_on_2xx_within_budget` | SLI numerator semantics |
| `test_all_seven_families_present_under_load` | scrape mock prometheus registry |

## Rollout

1. Land the kernel + adapter crates (already authored at
   `crates/oya-shared-hyperscaler-metrics-kernel` and
   `crates/oya-shared-hyperscaler-metrics-adapter-prometheus`).
2. Land this IP at messenger; observe canonical PrometheusRule
   alerts fire on real conditions (not absent-series) for 2 weeks.
3. Generalize to social, audit-chain, ontology, notification (Flow A
   participants) under follow-on IPs per µservice.
4. Generalize to all 17 first-wave µservices under a single fitness
   lane `oya-governance-hyperscaler-metric-emission` that asserts
   every µservice's `app` crate constructs an adapter at composition
   root.

## Halt conditions

- Adapter construction fails at startup → halt rollout; investigate
  prometheus registry pre-existing entries.
- Mock-scrape test shows missing families → fix wiring in the BC that
  owns the missing family BEFORE landing.

## References

- ADR-0064 — canonical-base-and-localization-packs.
- ADR-0128 — hyperscaler architecture invariants.
- ADR-0131 — per-microservice flat layout.
- ADR-0139 — agentic SLO-gated promotion.
- `crates/oya-shared-hyperscaler-metrics-kernel`.
- `crates/oya-shared-hyperscaler-metrics-adapter-prometheus`.
- `microservices/observability/contracts/metric-naming-convention.md`.
- `docs/standards/cross-microservice-latency-budget.md` Flow B.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/messenger/IP-NEW-hyperscaler-metric-emission.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), KR-PIPA-2023-amendment(14400s/900s), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/messenger/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/messenger/slos/attachment-scan-freshness.openslo.yaml`, `microservices/messenger/slos/mention-fanout.openslo.yaml`, `microservices/messenger/slos/message-send-availability.openslo.yaml`, `microservices/messenger/slos/message-send-latency.openslo.yaml`, `microservices/messenger/policy/auditor-scope.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/messenger/IP-NEW-hyperscaler-metric-emission.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/messenger/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: eligible only when ADR-0344 D-9 compliance-pack exclusions do not bar deferral; otherwise the Cedar scheduler rejects delay while still emitting carbon fields.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
