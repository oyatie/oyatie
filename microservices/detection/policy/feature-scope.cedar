// Detection Cedar policy: feature-scope
// Binding ADRs: ADR-0243, ADR-0307, ADR-0308, ADR-0263.

forbid (principal, action, resource)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

permit (
  principal in Detection::Role::"operator",
  action == Detection::Action::"feature-store",
  resource in Detection::Resource::"tenantScoped"
)
when {
  principal.tenant_id == resource.tenant_id &&
  context has trace_id &&
  context has audit_id
};
