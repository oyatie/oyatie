---
doc_class: ImplementationPlan
ip_id: IP-026-real-time-segment-materializer
microservice: marketing-automation
related_adrs: [ADR-0244, ADR-0257, ADR-0263, ADR-0321]
journey_id: J-MA-26-real-time-buying-committee-segment
status: proposed
date: 2026-05-20
owner: axis-marketing-automation
tenant_class_aware: true
---

# IP-026: Real-Time Segment Materializer

## Context

This net-new slice covers real-time segment materialization not enumerated in the first 25 IPs. Marcus Chen needs a buying-committee segment to update from product events in under one second, displacing Marketo Smart Lists, HubSpot Active Lists, Mailchimp Premium advanced segmentation, Iterable segmentation, and Braze segments.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_segment_materialization` | `segment_id` | `uuid primary key` | Oyatie segment id. |
| `marketing_segment_materialization` | `tenant_id` | `uuid not null` | Tenant partition. |
| `marketing_segment_materialization` | `predicate_tree` | `jsonb not null` | Normalized rules from ontology. |
| `marketing_segment_materialization` | `member_count` | `bigint not null default 0` | Last committed count. |
| `marketing_segment_materialization` | `freshness_floor_ms` | `integer not null` | Default 750 ms for real-time journeys. |
| `marketing_segment_materialization` | `last_event_cursor` | `text not null` | Product-event stream cursor. |

## API Endpoints

REST `POST /v1/marketing-automation/segments/{segment_id}:materialize`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-ma000001",
  "predicate_tree": {"all": [{"trait": "account.arr", "gte": 50000}, {"event": "trial.invited", "within_days": 14}]},
  "freshness_floor_ms": 750,
  "dry_run": false
}
```

gRPC `MarketingSegmentService.Materialize(MaterializeSegmentRequest)` streams `SegmentDelta` responses for incremental updates.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"marketing.ops"` | `marketingAutomation::MaterializeSegment` | `MarketingSegment::*` | `tenant_id`, `predicate_fields`, `freshness_floor_ms` |
| `Service::"event-consumer"` | `marketingAutomation::ApplySegmentDelta` | `MarketingSegment::*` | `event_cursor`, `delta_count`, `data_classes` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Marketo Smart List | `MarketingSegment` | Smart-list filters become `predicate_tree`. |
| HubSpot Active List | `MarketingSegment` | Active criteria become incremental predicates. |
| Mailchimp Premium Segment | `MarketingSegment` | Static member ids become initial snapshot only. |
| Iterable Segment | `MarketingSegment` | User attributes map to ontology profile traits. |
| Braze Segment | `MarketingSegment` | Behavior filters map to event predicates. |

## Workflow Steps

1. `CompilePredicateTree` verifies every trait and event exists in ontology.
2. `AuthorizeTraits` denies use of restricted traits.
3. `BuildInitialSnapshot` computes first member set.
4. `SubscribeEventCursor` starts delta application.
5. `PublishFreshnessMetric` emits lag and member count.

Decision branches: restricted trait denies; high-cardinality predicate moves to async build; missing event cursor uses snapshot-only mode.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-SEGMENT-MATERIALIZED` | `tenant_id`, `segment_id`, `member_count`, `freshness_floor_ms` |
| `EVT-MARKETING-SEGMENT-DELTA-APPLIED` | `segment_id`, `event_cursor`, `added`, `removed` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Apply segment delta | 40 ms | 250 ms | 750 ms | 25k events/s/cell | 99.95% |
| Initial materialization | 1.5 s | 20 s | 60 s | 100 builds/hour/cell | 99.9% |

## Failure Modes + Recovery

- Event stream lag above floor: mark segment `stale`, block journey launch, and replay from `last_event_cursor`.
- Predicate references retired field: fail compile and return ontology replacement candidates.
- Member-count explosion: trip admission control and require manager approval.

## Migration Notes

Vendor static and dynamic lists must be imported with source timestamps. Mailchimp Premium segments frequently lack real-time event semantics, so first import is snapshot-only until product events catch up.

## Cross-µservice Handoffs

- `eventing` supplies product and engagement streams.
- `ontology` validates traits and event types.
- `workflow-engine` consumes segment freshness before launch.
- `audit-chain` seals materialization and delta events.
- `finops` receives segment build CPU and storage dimensions.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-026-real-time-segment-materializer.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/marketing-automation/IP-026-real-time-segment-materializer.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/marketing-automation/IP-026-real-time-segment-materializer.md` matched [`finops`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/marketing-automation/IP-026-real-time-segment-materializer.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/capacity-model.md`, `microservices/marketing-automation/compliance.md`, `microservices/marketing-automation/ARCHITECTURE.md`].
