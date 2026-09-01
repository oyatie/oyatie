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

// Operators of a tenant may read that tenant's ontology from the
// operations console. Reads carry no autonomy tier: looking is not
// acting, and a read-only operator must still be able to open the
// capability the shell renders for them.
permit(
  principal is Principal in Role::"foundry-operator",
  action == Action::"foundry.ontology.use",
  resource is OntologyObject
) when {
  context.surface == "ops-console" &&
  principal.tenant == resource.tenant
};

// Structural tenant isolation: whatever else ever permits, no action
// crosses tenants. Deliberately UNSCOPED by action — an action added to
// the vocabulary tomorrow sits inside this belt on the day it is born,
// instead of depending on someone remembering to widen a list.
forbid(
  principal is Principal,
  action,
  resource is OntologyObject
) when {
  principal.tenant != resource.tenant
};
