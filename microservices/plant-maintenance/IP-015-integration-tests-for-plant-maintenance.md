---
doc_class: ImplementationPlan
ip_id: IP-015
microservice: plant-maintenance
related_adrs: [ADR-0105, ADR-0131, ADR-0244, ADR-0252, ADR-0253, ADR-0257, ADR-0263, ADR-0294, ADR-0297, ADR-0314, ADR-0315, ADR-0329, ADR-0330, ADR-0331]
journey_ref: j208-asset-lifecycle-and-maintenance
sap_submodule: End-to-end test set covering SAP S/4HANA PM-EAM, PM-PRM, PM-WOC, PM-MRP, PM-WCM submodule lifecycles
service_surface: substrate
persona: integration-engineer, quality-engineer, maya-okafor (reliability)
status: Accepted
date: 2026-05-20
owner_team: axis-plant-maintenance + axis-erp-parity + axis-qa
planned_enforcement_ref: oya-governance-plant-maintenance-doc-set
---

# IP-015: Integration tests for `plant-maintenance` — End-to-end, saga, multi-region

## A. Intent

Implements the **integration test pyramid** for plant-maintenance: contract-tests (OpenAPI / AsyncAPI / proto schema conformance), end-to-end use-case tests (full stack with real Postgres + Kafka), saga tests (cross-µservice with test-doubles for inventory + identity + finops + permit), policy-soak tests (Cedar 60s soak per ADR-0294), multi-region tests, and chaos tests.

Test taxonomy follows the **Test Pyramid** (Mike Cohn) + the **Five-Test-Type model** (Spotify): unit / property / contract / integration / end-to-end. Coverage floors per ADR-0063 doc-rigor: ≥85% line, ≥75% branch, ≥95% on Cedar-gated paths.

Industry-precedent equivalents: Stripe API integration test set shape, AWS CloudFormation TestKit, Kubernetes e2e test harness, Spring Boot test-slice taxonomy.

### A.1 Why the integration test set is non-trivial

1. **Saga tests need test-doubles.** Inventory / identity / finops / permit-to-work must be doubled with deterministic behavior; chaos-mode test-doubles inject latency + failure to exercise compensation paths.
2. **Cedar policy soak.** Per ADR-0294, every Cedar fragment changes go through a ≥60s soak window; integration test runs the soak in a compressed simulation (`cedar_test_clock`).
3. **Multi-region tests.** Cell pair (Tier-1 US-east + Tier-1 EU-west) replicas must converge; test asserts within HLC bound.
4. **Backwards-compat tests.** Every PR runs prior-version OpenAPI fixtures against new code; v1 surface MUST remain compatible for 6 months minimum.
5. **Performance smoke.** Per use-case, p99 SLO target asserted in integration suite with k6 + Vegeta load.
6. **Failure-injection chaos.** ToxiProxy for network failures; pumba for container chaos; verified per IP failure-mode catalog.

## B. Acceptance criteria

- **AC-1:** Contract conformance tests assert OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3 schema present + lints clean.
- **AC-2:** End-to-end happy path test for each IP-007..012 use-case (24 use-cases total).
- **AC-3:** Saga tests for `LinkSerial`, `ReserveComponents`, `CreateWorkOrder` (with reservation), `CancelWorkOrder` (compensating).
- **AC-4:** Cedar soak test simulates 60s soak window via `cedar_test_clock`.
- **AC-5:** Multi-region replica test (cell pair US-east + EU-west) asserts HLC convergence.
- **AC-6:** Backwards-compat test runs `reference/openapi-v1-2026-05-01.yaml` against latest code.
- **AC-7:** Performance smoke asserts p99 SLO from each IP's §D-9 / §D-8.
- **AC-8:** Failure-injection tests cover ≥4 named scenarios per IP failure-mode catalog.
- **AC-9:** Coverage: ≥85% line, ≥75% branch, ≥95% on Cedar-gated paths.
- **AC-10:** All tests deterministic (no flakes); deterministic clock via `tokio::time::pause()`; deterministic UUIDs via fixed seed.

