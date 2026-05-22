---
doc_class: ImplementationPlan
microservice: production-planning
status: Accepted
date: 2026-05-20
owner_team: axis-production-planning + axis-erp-parity
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0244, ADR-0252, ADR-0253, ADR-0257, ADR-0263, ADR-0294, ADR-0297, ADR-0314, ADR-0315, ADR-0316]
planned_enforcement_ref: oya-governance-production-planning-doc-suite
ip_id: IP-015
journey_ref: j101
journey_slug: j101-multi-tier-supply-chain-formation
sap_submodule: Cross-cut — PP-BD/PP-CRP/PP-MRP/PP-SFC end-to-end scenarios mapped from SAP test catalogue
tenant_class: substrate
persona: qa-engineer + integration-engineer
---

# IP-015: Integration tests for production-planning

## A. Intent

Builds the **integration-test pyramid** for production-planning: end-to-end scenarios that exercise REST + gRPC + worker + adapter + Cedar + outbox + Kafka + Postgres + Valkey in coherent compositions. Domain unit tests (per IP-001..IP-006) and usecase unit tests (per IP-007..IP-012, IP-018..IP-025) remain in their respective IPs; this IP is for **cross-layer black-box** scenarios mirroring SAP customer-acceptance test catalogues (`PP-AT-100` through `PP-AT-500` family) and the ASUG / SAPinsider community parity suites.

### A.1 Test pyramid commitments

| Layer | Owner IP | Tools | Count target |
|---|---|---|---|
| Domain unit                  | IP-001..IP-006 + IP-018..IP-025 (domain slice if applicable) | `cargo test` pure | ≥80 per aggregate |
| Usecase unit                  | IP-007..IP-012 + IP-018..IP-025 | `cargo test` + port mocks | ≥40 per usecase |
| Adapter integration           | IP-013 | `cargo test --features integration` + testcontainers | ≥120 |
| API contract                  | IP-014 | OpenAPI + buf-breaking | ≥50 |
| **End-to-end (this IP)**     | IP-015 | testcontainers + docker-compose harness | ≥35 scenarios |
| Performance / load            | scorecards + SLO suite | `criterion`, `k6`, `vegeta` | ≥12 named scenarios |
| Chaos / resilience            | failure-modes.md | Litmus / chaos-mesh | ≥10 scenarios |

### A.2 Why this IP is non-trivial

End-to-end tests in a multi-cell, multi-tenant, HLC-ordered system are tricky. The harness must:

1. **Provision an isolated test cell** — Postgres + Kafka + Valkey + Cedar bundle + observability stack, all per-test reset.
2. **Seed deterministic tenants** — `acme-eu` (residency: EU), `acme-kr` (residency: KR pack), `acme-byok` (provider-credential BYOK provider mode, ADR-0255 §D-4); each with seeded calendar + BOM + routing.
3. **Stub external µservices** — `engineering-change`, `material-master`, `quality-management`, `plant-maintenance`, `warehouse`, `mes`, `inventory`, `costing` replaced by recorded contract stubs (per ADR-0294 envelope versions).
4. **Assert HLC monotonicity** end-to-end — every emitted envelope's HLC strictly greater than its predecessor on the same `(tenant, channel, partition_key)`.
5. **Assert Cedar default-deny** — fuzz one principal per tenant with empty policy bundle and assert all writes return 403.
6. **Replay validation** — re-run worker handlers against captured DLQ samples to validate dedupe logic.

## B. Acceptance criteria

