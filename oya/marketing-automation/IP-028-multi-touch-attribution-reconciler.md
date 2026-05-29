---
doc_class: ImplementationPlan
ip_id: IP-028-multi-touch-attribution-reconciler
microservice: marketing-automation
related_adrs: [ADR-0244, ADR-0257, ADR-0263, ADR-0321]
journey_id: J-MA-28-revenue-attribution-close
status: proposed
date: 2026-05-20
owner: axis-marketing-automation
tenant_class_aware: true
---

# IP-028: Multi-Touch Attribution Reconciler

## Context

This net-new slice lets Marcus Chen reconcile pipeline influence without handing revenue truth to a marketing vendor. It displaces Marketo Bizible-style attribution, HubSpot campaign attribution, Mailchimp Premium revenue reports, Iterable conversion tracking, and Braze Currents exports.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_attribution_touch` | `touch_id` | `uuid primary key` | One normalized touch. |
| `marketing_attribution_touch` | `tenant_id` | `uuid not null` | Tenant partition. |
| `marketing_attribution_touch` | `campaign_id` | `uuid not null` | Marketing campaign. |
| `marketing_attribution_touch` | `subject_ref` | `text not null` | Hashed contact/account ref. |
| `marketing_attribution_touch` | `touch_kind` | `text not null` | `impression`, `open`, `click`, `reply`, `form_submit`, `meeting_booked`. |
| `marketing_attribution_touch` | `source_vendor` | `text` | Migration origin. |
| `marketing_attribution_touch` | `revenue_event_ref` | `text` | CRM/order event ref. |
| `marketing_attribution_touch` | `credit_basis_points` | `integer not null default 0` | 0-10000 allocation. |

## API Endpoints

REST `POST /v1/marketing-automation/attribution:reconcile`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-ma000001",
  "campaign_id": "018f8ad2-0e0f-7ad2-cmp00028",
  "model": "position_based_40_20_40",
  "revenue_event_ref": "crm:opportunity:opp_443:closed_won",
  "window_days": 90
}
```

gRPC `MarketingAttributionService.Reconcile(ReconcileAttributionRequest)` returns allocation rows and a reconciliation audit id.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"revops.manager"` | `marketingAutomation::ReconcileAttribution` | `MarketingCampaign::*` | `tenant_id`, `model`, `window_days`, `revenue_event_ref` |
| `Service::"crm-adapter"` | `marketingAutomation::AttachRevenueEvent` | `AttributionTouch::*` | `crm_object_ref`, `deal_stage`, `amount_class` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Marketo Program Success | `MarketingAttributionTouch` | success event maps to `touch_kind=form_submit` or configured kind. |
| HubSpot Campaign Attribution | `MarketingAttributionTouch` | campaign interaction maps to touch row with source ref. |
| Mailchimp Premium Revenue Report | `MarketingAttributionTouch` | order id maps to `revenue_event_ref`. |
| Iterable Conversion Event | `MarketingAttributionTouch` | conversion name maps to touch kind. |
| Braze Currents Event | `MarketingAttributionTouch` | event stream payload maps to touch row. |

## Workflow Steps

1. `LoadTouches` reads campaign touches inside attribution window.
2. `LoadRevenueEvent` asks CRM or commerce owner for closed-won/order proof.
3. `DeduplicateVendorEvents` collapses duplicate source ids.
4. `AllocateCredit` applies configured model.
5. `SealReconciliation` writes allocation and audit evidence.

Branches: no revenue proof returns dry-run only; duplicate vendor event keeps earliest HLC; restricted amount class returns aggregate-only output.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-ATTRIBUTION-RECONCILED` | `tenant_id`, `campaign_id`, `model`, `touch_count`, `revenue_event_ref` |
| `EVT-DATA-EGRESS` | Emitted when attribution export leaves marketing boundary. |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Reconcile 10k touches | 400 ms | 2.5 s | 5 s | 200 jobs/hour/cell | 99.9% |
| Read attribution summary | 50 ms | 220 ms | 450 ms | 800 rps/cell | 99.95% |

## Failure Modes + Recovery

- CRM revenue event unavailable: keep reconciliation pending and retry with backoff.
- Attribution model changed mid-run: version model and require new run.
- Duplicate source event flood: cap per subject and emit anomaly evidence.

## Migration Notes

Vendor attribution exports disagree on identity, time windows, and revenue ownership. Migration imports raw touch history and recalculates credit inside Oyatie instead of trusting vendor allocation percentages.

## Cross-µservice Handoffs

- `crm` supplies opportunity and account refs.
- `commerce` supplies order events when applicable.
- `ontology` links campaign, account, and revenue nodes.
- `finops` consumes cost-per-attributed-pipeline aggregates.
- `audit-chain` seals reconciliation results.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-028-multi-touch-attribution-reconciler.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/marketing-automation/IP-028-multi-touch-attribution-reconciler.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/marketing-automation/IP-028-multi-touch-attribution-reconciler.md` matched [`attribution`, `finops`, `cost`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/marketing-automation/IP-028-multi-touch-attribution-reconciler.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/capacity-model.md`, `microservices/marketing-automation/compliance.md`, `microservices/marketing-automation/ARCHITECTURE.md`].
