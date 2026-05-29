---
doc_class: ImplementationPlan
ip_id: IP-001-tenant-scope-kernel
microservice: marketing-automation
related_adrs: [ADR-0242, ADR-0243, ADR-0244, ADR-0257, ADR-0263, ADR-0321]
journey_id: J-MA-01-consented-campaign-launch
status: proposed
date: 2026-05-20
owner: axis-marketing-automation
tenant_class_aware: true
---

# IP-001: Marketing Automation Tenant Scope Kernel

## Context

This slice creates the tenant-owned scope boundary for Marcus Chen launching a consented campaign without recreating Marketo workspace, HubSpot Business Unit, Mailchimp Premium audience, Iterable project, or Braze app-group isolation. The intern-buildable target is one kernel table plus one scope command that every campaign, segment, suppression, and attribution primitive must reference before any outbound message can be planned.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `marketing_tenant_scope` | `scope_id` | `uuid primary key` | Stable Oyatie scope identifier, not vendor workspace id. |
| `marketing_tenant_scope` | `tenant_id` | `uuid not null` | FK to tenancy; partition key. |
| `marketing_tenant_scope` | `audience_type` | `text not null` | `ENTERPRISE_MARKETING_OPERATOR`, `AGENCY_OPERATOR`, or `TENANT_ADMIN`. |
| `marketing_tenant_scope` | `default_consent_purpose` | `text not null` | Example: `product_marketing`, `transactional_notice`. |
| `marketing_tenant_scope` | `source_vendor_refs` | `jsonb not null default '{}'` | Marketo workspace id, HubSpot business unit id, Mailchimp audience id, Iterable project id, Braze app group id. |
| `marketing_tenant_scope` | `residency_pack_id` | `text not null` | GDPR, CASL, CAN-SPAM, KR-PIPA overlay. |
| `marketing_tenant_scope` | `created_at` | `timestamptz not null` | HLC stamped by tenancy boundary. |

## API Endpoints

REST `POST /v1/marketing-automation/tenant-scopes`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-ma000001",
  "audience_type": "ENTERPRISE_MARKETING_OPERATOR",
  "default_consent_purpose": "product_marketing",
  "source_vendor_refs": {
    "marketo_workspace_id": "ws-178",
    "hubspot_business_unit_id": "bu-42",
    "mailchimp_audience_id": "aud_83",
    "iterable_project_id": "it_proj_19",
    "braze_app_group_id": "ag_71"
  },
  "residency_pack_id": "GDPR"
}
```

gRPC `MarketingScopeService.CreateTenantScope(CreateTenantScopeRequest) returns (TenantScope)` mirrors the REST body and adds `request_context.trace_id`.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"marketing.ops"` | `marketingAutomation::CreateTenantScope` | `MarketingTenantScope::scope_id` | `tenant_id`, `purpose`, `source_vendor_refs`, `residency_pack_id` |
| `Service::"migration-worker"` | `marketingAutomation::BindVendorScope` | `MarketingTenantScope::scope_id` | `migration_batch_id`, `vendor`, `dry_run=true` |
| `User::"auditor"` | `marketingAutomation::ReadTenantScope` | `MarketingTenantScope::scope_id` | `read_reason`, `ticket_id`, `pack_id` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Marketo Workspace | `MarketingTenantScope` | `workspaceId -> source_vendor_refs.marketo_workspace_id`; workspace timezone stays advisory. |
| HubSpot Business Unit | `MarketingTenantScope` | `businessUnitId -> source_vendor_refs.hubspot_business_unit_id`; account owner maps to tenant admin principal. |
| Mailchimp Premium Audience | `MarketingAudienceScope` | `list_id -> source_vendor_refs.mailchimp_audience_id`; list-level merge fields become segment traits. |
| Iterable Project | `MarketingTenantScope` | `projectId -> source_vendor_refs.iterable_project_id`; API-key scope is not trusted as tenant scope. |
| Braze App Group | `MarketingTenantScope` | `app_group_id -> source_vendor_refs.braze_app_group_id`; apps become channel endpoints. |

## Workflow Steps

1. `ResolveTenant` rejects requests without active tenancy membership.
2. `NormalizeVendorRefs` validates exactly one id per vendor namespace and stores unknown ids in dry-run evidence only.
3. `EvaluateScopePermit` calls Cedar before persisting.
4. `CreateScopeRecord` writes `marketing_tenant_scope`.
5. `ProjectOntologyNode` emits `MarketingTenantScope`.
6. `SealAudit` emits scope creation evidence.

Decision branches: duplicate vendor ref returns `409 scope_conflict`; residency mismatch returns `403 residency_pack_denied`; missing purpose returns `422 consent_purpose_required`.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-MARKETING-SCOPE-CREATED` | `tenant_id`, `scope_id`, `purpose`, `source_vendor_refs_hash`, `cedar_decision_id` |
| `EVT-DATA-EGRESS` | Only when migration preview exports source ids to a vendor adapter. |
| `EVT-ERROR-MARKETING-SCOPE` | `tenant_id`, `vendor`, `error_code`, `recovery_branch` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Create tenant scope | 45 ms | 180 ms | 350 ms | 150 rps/cell | 99.95% |
| Read tenant scope | 12 ms | 60 ms | 120 ms | 1,500 rps/cell | 99.99% |

## Failure Modes + Recovery

- Vendor id collision: keep the first accepted scope immutable, return conflict evidence, and require `BindVendorScope` with auditor approval.
- Cedar deny: persist no row; emit refusal evidence with `cedar_decision_id`.
- Residency pack drift: mark scope `blocked_for_pack_rebind`, notify compliance, and block campaign execution until pack overlay is corrected.

## Migration Notes

Marketo partitions by workspace, HubSpot by business unit, Mailchimp Premium by audience, Iterable by project, and Braze by app group. Migration imports these as source references only; Oyatie tenancy remains the authority and no vendor permission model is copied into Cedar without explicit mapping in IP-002.

## Cross-µservice Handoffs

- `tenancy` owns tenant membership and tenant tree validation.
- `policy-engine` evaluates Cedar.
- `ontology` stores `MarketingTenantScope` projection.
- `audit-chain` seals `EVT-MARKETING-SCOPE-CREATED`.
- `data-boundary` labels campaign profile and consent signal classes.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/marketing-automation/IP-001-tenant-scope-kernel.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `EU-AI-ACT-2024-HIGH-RISK`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/marketing-automation/IP-001-tenant-scope-kernel.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/ARCHITECTURE.md`, `microservices/marketing-automation/PRD.md`, `microservices/marketing-automation/multi-region.md`, `microservices/marketing-automation/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/marketing-automation/IP-001-tenant-scope-kernel.md` matched [`attribution`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/marketing-automation/IP-001-tenant-scope-kernel.md`, `microservices/marketing-automation/manifest.json`, `microservices/marketing-automation/capacity-model.md`, `microservices/marketing-automation/compliance.md`, `microservices/marketing-automation/ARCHITECTURE.md`].
