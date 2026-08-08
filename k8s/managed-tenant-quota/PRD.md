# Managed K8s Tenant Quota — PRD

## Purpose

Per-tenant quota and RBAC enforcement for managed Kubernetes clusters.
This service exposes the `QuotaDecisionPort` that cluster-lifecycle calls
before provisioning any tenant cluster. ADR-0376.

## Acceptance Criteria

- `evaluate()` returns `Allow` when usage + request is within quota ceilings.
- `evaluate()` returns `Deny(reason)` when any dimension is exceeded.
- Cedar RBAC: TenantAdmin can write/read their own quota; cross-tenant access denied.
- PlatformOperator can set plan ceilings for any tenant.
- REST API: `PUT /tenants/{id}/quota`, `GET /tenants/{id}/quota`, `GET /tenants/{id}/usage`.
- Billing emission: typed `Unimplemented::BillingEmission` until wave follow-on.
- Audit chain emission: typed `Unimplemented::AuditChainEmission` until wave follow-on.
