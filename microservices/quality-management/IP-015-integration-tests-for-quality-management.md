---
doc_class: ImplementationPlan
ip_id: IP-015
microservice: quality-management
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0244
  - ADR-0253
  - ADR-0263
  - ADR-0294
  - ADR-0297
  - ADR-0314
  - ADR-0315
  - ADR-0329
  - ADR-0330
  - ADR-0331
  - ADR-0320
journey_ref: j101-multi-tier-supply-chain-formation
sap_submodule: QM-IM/QM-QC/QM-CA/QM-QN/QM-AU Test Harness
tenant_class: paid
billing_components:
  - per_usage
persona: Noor Singh, ERP QA lead
status: Accepted
date: 2026-05-20
owner_team: axis-quality-management + axis-erp-parity
---

# IP-015: Integration tests for quality-management SAP QM parity

## Context

- SAP QM submodule: full QM parity test harness.
- Topic: integration tests across inspection, control, CAPA, notifications, and audits.
- Persona: Noor Singh, ERP QA lead.
- Journey: j101 multi-tier supply-chain formation.
- Journey leg: a supplier receipt flows through lot creation, result recording, hold, notification, and certificate release.
- SAP precedent: end-to-end QM scenario testing across goods receipt and usage decision.
- Oyatie layer: integration test and verification harness.
- Boundary: test data, fixture contracts, policy checks, event replay, and SLO probes.
- ADR-0105 defines layer boundaries under test.
- ADR-0131 keeps the harness near the microservice.
- ADR-0244 requires tenant isolation test fixtures.
- ADR-0263 requires audit event assertions.
- ADR-0297 requires Cedar policy checks.
- ADR-0314 requires marketplace read-only assertions.
- ADR-0315 requires SAP QM parity coverage.
- ADR-0329/0330/0331 requires implementation-ready detail.
- Tests must prove handoffs, not only local happy paths.
- Tests must include negative cases and replay recovery.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE quality_management.integration_test_fixture (
  tenant_id UUID NOT NULL,
  fixture_id TEXT NOT NULL,
  fixture_name TEXT NOT NULL,
  sap_submodule TEXT NOT NULL,
  scenario_name TEXT NOT NULL,
  source_vendor TEXT NOT NULL,
  expected_event_class TEXT NOT NULL,
  fixture_hash TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, fixture_id)
);
CREATE TABLE quality_management.integration_test_evidence (
  tenant_id UUID NOT NULL,
  evidence_id TEXT NOT NULL,
  fixture_id TEXT NOT NULL,
  assertion_name TEXT NOT NULL,
  assertion_state TEXT NOT NULL,
  latency_ms INTEGER,
  event_id TEXT,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, evidence_id)
);
```

### Rust Types

```rust
pub struct QualityIntegrationFixture {
    pub tenant_id: TenantId,
    pub fixture_id: FixtureId,
    pub fixture_name: FixtureName,
    pub sap_submodule: SapQmSubmodule,
    pub scenario_name: ScenarioName,
    pub source_vendor: QmsVendor,
    pub expected_event_class: AuditEventClass,
    pub fixture_hash: FixtureHash,
}
pub struct QualityIntegrationEvidence {
    pub evidence_id: EvidenceId,
    pub fixture_id: FixtureId,
    pub assertion_name: AssertionName,
    pub assertion_state: AssertionState,
    pub latency_ms: Option<u32>,
    pub event_id: Option<EventId>,
}
pub enum SapQmSubmodule { QmIm, QmQc, QmCa, QmQn, QmAu }
pub enum AssertionState { Passed, Failed, Skipped, Blocked }
pub enum IntegrationHarnessError {
    FixtureHashMismatch,
    MissingAuditEvent,
    PolicyExpectedDenyButPermitted,
    HandoffNotObserved,
    SloProbeFailed,
}
```

## API Endpoints

### REST

- `POST /v1/quality-management/test-fixtures`.
- Registers a deterministic fixture.
- `POST /v1/quality-management/test-fixtures/{fixture_id}:run`.
- Runs a scoped integration scenario in test cell.
- `GET /v1/quality-management/test-fixtures/{fixture_id}/evidence`.
- Returns assertions, audit events, and SLO samples.
- `POST /v1/quality-management/test-fixtures/{fixture_id}:replay-events`.
- Replays outbox events into test consumers.

### gRPC

- Service: `quality_management.test_harness.v1.QualityIntegrationHarness`.
- `rpc RegisterFixture(RegisterFixtureRequest) returns (FixtureReceipt)`.
- `rpc RunFixture(RunFixtureRequest) returns (FixtureRunSummary)`.
- `rpc ReplayFixtureEvents(ReplayFixtureEventsRequest) returns (ReplaySummary)`.
- `rpc StreamFixtureEvidence(StreamFixtureEvidenceRequest) returns (stream FixtureEvidenceEvent)`.

### AsyncAPI

- Channel: `quality-management.test.fixture-ran.v1`.
- Channel: `quality-management.test.assertion-failed.v1`.
- Channel: `quality-management.test.replay-completed.v1`.
- Message: `QualityFixtureRan`.
- Message: `QualityAssertionFailed`.
- Payload carries `fixture_id`, `scenario_name`, `assertion_name`, `assertion_state`, `audit_event_class`.
- Consumers: CI, evidence dashboard, compliance, ontology.

## Cedar Policy Hooks

- Policy: `quality_management::test_harness::run_fixture`.
- Principal: `CiRunner` or `QualityQaLead`.
- Action: `quality_fixture_run`.
- Resource: `IntegrationTestFixture`.
- Context: `test_cell`, `source_vendor`, `sap_submodule`, `allowed_surfaces`.
- Policy: `quality_management::test_harness::replay_events`.
- Principal: `CiRunner`.
- Action: `quality_fixture_replay_events`.
- Resource: `FixtureEventLog`.
- Context: `event_count`, `target_consumer`, `replay_window`, `pack_ids`.
- Forbid: fixture run outside test cell.
- Forbid: replay into production topic.
- Forbid: fixture hash mismatch.
- Forbid: marketplace settlement mutation observed.

## Ontology Projection

- Vendor object: SAP QM end-to-end scenario evidence.
- Oyatie object: `quality_management.integration_test_fixture`.
- SAP goods receipt lot scenario -> fixture `goods-receipt-to-usage-decision`.
- SAP certificate scenario -> fixture `accepted-lot-to-coa-release`.
- SAP notification scenario -> fixture `failed-result-to-defect-notification`.
- SAP audit scenario -> fixture `audit-evidence-to-finding-close`.
- SAP DMR scenario -> fixture `supplier-history-to-reduced-inspection`.
- IQS-AQM fixture -> audit evidence import test.
- TIPQA fixture -> receiving lot import test.
- TrackWise fixture -> deviation and CAPA test.
- MasterControl fixture -> document and certificate test.
- ETQ Reliance fixture -> complaint mirror test.
- Projection freshness floor: CI run only.
- Projection consumer: evidence dashboard.
- Projection rule: failed assertions project as blockers.

## Workflow Steps

- Node `fixture-register`: deterministic fixture stored.
- Node `fixture-hash-check`: fixture hash validated.
- Decision `hash-mismatch`: block run.
- Node `test-cell-check`: ensure isolated test cell.
- Decision `production-topic`: block replay.
- Node `seed-domain`: create plans, lots, vendors, and audit templates.
- Node `run-happy-path`: execute full scenario.
- Node `run-negative-path`: execute expected denies.
- Node `assert-events`: require ADR-0263 event classes.
- Node `assert-policy`: require Cedar permit and deny evidence.
- Node `assert-handoffs`: verify warehouse, workflow, compliance, ontology signals.
- Node `assert-marketplace-readonly`: check no settlement mutation.
- Node `assert-slo`: record p50, p95, and p99 samples.
- Node `replay-outbox`: replay event log to consumers.
- Decision `assertion-failed`: mark fixture failed and stop promotion.
- Node `evidence-write`: persist test evidence.
- Node `audit-seal`: emit harness event.
- Node `close`: fixture has terminal evidence state.

## Audit Events

- `EVT-QUALITY_MANAGEMENT-TEST-FIXTURE_REGISTERED`.
- `EVT-QUALITY_MANAGEMENT-TEST-FIXTURE_RAN`.
- `EVT-QUALITY_MANAGEMENT-TEST-ASSERTION_FAILED`.
- `EVT-QUALITY_MANAGEMENT-TEST-REPLAY_COMPLETED`.
- `EVT-QUALITY_MANAGEMENT-INTEGRATION_TEST-IP_ACCEPTED`.
- ADR-0263 envelope stores `fixture_id`.
- ADR-0263 envelope stores `scenario_name`.
- ADR-0263 envelope stores `assertion_name`.
- ADR-0263 envelope stores `source_vendor`.
- ADR-0263 envelope stores `sap_submodule`.

## SLO Targets

- Fixture registration p95: 150 ms.
- Single fixture run p95: 45 seconds.
- Event replay p95: 20 seconds for 1,000 events.
- Evidence read p95: 200 ms.
- Throughput: 20 concurrent fixtures per test cell.
- Availability: 99.9 percent for CI harness.
- Rationale: CI must finish quickly enough to gate frequent documentation and contract changes.

## Failure Modes and Recovery

- Failure: fixture hash mismatch.
- Recovery: `TEST-FIXTURE-HASH-BLOCK` refuses run and reports expected hash.
- Failure: expected audit event is missing.
- Recovery: `TEST-AUDIT-MISSING-FAIL` marks assertion failed and blocks promotion.
- Failure: Cedar expected deny permits.
- Recovery: `TEST-POLICY-UNSAFE-FAIL` blocks release immediately.
- Failure: handoff consumer does not ACK.
- Recovery: `TEST-HANDOFF-REPLAY` replays event and marks recovery evidence.
- Failure: marketplace settlement mutation observed.
- Recovery: `TEST-MARKETPLACE-MUTATION-FAIL` hard-fails fixture.
- Failure: SLO probe exceeds p99 target.
- Recovery: `TEST-SLO-BLOCK` stores latency evidence and blocks promotion.

## Migration Notes

- Source vendor: SAP QM.
- Fixtures include SAP-style plan, lot, result, certificate, notification, and audit records.
- Source vendor: IQS-AQM contributes audit checklist fixtures.
- Source vendor: TIPQA contributes receiving inspection fixtures.
- Source vendor: Sparta Systems TrackWise contributes deviation/CAPA fixtures.
- Source vendor: MasterControl contributes document-control fixtures.
- Source vendor: ETQ Reliance contributes complaint and nonconformance fixtures.
- Migration rejects become negative test fixtures.
- Rollback path: disable fixture execution, retain evidence rows.
- Test fixtures must not include production secrets or raw customer data.

## Cross-microservice Handoffs

- To warehouse: receipt, hold, and stock posting assertions.
- To production-planning: production release inspection assertion.
- To workflow-engine: task and deadline assertions.
- To compliance: audit and clause evidence assertions.
- To ontology: projection freshness assertions.
- To customer-portal: complaint mirror and certificate mirror assertions.
- To supplier-portal: supplier notification route assertions.
- To marketplace: read-only supplier trust assertions.

## Verification

- Unit: fixture hash mismatch blocks run.
- Unit: production topic replay denied.
- Unit: missing audit event fails assertion.
- Contract: REST evidence endpoint returns all assertions.
- Contract: gRPC fixture stream emits terminal state.
- Event: fixture ran event validates.
- Policy: Cedar denies marketplace mutation observation.
- Projection: SAP full-scenario fixture maps field-for-field.
- SLO: fixture run p95 under 45 seconds.
- Evidence: emit `EVT-QUALITY_MANAGEMENT-INTEGRATION_TEST-IP_ACCEPTED`.
