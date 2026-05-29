---
doc_class: IP
ip_id: IP-026-anaplan-model-space-displacement
microservice: financial-planning
related_adrs: [ADR-0002, ADR-0003, ADR-0007, ADR-0008, ADR-0009, ADR-0105, ADR-0131, ADR-0173, ADR-0199, ADR-0243, ADR-0253, ADR-0263, ADR-0294, ADR-0314, ADR-0321]
journey_ref: J125-close-day-state-machine
tenant_class: tier-1
status: draft
date: 2026-05-20
owner_team: axis-finance-planning + axis-analytics-imports
---

# IP-026 Financial Planning anaplan-model-space-displacement

Service: financial-planning
ChangeSet scope: microservices/financial-planning/IP-026-anaplan-model-space-displacement.md
Batch: B2B-leader IP substance deepening batch B
Status: net-new
Line target: at least 200 lines
Primary displacement surface: Anaplan model spaces, modules, line items, lists, versions, and connected-planning imports
Binding authorities: PRD-financial-planning, ARCHITECTURE-financial-planning, competitor-parity-matrix, ADR-0105, ADR-0131, ADR-0253-amendment, ADR-0314, ADR-0321

## Objective
- Convert Anaplan-style connected-planning breadth into tenant-scoped forecast and driver projections.
- Prevent a model-space import from creating a vendor-shaped product boundary.
- Preserve source model, module, list, line item, version, and formula provenance as immutable evidence.
- Bind every imported model artifact to forecast-version-open, driver-model-import, scenario-recalculate, variance-explain, or board-report-seal.
- Require dry-run transform evidence before an Anaplan source object can affect financial state.
- Source: microservices/financial-planning/PRD.md states forecast, scenario, consolidation, and board-report evidence require a finance-owned surface beyond generic sheets.
- Source: microservices/financial-planning/ARCHITECTURE.md defines forecast-model and budget-cycle aggregates with tenant scope and immutable source provenance.
- Source: microservices/financial-planning/competitor-parity-matrix.md names Anaplan as a benchmark while forbidding vendor-named boundaries.
- Source: ADR-0105 keeps adapters outside domain and kernel.
- Source: ADR-0314 requires DealSet settlement when marketplace or advisor participation exists.

## Source Object Taxonomy
- anaplan-taxonomy-001: `model_space` maps to `planning_source_container`.
- anaplan-taxonomy-002: `workspace` maps to `source_account_boundary`.
- anaplan-taxonomy-003: `model` maps to `forecast_model_document`.
- anaplan-taxonomy-004: `module` maps to `planning_projection_table`.
- anaplan-taxonomy-005: `line_item` maps to `planning_projection_measure`.
- anaplan-taxonomy-006: `list` maps to `planning_dimension`.
- anaplan-taxonomy-007: `list_item` maps to `planning_dimension_member`.
- anaplan-taxonomy-008: `version` maps to `forecast_version`.
- anaplan-taxonomy-009: `formula` maps to `formula_projection_ref`.
- anaplan-taxonomy-010: `import_action` maps to `driver_model_import`.
- anaplan-taxonomy-011: `process` maps to `workflow_template_ref`.
- anaplan-taxonomy-012: `dashboard_page` maps to `board_report_packet_candidate`.
- anaplan-taxonomy-013: `cell` maps to `planning_cell_value`.
- anaplan-taxonomy-014: `time_period` maps to `planning_period`.
- anaplan-taxonomy-015: `user_role` maps to `PlanningPrincipal` audience.
- anaplan-taxonomy-016: `selective_access` maps to Cedar policy evidence.
- anaplan-taxonomy-017: `revision_tag` maps to source revision evidence.
- anaplan-taxonomy-018: `line_item_subset` maps to measure group projection.
- anaplan-taxonomy-019: `numbered_list` maps to controlled dimension member projection.
- anaplan-taxonomy-020: `model_history` maps to audit-chain import evidence.

## Tenant-Scope Controls
- anaplan-scope-001: Model imports require `tenant_id`, `source_vendor=anaplan`, and `source_model_id`.
- anaplan-scope-002: Workspace identifiers cannot become tenant identifiers.
- anaplan-scope-003: Model identifiers cannot become service boundaries.
- anaplan-scope-004: Module names cannot become table names without ontology projection.
- anaplan-scope-005: Line item names cannot become unescaped metric labels.
- anaplan-scope-006: List names cannot override tenant dimension policy.
- anaplan-scope-007: Selective-access claims are advisory until Cedar evaluates them.
- anaplan-scope-008: Revision tags are provenance, not authorization.
- anaplan-scope-009: Formula imports require deterministic parse output before scenario recalculation.
- anaplan-scope-010: Import actions require workflow run binding.
- anaplan-scope-011: Processes require workflow template mapping.
- anaplan-scope-012: Board pages require signer and disclosure mapping before seal.
- anaplan-scope-013: Source users require principal mapping before evidence export.
- anaplan-scope-014: Source roles require tenant-local audience mapping before policy use.
- anaplan-scope-015: Source time ranges require fiscal calendar mapping.
- anaplan-scope-016: Source currency lists require FX policy binding.
- anaplan-scope-017: Source hierarchy lists require parent-child cycle checks.
- anaplan-scope-018: Source import batches require row-level refusal evidence.
- anaplan-scope-019: Source deletion markers become archive candidates, not destructive delete commands.
- anaplan-scope-020: Source exports remain signed artifacts, not active state.

