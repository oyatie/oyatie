// identity µservice — ACR step-up policy fragment per ADR-0189
// Authority: ADR-0189 step-up authentication
// Imported by consumer µservices that gate sensitive actions on ACR floor.

// =============================================================================
// Routine actions (read-mostly).
// =============================================================================

permit (
  principal,
  action in [
    IdentityAction::"GetSelf",
    IdentityAction::"ListOwnCredentials",
    IdentityAction::"ScimListUsers",
    IdentityAction::"ScimGetUser"
  ],
  resource
) when {
  principal.acr_level >= AcrLevel::"routine"
};

// =============================================================================
// Elevated actions (mutate own data).
// =============================================================================

permit (
  principal,
  action in [
    IdentityAction::"RegisterPasskey",
    IdentityAction::"UpdateOwnProfile",
    IdentityAction::"ScimCreateUser",
    IdentityAction::"ScimPatchUser",
    IdentityAction::"ScimCreateGroup",
    IdentityAction::"ScimPatchGroup"
  ],
  resource
) when {
  principal.acr_level >= AcrLevel::"elevated"
};

// =============================================================================
// Sensitive actions (admin / secret-touching).
// =============================================================================

permit (
  principal,
  action in [
    IdentityAction::"DeletePasskey",
    IdentityAction::"ScimDeleteUser",
    IdentityAction::"ScimDeleteGroup",
    IdentityAction::"BindFederation",
    IdentityAction::"UnbindFederation",
    IdentityAction::"AdminPinUserAcr",
    IdentityAction::"AdminRevokeSession"
  ],
  resource
) when {
  principal.acr_level >= AcrLevel::"sensitive"
} unless {
  // Sensitive ACR session-age max 1h per ADR-0189.
  context.now - principal.acr_event_at > 3600
};

// =============================================================================
// Critical actions (key rotation / tenant deletion / billing currency).
// =============================================================================

permit (
  principal,
  action in [
    IdentityAction::"RotateJwks",
    IdentityAction::"DeleteTenant",
    IdentityAction::"OperatorRecoveryRevokeCredential",
    IdentityAction::"OperatorRecoveryForceReauth",
    IdentityAction::"AuditExportAll"
  ],
  resource
) when {
  principal.acr_level >= AcrLevel::"critical" &&
  context.it_approval_token_present &&
  context.it_approval_token_bound_to_resource &&
  context.now - context.it_approval_token_issued_at <= 300
} unless {
  // Critical ACR session-age max 15min.
  context.now - principal.acr_event_at > 900
};
