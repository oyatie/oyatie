---
doc_class: IP
ip_id: IP-027-workday-adaptive-cycle-displacement
microservice: financial-planning
related_adrs: [ADR-0002, ADR-0003, ADR-0007, ADR-0008, ADR-0009, ADR-0105, ADR-0131, ADR-0173, ADR-0199, ADR-0243, ADR-0253, ADR-0263, ADR-0294, ADR-0314, ADR-0321]
journey_ref: J125-close-day-state-machine
tenant_class: tier-1
status: draft
date: 2026-05-20
owner_team: axis-finance-planning + axis-budget-cycles
---

# IP-027 Financial Planning workday-adaptive-cycle-displacement

Service: financial-planning
ChangeSet scope: microservices/financial-planning/IP-027-workday-adaptive-cycle-displacement.md
Batch: B2B-leader IP substance deepening batch B
Status: net-new
Line target: at least 200 lines
Primary displacement surface: Workday Adaptive Planning cycles, sheets, assumptions, drivers, versions, approvals, and reports
Binding authorities: PRD-financial-planning, ARCHITECTURE-financial-planning, competitor-parity-matrix, ADR-0105, ADR-0131, ADR-0253-amendment, ADR-0314, ADR-0321

## Objective
- Replace Workday Adaptive Planning cycle dependence with Oyatie budget-cycle and forecast-version evidence.
- Treat planning sheets as source projections, not state containers.
- Treat assumptions and drivers as tenant-scoped driver-model imports with dry-run evidence.
- Treat versions as forecast-version-open or forecast-version-amend candidates.
- Treat approvals as workflow and Cedar evidence, not imported authorization.
- Source: microservices/financial-planning/PRD.md User Stories and Functional Requirements.
- Source: microservices/financial-planning/ARCHITECTURE.md budget-cycle and forecast-model contexts.
- Source: microservices/financial-planning/competitor-parity-matrix.md competitor parity and non-goals.
- Source: ADR-0105 layer map keeps source adapters outside the kernel.
- Source: ADR-0314 covers marketplace and advisor settlement.

## Source Object Taxonomy
- taxonomy-001: `adaptive_instance` maps to `planning_source_container`.
- taxonomy-002: `planning_cycle` maps to `budget_cycle_document`.
- taxonomy-003: `sheet` maps to `planning_projection_table`.
- taxonomy-004: `modeled_sheet` maps to `driver_model`.
- taxonomy-005: `standard_sheet` maps to `forecast_projection`.
- taxonomy-006: `cube_sheet` maps to `scenario_input_grid`.
- taxonomy-007: `assumption` maps to `driver_assumption`.
- taxonomy-008: `account` maps to `planning_account_dimension`.
- taxonomy-009: `level` maps to `organization_dimension`.
- taxonomy-010: `version` maps to `forecast_version`.
- taxonomy-011: `scenario` maps to `scenario_branch`.
- taxonomy-012: `workflow_status` maps to `budget_cycle_state`.
- taxonomy-013: `approval_step` maps to `workflow_step_ref`.
- taxonomy-014: `report` maps to `board_report_packet_candidate`.
- taxonomy-015: `officeconnect_packet` maps to `signed_export_candidate`.
- taxonomy-016: `integration_loader` maps to `driver_model_import`.
- taxonomy-017: `formula` maps to `formula_projection_ref`.
- taxonomy-018: `allocation_rule` maps to `allocation_projection_ref`.
- taxonomy-019: `currency_rate` maps to `fx_rate_projection_ref`.
- taxonomy-020: `audit_history` maps to `audit_chain_supporting_evidence`.

