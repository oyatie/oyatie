---
doc_class: IP
ip_id: IP-028-oracle-epm-onestream-close-displacement
microservice: financial-planning
related_adrs: [ADR-0002, ADR-0003, ADR-0007, ADR-0008, ADR-0009, ADR-0105, ADR-0131, ADR-0173, ADR-0199, ADR-0243, ADR-0253, ADR-0263, ADR-0294, ADR-0314, ADR-0321]
journey_ref: J125-close-day-state-machine
tenant_class: tier-1
status: draft
date: 2026-05-20
owner_team: axis-finance-planning + axis-close-consolidation
---

# IP-028 Financial Planning oracle-epm-onestream-close-displacement

Service: financial-planning
ChangeSet scope: microservices/financial-planning/IP-028-oracle-epm-onestream-close-displacement.md
Batch: B2B-leader IP substance deepening batch B
Status: net-new
Line target: at least 200 lines
Primary displacement surface: Oracle EPM Cloud and OneStream close, consolidation, account hierarchy, intercompany, currency, and board-close evidence
Binding authorities: PRD-financial-planning, ARCHITECTURE-financial-planning, competitor-parity-matrix, ADR-0105, ADR-0131, ADR-0253-amendment, ADR-0314, ADR-0321

## Objective
- Replace Oracle EPM Cloud and OneStream close dependence with Oyatie consolidation-close evidence.
- Treat account hierarchies, entity hierarchies, intercompany eliminations, journals, and currency tables as source projections.
- Prevent close adapters from bypassing tenant scope, Cedar, residency, audit-chain, and rollback.
- Bind close completion to board-report-seal prerequisites.
- Preserve source close provenance without accepting vendor workflow state as final authority.
- Source: microservices/financial-planning/PRD.md consolidation and board-report requirements.
- Source: microservices/financial-planning/ARCHITECTURE.md consolidation bounded context.
- Source: microservices/financial-planning/competitor-parity-matrix.md OneStream and benchmark close parity.
- Source: ADR-0105 layer rules.
- Source: ADR-0321 documentation-rigor anchors.

## Source Object Taxonomy
- taxonomy-001: `oracle_epm_application` maps to `planning_source_container`.
- taxonomy-002: `onestream_application` maps to `planning_source_container`.
- taxonomy-003: `cube` maps to `consolidation_cube_projection`.
- taxonomy-004: `account_hierarchy` maps to `planning_account_dimension`.
- taxonomy-005: `entity_hierarchy` maps to `legal_entity_dimension`.
- taxonomy-006: `intercompany_partner` maps to `intercompany_dimension`.
- taxonomy-007: `scenario` maps to `scenario_branch`.
- taxonomy-008: `period` maps to `close_period`.
- taxonomy-009: `movement` maps to `cashflow_movement_dimension`.
- taxonomy-010: `currency_rate` maps to `fx_rate_projection_ref`.
- taxonomy-011: `journal` maps to `close_adjustment_source`.
- taxonomy-012: `elimination_rule` maps to `elimination_projection_ref`.
- taxonomy-013: `ownership_rule` maps to `ownership_projection_ref`.
- taxonomy-014: `translation_rule` maps to `translation_projection_ref`.
- taxonomy-015: `data_management_job` maps to `close_import_batch`.
- taxonomy-016: `workflow_certification` maps to `close_approval_evidence`.
- taxonomy-017: `task_manager_task` maps to `workflow_step_ref`.
- taxonomy-018: `cube_view` maps to `board_report_packet_candidate`.
- taxonomy-019: `dashboard_book` maps to `signed_export_candidate`.
- taxonomy-020: `audit_history` maps to `audit_chain_supporting_evidence`.

