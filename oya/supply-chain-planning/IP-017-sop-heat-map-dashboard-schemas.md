---
doc_class: ImplementationPlan
ip_id: IP-017
microservice: supply-chain-planning
related_adrs: [ADR-0105, ADR-0243, ADR-0244, ADR-0253, ADR-0263, ADR-0315, ADR-0131, ADR-0132]
journey_id: j123-multi-tenant-coordinated-product-launch
journey_link: docs/user-journeys/j123-multi-tenant-coordinated-product-launch/story.md
status: Accepted
date: 2026-05-20
owner: axis-supply-chain-planning + axis-erp-parity
tenant_class: paid
billing_components:
  - per_usage
sap_submodule_equivalents: [IBP-SOP sales and operations planning, IBP-SOP planning view, IBP control tower alert tiles]
---

# IP-017: S and OP heat map dashboard schemas

## 1. Context with why, journey leg, named persona

This IP defines the schema and API surface for S and OP heat maps that show
where demand, supply, margin, capacity, and service risk diverge.

Why this matters: executives and planners need one dashboard substrate that can
trace a red cell back to the exact planning scenario, input measure, policy
decision, and audit evidence.

Journey leg: j123 leg 04, "surface launch readiness exceptions and drill into scenario risk".

Named persona: Daniel, S and OP meeting owner for North America appliances.

Daniel runs a weekly executive review and needs heat maps that reconcile to
approved scenario data, not screenshots or spreadsheet copies.

SAP equivalent: IBP-SOP planning view with alert key figures and control tower
tiles.

Oracle equivalent: Sales and Operations Planning exception dashboards.

Microsoft equivalent: Supply risk and planning optimization workspaces.

The implementation is schema-first because dashboard cells must remain stable
across UI, API, export, audit, and ontology projections.

The heat map never computes its own plan; it reads governed measures from
demand-plan, supply-network-plan, ATP, CTP, and allocation outputs.

The feature belongs in the rest, application, and governance bands from
ADR-0105.

## 2. Scope

In scope: dashboard definition schema.

In scope: heat map cell materialization table.

In scope: drill path metadata.

In scope: REST and gRPC dashboard query APIs.

In scope: Cedar policy for view, export, and annotate.

In scope: ontology projection to PlanningRiskHeatMap.

Out of scope: chart rendering implementation.

Out of scope: changing S and OP consensus plan calculations.

Out of scope: BI vendor embedding.

Out of scope: financial close approval workflows.

## 3. Data Model Deltas

Create table scp_sop_heat_map_definition.

Column tenant_id: uuid, required, partition key.

Column heat_map_id: uuid, required.

Column display_name: text, required.

Column planning_scenario_id: uuid, required.

Column horizon_start: date, required.

Column horizon_end: date, required.

Column row_dimension: enum, values product_family, dc, region, customer_segment.

Column column_dimension: enum, values week, month, scenario, measure.

Column measure_set_id: text, required.

Column color_scale_id: text, required.

Column owner_principal_id: uuid, required.

Column status: enum, values draft, active, archived.

Column created_at: timestamptz, required.

Create table scp_sop_heat_map_measure_binding.

Column tenant_id: uuid, required.

Column heat_map_id: uuid, required.

Column measure_binding_id: uuid, required.

Column measure_name: text, required.

Column source_context: enum, values demand_plan, supply_network_plan, atp,
ctp, allocation, finance_margin.

Column source_measure_ref: text, required.

Column aggregation_method: enum, values sum, weighted_average, min, max,
ratio.

Column warning_threshold: numeric(18,6), required.

Column critical_threshold: numeric(18,6), required.

Column direction: enum, values higher_is_risk, lower_is_risk, variance_is_risk.

Create table scp_sop_heat_map_cell.

Column tenant_id: uuid, required, partition key.

Column heat_map_id: uuid, required.

Column cell_id: uuid, required.

Column row_key: text, required.

Column row_label: text, required.

Column column_key: text, required.

Column column_label: text, required.

Column measure_name: text, required.

Column measure_value: numeric(18,6), required.

Column risk_band: enum, values green, amber, red, black.

Column contributing_source_json: jsonb, required.

Column drill_path_json: jsonb, required.

Column refreshed_at: timestamptz, required.

Column audit_id: uuid, required.

Create table scp_sop_heat_map_annotation.

Column tenant_id: uuid, required.

Column annotation_id: uuid, required.

Column cell_id: uuid, required.

Column principal_id: uuid, required.

Column annotation_text: text, required.

Column disposition: enum, values watching, action_opened, accepted_risk,
resolved.

