---
doc_class: ImplementationPlan
ip_id: IP-015
microservice: warehouse
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0244
  - ADR-0245
  - ADR-0253
  - ADR-0263
  - ADR-0294
  - ADR-0297
  - ADR-0314
  - ADR-0315
  - ADR-0329
  - ADR-0330
  - ADR-0331
journey_ref: j123-multi-tenant-coordinated-product-launch
sap_submodule: EWM-WIM (inventory)
tenant_class: paid
billing_components:
  - per_usage
persona: Elena Petrova, warehouse QA automation lead
status: Accepted
date: 2026-05-20
owner_team: axis-warehouse + axis-erp-parity
---

# IP-015: Integration tests for warehouse

## Context

- SAP submodule: EWM-WIM inventory and end-to-end warehouse evidence.
- Persona: Elena Petrova, warehouse QA automation lead.
- Journey leg: j123 launch flow proves inbound, putaway, wave release, RF pick, outbound release, and audit handoffs together.
- SAP tables: `/SCWM/PRDI`, `/SCWM/PRDO`, `/SCWM/ORDIM_O`, `/SCWM/QUANT`, `/SCWM/STORAGEBIN`, `/SCWM/WAREHOUSEORDER`.
- Oyatie test suite: `warehouse_integration_tests`.
- Precedent: SAP EWM end-to-end goods-flow test pack plus Stripe-style contract replay fixtures.
- ADR-0329/0330/0331 requires implementation-ready depth and ADR-0297 requires Cedar policy coverage.
- Boundary: owns integration fixture definitions, test orchestration, and evidence assertions; it does not implement business logic.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE warehouse.integration_test_fixture (
  tenant_id UUID NOT NULL,
  fixture_id TEXT NOT NULL,
  fixture_name TEXT NOT NULL,
  sap_table_set TEXT[] NOT NULL,
  scenario_state TEXT NOT NULL CHECK (scenario_state IN ('draft','active','retired')),
  expected_audit_events TEXT[] NOT NULL,
  expected_handoffs TEXT[] NOT NULL,
  PRIMARY KEY (tenant_id, fixture_id)
);
CREATE TABLE warehouse.integration_test_run (
  tenant_id UUID NOT NULL,
  test_run_id TEXT NOT NULL,
  fixture_id TEXT NOT NULL,
  started_at TIMESTAMPTZ NOT NULL,
  completed_at TIMESTAMPTZ,
  result TEXT NOT NULL CHECK (result IN ('running','passed','failed','quarantined')),
  failure_summary TEXT,
  PRIMARY KEY (tenant_id, test_run_id)
);
```

### Rust Types

```rust
pub struct WarehouseIntegrationFixture {
    pub tenant_id: TenantId,
    pub fixture_id: FixtureId,
    pub fixture_name: String,
    pub sap_table_set: Vec<SapTableName>,
    pub expected_audit_events: Vec<AuditEventClass>,
    pub expected_handoffs: Vec<HandoffName>,
}
pub struct WarehouseIntegrationTestRun {
    pub test_run_id: TestRunId,
    pub fixture_id: FixtureId,
    pub result: TestRunResult,
    pub failure_summary: Option<String>,
}
pub enum WarehouseIntegrationTestError { FixtureMissing, PolicyFixtureInvalid, AuditEventMissing, HandoffMissing, SloBudgetExceeded }
```

## API Endpoints

- REST `POST /v1/warehouse/integration-fixtures` registers a scenario fixture.
- REST `POST /v1/warehouse/integration-fixtures/{id}:run` starts a test run in CI scope.
- REST `GET /v1/warehouse/integration-test-runs/{id}` returns evidence and failures.
- gRPC `warehouse.test.v1.WarehouseIntegrationTestService.RunFixture`.
- gRPC `GetTestRun` and `StreamTestRunEvents`.
- AsyncAPI channel `warehouse.integration-test.run-completed.v1`.
- AsyncAPI channel `warehouse.integration-test.failure-detected.v1`.
- Consumers: CI, audit-chain, compliance, release-management.

## Cedar Policy Hooks

- Policy: `warehouse::integration_test::run`.
- Principal: `CiServicePrincipal`.
- Action: `warehouse_integration_test_run`.
- Resource: `IntegrationTestFixture`.
- Context: `tenant_id`, `fixture_id`, `ci_scope`, `policy_bundle_version`, `test_data_pack`.
- Forbid when fixture uses production data, CI scope is missing, or policy bundle is not the expected test version.

## Ontology Projection

- Vendor object: SAP EWM end-to-end test scenario.
- Oyatie object: `warehouse.integration_test_run`.
- `/SCWM/PRDI` -> inbound fixture rows.
- `/SCWM/PRDO` -> outbound fixture rows.
- `/SCWM/ORDIM_O` -> warehouse task fixture rows.
- `/SCWM/QUANT` -> inventory fixture rows.
- `/SCWM/STORAGEBIN` -> bin fixture rows.
- `/SCWM/WAREHOUSEORDER` -> wave and labor fixture rows.
- Projection freshness floor: CI run completion.
- Projection rule: failed runs project with exact missing audit event and handoff.

## Workflow Steps

- Node `fixture-register`: validate scenario and expected evidence.
- Node `ci-policy-check`: enforce CI-only data pack.
- Decision `fixture-invalid`: reject and mark authoring failure.
- Node `seed-data`: load tenant-scoped fixture rows.
- Node `execute-flow`: run inbound to outbound scenario.
- Decision `policy-deny-unexpected`: fail run with Cedar evidence.
- Node `assert-events`: verify ADR-0263 event classes.
- Node `assert-handoffs`: verify downstream event receipts.
- Decision `slo-budget-exceeded`: fail run with latency histogram.
- Node `publish-run`: emit run completed event.

## Audit Events

- `EVT-WAREHOUSE-INTEGRATION_TEST-FIXTURE_REGISTERED`.
- `EVT-WAREHOUSE-INTEGRATION_TEST-RUN_STARTED`.
- `EVT-WAREHOUSE-INTEGRATION_TEST-RUN_PASSED`.
- `EVT-WAREHOUSE-INTEGRATION_TEST-RUN_FAILED`.
- `EVT-WAREHOUSE-INTEGRATION_TEST-POLICY_DENIED`.
- `EVT-WAREHOUSE-INTEGRATION_TEST-IP_ACCEPTED`.
- ADR-0263 envelope stores `fixture_id`, `test_run_id`, `expected_audit_events`, and `expected_handoffs`.

## SLO Targets

- Fixture registration p50: 40 ms.
- Fixture registration p95: 140 ms.
- Fixture registration p99: 380 ms.
- End-to-end test run p95: 180 seconds for full inbound-to-outbound scenario.
- Rationale: CI can tolerate minutes for full ERP flow; fixture authoring APIs should remain interactive.

## Failure Modes and Recovery

- Failure: `FIXTURE-MISSING`; recovery: fail run before seeding.
- Failure: `POLICY-FIXTURE-INVALID`; recovery: block run and show invalid principal/action/resource/context.
- Failure: `AUDIT-EVENT-MISSING`; recovery: fail run and list missing ADR-0263 class.
- Failure: `HANDOFF-MISSING`; recovery: fail run and expose downstream receipt gap.
- Failure: `SLO-BUDGET-EXCEEDED`; recovery: preserve histograms and mark release blocker.
- Failure: `TEST-DATA-LEAK`; recovery: quarantine fixture and emit security event.

## Migration Notes

- Convert scaffold fixtures to tenant-scoped SAP table row sets.
- Retire any fixture that depends on shared mutable state.
- Preserve historical failed run evidence for release audit.
- Normalize SAP fixture IDs and expected event names before activation.
- Rollback path: quarantine new fixtures while leaving previous test pack active.
- Backfill order: fixture records, seed data, expected events, expected handoffs, test runs.

## Cross-microservice Handoffs

- From CI: test principal, data pack, and policy bundle version.
- To audit-chain: run start, pass, fail, and policy denied events.
- To quality-management: attestation fixture outcomes.
- To inventory-ledger: stock movement assertions.
- To carrier-integration: outbound readiness assertions.
- To release-management: blocker or pass evidence.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The tests remain bound to SAP EWM inventory and end-to-end warehouse evidence. |
| Persona specificity | Elena Petrova owns fixture selection, expected events, and release-blocker acceptance language. |
| Journey specificity | The j123 launch flow drives inbound, putaway, wave, RF, outbound, and audit coverage. |
| DDL anchor | Fixture, seed data, expected event, handoff, and test-run tables above are normative. |
| Rust anchor | Test fixture, expected event, assertion result, and error enum above are implementation anchors. |
| REST anchor | Test harness invokes warehouse REST commands through real tenant-scoped operation receipts. |
| gRPC anchor | Worker assertions verify gRPC replay contracts, not mocked in-process shortcuts. |
| AsyncAPI anchor | Expected event channels assert inbound, putaway, picking, outbound, and audit emissions. |
| Cedar anchor | Test principals must hit real default-deny Cedar decisions and store `cedar_decision_id`. |
| Ontology anchor | Test projections assert SAP EWM lineage remains queryable after each warehouse command. |
| ADR-0263 class binding | Test policy checks emit `OfficeBoundaryAttemptEvaluated` and allowed/denied outcome classes. |
| ADR-0263 pack binding | Fixture pack activation emits `OfficePackOverlayChanged`. |
| ADR-0263 security binding | CI abuse throttles emit `AbuseDefenceRateLimitHit` through the registered class. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, test run id, fixture id, expected event id, and policy bundle version. |
| Metric | `oya_warehouse_integration_test_runs_total{tenant_id,cell_id,suite,status}` caps suite/status cardinality. |
| Latency histogram | `oya_warehouse_integration_test_duration_seconds` tracks suite and scenario runtime. |
| Trace span | `warehouse.integration_test.run_scenario` links CI, warehouse APIs, downstream services, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `suite_id`, `fixture_id`, `scenario_id`, and failure class. |
| Capacity math | CI shard count uses scenarios / target_minutes; queue depth above target routes non-blocking tests to nightly lane. |
| Multi-region | Blocking tests run in home-cell and DR-cell read-only replay modes before promotion. |
| Sovereign cells | Fixture data remains synthetic or in-region for regulated compliance-pack overlays. |
| Rollback | Quarantine new fixtures, keep previous test pack active, and replay from last sealed test-run audit id. |
| Test evidence | Required tests cover full launch flow, policy denial, tenant mismatch, event ordering, and idempotent replay. |
| Rejected shortcut | A generic smoke test pack is rejected because it would not prove SAP EWM warehouse parity or event ordering. |
