permit(
  principal,
  action == Action::"admitEdgeRequest",
  resource
) when {
  principal.tenant_id == resource.tenant_id &&
  principal.cell_id == resource.cell_id
};
