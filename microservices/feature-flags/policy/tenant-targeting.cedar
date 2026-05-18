permit(
  principal,
  action == Action::"evaluateFlag",
  resource
) when {
  principal.tenant_id == resource.tenant_id &&
  resource.flag_state != "archived"
};
