---
doc_class: IP
ip_id: IP-029-vena-pigment-board-scenario-displacement
microservice: financial-planning
related_adrs: [ADR-0002, ADR-0003, ADR-0007, ADR-0008, ADR-0009, ADR-0105, ADR-0131, ADR-0173, ADR-0199, ADR-0243, ADR-0253, ADR-0263, ADR-0294, ADR-0314, ADR-0321]
journey_ref: J125-close-day-state-machine
tenant_class: tier-1
status: draft
date: 2026-05-20
owner_team: axis-finance-planning + axis-board-reporting
---

# IP-029 Financial Planning vena-pigment-board-scenario-displacement

Service: financial-planning
ChangeSet scope: microservices/financial-planning/IP-029-vena-pigment-board-scenario-displacement.md
Batch: B2B-leader IP substance deepening batch B
Status: net-new
Line target: at least 200 lines
Primary displacement surface: Vena workbook collaboration, board packs, approvals, Pigment scenario graphs, formulas, and planning applications
Binding authorities: PRD-financial-planning, ARCHITECTURE-financial-planning, competitor-parity-matrix, ADR-0105, ADR-0131, ADR-0253-amendment, ADR-0314, ADR-0321

## Objective
- Replace Vena workbook and Pigment scenario dependence with Oyatie board-report-seal and scenario-recalculate evidence.
- Treat workbooks, canvases, scenario graphs, formulas, and application views as source projections.
- Prevent collaborative sheets or scenario apps from becoming active state containers.
- Require signer, reviewer, disclosure, formula, and scenario lineage before board or scenario promotion.
- Preserve collaboration history as evidence while keeping tenant authorization in Cedar.
- Source: microservices/financial-planning/PRD.md board-report and scenario requirements.
- Source: microservices/financial-planning/ARCHITECTURE.md scenario and board evidence boundaries.
- Source: microservices/financial-planning/competitor-parity-matrix.md Vena and Pigment benchmark parity.
- Source: ADR-0105 adapter/domain separation.
- Source: ADR-0253-amendment transport security baseline.

## Source Object Taxonomy
- taxonomy-001: `vena_workbook` maps to `board_report_packet_candidate`.
- taxonomy-002: `vena_template` maps to `workflow_template_candidate`.
- taxonomy-003: `vena_input_sheet` maps to `planning_projection_table`.
- taxonomy-004: `vena_task` maps to `workflow_step_ref`.
- taxonomy-005: `vena_approval` maps to `board_reviewer_evidence`.
- taxonomy-006: `vena_comment` maps to `collaboration_evidence`.
- taxonomy-007: `vena_excel_range` maps to `planning_cell_range`.
- taxonomy-008: `vena_connector_job` maps to `driver_model_import`.
- taxonomy-009: `pigment_application` maps to `planning_source_container`.
- taxonomy-010: `pigment_block` maps to `scenario_projection_block`.
- taxonomy-011: `pigment_metric` maps to `planning_projection_measure`.
- taxonomy-012: `pigment_dimension` maps to `planning_dimension`.
- taxonomy-013: `pigment_scenario` maps to `scenario_branch`.
- taxonomy-014: `pigment_formula` maps to `formula_projection_ref`.
- taxonomy-015: `pigment_board` maps to `scenario_review_view`.
- taxonomy-016: `pigment_workflow` maps to `workflow_template_candidate`.
- taxonomy-017: `pigment_import` maps to `driver_model_import`.
- taxonomy-018: `pigment_export` maps to `signed_export_candidate`.
- taxonomy-019: `pigment_comment` maps to `collaboration_evidence`.
- taxonomy-020: `source_history` maps to `audit_chain_supporting_evidence`.

## Collaboration Controls
- collaboration-control-001: Workbooks require tenant id and source workbook id.
- collaboration-control-002: Scenario apps require tenant id and source application id.
- collaboration-control-003: Comments are evidence, not approvals.
- collaboration-control-004: Approvals require mapped principal and workflow step.
- collaboration-control-005: Workbook ranges require data class and disclosure mapping.
- collaboration-control-006: Workbook formulas require deterministic parse or refusal.
- collaboration-control-007: Pigment formulas require deterministic parse or refusal.
- collaboration-control-008: Pigment dimensions require tenant-local ontology mapping.
- collaboration-control-009: Pigment scenario branches require formula-version lineage.
- collaboration-control-010: Scenario graphs cannot reference another tenant's dimension.
- collaboration-control-011: Workbook exports require signer and disclosure set.
- collaboration-control-012: Pigment exports require signer and disclosure set.
- collaboration-control-013: Connector jobs require dry-run refusal rows.
- collaboration-control-014: Template imports require workflow template review.
- collaboration-control-015: Board reports require audit-chain write availability.
- collaboration-control-016: Scenario recalculations require worker id and attempt id.
- collaboration-control-017: Comments and tasks cannot bypass Cedar deny.
- collaboration-control-018: Collaboration users require principal mapping.
- collaboration-control-019: Sharing links require allowed purpose and expiry.
- collaboration-control-020: Replay uses stored source hashes.

