---
doc_class: ImplementationPlan
ip_id: IP-002-cedar-default-deny
microservice: contact-center
related_adrs: [ADR-0243, ADR-0244, ADR-0246, ADR-0263, ADR-0294, ADR-0321]
journey_id: J-CC-02-supervisor-permitted-intervention
status: proposed
date: 2026-05-20
owner: axis-contact-center
availability: paid
---

# IP-002: Contact Center Cedar Default Deny

## Context

This slice replaces Genesys roles, NICE CXone security profiles, Five9 admin profiles, Talkdesk roles, and AWS security profiles with explicit Cedar checks. Omar Watkins is the named persona: he must prove supervisors cannot barge, monitor, export recordings, or reroute emergency calls without tenant, pack, and reason context.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `contact_center_policy_binding` | `binding_id` | `uuid primary key` | Immutable contact-center policy binding. |
| `contact_center_policy_binding` | `tenant_id` | `uuid not null` | Tenant partition. |
| `contact_center_policy_binding` | `scope_id` | `uuid not null` | FK to contact-center scope. |
| `contact_center_policy_binding` | `action_name` | `text not null` | Cedar action. |
| `contact_center_policy_binding` | `queue_allowlist` | `text[] not null` | Queues covered by binding. |
| `contact_center_policy_binding` | `recording_access_mode` | `text not null` | `none`, `metadata`, `redacted`, `full`. |
| `contact_center_policy_binding` | `policy_version` | `bigint not null` | Fragment version. |
| `contact_center_policy_binding` | `soak_started_at` | `timestamptz not null` | ADR-0294 fragment soak. |

## API Endpoints

REST `POST /v1/contact-center/policy/evaluate`

```json
{
  "principal": "User::supervisor.17",
  "action": "contactCenter::MonitorInteraction",
  "resource": "Interaction::int_9921",
  "context": {
    "tenant_id": "018f8ad2-0e0f-7ad2-92d2-cc000001",
    "queue_id": "queue_vip_support",
    "recording_access_mode": "redacted",
    "reason_code": "quality_review"
  }
}
```

gRPC `ContactCenterPolicyService.Evaluate(EvaluateContactPolicyRequest)` returns `decision`, `policy_version`, `deny_reason`, and `audit_event_id`.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"agent"` | `contactCenter::AcceptInteraction` | `Interaction::*` | `tenant_id`, `queue_id`, `channel`, `agent_state` |
| `User::"supervisor"` | `contactCenter::MonitorInteraction` | `Interaction::*` | `queue_id`, `reason_code`, `recording_access_mode` |
| `User::"admin"` | `contactCenter::ExportRecording` | `Recording::*` | `ticket_id`, `pack_id`, `redaction_required` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Genesys Role | `ContactCenterPermitBinding` | permissions become action names and queue scopes. |
| NICE CXone Security Profile | `ContactCenterPermitBinding` | module rights map to actions. |
| Five9 Profile | `ContactCenterPermitBinding` | admin flags map to explicit resources. |
| Talkdesk Role | `ContactCenterPermitBinding` | role capabilities map to action rows. |
| AWS Security Profile | `ContactCenterPermitBinding` | permissions map to Cedar actions; ARNs stay source refs. |

## Workflow Steps

1. `LoadPolicyBinding` fetches scope and active fragment version.
2. `CompileInteractionContext` adds queue, channel, recording, and reason.
3. `EvaluateDefaultDeny` uses policy-engine library-first evaluation.
4. `ExplainDeny` returns missing queue, role, reason, or pack detail.
5. `SealDecision` writes audit event.

Branches: missing reason denies monitor/export; emergency interaction forbids silent monitor unless emergency policy permits; stale fragment denies mutation.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-CONTACT-CENTER-POLICY-DECISION` | `tenant_id`, `principal`, `action`, `resource`, `decision`, `policy_version` |
| `EVT-CONTACT-CENTER-POLICY-DENIED` | `deny_reason`, `queue_id`, `recording_access_mode`, `cedar_decision_id` |
| `EVT-CAPABILITY-INVOKED` | Emitted before supervisor intervention capability runs. |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Policy evaluate | 7 ms | 30 ms | 65 ms | 12k eval/s/cell | 99.99% |
| Policy publish | 80 ms | 450 ms | 900 ms | 20 publishes/min/cell | 99.95% |

## Failure Modes + Recovery

- Policy-engine unavailable: fail closed for monitor/export/reroute; allow current answered calls to continue.
- Fragment soak incomplete: reject publish and keep prior version.
- Vendor admin import too broad: stage in shadow binding and require admin remap.

## Migration Notes

Vendor roles often bundle queue management, recording export, and admin settings. Migration must decompose them into action/resource/context bindings and never infer `ExportRecording` from a generic administrator role.

## Cross-µservice Handoffs

- `policy-engine` evaluates Cedar.
- `identity` resolves agent, supervisor, and service principals.
- `audit-chain` seals decisions.
- `workflow-engine` consumes policy results for routing flows.
- `data-boundary` tags recording and transcript access classes.