## C. Verification

```bash
cargo test -p oya-plant-maintenance-integration-tests -- contract::openapi_lint
cargo test -p oya-plant-maintenance-integration-tests -- contract::asyncapi_lint
cargo test -p oya-plant-maintenance-integration-tests -- contract::proto_lint
cargo test -p oya-plant-maintenance-integration-tests -- e2e::equipment_create
cargo test -p oya-plant-maintenance-integration-tests -- e2e::plan_publish_critical_dual_approver
cargo test -p oya-plant-maintenance-integration-tests -- e2e::wo_full_lifecycle_pm
cargo test -p oya-plant-maintenance-integration-tests -- e2e::wo_full_lifecycle_breakdown
cargo test -p oya-plant-maintenance-integration-tests -- e2e::reservation_with_kit
cargo test -p oya-plant-maintenance-integration-tests -- e2e::dispatch_offer_accept_complete
cargo test -p oya-plant-maintenance-integration-tests -- e2e::downtime_open_close_cost_impute
cargo test -p oya-plant-maintenance-integration-tests -- e2e::oee_rollup_plant_subtree
cargo test -p oya-plant-maintenance-integration-tests -- saga::link_serial_compensation
cargo test -p oya-plant-maintenance-integration-tests -- saga::create_wo_with_reservation_saga
cargo test -p oya-plant-maintenance-integration-tests -- saga::cancel_wo_compensates
cargo test -p oya-plant-maintenance-integration-tests -- policy::cedar_soak_60s_simulated
cargo test -p oya-plant-maintenance-integration-tests -- multi_region::hlc_convergence
cargo test -p oya-plant-maintenance-integration-tests -- backwards_compat::v1_reference
cargo test -p oya-plant-maintenance-integration-tests -- perf::wo_create_p99
cargo test -p oya-plant-maintenance-integration-tests -- chaos::pg_pool_exhaustion
cargo test -p oya-plant-maintenance-integration-tests -- chaos::kafka_unreachable
cargo test -p oya-plant-maintenance-integration-tests -- chaos::inventory_grpc_500ms_latency
```

## D. Detailed mechanics

### D-1. Test harness layout

```
microservices/plant-maintenance/tests/
  contract/
    openapi_lint.rs
    asyncapi_lint.rs
    proto_lint.rs
    fixtures/
      reference/openapi-v1-2026-05-01.yaml
  e2e/
    equipment_create.rs
    plan_publish_critical_dual_approver.rs
    wo_full_lifecycle_pm.rs
    wo_full_lifecycle_breakdown.rs
    reservation_with_kit.rs
    dispatch_offer_accept_complete.rs
    downtime_open_close_cost_impute.rs
    oee_rollup_plant_subtree.rs
  saga/
    link_serial_compensation.rs
    create_wo_with_reservation_saga.rs
    cancel_wo_compensates.rs
  policy/
    cedar_soak_60s_simulated.rs
    cedar_principal_change_audit.rs
  multi_region/
    hlc_convergence.rs
    cell_failover.rs
  backwards_compat/
    v1_reference.rs
  perf/
    wo_create_p99.rs
    dispatch_offer_p95.rs
    oee_refresh_p99.rs
  chaos/
    pg_pool_exhaustion.rs
    kafka_unreachable.rs
    inventory_grpc_500ms_latency.rs
    identity_circuit_open.rs
    finops_settlement_lag.rs
  helpers/
    test_doubles.rs     -- in-process doubles for inventory/identity/permit/finops
    chaos_proxy.rs       -- toxiproxy wrapper
    cedar_test_clock.rs
    deterministic_uuid.rs
```

### D-2. End-to-end test (`wo_full_lifecycle_pm.rs`)

