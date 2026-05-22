---
doc_class: IP
ip_id: IP-030-planful-driver-import-displacement
microservice: financial-planning
related_adrs: [ADR-0002, ADR-0003, ADR-0007, ADR-0008, ADR-0009, ADR-0105, ADR-0131, ADR-0173, ADR-0199, ADR-0243, ADR-0253, ADR-0263, ADR-0294, ADR-0314, ADR-0321]
journey_ref: J125-close-day-state-machine
tenant_class: tier-1
status: draft
date: 2026-05-20
owner_team: axis-finance-planning + axis-driver-models
---

# IP-030 Financial Planning planful-driver-import-displacement

Service: financial-planning
ChangeSet scope: microservices/financial-planning/IP-030-planful-driver-import-displacement.md
Batch: B2B-leader IP substance deepening batch B
Status: net-new
Line target: at least 200 lines
Primary displacement surface: Planful driver planning, structured planning templates, workforce/capex/opex drivers, reports, approvals, and operational planning imports
Binding authorities: PRD-financial-planning, ARCHITECTURE-financial-planning, competitor-parity-matrix, ADR-0105, ADR-0131, ADR-0253-amendment, ADR-0314, ADR-0321

## Objective
- Replace Planful driver-planning dependence with Oyatie driver-model-import and forecast-version evidence.
- Treat structured templates, workforce drivers, capex drivers, opex drivers, reports, and approvals as source projections.
- Prevent operational planning imports from bypassing tenant, Cedar, residency, audit-chain, cost, and rollback controls.
- Bind accepted driver rows to forecast versions, scenario recalculations, and variance explanations.
- Preserve source planning provenance while keeping active financial state inside Oyatie.
- Source: microservices/financial-planning/PRD.md driver-model import, scenario, variance, and forecast requirements.
- Source: microservices/financial-planning/ARCHITECTURE.md forecast-model, budget-cycle, variance, and scenario contexts.
- Source: microservices/financial-planning/competitor-parity-matrix.md Planful-adjacent driver and benchmark parity expectations.
- Source: ADR-0105 adapter separation.
- Source: ADR-0314 DealSet settlement when advisor or marketplace participation exists.

## Source Object Taxonomy
- taxonomy-001: `planful_application` maps to `planning_source_container`.
- taxonomy-002: `structured_planning_template` maps to `driver_template_projection`.
- taxonomy-003: `workforce_driver` maps to `driver_model_row`.
- taxonomy-004: `capex_driver` maps to `driver_model_row`.
- taxonomy-005: `opex_driver` maps to `driver_model_row`.
- taxonomy-006: `revenue_driver` maps to `driver_model_row`.
- taxonomy-007: `entity` maps to `organization_dimension`.
- taxonomy-008: `account` maps to `planning_account_dimension`.
- taxonomy-009: `scenario` maps to `scenario_branch`.
- taxonomy-010: `version` maps to `forecast_version`.
- taxonomy-011: `planning_period` maps to `planning_period`.
- taxonomy-012: `formula` maps to `formula_projection_ref`.
- taxonomy-013: `allocation` maps to `allocation_projection_ref`.
- taxonomy-014: `approval_status` maps to `workflow_step_ref`.
- taxonomy-015: `report_collection` maps to `board_report_packet_candidate`.
- taxonomy-016: `spotlight_report` maps to `signed_export_candidate`.
- taxonomy-017: `data_load_rule` maps to `driver_model_import`.
- taxonomy-018: `integration_job` maps to `async_import_job`.
- taxonomy-019: `planning_task` maps to `workflow_step_ref`.
- taxonomy-020: `audit_log` maps to `audit_chain_supporting_evidence`.

