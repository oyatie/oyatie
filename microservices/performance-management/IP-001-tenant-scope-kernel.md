---
doc_class: ImplementationPlan
ip_id: IP-001-tenant-scope-kernel
microservice: performance-management
related_adrs: [ADR-0242, ADR-0243, ADR-0244, ADR-0257, ADR-0263, ADR-0321]
journey_id: J-PM-01-tenant-review-cycle-launch
status: proposed
date: 2026-05-20
owner: axis-performance-management
capability_tier: T2
---

# IP-001: Performance Management Tenant Scope Kernel

## Context

This slice creates the tenant boundary for goals, review cycles, feedback, calibration, and engagement data. Nadia Singh is the named persona activating a review cycle while avoiding vendor lock-in to Lattice workspace, Culture Amp account, Workday Talent tenant, or 15Five company boundaries.

## Data Model Deltas

| Table | Column | Type | Notes |
|---|---|---|---|
| `performance_tenant_scope` | `scope_id` | `uuid primary key` | Stable performance scope. |
| `performance_tenant_scope` | `tenant_id` | `uuid not null` | Tenant partition. |
| `performance_tenant_scope` | `worker_population_ref` | `text not null` | Population from HRIS/identity. |
| `performance_tenant_scope` | `review_locale_policy` | `jsonb not null` | Locale and works-council constraints. |
| `performance_tenant_scope` | `source_vendor_refs` | `jsonb not null default '{}'` | Lattice, Culture Amp, Workday Talent, 15Five source ids. |
| `performance_tenant_scope` | `labor_pack_id` | `text not null` | EU-worker-council, US-labor, KR-labor overlay. |
| `performance_tenant_scope` | `created_at` | `timestamptz not null` | HLC timestamp. |

## API Endpoints

REST `POST /v1/performance-management/scopes`

```json
{
  "tenant_id": "018f8ad2-0e0f-7ad2-92d2-pm000001",
  "worker_population_ref": "hris-population:engineering-emea",
  "labor_pack_id": "EU-worker-council",
  "review_locale_policy": {"default_locale": "en", "required_locales": ["en", "de"]},
  "source_vendor_refs": {
    "lattice_workspace_id": "lat_ws_17",
    "culture_amp_account_id": "ca_903",
    "workday_talent_tenant": "wd_tenant_44",
    "fifteenfive_company_id": "ff_51"
  }
}
```

gRPC `PerformanceScopeService.CreateScope(CreatePerformanceScopeRequest)` returns `scope_id` and labor-pack validation evidence.

## Cedar Policy Hooks

| Principal | Action | Resource | Context |
|---|---|---|---|
| `User::"hrbp"` | `performanceManagement::CreateScope` | `PerformanceScope::*` | `tenant_id`, `worker_population_ref`, `labor_pack_id` |
| `Service::"migration-worker"` | `performanceManagement::BindVendorScope` | `PerformanceScope::*` | `source_vendor`, `migration_batch_id`, `dry_run` |
| `User::"auditor"` | `performanceManagement::ReadScope` | `PerformanceScope::*` | `read_reason`, `ticket_id`, `pack_id` |

## Ontology Projection

| Vendor object | Oyatie object | Field deltas |
|---|---|---|
| Lattice Workspace | `PerformanceTenantScope` | workspace id maps to source refs; departments map through worker population. |
| Culture Amp Account | `PerformanceTenantScope` | account id maps to source refs; survey programs project separately. |
| Workday Talent Tenant | `PerformanceTenantScope` | tenant id maps to source refs; worker ids remain HRIS refs. |
| 15Five Company | `PerformanceTenantScope` | company id maps to source refs; roles are not copied to Cedar. |

## Workflow Steps

1. `ResolveTenant` verifies tenant and HR pack activation.
2. `ResolveWorkerPopulation` checks HRIS population ownership.
3. `NormalizeVendorRefs` validates source ids.
4. `EvaluateScopePermit` calls Cedar.
5. `PersistScope` writes `performance_tenant_scope`.
6. `ProjectScope` writes ontology node and audit event.

Branches: works-council pack missing denies launch; vendor id conflict returns `409`; unknown worker population returns `422`.

## Audit Events

| Event class | Payload fields |
|---|---|
| `EVT-PERFORMANCE-SCOPE-CREATED` | `tenant_id`, `scope_id`, `worker_population_ref`, `labor_pack_id`, `cedar_decision_id` |
| `EVT-DATA-EGRESS` | Emitted when vendor migration preview exports worker ids. |
| `EVT-ERROR-PERFORMANCE-SCOPE` | `source_vendor`, `error_code`, `recovery_branch` |

## SLO Targets

| Operation | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| Create performance scope | 55 ms | 210 ms | 420 ms | 100 rps/cell | 99.95% |
| Read performance scope | 15 ms | 70 ms | 140 ms | 1,500 rps/cell | 99.99% |

## Failure Modes + Recovery

- HRIS population unavailable: create no scope and return retryable dependency error.
- Labor pack mismatch: block launch until pack owner approves policy.
- Vendor scope collision: keep source ref in migration quarantine for manual mapping.

## Migration Notes

Lattice, Culture Amp, Workday Talent, and 15Five use different tenant and employee identifiers. Migration must bind all source ids to Oyatie worker population refs and never treat vendor workspace membership as tenant membership.

## Cross-µservice Handoffs

- `tenancy` validates tenant membership.
- `identity` resolves employee and manager principals.
- `hris` supplies worker population refs.
- `policy-engine` evaluates Cedar.
- `audit-chain` seals scope events.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/performance-management/IP-001-tenant-scope-kernel.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/performance-management/IP-001-tenant-scope-kernel.md`, `microservices/performance-management/manifest.json`, `microservices/performance-management/ARCHITECTURE.md`, `microservices/performance-management/PRD.md`, `microservices/performance-management/multi-region.md`, `microservices/performance-management/capacity-model.md`].
