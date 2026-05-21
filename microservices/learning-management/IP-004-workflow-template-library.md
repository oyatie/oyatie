---
doc_class: ImplementationPlan
ip_id: IP-004-workflow-template-library
microservice: learning-management
related_adrs: [ADR-0244, ADR-0257, ADR-0263]
journey_id: J-LMS-04-repeatable-learning-workflows
status: proposed
date: 2026-05-20
owner: axis-learning-management
capability_tier: T2
---

# IP-004: Workflow Template Library

## Context

This slice defines reusable learning workflow templates for course assignment, learner enrollment, manager approval, assessment, completion evidence, credential issue, and compliance renewal. It supports Hana Mori, the global enablement lead, replacing Workday Learning campaigns, Cornerstone learning plans, Docebo automations, 360Learning paths, and LinkedIn Learning Enterprise content assignments with named, inspectable workflow nodes.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `learning_workflow_template` | `template_id` | `uuid primary key` | Versioned workflow template. |
| `learning_workflow_template` | `tenant_id` | `uuid not null` | Tenant partition. |
| `learning_workflow_template` | `template_code` | `text not null` | Stable template code. |
| `learning_workflow_template` | `workflow_nodes` | `jsonb not null` | Named node graph. |
| `learning_workflow_template` | `decision_branches` | `jsonb not null` | Branch predicates and outcomes. |
| `learning_workflow_template` | `slo_profile` | `jsonb not null` | Runtime SLO budget. |
| `learning_workflow_template` | `version` | `integer not null` | Monotonic version. |

## API Endpoints

REST `POST /v1/learning-management/workflow-templates`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-lms00004",
  "template_code": "compliance-training-renewal-v1",
  "workflow_nodes": [
    {"node": "AssignCourse", "type": "command"},
    {"node": "AwaitCompletion", "type": "wait"},
    {"node": "EvaluateAssessment", "type": "decision"},
    {"node": "IssueCredential", "type": "command"}
  ],
  "decision_branches": {
    "EvaluateAssessment": {"pass": "IssueCredential", "fail": "ScheduleRetake"}
  }
}
```

gRPC `LearningWorkflowTemplateService.Publish(PublishLearningWorkflowTemplateRequest)` returns `template_id`, `version`, and `validation_warnings`.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"learning-architect"` | `learningManagement::PublishWorkflowTemplate` | `LearningWorkflowTemplate::*` | `tenant_id`, `template_code`, `version` |
| `Service::"workflow-engine"` | `learningManagement::ExecuteWorkflowNode` | `LearningWorkflowRun::*` | `node`, `template_code`, `learner_ref` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Cornerstone Learning Assignment | `LearningWorkflowTemplate` | assignment, due date, and recurrence map to nodes. |
| Workday Learning Campaign | `LearningWorkflowTemplate` | campaign audience maps to assignment branch. |
| Docebo Automation | `LearningWorkflowTemplate` | trigger/action pair maps to workflow nodes. |
| 360Learning Path | `LearningWorkflowTemplate` | collaborative review maps to instructor approval node. |
| LinkedIn Learning Enterprise Recommendation | `LearningWorkflowTemplate` | content recommendation maps to optional assignment node. |

## Workflow Steps

1. `ValidateTemplateGraph` ensures all branches point to named nodes.
2. `CompileDecisionBranches` converts branch predicates into workflow-engine format.
3. `EvaluatePublishPolicy` checks learning architect permission.
4. `PersistTemplateVersion` stores immutable version.
5. `RegisterTemplate` exposes the template to assignment and campaign commands.

Branches: missing failure branch rejects; credential node without completion evidence rejects; provider-only content assignment skips assessment node.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-LEARNING-WORKFLOW-TEMPLATE-PUBLISHED` | `tenant_id`, `template_code`, `template_id`, `version` |
| `EVT-LEARNING-WORKFLOW-TEMPLATE-REJECTED` | `tenant_id`, `template_code`, `validation_error` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Publish template | 60 ms | 240 ms | 600 ms | 200 rps/cell | 99.95% |
| Resolve template for run | 5 ms | 20 ms | 50 ms | 25k lookups/s/cell | 99.99% |

## Failure Modes + Recovery

- Invalid branch graph: reject before persistence with node names in response.
- Workflow-engine registration timeout: persist version as unpublished and retry registration.
- Provider content missing: mark optional node skipped and keep compliance nodes active.

## Migration Notes

Vendor automations import as draft workflow templates. Opaque Workday campaign logic and Docebo automation scripts must be expanded into named nodes before publication.

## Cross-µservice Handoffs

- `workflow-engine` executes published node graphs.
- `notification` sends assignment and due-date reminders.
- `credentialing` issues credentials after sealed completion.
- `audit-chain` records publication and rejection events.
- `calendar` receives instructor-led session nodes.