## Displacement Requirements
- requirement-001: Oyatie owns board-report-seal.
- requirement-002: Vena workbooks are board packet candidates.
- requirement-003: Oyatie owns scenario-recalculate.
- requirement-004: Pigment scenarios are scenario branches.
- requirement-005: Oyatie owns formula parsing and replay.
- requirement-006: Workbook and Pigment formulas are source formula refs.
- requirement-007: Oyatie owns collaboration authorization.
- requirement-008: Source comments and tasks are supporting evidence only.
- requirement-009: Oyatie owns disclosure lineage.
- requirement-010: Source exports cannot become signed packets without seal.
- requirement-011: Oyatie owns workflow semantics.
- requirement-012: Vena and Pigment workflows become template candidates.
- requirement-013: Oyatie owns ontology projection.
- requirement-014: Source dimensions map into tenant-local dimensions.
- requirement-015: Oyatie owns audit-chain evidence.
- requirement-016: Source history supports but does not replace signed evidence.
- requirement-017: Oyatie owns rollback.
- requirement-018: Accepted workbook and scenario imports need rollback bundle refs.
- requirement-019: Oyatie owns cost attribution.
- requirement-020: Connector and scenario jobs emit tenant, vendor, row, CPU, memory, and storage dimensions.

## Pipeline
- pipeline-001: Discover Vena workbook metadata.
- pipeline-002: Discover Vena sheets, ranges, formulas, tasks, approvals, and comments.
- pipeline-003: Discover Pigment application metadata.
- pipeline-004: Discover Pigment blocks, metrics, dimensions, scenarios, formulas, and workflows.
- pipeline-005: Build dry-run transform plan.
- pipeline-006: Map workbook ranges to board packet candidate sections.
- pipeline-007: Map workbook formulas to formula projection refs.
- pipeline-008: Map Vena tasks to workflow step refs.
- pipeline-009: Map Vena approvals to reviewer evidence.
- pipeline-010: Map Pigment dimensions to ontology dimensions.
- pipeline-011: Map Pigment metrics to planning measures.
- pipeline-012: Map Pigment scenario branches to scenario recalculation candidates.
- pipeline-013: Map Pigment formulas to formula projection refs.
- pipeline-014: Evaluate Cedar and pack overlays.
- pipeline-015: Reject unmapped principals and external sharing links.
- pipeline-016: Reject formulas with nondeterministic parse output.
- pipeline-017: Reject scenario graphs with cross-tenant references.
- pipeline-018: Accept clean batches into worker queue.
- pipeline-019: Emit audit-chain evidence.
- pipeline-020: Expose read projections only after evidence is signed.

## Failure Modes
- failure-001: Workbook has missing tenant binding.
- failure-002: Workbook range lacks data class.
- failure-003: Workbook formula cannot parse.
- failure-004: Workbook approval lacks mapped principal.
- failure-005: Workbook export lacks signer.
- failure-006: Workbook sharing link lacks allowed purpose.
- failure-007: Vena task claims approval but Cedar denies principal.
- failure-008: Pigment app has missing tenant binding.
- failure-009: Pigment formula cannot parse.
- failure-010: Pigment dimension maps outside tenant ontology.
- failure-011: Pigment scenario references another tenant's branch.
- failure-012: Pigment workflow skips Oyatie workflow step.
- failure-013: Source comment attempts approval semantics.
- failure-014: Connector job lacks rollback bundle.
- failure-015: Source export violates pack residency.
- failure-016: Audit-chain unavailable for board seal.
- failure-017: Scenario recalculation exceeds cost budget.
- failure-018: Duplicate idempotency key has different source hash.
- failure-019: Replay manifest omits formula version.
- failure-020: Board packet references unapproved scenario state.

