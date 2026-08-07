---
doc_class: ImplementationPlan
ip_id: IP-030-callback-and-sla-rescheduler
microservice: contact-center
related_adrs: [ADR-0035, ADR-0243, ADR-0263, ADR-0321]
journey_id: J-CC-30-callback-sla-recovery
status: proposed
date: 2026-05-20
owner: axis-contact-center
availability: paid
---

# IP-030: Callback and SLA Rescheduler

## Context

This net-new slice makes callback offers and missed-SLA recovery explicit. It displaces Genesys callbacks, NICE CXone callbacks, Five9 callbacks, Talkdesk callback scheduling, and AWS callback flows. The target persona is Omar Watkins handling a queue incident where customers must not be silently dropped.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `contact_callback_request` | `callback_id` | `uuid primary key` | One callback request. |
| `contact_callback_request` | `tenant_id` | `uuid not null` | Tenant partition. |
| `contact_callback_request` | `queue_id` | `uuid not null` | Owning queue. |
| `contact_callback_request` | `customer_ref` | `text not null` | Hashed customer/profile ref. |
| `contact_callback_request` | `requested_after` | `timestamptz not null` | Earliest callback time. |
| `contact_callback_request` | `sla_deadline` | `timestamptz not null` | Latest acceptable time. |
| `contact_callback_request` | `state` | `text not null` | `scheduled`, `rescheduled`, `completed`, `missed`, `cancelled`. |

## API Endpoints

REST `POST /v1/contact-center/callbacks/{callback_id}:reschedule`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-cc000001",
  "reason_code": "queue_over_capacity",
  "new_requested_after": "2026-05-20T20:00:00Z",
  "customer_notice_channel": "sms"
}
```

gRPC `CallbackSlaService.Reschedule(RescheduleCallbackRequest)` returns `state`, `sla_risk`, and `notice_event_id`.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `Service::"callback-worker"` | `contactCenter::ScheduleCallback` | `CallbackRequest::*` | `tenant_id`, `queue_id`, `sla_deadline` |
| `User::"supervisor"` | `contactCenter::RescheduleCallback` | `CallbackRequest::*` | `reason_code`, `sla_risk`, `notice_channel` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Genesys Callback | `ContactCallbackRequest` | callback id maps to source ref. |
| NICE CXone Callback | `ContactCallbackRequest` | skill and contact id map to queue/customer refs. |
| Five9 Callback | `ContactCallbackRequest` | campaign callback maps to callback request. |
| Talkdesk Callback | `ContactCallbackRequest` | scheduled callback maps to requested_after. |
| AWS Callback Contact | `ContactCallbackRequest` | contact id maps to source ref. |

## Workflow Steps

1. `CreateCallback` writes initial request and SLA.
2. `MonitorSlaBurn` calculates risk from queue state.
3. `RescheduleOrEscalate` chooses new time or supervisor escalation.
4. `NotifyCustomer` sends notice through notification service.
5. `SealCallbackState` emits state change event.

Branches: SLA cannot be met escalates supervisor; customer notice denied cancels reschedule; callback completion closes workflow.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-CONTACT-CENTER-CALLBACK-SCHEDULED` | `tenant_id`, `callback_id`, `queue_id`, `sla_deadline` |
| `EVT-CONTACT-CENTER-CALLBACK-RESCHEDULED` | `callback_id`, `reason_code`, `new_requested_after`, `notice_event_id` |
| `EVT-CONTACT-CENTER-SLA-MISSED` | `callback_id`, `queue_id`, `sla_deadline`, `missed_at` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Schedule callback | 45 ms | 180 ms | 350 ms | 1k rps/cell | 99.95% |
| SLA monitor tick | 100 ms | 600 ms | 1.5 s | 100k callbacks/tick/cell | 99.9% |

## Failure Modes + Recovery

- Queue outage: reschedule all affected callbacks and escalate SLA risk.
- Notification failure: retain callback, mark notice pending, retry with alternate channel.
- Callback worker duplicate fire: idempotency key prevents double call.

## Migration Notes

Vendor callbacks often hide queue SLA and customer notice state. Migration imports scheduled callbacks with source ids, then recalculates SLA risk using Oyatie queue state.

## Cross-µservice Handoffs

- `workflow-engine` schedules callback workflows.
- `notification` sends customer notices.
- `presence` and routing policy provide queue capacity.
- `audit-chain` seals callback lifecycle events.
- `customer-profile` resolves customer communication preferences.