## Close Controls
- close-control-001: Close imports require tenant id, source vendor, source application id, and close period.
- close-control-002: Account hierarchies require acyclic parent-child validation.
- close-control-003: Entity hierarchies require jurisdiction and ownership validation.
- close-control-004: Intercompany partner mappings require tenant-local entity refs.
- close-control-005: Currency rates require effective-date and pack validation.
- close-control-006: Journals require preparer, approver, reason, and source hash.
- close-control-007: Elimination rules require deterministic projection output.
- close-control-008: Ownership rules require no negative or impossible ownership percentages.
- close-control-009: Translation rules require source currency and target currency evidence.
- close-control-010: Data management jobs require dry-run refusal rows.
- close-control-011: Workflow certifications require Oyatie workflow step mapping.
- close-control-012: Task manager tasks cannot mark Oyatie close complete alone.
- close-control-013: Cube views cannot become board packets without signer lineage.
- close-control-014: Dashboard books become signed artifacts only after board-report-seal.
- close-control-015: Source audit history supports but never replaces audit-chain.
- close-control-016: Cross-region close data follows pack residency.
- close-control-017: Close mismatch opens a runbook path before board seal.
- close-control-018: Close reopen requires principal, reason, and rollback bundle.
- close-control-019: Close archive emits audit evidence.
- close-control-020: Close replay uses stored hashes, not live source reads.

## Displacement Requirements
- requirement-001: Oyatie owns the consolidation-close aggregate.
- requirement-002: Oracle EPM Cloud applications are source containers.
- requirement-003: OneStream applications are source containers.
- requirement-004: Oyatie owns account hierarchy projection.
- requirement-005: Oyatie owns entity hierarchy projection.
- requirement-006: Oyatie owns intercompany validation.
- requirement-007: Oyatie owns journal acceptance.
- requirement-008: Oyatie owns elimination projection.
- requirement-009: Oyatie owns ownership projection.
- requirement-010: Oyatie owns currency translation projection.
- requirement-011: Oyatie owns close workflow completion.
- requirement-012: Vendor workflow certifications are supporting evidence.
- requirement-013: Oyatie owns board-report seal.
- requirement-014: Cube views and dashboard books are packet candidates.
- requirement-015: Oyatie owns tenant authorization.
- requirement-016: Vendor task privileges cannot bypass Cedar.
- requirement-017: Oyatie owns rollback and replay.
- requirement-018: Every accepted close import has compensation refs.
- requirement-019: Oyatie owns cost and capacity attribution.
- requirement-020: Every close job emits tenant, vendor, row, CPU, memory, and storage dimensions.

## Pipeline
- pipeline-001: Discover application metadata.
- pipeline-002: Discover cube roster.
- pipeline-003: Discover account hierarchy.
- pipeline-004: Discover entity hierarchy.
- pipeline-005: Discover intercompany partners.
- pipeline-006: Discover scenario and period mapping.
- pipeline-007: Discover currency and translation tables.
- pipeline-008: Discover journals and close adjustments.
- pipeline-009: Discover elimination and ownership rules.
- pipeline-010: Discover workflow certifications and tasks.
- pipeline-011: Discover cube views and board books.
- pipeline-012: Build dry-run transform plan.
- pipeline-013: Run hierarchy cycle validation.
- pipeline-014: Run ownership percentage validation.
- pipeline-015: Run intercompany tenant validation.
- pipeline-016: Run FX effective-date validation.
- pipeline-017: Run Cedar and pack checks.
- pipeline-018: Accept clean close batches into worker queue.
- pipeline-019: Emit mismatch and refusal evidence.
- pipeline-020: Seal read projection only after audit-chain event.

## Failure Modes
- failure-001: Account hierarchy contains cycle.
- failure-002: Entity hierarchy crosses tenant boundary.
- failure-003: Intercompany partner has no mapped legal entity.
- failure-004: Journal lacks approver.
- failure-005: Journal has stale source revision.
- failure-006: Elimination rule cannot parse deterministically.
- failure-007: Ownership rule exceeds 100 percent.
- failure-008: Translation rule lacks currency evidence.
- failure-009: Currency table has overlapping effective dates.
- failure-010: Close job lacks rollback bundle.
- failure-011: Vendor workflow claims complete while Oyatie tasks remain open.
- failure-012: Cube view attempts unsigned board export.
- failure-013: Source region violates pack residency.
- failure-014: Source task privilege conflicts with Cedar deny.
- failure-015: Data management job duplicates idempotency key with different hash.
- failure-016: Audit-chain unavailable for close-complete event.
- failure-017: Cost budget exceeded during close import.
- failure-018: Close mismatch cannot identify source row.
- failure-019: Board-report seal references unapproved close period.
- failure-020: Replay manifest missing accepted transform ids.