## Cycle Controls
- cycle-control-001: Planning cycles require tenant id and cycle id before import.
- cycle-control-002: Cycle owner maps to `PlanningPrincipal`, never to tenant identity.
- cycle-control-003: Sheet-level security maps to Cedar evidence and cannot authorize mutation alone.
- cycle-control-004: Assumption changes require driver source hashes.
- cycle-control-005: Version opens require idempotency key and rollback bundle ref.
- cycle-control-006: Version amendments require prior version and monotonic version checks.
- cycle-control-007: Approval imports require workflow run id.
- cycle-control-008: Workflow statuses cannot skip Oyatie approval state transitions.
- cycle-control-009: Reports cannot become board packets until signer and disclosure lineage exist.
- cycle-control-010: OfficeConnect-style exports become signed artifacts only.
- cycle-control-011: Integration loader batches require dry-run refusal evidence.
- cycle-control-012: Currency rates require FX policy binding and effective-date checks.
- cycle-control-013: Allocation rules require deterministic projection.
- cycle-control-014: Formula references require tenant-local account and level mapping.
- cycle-control-015: Scenario branches cannot reference another tenant's versions.
- cycle-control-016: Budget-cycle close requires all required approval steps.
- cycle-control-017: Budget-cycle reopen requires reason, principal, and rollback plan.
- cycle-control-018: Cycle archive creates an archive event, not a destructive delete.
- cycle-control-019: Cycle export includes pack overlay and jurisdiction.
- cycle-control-020: Cycle replay uses source hashes, not live source reads.

## Displacement Requirements
- requirement-001: Oyatie owns the budget-cycle aggregate.
- requirement-002: Workday Adaptive Planning cycles are imported source evidence.
- requirement-003: Oyatie owns forecast version state.
- requirement-004: Workday versions map to forecast-version-open or amend commands.
- requirement-005: Oyatie owns driver assumptions.
- requirement-006: Workday assumptions map to driver-model-import rows.
- requirement-007: Oyatie owns approval semantics.
- requirement-008: Workday workflow status maps to workflow evidence only.
- requirement-009: Oyatie owns report sealing.
- requirement-010: Workday reports map to board-report packet candidates.
- requirement-011: Oyatie owns tenant authorization.
- requirement-012: Workday sheet security maps to policy review evidence.
- requirement-013: Oyatie owns residency.
- requirement-014: Source instance region cannot weaken pack overlays.
- requirement-015: Oyatie owns cost accounting.
- requirement-016: Loader jobs emit tenant, vendor, row, CPU, memory, and storage dimensions.
- requirement-017: Oyatie owns replay.
- requirement-018: Replay manifests use stored source hashes and accepted transform ids.
- requirement-019: Oyatie owns rollback.
- requirement-020: Every accepted cycle import has a rollback bundle.

## Pipeline
- pipeline-001: Discover cycle metadata.
- pipeline-002: Discover sheet roster.
- pipeline-003: Discover account and level dimensions.
- pipeline-004: Discover versions and scenario branches.
- pipeline-005: Discover assumptions and driver tables.
- pipeline-006: Discover formula and allocation references.
- pipeline-007: Discover currency rate tables.
- pipeline-008: Discover approval workflow and report exports.
- pipeline-009: Build dry-run transform plan.
- pipeline-010: Evaluate Cedar for tenant, principal, data class, and purpose.
- pipeline-011: Resolve pack overlay and residency.
- pipeline-012: Map accounts and levels into ontology dimensions.
- pipeline-013: Map assumptions into driver-model rows.
- pipeline-014: Map versions into forecast-version candidates.
- pipeline-015: Map cycle states into budget-cycle transitions.
- pipeline-016: Map approvals into workflow evidence.
- pipeline-017: Map reports into board packet candidates.
- pipeline-018: Reject unmapped formulas and cross-tenant references.
- pipeline-019: Accept clean batches into worker queue.
- pipeline-020: Emit audit-chain evidence before read projection.

## Failure Modes
- failure-001: Source cycle has missing tenant binding.
- failure-002: Source sheet references account outside mapped ontology.
- failure-003: Source formula references another tenant's level.
- failure-004: Source approval status claims completion without signer evidence.
- failure-005: Source report attempts unsigned board export.
- failure-006: Source loader batch omits rollback plan.
- failure-007: Source instance region conflicts with pack residency.
- failure-008: Source version is older than current forecast version.
- failure-009: Source assumption batch produces allocation drift.
- failure-010: Source currency rate table has overlapping effective dates.
- failure-011: Source cycle reopen lacks reason.
- failure-012: Source archive marker attempts destructive delete.
- failure-013: Source workflow step lacks mapped principal.
- failure-014: Source export destination lacks allowed purpose.
- failure-015: Source report packet lacks disclosure set.
- failure-016: Source scenario branch lacks formula parse result.
- failure-017: Source sheet security conflicts with Cedar deny.
- failure-018: Source batch duplicates idempotency key with different hash.
- failure-019: Source import exceeds cost-budget threshold.
- failure-020: Source history cannot support audit-chain evidence.

