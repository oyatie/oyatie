# Managed K8s Tenant Quota — Threat Model

## ADR-0376 Threat Model

### Quota Bypass
**Risk**: A tenant requests more clusters/nodes/vCPU/RAM than their quota allows.
**Mitigation**: `evaluate()` in the kernel checks all dimensions; deny-by-default
when no quota record exists (`QuotaPortError::NotFound` → HTTP 404, not allow).

### RBAC Escalation
**Risk**: A principal grants themselves a higher role (TenantAdmin → PlatformOperator).
**Mitigation**: Cedar default-deny; roles are assigned by the platform, not self-served.
No `permit` exists for self-role-grant. Forbid-wins semantics prevent escalation.

### Cross-Tenant Limit Read
**Risk**: A tenant reads or alters another tenant's quota or usage records.
**Mitigation**: Cedar `tenant_id` condition on all read/write policies. The kernel's
`evaluate()` short-circuits with `Deny(TenantMismatch)` if tenant IDs differ.
HTTP handlers validate `path.tenant_id == body.tenant_id`.