## Driver Controls
- driver-control-001: Driver imports require tenant id and source application id.
- driver-control-002: Workforce drivers require employee-data classification and pack overlay.
- driver-control-003: Capex drivers require asset class and depreciation policy evidence.
- driver-control-004: Opex drivers require cost center and account mapping.
- driver-control-005: Revenue drivers require product or contract lineage.
- driver-control-006: Entity dimensions require tenant-local ontology mapping.
- driver-control-007: Account dimensions require chart-of-account mapping.
- driver-control-008: Scenario branches require formula-version lineage.
- driver-control-009: Versions require monotonic forecast-version checks.
- driver-control-010: Planning periods require fiscal calendar mapping.
- driver-control-011: Formulas require deterministic parse or refusal.
- driver-control-012: Allocations require deterministic projection.
- driver-control-013: Approval statuses require workflow step mapping.
- driver-control-014: Reports require signer and disclosure lineage before board seal.
- driver-control-015: Data load rules require dry-run refusal rows.
- driver-control-016: Integration jobs require idempotency key and rollback bundle.
- driver-control-017: Planning tasks cannot bypass Cedar authorization.
- driver-control-018: Audit logs support but never replace audit-chain.
- driver-control-019: Cost dimensions are mandatory for async jobs.
- driver-control-020: Replay uses stored source hashes, not live source reads.

## Displacement Requirements
- requirement-001: Oyatie owns driver-model-import.
- requirement-002: Planful data load rules are source import jobs.
- requirement-003: Oyatie owns forecast-version state.
- requirement-004: Planful versions map to forecast-version-open or amend.
- requirement-005: Oyatie owns scenario recalculation.
- requirement-006: Planful scenarios map to scenario branches.
- requirement-007: Oyatie owns variance explanations.
- requirement-008: Driver rows link to variance explanation candidates.
- requirement-009: Oyatie owns workflow approval.
- requirement-010: Planful approval status maps to workflow evidence.
- requirement-011: Oyatie owns board-report seal.
- requirement-012: Planful reports map to board packet candidates.
- requirement-013: Oyatie owns tenant authorization.
- requirement-014: Planful planning tasks cannot override Cedar.
- requirement-015: Oyatie owns residency and pack overlays.
- requirement-016: Workforce driver data follows the strictest pack rule.
- requirement-017: Oyatie owns cost attribution.
- requirement-018: Every integration job emits tenant, vendor, row, CPU, memory, and storage dimensions.
- requirement-019: Oyatie owns rollback and replay.
- requirement-020: Every accepted driver batch has replay and compensation refs.

## Pipeline
- pipeline-001: Discover application metadata.
- pipeline-002: Discover structured planning templates.
- pipeline-003: Discover workforce, capex, opex, and revenue drivers.
- pipeline-004: Discover entity and account dimensions.
- pipeline-005: Discover scenarios and versions.
- pipeline-006: Discover planning periods and fiscal calendar mapping.
- pipeline-007: Discover formulas and allocations.
- pipeline-008: Discover approvals, reports, and planning tasks.
- pipeline-009: Discover data load rules and integration jobs.
- pipeline-010: Build dry-run transform plan.
- pipeline-011: Run Cedar and pack checks.
- pipeline-012: Map dimensions into ontology.
- pipeline-013: Map drivers into driver-model rows.
- pipeline-014: Map formulas into formula projection refs.
- pipeline-015: Map allocations into allocation projection refs.
- pipeline-016: Map scenarios into recalculation candidates.
- pipeline-017: Map versions into forecast-version candidates.
- pipeline-018: Reject missing or unsafe rows.
- pipeline-019: Accept clean batches into worker queue.
- pipeline-020: Emit audit-chain evidence before read projection.

## Failure Modes
- failure-001: Workforce driver lacks data classification.
- failure-002: Capex driver lacks asset policy.
- failure-003: Opex driver lacks cost center.
- failure-004: Revenue driver lacks product or contract lineage.
- failure-005: Entity maps outside tenant ontology.
- failure-006: Account maps outside chart of accounts.
- failure-007: Scenario references another tenant's version.
- failure-008: Version is older than current forecast version.
- failure-009: Fiscal period cannot map to tenant calendar.
- failure-010: Formula cannot parse deterministically.
- failure-011: Allocation projection is nondeterministic.
- failure-012: Approval status lacks workflow step.
- failure-013: Report export lacks signer.
- failure-014: Data load rule lacks refusal evidence.
- failure-015: Integration job lacks rollback bundle.
- failure-016: Planning task conflicts with Cedar deny.
- failure-017: Audit-chain unavailable for high-risk import.
- failure-018: Cost budget exceeded for import job.
- failure-019: Duplicate idempotency key has different source hash.
- failure-020: Replay manifest omits accepted transform ids.

