---
doc_class: ImplementationPlan
ip_id: IP-003-ontology-projection
microservice: marketing-automation
related_adrs: [ADR-0244, ADR-0257, ADR-0263, ADR-0272, ADR-0321]
journey_id: J-MA-03-vendor-audience-import
status: proposed
date: 2026-05-20
owner: axis-marketing-automation
tenant_class_aware: true
---

# IP-003: Marketing Automation Ontology Projection

## Context

This slice lets Diana Alvarez import vendor campaign objects for several agency clients while preserving Oyatie ontology names. It subsumes Marketo Smart Lists, HubSpot Lists, Mailchimp Premium Segments, Iterable Catalog Collections, and Braze Segments into explicit `MarketingAudience`, `MarketingSegmentRule`, and `MarketingConsentSnapshot` objects.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_object_projection` | `projection_id` | `uuid primary key` | One row per source object projection. |
| `marketing_object_projection` | `tenant_id` | `uuid not null` | Tenant partition. |
| `marketing_object_projection` | `source_vendor` | `text not null` | `marketo`, `hubspot`, `mailchimp_premium`, `iterable`, `braze`. |
| `marketing_object_projection` | `source_object_type` | `text not null` | `smart_list`, `active_list`, `segment`, `collection`, `braze_segment`. |
| `marketing_object_projection` | `oyatie_object_type` | `text not null` | Ontology node type. |
| `marketing_object_projection` | `field_delta` | `jsonb not null` | Normalized source-to-target mapping. |
| `marketing_object_projection` | `projection_hash` | `bytea not null` | Idempotency and replay guard. |

## API Endpoints

REST `POST /v1/marketing-automation/projections/preview`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-ma000001",
  "source_vendor": "marketo",
  "source_object_type": "smart_list",
  "source_object_id": "SL-4412",
  "target_object_type": "MarketingSegmentRule",
  "sample_limit": 1000
}
```

gRPC `MarketingProjectionService.PreviewProjection(PreviewProjectionRequest)` returns `field_delta`, `rejected_fields[]`, and `ontology_write_plan`.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"agency.operator"` | `marketingAutomation::PreviewProjection` | `VendorMarketingObject::*` | `tenant_id`, `client_tenant_id`, `source_vendor`, `sample_limit` |
| `Service::"projection-worker"` | `ontology::WriteProjection` | `MarketingAudience::*` | `projection_hash`, `data_classes`, `purpose` |
| `User::"auditor"` | `marketingAutomation::ReadProjectionDelta` | `MarketingProjection::*` | `ticket_id`, `read_reason` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Marketo Smart List | `MarketingSegmentRule` | `filters[] -> predicate_tree`; `smartListId -> source_ref.id`. |
| HubSpot Active List | `MarketingAudience` | `listId -> source_ref.id`; dynamic criteria become `predicate_tree`. |
| Mailchimp Premium Segment | `MarketingAudience` | `conditions[] -> predicate_tree`; merge fields become `profile_trait_ref`. |
| Iterable Catalog Collection | `MarketingCatalogAudience` | `collectionId -> source_ref.id`; item criteria become catalog predicates. |
| Braze Segment | `MarketingSegmentRule` | `filters -> predicate_tree`; app-group id becomes scope ref. |

## Workflow Steps

1. `FetchSourceMetadata` reads only schema and sample counts, not full PII.
2. `MapFields` produces deterministic deltas.
3. `ClassifyData` tags every field as campaign profile, consent signal, or attribution event.
4. `PreviewOntologyWrite` validates target node and edge types.
5. `CommitProjection` writes projection rows and requests ontology write.

Branches: unknown source field enters `rejected_fields`; high-cardinality sample enters async preview; data-class mismatch refuses commit.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-PROJECTION-PREVIEWED` | `tenant_id`, `source_vendor`, `source_object_id`, `projection_hash` |
| `EVT-MARKETING-PROJECTION-COMMITTED` | `projection_id`, `oyatie_object_type`, `field_delta_hash`, `ontology_write_id` |
| `EVT-DATA-EGRESS` | Emitted if preview requires source-system sample export. |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Projection preview metadata-only | 120 ms | 600 ms | 1.5 s | 300 previews/min/cell | 99.9% |
| Projection commit | 80 ms | 350 ms | 800 ms | 100 commits/min/cell | 99.95% |

## Failure Modes + Recovery

- Vendor schema drift: freeze commit, persist preview with `schema_drift=true`, and require a new field mapping.
- Ontology write rejected: keep projection row in `pending_ontology_write` and retry with same hash.
- Sample export denied: continue metadata-only preview and show missing-confidence warning.

## Migration Notes

Marketo and HubSpot dynamic lists often depend on behavior events that Mailchimp Premium and Braze model differently. Migration must map predicates, not vendor labels, and must store rejected predicates for manual completion before campaign launch.

## Cross-µservice Handoffs

- `ontology` owns node and edge creation.
- `data-boundary` classifies projected fields.
- `consent` supplies purpose and lawful-basis references.
- `audit-chain` seals projection events.
- `data-pipeline` can later backfill large historical event predicates without changing this IP.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-003-ontology-projection.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/marketing-automation/IP-003-ontology-projection.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/marketing-automation/IP-003-ontology-projection.md` matched [`attribution`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/marketing-automation/IP-003-ontology-projection.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/capacity-model.md`, `microservices/marketing-automation/compliance.md`, `microservices/marketing-automation/ARCHITECTURE.md`].
