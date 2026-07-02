// FD-001 shared platform contracts — policy seed.
// Cedar semantics: deny-by-default, forbid-overrides-permit, order-independent
// (formally verified; arXiv 2403.04651). The structural invariant below is
// therefore unconditional: NO permit anywhere in the set — static, templated,
// or future — can grant across a tenant boundary.

// Structural cell/tenant isolation invariant. Mirrors
// src/tenancy.rs::check_resource_isolation on the policy plane: a principal
// may never act on another tenant's resource, whatever else is permitted.
@id("structural-tenant-isolation")
forbid (principal, action, resource)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id != resource.tenant_id
};

// Security-critical global gate, encoded as a FORBID so it cannot be bypassed.
// Reading a restricted-classified resource WITHOUT an asserted step-up class
// "a" is forbidden. Forbid-overrides-permit makes this unconditional: a
// per-tenant overlay `permit` (or any future permit) can NEVER grant a
// restricted read without step-up, because the union is permit-union and a
// deny-by-OMISSION gate (the abac-step-up-restricted-read permit alone) would
// otherwise be defeated by any overlay permit. The `unless` carries the sole
// exception: a principal that HAS step_up_class "a". (step_up_class is optional
// in the schema, so the `has` guard is load-bearing: a principal without the
// attribute does not satisfy the unless and the read stays forbidden.)
@id("forbid-restricted-read-without-step-up")
forbid (
  principal,
  action == OyaPlatform::Action::"ReadResource",
  resource
)
when { resource.data_class == "restricted" }
unless { principal has step_up_class && principal.step_up_class == "a" };

// RBAC example: group-based grant. Members of the tenant-admins group may
// administer their own tenant (the when-clause keeps the grant tenant-scoped;
// the structural forbid above backs it unconditionally).
@id("rbac-tenant-admin-group")
permit (
  principal in OyaPlatform::Group::"tenant-admins",
  action == OyaPlatform::Action::"AdministerTenant",
  resource
)
when { principal.tenant_id == resource.tenant_id };

// ABAC example: attribute condition. Reading a restricted-classified resource
// requires the principal to have asserted step-up class "a" (step_up_class is
// optional in the schema, so the `has` guard is load-bearing: workloads and
// principals without the attribute fall through to deny-by-default).
@id("abac-step-up-restricted-read")
permit (
  principal is OyaPlatform::Principal,
  action == OyaPlatform::Action::"ReadResource",
  resource
)
when {
  resource.data_class == "restricted" &&
  principal has step_up_class &&
  principal.step_up_class == "a"
};
