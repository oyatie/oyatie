---
doc_class: ImplementationPlan
ip_id: IP-051-customer-analytics
microservice: marketing-automation
related_adrs: [ADR-0244, ADR-0263, ADR-0321, ADR-0328]
bounded_context: customer-analytics
journey_id: J-MA-51-tenant-facing-marketing-reports
status: proposed
date: 2026-05-21
owner: axis-marketing-automation
tenant_class_aware: true
---

# IP-051: Customer-facing Analytics

## Context

HubSpot Marketing Analytics + Marketo Performance Insights + Mailchimp Reports + Recipient Activity are tenant-visible report surfaces distinct from operator dashboards under `dashboards/`. This slice owns the tenant-facing report aggregate; analytics µservice provides the underlying data substrate. Differentiator is data-class boundary enforcement on export — counterparts allow unrestricted export.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_customer_report` | `report_id` | `uuid primary key` | Report definition. |
| `marketing_customer_report` | `tenant_id` | `uuid not null` | Tenant. |
| `marketing_customer_report` | `report_kind` | `text not null` | campaign_performance / journey_conversion / attribution_by_source / email_engagement / ab_test_results / lifecycle_funnel / abm_account_engagement / deliverability_health. |
| `marketing_customer_report` | `scope` | `jsonb not null` | Filter (campaign_id / segment_id / date range / channel). |
| `marketing_customer_report` | `schedule` | `text` | Optional cron / RRule. |
| `marketing_customer_report` | `export_destinations` | `text[]` | Optional webhook subscriptions to deliver report. |
| `marketing_customer_report` | `data_class_boundary` | `text[] not null` | Allowed data classes for export (INTERNAL_ONLY / PII_QUASI / etc.). |
| `marketing_customer_report` | `created_at_hlc` | `hlc not null` | HLC. |
| `marketing_customer_report_run` | `run_id` | `uuid primary key` | Per-run row. |
| `marketing_customer_report_run` | `report_id` | `uuid not null` | FK. |
| `marketing_customer_report_run` | `ran_at_hlc` | `hlc not null` | HLC. |
| `marketing_customer_report_run` | `row_count` | `bigint not null` | Result row count. |
| `marketing_customer_report_run` | `result_storage_ref` | `text not null` | drive reference. |

## API Endpoints

REST `POST /v1/marketing-automation/customer-analytics/reports`.

REST `POST /v1/marketing-automation/customer-analytics/reports/{report_id}:run` runs once.

REST `POST /v1/marketing-automation/customer-analytics/reports/{report_id}/runs/{run_id}:export?format=csv|xlsx|pdf`.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"marketing.ops"` | `marketingAutomation::DefineCustomerReport` | `MarketingCustomerReport::*` | `tenant_class` |
| `User::"marketing.ops"` | `marketingAutomation::ExportCustomerReport` | `MarketingCustomerReportRun::run_id` | `requested_format`, `requested_data_classes`, `report_data_class_boundary` |

Cedar denies export when `requested_data_classes ⊄ data_class_boundary`.

## Workflow Steps

1. `ValidateReportKind` against canonical kind registry.
2. `Authorize` calls Cedar.
3. `PersistReport` writes definition.
4. On run, `LoadDataFromAnalytics` calls analytics µservice with scope filter.
5. `EnforceDataClassBoundary` strips disallowed columns.
6. `StoreResult` writes to drive.
7. `EmitRun` emits event.
8. On export, `EnforceExportBoundary` re-checks; deliver via destination.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-CUSTOMER-REPORT-DEFINED` | `report_id`, `report_kind`, `data_class_boundary` |
| `EVT-MARKETING-CUSTOMER-REPORT-RAN` | `report_id`, `run_id`, `row_count` |
| `EVT-MARKETING-CUSTOMER-REPORT-EXPORTED` | `run_id`, `format`, `destination`, `cedar_decision_id` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Run report (small) | 500 ms | 3 s | 10 s | 100 reports/hour/cell | 99.9% |
| Run report (large, 1M rows) | 30 s | 120 s | 300 s | 10 jobs/hour/cell | 99.9% |
| Export report | 200 ms | 1 s | 3 s | 50 exports/hour/cell | 99.9% |

## Migration Notes

HubSpot Marketing Analytics + Marketo Performance Insights + Mailchimp Reports vendor-specific report definitions migrate as Oyatie `marketing_customer_report` definitions; report kind mapping in migration playbooks.

## Cross-µservice Handoffs

- `analytics` µservice provides data substrate.
- `data-boundary` enforces data-class export controls.
- `drive` stores report results.
- `webhook-subscription` delivers reports to external destinations.
- `audit-chain` seals every run + export.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-051-customer-analytics.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/marketing-automation/IP-051-customer-analytics.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/marketing-automation/IP-051-customer-analytics.md` matched [`attribution`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/marketing-automation/IP-051-customer-analytics.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/capacity-model.md`, `microservices/marketing-automation/compliance.md`, `microservices/marketing-automation/ARCHITECTURE.md`].
