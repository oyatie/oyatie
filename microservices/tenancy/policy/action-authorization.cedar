// Cedar v4.2 LTS — tenancy µservice action-authorization (default-deny baseline)
// Binding ADR: ADR-0243 (Cedar universal gate) + ADR-0244 (tenant scoping)
// Soak: ≥60s per ADR-0294
// x-fragment-soak-seconds: 60

// Tenant admins may manage their own tenant lifecycle (suspend / unsuspend / rename / etc.).
permit (
    principal in oyatie::Tenancy::TenantAdminGroup::"tenant-admins",
    action in [
        Action::"rename_tenant",
        Action::"set_residency",
        Action::"manage_membership",
        Action::"set_quota",
        Action::"request_dr_pair_promotion",
        Action::"complete_kyb_kyc"
    ],
    resource
) when {
    principal.tenant_id == resource.tenant_id
    && principal.audience_type == "B2B_TENANT_ADMIN"
};

// Substrate principals: lifecycle / cell-assignment / DSR.
permit (
    principal in [
        oyatie::Tenancy::Principal::"lifecycle-controller",
        oyatie::Tenancy::Principal::"cell-assignment-controller",
        oyatie::Tenancy::Principal::"isolation-policy-emitter",
        oyatie::Tenancy::Principal::"dsr-cascade-runner",
        oyatie::Tenancy::Principal::"kyb-kyc-verifier"
    ],
    action in [
        Action::"create_tenant",
        Action::"suspend_tenant",
        Action::"delete_tenant",
        Action::"assign_cell",
        Action::"emit_rls_policy",
        Action::"cascade_dsr",
        Action::"verify_kyb_kyc"
    ],
    resource
) when {
    principal.spiffe_attested == true
};

// Platform-owner reserved-namespace: only platform-owner principals may create/delete in the
// reserved namespace per ADR-0244 + ADR-0284.
forbid (
    principal,
    action in [Action::"create_tenant", Action::"rename_tenant"],
    resource
) when {
    resource.tenant_slug in oyatie::Tenancy::ReservedNamespace::"all"
    && !(principal.tenant_id == oyatie::Tenancy::Tenant::"oyatie-platform-owner")
};

// Cross-tenant defence-in-depth: forbid any principal acting on a tenant not theirs.
forbid (principal, action, resource) when {
    principal.tenant_id != resource.tenant_id
    && principal.principal_kind != "workload"
};