## Tests and Evidence
- evidence-001: Contract test accepts `source_vendor=vena`.
- evidence-002: Contract test accepts `source_vendor=pigment`.
- evidence-003: Contract test requires workbook id for Vena imports.
- evidence-004: Contract test requires application id for Pigment imports.
- evidence-005: Property test rejects nondeterministic workbook formulas.
- evidence-006: Property test rejects nondeterministic Pigment formulas.
- evidence-007: Property test rejects cross-tenant scenario graphs.
- evidence-008: Property test preserves formula version lineage.
- evidence-009: Replay test imports Vena workbook sections into board packet candidates.
- evidence-010: Replay test imports Vena approval evidence without bypassing Cedar.
- evidence-011: Replay test imports Pigment scenario branches.
- evidence-012: Replay test rejects Pigment dimensions outside tenant ontology.
- evidence-013: Cedar test blocks comment-as-approval.
- evidence-014: Residency test blocks unsafe export.
- evidence-015: Audit test pauses board seal during audit-chain outage.
- evidence-016: Cost test emits connector and scenario dimensions.
- evidence-017: Board test requires signer and disclosure set.
- evidence-018: Rollback test compensates accepted workbook and scenario imports.
- evidence-019: Promotion test prevents Vena or Pigment service boundary creation.
- evidence-020: Documentation test counts Source and ADR anchors.

## Acceptance Criteria
- AC-001: Vena workbooks become board packet candidates.
- AC-002: Pigment scenarios become scenario recalculation candidates.
- AC-003: Source formulas require deterministic parse evidence.
- AC-004: Source collaboration history is evidence only.
- AC-005: Cedar owns authorization.
- AC-006: Board-report-seal owns signed exports.
- AC-007: Scenario-recalculate owns branch promotion.
- AC-008: Rollback bundles exist for accepted imports.
- AC-009: The file has at least 200 lines of Vena and Pigment-specific substance.
- AC-010: Citation density is measurable through Source, ADR, PRD, ARCHITECTURE, and competitor-parity references.

## Rollback
- rollback-001: Restore prior board packet candidate state.
- rollback-002: Restore prior scenario branch projection.
- rollback-003: Preserve workbook and app source hashes.
- rollback-004: Preserve collaboration evidence.
- rollback-005: Preserve formula refusal rows.
- rollback-006: Emit audit-chain rollback evidence.
- rollback-007: Keep exports unsigned until resealed.
- rollback-008: Re-run Cedar before replay.
- rollback-009: Re-check pack residency before export.
- rollback-010: Do not delete source comments or history.

## Deepening Appendix
- appendix-001: Vena workbook ids remain source refs.
- appendix-002: Vena template ids remain workflow template candidates.
- appendix-003: Vena approval records remain reviewer evidence.
- appendix-004: Vena comments remain collaboration evidence.
- appendix-005: Vena exports remain unsigned until board-report-seal succeeds.
- appendix-006: Pigment application ids remain source refs.
- appendix-007: Pigment block ids remain scenario projection refs.
- appendix-008: Pigment metric ids remain measure refs.
- appendix-009: Pigment scenario ids remain scenario branch refs.
- appendix-010: Pigment formulas require deterministic parse evidence.
- appendix-011: Pigment workflows remain workflow template candidates.
- appendix-012: Pigment comments remain collaboration evidence.
- appendix-013: Source collaboration state cannot bypass Cedar.
- appendix-014: Source sharing links require allowed purpose and expiry.
- appendix-015: Source: microservices/financial-planning/PRD.md UX Flows.
- appendix-016: Source: microservices/financial-planning/ARCHITECTURE.md scenario context.
- appendix-017: Source: microservices/financial-planning/competitor-parity-matrix.md Observability and audit events.
- appendix-018: Source: ADR-0105 adapter boundary.
- appendix-019: Source: ADR-0253-amendment transport baseline.
- appendix-020: Acceptance evidence requires no Vena service boundary.
- appendix-021: Acceptance evidence requires no Pigment service boundary.
- appendix-022: Acceptance evidence requires formula lineage.
- appendix-023: Acceptance evidence requires replay manifest.
- appendix-024: Acceptance evidence requires rollback bundle.
- appendix-025: Acceptance evidence requires tenant-scoped scenario projection.

## Required Section Addendum

## Context
- Persona: Aisha Grant, CFO office reporting lead, migrates Vena workbook collaboration and Pigment scenario branches into Oyatie board-report and scenario state.
- Vendor surface subsumed: Vena workbook/template/tab/locked range/comment/approval and Pigment block/metric/list/scenario/branch/simulation.