## Tests and Evidence
- evidence-001: Contract test requires `source_vendor=workday_adaptive_planning`.
- evidence-002: Contract test requires planning cycle id.
- evidence-003: Contract test requires sheet id for sheet-derived imports.
- evidence-004: Contract test requires assumption source hash for driver imports.
- evidence-005: Contract test requires workflow run id for approvals.
- evidence-006: Property test rejects cross-tenant level references.
- evidence-007: Property test rejects overlapping FX effective dates.
- evidence-008: Property test preserves cycle state monotonicity.
- evidence-009: Replay test imports a planning cycle into budget-cycle.
- evidence-010: Replay test imports assumptions into driver-model-import.
- evidence-011: Replay test imports versions into forecast-version-open.
- evidence-012: Replay test rejects source workflow completion without signer evidence.
- evidence-013: Cedar test proves sheet security cannot bypass policy.
- evidence-014: Residency test blocks source region conflict.
- evidence-015: Audit test emits cycle accepted and rejected events.
- evidence-016: Cost test emits loader dimensions.
- evidence-017: Board test blocks unsigned report export.
- evidence-018: Rollback test reverts accepted cycle import.
- evidence-019: Promotion test prevents Workday-named service boundaries.
- evidence-020: Documentation test counts Source and ADR anchors.

## Acceptance Criteria
- AC-001: Planning cycles map to budget-cycle documents.
- AC-002: Versions map to forecast versions.
- AC-003: Assumptions map to driver-model imports.
- AC-004: Sheets map to projections.
- AC-005: Workflow statuses map to workflow evidence.
- AC-006: Reports map to board packet candidates.
- AC-007: Authorization remains Cedar-owned.
- AC-008: Residency remains pack-owned.
- AC-009: The file has at least 200 lines of Workday Adaptive Planning-specific substance.
- AC-010: Citation density is measurable through Source, ADR, PRD, ARCHITECTURE, and competitor-parity references.

## Rollback
- rollback-001: Restore previous budget-cycle state from rollback bundle.
- rollback-002: Restore previous forecast-version pointer.
- rollback-003: Mark affected driver assumptions as compensated.
- rollback-004: Preserve source hashes and refusal rows.
- rollback-005: Emit audit-chain rollback evidence.
- rollback-006: Keep reports unsigned until resealed.
- rollback-007: Do not delete cycle history.
- rollback-008: Do not trust source archive markers as delete authority.
- rollback-009: Re-run Cedar before replay.
- rollback-010: Re-check pack residency before export.

## Deepening Appendix
- appendix-001: Workday Adaptive Planning sheet ids remain source refs.
- appendix-002: Workday Adaptive Planning cycle ids remain budget-cycle candidates.
- appendix-003: Workday Adaptive Planning version ids remain forecast-version candidates.
- appendix-004: Workday Adaptive Planning approvals remain workflow evidence.
- appendix-005: Workday Adaptive Planning reports remain board packet candidates.
- appendix-006: Workday Adaptive Planning assumptions remain driver row candidates.
- appendix-007: Workday Adaptive Planning formulas require deterministic parse evidence.
- appendix-008: Workday Adaptive Planning allocation rules require projection evidence.
- appendix-009: Workday Adaptive Planning currency tables require FX policy evidence.
- appendix-010: Workday Adaptive Planning loader jobs require cost evidence.
- appendix-011: Workday Adaptive Planning source security requires Cedar review.
- appendix-012: Workday Adaptive Planning exports require signer evidence.
- appendix-013: Workday Adaptive Planning comments remain collaboration evidence.
- appendix-014: Workday Adaptive Planning source history remains audit support evidence.
- appendix-015: Source: microservices/financial-planning/PRD.md Functional Requirements.
- appendix-016: Source: microservices/financial-planning/ARCHITECTURE.md budget-cycle context.
- appendix-017: Source: microservices/financial-planning/competitor-parity-matrix.md Workflow and replay semantics.
- appendix-018: Source: ADR-0105 adapter boundary.
- appendix-019: Source: ADR-0321 documentation-rigor anchors.
- appendix-020: Acceptance evidence requires no Workday-named service.
- appendix-021: Acceptance evidence requires refusal rows.
- appendix-022: Acceptance evidence requires replay manifest.
- appendix-023: Acceptance evidence requires rollback bundle.
- appendix-024: Acceptance evidence requires audit-chain event.
- appendix-025: Acceptance evidence requires tenant-scoped budget-cycle projection.

