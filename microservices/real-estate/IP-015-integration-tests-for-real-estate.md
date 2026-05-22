---
doc_class: ImplementationPlan
ip_id: IP-015
microservice: real-estate
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
journey_ref: j137-corporate-internal-audit-sox-controls-test
sap_submodule: RE-FX-AC (lease accounting)
tenant_class: paid
billing_components:
  - per_usage
persona: Renata Costa, real-estate QA automation lead
status: Accepted
date: 2026-05-20
owner_team: axis-real-estate + axis-erp-parity
---

# IP-015: Integration tests for real-estate

## Context

- SAP submodule: RE-FX-AC end-to-end lease accounting evidence.
- Persona: Renata Costa, real-estate QA automation lead.
- Journey leg: j137 SOX control test runs contract, object, occupancy, rent, accounting, and service-request paths together.
- SAP tables: `VICDCONTRACT`, `VICDOBJASS`, `VICDCONDLINE`, `VICDADJREASN`, `VIBDRO`, `VIBDBU`.
- Oyatie suite: `real_estate_integration_tests`.
- Precedent: SAP RE-FX regression pack plus Workday tenant data-load validation.
- ADR-0329/0330/0331 requires implementation-ready ERP evidence and ADR-0297 requires Cedar coverage.
- Boundary: owns fixture records, run state, and assertions; it does not implement service logic.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE real_estate.integration_test_fixture (
  tenant_id UUID NOT NULL,
  fixture_id TEXT NOT NULL,
  fixture_name TEXT NOT NULL,
  sap_table_set TEXT[] NOT NULL,
  scenario_state TEXT NOT NULL CHECK (scenario_state IN ('draft','active','retired')),
  expected_audit_events TEXT[] NOT NULL,
  expected_handoffs TEXT[] NOT NULL,
  PRIMARY KEY (tenant_id, fixture_id)
);
CREATE TABLE real_estate.integration_test_run (
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
pub struct RealEstateIntegrationFixture {
    pub tenant_id: TenantId,
    pub fixture_id: FixtureId,
    pub fixture_name: String,
    pub sap_table_set: Vec<SapTableName>,
    pub expected_audit_events: Vec<AuditEventClass>,
    pub expected_handoffs: Vec<HandoffName>,
}
pub struct RealEstateIntegrationTestRun {
    pub test_run_id: TestRunId,
    pub fixture_id: FixtureId,
    pub result: TestRunResult,
    pub failure_summary: Option<String>,
}
pub enum RealEstateIntegrationTestError { FixtureMissing, PolicyFixtureInvalid, AuditEventMissing, HandoffMissing, SloBudgetExceeded }
```

## API Endpoints

- REST `POST /v1/real-estate/integration-fixtures` registers fixture.
- REST `POST /v1/real-estate/integration-fixtures/{id}:run` runs fixture in CI scope.
- REST `GET /v1/real-estate/integration-test-runs/{id}` returns evidence.
- gRPC `real_estate.test.v1.RealEstateIntegrationTestService.RunFixture`.
- gRPC `GetTestRun` and `StreamTestRunEvents`.
- AsyncAPI channel `real-estate.integration-test.run-completed.v1`.
- AsyncAPI channel `real-estate.integration-test.failure-detected.v1`.
- Consumers: CI, audit-chain, compliance, release-management.

## Cedar Policy Hooks

- Policy: `real_estate::integration_test::run`.
- Principal: `CiServicePrincipal`.
- Action: `real_estate_integration_test_run`.
- Resource: `IntegrationTestFixture`.
- Context: `tenant_id`, `fixture_id`, `ci_scope`, `policy_bundle_version`, `test_data_pack`.
- Forbid when fixture uses production data, CI scope is absent, policy bundle is wrong, or expected audit list is incomplete.

## Ontology Projection

- Vendor object: SAP RE-FX end-to-end fixture.
- Oyatie object: `real_estate.integration_test_run`.
- `VICDCONTRACT` -> lease contract fixture rows.
- `VICDOBJASS` -> object assignment fixture rows.
- `VICDCONDLINE` -> rent condition fixture rows.
- `VICDADJREASN` -> adjustment reason fixture rows.
- `VIBDRO` and `VIBDBU` -> architectural object fixture rows.
- Expected handoffs -> service integration assertions.
- Projection freshness floor: CI run completion.
- Projection rule: failed runs identify missing audit event or handoff by exact class.

## Workflow Steps

- Node `fixture-register`: validate scenario and expected events.
- Node `ci-policy-check`: enforce CI-only data pack.
- Decision `fixture-invalid`: reject fixture.
- Node `seed-data`: load tenant-scoped RE-FX rows.
- Node `execute-flow`: run contract-to-accounting scenario.
- Decision `policy-deny-unexpected`: fail run.
- Node `assert-events`: verify ADR-0263 classes.
- Node `assert-handoffs`: verify downstream receipts.
- Decision `slo-budget-exceeded`: fail run with histogram.
- Node `publish-run`: emit run completion.

## Audit Events

- `EVT-REAL_ESTATE-INTEGRATION_TEST-FIXTURE_REGISTERED`.
- `EVT-REAL_ESTATE-INTEGRATION_TEST-RUN_STARTED`.
- `EVT-REAL_ESTATE-INTEGRATION_TEST-RUN_PASSED`.
- `EVT-REAL_ESTATE-INTEGRATION_TEST-RUN_FAILED`.
- `EVT-REAL_ESTATE-INTEGRATION_TEST-POLICY_DENIED`.
- `EVT-REAL_ESTATE-INTEGRATION_TEST-IP_ACCEPTED`.
- ADR-0263 envelope stores fixture ID, test run ID, expected audit events, and expected handoffs.

## SLO Targets

- Fixture registration p50: 40 ms.
- Fixture registration p95: 140 ms.
- Fixture registration p99: 380 ms.
- End-to-end fixture run p95: 180 seconds.
- Rationale: CI can take minutes for full lease flow; fixture authoring remains interactive.

## Failure Modes and Recovery

- Failure: `FIXTURE-MISSING`; recovery: fail run before seed.
- Failure: `POLICY-FIXTURE-INVALID`; recovery: block run and show invalid Cedar tuple.
- Failure: `AUDIT-EVENT-MISSING`; recovery: fail run with missing class list.
- Failure: `HANDOFF-MISSING`; recovery: fail run and show downstream receipt gap.
- Failure: `SLO-BUDGET-EXCEEDED`; recovery: preserve histograms and block release.
- Failure: `TEST-DATA-LEAK`; recovery: quarantine fixture and emit security event.

## Migration Notes

- Convert scaffold fixtures to tenant-scoped RE-FX table row sets.
- Retire fixtures that depend on shared mutable state.
- Preserve failed run evidence for release audit.
- Normalize SAP fixture IDs and expected event names.
- Rollback path: quarantine new fixtures and keep previous pack active.
- Backfill order: fixture records, seed data, expected events, expected handoffs, test runs.

## Cross-microservice Handoffs

- From CI: principal, policy bundle, and test data pack.
- To audit-chain: run lifecycle events.
- To finance-ledger: posting fixture assertion.
- To payments: rent due-line assertion.
- To workflow-engine: approval fixture assertion.
- To release-management: pass/fail evidence.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The tests remain bound to SAP RE-FX-AC end-to-end lease accounting evidence. |
| Persona specificity | Renata Costa owns fixture selection, expected events, and release-blocker language. |
| Journey specificity | The j137 SOX control test drives contract, object, occupancy, rent, accounting, and service-request coverage. |
| DDL anchor | Fixture, seed data, expected event, expected handoff, and test-run tables above are normative. |
| Rust anchor | Fixture, expected event, assertion result, and error types above are implementation anchors. |
| REST anchor | Test harness invokes tenant-scoped real-estate REST operations through command receipts. |
| gRPC anchor | Worker assertions verify gRPC replay contracts rather than mocks. |
| AsyncAPI anchor | Expected channels assert lease, rent, accounting, service request, and audit emissions. |
| Cedar anchor | Test principals hit real default-deny Cedar decisions and store `cedar_decision_id`. |
| Ontology anchor | SAP RE-FX lineage remains queryable after every fixture mutation. |
| ADR-0263 class binding | Test policy checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Fixture pack activation emits `OfficePackOverlayChanged`. |
| ADR-0263 security binding | CI abuse throttles emit `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, test run id, fixture id, expected event id, and policy bundle version. |
| Metric | `oya_real_estate_integration_test_runs_total{tenant_id,cell_id,suite,status}` caps suite/status cardinality. |
| Latency histogram | `oya_real_estate_integration_test_duration_seconds` tracks suite and scenario runtime. |
| Trace span | `real_estate.integration_test.run_scenario` links CI, APIs, downstream services, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `suite_id`, `fixture_id`, `scenario_id`, and failure class. |
| Capacity math | CI shard count uses scenarios / target_minutes; long-running fixtures move to nightly when queue risk exceeds cutoff. |
| Multi-region | Blocking tests run in home-cell and DR read-only replay modes before promotion. |
| Sovereign cells | Fixture data remains synthetic or in-region for regulated compliance-pack overlays. |
| Rollback | Quarantine new fixtures, keep previous pack active, and replay from last sealed test-run audit id. |
| Test evidence | Required tests cover lease approval, rent due-line, accounting posting, service request, policy denial, and event ordering. |
| Rejected shortcut | A generic smoke suite is rejected because it would not prove SAP RE-FX SOX or event-ordering evidence. |
