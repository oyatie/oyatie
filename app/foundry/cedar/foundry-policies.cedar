// Foundry V1 authorization posture: deny-by-default, one real grant,
// one structural invariant. cedar-policy strict-validates this file in
// foundry-spine's cedar_policy_gate tests; a permit accompanies every
// forbid — a bare forbid nullifies its own file.

// Operators of a tenant may invoke Actions on the operations console at
// assist autonomy or below, within their own tenant.
permit(
  principal is Principal in Role::"foundry-operator",
  action == Action::"InvokeAction",
  resource is OntologyObject
) when {
  context.surface == "ops-console" &&
  context.autonomy_tier <= 1 &&
  principal.tenant == resource.tenant
};

// Structural tenant isolation: whatever else ever permits, an
// invocation never crosses tenants.
forbid(
  principal is Principal,
  action == Action::"InvokeAction",
  resource is OntologyObject
) when {
  principal.tenant != resource.tenant
};