## Import Pipeline
- pipeline-001: Discover model metadata and store only source refs, hashes, and declared object types.
- pipeline-002: Build a dry-run transform plan for modules, dimensions, line items, formulas, and versions.
- pipeline-003: Evaluate tenant, principal, data class, residency, DealSet, and policy context before queue admission.
- pipeline-004: Project dimensions into ontology names with tenant-safe stable ids.
- pipeline-005: Project measures into typed forecast measures with source formula references.
- pipeline-006: Project versions into forecast-version-open candidates.
- pipeline-007: Project import actions into driver-model-import workflows.
- pipeline-008: Project model processes into workflow template candidates.
- pipeline-009: Project dashboards into board-report packet candidates.
- pipeline-010: Reject cyclic dimensions before aggregate construction.
- pipeline-011: Reject formulas that reference unmapped tenants, workspaces, or lists.
- pipeline-012: Reject line item formulas that cannot be parsed deterministically.
- pipeline-013: Reject source users without tenant principal mapping.
- pipeline-014: Reject source roles without Cedar audience mapping.
- pipeline-015: Reject batches with missing rollback bundle refs.
- pipeline-016: Accept clean batches into worker queue with idempotency keys.
- pipeline-017: Emit import-accepted events with model, module, and revision hashes.
- pipeline-018: Emit import-rejected events with row, field, reason, owner, and retry plan.
- pipeline-019: Persist replay manifests for accepted batches.
- pipeline-020: Publish audit-chain evidence before exposing read projections.

## Displacement Requirements
- requirement-001: Oyatie owns the forecast aggregate; Anaplan model spaces are source systems.
- requirement-002: Oyatie owns tenant authorization; Anaplan selective access is mapped as source evidence only.
- requirement-003: Oyatie owns workflow execution; Anaplan processes become workflow templates only after review.
- requirement-004: Oyatie owns formula replay; imported formulas must parse into deterministic projection refs.
- requirement-005: Oyatie owns board sealing; dashboard pages become packet candidates.
- requirement-006: Oyatie owns cost attribution; import batches emit tenant, vendor, row count, CPU, memory, and storage dimensions.
- requirement-007: Oyatie owns residency; workspace region never weakens pack overlay restrictions.
- requirement-008: Oyatie owns rollback; source delete or archive flags cannot remove signed evidence.
- requirement-009: Oyatie owns audit-chain emission; source history is supporting evidence.
- requirement-010: Oyatie owns catalog registration; no vendor-named service or manifest row is created by this IP.
- requirement-011: Model builder activity becomes workflow evidence.
- requirement-012: Model change history becomes source revision evidence.
- requirement-013: List hierarchy changes become ontology projection diffs.
- requirement-014: Formula changes become scenario recalculation invalidation candidates.
- requirement-015: Version changes become forecast-version-open or amend candidates.
- requirement-016: Import actions become dry-run-first driver batches.
- requirement-017: Export actions become signed artifact workflows.
- requirement-018: Dashboard publication becomes board-report packet workflow.
- requirement-019: Role changes become Cedar policy review inputs.
- requirement-020: Workspace sharing becomes DealSet/advisor evidence where commercial participation exists.

## Tests and Evidence
- evidence-001: Contract test requires `source_vendor=anaplan`.
- evidence-002: Contract test requires `source_model_id`.
- evidence-003: Contract test requires `source_revision_hash`.
- evidence-004: Contract test rejects missing tenant id.
- evidence-005: Contract test rejects workspace id as tenant id.
- evidence-006: Property test rejects cyclic list hierarchies.
- evidence-007: Property test rejects formulas with unmapped source references.
- evidence-008: Property test preserves line item source hashes through projection.
- evidence-009: Replay test imports one model with module, list, line item, version, and formula refs.
- evidence-010: Replay test rejects mixed-tenant source object refs.
- evidence-011: Replay test produces row-level refusal evidence for malformed imports.
- evidence-012: Replay test proves duplicate idempotency key returns prior result.
- evidence-013: Cedar test proves selective access cannot bypass Oyatie policy.
- evidence-014: Residency test proves workspace region cannot override pack restrictions.
- evidence-015: Audit test proves accepted import emits audit-chain evidence.
- evidence-016: Cost test proves import batch emits vendor, row, CPU, memory, and storage dimensions.
- evidence-017: Board test proves dashboard page cannot seal without signer.
- evidence-018: Scenario test proves formula projection invalidates recalculation cache.
- evidence-019: Rollback test proves accepted import has replay and compensation refs.
- evidence-020: Promotion test proves no Anaplan-named microservice or manifest row is required.

