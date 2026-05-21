---
doc_class: IP
ip_id: IP-004
microservice: financial-planning
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0246
  - ADR-0253-amendment
  - ADR-0257
  - ADR-0258
  - ADR-0263
  - ADR-0294
  - ADR-0296
  - ADR-0297
  - ADR-0314
  - ADR-0321
journey_ref: J-FP-004-workflow-template-library
tenant_class: product-critical
status: implementation-ready
date: 2026-05-20
owner_team: axis-financial-planning + axis-workflow
---

# IP-004 Financial Planning workflow-template-library

Service: financial-planning
ChangeSet scope: microservices/financial-planning/IP-004-workflow-template-library.md
Benchmarks: Anaplan, Workday Adaptive Planning, OneStream, Vena, Pigment
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321

## Objective
- workflow-template-library-objective 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- workflow-template-library-objective 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- workflow-template-library-objective 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- workflow-template-library-objective 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- workflow-template-library-objective 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- workflow-template-library-objective 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Prerequisites
- workflow-template-library-prerequisites 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- workflow-template-library-prerequisites 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- workflow-template-library-prerequisites 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- workflow-template-library-prerequisites 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- workflow-template-library-prerequisites 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- workflow-template-library-prerequisites 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Implementation steps
- workflow-template-library-implementation-steps 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- workflow-template-library-implementation-steps 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- workflow-template-library-implementation-steps 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- workflow-template-library-implementation-steps 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- workflow-template-library-implementation-steps 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- workflow-template-library-implementation-steps 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Tests and evidence
- workflow-template-library-tests-and-evidence 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- workflow-template-library-tests-and-evidence 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- workflow-template-library-tests-and-evidence 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- workflow-template-library-tests-and-evidence 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- workflow-template-library-tests-and-evidence 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- workflow-template-library-tests-and-evidence 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Rollback
- workflow-template-library-rollback 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- workflow-template-library-rollback 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- workflow-template-library-rollback 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- workflow-template-library-rollback 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- workflow-template-library-rollback 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- workflow-template-library-rollback 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Acceptance criteria
- workflow-template-library-acceptance-criteria 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- workflow-template-library-acceptance-criteria 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- workflow-template-library-acceptance-criteria 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- workflow-template-library-acceptance-criteria 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- workflow-template-library-acceptance-criteria 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- workflow-template-library-acceptance-criteria 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Context
- IP-004 defines reusable finance planning workflow templates, not ad hoc workflow instances.
- The workflow library must cover forecast version open, driver import, scenario recalculate, consolidation close, variance explain, board report seal, and budget lock breakglass.
- Vendor parity requires named templates that can model Anaplan model-change approval, Workday Adaptive Planning budget cycles, Oracle EPM Cloud close orchestration, OneStream certification, Vena workbook approval, Pigment scenario simulation, Planful driver import, IBM Planning Analytics chore review, Board capsule procedures, and Jedox rule promotion.
- Templates are tenant-scoped and versioned; published templates are immutable.
- Draft templates may be changed only by finance-planning-owner principals.
- Workflow templates depend on IP-003 ontology projection for object identifiers and on IP-008 policy hooks for action admission.
- Runtime execution remains in workflow-engine; this file owns the Financial Planning template catalog.
- Template nodes must carry audit event classes so runtime execution can emit ADR-0263 evidence without string guessing.
- Every template exposes branch names because finance operators need human-readable close and forecast evidence.
- The library must support migration from vendor-native workflow concepts without importing vendor execution engines.

## Data Model Deltas
```sql
CREATE TABLE financial_planning_workflow_template (
  template_id UUID PRIMARY KEY,
  tenant_id UUID NOT NULL,
  template_key TEXT NOT NULL,
  template_version BIGINT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('draft', 'published', 'retired')),
  vendor_lineage TEXT[] NOT NULL DEFAULT '{}',
  entry_object_type TEXT NOT NULL,
  node_graph JSONB NOT NULL,
  branch_contract JSONB NOT NULL,
  audit_event_classes TEXT[] NOT NULL,
  policy_action TEXT NOT NULL,
  published_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, template_key, template_version)
);
CREATE INDEX fp_workflow_template_status_idx
  ON financial_planning_workflow_template (tenant_id, template_key, status);
CREATE INDEX fp_workflow_template_graph_gin
  ON financial_planning_workflow_template USING gin (node_graph jsonb_path_ops);
```

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FinancialPlanningTemplateKey {
    ForecastVersionOpen,
    DriverModelImport,
    ScenarioRecalculate,
    ConsolidationClose,
    VarianceExplain,
    BoardReportSeal,
    BudgetLockBreakglass,
}

