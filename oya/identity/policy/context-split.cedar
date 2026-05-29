// ADR-0215 context split: tenant admins cannot see or switch into personal contexts.
permit(
  principal,
  Action::"ResolvePrincipalContext",
  resource
)
when {
  principal.id == resource.principal_id &&
  principal.allowed_contexts.contains(resource.context_id)
};

forbid(
  principal,
  Action::"ResolvePrincipalContext",
  resource
)
when {
  principal.role == "tenant-admin" &&
  resource.context_type == "personal"
};

forbid(
  principal,
  Action::"SwitchPrincipalContext",
  resource
)
when {
  !principal.allowed_contexts.contains(resource.context_id)
};
