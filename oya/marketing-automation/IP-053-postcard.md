---
doc_class: ImplementationPlan
ip_id: IP-053-postcard
microservice: marketing-automation
related_adrs: [ADR-0244, ADR-0251, ADR-0263, ADR-0321, ADR-0328]
bounded_context: postcard
journey_id: J-MA-53-direct-mail-postcard
status: proposed
date: 2026-05-21
owner: axis-marketing-automation
tenant_class_aware: true
---

# IP-053: Postcard (direct mail)

## Context

Mailchimp Postcard is a unique counterpart capability for physical direct-mail. HubSpot + Marketo handle via partner integrations. This slice adds physical-mail through a postal-provider adapter (Lob / PostGrid / etc.) without requiring tenant-side custom integration.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_postcard` | `postcard_id` | `uuid primary key` | Postcard id. |
| `marketing_postcard` | `tenant_id` | `uuid not null` | Tenant. |
| `marketing_postcard` | `front_template_ref` | `uuid not null` | FK to marketing_asset. |
| `marketing_postcard` | `back_template_ref` | `uuid not null` | FK to marketing_asset. |
| `marketing_postcard` | `addressed_audience_descriptor` | `jsonb not null` | Subject hashes with verified postal addresses. |
| `marketing_postcard` | `postal_provider` | `text not null` | lob / postgrid / etc. |
| `marketing_postcard_send` | `send_id` | `uuid primary key` | Per-recipient send. |
| `marketing_postcard_send` | `postcard_id` | `uuid not null` | FK. |
| `marketing_postcard_send` | `subject_hash` | `text not null` | Subject. |
| `marketing_postcard_send` | `postal_provider_send_id` | `text` | Provider id. |
| `marketing_postcard_send` | `status` | `text not null` | composed / addressed / sent / delivered / returned_to_sender. |
| `marketing_postcard_send` | `delivered_at_hlc` | `hlc` | HLC. |

## API Endpoints

REST `POST /v1/marketing-automation/postcards` creates.

REST `POST /v1/marketing-automation/postcards/{postcard_id}:send` triggers per-recipient send via postal provider.

REST `POST /webhooks/postal/{provider}/delivery-update` records delivery status from provider.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"marketing.ops"` | `marketingAutomation::SendPostcard` | `MarketingPostcard::*` | `tenant_class`, `recipient_count`, `postal_provider_credentials_resolved` |

## Workflow Steps

1. `ValidateTemplates` (front + back).
2. `ValidatePostalAddresses` per recipient (USPS API or provider-side validation).
3. `Authorize` Cedar.
4. `EnqueueSend` per recipient.
5. `DispatchToProvider` calls postal provider API.
6. `RecordProviderSendId` writes.
7. On delivery webhook, update status + emit event.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-POSTCARD-COMPOSED` | `postcard_id`, `tenant_class` |
| `EVT-MARKETING-POSTCARD-SENT` | `send_id`, `subject_hash`, `postal_provider`, `postal_provider_send_id` |
| `EVT-MARKETING-POSTCARD-DELIVERED` | `send_id`, `delivered_at_hlc` |
| `EVT-MARKETING-POSTCARD-RETURNED` | `send_id`, `return_reason` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Compose postcard | 80 ms | 300 ms | 700 ms | 50 rps/cell | 99.9% |
| Per-recipient send | 200 ms | 800 ms | 2 s | 500 rps/cell | 99.9% |

## Cross-µservice Handoffs

- Postal-provider adapter (external; mediated via adapter trait).
- `audit-chain` seals events.
- `finops` consumes per_usage `postcards_sent` meter.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-053-postcard.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/marketing-automation/IP-053-postcard.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/marketing-automation/IP-053-postcard.md` matched [`finops`, `per_usage`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/marketing-automation/IP-053-postcard.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/capacity-model.md`, `microservices/marketing-automation/compliance.md`, `microservices/marketing-automation/ARCHITECTURE.md`].