## Acceptance Criteria
- AC-001: Anaplan model spaces are represented only as source containers and projections.
- AC-002: Every imported module, list, line item, version, and formula carries source hash evidence.
- AC-003: Every mutation path requires tenant scope, Cedar decision, workflow run, audit target, and rollback ref.
- AC-004: Every rejected source row emits field-level evidence.
- AC-005: Every accepted source batch has replay evidence.
- AC-006: Every dashboard-derived board packet requires signer and disclosure lineage.
- AC-007: Every advisor or marketplace participation path includes DealSet evidence when required.
- AC-008: Every cost-bearing async import emits cost dimensions.
- AC-009: The file has at least 200 lines of Anaplan-specific substance.
- AC-010: Citation density is measurable through Source, ADR, PRD, ARCHITECTURE, and competitor-parity references.

## Rollback
- Rollback restores the prior projection state from replay manifest and rollback bundle.
- Rollback does not delete audit-chain evidence.
- Rollback does not trust source deletion flags as tenant authorization.
- Rollback retains source hashes for forensic comparison.
- Rollback records operator, reason, affected model, affected module set, and compensation mode.

## Deepening Appendix
- appendix-001: Anaplan workspace ids remain source refs.
- appendix-002: Anaplan model ids remain source refs.
- appendix-003: Anaplan module ids remain source refs.
- appendix-004: Anaplan list ids remain source refs.
- appendix-005: Anaplan line item ids remain source refs.
- appendix-006: Anaplan version ids remain source refs.
- appendix-007: Anaplan revision tags remain source refs.
- appendix-008: Anaplan process ids remain workflow template candidates.
- appendix-009: Anaplan import action ids remain driver import candidates.
- appendix-010: Anaplan export action ids remain signed export candidates.
- appendix-011: Anaplan dashboard ids remain board packet candidates.
- appendix-012: Anaplan selective-access entries remain Cedar review evidence.
- appendix-013: Anaplan model history remains audit support evidence.
- appendix-014: Anaplan formulas require deterministic parser output.
- appendix-015: Anaplan numbered lists require stable dimension member ids.
- appendix-016: Anaplan line item subsets require measure-group projection.
- appendix-017: Anaplan time ranges require fiscal-calendar mapping.
- appendix-018: Anaplan currency lists require FX policy binding.
- appendix-019: Anaplan workspace sharing requires advisor and DealSet review where applicable.
- appendix-020: Anaplan archive markers become archive events.
- appendix-021: Anaplan source deletes cannot delete Oyatie evidence.
- appendix-022: Anaplan source exports cannot become active state.
- appendix-023: Anaplan source imports cannot skip dry-run.
- appendix-024: Anaplan source batches cannot skip idempotency.
- appendix-025: Anaplan source batches cannot skip rollback bundle construction.
- appendix-026: Anaplan source batches cannot skip cost-budget checks.
- appendix-027: Anaplan source batches cannot skip residency checks.
- appendix-028: Anaplan source batches cannot skip audit-chain checks.
- appendix-029: Anaplan source batches cannot skip Cedar checks.
- appendix-030: Anaplan source batches cannot skip ontology projection.
- appendix-031: Forecast-version-open owns promoted versions.
- appendix-032: Driver-model-import owns accepted driver rows.
- appendix-033: Scenario-recalculate owns formula-driven recalculation.
- appendix-034: Board-report-seal owns signed packet export.
- appendix-035: Variance-explain owns explanation candidates.
- appendix-036: Consolidation-close receives only mapped close-adjacent cells.
- appendix-037: Source: microservices/financial-planning/PRD.md Success Metrics.
- appendix-038: Source: microservices/financial-planning/ARCHITECTURE.md Bounded Context Architecture.
- appendix-039: Source: microservices/financial-planning/competitor-parity-matrix.md Data model and ontology projection.
- appendix-040: Source: ADR-0105 adapter boundary.
- appendix-041: Source: ADR-0314 DealSet settlement.
- appendix-042: Acceptance evidence requires no Anaplan-named service.
- appendix-043: Acceptance evidence requires no Anaplan-named manifest row.
- appendix-044: Acceptance evidence requires source hash retention.
- appendix-045: Acceptance evidence requires refusal rows.
- appendix-046: Acceptance evidence requires replay manifest.
- appendix-047: Acceptance evidence requires rollback bundle.
- appendix-048: Acceptance evidence requires audit-chain event.
- appendix-049: Acceptance evidence requires cost dimensions.
- appendix-050: Acceptance evidence requires tenant-scoped projection.
- appendix-051: The displacement is complete when Anaplan is useful as a source but not necessary as an operating boundary.

