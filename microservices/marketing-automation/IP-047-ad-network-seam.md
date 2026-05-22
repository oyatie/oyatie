---
doc_class: ImplementationPlan
ip_id: IP-047-ad-network-seam
microservice: marketing-automation
related_adrs: [ADR-0244, ADR-0263, ADR-0314, ADR-0321, ADR-0328]
bounded_context: ad-network-seam
journey_id: J-MA-47-ad-network-integration-seam
status: proposed
date: 2026-05-21
owner: axis-marketing-automation
tenant_class_aware: true
delegation_destination: advertising-platform µservice
open_question_settled: Q-022
---

# IP-047: Ad Network Seam

## Context

HubSpot Ads + Marketo Ad Bridge + Mailchimp Ads expose ad-network integrations for Google Ads + LinkedIn Ads + Facebook Ads + TikTok Ads. The µservice tree's coherence audit identified ad-network as a delegated capability per Q-022. This seam declares the integration contract with the advertising-platform µservice — marketing-automation does not own ad-platform adapters; it owns audience sync via marketplace audience-license (ADR-0314) and ingests ad events into attribution.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_ad_network_audience_sync` | `sync_id` | `uuid primary key` | Audience sync row. |
| `marketing_ad_network_audience_sync` | `tenant_id` | `uuid not null` | Tenant. |
| `marketing_ad_network_audience_sync` | `target_segment_id` | `uuid not null` | FK to marketing_segment. |
| `marketing_ad_network_audience_sync` | `ad_provider` | `text not null` | google_ads / linkedin_ads / facebook_ads / tiktok_ads. |
| `marketing_ad_network_audience_sync` | `marketplace_deal_set_id` | `uuid not null` | ADR-0314 deal set ref. |
| `marketing_ad_network_audience_sync` | `last_sync_hlc` | `hlc` | HLC. |
| `marketing_ad_network_audience_sync` | `last_member_count` | `bigint` | Last synced member count. |

## API Endpoints

REST `POST /v1/marketing-automation/ad-network/audience-syncs`:

```json
{
  "tenant_id": "...",
  "target_segment_id": "...",
  "ad_provider": "linkedin_ads",
  "marketplace_deal_set_id": "..."
}
```

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"marketing.ops"` | `marketingAutomation::SyncAudienceToAdNetwork` | `MarketingAdNetworkAudienceSync::*` | `tenant_class`, `marketplace_deal_set_authorized`, `pack_overlay` |

## Workflow Steps

1. `ValidateDealSet` confirms marketplace DealSet authorizes the audience-license per ADR-0314.
2. `Authorize` calls Cedar.
3. `PostToAdvertisingPlatform` posts the audience descriptor + provider to advertising-platform µservice over gRPC.
4. `RecordSync` writes row.
5. `EmitSync` emits `EVT-MARKETING-AD-NETWORK-AUDIENCE-SYNCED`.
6. Ad events from advertising-platform feed back into `attribution.touches` as engagement events.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-AD-NETWORK-AUDIENCE-SYNCED` | `sync_id`, `ad_provider`, `marketplace_deal_set_id`, `tenant_class` |
| `EVT-MARKETING-AD-NETWORK-EVENT-INGESTED` | `sync_id`, `ad_event_kind`, `subject_hash`, `recorded_at_hlc` |

## SLO Targets

Delegated to advertising-platform for actual ad-platform latency.

## Migration Notes

HubSpot Ads + Marketo Ad Bridge + Mailchimp Ads vendor-credential bindings are reset at migration; marketing-automation provides the sync contract, advertising-platform handles per-network OAuth + credential rotation.

## Cross-µservice Handoffs

- `advertising-platform` owns per-ad-network adapter (Google Ads + LinkedIn Ads + Facebook Ads + TikTok Ads).
- `marketplace` settles audience-license DealSet per ADR-0314.
- `attribution` consumes ad events as touch events.
- `segment` (this µservice) provides target audience.
- `audit-chain` seals events.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-047-ad-network-seam.md` matched [`SLO`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/marketing-automation/IP-047-ad-network-seam.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/marketing-automation/IP-047-ad-network-seam.md` matched [`attribution`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/marketing-automation/IP-047-ad-network-seam.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/capacity-model.md`, `microservices/marketing-automation/compliance.md`, `microservices/marketing-automation/ARCHITECTURE.md`].
