---
doc_class: ImplementationPlan
ip_id: IP-028-workforce-adherence-stream
microservice: contact-center
related_adrs: [ADR-0244, ADR-0257, ADR-0263, ADR-0321]
journey_id: J-CC-28-agent-adherence-realtime
status: proposed
date: 2026-05-20
owner: axis-contact-center
availability: paid
---

# IP-028: Workforce Adherence Stream

## Context

This net-new slice tracks agent state against schedules without becoming a full workforce-management suite. It displaces Genesys WEM adherence, NICE CXone WFM adherence, Five9 workforce signals, Talkdesk Workforce Management, and AWS agent event streams.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `contact_agent_adherence` | `adherence_id` | `uuid primary key` | One state interval. |
| `contact_agent_adherence` | `tenant_id` | `uuid not null` | Tenant partition. |
| `contact_agent_adherence` | `agent_principal_id` | `text not null` | Identity principal. |
| `contact_agent_adherence` | `scheduled_state` | `text not null` | Expected state. |
| `contact_agent_adherence` | `observed_state` | `text not null` | Live state. |
| `contact_agent_adherence` | `interval_start` | `timestamptz not null` | Start time. |
| `contact_agent_adherence` | `interval_end` | `timestamptz` | End time. |

## API Endpoints

REST `POST /v1/contact-center/agents/{agent_id}/adherence-events`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-cc000001",
  "scheduled_state": "available",
  "observed_state": "after_call_work",
  "interval_start": "2026-05-20T18:00:00Z",
  "source_vendor": "genesys"
}
```

gRPC `AgentAdherenceService.StreamAdherence(StreamAdherenceRequest)` streams agent adherence deltas for dashboards.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `Service::"presence-sync"` | `contactCenter::WriteAdherenceEvent` | `AgentAdherence::*` | `tenant_id`, `agent_principal_id`, `observed_state` |
| `User::"supervisor"` | `contactCenter::ReadAdherence` | `AgentAdherence::*` | `queue_id`, `date_range`, `reason_code` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Genesys Agent Routing Status | `AgentAdherenceInterval` | routing status maps to observed state. |
| NICE CXone Agent State | `AgentAdherenceInterval` | state id maps to observed state. |
| Five9 Agent State | `AgentAdherenceInterval` | status maps to observed state. |
| Talkdesk Agent Activity | `AgentAdherenceInterval` | activity maps to observed state. |
| AWS Agent Event | `AgentAdherenceInterval` | event type maps to interval transition. |

## Workflow Steps

1. `ResolveAgentPrincipal` maps vendor agent id to identity principal.
2. `LoadSchedule` reads expected state from workforce source.
3. `AppendAdherenceInterval` writes interval row.
4. `ComputeDeviation` flags late, unavailable, or over-break states.
5. `PublishDashboardDelta` sends real-time update.

Branches: missing schedule writes observed-only interval; restricted labor pack aggregates by queue; stale vendor event ignored if older than latest HLC.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-CONTACT-CENTER-ADHERENCE-WRITTEN` | `tenant_id`, `agent_principal_id`, `scheduled_state`, `observed_state` |
| `EVT-CONTACT-CENTER-ADHERENCE-READ` | `tenant_id`, `queue_id`, `principal`, `reason_code` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Write adherence event | 20 ms | 90 ms | 180 ms | 15k events/s/cell | 99.95% |
| Dashboard stream fanout | 80 ms | 500 ms | 1.2 s | 5k subscribers/cell | 99.9% |

## Failure Modes + Recovery

- Vendor event gap: mark interval uncertain and request backfill.
- Labor pack denies individual read: return aggregate only.
- Schedule source unavailable: write observed-only and retry reconciliation.

## Migration Notes

Vendor WFM semantics are not uniform. Migration imports raw state history and maps only stable state categories: available, busy, after-call-work, break, offline, training.

## Cross-µservice Handoffs

- `identity` resolves agent principals.
- `presence` supplies live state.
- `audit-chain` seals writes and reads.
- `analytics` consumes aggregate adherence metrics.
- `performance-management` can consume approved coaching signals without owning raw adherence.