## Tests and Evidence
- evidence-001: Contract test accepts `source_vendor=oracle_epm_cloud`.
- evidence-002: Contract test accepts `source_vendor=onestream`.
- evidence-003: Contract test requires close period.
- evidence-004: Contract test requires application id.
- evidence-005: Property test rejects account hierarchy cycles.
- evidence-006: Property test rejects entity cross-tenant references.
- evidence-007: Property test rejects impossible ownership percentages.
- evidence-008: Property test rejects overlapping FX rates.
- evidence-009: Replay test imports Oracle EPM account hierarchy.
- evidence-010: Replay test imports OneStream consolidation cells.
- evidence-011: Replay test rejects missing journal approver.
- evidence-012: Replay test emits close mismatch evidence.
- evidence-013: Cedar test blocks vendor task privilege bypass.
- evidence-014: Residency test blocks cross-pack close export.
- evidence-015: Audit test pauses close-complete during audit-chain outage.
- evidence-016: Cost test emits close job dimensions.
- evidence-017: Board test blocks unsealed cube view export.
- evidence-018: Rollback test compensates accepted close import.
- evidence-019: Promotion test prevents Oracle or OneStream service boundary creation.
- evidence-020: Documentation test counts Source and ADR anchors.

## Acceptance Criteria
- AC-001: Oracle EPM Cloud and OneStream are source vendors only.
- AC-002: Consolidation-close remains Oyatie-owned.
- AC-003: Account, entity, intercompany, currency, journal, and elimination artifacts are projected with evidence.
- AC-004: Vendor workflow status cannot complete Oyatie close alone.
- AC-005: Board-report-seal depends on approved close evidence.
- AC-006: Cedar and pack residency gate every close mutation.
- AC-007: Audit-chain evidence is emitted before read projection.
- AC-008: Rollback bundles exist for accepted close imports.
- AC-009: The file has at least 200 lines of Oracle EPM Cloud and OneStream-specific substance.
- AC-010: Citation density is measurable through Source, ADR, PRD, ARCHITECTURE, and competitor-parity references.

## Rollback
- rollback-001: Restore prior close period state.
- rollback-002: Compensate accepted journal projections.
- rollback-003: Revert hierarchy projection pointer.
- rollback-004: Preserve source hashes.
- rollback-005: Preserve refusal rows.
- rollback-006: Emit audit-chain rollback evidence.
- rollback-007: Keep board packet unsealed until close is resealed.
- rollback-008: Re-run Cedar before replay.
- rollback-009: Re-check residency before export.
- rollback-010: Do not delete vendor source history.

## Deepening Appendix
- appendix-001: Oracle EPM Cloud application ids remain source refs.
- appendix-002: OneStream application ids remain source refs.
- appendix-003: Cubes remain consolidation projection sources.
- appendix-004: Account hierarchies require cycle checks.
- appendix-005: Entity hierarchies require jurisdiction checks.
- appendix-006: Intercompany partners require tenant-local entity mapping.
- appendix-007: Journals require preparer and approver evidence.
- appendix-008: Eliminations require deterministic projection evidence.
- appendix-009: Ownership rules require percentage validation.
- appendix-010: Currency translation requires FX policy evidence.
- appendix-011: Close certifications remain workflow evidence.
- appendix-012: Task manager state cannot complete Oyatie close alone.
- appendix-013: Cube views remain board packet candidates.
- appendix-014: Dashboard books remain signed export candidates.
- appendix-015: Source: microservices/financial-planning/PRD.md User Stories for consolidation.
- appendix-016: Source: microservices/financial-planning/ARCHITECTURE.md consolidation context.
- appendix-017: Source: microservices/financial-planning/competitor-parity-matrix.md Regional packs and residency.
- appendix-018: Source: ADR-0105 adapter boundary.
- appendix-019: Source: ADR-0321 documentation-rigor anchors.
- appendix-020: Acceptance evidence requires no Oracle EPM Cloud service boundary.
- appendix-021: Acceptance evidence requires no OneStream service boundary.
- appendix-022: Acceptance evidence requires close mismatch evidence.
- appendix-023: Acceptance evidence requires replay manifest.
- appendix-024: Acceptance evidence requires rollback bundle.
- appendix-025: Acceptance evidence requires tenant-scoped close projection.

## Required Section Addendum

## Context
- Persona: Victor Alvarez, corporate controller, migrates Oracle EPM Cloud cubes and OneStream workflow profiles into Oyatie consolidation-close.
- Vendor surface subsumed: Oracle EPM cube/dimension/member/data map/journal and OneStream workflow profile/entity/scenario/certification.

