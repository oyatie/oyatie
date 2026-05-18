forbid(
  principal,
  action == Action::"createRuntimeScope",
  resource
) when {
  resource.microservice == "connect"
};

permit(
  principal,
  action == Action::"readRetirementStatus",
  resource
) when {
  principal.tenant_id == resource.tenant_id
};