- **AC-1:** End-to-end scenario list ≥35; each maps to an SAP customer-acceptance test ID and is named `e2e_{scenario_id}_{slug}`.
- **AC-2:** Harness boots in ≤45s on commodity laptop (16-core / 32GB); CI budget ≤90s.
- **AC-3:** Tenants `acme-eu`, `acme-kr`, `acme-byok` seeded deterministically; same seed → identical IDs across runs.
- **AC-4:** Cedar default-deny fuzz scenario runs against every write endpoint; 0 false-permits.
- **AC-5:** HLC monotonicity assertion runs on every scenario's captured envelopes.
- **AC-6:** Test coverage for: BOM publish → MRP run → planned-order → production-order create → release → confirm → close.
- **AC-7:** Cross-residency negative test: `acme-eu` principal cannot write to `acme-kr` order.
- **AC-8:** Performance scenarios named: `bom_publish_throughput`, `mrp_run_warmpath_latency`, `release_pipeline_throughput`, `confirm_idempotency_latency`, `capacity_query_cache_hit_latency`, etc.
- **AC-9:** Chaos scenarios named: `kafka_partition_loss`, `postgres_failover`, `valkey_outage`, `cedar_bundle_rejection`, `outbox_dispatcher_crash`, `hlc_clock_skew`, etc.
- **AC-10:** Audit-event capture — every scenario asserts the expected `EVT-PRODUCTION_PLANNING-...` audit events.

## C. Verification

```bash
# unit-equivalent (no docker)
cargo test -p oya-production-planning-it --features mocks

# full integration (requires docker)
cargo test -p oya-production-planning-it --features integration -- --nocapture

# performance
cargo bench -p oya-production-planning-it

# chaos (requires kind cluster)
make pp-chaos-suite

# parity matrix vs sap acceptance catalogue
cargo test -p oya-production-planning-it --features integration -- sap_acceptance_matrix
```

## D. Detailed mechanics

### D-1. End-to-end scenario inventory (≥35)

| ID | SAP-AT mapping | Scenario | Owner |
|---|---|---|---|
| `e2e_001_bom_lifecycle` | CS01-CS02-CS03 | Create BOM, edit, activate, retire | qa |
| `e2e_002_routing_lifecycle` | CA01-CA02-CA03 | Create routing, alt selection, publish | qa |
| `e2e_003_calendar_overlay` | CR01-CM01 | Publish calendar, ingest overlay, query | qa |
| `e2e_004_mrp_run_warmpath` | MD02 | Full MRP run on small demand set | qa |
| `e2e_005_mrp_run_explosion_large` | MD02 stress | MRP run on 50k materials | perf-eng |
| `e2e_006_planned_to_production_convert` | CO40 | Convert planned → production order | qa |
| `e2e_007_order_release_three_lanes` | CO02-rel | Release with warehouse + MES + QM lanes | qa |
| `e2e_008_order_release_qm_denied_lane` | CO02-rel + QM block | QM Cedar denies; order still releases | qa |
| `e2e_009_partial_confirmation` | CO11N | Multiple partial confirms; cumulative quantity | qa |
| `e2e_010_idempotent_confirm_replay` | CO11N replay | Same confirm_counter twice | qa |
| `e2e_011_overdelivery_rejected` | CO11N + overconfirm | confirmed > target | qa |
| `e2e_012_order_cancel_after_release` | CO02-cancel | Release then cancel; inverse-reserve | qa |
| `e2e_013_quality_hold_roundtrip` | QM hold + release | Order in_progress → on_hold → in_progress | qa |
| `e2e_014_teco_close_costing_handoff` | CO13 | TECO emits cost variance event | qa |
| `e2e_015_repetitive_manufacturing_backflush` | MFBF | REM order with backflush=true | qa |
| `e2e_016_engineering_change_invalidates_routing` | CC02 + CA02 | Publish ECN; routing cache invalidates | qa |
| `e2e_017_capacity_reservation_conflict` | CM25 conflict | Two orders compete for same slot | qa |
| `e2e_018_ddmrp_buffer_breach_triggers_order` | DD-MRP | DDMRP buffer breach → planned order | qa |
| `e2e_019_sop_executive_signoff_gate` | S&OP | Cedar gate on executive sign-off | qa |
| `e2e_020_production_version_selection` | C223 | Multiple production versions; selection deterministic | qa |
| `e2e_021_capacity_leveling_forward` | CM27 | Forward scheduling | qa |
| `e2e_022_capacity_leveling_backward` | CM27 | Backward scheduling | qa |
| `e2e_023_long_term_planning_scenario` | MS02 | LTP planning version | qa |
| `e2e_024_alt_routing_engaged_on_bottleneck` | CA02 + CM27 | Alternative routing engaged | qa |
| `e2e_025_mes_handshake_bidirectional` | PI-MES | MES confirms operation | qa |
| `e2e_026_line_balancing_takt_time` | LBL | Line balancing scenario | qa |
| `e2e_027_cross_tenant_load_rejected` | security | acme-eu cannot load acme-kr order | security |
| `e2e_028_cross_residency_pack_enforced` | pack | KR pack rejects EU residency write | security |
| `e2e_029_cedar_default_deny_fuzz` | security | empty policy → all denies | security |
| `e2e_030_outbox_drain_hlc_order` | substrate | dispatcher preserves HLC order | qa |
| `e2e_031_worker_dlq_on_schema_drift` | substrate | bad envelope → DLQ | qa |
| `e2e_032_worker_dedupe_replay` | substrate | duplicate event_id → no double-apply | qa |
| `e2e_033_byok_provider_credentials_used` | tenant pack | acme-byok uses tenant LLM keys | qa |
| `e2e_034_eu_ai_act_explainability_record_emitted` | AI Act | DDMRP recalc emits explainability record | compliance |
| `e2e_035_full_journey_j101_replay` | journey | replay j101 end-to-end | qa |