Column audit_event_class: text, required.

## 4. API Endpoints

REST POST /v1/supply-chain-planning/sop/heat-maps

REST GET /v1/supply-chain-planning/sop/heat-maps/{heat_map_id}

REST GET /v1/supply-chain-planning/sop/heat-maps/{heat_map_id}/cells

REST POST /v1/supply-chain-planning/sop/heat-map-cells/{cell_id}/annotations

gRPC SopHeatMapService.CreateHeatMap accepts CreateHeatMapRequest.

gRPC SopHeatMapService.QueryCells accepts QueryHeatMapCellsRequest.

gRPC SopHeatMapService.AnnotateCell accepts AnnotateHeatMapCellRequest.

Example create request:

```json
{
  "tenant_id": "22222222-2222-2222-2222-222222222222",
  "display_name": "NA weekly S and OP risk",
  "planning_scenario_id": "scenario-na-w21-consensus",
  "row_dimension": "product_family",
  "column_dimension": "week",
  "measure_set_id": "service-capacity-margin",
  "color_scale_id": "scp-standard-risk"
}
```

Example cell response:

```json
{
  "heat_map_id": "hm-017-na-w21",
  "cell_id": "cell-appliances-w22",
  "row_label": "Laundry",
  "column_label": "2026-W22",
  "risk_band": "red",
  "measure_value": 0.812,
  "drill_path": ["supply_network_plan", "atp", "allocation"]
}
```

Query filters include risk_band, row_key, column_key, measure_name, and updated
since timestamp.

Export uses REST GET /v1/supply-chain-planning/sop/heat-maps/{id}/export.csv.

Export responses include audit_id and policy_decision_id in headers.

## 5. Cedar Policy Hooks

Principal type: SopPlanner, ExecutiveViewer, ComplianceAuditor.

Action: scp::Action::"ViewSopHeatMap".

Action: scp::Action::"ExportSopHeatMap".

Action: scp::Action::"AnnotateSopHeatMapCell".

Resource: scp::SopHeatMap::"<tenant_id>/<heat_map_id>".

Context tenant_id must equal principal.tenant_id.

Context tenant_class must be compatible with the principal tenant_class grant; paid-tenant actions also require billing_components to be a subset of the principal billing-component grant.

Context export_format csv requires principal.data_export_allowed.

Context measure_set_id finance_margin requires principal.finance_view_allowed.

Context annotation_text is rejected when classification exceeds resource class.

Policy denial emits SopHeatMapPolicyDenied.

Allowed export emits SopHeatMapExported.

Allowed annotation emits SopHeatMapCellAnnotated.

## 6. Ontology Projection Field Mapping

scp_sop_heat_map_definition.heat_map_id maps to PlanningRiskHeatMap.id.

display_name maps to PlanningRiskHeatMap.name.

planning_scenario_id maps to PlanningScenario.id.

horizon_start and horizon_end map to PlanningHorizon.

row_dimension maps to PlanningRiskHeatMap.row_axis.

column_dimension maps to PlanningRiskHeatMap.column_axis.

measure_set_id maps to RiskMeasureSet.id.

cell_id maps to PlanningRiskHeatMapCell.id.

risk_band maps to PlanningRiskHeatMapCell.risk_band.

contributing_source_json maps to RiskEvidence.sources.

drill_path_json maps to WorkflowDrillPath.nodes.

annotation_id maps to PlanningDecisionAnnotation.id.

audit_id maps to AuditEvidence.id.

## 7. Workflow Steps

Node HeatMapDefinitionValidate checks dimensions and measure bindings.

Node ScenarioSnapshotAuthorize checks scenario view rights.

Node MeasureBindingResolve maps source measures to canonical measures.

Node CellAggregationRun materializes cell values.

Node RiskBandClassify applies thresholds and direction.

Node DrillPathAttach links cells to source contexts.

Node DashboardCachePublish writes cache-ready cell pages.

Node AnnotationAccept validates annotation text and disposition.

Node AuditSeal emits ADR-0263 events.

Node OntologyProject writes heat map and cell nodes.

Branch MissingMeasureBinding fails definition activation.

Branch SourceContextUnavailable marks cells black with degraded reason.

Branch FinanceMeasureRestricted hides value and keeps redacted drill path.

Branch ExportDenied returns policy denial without writing export.

Branch AnnotationOnArchivedMap rejects with archived state.

## 8. Audit Events

SopHeatMapDefinitionCreated records dimensions and measure set.

SopHeatMapDefinitionActivated records scenario and policy bundle.

SopHeatMapCellsMaterialized records cell count and source contexts.