## Data Model Deltas
```sql
create table fp_board_scenario_imports (
    board_scenario_import_id uuid primary key,
    tenant_id uuid not null,
    source_vendor text not null check (source_vendor in ('vena','pigment')),
    workbook_or_block_ref text not null,
    scenario_ref text,
    board_packet_id uuid,
    signer_principal_id uuid,
    source_hash text not null,
    audit_event_class text not null
);
```
```rust
pub struct BoardScenarioImport { pub board_scenario_import_id: Uuid, pub tenant_id: Uuid, pub source_vendor: BoardScenarioVendor, pub workbook_or_block_ref: String, pub scenario_ref: Option<String>, pub board_packet_id: Option<Uuid>, pub signer_principal_id: Option<Uuid>, pub source_hash: String, pub audit_event_class: AuditEventClass }
```

## API Endpoints
```http
POST /v1/financial-planning/migrations/board-scenario/imports
{"tenant_id":"t_finance","source_vendor":"pigment","workbook_or_block_ref":"block_margin_bridge","scenario_ref":"downside_case","dry_run":true}
```
```yaml
grpc: {service: oyatie.financial_planning.BoardScenarioMigrationService, rpc: ImportBoardScenarioSurface}
asyncapi: {publish: financial-planning.board-scenario.imported.v1, payload: {board_scenario_import_id: uuid, source_vendor: string, audit_event_class: string}}
```

## Cedar Policy Hooks
```cedar
permit(principal, action == FinanceAction::"board-scenario-import", resource)
when { context.tenant_id == resource.tenant_id && context.source_hash != "" && context.audit_chain_status == "available" };
forbid(principal, action, resource)
when { action == FinanceAction::"board-report-seal" && context.signer_principal_id == "" };
```

## Ontology Projection
| Vendor object | Oyatie object | Field delta |
|---|---|---|
| Vena workbook | `BoardReportPacketCandidate` | workbook id becomes packet source ref |
| Vena locked range | `BoardEvidenceRange` | range id becomes evidence ref |
| Pigment block | `ScenarioAssumptionGraph` | block name becomes assumption group |
| Pigment branch | `ScenarioVersion` | branch id becomes scenario version ref |

## Workflow Steps
- Node `source-collaboration-read`: collect workbook, comment, branch, and simulation metadata.
- Branch `unsigned-board-packet`: block board seal until signer principal exists.
- Node `scenario-graph-project`: convert Pigment blocks into assumption graph nodes.
- Branch `cross-tenant-branch-ref`: deny and emit scenario quarantine event.
- Node `board-packet-build`: create packet candidate with evidence ranges and reviewer chain.

## Audit Events
- `FinancialPlanningBoardScenarioImportStarted`
- `FinancialPlanningVenaWorkbookProjected`
- `FinancialPlanningPigmentScenarioProjected`
- `FinancialPlanningBoardPacketSealDenied`
- `FinancialPlanningBoardScenarioActivated`

## SLO Targets
| Path | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| workbook projection | 90 ms | 550 ms | 1.1 s | 20k cells/min | 99.9% |
| scenario branch projection | 70 ms | 420 ms | 900 ms | 800 branches/min | 99.95% |

## Failure Modes + Recovery
- `unsigned-board-packet`: block seal and route signer task.
- `scenario-cross-tenant-ref`: quarantine branch and emit policy denial.
- `workbook-range-hash-drift`: reject packet candidate and require new dry-run.
- `comment-author-unmapped`: import comment as evidence only until principal maps.

## Migration Notes
- Vena workbooks become board packet candidates.
- Vena comments become review evidence.
- Pigment blocks become scenario assumption nodes.
- Pigment simulations become recalculation requests with replay evidence.

## Cross-Microservice Handoffs
- workflow-engine owns reviewer and signer tasks.
- ontology receives scenario graph deltas.
- policy-engine evaluates board seal and scenario import.
- audit-chain seals ADR-0263 evidence.
- analytics receives activated scenario projections.

## Wave 15 counterpart anchor
- Counterpart baseline: Anaplan, Workday Adaptive Planning, Oracle EPM, OneStream, Vena, Pigment, Planful, Snowflake. This preserved IP already exceeded the stamp-shell line signature; the added anchor makes the counterpart comparison explicit for Wave 15 verification.