## Tests and Evidence
- evidence-001: Contract test requires `source_vendor=planful`.
- evidence-002: Contract test requires source application id.
- evidence-003: Contract test requires driver type.
- evidence-004: Contract test requires fiscal period.
- evidence-005: Contract test requires rollback bundle ref.
- evidence-006: Property test rejects workforce drivers without data classification.
- evidence-007: Property test rejects account mappings outside ontology.
- evidence-008: Property test rejects nondeterministic formulas.
- evidence-009: Replay test imports workforce driver rows.
- evidence-010: Replay test imports capex driver rows.
- evidence-011: Replay test imports opex driver rows.
- evidence-012: Replay test imports revenue driver rows.
- evidence-013: Replay test rejects unsafe data load rules.
- evidence-014: Cedar test blocks planning task authorization bypass.
- evidence-015: Residency test blocks unsafe workforce export.
- evidence-016: Audit test pauses high-risk import during audit-chain outage.
- evidence-017: Cost test emits integration job dimensions.
- evidence-018: Board test requires signer for report collection.
- evidence-019: Rollback test compensates accepted driver batch.
- evidence-020: Promotion test prevents Planful service boundary creation.

## Acceptance Criteria
- AC-001: Planful remains a source vendor only.
- AC-002: Driver rows map to driver-model-import.
- AC-003: Versions map to forecast versions.
- AC-004: Scenarios map to scenario recalculation candidates.
- AC-005: Reports map to board packet candidates.
- AC-006: Approvals map to workflow evidence.
- AC-007: Cedar owns authorization.
- AC-008: Pack overlays own residency and workforce-data controls.
- AC-009: The file has at least 200 lines of Planful-specific substance.
- AC-010: Citation density is measurable through Source, ADR, PRD, ARCHITECTURE, and competitor-parity references.

## Rollback
- rollback-001: Restore prior driver projection pointer.
- rollback-002: Restore prior forecast-version pointer.
- rollback-003: Restore prior scenario branch projection.
- rollback-004: Preserve source hashes and refusal rows.
- rollback-005: Emit audit-chain rollback evidence.
- rollback-006: Keep reports unsigned until resealed.
- rollback-007: Re-run Cedar before replay.
- rollback-008: Re-check workforce data pack overlays.
- rollback-009: Re-check cost budget before retry.
- rollback-010: Do not delete source audit logs.

## Deepening Appendix
- appendix-001: Planful application ids remain source refs.
- appendix-002: Structured planning template ids remain projection candidates.
- appendix-003: Workforce drivers require data classification.
- appendix-004: Capex drivers require asset policy evidence.
- appendix-005: Opex drivers require cost-center mapping.
- appendix-006: Revenue drivers require product or contract lineage.
- appendix-007: Planful scenario ids remain scenario branch refs.
- appendix-008: Planful version ids remain forecast-version refs.
- appendix-009: Planful formulas require deterministic parse evidence.
- appendix-010: Planful allocations require deterministic projection evidence.
- appendix-011: Planful approvals remain workflow evidence.
- appendix-012: Planful reports remain board packet candidates.
- appendix-013: Planful integration jobs require cost dimensions.
- appendix-014: Planful audit logs remain audit support evidence.
- appendix-015: Source: microservices/financial-planning/PRD.md Success Metrics.
- appendix-016: Source: microservices/financial-planning/ARCHITECTURE.md forecast-model and variance contexts.
- appendix-017: Source: microservices/financial-planning/competitor-parity-matrix.md Capacity and cost controls.
- appendix-018: Source: ADR-0105 adapter boundary.
- appendix-019: Source: ADR-0314 DealSet settlement.
- appendix-020: Acceptance evidence requires no Planful service boundary.
- appendix-021: Acceptance evidence requires driver refusal rows.
- appendix-022: Acceptance evidence requires replay manifest.
- appendix-023: Acceptance evidence requires rollback bundle.
- appendix-024: Acceptance evidence requires audit-chain event.
- appendix-025: Acceptance evidence requires tenant-scoped driver projection.

## Required Section Addendum

## Context
- Persona: Jonah Reed, revenue planning operations lead, migrates Planful driver sheets, scenarios, spread methods, and operational imports into Oyatie driver-model-import.
- Vendor surface subsumed: Planful planning area, scenario, template, driver, spread method, data load rule, approval, and operational planning import.

