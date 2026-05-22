---
doc_class: ImplementationPlan
ip_id: IP-004-workflow-template-library
microservice: marketing-automation
related_adrs: [ADR-0035, ADR-0243, ADR-0257, ADR-0263, ADR-0321]
journey_id: J-MA-04-cross-channel-nurture
status: proposed
date: 2026-05-20
owner: axis-marketing-automation
tenant_class_aware: true
---

# IP-004: Marketing Automation Workflow Template Library

## Context

This slice gives Marcus Chen a buildable nurture workflow that subsumes Marketo Engagement Programs, HubSpot Workflows, Mailchimp Premium Customer Journeys, Iterable Journeys, and Braze Canvas. The key distinction: vendor canvases become versioned workflow templates whose nodes call Cedar, consent, frequency, and audit services before any channel send.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_workflow_template` | `template_id` | `uuid primary key` | Versioned template id. |
| `marketing_workflow_template` | `tenant_id` | `uuid not null` | Tenant owner. |
| `marketing_workflow_template` | `template_slug` | `text not null` | Example `q2-upgrade-nurture`. |
| `marketing_workflow_template` | `node_graph` | `jsonb not null` | DAG/state-machine hybrid nodes. |
| `marketing_workflow_template` | `entry_criteria_projection_id` | `uuid not null` | Segment projection from IP-003. |
| `marketing_workflow_template` | `active_version` | `integer not null` | Incremented on publish. |
| `marketing_workflow_template` | `rollback_version` | `integer` | Last known safe version. |

## API Endpoints

REST `POST /v1/marketing-automation/workflow-templates`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-ma000001",
  "template_slug": "q2-upgrade-nurture",
  "entry_criteria_projection_id": "01HXMA_PROJ",
  "node_graph": {
    "nodes": [
      {"id": "entry.segment.match", "kind": "segment_gate"},
      {"id": "decision.consent.email", "kind": "consent_gate"},
      {"id": "action.mail.send", "kind": "mail_handoff"},
      {"id": "wait.product.signal", "kind": "event_wait"}
    ]
  }
}
```

gRPC `MarketingWorkflowTemplateService.Publish(PublishTemplateRequest)` returns `template_id`, `active_version`, and `workflow_engine_template_ref`.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"marketing.manager"` | `marketingAutomation::PublishWorkflowTemplate` | `WorkflowTemplate::*` | `tenant_id`, `node_kinds`, `entry_projection_id` |
| `Service::"journey-runner"` | `marketingAutomation::ExecuteWorkflowNode` | `WorkflowNode::*` | `tenant_id`, `node_id`, `purpose`, `channel` |
| `User::"auditor"` | `marketingAutomation::ReadWorkflowTemplate` | `WorkflowTemplate::*` | `read_reason`, `version` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Marketo Engagement Program | `MarketingWorkflowTemplate` | streams become `node_graph.nodes`; transitions become edges. |
| HubSpot Workflow | `MarketingWorkflowTemplate` | enrollment triggers become `entry_criteria_projection_id`. |
| Mailchimp Premium Customer Journey | `MarketingWorkflowTemplate` | journey points become typed nodes. |
| Iterable Journey | `MarketingWorkflowTemplate` | campaign steps become channel handoff nodes. |
| Braze Canvas | `MarketingWorkflowTemplate` | canvas variants become branch nodes with experiment metadata. |

## Workflow Steps

1. `ValidateNodeKinds` refuses unknown or vendor-only nodes.
2. `ResolveEntryProjection` ensures IP-003 committed a target segment.
3. `AttachPolicyGates` inserts Cedar and consent gates before channel nodes.
4. `PublishToWorkflowEngine` registers the DAG/state-machine template.
5. `SealTemplateVersion` stores active and rollback versions.

Branches: channel node without consent gate is rewritten or denied; vendor wait node without maximum duration is capped at 30 days; template publish denied if rollback version is missing after first publish.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-WORKFLOW-TEMPLATE-PUBLISHED` | `tenant_id`, `template_id`, `active_version`, `node_count` |
| `EVT-MARKETING-WORKFLOW-NODE-DENIED` | `tenant_id`, `template_id`, `node_id`, `deny_reason` |
| `EVT-CAPABILITY-INVOKED` | Template publish capability invocation. |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Template validation | 65 ms | 220 ms | 500 ms | 500 templates/min/cell | 99.95% |
| Publish to workflow-engine | 110 ms | 700 ms | 1.4 s | 120 publishes/min/cell | 99.9% |

## Failure Modes + Recovery

- Workflow-engine publish timeout: keep template in `publish_pending` and retry idempotently with same version.
- Unsafe vendor node: deny publish and return node-specific remediation.
- Bad version: rollback to `rollback_version` and emit node graph diff.

## Migration Notes

Marketo, HubSpot, Mailchimp Premium, Iterable, and Braze all expose UI canvas features that can hide provider-specific behavior. Migration imports graph shape only; sends, waits, split tests, and exits become typed nodes that call Oyatie services.

## Cross-µservice Handoffs

- `workflow-engine` executes templates.
- `mail`, `messenger`, and `notification` own channel delivery.
- `consent` evaluates purpose at every send node.
- `audit-chain` records publish and node-deny evidence.
- `experimentation` can later consume branch metadata without owning workflows.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-004-workflow-template-library.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/marketing-automation/IP-004-workflow-template-library.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].