```rust
#[tokio::test]
async fn wo_full_lifecycle_pm() {
    let harness = TestHarness::new_pg_and_kafka().await;
    let ctx = test_context(&harness, "acme");

    // 1. Create equipment + floc
    let eq = harness.rest_post::<EquipmentRef>("/v1/tenants/acme/equipment", &create_eq_payload(&ctx)).await.unwrap();
    // 2. Create maintenance plan
    let plan = harness.rest_post::<PlanRef>("/v1/tenants/acme/maintenance-plans", &create_plan_payload(&eq)).await.unwrap();
    // 3. Activate plan
    harness.rest_post::<PlanRef>(&format!("/v1/tenants/acme/maintenance-plans/{}/activate", plan.plan_id), &json!({})).await.unwrap();
    // 4. Wait for deadline monitor to fire
    harness.advance_clock(Duration::days(31)).await;
    harness.tick_deadline_monitor().await;
    let event: PlanDueEvent = harness.assert_kafka_emit("plant-maintenance.plan.due.v1").await;
    // 5. Tasks µservice creates WO from plan.due event (test double)
    let wo = harness.create_wo_from_plan_due(&event).await;
    // 6. Release WO
    harness.rest_post::<WoRef>(&format!("/v1/tenants/acme/work-orders/{}/release", wo.wo_id), &json!({})).await.unwrap();
    // 7. Confirm operation
    for op in &wo.operations {
        harness.rest_post::<ConfirmRef>(&format!("/v1/tenants/acme/work-orders/{}/operations/{}/confirm", wo.wo_id, op.op_no),
                                        &confirm_payload(op)).await.unwrap();
    }
    // 8. TECO
    harness.rest_post::<WoRef>(&format!("/v1/tenants/acme/work-orders/{}/teco", wo.wo_id), &json!({})).await.unwrap();
    // 9. Settlement event from finops test double
    harness.emit_finops_settled(&wo).await;
    // 10. Close
    harness.rest_post::<WoRef>(&format!("/v1/tenants/acme/work-orders/{}/close", wo.wo_id), &json!({})).await.unwrap();

    // Assertions
    let final_state = harness.rest_get::<WorkOrder>(&format!("/v1/tenants/acme/work-orders/{}", wo.wo_id)).await.unwrap();
    assert_eq!(final_state.state, WoState::Clsd);
    let audit_events = harness.audit_events_for(&wo.wo_id);
    assert!(audit_events.contains_class("EVT-PLANT_MAINTENANCE-WORK_ORDER-CREATED"));
    assert!(audit_events.contains_class("EVT-PLANT_MAINTENANCE-WORK_ORDER-RELEASED"));
    assert!(audit_events.contains_class("EVT-PLANT_MAINTENANCE-WORK_ORDER-OPERATION_CONFIRMED"));
    assert!(audit_events.contains_class("EVT-PLANT_MAINTENANCE-WORK_ORDER-TECO"));
    assert!(audit_events.contains_class("EVT-PLANT_MAINTENANCE-WORK_ORDER-CLOSED"));
}
```

### D-3. Saga compensation test (`saga::cancel_wo_compensates`)

```rust
#[tokio::test]
async fn cancel_wo_compensates_reservation() {
    let harness = TestHarness::new_pg_and_kafka().await;
    let ctx = test_context(&harness, "acme");
    let wo = harness.create_wo_with_reservation(&ctx).await;

    // Pre-condition: reservation present
    let res_before = harness.inv_double.reservation(&wo.reservation_id).await.unwrap();
    assert!(matches!(res_before.state, ReservationState::Active));

    // Cancel WO
    harness.rest_post::<()>(&format!("/v1/tenants/acme/work-orders/{}/cancel", wo.wo_id), &json!({})).await.unwrap();

    // Assertions
    let wo_after = harness.rest_get::<WorkOrder>(&format!("/v1/tenants/acme/work-orders/{}", wo.wo_id)).await.unwrap();
    assert_eq!(wo_after.state, WoState::Dlt);
    let res_after = harness.inv_double.reservation(&wo.reservation_id).await.unwrap();
    assert!(matches!(res_after.state, ReservationState::Cancelled));
    let audit_events = harness.audit_events_for(&wo.wo_id);
    assert!(audit_events.contains_class("EVT-PLANT_MAINTENANCE-WO_USECASE-CANCEL_COMPENSATED"));
}
```

