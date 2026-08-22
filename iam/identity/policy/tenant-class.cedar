// identity µservice — tenant_class Cedar policy fragment
// Authority: ADR-0329 (tenant_class retirement) + ADR-0330 (tenant_class model) +
//            ADR-0331 §D-4 (canonical fragment shape) + ADR-0243 (Cedar gate) +
//            ADR-0244 (tenant scoping).
// Wave: 15A-IDENTITY-FIX authored 2026-05-21.
//
// identity is THE keystone µservice for tenant_class principal-claim emission:
// every other µservice in the corpus reads `principal.tenant_class` and
// `principal.billing_components` from claims emitted by identity (the OIDC
// issuer) + cloud-iam (the principal-issuance authority that reads from
// cloud-billing). The fragments below gate identity's own behavior against
// those claims and forbid retired capability-adoption predicates.
//
// Audit: every deny emits identity_tenant_class_unauthorized_attempt_total
// with labels {action, tenant_class, deny_reason}.
// Sweep: imports microservices/governance/policy/cedar-canonical-imports.cedar
// Allowlist: this fragment contains no demo_trial/paid with per_seat billing_component/paid with per_usage billing_component/paid with compliance_pack gating strings.

// =============================================================================
// (1) Forbid paid-only operations for demo_trial principals.
//     Maps to ADR-0331 §D-4 template clause (1) +
//     ADR-0330 §B.3.6 (compliance pack ban) +
//     ADR-0330 §B.3.7 (BYOK ban) +
//     ADR-0330 §B.3.8 (marketplace seller ban) +
//     ADR-0249 (marketplace purchase gating).
// =============================================================================

forbid (
  principal,
  action in [
    IdentityAction::"ActivateCompliancePack",
    IdentityAction::"EnableProviderByok",
    IdentityAction::"BindExternalIdpFederation",
    IdentityAction::"PublishMarketplaceListing",
    IdentityAction::"PurchaseMarketplaceListing"
  ],
  resource
) when {
  principal.tenant_class == "demo_trial"
};

// =============================================================================
// (2) Permit paid operations that require a specific billing component.
//     Maps to ADR-0331 §D-4 template clause (2).
//     Identity emits per_usage meters (e.g., identity.oidc.token_issued)
//     therefore per_usage-gated metered operations must check the principal's
//     billing_components set.
// =============================================================================

permit (
  principal,
  action in [
    IdentityAction::"IssueMeteredOidcToken",
    IdentityAction::"RegisterMeteredWebauthnCredential",
    IdentityAction::"ProvisionMeteredScimUser"
  ],
  resource
) when {
  principal.tenant_class == "paid" &&
  principal.billing_components.contains("per_usage")
};

permit (
  principal,
  action == IdentityAction::"AllocateNewSeat",
  resource is User
) when {
  principal.tenant_class == "paid" &&
  principal.billing_components.contains("per_seat") &&
  context.active_seat_count < context.contracted_seat_ceiling
};

// =============================================================================
// (3) Forbid demo_trial principals that exceed the cap shape.
//     Caps load from microservices/identity/capabilities/tenant-class-caps.yaml
//     and reach Cedar via the per-µservice context resolver
//     (ADR-0243 §D-1).
// =============================================================================

forbid (
  principal,
  action in [
    IdentityAction::"RegisterPasskey",
    IdentityAction::"ProvisionScimUser",
    IdentityAction::"IssueOidcToken"
  ],
  resource
) when {
  principal.tenant_class == "demo_trial" &&
  (
    context.current_user_count       >= context.demo_trial_max_users_per_tenant ||
    context.current_passkey_count    >= context.demo_trial_max_passkeys_per_user ||
    context.current_session_count    >= context.demo_trial_max_active_sessions ||
    context.scim_ops_in_window       >= context.demo_trial_scim_ops_per_minute ||
    context.oidc_token_issues_today  >= context.demo_trial_oidc_token_issues_per_day
  )
};

// =============================================================================
// (4) Forbid demo_trial principals after the time expiry.
//     Maps to ADR-0330 §B.3.4 (default 30-day window) +
//     ADR-0331 §D-4 template clause (4).
// =============================================================================

forbid (
  principal,
  action,
  resource
) when {
  principal.tenant_class == "demo_trial" &&
  principal has demo_trial_expires_at &&
  context.request_time > principal.demo_trial_expires_at &&
  // Grace window from ADR-0330 §B.3.11 (default 7 days, read-only during grace).
  !(
    context.request_time < principal.demo_trial_grace_expires_at &&
    action in [IdentityAction::"GetSelf", IdentityAction::"ListOwnCredentials"]
  )
};

// =============================================================================
// (5) Forbid downgrade from paid back to demo_trial per ADR-0330 §B.1.3.
//     Tenant_class transitions are one-way demo_trial → paid.
// =============================================================================

forbid (
  principal,
  action == IdentityAction::"ChangeTenantClass",
  resource
) when {
  principal.tenant_class == "paid" &&
  context.target_tenant_class == "demo_trial"
};

// =============================================================================
// (6) Cross-tenant principal-claim isolation: a principal in tenant A may
//     never receive a tenant_class claim describing tenant B.
//     This is the identity-owned half of ADR-0244 §D-5 cross-tenant deny.
// =============================================================================

forbid (
  principal,
  action == IdentityAction::"EmitTenantClassClaim",
  resource is Tenant
) when {
  principal.tenant_id != resource.tenant_id
};

// =============================================================================
// (7) Forbid step-up to acr=critical for demo_trial principals.
//     ACR critical is reserved for paid tenants under JIT IT-approval
//     (ADR-identity-005); demo_trial best-effort posture cannot grant it.
// =============================================================================

forbid (
  principal,
  action == IdentityAction::"GrantStepUpAcr",
  resource
) when {
  principal.tenant_class == "demo_trial" &&
  context.requested_acr_level == "critical"
};

// =============================================================================
// (8) Cap-breached demo_trial principals lose write paths but retain reads.
//     Aligns with ADR-0330 §B.3.11 grace-window behavior.
// =============================================================================

forbid (
  principal,
  action in [
    IdentityAction::"RegisterPasskey",
    IdentityAction::"ProvisionScimUser",
    IdentityAction::"PatchUser",
    IdentityAction::"BindExternalIdp",
    IdentityAction::"GrantStepUpAcr"
  ],
  resource
) when {
  principal.tenant_class == "demo_trial" &&
  principal.cap_breached == true
};

// =============================================================================
// (9) Foundry principals (oyatie.foundry.*) carry tenant_class = paid per
//     ADR-0247 §D-2 and operate under billing_components = {} (system tenant).
//     Permit the Foundry to issue claims for any tenant; deny attempts to
//     spoof tenant_class on a non-Foundry principal.
// =============================================================================

permit (
  principal is FoundryPrincipal,
  action == IdentityAction::"IssueTenantClassClaim",
  resource is Tenant
) when {
  principal.tenant_id == "oyatie.foundry" &&
  principal.tenant_class == "paid"
};

forbid (
  principal,
  action == IdentityAction::"IssueTenantClassClaim",
  resource
) unless {
  principal is FoundryPrincipal ||
  (principal.role == "tenant-admin" && principal.tenant_id == resource.tenant_id)
};