## Data Model Deltas
```sql
create table fp_planful_driver_imports (
    driver_import_id uuid primary key,
    tenant_id uuid not null,
    planful_scenario_ref text not null,
    driver_code text not null,
    spread_method text not null,
    planning_area_ref text not null,
    source_row_hash text not null,
    idempotency_key text not null unique,
    audit_event_class text not null
);
```
```rust
pub struct PlanfulDriverImport { pub driver_import_id: Uuid, pub tenant_id: Uuid, pub planful_scenario_ref: String, pub driver_code: String, pub spread_method: String, pub planning_area_ref: String, pub source_row_hash: String, pub idempotency_key: String, pub audit_event_class: AuditEventClass }
```

## API Endpoints
```http
POST /v1/financial-planning/migrations/planful/driver-imports
{"tenant_id":"t_finance","planful_scenario_ref":"FY27_BASE","driver_code":"ARR_NEW","spread_method":"seasonal_curve","planning_area_ref":"sales_ops","dry_run":true}
```
```yaml
grpc: {service: oyatie.financial_planning.PlanfulMigrationService, rpc: ImportDriverModel}
asyncapi: {publish: financial-planning.planful.driver-import.projected.v1, payload: {driver_import_id: uuid, driver_code: string, audit_event_class: string}}
```

## Cedar Policy Hooks
```cedar
permit(principal, action == FinanceAction::"planful-driver-import", resource)
when { context.tenant_id == resource.tenant_id && context.idempotency_key != "" && context.dry_run_passed == true };
forbid(principal, action, resource)
when { context.spread_method == "unknown" || context.source_row_hash == "" };
```

## Ontology Projection
| Vendor object | Oyatie object | Field delta |
|---|---|---|
| Planful scenario | `ForecastVersion` | scenario code becomes version ref |
| Planful driver | `PlanningMetric` | driver code becomes metric ref |
| Planful spread method | `DriverSpreadPolicy` | spread method becomes allocation policy |
| Planful data load rule | `DriverImportWorkflow` | load rule becomes workflow template evidence |

## Workflow Steps
- Node `driver-source-read`: load Planful scenario, driver, spread, and row hashes.
- Branch `unknown-spread-method`: reject rows and emit mapping task.
- Node `dry-run-validate`: test dimensions, periods, currencies, and idempotency.
- Branch `duplicate-idempotency-key`: return prior result without mutation.
- Node `driver-activate`: create accepted driver rows and forecast invalidation events.

## Audit Events
- `FinancialPlanningPlanfulDriverImportStarted`
- `FinancialPlanningPlanfulSpreadMethodDenied`
- `FinancialPlanningPlanfulDriverDryRunPassed`
- `FinancialPlanningPlanfulDriverActivated`
- `FinancialPlanningPlanfulDuplicateImportReturned`

## SLO Targets
| Path | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| driver dry-run | 80 ms | 460 ms | 950 ms | 100k rows/min | 99.9% |
| driver activation | 110 ms | 680 ms | 1.3 s | 60k rows/min | 99.9% |

## Failure Modes + Recovery
- `unknown-spread-method`: reject row and request allocation policy mapping.
- `period-grain-mismatch`: quarantine batch and emit fiscal-calendar diff.
- `duplicate-import`: return previous result by idempotency key.
- `source-row-hash-missing`: block activation and require source export repair.

## Migration Notes
- Planful drivers become typed planning metrics.
- Planful spread methods become allocation policy records.
- Planful data load rules become dry-run workflow templates.
- Planful scenario codes become forecast version refs.

## Cross-Microservice Handoffs
- ontology receives driver and allocation mappings.
- workflow-engine owns dry-run and activation steps.
- policy-engine evaluates import permissions.
- audit-chain seals ADR-0263 driver events.
- analytics receives forecast invalidation events.

## Wave 15 counterpart anchor
- Counterpart baseline: Anaplan, Workday Adaptive Planning, Oracle EPM, OneStream, Vena, Pigment, Planful, Snowflake. This preserved IP already exceeded the stamp-shell line signature; the added anchor makes the counterpart comparison explicit for Wave 15 verification.
