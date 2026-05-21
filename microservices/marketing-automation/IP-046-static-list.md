---
doc_class: ImplementationPlan
ip_id: IP-046-static-list
microservice: marketing-automation
related_adrs: [ADR-0244, ADR-0263, ADR-0321, ADR-0328]
bounded_context: static-list
journey_id: J-MA-46-explicit-membership-list
status: proposed
date: 2026-05-21
owner: axis-marketing-automation
tenant_class_aware: true
---

# IP-046: Static List

## Context

Static lists hold explicit-membership rows (subject_hash + provenance). Distinct from segment (predicate-derived dynamic membership). HubSpot Static List + Marketo Static List + Mailchimp Tag are direct counterparts. Used for hand-curated lists, manual imports, and post-event invitee lists.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_static_list` | `list_id` | `uuid primary key` | Static list id. |
| `marketing_static_list` | `tenant_id` | `uuid not null` | Tenant. |
| `marketing_static_list` | `name` | `text not null` | Unique per tenant. |
| `marketing_static_list` | `purpose` | `text not null` | Purpose tag. |
| `marketing_static_list` | `archived` | `boolean not null default false` | Archival flag. |
| `marketing_static_list_member` | `member_id` | `uuid primary key` | Membership row. |
| `marketing_static_list_member` | `list_id` | `uuid not null` | FK. |
| `marketing_static_list_member` | `subject_hash` | `text not null` | Subject ref. |
| `marketing_static_list_member` | `source_vendor` | `text` | manual / hubspot / marketo / mailchimp / csv_import. |
| `marketing_static_list_member` | `source_object_id` | `text` | Source row id. |
| `marketing_static_list_member` | `source_timestamp` | `timestamptz` | Source timestamp. |
| `marketing_static_list_member` | `import_batch_id` | `uuid` | For import dry-runs. |
| `marketing_static_list_member` | `added_at_hlc` | `hlc not null` | HLC. |

## API Endpoints

REST `POST /v1/marketing-automation/static-lists`.

REST `POST /v1/marketing-automation/static-lists/{list_id}/members` adds members.

REST `POST /v1/marketing-automation/static-lists/{list_id}:import` accepts a CSV or vendor export bundle.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"marketing.ops"` | `marketingAutomation::CreateStaticList` | `MarketingStaticList::*` | `tenant_class` |
| `Service::"import-worker"` | `marketingAutomation::ImportMembers` | `MarketingStaticList::list_id` | `dry_run`, `import_batch_id` |

## Workflow Steps

1. `ValidateListNameUnique`.
2. `Authorize` calls Cedar.
3. On import, `DryRun` produces row-level rejection report (duplicate subject_hash, malformed input).
4. `WriteMembership` on success.
5. `EmitChange` emits events.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-STATIC-LIST-CREATED` | `list_id`, `name`, `purpose`, `tenant_class` |
| `EVT-MARKETING-STATIC-LIST-MEMBER-ADDED` | `list_id`, `member_id`, `subject_hash`, `source_vendor` |
| `EVT-MARKETING-STATIC-LIST-IMPORT-COMPLETED` | `list_id`, `import_batch_id`, `accepted_count`, `rejected_count` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Add member | 25 ms | 100 ms | 250 ms | 5000 rps/cell | 99.99% |
| Import batch (10k rows) | 5 s | 20 s | 60 s | 10 imports/hour/cell | 99.9% |

## Failure Modes + Recovery

- Duplicate subject_hash in import → reject row with reason `duplicate_subject_in_list`; preserve in rejection report.
- Import worker crash mid-batch → resume from `import_batch_id` checkpoint.

## Migration Notes

HubSpot + Marketo Static List + Mailchimp Tag export as CSV with subject identifiers; preserved with original timestamp + source vendor flag.

## Cross-µservice Handoffs

- `audit-chain` seals every change.
- `workflow-canvas` triggers on list-membership-change (entry trigger).

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-046-static-list.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/marketing-automation/IP-046-static-list.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].
