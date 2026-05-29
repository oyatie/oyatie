---
doc_class: ImplementationPlan
ip_id: IP-003-ontology-projection
microservice: contact-center
related_adrs: [ADR-0244, ADR-0257, ADR-0263, ADR-0321]
journey_id: J-CC-03-vendor-routing-import
status: proposed
date: 2026-05-20
owner: axis-contact-center
availability: paid
---

# IP-003: Contact Center Ontology Projection

## Context

This slice maps vendor routing and interaction records into ontology primitives before implementation begins. Diana Alvarez needs multi-client migration from Genesys, NICE CXone, Five9, Talkdesk, and AWS without carrying vendor-specific queue, contact flow, and recording labels into the service.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `contact_center_projection` | `projection_id` | `uuid primary key` | One source object projection. |
| `contact_center_projection` | `tenant_id` | `uuid not null` | Tenant partition. |
| `contact_center_projection` | `source_vendor` | `text not null` | `genesys`, `nice_cxone`, `five9`, `talkdesk`, `aws_connect`. |
| `contact_center_projection` | `source_object_type` | `text not null` | Queue, interaction, flow, agent, recording. |
| `contact_center_projection` | `oyatie_object_type` | `text not null` | Ontology target. |
| `contact_center_projection` | `field_delta` | `jsonb not null` | Deterministic mapping. |
| `contact_center_projection` | `projection_hash` | `bytea not null` | Replay guard. |

## API Endpoints

REST `POST /v1/contact-center/projections/preview`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-cc000001",
  "source_vendor": "aws_connect",
  "source_object_type": "contact_flow",
  "source_object_id": "cf-ivr-main",
  "target_object_type": "ContactRoutingFlow"
}
```

gRPC `ContactCenterProjectionService.PreviewProjection(PreviewContactProjectionRequest)` returns `field_delta`, `rejected_fields[]`, and `ontology_write_plan`.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"migration.operator"` | `contactCenter::PreviewProjection` | `VendorContactObject::*` | `tenant_id`, `source_vendor`, `source_object_type` |
| `Service::"projection-worker"` | `ontology::WriteProjection` | `ContactCenterObject::*` | `projection_hash`, `data_classes`, `pack_id` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Genesys Queue | `ContactQueue` | `queueId -> source_ref.id`; division maps through scope. |
| NICE CXone Skill | `ContactQueue` | skill id maps to queue plus skill tags. |
| Five9 Campaign/List | `OutboundDialerPlan` | campaign id maps to plan; list id maps to audience source ref. |
| Talkdesk Ring Group | `ContactQueue` | ring-group id maps to queue. |
| AWS Contact Flow | `ContactRoutingFlow` | flow ARN maps to source ref; blocks become workflow nodes. |

## Workflow Steps

1. `FetchVendorSchema` reads metadata and no recording payload.
2. `MapContactFields` produces queue, agent, interaction, or flow deltas.
3. `ClassifyData` labels call session, transcript, and recording consent classes.
4. `PreviewOntologyWrite` validates node and edge types.
5. `CommitProjection` writes projection and ontology request.

Branches: recording payload fields are rejected from projection; missing queue owner returns `422`; flow loops require workflow review.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-CONTACT-CENTER-PROJECTION-PREVIEWED` | `tenant_id`, `source_vendor`, `source_object_id`, `projection_hash` |
| `EVT-CONTACT-CENTER-PROJECTION-COMMITTED` | `projection_id`, `oyatie_object_type`, `ontology_write_id` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Projection preview | 130 ms | 700 ms | 1.6 s | 250 previews/min/cell | 99.9% |
| Projection commit | 70 ms | 320 ms | 700 ms | 100 commits/min/cell | 99.95% |

## Failure Modes + Recovery

- Vendor schema drift: freeze commit and persist rejected mapping.
- Ontology write rejected: keep projection pending and retry idempotently.
- Flow graph cycle: route to workflow review with cycle nodes named.

## Migration Notes

Genesys and AWS model flows as graphs, while NICE CXone, Five9, and Talkdesk split skills, queues, and campaigns differently. Import must map behavior and resources, not product labels.

## Cross-µservice Handoffs

- `ontology` owns projected objects.
- `workflow-engine` receives routing flow nodes.
- `data-boundary` classifies interaction and recording fields.
- `audit-chain` seals projection events.
- `telephony-adapter` supplies vendor metadata.