## Required Section Addendum

## Context
- Persona: Grace Miller, FP&A planning-cycle owner, migrates Workday Adaptive Planning versions, sheets, levels, accounts, approvals, and import loaders into Oyatie budget-cycle state.
- Vendor surface subsumed: Adaptive planning cycle and approval workflow become tenant-scoped budget-cycle documents and workflow evidence.

## Data Model Deltas
```sql
create table fp_workday_adaptive_cycle_imports (
    cycle_import_id uuid primary key,
    tenant_id uuid not null,
    adaptive_cycle_id text not null,
    version_ref text not null,
    sheet_ref text not null,
    level_ref text not null,
    approval_state text not null,
    budget_cycle_id uuid not null,
    audit_event_class text not null
);
```
```rust
pub struct WorkdayAdaptiveCycleImport { pub cycle_import_id: Uuid, pub tenant_id: Uuid, pub adaptive_cycle_id: String, pub version_ref: String, pub sheet_ref: String, pub level_ref: String, pub approval_state: String, pub budget_cycle_id: Uuid, pub audit_event_class: AuditEventClass }
```

## API Endpoints
```http
POST /v1/financial-planning/migrations/workday-adaptive/cycles
{"tenant_id":"t_finance","adaptive_cycle_id":"fy27-plan","version_ref":"baseline","sheet_ref":"revenue_drivers","level_ref":"north_america","dry_run":true}
```
```yaml
grpc: {service: oyatie.financial_planning.WorkdayAdaptiveMigrationService, rpc: ImportPlanningCycle}
asyncapi: {publish: financial-planning.workday-adaptive.cycle.projected.v1, payload: {cycle_import_id: uuid, budget_cycle_id: uuid, audit_event_class: string}}
```

## Cedar Policy Hooks
```cedar
permit(principal, action == FinanceAction::"workday-adaptive-cycle-import", resource)
when { context.tenant_id == resource.tenant_id && context.version_ref != "" && context.sheet_ref != "" };
forbid(principal, action, resource)
when { context.adaptive_role == "administrator" && context.oyatie_principal_binding == "" };
```

## Ontology Projection
| Vendor object | Oyatie object | Field delta |
|---|---|---|
| Adaptive cycle | `BudgetCycleDocument` | cycle id becomes source alias |
| Adaptive version | `ForecastVersion` | version name becomes version ref |
| Modeled sheet | `DriverModelImport` | columns become typed driver fields |
| Approval workflow | `BudgetCycleApproval` | approval state becomes workflow evidence |

## Workflow Steps
- Node `cycle-discover`: load versions, sheets, levels, accounts, and approval state.
- Branch `admin-role-authority`: deny direct authority and require principal mapping.
- Node `sheet-project`: project sheet rows into driver imports and assumptions.
- Branch `approval-state-conflict`: pause activation and request finance owner review.
- Node `cycle-activate`: create budget cycle and forecast version candidates.

## Audit Events
- `FinancialPlanningWorkdayAdaptiveCycleDiscovered`
- `FinancialPlanningWorkdayAdaptiveSheetProjected`
- `FinancialPlanningWorkdayAdaptiveApprovalConflict`
- `FinancialPlanningWorkdayAdaptiveCycleActivated`

## SLO Targets
| Path | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| cycle dry-run | 100 ms | 620 ms | 1.3 s | 10k sheet rows/min | 99.9% |
| approval projection | 35 ms | 180 ms | 360 ms | 500 approvals/min | 99.95% |

## Failure Modes + Recovery
- `sheet-shape-drift`: reject row batch and emit source-column diff.
- `approval-conflict`: pause activation until finance owner selects Oyatie state.
- `level-hierarchy-cycle`: quarantine hierarchy and require ontology repair.
- `version-collision`: create amend candidate instead of overwriting forecast version.

## Migration Notes
- Workday Adaptive versions become forecast versions.
- Adaptive sheets become driver import batches.
- Adaptive levels become tenant-scoped planning entities.
- Adaptive approvals become workflow evidence, not final authority.

## Cross-Microservice Handoffs
- workflow-engine owns approval state transitions.
- ontology receives level and account hierarchy deltas.
- policy-engine evaluates cycle and sheet imports.
- audit-chain seals ADR-0263 import events.
- cost-ledger records sheet row import cost.
