---
doc_class: ImplementationPlan
ip_id: IP-001-tenant-scope-kernel
microservice: learning-management
related_adrs: [ADR-0242, ADR-0244, ADR-0257, ADR-0263]
journey_id: J-LMS-01-tenant-learning-program-launch
status: proposed
date: 2026-05-20
owner: axis-learning-management
capability_tier: T2
---

# IP-001: Tenant Scope Kernel

## Context

This slice makes the learning tenant boundary explicit before catalog, enrollment, transcript, and credential workflows run. It supports Priya Nair, the enterprise learning administrator, launching a regulated onboarding program without leaking Cornerstone learning objects, Workday Learning campaigns, Docebo branches, 360Learning cohorts, or LinkedIn Learning Enterprise provider entitlements across tenants.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `learning_tenant_scope` | `scope_id` | `uuid primary key` | One tenant learning boundary. |
| `learning_tenant_scope` | `tenant_id` | `uuid not null` | Partition key used by every LMS table. |
| `learning_tenant_scope` | `learning_org_ref` | `text not null` | HRIS org, academy, or business-unit ref. |
| `learning_tenant_scope` | `provider_account_ref` | `text` | External provider account id. |
| `learning_tenant_scope` | `catalog_visibility` | `jsonb not null` | Audience, locale, region, and branch rules. |
| `learning_tenant_scope` | `transcript_retention_days` | `integer not null` | Transcript retention window. |
| `learning_tenant_scope` | `created_at` | `timestamptz not null` | Creation timestamp. |

## API Endpoints

REST `POST /v1/learning-management/tenant-scopes`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-lms00001",
  "learning_org_ref": "hris:org:global-sales",
  "provider_account_ref": "linkedin-learning:acct:ent-4421",
  "catalog_visibility": {
    "audiences": ["new-hire", "manager"],
    "regions": ["US", "CA"],
    "locales": ["en-US", "fr-CA"]
  },
  "transcript_retention_days": 2555
}
```

gRPC `LearningTenantScopeService.CreateScope(CreateLearningTenantScopeRequest)` returns `scope_id`, `effective_catalog_filter`, and `audit_event_id`.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"learning-admin"` | `learningManagement::CreateTenantScope` | `LearningTenantScope::*` | `tenant_id`, `learning_org_ref`, `catalog_visibility` |
| `Service::"catalog-sync"` | `learningManagement::ReadTenantScope` | `LearningTenantScope::*` | `tenant_id`, `provider_account_ref`, `source_vendor` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Cornerstone Division | `LearningTenantScope` | division id maps to `learning_org_ref`; OU rules map to `catalog_visibility`. |
| Workday Learning Tenant | `LearningTenantScope` | learning organization maps to `learning_org_ref`. |
| Docebo Branch | `LearningTenantScope` | branch code maps to org ref and locale rules. |
| 360Learning Group | `LearningAudience` | group membership projects into visibility audiences. |
| LinkedIn Learning Enterprise Account | `ProviderAccountEntitlement` | account id maps to `provider_account_ref`. |

## Workflow Steps

1. `ResolveLearningOrg` confirms HRIS org exists and is active.
2. `NormalizeProviderAccount` binds optional provider account to tenant.
3. `CompileCatalogVisibility` converts audience, region, and locale rules into an executable filter.
4. `EvaluateCedarCreate` denies cross-tenant or unauthorized scope creation.
5. `PersistTenantScope` writes the boundary and emits audit evidence.

Branches: missing HRIS org returns `422`; provider account already owned by another tenant returns `409`; empty audience set creates an admin-only catalog.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-LEARNING-TENANT-SCOPE-CREATED` | `tenant_id`, `scope_id`, `learning_org_ref`, `provider_account_ref` |
| `EVT-LEARNING-TENANT-SCOPE-DENIED` | `tenant_id`, `learning_org_ref`, `deny_reason` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Create tenant scope | 35 ms | 140 ms | 320 ms | 500 rps/cell | 99.95% |
| Read scope for catalog sync | 4 ms | 18 ms | 45 ms | 20k reads/s/cell | 99.99% |

## Failure Modes + Recovery

- HRIS org lookup outage: create remains pending and retries with idempotency key.
- Provider account conflict: reject and include owning tenant hash for support-only resolution.
- Visibility rule compile failure: keep scope inactive and emit failed compilation audit evidence.

## Migration Notes

Cornerstone OUs, Workday Learning organizations, Docebo branches, 360Learning groups, and LinkedIn Learning Enterprise account mappings import as inactive scopes until an administrator confirms audience visibility.

## Cross-µservice Handoffs

- `hris` validates org and worker audience references.
- `identity-access` supplies learning-admin principals.
- `audit-chain` seals scope creation and denial events.
- `data-residency` evaluates transcript retention constraints.
- `content-provider-integration` uses the scope during provider catalog sync.
