// application µservice — tenant-scope policy fragment
// Cedar v4 schema; conforms to microservices/application/policy/schema.cedarschema
//
// Purpose: enforce per-tenant route + module-access scope for shell-routing,
//          module-loader, and admin-portal requests.
// Owner:   ops-security + axis-application
// Audit:   every deny emits oya_application_route_denied_total + oya_application_module_load_denied_total
// References: route-isolation.md §"Invariants"
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// Default deny — every route + module access is refused unless explicit permit
// =============================================================================

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — employee renders tenant-scoped route
// =============================================================================
//
// A TenantUser bound to a tenant may render any Route in the same tenant
// whose required_roles intersect the user's role-set.

permit (
  principal in TenantUser::?u,
  action == Action::"render_route",
  resource in Route::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has roles &&
  resource has required_roles &&
  (principal.roles.containsAny(resource.required_roles))
};

// =============================================================================
// PERMIT 2 — tenant admin renders admin route with WebAuthn step-up
// =============================================================================

permit (
  principal in TenantUser::?u,
  action == Action::"render_route",
  resource in Route::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  "tenant-admin" in principal.roles &&
  resource has admin_scope &&
  resource.admin_scope == true &&
  principal has mfa_factor &&
  principal.mfa_factor == "webauthn"
};

// =============================================================================
// PERMIT 3 — employee loads product module (tenant-enabled + tenant-scoped)
// =============================================================================

permit (
  principal in TenantUser::?u,
  action == Action::"load_module",
  resource in Module::?m
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  resource has enabled &&
  resource.enabled == true &&
  resource has required_roles &&
  (principal.roles.containsAny(resource.required_roles))
};

// =============================================================================
// PERMIT 4 — pack residency match
// =============================================================================
//
// All permits above are conjuncted with this pack-residency check at the
// usecase layer. Cedar fragment for completeness.

permit (
  principal,
  action,
  resource
)
when {
  principal has pack &&
  resource has pack_residency &&
  (resource.pack_residency == principal.pack ||
   resource.pack_residency == "any")
};

// =============================================================================
// FORBID — explicit cross-tenant deny (defence in depth)
// =============================================================================

forbid (
  principal,
  action in [
    Action::"render_route",
    Action::"load_module",
    Action::"read_audit"
  ],
  resource
)
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// FORBID — never render route flagged SECRET-class without operator JIT
// =============================================================================

forbid (
  principal,
  action,
  resource
)
when {
  resource has data_class &&
  resource.data_class == "SECRET"
}
unless {
  principal has jit_token &&
  principal.jit_token.valid_until > context.now &&
  "oyatie-operator" in principal.roles
};

// =============================================================================
// FORBID — refuse PHI route to user without HIPAA training attestation
// =============================================================================

forbid (
  principal,
  action == Action::"render_route",
  resource in Route::?r
)
when {
  resource has data_class &&
  resource.data_class == "SENSITIVE_HIPAA_PHI"
}
unless {
  principal has hipaa_attestation &&
  principal.hipaa_attestation == true
};