### D-4. Cedar soak test (`policy::cedar_soak_60s_simulated`)

```rust
#[tokio::test]
async fn cedar_soak_60s_simulated() {
    let harness = TestHarness::new_pg_and_kafka().await;
    let clock = harness.cedar_test_clock();
    let new_bundle = harness.load_bundle("fixtures/policy/2026.05.20-r4.cedar");

    harness.cedar_publish_with_soak(&new_bundle, Duration::seconds(60)).await;
    // During soak: old bundle still in force
    clock.advance(Duration::seconds(30)).await;
    let dec = harness.cedar_eval(&sample_request()).await;
    assert_eq!(dec.bundle_version, "2026.05.20-r3");
    // After soak: new bundle in force
    clock.advance(Duration::seconds(35)).await;
    let dec = harness.cedar_eval(&sample_request()).await;
    assert_eq!(dec.bundle_version, "2026.05.20-r4");
}
```

### D-5. Multi-region replica test

```rust
#[tokio::test]
async fn hlc_convergence_us_east_eu_west() {
    let harness = MultiRegionHarness::new(["us-east-1", "eu-west-1"]).await;
    let ctx = test_context_us(&harness, "acme");

    // Write in us-east
    let eq = harness.us_east().rest_post::<EquipmentRef>("/v1/tenants/acme/equipment", &create_eq_payload(&ctx)).await.unwrap();

    // Replicate
    harness.replicate_all().await;
    harness.advance_replication_clock(Duration::seconds(5)).await;

    // Read in eu-west
    let eu_eq = harness.eu_west().rest_get::<Equipment>(&format!("/v1/tenants/acme/equipment/{}", eq.equipment_id)).await.unwrap();
    assert_eq!(eu_eq.equipment_id, eq.equipment_id);
    assert!(eu_eq.hlc >= eq.hlc);
}
```

### D-6. Performance smoke

```rust
#[tokio::test]
async fn wo_create_p99_under_slo() {
    let harness = TestHarness::new_pg_and_kafka().await;
    let ctx = test_context(&harness, "acme");
    let mut hist = hdrhistogram::Histogram::<u64>::new(3).unwrap();
    for _ in 0..1000 {
        let start = Instant::now();
        let _ = harness.rest_post::<WoRef>("/v1/tenants/acme/work-orders", &create_wo_payload_no_parts(&ctx)).await.unwrap();
        hist.record(start.elapsed().as_micros() as u64).unwrap();
    }
    assert!(hist.value_at_quantile(0.99) <= 160_000, "p99 over budget: {}μs", hist.value_at_quantile(0.99));
}
```

### D-7. Chaos test (PG pool exhaustion)

```rust
#[tokio::test]
async fn chaos_pg_pool_exhaustion() {
    let harness = TestHarness::with_pg_max_connections(2).await;   // intentionally tiny
    let ctx = test_context(&harness, "acme");
    let mut handles = Vec::new();
    for _ in 0..50 {
        handles.push(tokio::spawn({ let h = harness.clone(); let c = ctx.clone();
            async move { h.rest_post::<EquipmentRef>("/v1/tenants/acme/equipment", &create_eq_payload(&c)).await }
        }));
    }
    let results: Vec<_> = futures::future::join_all(handles).await;
    let successes = results.iter().filter(|r| r.as_ref().map_or(false, |r| r.is_ok())).count();
    let circuit_opens = harness.metric("pm_pg_circuit_open_total").await;
    assert!(successes > 0, "some requests should succeed");
    assert!(circuit_opens > 0, "circuit breaker should engage under exhaustion");
}
```