## Data Model Deltas
```sql
create table fp_close_displacement_imports (
    close_import_id uuid primary key,
    tenant_id uuid not null,
    source_vendor text not null check (source_vendor in ('oracle_epm_cloud','onestream')),
    close_period text not null,
    entity_ref text not null,
    scenario_ref text not null,
    source_cube_ref text not null,
    consolidation_rule_hash text not null,
    audit_event_class text not null
);
```
```rust
pub struct CloseDisplacementImport { pub close_import_id: Uuid, pub tenant_id: Uuid, pub source_vendor: CloseVendor, pub close_period: String, pub entity_ref: String, pub scenario_ref: String, pub source_cube_ref: String, pub consolidation_rule_hash: String, pub audit_event_class: AuditEventClass }
```

## API Endpoints
```http
POST /v1/financial-planning/migrations/close-suites/imports
{"tenant_id":"t_finance","source_vendor":"onestream","close_period":"2026-Q2","entity_ref":"legal-us-001","scenario_ref":"actual","source_cube_ref":"finance_cube"}
```
```yaml
grpc: {service: oyatie.financial_planning.CloseSuiteMigrationService, rpc: ImportCloseSurface}
asyncapi: {publish: financial-planning.close-suite.imported.v1, payload: {close_import_id: uuid, source_vendor: string, audit_event_class: string}}
```

## Cedar Policy Hooks
```cedar
permit(principal, action == FinanceAction::"close-suite-import", resource)
when { context.tenant_id == resource.tenant_id && context.close_period != "" && context.audit_chain_status == "available" };
forbid(principal, action, resource)
when { context.vendor_close_admin == true && context.oyatie_controller_binding == "" };
```

## Ontology Projection
| Vendor object | Oyatie object | Field delta |
|---|---|---|
| Oracle EPM cube | `ConsolidationCubeProjection` | cube name becomes source alias |
| Oracle dimension member | `CloseEntityMember` | member alias becomes display metadata |
| OneStream workflow profile | `CloseWorkflowNode` | profile status becomes workflow evidence |
| OneStream certification | `CloseCertificationEvidence` | certifier becomes Oyatie signer ref |

## Workflow Steps
- Node `close-source-discover`: gather cube, entity, scenario, journal, profile, and certification metadata.
- Branch `vendor-admin-authority`: deny and require controller principal binding.
- Node `rule-hash-validate`: verify consolidation rule hashes and currency tables.
- Branch `entity-hierarchy-cycle`: quarantine entity tree and block close activation.
- Node `close-activate`: create consolidation-close candidate and mismatch evidence.

## Audit Events
- `FinancialPlanningCloseSuiteImportStarted`
- `FinancialPlanningCloseRuleHashValidated`
- `FinancialPlanningCloseVendorAdminDenied`
- `FinancialPlanningCloseEntityHierarchyBlocked`
- `FinancialPlanningCloseSuiteActivated`

## SLO Targets
| Path | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| close import dry-run | 150 ms | 850 ms | 1.7 s | 30k cells/min | 99.9% |
| rule validation | 60 ms | 320 ms | 700 ms | 1,000 rules/min | 99.95% |

## Failure Modes + Recovery
- `entity-hierarchy-cycle`: reject import and emit hierarchy repair packet.
- `currency-table-mismatch`: block close activation and require FX policy selection.
- `vendor-admin-overreach`: deny direct authority and request controller binding.
- `rule-hash-drift`: quarantine rule set and rerun dry-run from source snapshot.

## Migration Notes
- Oracle EPM Cloud dimensions and data maps become close projections.
- OneStream workflow profiles become workflow nodes with signer evidence.
- Vendor journals become close adjustment candidates.
- Vendor certifications become audit evidence, not close authority.

## Cross-Microservice Handoffs
- workflow-engine owns close state machine nodes.
- ontology receives entity and dimension projections.
- policy-engine evaluates close import and certification actions.
- audit-chain seals ADR-0263 close events.
- board-reporting receives only activated close candidates.

## Wave 15 counterpart anchor
- Counterpart baseline: Anaplan, Workday Adaptive Planning, Oracle EPM, OneStream, Vena, Pigment, Planful, Snowflake. This preserved IP already exceeded the stamp-shell line signature; the added anchor makes the counterpart comparison explicit for Wave 15 verification.
