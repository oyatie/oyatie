// api-gateway µservice — route-authorization fragment
// Cedar v4.2 LTS
// Authority: ADR-0157 + ADR-0243 (Cedar universal gate) + ADR-0294 (fragment soak).
// Owner: axis-network + ops-security.
// Default-deny baseline at tenant-scope.cedar; this fragment adds route-class permits.

// =============================================================================
// PERMIT 1 — public anonymous read on public-read routes
// =============================================================================

permit (
  principal,
  action == Action::"Route::read_public",
  resource in PublicRoute::?r
)
when {
  resource has visibility &&
  resource.visibility == "public" &&
  principal has bot_score &&
  principal.bot_score < 80
};

// =============================================================================
// PERMIT 2 — authenticated tenant reads tenant-scoped routes
// =============================================================================

permit (
  principal in TenantPrincipal::?p,
  action in [
    Action::"Route::read_tenant",
    Action::"Route::list_tenant"
  ],
  resource in TenantRoute::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has auth_class &&
  principal.auth_class in ["passkey", "webauthn", "totp", "session"]
};

// =============================================================================
// PERMIT 3 — authenticated tenant writes tenant-scoped routes
// =============================================================================

permit (
  principal in TenantPrincipal::?p,
  action in [
    Action::"Route::write_tenant",
    Action::"Route::create_tenant",
    Action::"Route::update_tenant",
    Action::"Route::delete_tenant"
  ],
  resource in TenantRoute::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has auth_class &&
  principal.auth_class in ["passkey", "webauthn"] &&
  principal has step_up_age_seconds &&
  principal.step_up_age_seconds < 900
};

// =============================================================================
// PERMIT 4 — partner tenant_class API consumption (paid)
// =============================================================================

permit (
  principal in PartnerPrincipal::?p,
  action == Action::"Route::partner_api",
  resource in PartnerRoute::?r
)
when {
  principal has tenant_class &&
  principal.tenant_class == "paid" &&
  principal has tos_accepted_version &&
  principal.tos_accepted_version >= resource.minimum_tos_version
};

// =============================================================================
// PERMIT 5 — admin / operator routes (step-up required)
// =============================================================================

permit (
  principal in AdminPrincipal::?p,
  action in [
    Action::"Route::admin_read",
    Action::"Route::admin_write"
  ],
  resource in AdminRoute::?r
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id &&
  principal has auth_class &&
  principal.auth_class == "passkey" &&
  principal has step_up_age_seconds &&
  principal.step_up_age_seconds < 300 &&
  principal has hardware_attested &&
  principal.hardware_attested == true
};

// =============================================================================
// PERMIT 6 — machine-to-machine (SPIFFE SVID required)
// =============================================================================

permit (
  principal in MachinePrincipal::?p,
  action in [
    Action::"Route::m2m_read",
    Action::"Route::m2m_write"
  ],
  resource in MachineRoute::?r
)
when {
  principal has spiffe_id &&
  principal.spiffe_id like "spiffe://oyatie.cell-*/*" &&
  principal has svid_age_seconds &&
  principal.svid_age_seconds < 3600
};

// =============================================================================
// FORBID — cross-tenant route access (defence in depth)
// =============================================================================

forbid (
  principal,
  action,
  resource
)
unless {
  resource has visibility &&
  resource.visibility == "public"
}
unless {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id == resource.tenant_id
};

// =============================================================================
// FORBID — admin route without step-up
// =============================================================================

forbid (
  principal,
  action == Action::"Route::admin_write",
  resource
)
unless {
  principal has step_up_age_seconds &&
  principal.step_up_age_seconds < 300
};

// =============================================================================
// FORBID — sov-cell-kr admin from non-KR origin
// =============================================================================

forbid (
  principal,
  action == Action::"Route::admin_write",
  resource in AdminRoute::?r
)
when {
  resource has cell_jurisdiction &&
  resource.cell_jurisdiction == "kr" &&
  principal has origin_country &&
  principal.origin_country != "KR"
};