### D-2. Harness layout

```
crates/oya-production-planning-it/
  Cargo.toml
  tests/
    e2e_001_bom_lifecycle.rs
    e2e_002_routing_lifecycle.rs
    ...
  src/
    harness/
      mod.rs              # boot, seed, teardown
      containers.rs       # testcontainers Postgres/Kafka/Valkey/Otel
      seeds.rs            # tenant seeding
      stubs.rs            # external µservice stubs (axum + tonic)
      assertions.rs       # HLC monotonicity, audit-event capture
      cedar_bundle.rs     # bundle helpers
```

### D-3. Tenant seeding deterministic

```rust
pub async fn seed_tenant_acme_eu(pool: &PgPool) -> SeededTenant {
    let tenant_id = TenantId::new("acme-eu");
    seed_calendar(pool, &tenant_id, "P01-EU", &gregorian_2_shift()).await;
    seed_bom(pool, &tenant_id, "FG-0001", "P01-EU", &three_component_bom()).await;
    seed_routing(pool, &tenant_id, "RTG-FG-0001", "ALT-A", "P01-EU",
                 &five_step_routing()).await;
    seed_work_centers(pool, &tenant_id, &["WC-MILL", "WC-LATHE", "WC-ASSY"]).await;
    SeededTenant { tenant_id, plant_code: "P01-EU".into(), routing_key: ... }
}
```

### D-4. HLC monotonicity assertion

```rust
pub async fn assert_hlc_monotone(captured: &[(String, String, Hlc)]) {
    // (tenant, channel, hlc) — group, sort, assert strictly increasing within group
    let mut groups: BTreeMap<(String,String), Vec<Hlc>> = BTreeMap::new();
    for (t, c, h) in captured {
        groups.entry((t.clone(), c.clone())).or_default().push(h.clone());
    }
    for ((t, c), hlcs) in &groups {
        let mut prev: Option<Hlc> = None;
        for h in hlcs {
            if let Some(p) = &prev {
                assert!(p < h, "HLC non-monotone on (tenant={}, channel={}): {} -> {}", t, c, p, h);
            }
            prev = Some(h.clone());
        }
    }
}
```

### D-5. Cedar default-deny fuzz

```rust
#[tokio::test]
async fn e2e_029_cedar_default_deny_fuzz() {
    let harness = Harness::boot_with_empty_cedar_bundle().await;
    let routes = harness.openapi_writes();  // all POST/PUT/PATCH/DELETE
    for r in routes {
        let resp = harness.request_with_dummy_principal(&r).await;
        assert!(matches!(resp.status(), 403 | 401),
                "route {} permitted under empty bundle: {}", r.path, resp.status());
    }
}
```

