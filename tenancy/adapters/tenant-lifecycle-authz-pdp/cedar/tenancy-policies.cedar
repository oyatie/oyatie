// AUTH-005 / ADR-0564 D7 — tenancy tenant-lifecycle authorization policy seed.
//
// Cedar semantics: deny-by-default, forbid-overrides-permit, order-independent
// (formally verified; arXiv 2403.04651). The structural cross-tenant forbid is
// therefore unconditional: NO permit anywhere in the set — static, templated,
// or future — can grant a tenant operator authority over another tenant.

// Structural tenant-isolation invariant: a principal carrying a tenant scope
// may never administer a tenant other than its own, whatever else is permitted.
@id("structural-tenant-isolation")
forbid (principal, action, resource)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id != resource.tenant_id
};

// PER-TENANT operator grant: a principal whose proven tenant scope equals the
// target tenant may administer it (read/provision/suspend/resume/retire). The
// `has tenant_id` guard is load-bearing — a platform-only principal (no tenant
// scope) falls through to deny-by-default on per-tenant ops, and the structural
// forbid above neutralizes any cross-tenant attempt.
@id("tenant-operator-administer")
permit (
  principal is OyaTenancy::Principal,
  action == OyaTenancy::Action::"AdministerTenant",
  resource is OyaTenancy::Tenant
)
when {
  principal has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// PLATFORM-ADMIN grant for register: only a principal asserting the
// platform-admin axis may register a tenant on the control plane.
@id("platform-admin-register")
permit (
  principal is OyaTenancy::Principal,
  action == OyaTenancy::Action::"RegisterTenant",
  resource is OyaTenancy::TenancyControlPlane
)
when { principal.platform_admin == true };

// PLATFORM-ADMIN grant for list: enumerating all tenants discloses the whole
// control plane and likewise requires the platform-admin axis.
@id("platform-admin-list")
permit (
  principal is OyaTenancy::Principal,
  action == OyaTenancy::Action::"ListTenants",
  resource is OyaTenancy::TenancyControlPlane
)
when { principal.platform_admin == true };
