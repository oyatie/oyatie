---
doc_class: ImplementationPlan
ip_id: IP-028-content-provider-catalog-federation
microservice: learning-management
related_adrs: [ADR-0244, ADR-0257, ADR-0263]
journey_id: J-LMS-28-provider-catalog-federation
status: proposed
date: 2026-05-20
owner: axis-learning-management
tenant_class: ["demo_trial", "paid"]
---

# IP-028: Content Provider Catalog Federation

## Context

This net-new slice federates provider and internal catalog entries while preserving tenant visibility, license entitlements, locale, duration, skill tags, and retirement status. It supports Elena Garcia replacing Cornerstone and Docebo catalogs while federating LinkedIn Learning Enterprise, 360Learning, and Workday Learning content in one governed search surface.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `learning_provider_catalog_item` | `catalog_item_id` | `uuid primary key` | Federated item. |
| `learning_provider_catalog_item` | `tenant_id` | `uuid not null` | Tenant partition. |
| `learning_provider_catalog_item` | `provider` | `text not null` | internal, cornerstone, workday, docebo, 360learning, linkedin. |
| `learning_provider_catalog_item` | `provider_content_ref` | `text not null` | External content ref. |
| `learning_provider_catalog_item` | `license_state` | `text not null` | active, exhausted, expired, unknown. |
| `learning_provider_catalog_item` | `skill_refs` | `text[] not null` | Skill coverage. |
| `learning_provider_catalog_item` | `retired_at` | `timestamptz` | Provider retirement timestamp. |

## API Endpoints

REST `POST /v1/learning-management/provider-catalog:sync`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-lms00028",
  "provider": "linkedin-learning",
  "provider_account_ref": "linkedin-learning:acct:ent-4421",
  "sync_mode": "delta",
  "changed_since": "2026-05-19T00:00:00Z"
}
```

gRPC `LearningProviderCatalogService.Sync(SyncProviderCatalogRequest)` returns `items_upserted`, `items_retired`, `license_warnings`, and `sync_event_id`.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `Service::"provider-sync"` | `learningManagement::SyncProviderCatalog` | `ProviderCatalog::*` | `tenant_id`, `provider`, `provider_account_ref` |
| `User::"learner"` | `learningManagement::SearchProviderCatalog` | `ProviderCatalogItem::*` | `tenant_id`, `audience`, `license_state`, `locale` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Cornerstone Course Catalog Item | `ProviderCatalogItem` | course code maps to provider content ref. |
| Workday Learning Digital Course | `ProviderCatalogItem` | learning object id and eligibility map to item. |
| Docebo Course | `ProviderCatalogItem` | branch and language map to visibility fields. |
| 360Learning Course | `ProviderCatalogItem` | author group maps to instructor metadata. |
| LinkedIn Learning Enterprise Asset | `ProviderCatalogItem` | asset urn, duration, and skill tags map directly. |

## Workflow Steps

1. `LoadProviderAccount` validates account entitlement and tenant scope.
2. `FetchProviderDelta` pulls changed course assets.
3. `NormalizeCatalogItem` maps title, locale, duration, skills, and license.
4. `EvaluateVisibilityPolicy` filters by tenant audience and region.
5. `UpsertFederatedCatalog` writes active or retired items.

Branches: expired license marks item unsearchable; missing locale defaults to tenant default; retired provider asset tombstones search result.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-LEARNING-PROVIDER-CATALOG-SYNCED` | `tenant_id`, `provider`, `items_upserted`, `items_retired` |
| `EVT-LEARNING-PROVIDER-CATALOG-LICENSE-BLOCKED` | `tenant_id`, `provider`, `provider_content_ref`, `license_state` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Delta sync 1k items | 1.2 s | 7 s | 18 s | 50 syncs/min/cell | 99.9% |
| Catalog search item read | 18 ms | 80 ms | 180 ms | 5k rps/cell | 99.95% |

## Failure Modes + Recovery

- Provider API rate limit: checkpoint cursor and resume after backoff.
- License entitlement missing: mark item blocked and keep prior active item until expiry.
- Mapping conflict: quarantine item and emit provider-specific error payload.

## Migration Notes

Catalog imports from Cornerstone, Workday Learning, Docebo, 360Learning, and LinkedIn Learning Enterprise must preserve provider refs so support can trace content back to source during dual-run.

## Cross-µservice Handoffs

- `content-provider-integration` executes provider API calls.
- `search` indexes active federated items.
- `skills-graph` consumes skill tags.
- `billing-entitlements` supplies provider license state.
- `audit-chain` seals sync and license-block events.
