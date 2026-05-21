---
doc_class: ImplementationPlan
ip_id: IP-004-workflow-template-library
microservice: performance-management
related_adrs: [ADR-0035, ADR-0243, ADR-0257, ADR-0263, ADR-0321]
journey_id: J-PM-04-review-cycle-workflow
status: proposed
date: 2026-05-20
owner: axis-performance-management
capability_tier: T2
---

# IP-004: Performance Management Workflow Template Library

## Context

This slice makes review, feedback, and calibration workflows explicit. It subsumes Lattice review cycles, Culture Amp survey programs, Workday Talent review templates, and 15Five Best-Self Review flows. Marcus Chen needs a review cycle that can pause for works-council approval, route manager reviews, and seal calibration evidence.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `performance_workflow_template` | `template_id` | `uuid primary key` | Versioned template id. |
| `performance_workflow_template` | `tenant_id` | `uuid not null` | Tenant owner. |
| `performance_workflow_template` | `template_kind` | `text not null` | `review_cycle`, `goal_cycle`, `engagement_pulse`, `calibration`. |
| `performance_workflow_template` | `node_graph` | `jsonb not null` | Workflow nodes and branches. |
| `performance_workflow_template` | `active_version` | `integer not null` | Published version. |
| `performance_workflow_template` | `rollback_version` | `integer` | Last safe version. |

## API Endpoints

REST `POST /v1/performance-management/workflow-templates`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-pm000001",
  "template_kind": "review_cycle",
  "node_graph": {
    "nodes": [
      {"id": "gate.works_council", "kind": "labor_pack_gate"},
      {"id": "task.manager_review", "kind": "manager_review"},
      {"id": "task.employee_ack", "kind": "employee_ack"},
      {"id": "calibration.panel", "kind": "calibration_gate"}
    ]
  }
}
```

gRPC `PerformanceWorkflowTemplateService.Publish(PublishPerformanceTemplateRequest)` returns workflow-engine template ref.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"hrbp"` | `performanceManagement::PublishWorkflowTemplate` | `PerformanceWorkflowTemplate::*` | `tenant_id`, `template_kind`, `node_kinds` |
| `Service::"review-runner"` | `performanceManagement::ExecuteWorkflowNode` | `PerformanceWorkflowNode::*` | `cycle_id`, `node_id`, `worker_relation` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Lattice Review Cycle | `PerformanceWorkflowTemplate` | steps map to review and calibration nodes. |
| Culture Amp Survey Program | `PerformanceWorkflowTemplate` | launch, reminder, close steps map to nodes. |
| Workday Talent Review Template | `PerformanceWorkflowTemplate` | routing rules map to manager-chain nodes. |
| 15Five Review Flow | `PerformanceWorkflowTemplate` | check-in/review steps map to feedback and review nodes. |

## Workflow Steps

1. `ValidateNodeKinds` rejects vendor-only automation.
2. `AttachLaborPackGates` inserts pack approval nodes.
3. `AttachPolicyGates` inserts Cedar checks for sensitive transitions.
4. `PublishToWorkflowEngine` registers template.
5. `SealTemplateVersion` emits publish evidence.

Branches: labor pack requires works-council node; calibration node without cohort floor denies publish; rollback version required after first publish.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-PERFORMANCE-WORKFLOW-TEMPLATE-PUBLISHED` | `tenant_id`, `template_id`, `template_kind`, `active_version` |
| `EVT-PERFORMANCE-WORKFLOW-NODE-DENIED` | `cycle_id`, `node_id`, `deny_reason` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Template validation | 70 ms | 250 ms | 550 ms | 300 templates/min/cell | 99.95% |
| Runtime node decision | 8 ms | 35 ms | 80 ms | 10k node decisions/s/cell | 99.99% |

## Failure Modes + Recovery

- Workflow-engine publish timeout: retry idempotently with same version.
- Labor gate missing: deny publish and return required node.
- Runtime node denied: pause cycle and notify HRBP.

## Migration Notes

Vendor review templates often bundle survey, review, and calibration behavior. Migration extracts the graph and maps side effects to explicit nodes instead of copying opaque automation.

## Cross-µservice Handoffs

- `workflow-engine` executes templates.
- `policy-engine` gates sensitive nodes.
- `hris` supplies manager chain and population.
- `notification` sends review reminders.
- `audit-chain` seals workflow publish and node events.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/performance-management/IP-004-workflow-template-library.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/performance-management/IP-004-workflow-template-library.md`, `microservices/performance-management/manifest.json`, `microservices/performance-management/ARCHITECTURE.md`, `microservices/performance-management/PRD.md`, `microservices/performance-management/multi-region.md`, `microservices/performance-management/capacity-model.md`].
