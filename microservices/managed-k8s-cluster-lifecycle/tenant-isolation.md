# Managed K8s Tenant Quota — Tenant Isolation

## Isolation Guarantees (ADR-0376 / ADR-0007)

1. **Quota record isolation**: Each `TenantQuota` is keyed by `TenantId`. No API
   returns quota data for a tenant other than the one in the authenticated request.

2. **Cedar default-deny**: All RBAC policies require the principal's `tenant_id`
   scope to match the target resource. A principal without a matching scope is denied.

3. **evaluate() cross-tenant guard**: The kernel function short-circuits with
   `Deny(TenantMismatch)` if `quota.tenant_id != request.tenant_id`.

4. **HTTP path/body consistency**: `PUT /tenants/{id}/quota` rejects requests where
   `path.id != body.tenant_id` with HTTP 400.

5. **No shared mutable state**: Each tenant's quota and usage records are stored
   under separate keys; no cross-tenant aggregation is performed.
