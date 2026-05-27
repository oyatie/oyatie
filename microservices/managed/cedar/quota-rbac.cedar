// Cedar RBAC policies for managed-K8s tenant quota.
// ADR-0376 / ADR-0007 / ADR-0183.
// These are the reference policies; production uses CedarWorkloadAuthorizer
// compiled from structured Policy objects in adapter-cedar.

// TenantAdmin: write own-tenant quota only.
@id("quota-write-tenant-admin")
permit (
  principal is Workload,
  action == Action::"quota:Write",
  resource is QuotaRecord
)
when {
  principal.scopes.contains("quota:write")
};

// TenantViewer + TenantAdmin: read own-tenant quota only.
@id("quota-read-tenant")
permit (
  principal is Workload,
  action == Action::"quota:Read",
  resource is QuotaRecord
)
when {
  principal.scopes.contains("quota:read")
};

// PlatformOperator: write any tenant's quota (ceiling management).
@id("quota-write-platform-operator")
permit (
  principal is Workload,
  action == Action::"quota:Write",
  resource is QuotaRecord
)
when {
  principal.scopes.contains("quota:platform:write")
};
