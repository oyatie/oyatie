// ADR-0218 Tenant Admin Console: tenant admins manage tenant-local work contexts only.
permit(
  principal,
  action in [
    Action::"CreateTenantPolicyDraft",
    Action::"SimulateTenantPolicyDraft",
    Action::"ApplyTenantPolicyDraft",
    Action::"ReviewJitAccess"
  ],
  resource
)
when {
  principal.role == "tenant-admin" &&
  principal.tenant_id == resource.tenant_id &&
  principal.active_context_id == resource.context_id &&
  resource.context_type == "work"
};

forbid(
  principal,
  action,
  resource
)
when {
  resource.context_type == "personal"
};

forbid(
  principal,
  Action::"ApplyTenantPolicyDraft",
  resource
)
when {
  resource.simulation_result == "refused"
};
