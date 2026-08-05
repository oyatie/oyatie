# Managed K8s Cluster Lifecycle — Tenant Isolation

## Isolation guarantees (ADR-0376 / ADR-0007)

1. **Gateway tenant principal**: Cluster-lifecycle accepts tenant identity only from
   the upstream gateway-injected principal. Caller-supplied tenant context is rejected
   when it is missing, malformed, or inconsistent with the authenticated request.

2. **Quota-before-provisioning dependency**: The service calls the
   `managed-k8s-tenant-quota` dependency through `QuotaDecisionPort` before any
   control-plane host invocation. Deny, not-found, and persistence/unavailable
   decisions are hard admission failures.

3. **No quota administration surface**: Cluster-lifecycle does not own tenant-quota
   storage, read/write APIs, quota RBAC administration, or quota mutation semantics.
   Any quota-service behavior named here is a dependency contract, not source
   authority for the cluster-lifecycle service.

4. **Tenant-scoped provisioning request**: Only after quota allow does
   cluster-lifecycle map the tenant-scoped cluster request to
   `managed-k8s-control-plane-host`. The tenant principal is preserved on the
   provisioning request and no cross-tenant cluster handle is returned.

5. **Deterministic foundation ceiling**: Current claims are limited to the
   dogfood/design deterministic foundation for create admission. Live provider
   actions, public GA, billing readiness, and measured SLO compliance remain
   out of scope until follow-on evidence exists.