### D-6. Performance scenarios (criterion)

```rust
fn bom_publish_throughput(c: &mut Criterion) {
    let h = block_on(Harness::boot()); let t = block_on(seed_tenant_acme_eu(&h.pool));
    c.bench_function("bom_publish_throughput_5krows", |b| {
        b.iter(|| { block_on(h.publish_bom(&t.tenant_id, &fixture_bom_5k_rows())); });
    });
}
```

### D-7. Chaos scenarios (chaos-mesh)

```yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: PodChaos
metadata: { name: pp-postgres-failover }
spec:
  action: pod-kill
  mode: one
  selector: { labelSelectors: { app: postgres-primary, ns: oyatie-test } }
```

Test asserts that the release-pipeline scenario completes successfully even with primary kill mid-test (failover within 10s, retries succeed).

### D-8. Audit-event capture pattern

```rust
let captured = harness.audit_sink.drain_for_tenant(&t.tenant_id).await;
assert!(captured.iter().any(|e| e.class == "EVT-PRODUCTION_PLANNING-PRODUCTION_ORDER-RELEASED"));
assert!(captured.iter().any(|e| e.class == "EVT-PRODUCTION_PLANNING-SHOP_FLOOR_RELEASE-EMITTED"));
```

### D-9. SLO floor enforced by tests

Each performance scenario asserts the p99 SLO floor stated in upstream IPs (e.g., IP-011 D-9). Test fails if measured p99 > 1.2× floor (20% headroom for CI noise).

### D-10. Failure modes & recovery

1. **Flaky scenario (>1% retry rate in CI)** — quarantine via `#[ignore]` + GitHub issue; root-cause within 7d.
2. **Harness boot timeout (>90s)** — investigate container pull cache; pin image digests; runbook `runbooks/it-harness-boot-timeout.md`.
3. **HLC monotonicity false-positive** — usually NTP drift in CI runner; pin host clock; runbook `runbooks/hlc-skew-in-ci.md`.
4. **Cedar bundle drift between unit + integration** — single source of truth in `policy/` directory; harness loads same file; CI lint catches drift.
5. **Stub-contract drift** — stubs validated against same OpenAPI/AsyncAPI/proto schemas as production adapters; PR check.
6. **DLQ accumulation in test runs** — assert empty DLQ at scenario teardown; failure surfaces test pollution.

### D-11. Migration notes

Source vendor surface: SAP customer-acceptance test catalogue (`PP-AT-*`), SAP Solution Manager Test Workbench, Tricentis TOSCA SAP packages, Worksoft Certify. Each Oyatie scenario maps 1:1 to an SAP-AT ID for parity audit (per ADR-0315/ADR-0316).

### D-12. Ontology projection

Tests assert that ontology delta emitted by usecase matches expected node/edge set; verified via `ontology_delta_to_graph()` helper.

### D-13. Cross-µservice handoffs

Tests use **stub** µservices for warehouse/MES/QM/etc. Cross-µservice contract tests (real µservices, in-process) live in `microservices/<peer>/IP-XXX-handoff.md` (e.g., IP-016, IP-017).

## E. Failure-mode summary

See D-10.

## F. Migration / rollback

Integration-test crate is build-time only; rollback = revert IP changes; CI lane retains historical pass/fail traces.

## G. References

- ADR-0105, ADR-0244, ADR-0263, ADR-0294, ADR-0297, ADR-0314, ADR-0315, ADR-0316.
- testcontainers-rs, criterion-rs, chaos-mesh.
- SAP Solution Manager Test Workbench, ASUG PP test community baseline.
- Benchmarks: SAP-AT catalogue | Oracle Fusion Cloud Manufacturing Test Catalog | Microsoft Dynamics 365 SCM RSAT (Regression Suite Automation Tool) | NetSuite SuiteAnalytics QA pack.

## H. Out of scope

- Domain unit tests (per IP-001..IP-006), usecase unit tests (IP-007..IP-012), adapter unit tests (IP-013), API contract tests (IP-014).

— end IP-015 —
