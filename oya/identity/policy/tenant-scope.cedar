// identity µservice — tenant-scope Cedar policy
// Authority: ADR-0183 (Cedar app-authz), ADR-0162 (per-tenant audit slicing)
// Owner:   ops-security + axis-identity
// Audit:   every deny emits oya_identity_tenant_unauthorized_attempt_total
// Sweep:   imports microservices/governance/policy/cedar-canonical-imports.cedar

// =============================================================================
// Default deny — every action refused unless an explicit permit matches.
// =============================================================================

forbid (principal, action, resource);

// =============================================================================
// Per-tenant scope: principal can read/mutate ONLY resources in their tenant.
// =============================================================================

permit (
  principal,
  action in [
    IdentityAction::"GetSelf",
    IdentityAction::"ListOwnCredentials",
    IdentityAction::"RegisterPasskey",
    IdentityAction::"AuthenticatePasskey"
  ],
  resource is User
) when {
  principal.tenant_id == resource.tenant_id &&
  principal.id == resource.id
};

// =============================================================================
// Tenant admin scope: tenant-admin role can act on any user in the same tenant.
// =============================================================================

permit (
  principal,
  action in [
    IdentityAction::"ListUsers",
    IdentityAction::"GetUser",
    IdentityAction::"PatchUser",
    IdentityAction::"SuspendUser"
  ],
  resource is User
) when {
  principal.tenant_id == resource.tenant_id &&
  principal.role == "tenant-admin"
} unless {
  // Cross-tenant principal-resource is structurally impossible due to tenant_id check;
  // this unless-clause is a belt-and-braces for any policy authored upstream that might
  // bypass the when-clause due to a typo.
  principal.tenant_id != resource.tenant_id
};

// =============================================================================
// Operator scope: an ops-on-call principal with `acr=critical` may act on any user
// for the operator-mediated recovery flow.
// =============================================================================

permit (
  principal,
  action in [
    IdentityAction::"OperatorRecoveryRevokeCredential",
    IdentityAction::"OperatorRecoveryPinAcr",
    IdentityAction::"OperatorRecoveryForceReauth"
  ],
  resource is User
) when {
  principal.role == "ops-security" &&
  principal.acr_level == AcrLevel::"critical" &&
  context.operator_ticket_id != ""
};

// =============================================================================
// Cross-tenant SCIM bearer refusal: SCIM bearer A cannot read/mutate tenant B.
// =============================================================================

forbid (
  principal,
  action in [
    IdentityAction::"ScimListUsers",
    IdentityAction::"ScimGetUser",
    IdentityAction::"ScimCreateUser",
    IdentityAction::"ScimPatchUser",
    IdentityAction::"ScimDeleteUser",
    IdentityAction::"ScimListGroups",
    IdentityAction::"ScimGetGroup",
    IdentityAction::"ScimCreateGroup",
    IdentityAction::"ScimPatchGroup",
    IdentityAction::"ScimDeleteGroup"
  ],
  resource
) when {
  principal.scim_bearer_tenant_id != resource.tenant_id
};