SopHeatMapRiskBandChanged records previous and next band.

SopHeatMapViewed records principal, filter, and row count.

SopHeatMapExported records export format and row count.

SopHeatMapCellAnnotated records annotation disposition.

SopHeatMapPolicyDenied records Cedar action and decision id.

SopHeatMapOntologyProjected records projection id.

Event envelopes use EVT-SUPPLY_CHAIN_PLANNING-SOP_HEAT_MAP prefixes.

Every event carries tenant_id, principal_id, trace_id, audit_id,
policy_bundle_version, scenario_id, and residency_pack.

## 9. SLO Targets

p50 cell query latency: 120 ms for cached 500-cell page.

p95 cell query latency: 650 ms for filtered 5000-cell result.

p99 cell query latency: 1400 ms for uncached measure drill query.

Throughput target: 500 cell-query requests per minute per tenant.

Materialization throughput target: 1 million cells per 12 minutes per tenant.

Availability target: 99.95 percent monthly for dashboard read APIs.

Rationale: S and OP meetings require interactive read latency, while background
materialization can run before meeting start.

## 10. Failure Modes + Recovery

Failure mode: source measure binding points to retired measure.

Recovery: block activation and show binding repair message.

Failure mode: scenario snapshot is deleted.

Recovery: archive heat map definition and retain old audit evidence.

Failure mode: materialization job times out.

Recovery: preserve last successful cells and mark refresh degraded.

Failure mode: finance margin policy changes mid-session.

Recovery: re-evaluate Cedar on every export and drill.

Failure mode: annotation write succeeds but audit emission fails.

Recovery: keep annotation pending-visible only to author and retry audit seal.

Failure mode: ontology projection fails.

Recovery: keep dashboard available and queue projection retry.

## 11. Migration Notes with source vendor surfaces

SAP IBP-SOP source: planning view dimensions and alert key figures.

SAP IBP-SOP source: scenario versions and consensus demand measures.

SAP control tower source: alert tile thresholds and drill links.

Oracle S and OP source: exception dashboard measure catalog.

Kinaxis source: workbook cells and scenario exception tables.

Migration maps vendor workbook to heat_map_definition.

Migration maps vendor alert key figure to heat_map_measure_binding.

Migration maps vendor red/yellow/green scale to color_scale_id.

Migration preserves source dashboard id in source_measure_ref.

Migration stores export checksums in audit evidence.

## 12. Cross-Microservice Handoffs

Handoff from demand-plan provides demand variance measures.

Handoff from ATP provides service-risk measures.

Handoff from CTP provides capacity feasibility measures.

Handoff from allocation provides shortage fairness measures.

Handoff from finance provides margin risk only with policy approval.

Handoff to workflow-engine opens action items from annotations.

Handoff to notification sends meeting-ready dashboard alerts.

Handoff to audit-chain emits view, export, and annotation events.

Handoff to ontology publishes PlanningRiskHeatMap graph nodes.

## 13. Intern Build Notes

Build step 01: create definition, binding, cell, and annotation migrations.

Build step 02: add unique keys on tenant_id plus heat_map_id.

Build step 03: add cell index on tenant_id, heat_map_id, risk_band.

Build step 04: implement measure binding validation.

Build step 05: implement source context adapters as read-only ports.

Build step 06: implement risk band classification as deterministic function.

Build step 07: implement REST create and query endpoints.

Build step 08: implement gRPC QueryCells pagination.

Build step 09: implement annotation endpoint with audit pending state.

Build step 10: implement CSV export with policy headers.

Build step 11: implement Cedar fixture for executive view.

Build step 12: implement Cedar fixture for finance redaction.

Build step 13: implement Cedar fixture for export denial.

Build step 14: write contract tests for create heat map.

Build step 15: write contract tests for query filters.

Build step 16: write contract tests for annotation disposition.

Build step 17: write audit fixtures for exported and annotated events.

Build step 18: write materialization test for retired measure binding.

Build step 19: write two-tenant isolation test.

Build step 20: add dashboard cache invalidation on scenario refresh.

Build step 21: add degraded marker for stale materialization.

Build step 22: add ontology projection retry queue.

Build step 23: add migration evidence for SAP planning view mapping.

Build step 24: verify p95 query target with 5000-cell filter.

Build step 25: verify materialization target with 1 million cell fixture.

Build step 26: verify archived heat map blocks annotation.

Build step 27: verify all exports carry audit_id.

Build step 28: verify redacted finance cells keep drill-safe metadata.

Build step 29: add rollback migration in reverse dependency order.

Build step 30: attach PR evidence for policy, API, audit, and SLO checks.
