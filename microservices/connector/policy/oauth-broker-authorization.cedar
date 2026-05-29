// connector service — oauth-broker-authorization policy fragment
// Cedar v4.2 LTS
//
// Purpose: gate OAuth grant creation / revocation / rotation; enforce step-up
//          auth on TAB provider-credential BYOK provisioning (ADR-0255 §D-4); enforce per-tenant scope.
// Owner:   axis-integration + ops-security
// References: ARCHITECTURE.md §principals; threat-model.md T-S-02/T-T-02/T-E-02
// soak-window-seconds: 60
// publish-stage: active

// =============================================================================
// Default deny
// =============================================================================

forbid (principal, action, resource);

// =============================================================================
// PERMIT 1 — TIE initiates OAuth grant for their tenant
// =============================================================================

permit (
  principal in TenantPrincipal::?p,
  action == Action::"oauth_grant_initiate",
  resource in OAuthClient::?c
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  resource.tenant_id == principal.tenant_id &&
  principal has role &&
  principal.role in ["tenant_integration_engineer", "tenant_admin"]
};

// =============================================================================
// PERMIT 2 — TAB provisions provider-credential BYOK OAuth client (ADR-0255 §D-4; step-up auth required)
// =============================================================================

permit (
  principal in TenantPrincipal::?p,
  action == Action::"oauth_client_provision",
  resource in OAuthClient::?c
)
when {
  principal has role &&
  principal.role == "tenant_admin" &&
  context has step_up_auth_class &&
  context.step_up_auth_class in ["webauthn_passkey", "fido2_security_key"] &&
  context has step_up_auth_age_seconds &&
  context.step_up_auth_age_seconds <= 300
};

// =============================================================================
// PERMIT 3 — TAB revokes OAuth grant (step-up auth required)
// =============================================================================

permit (
  principal in TenantPrincipal::?p,
  action == Action::"oauth_grant_revoke",
  resource in OAuthGrant::?g
)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  resource.tenant_id == principal.tenant_id &&
  principal has role &&
  principal.role == "tenant_admin" &&
  context has step_up_auth_class &&
  context.step_up_auth_class in ["webauthn_passkey", "fido2_security_key"]
};

// =============================================================================
// PERMIT 4 — broker worker rotates expired refresh tokens automatically
// =============================================================================

permit (
  principal == Service::"oyatie.connector.oauth-broker-api",
  action == Action::"oauth_token_refresh",
  resource in OAuthGrant::?g
)
when {
  resource has status &&
  resource.status == "ACTIVE" &&
  resource has expires_at &&
  context has now &&
  context.now >= resource.expires_at - 300
};

// =============================================================================
// FORBID — redirect_uri not in allow-list
// =============================================================================

forbid (
  principal,
  action == Action::"oauth_grant_initiate",
  resource in OAuthClient::?c
)
when {
  resource has allowed_redirect_uris &&
  context has redirect_uri &&
  !(context.redirect_uri in resource.allowed_redirect_uris)
};

// =============================================================================
// FORBID — state nonce invalid or expired
// =============================================================================

forbid (
  principal,
  action == Action::"oauth_grant_callback",
  resource
)
when {
  context has state_valid &&
  context.state_valid != true
};
