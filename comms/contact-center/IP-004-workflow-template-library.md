---
doc_class: ImplementationPlan
ip_id: IP-004-workflow-template-library
microservice: contact-center
related_adrs: [ADR-0035, ADR-0243, ADR-0257, ADR-0263, ADR-0321]
journey_id: J-CC-04-voice-chat-routing-flow
status: proposed
date: 2026-05-20
owner: axis-contact-center
availability: paid
---

# IP-004: Contact Center Workflow Template Library

## Context

This slice makes routing flows buildable as workflow templates rather than vendor scripts. It subsumes Genesys Architect flows, NICE CXone Studio scripts, Five9 IVR scripts, Talkdesk Studio flows, and AWS contact flows. The named persona is Marcus Chen, who needs predictable service recovery when a VIP queue, callback, or escalation branch fails.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `contact_routing_template` | `template_id` | `uuid primary key` | Versioned routing template. |
| `contact_routing_template` | `tenant_id` | `uuid not null` | Tenant owner. |
| `contact_routing_template` | `entry_channel` | `text not null` | `voice`, `chat`, `sms`, `email`. |
| `contact_routing_template` | `node_graph` | `jsonb not null` | Queue, IVR, policy, callback, and escalation nodes. |
| `contact_routing_template` | `active_version` | `integer not null` | Published version. |
| `contact_routing_template` | `rollback_version` | `integer` | Last safe version. |

## API Endpoints

REST `POST /v1/contact-center/routing-templates`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-cc000001",
  "entry_channel": "voice",
  "node_graph": {
    "nodes": [
      {"id": "entry.voice", "kind": "ingress"},
      {"id": "decision.recording-consent", "kind": "consent_gate"},
      {"id": "route.vip", "kind": "queue_route"},
      {"id": "fallback.callback", "kind": "callback_offer"}
    ]
  }
}
```

gRPC `ContactRoutingTemplateService.Publish(PublishContactTemplateRequest)` returns `template_id`, `active_version`, and workflow-engine reference.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"contact-center.admin"` | `contactCenter::PublishRoutingTemplate` | `ContactRoutingTemplate::*` | `tenant_id`, `entry_channel`, `node_kinds` |
| `Service::"router"` | `contactCenter::ExecuteRoutingNode` | `RoutingNode::*` | `interaction_id`, `node_id`, `queue_id`, `emergency_flag` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Genesys Architect Flow | `ContactRoutingTemplate` | flow actions become typed nodes. |
| NICE CXone Studio Script | `ContactRoutingTemplate` | script branches become workflow branches. |
| Five9 IVR Script | `ContactRoutingTemplate` | prompts become metadata; decisions become nodes. |
| Talkdesk Studio Flow | `ContactRoutingTemplate` | flow components map to node kinds. |
| AWS Contact Flow | `ContactRoutingTemplate` | contact blocks map to nodes and edges. |

## Workflow Steps

1. `ValidateNodeKinds` rejects vendor-only script operations.
2. `AttachPolicyGates` inserts Cedar checks for monitor, transfer, and export nodes.
3. `AttachConsentGate` inserts recording consent before capture.
4. `PublishToWorkflowEngine` registers template.
5. `SealTemplateVersion` emits publish event.

Branches: emergency flag routes to emergency-safe template; recording node without consent gate denies; callback node without SLA returns validation error.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-CONTACT-CENTER-ROUTING-TEMPLATE-PUBLISHED` | `tenant_id`, `template_id`, `active_version`, `entry_channel` |
| `EVT-CONTACT-CENTER-ROUTING-NODE-DENIED` | `interaction_id`, `node_id`, `deny_reason` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Template validation | 55 ms | 210 ms | 480 ms | 400 templates/min/cell | 99.95% |
| Runtime node decision | 5 ms | 20 ms | 50 ms | 20k node decisions/s/cell | 99.99% |

## Failure Modes + Recovery

- Workflow-engine publish timeout: keep template pending and retry with same version.
- Runtime node denied: follow branch fallback and emit node-denied evidence.
- Vendor flow loop import: halt publish with loop path and remediation.

## Migration Notes

Vendor flow engines embed prompts, queues, Lambdas, and API calls. Migration imports graph shape only; side effects become typed Oyatie handoff nodes.

## Cross-µservice Handoffs

- `workflow-engine` executes routing templates.
- `consent` handles recording consent.
- `telephony-adapter` owns call ingress and media setup.
- `audit-chain` seals template publish and denied nodes.
- `notification` handles callback reminders.