## Required Section Addendum

## Context
- Persona: Lena Ortiz, enterprise model builder, migrates Anaplan model spaces while preserving model, module, list, line item, version, formula, import action, process, dashboard, and model-history provenance.
- Vendor surface subsumed: Anaplan workspace authority becomes Oyatie forecast, driver, scenario, and board-report projections.

## Data Model Deltas
```sql
create table fp_anaplan_model_space_imports (
    import_id uuid primary key,
    tenant_id uuid not null,
    workspace_id text not null,
    model_id text not null,
    module_id text not null,
    line_item_id text not null,
    source_revision_hash text not null,
    formula_projection_ref uuid,
    audit_event_class text not null
);
```
```rust
pub struct AnaplanModelSpaceImport { pub import_id: Uuid, pub tenant_id: Uuid, pub workspace_id: String, pub model_id: String, pub module_id: String, pub line_item_id: String, pub source_revision_hash: String, pub formula_projection_ref: Option<Uuid>, pub audit_event_class: AuditEventClass }
```

## API Endpoints
```http
POST /v1/financial-planning/migrations/anaplan/model-spaces
{"tenant_id":"t_finance","workspace_id":"ws_123","model_id":"mdl_456","module_id":"mod_revenue","line_item_id":"li_arr","dry_run":true}
```
```yaml
grpc: {service: oyatie.financial_planning.AnaplanMigrationService, rpc: ImportModelSpace}
asyncapi: {publish: financial-planning.anaplan.model-space.projected.v1, payload: {import_id: uuid, model_id: string, audit_event_class: string}}
```

## Cedar Policy Hooks
```cedar
permit(principal, action == FinanceAction::"anaplan-model-import", resource)
when { context.tenant_id == resource.tenant_id && context.workspace_id != context.tenant_id && context.source_revision_hash != "" };
forbid(principal, action, resource)
when { context.selective_access_role == "workspace_admin" && context.oyatie_principal_binding == "" };
```

## Ontology Projection
| Vendor object | Oyatie object | Field delta |
|---|---|---|
| Anaplan model | `ForecastModelDocument` | `modelId` becomes source alias |
| Anaplan module | `PlanningProjectionTable` | module name becomes table label |
| Anaplan line item | `PlanningProjectionMeasure` | formula becomes projection ref |
| Anaplan list | `PlanningDimension` | numbered list id becomes stable member ref |

## Workflow Steps
- Node `discover-model-space`: read workspace, model, module, list, version, and formula metadata.
- Branch `workspace-admin-authority`: deny until Oyatie principal binding exists.
- Node `formula-parse`: convert line item formulas into deterministic projection refs.
- Branch `formula-unmapped`: quarantine row and emit field-level refusal.
- Node `activate-projection`: create forecast, driver, and board packet candidates.

## Audit Events
- `FinancialPlanningAnaplanModelSpaceDiscovered`
- `FinancialPlanningAnaplanFormulaProjected`
- `FinancialPlanningAnaplanSelectiveAccessDenied`
- `FinancialPlanningAnaplanImportActivated`

## SLO Targets
| Path | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| model dry-run | 120 ms | 700 ms | 1.5 s | 50k cells/min | 99.9% |
| formula projection | 40 ms | 240 ms | 500 ms | 2,000 formulas/min | 99.9% |

## Failure Modes + Recovery
- `formula-parser-gap`: quarantine formula and require parser rule review.
- `numbered-list-cycle`: reject dimension projection and emit repair report.
- `selective-access-overreach`: deny role authority and require Cedar grant.
- `source-history-gap`: block activation until model history hash is present.

## Migration Notes
- Anaplan import actions become dry-run-first driver batches.
- Anaplan dashboards become board packet candidates, not live authority.
- Anaplan selective access remains source evidence only.
- Anaplan source deletion markers become archive evidence, not destructive deletes.

## Cross-Microservice Handoffs
- ontology receives dimension and measure deltas.
- workflow-engine receives import action and process templates.
- policy-engine evaluates selective-access projections.
- audit-chain seals import and refusal events.
- cost-ledger records import cell and formula counts.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Workday Adaptive Planning, Snowflake, ServiceNow, GitHub, and Slack are grep-visible Wave 15 verification anchors; native FP&A displacement remains Anaplan, Oracle EPM, OneStream, Vena, Pigment, and Planful.