#[derive(Clone, Debug)]
pub struct WorkflowTemplateNode {
    pub node_id: String,
    pub node_kind: String,
    pub policy_action: String,
    pub audit_class: String,
    pub timeout_seconds: u32,
}

#[derive(Clone, Debug)]
pub struct FinancialPlanningWorkflowTemplate {
    pub tenant_id: uuid::Uuid,
    pub template_key: FinancialPlanningTemplateKey,
    pub template_version: i64,
    pub vendor_lineage: Vec<String>,
    pub nodes: Vec<WorkflowTemplateNode>,
    pub branch_contract: serde_json::Value,
}
```

## API Endpoints
- REST `POST /v1/financial-planning/workflow-templates`
```json
{
  "tenant_id": "018f9a60-7b8d-7f11-a9f1-0c7f4b9f0004",
  "template_key": "consolidation-close",
  "vendor_lineage": ["Oracle EPM Cloud", "OneStream", "Vena"],
  "entry_object_type": "ConsolidationNode",
  "nodes": [
    {"node_id": "close-window-open", "node_kind": "gate", "audit_class": "ADR0263WorkflowTemplatePublished"},
    {"node_id": "intercompany-elimination", "node_kind": "task", "audit_class": "ADR0263WorkflowNodeCompleted"}
  ]
}
```
- REST `POST /v1/financial-planning/workflow-templates/{template_id}:publish` freezes the draft and emits template evidence.
- gRPC `PublishFinancialPlanningTemplate(PublishFinancialPlanningTemplateRequest) returns (PublishFinancialPlanningTemplateResponse)` lets workflow-engine validate compiled node graphs.
```json
{
  "tenant_id": "018f9a60-7b8d-7f11-a9f1-0c7f4b9f0004",
  "template_id": "018f9a60-7b8d-7f11-a9f1-0c7f4b9f2404",
  "template_key": "consolidation-close",
  "compiled_by": "workflow-engine",
  "expected_version": 4
}
```
- AsyncAPI topic `financial-planning.workflow-template.published.v1`
```json
{
  "event_id": "evt-fp-template-004",
  "tenant_id": "018f9a60-7b8d-7f11-a9f1-0c7f4b9f0004",
  "template_key": "driver-model-import",
  "template_version": 7,
  "vendor_lineage": ["Planful", "Anaplan", "Pigment"]
}
```

## Cedar Policy Hooks
```cedar
permit (
  principal in FinancialPlanning::Role::"finance-planning-owner",
  action == FinancialPlanning::Action::"workflow_template.publish",
  resource
) when {
  resource.tenant_id == principal.tenant_id &&
  resource.status == "draft" &&
  context.audit_class == "ADR0263WorkflowTemplatePublished" &&
  context.compiled_by == "workflow-engine"
};
```
- Principal: finance-planning-owner for publish, finance-planning-analyst for read, workflow-engine service for compile verification.
- Action: `workflow_template.create`, `workflow_template.publish`, `workflow_template.retire`, `workflow_template.read`.
- Resource: `FinancialPlanning::WorkflowTemplate::<template_id>`.
- Context: `tenant_id`, `template_key`, `compiled_by`, `vendor_lineage`, `audit_class`.

## Ontology Projection
- Anaplan `Process` becomes `WorkflowTemplate`: `processName -> template_key`, `actionList -> node_graph`, `modelId -> vendor_lineage`.
- Workday Adaptive Planning `WorkflowStatus` becomes `WorkflowBranch`: `statusName -> branch_name`, `stepOwner -> principal_role`, `version -> entry_object_ref`.
- Oracle EPM Cloud `TaskManagerTask` becomes `WorkflowTemplateNode`: `taskId -> node_id`, `predecessors -> depends_on`, `dueDate -> timeout_policy`.
- OneStream `CertificationStep` becomes `WorkflowTemplateNode`: `certifier -> approver_role`, `profile -> consolidation_node_ref`.
- Vena `ApprovalWorkflow` becomes `WorkflowTemplate`: `templateId -> template_key`, `approverRange -> evidence_binding`.
- Pigment `ApplicationWorkflow` becomes `WorkflowBranch`: `simulationStep -> branch_name`, `blockRef -> scenario_assumption_ref`.
- Planful `ProcessFlow` becomes `WorkflowTemplate`: `driverLoadStep -> node_id`, `spreadMethodReview -> branch_contract`.
- IBM Planning Analytics `Chore` becomes `WorkflowTemplateNode`: `choreName -> node_id`, `processSequence -> depends_on`.
- Board `Procedure` becomes `WorkflowTemplateNode`: `procedureName -> node_id`, `capsuleScreen -> operator_surface`.
- Jedox `IntegratorJob` becomes `WorkflowTemplateNode`: `jobName -> node_id`, `loadMode -> branch_condition`.

## Workflow Steps
- Node `draft-template-created`: validate template key and vendor lineage.
- Node `bind-entry-object`: require IP-003 ontology object type.
- Node `compile-node-graph`: ask workflow-engine to validate graph shape.
- Branch `forecast-template`: requires version-open and assumption-review nodes.
- Branch `close-template`: requires consolidation, certification, variance, and seal nodes.
- Branch `import-template`: requires source-validate, transform, reconcile, and rollback nodes.
- Node `attach-policy-actions`: bind Cedar action names to every command node.
- Node `attach-audit-classes`: bind ADR-0263 classes before publish.
- Node `publish-template`: immutable version write.
- Node `notify-runtime`: emit AsyncAPI published event.

## Audit Events
- `ADR0263WorkflowTemplateCreated`: draft template created.
- `ADR0263WorkflowTemplateCompiled`: workflow-engine validated node graph.
- `ADR0263WorkflowTemplatePublished`: immutable template published.
- `ADR0263WorkflowTemplateRetired`: template version retired.
- `ADR0263WorkflowNodeCompleted`: runtime node completion class embedded into template.
- `ADR0263WorkflowBranchSelected`: runtime branch selection class embedded into template.

## SLO Targets
- p50 template read latency: 25 ms.
- p95 template read latency: 90 ms.
- p99 template read latency: 220 ms.
- Throughput: 300 template publish validations per tenant per hour.
- Availability: 99.95% for template read and publish APIs.
- Compile freshness: workflow-engine compile response within 3 seconds at p95.

## Failure Modes + Recovery
- Template graph has a cycle: reject publish, keep draft, emit compile failure evidence, and return node cycle path.
- Missing policy action on node: reject publish and identify node ids needing IP-008 action binding.
- Vendor workflow contains executable script: strip script body, store provenance only, and require human-authored Oyatie node replacement.
- Workflow-engine compile timeout: retain draft and retry compile with same idempotency key.
- Published template needs emergency withdrawal: retire version, publish replacement version, and keep runtime executions pinned to original version.
- Audit class omitted: fail closed because ADR-0263 evidence would be incomplete.

## Migration Notes
- Anaplan process imports map to reviewable templates but Anaplan action scripts do not execute in Oyatie.
- Workday Adaptive Planning budget-cycle statuses map to branch names and approver roles.
- Oracle EPM Cloud Task Manager tasks seed close and certification templates.
- OneStream certification workflows seed consolidation-close branch contracts.
- Vena approval workflows seed board-report-seal and variance-explain templates.
- Pigment application workflows seed scenario-recalculate templates.
- Planful process flows seed driver-model-import templates.
- IBM Planning Analytics chores become reviewable workflow nodes, not background shell jobs.
- Board procedures seed operator-guided workflow nodes.
- Jedox Integrator jobs seed import nodes after connector validation.

## Cross-Microservice Handoffs
- To `workflow-engine`: compile and execute published templates.
- To `ontology`: resolve entry object identifiers and node object references.
- To `audit-chain`: seal template lifecycle and embedded runtime classes.
- To `policy-cedar`: validate every declared template action.
- To `connect`: ingest vendor workflow metadata from migration adapters.
- To `financial-planning` IP-005 and IP-006: expose REST and AsyncAPI surfaces for template lifecycle.
- To `ops-dashboard-control-center`: display template publish failures and retired-version drift.
