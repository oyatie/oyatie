---
doc_class: ImplementationPlan
ip_id: IP-026-omnichannel-routing-policy-engine
microservice: contact-center
related_adrs: [ADR-0243, ADR-0257, ADR-0263, ADR-0321]
journey_id: J-CC-26-omnichannel-sla-routing
status: proposed
date: 2026-05-20
owner: axis-contact-center
availability: paid
---

# IP-026: Omnichannel Routing Policy Engine

## Context

This net-new slice covers SLA-aware routing across voice, chat, SMS, email, and callback. It displaces Genesys Cloud routing, NICE CXone ACD, Five9 routing profiles, Talkdesk routing, and AWS queues with a policy engine that names every decision input.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `contact_routing_policy` | `policy_id` | `uuid primary key` | Routing policy id. |
| `contact_routing_policy` | `tenant_id` | `uuid not null` | Tenant partition. |
| `contact_routing_policy` | `channel` | `text not null` | Voice/chat/sms/email/callback. |
| `contact_routing_policy` | `priority_expression` | `jsonb not null` | SLA, customer tier, skill, locale. |
| `contact_routing_policy` | `fallback_queue_id` | `uuid not null` | Safe fallback queue. |
| `contact_routing_policy` | `max_wait_seconds` | `integer not null` | SLA threshold. |

## API Endpoints

REST `POST /v1/contact-center/routing:decide`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-cc000001",
  "interaction_id": "int_9921",
  "channel": "chat",
  "customer_segment": "enterprise",
  "required_skills": ["korean", "billing"],
  "sla_deadline": "2026-05-20T19:00:00Z"
}
```

gRPC `ContactRoutingPolicyService.Decide(RouteDecisionRequest)` returns `queue_id`, `priority_score`, and `decision_event_id`.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `Service::"router"` | `contactCenter::DecideRoute` | `ContactRoutingPolicy::*` | `tenant_id`, `channel`, `required_skills`, `sla_deadline` |
| `User::"supervisor"` | `contactCenter::OverrideRoute` | `Interaction::*` | `reason_code`, `from_queue`, `to_queue` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Genesys Queue Routing | `ContactRoutingPolicy` | skill expressions map to priority expression. |
| NICE CXone ACD Skill | `ContactRoutingPolicy` | skill priority maps to route weights. |
| Five9 Routing Profile | `ContactRoutingPolicy` | profile rules become priority expression. |
| Talkdesk Routing Rule | `ContactRoutingPolicy` | conditions become expression clauses. |
| AWS Queue Routing | `ContactRoutingPolicy` | queue and contact attributes become policy inputs. |

## Workflow Steps

1. `LoadRoutingPolicy` reads active channel policy.
2. `ScoreInteraction` computes priority and SLA burn.
3. `EvaluateCedar` checks route and override permits.
4. `ChooseQueue` selects primary or fallback queue.
5. `SealDecision` emits route decision evidence.

Branches: no skill match uses fallback queue; SLA burn above threshold escalates priority; override without reason denies.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-CONTACT-CENTER-ROUTE-DECIDED` | `tenant_id`, `interaction_id`, `channel`, `queue_id`, `priority_score` |
| `EVT-CONTACT-CENTER-ROUTE-OVERRIDDEN` | `interaction_id`, `from_queue`, `to_queue`, `reason_code` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Route decision | 6 ms | 25 ms | 55 ms | 30k decisions/s/cell | 99.99% |
| Supervisor override | 20 ms | 90 ms | 180 ms | 1k overrides/min/cell | 99.95% |

## Failure Modes + Recovery

- Policy missing: route to fallback queue and emit degraded decision.
- Queue saturated: choose next eligible queue with same skill set.
- Stale agent presence: exclude stale agents and request presence refresh.

## Migration Notes

Vendor ACD rules must be imported as decision expressions with test fixtures. No opaque vendor route script is considered authoritative after migration.

## Cross-µservice Handoffs

- `presence` supplies agent availability.
- `workflow-engine` hosts routing templates.
- `policy-engine` evaluates Cedar permits.
- `audit-chain` seals route decisions.
- `notification` receives callback escalation events.
