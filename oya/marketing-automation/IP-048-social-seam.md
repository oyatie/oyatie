---
doc_class: ImplementationPlan
ip_id: IP-048-social-seam
microservice: marketing-automation
related_adrs: [ADR-0244, ADR-0263, ADR-0321, ADR-0328]
bounded_context: social-seam
journey_id: J-MA-48-social-media-publishing-seam
status: proposed
date: 2026-05-21
owner: axis-marketing-automation
tenant_class_aware: true
delegation_destination: social µservice
open_question_settled: Q-011
---

# IP-048: Social Seam

## Context

HubSpot Social + Marketo Engage Social + Mailchimp Social Posting are universal in counterparts. Oyatie has a dedicated social µservice; this seam declares the marketing-automation ↔ social contract: marketing-automation owns campaign references to social posts and consumes engagement metrics; social µservice owns publishing + monitoring + listening adapters.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_social_asset_ref` | `ref_id` | `uuid primary key` | Reference id. |
| `marketing_social_asset_ref` | `tenant_id` | `uuid not null` | Tenant. |
| `marketing_social_asset_ref` | `campaign_id` | `uuid not null` | FK to marketing_campaign. |
| `marketing_social_asset_ref` | `social_asset_id` | `uuid not null` | FK to social.social_asset. |
| `marketing_social_asset_ref` | `social_platform` | `text not null` | linkedin / twitter / facebook / instagram / tiktok. |
| `marketing_social_asset_ref` | `scheduled_at` | `tstzrange` | If pre-scheduled. |
| `marketing_social_asset_ref` | `created_at_hlc` | `hlc not null` | HLC. |

## API Endpoints

REST `POST /v1/marketing-automation/social/asset-refs` registers a social asset reference under a campaign.

REST `GET /v1/marketing-automation/social/engagement?campaign_id={id}` aggregates engagement metrics across referenced social assets.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"marketing.ops"` | `marketingAutomation::ReferenceSocialAsset` | `MarketingSocialAssetRef::*` | `tenant_class` |

## Workflow Steps

1. `ValidateSocialAsset` confirms social.social_asset_id resolves via social contract.
2. `Authorize` calls Cedar.
3. `PersistRef` writes row.
4. On engagement event from social, ingest as `attribution.touch`.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-SOCIAL-ASSET-REFERENCED` | `ref_id`, `campaign_id`, `social_asset_id`, `social_platform` |
| `EVT-MARKETING-SOCIAL-POST-SCHEDULED` | `ref_id`, `scheduled_at` |

## Cross-µservice Handoffs

- `social` owns publishing + monitoring adapters.
- `marketing-calendar` displays social entries.
- `attribution` consumes social engagement events.
- `audit-chain` seals events.

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/marketing-automation/IP-048-social-seam.md` matched [`attribution`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/marketing-automation/IP-048-social-seam.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/capacity-model.md`, `microservices/marketing-automation/compliance.md`, `microservices/marketing-automation/ARCHITECTURE.md`].
