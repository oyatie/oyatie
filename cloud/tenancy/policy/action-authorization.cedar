// Cedar v4.2 LTS — tenancy µservice action-authorization (default-deny baseline)
// Binding ADR: ADR-0243 (Cedar universal gate) + ADR-0244 (tenant scoping)
// Soak: ≥60s per ADR-0294
// x-fragment-soak-seconds: 60

// Tenant admins may manage scoped tenant resources only inside their own tenant.
permit (
    principal in oyatie::Tenancy::TenantAdminGroup::"tenant-admins",
    action in [
        Action::"rename_tenant",
        Action::"set_residency",
        Action::"set_quota",
        Action::"request_dr_pair_promotion"
    ],
    resource is Tenant
) when {
    principal.tenant_id == resource.tenant_id
    && principal.audience_type == "B2B_TENANT_ADMIN"
};

permit (
    principal in oyatie::Tenancy::TenantAdminGroup::"tenant-admins",
    action == Action::"manage_membership",
    resource is TenantMembership
) when {
    principal.tenant_id == resource.tenant_id
    && principal.audience_type == "B2B_TENANT_ADMIN"
};

permit (
    principal in oyatie::Tenancy::TenantAdminGroup::"tenant-admins",
    action == Action::"complete_kyb_kyc",
    resource is KybKycCase
) when {
    principal.tenant_id == resource.tenant_id
    && principal.audience_type == "B2B_TENANT_ADMIN"
};

// Substrate principals are least-privilege: each controller gets only its own job/action pair.
permit (
    principal == oyatie::Tenancy::Principal::"lifecycle-controller",
    action in [
        Action::"create_tenant",
        Action::"suspend_tenant",
        Action::"delete_tenant"
    ],
    resource is TenantLifecycleJob
) when {
    principal.spiffe_attested == true &&
    resource.microservice == "tenancy"
};

permit (
    principal == oyatie::Tenancy::Principal::"cell-assignment-controller",
    action == Action::"assign_cell",
    resource is CellAssignmentJob
) when {
    principal.spiffe_attested == true &&
    resource.microservice == "tenancy"
};

permit (
    principal == oyatie::Tenancy::Principal::"isolation-policy-emitter",
    action == Action::"emit_rls_policy",
    resource is RlsPolicyEmissionJob
) when {
    principal.spiffe_attested == true &&
    resource.microservice == "tenancy"
};

permit (
    principal == oyatie::Tenancy::Principal::"dsr-cascade-runner",
    action == Action::"cascade_dsr",
    resource is DsrCascadeJob
) when {
    principal.spiffe_attested == true &&
    resource.microservice == "tenancy"
};

permit (
    principal == oyatie::Tenancy::Principal::"kyb-kyc-verifier",
    action == Action::"verify_kyb_kyc",
    resource is KybKycCase
) when {
    principal.spiffe_attested == true &&
    resource.microservice == "tenancy"
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