### D-8. Coverage targets

```toml
# coverage.toml
line_floor   = 85
branch_floor = 75
cedar_path_floor = 95
exclude = ["build.rs", "src/main.rs"]
```

### D-9. SLO assertions per IP

| IP | SLO asserted | Test |
|---|---|---|
| IP-001/007 | CreateEquipment p99 ≤ 65ms | `perf::equipment_create_p99` |
| IP-002/008 | OnCompletion p99 ≤ 160ms | `perf::on_completion_p99` |
| IP-003/009 | WO create (saga) p99 ≤ 380ms | `perf::wo_create_saga_p99` |
| IP-004/010 | Reserve (kit 8) p99 ≤ 580ms | `perf::reserve_kit_p99` |
| IP-005/011 | Dispatch offer p99 ≤ 320ms | `perf::dispatch_offer_p99` |
| IP-006/012 | OEE refresh (subtree) p99 ≤ 7s | `perf::oee_refresh_subtree_p99` |
| IP-014 | REST GET equipment p99 ≤ 42ms | `perf::rest_load_equipment_p99` |

### D-10. Audit-event registry (test-set-emitted)

| Event class | Severity | Emitter |
|---|---|---|
| `EVT-PLANT_MAINTENANCE-TEST-COVERAGE_BELOW_FLOOR` | warning | CI lane |
| `EVT-PLANT_MAINTENANCE-TEST-PERF_SLO_VIOLATED` | warning | CI lane |
| `EVT-PLANT_MAINTENANCE-TEST-CHAOS_REGRESSION_DETECTED` | warning | CI lane |

### D-11. Failure modes & recovery

1. **`FlakeRate`** — flaky test threshold > 0.5% triggers CI block. Runbook `runbooks/test-flake.md`.
2. **`CoverageDrop`** — coverage below 85% line / 75% branch / 95% Cedar fails CI. Runbook `runbooks/coverage-drop.md`.
3. **`SLOViolated`** — perf-smoke fails. Block merge; investigate hot path. Runbook `runbooks/slo-perf-violation.md`.
4. **`ContractDrift`** — OpenAPI / AsyncAPI / proto schema drift detected by `oasdiff`/`buf breaking`. Block merge. Runbook `runbooks/contract-drift.md`.
5. **`MultiRegionConvergenceTimeout`** — replication lag > 30s. Investigation. Runbook `runbooks/multi-region-lag.md`.
6. **`BackwardsCompatBroken`** — v1 fixture fails against new code. Block merge unless major-version ADR landed. Runbook `runbooks/backwards-compat-broken.md`.

### D-12. Migration notes

Test harness is itself a Rust crate `oya-plant-maintenance-integration-tests`; lives outside the main µservice crate set; depends on it via dev-dependencies.

### D-13. Cross-µservice handoffs

Integration tests use test-doubles for inventory-management, identity, workplace-integration, production-planning, oya-cloud-finops, permit-to-work. Each double is in `helpers/test_doubles.rs` with a deterministic API; replaceable with real µservices in nightly e2e cluster runs.

## E. Failure-mode summary

See D-11.

## F. Migration / rollback

Tests are forward-only; flake-quarantine is a 24h grace period before block. New tests must land alongside the code they cover (ADR-0250 build-ahead-of-certification).

## G. References

- ADR-0063 (doc-coverage), ADR-0105, ADR-0244, ADR-0250, ADR-0263, ADR-0294, ADR-0297, ADR-0314..0316.
- Mike Cohn, *Succeeding with Agile* — Test Pyramid.
- Spotify five-test-type model.
- Stripe API test design notes.
- ToxiProxy, pumba, k6, Vegeta tooling documentation.

## H. Out of scope

- Domain (IPs 001-006), use-cases (IPs 007-012), adapters (IP-013), delivery surfaces (IP-014).

— end IP-015 —
